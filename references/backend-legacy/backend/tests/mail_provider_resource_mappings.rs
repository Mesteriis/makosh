use makosh_backend_testkit::context::TestContext;
use makosh_communications_api::accounts::{CommunicationProviderKind, NewProviderAccount};
use makosh_communications_api::mail_resources::{
    DiscoveredMailProviderResource, MailProviderResourceCommandPort, MailProviderResourceKind,
    MailProviderSemanticRole,
};
use makosh_communications_postgres::provider_store::CommunicationProviderAccountStore;
use makosh_hub_backend::domains::communications::folders::{
    CommunicationFolderStore, NewCommunicationFolder,
};
use makosh_hub_backend::domains::communications::provider_resources::{
    MailProviderResourceMappingUpdate, MailProviderResourceStore, NewMailProviderResource,
};
use serde_json::json;

#[tokio::test]
async fn manual_provider_resource_mapping_wins_over_later_discovery() {
    let context = TestContext::new().await;
    let pool = context.pool().clone();
    let account_id = "mail-provider-resource-mapping";
    CommunicationProviderAccountStore::new(pool.clone())
        .upsert(&NewProviderAccount::new(
            account_id,
            CommunicationProviderKind::Imap,
            "Provider mapping fixture",
            "mapping@example.test",
        ))
        .await
        .expect("provider account");
    let store = MailProviderResourceStore::new(pool);

    let first = store
        .upsert_discovered(
            &NewMailProviderResource::new(
                account_id,
                MailProviderResourceKind::Folder,
                "Sent Messages",
                "Sent Messages",
            )
            .semantic_role(MailProviderSemanticRole::Sent),
        )
        .await
        .expect("first discovered Sent folder");
    let second = store
        .upsert_discovered(
            &NewMailProviderResource::new(
                account_id,
                MailProviderResourceKind::Folder,
                "Sent",
                "Sent",
            )
            .semantic_role(MailProviderSemanticRole::Sent),
        )
        .await
        .expect("replacement discovered Sent folder");
    assert_eq!(
        store
            .resource(&first.mapping_id)
            .await
            .expect("first resource lookup")
            .expect("first resource")
            .semantic_role,
        None
    );
    assert_eq!(second.semantic_role, Some(MailProviderSemanticRole::Sent));

    let manually_mapped = store
        .set_manual_mapping(
            &first.mapping_id,
            &MailProviderResourceMappingUpdate {
                semantic_role: Some(MailProviderSemanticRole::Sent),
                local_folder_id: None,
            },
        )
        .await
        .expect("manual mapping")
        .expect("mapped resource");
    assert_eq!(manually_mapped.mapping_source, "manual");

    let rediscovered = store
        .upsert_discovered(
            &NewMailProviderResource::new(
                account_id,
                MailProviderResourceKind::Folder,
                "Sent",
                "Sent Mail",
            )
            .semantic_role(MailProviderSemanticRole::Sent),
        )
        .await
        .expect("rediscovered Sent folder");
    assert_eq!(rediscovered.semantic_role, None);
    assert_eq!(
        store
            .resource_for_role(
                account_id,
                MailProviderResourceKind::Folder,
                MailProviderSemanticRole::Sent,
            )
            .await
            .expect("role lookup")
            .expect("manual role resource")
            .mapping_id,
        first.mapping_id
    );
}

#[tokio::test]
async fn provider_discovery_port_persists_provider_neutral_resource_facts() {
    let context = TestContext::new().await;
    let pool = context.pool().clone();
    let account_id = "mail-provider-resource-discovery-port";
    CommunicationProviderAccountStore::new(pool.clone())
        .upsert(&NewProviderAccount::new(
            account_id,
            CommunicationProviderKind::Gmail,
            "Discovery port fixture",
            "discovery@example.test",
        ))
        .await
        .expect("provider account");
    let store = MailProviderResourceStore::new(pool);

    MailProviderResourceCommandPort::record_discovered_resources(
        &store,
        account_id,
        &[DiscoveredMailProviderResource {
            resource_kind: MailProviderResourceKind::Label,
            provider_resource_id: "Label_42".to_owned(),
            display_name: "Follow up".to_owned(),
            semantic_role: Some(MailProviderSemanticRole::User),
            selectable: true,
            writable: true,
            capabilities: json!({ "gmail_label_type": "user" }),
        }],
    )
    .await
    .expect("record provider discovery");

    let resources = store
        .list_for_account(account_id)
        .await
        .expect("list discovered resources");
    assert!(resources.iter().any(|resource| {
        resource.provider_resource_id == "Label_42"
            && resource.semantic_role == Some(MailProviderSemanticRole::User)
            && resource.mapping_source == "discovered"
    }));
}

#[tokio::test]
async fn provider_resource_lookup_resolves_label_name_and_local_folder_mapping() {
    let context = TestContext::new().await;
    let pool = context.pool().clone();
    let account_id = "mail-provider-resource-lookup";
    CommunicationProviderAccountStore::new(pool.clone())
        .upsert(&NewProviderAccount::new(
            account_id,
            CommunicationProviderKind::Imap,
            "Lookup fixture",
            "lookup@example.test",
        ))
        .await
        .expect("provider account");
    let store = MailProviderResourceStore::new(pool.clone());
    store
        .upsert_discovered(
            &NewMailProviderResource::new(
                account_id,
                MailProviderResourceKind::Label,
                "Label_42",
                "Follow up",
            )
            .semantic_role(MailProviderSemanticRole::User),
        )
        .await
        .expect("discovered label");
    let folder = CommunicationFolderStore::new(pool)
        .create(NewCommunicationFolder {
            folder_id: Some("folder-provider-mapped".to_owned()),
            account_id: Some(account_id.to_owned()),
            name: "Projects".to_owned(),
            description: None,
            color: None,
            sort_order: None,
        })
        .await
        .expect("local folder");
    store
        .upsert_discovered(&NewMailProviderResource {
            account_id: account_id.to_owned(),
            resource_kind: MailProviderResourceKind::Folder,
            provider_resource_id: "Projects/2026".to_owned(),
            display_name: "Projects".to_owned(),
            semantic_role: None,
            local_folder_id: Some(folder.folder_id.clone()),
            selectable: true,
            writable: true,
            capabilities: json!({}),
            observed_at: chrono::Utc::now(),
        })
        .await
        .expect("discovered folder");

    assert_eq!(
        store
            .resource_for_display_name(account_id, MailProviderResourceKind::Label, " follow UP ")
            .await
            .expect("label lookup")
            .expect("provider label")
            .provider_resource_id,
        "Label_42"
    );
    assert_eq!(
        store
            .resource_for_local_folder(account_id, &folder.folder_id)
            .await
            .expect("folder lookup")
            .expect("provider folder")
            .provider_resource_id,
        "Projects/2026"
    );
}
