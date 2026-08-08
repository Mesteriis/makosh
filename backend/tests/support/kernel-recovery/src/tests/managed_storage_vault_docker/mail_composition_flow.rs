//! Managed Gateway conformance for Mail-owned composition state.

use makosh_mail_api::{
    MailClientRequestV1, MailClientResponseV1,
    client_contract::MailClientContractV1,
    composition::{
        MailCompositionCommandV1, MailCompositionModeV1, MailCompositionQueryResponseV1,
        MailCompositionQueryV1, MailDraftInputV1, MailSignatureInputV1, MailTemplateInputV1,
        MailTemplateVariableValueV1,
    },
};
use makosh_mail_runtime::client_port::{
    MailClientPortErrorV1, decode_module_response, encode_module_request,
};
use makosh_runtime_protocol::v1::ModuleClientResponseV1;
use prost::Message;

use super::*;
use crate::modules::capability::router::{
    ManagedCapabilityRouteRequest, route_managed_client_request,
};

pub(super) fn assert_mail_composition(
    store: &SqliteControlStore,
    supervisor: &ManagedRuntimeSupervisor,
    mail: &StartedMailRuntime,
) {
    let template = MailTemplateInputV1 {
        connection_id: MAIL_ACCOUNT_ID.to_owned(),
        template_id: "managed-template-1".to_owned(),
        name: "Managed greeting".to_owned(),
        subject_template: "Hello {{name}}".to_owned(),
        text_body_template: "Hi {{name}}, {{message}}".to_owned(),
        variables: vec!["name".to_owned(), "message".to_owned()],
        locale: Some("en".to_owned()),
    };
    let template_receipt = mutate(
        store,
        supervisor,
        mail,
        191,
        MailCompositionCommandV1::UpsertTemplate {
            operation_id: "managed-mail-template-create".to_owned(),
            template,
            expected_revision: None,
        },
    );
    assert_eq!(template_receipt.revision, 1);

    let signature = MailSignatureInputV1 {
        connection_id: MAIL_ACCOUNT_ID.to_owned(),
        signature_id: "managed-signature-1".to_owned(),
        name: "Default signature".to_owned(),
        text_body: "Regards,\nOwner".to_owned(),
        is_default: true,
    };
    let signature_receipt = mutate(
        store,
        supervisor,
        mail,
        192,
        MailCompositionCommandV1::UpsertSignature {
            operation_id: "managed-mail-signature-create".to_owned(),
            signature,
            expected_revision: None,
        },
    );
    assert_eq!(signature_receipt.revision, 1);
    assert_default_signature_switch_is_atomic(store, supervisor, mail);

    let first_draft = draft("managed-draft-1", "First draft");
    let first_receipt = mutate(
        store,
        supervisor,
        mail,
        193,
        MailCompositionCommandV1::UpsertDraft {
            operation_id: "managed-mail-draft-create".to_owned(),
            draft: first_draft.clone(),
            expected_revision: None,
        },
    );
    assert_eq!(first_receipt.revision, 1);
    let duplicate = mutate(
        store,
        supervisor,
        mail,
        194,
        MailCompositionCommandV1::UpsertDraft {
            operation_id: "managed-mail-draft-create".to_owned(),
            draft: first_draft,
            expected_revision: None,
        },
    );
    assert_eq!(duplicate, first_receipt);

    let second_receipt = mutate(
        store,
        supervisor,
        mail,
        195,
        MailCompositionCommandV1::UpsertDraft {
            operation_id: "managed-mail-draft-create-2".to_owned(),
            draft: draft("managed-draft-2", "Second draft"),
            expected_revision: None,
        },
    );
    assert_eq!(second_receipt.revision, 1);

    let preview = query(
        store,
        supervisor,
        mail,
        196,
        MailCompositionQueryV1::PreviewTemplate {
            connection_id: MAIL_ACCOUNT_ID.to_owned(),
            template_id: "managed-template-1".to_owned(),
            values: vec![
                MailTemplateVariableValueV1 {
                    name: "name".to_owned(),
                    value: "Alice".to_owned(),
                },
                MailTemplateVariableValueV1 {
                    name: "message".to_owned(),
                    value: "The managed route is live.".to_owned(),
                },
            ],
        },
    );
    let MailCompositionQueryResponseV1::TemplatePreview(preview) = preview else {
        panic!("Mail composition preview returned the wrong response")
    };
    assert!(preview.ready);
    assert_eq!(preview.subject, "Hello Alice");
    assert_eq!(preview.text_body, "Hi Alice, The managed route is live.");

    let drafts = query(
        store,
        supervisor,
        mail,
        197,
        MailCompositionQueryV1::ListDrafts {
            connection_id: MAIL_ACCOUNT_ID.to_owned(),
            cursor: None,
            limit: 1,
        },
    );
    let MailCompositionQueryResponseV1::Drafts(drafts) = drafts else {
        panic!("Mail composition draft list returned the wrong response")
    };
    assert_eq!(drafts.items.len(), 1);
    let cursor = drafts.next_cursor.expect("bounded draft cursor");
    assert_wrong_scope_cursor_is_rejected(store, supervisor, mail, cursor);
    assert_cross_account_query_is_rejected(store, supervisor, mail);
    assert_conflicting_operation_is_rejected(store, supervisor, mail);
    assert_stale_revision_is_rejected(store, supervisor, mail);
    assert_runtime_active(supervisor, mail);
}

pub(super) fn assert_mail_composition_survives_restart(
    store: &SqliteControlStore,
    supervisor: &ManagedRuntimeSupervisor,
    mail: &StartedMailRuntime,
) {
    let last_failure = supervisor
        .last_failure(&mail.registration_id)
        .expect("observe Mail failure after managed restart");
    assert!(
        supervisor
            .is_active(&mail.registration_id)
            .expect("observe Mail after managed restart"),
        "Mail must remain active after managed restart; last_failure={last_failure:?}",
    );
    assert!(
        last_failure.is_none(),
        "Mail managed restart recorded a process failure: {last_failure:?}",
    );
    for (request_id, query_value) in [
        (
            202,
            MailCompositionQueryV1::GetDraft {
                connection_id: MAIL_ACCOUNT_ID.to_owned(),
                draft_id: "managed-draft-1".to_owned(),
            },
        ),
        (
            203,
            MailCompositionQueryV1::GetTemplate {
                connection_id: MAIL_ACCOUNT_ID.to_owned(),
                template_id: "managed-template-1".to_owned(),
            },
        ),
        (
            204,
            MailCompositionQueryV1::GetSignature {
                connection_id: MAIL_ACCOUNT_ID.to_owned(),
                signature_id: "managed-signature-1".to_owned(),
            },
        ),
    ] {
        let response = query(store, supervisor, mail, request_id, query_value);
        assert!(
            matches!(
                response,
                MailCompositionQueryResponseV1::Draft(_)
                    | MailCompositionQueryResponseV1::Template(_)
                    | MailCompositionQueryResponseV1::Signature(_)
            ),
            "Mail composition entity must survive a managed runtime restart",
        );
    }
}

fn draft(draft_id: &str, subject: &str) -> MailDraftInputV1 {
    MailDraftInputV1 {
        connection_id: MAIL_ACCOUNT_ID.to_owned(),
        draft_id: draft_id.to_owned(),
        mode: MailCompositionModeV1::Reply,
        provider_conversation_id: Some("managed-provider-thread".to_owned()),
        in_reply_to_provider_message_id: Some("managed-provider-message".to_owned()),
        to_recipients: vec!["recipient@example.test".to_owned()],
        cc_recipients: Vec::new(),
        bcc_recipients: Vec::new(),
        subject: subject.to_owned(),
        text_body: "Managed composition body".to_owned(),
        template_id: Some("managed-template-1".to_owned()),
        signature_id: Some("managed-signature-1".to_owned()),
    }
}

fn mutate(
    store: &SqliteControlStore,
    supervisor: &ManagedRuntimeSupervisor,
    mail: &StartedMailRuntime,
    request_id: u64,
    command: MailCompositionCommandV1,
) -> makosh_mail_api::composition::MailCompositionMutationReceiptV1 {
    let request = encode_module_request(
        request_id,
        &MailClientRequestV1::CompositionCommand(command),
    )
    .expect("encode Mail composition command");
    let bytes = route(
        store,
        supervisor,
        mail,
        MailClientContractV1::CompositionCommand,
        &request,
    )
    .expect("route Mail composition command");
    let (actual_request_id, response) =
        decode_module_response(MailClientContractV1::CompositionCommand, &bytes)
            .expect("decode Mail composition command");
    assert_eq!(actual_request_id, request_id);
    let MailClientResponseV1::CompositionMutation(receipt) = response else {
        panic!("Mail composition command returned the wrong response")
    };
    receipt
}

fn query(
    store: &SqliteControlStore,
    supervisor: &ManagedRuntimeSupervisor,
    mail: &StartedMailRuntime,
    request_id: u64,
    query: MailCompositionQueryV1,
) -> MailCompositionQueryResponseV1 {
    let request = encode_module_request(request_id, &MailClientRequestV1::CompositionQuery(query))
        .expect("encode Mail composition query");
    let bytes = route(
        store,
        supervisor,
        mail,
        MailClientContractV1::CompositionQuery,
        &request,
    )
    .expect("route Mail composition query");
    let (actual_request_id, response) = decode_module_response(
        MailClientContractV1::CompositionQuery,
        &bytes,
    )
    .unwrap_or_else(|error| {
        let error_code = ModuleClientResponseV1::decode(bytes.as_slice())
            .map(|response| response.error_code)
            .unwrap_or_else(|_| "INVALID_ENVELOPE".to_owned());
        panic!("decode Mail composition query request {request_id}: {error:?} ({error_code})")
    });
    assert_eq!(actual_request_id, request_id);
    let MailClientResponseV1::CompositionQuery(response) = response else {
        panic!("Mail composition query returned the wrong response")
    };
    response
}

fn assert_wrong_scope_cursor_is_rejected(
    store: &SqliteControlStore,
    supervisor: &ManagedRuntimeSupervisor,
    mail: &StartedMailRuntime,
    cursor: String,
) {
    assert_rejected(
        store,
        supervisor,
        mail,
        198,
        MailClientContractV1::CompositionQuery,
        MailClientRequestV1::CompositionQuery(MailCompositionQueryV1::ListTemplates {
            connection_id: MAIL_ACCOUNT_ID.to_owned(),
            cursor: Some(cursor),
            limit: 1,
        }),
    );
}

fn assert_cross_account_query_is_rejected(
    store: &SqliteControlStore,
    supervisor: &ManagedRuntimeSupervisor,
    mail: &StartedMailRuntime,
) {
    assert_rejected(
        store,
        supervisor,
        mail,
        199,
        MailClientContractV1::CompositionQuery,
        MailClientRequestV1::CompositionQuery(MailCompositionQueryV1::ListDrafts {
            connection_id: "another-account".to_owned(),
            cursor: None,
            limit: 1,
        }),
    );
}

fn assert_conflicting_operation_is_rejected(
    store: &SqliteControlStore,
    supervisor: &ManagedRuntimeSupervisor,
    mail: &StartedMailRuntime,
) {
    assert_rejected(
        store,
        supervisor,
        mail,
        200,
        MailClientContractV1::CompositionCommand,
        MailClientRequestV1::CompositionCommand(MailCompositionCommandV1::UpsertDraft {
            operation_id: "managed-mail-draft-create".to_owned(),
            draft: draft("managed-draft-conflict", "Conflicting payload"),
            expected_revision: None,
        }),
    );
}

fn assert_stale_revision_is_rejected(
    store: &SqliteControlStore,
    supervisor: &ManagedRuntimeSupervisor,
    mail: &StartedMailRuntime,
) {
    assert_rejected(
        store,
        supervisor,
        mail,
        201,
        MailClientContractV1::CompositionCommand,
        MailClientRequestV1::CompositionCommand(MailCompositionCommandV1::UpsertDraft {
            operation_id: "managed-mail-draft-stale".to_owned(),
            draft: draft("managed-draft-1", "Stale update"),
            expected_revision: Some(9),
        }),
    );
}

fn assert_default_signature_switch_is_atomic(
    store: &SqliteControlStore,
    supervisor: &ManagedRuntimeSupervisor,
    mail: &StartedMailRuntime,
) {
    let receipt = mutate(
        store,
        supervisor,
        mail,
        205,
        MailCompositionCommandV1::UpsertSignature {
            operation_id: "managed-mail-signature-default-switch".to_owned(),
            signature: MailSignatureInputV1 {
                connection_id: MAIL_ACCOUNT_ID.to_owned(),
                signature_id: "managed-signature-2".to_owned(),
                name: "New default signature".to_owned(),
                text_body: "Best,\nOwner".to_owned(),
                is_default: true,
            },
            expected_revision: None,
        },
    );
    assert_eq!(receipt.revision, 1);
    let response = query(
        store,
        supervisor,
        mail,
        206,
        MailCompositionQueryV1::ListSignatures {
            connection_id: MAIL_ACCOUNT_ID.to_owned(),
            cursor: None,
            limit: 100,
        },
    );
    let MailCompositionQueryResponseV1::Signatures(page) = response else {
        panic!("Mail composition signatures returned the wrong response")
    };
    let defaults = page
        .items
        .iter()
        .filter(|signature| signature.is_default)
        .collect::<Vec<_>>();
    assert_eq!(defaults.len(), 1);
    assert_eq!(defaults[0].signature_id, "managed-signature-2");
    let previous = page
        .items
        .iter()
        .find(|signature| signature.signature_id == "managed-signature-1")
        .expect("previous default signature");
    assert!(!previous.is_default);
    assert_eq!(previous.revision, 2);
}

fn assert_rejected(
    store: &SqliteControlStore,
    supervisor: &ManagedRuntimeSupervisor,
    mail: &StartedMailRuntime,
    request_id: u64,
    contract: MailClientContractV1,
    request: MailClientRequestV1,
) {
    let request = encode_module_request(request_id, &request).expect("encode rejected request");
    let bytes = route(store, supervisor, mail, contract, &request).expect("route rejected request");
    assert_eq!(
        decode_module_response(contract, &bytes),
        Err(MailClientPortErrorV1::Runtime)
    );
    assert_runtime_active(supervisor, mail);
}

fn route(
    store: &SqliteControlStore,
    supervisor: &ManagedRuntimeSupervisor,
    mail: &StartedMailRuntime,
    contract: MailClientContractV1,
    request: &[u8],
) -> Result<Vec<u8>, String> {
    let route = ManagedCapabilityRouteRequest::new(
        &mail.registration_id,
        &mail.runtime_instance_id,
        mail.runtime_generation,
        mail.grant_epoch,
        contract.capability_id(),
        request,
    );
    route_managed_client_request(store, &supervisor.relay_port(), &route)
}

fn assert_runtime_active(supervisor: &ManagedRuntimeSupervisor, mail: &StartedMailRuntime) {
    assert!(
        supervisor
            .is_active(&mail.registration_id)
            .expect("observe Mail after rejected composition request"),
        "rejected Mail composition request must not terminate the managed runtime"
    );
}
