//! Exact admission, storage, Vault and release assembly for managed Mail conformance.

use super::*;

use makosh_contacts_mail_sync_source_api::CONTACT_MAIL_SYNC_SOURCE_BLOB_TARGET_CAPABILITY_ID_V1;
use makosh_mail_address_book_contract::MAIL_ADDRESS_BOOK_CAPABILITY_ID_V1;
use makosh_mail_api::{
    MailCredentialPurpose,
    account::{MailBindCredentialRequestV1, MailCredentialPurposeV1},
    client_contract::{MAIL_MODULE_ID, MAIL_OWNER_ID, MailClientContractV1},
};
use makosh_mail_persistence::GmailOAuthCredentialBindingV1;
use makosh_mail_runtime::{
    admission::{
        MAIL_ATTACHMENT_ANCHOR_CONSUME_CAPABILITY_ID,
        MAIL_ATTACHMENT_BLOB_ADMISSION_PUBLISH_CAPABILITY_ID,
        MAIL_ATTACHMENT_SAFETY_STATE_CONSUME_CAPABILITY_ID,
        MAIL_ATTACHMENT_SCAN_CANDIDATE_PUBLISH_CAPABILITY_ID, MAIL_BLOB_CAPABILITY_ID,
        MAIL_COMMUNICATION_OBSERVED_PUBLISH_CAPABILITY_ID, MAIL_CREDENTIAL_LEASE_TTL_SECONDS,
        MAIL_DELIVERY_INTENT_TARGET_CAPABILITY_ID_V1,
        MAIL_GMAIL_CREDENTIAL_LIFECYCLE_CAPABILITY_ID, MAIL_GMAIL_CREDENTIALS_CAPABILITY_ID,
        MAIL_GMAIL_OAUTH_REFRESH_CREDENTIALS_CAPABILITY_ID,
        MAIL_GMAIL_OAUTH_SETUP_CREDENTIALS_CAPABILITY_ID,
        MAIL_GMAIL_REFRESH_CREDENTIAL_LIFECYCLE_CAPABILITY_ID,
        MAIL_IMAP_CREDENTIAL_LIFECYCLE_CAPABILITY_ID, MAIL_IMAP_CREDENTIALS_CAPABILITY_ID,
        MAIL_RETAINED_EVIDENCE_REPLAY_CAPABILITY_ID, MAIL_SMTP_CREDENTIAL_LIFECYCLE_CAPABILITY_ID,
        MAIL_SMTP_CREDENTIALS_CAPABILITY_ID, MAIL_STORAGE_CAPABILITY_ID, mail_module_descriptor_v1,
    },
    settings::mail_settings_schema_bytes_v2,
    storage_bundle::mail_runtime_storage_bundle_v1,
};
use makosh_vault_key_provider::WrappingKeyProvider;
use makosh_vault_key_provider_file::FileWrappingKeyProvider;
use makosh_vault_protocol::{SecretClassV1, VaultActionV1, VaultPurposeRequestV1};
use makosh_vault_store_sqlcipher::{SecretRecordId, SecretRecordScope, VaultStore};

const MAIL_RELEASE_ARTIFACT_ID: &str = "integration.mail";
pub(super) const MAIL_ACCOUNT_ID: &str = "mail-account-1";

pub(super) struct AdmittedMailRuntime {
    registration_id: String,
    capability_ids: Vec<String>,
}

pub(super) struct StartedMailRuntime {
    pub(super) registration_id: String,
    pub(super) runtime_instance_id: String,
    pub(super) runtime_generation: u64,
    pub(super) grant_epoch: u64,
    capability_ids: Vec<String>,
}

pub(super) struct MailSmtpFixtureSettingsV1 {
    pub(super) port: u16,
    pub(super) ca_certificate_pem: String,
}

pub(super) struct MailGmailFixtureSettingsV1 {
    pub(super) port: u16,
    pub(super) ca_certificate_pem: String,
    pub(super) oauth: Option<MailGmailOAuthFixtureSettingsV1>,
}

pub(super) struct MailGmailOAuthFixtureSettingsV1 {
    pub(super) host: String,
    pub(super) port: u16,
    pub(super) ca_certificate_pem: String,
}

pub(super) struct MailCardDavFixtureSettingsV1 {
    pub(super) imap_port: u16,
    pub(super) carddav_port: u16,
    pub(super) ca_certificate_pem: String,
}

pub(super) struct SeededGmailCredentialBindingV1 {
    imap_password_record_id: SecretRecordId,
    smtp_password_record_id: SecretRecordId,
    access_token_record_id: [u8; 16],
    refresh_credential_record_id: [u8; 16],
}

impl SeededGmailCredentialBindingV1 {
    pub(super) fn binding(&self) -> GmailOAuthCredentialBindingV1 {
        GmailOAuthCredentialBindingV1 {
            access_token_record_id: self.access_token_record_id,
            access_token_revision: 1,
            refresh_credential_record_id: self.refresh_credential_record_id,
            refresh_credential_revision: 1,
            access_token_expires_at_unix_seconds: i64::MAX,
            scope_sha256: Sha256::digest(b"managed-mail-gmail-delivery-scope").into(),
            permanent_delete_authorized: false,
            contacts_write_authorized: false,
        }
    }

    pub(super) fn contacts_binding(&self) -> GmailOAuthCredentialBindingV1 {
        GmailOAuthCredentialBindingV1 {
            contacts_write_authorized: true,
            ..self.binding()
        }
    }
}

#[derive(Clone, Copy)]
enum MailAdmissionProfileV1 {
    ImapSync,
    AccountCredentialLifecycle,
    SmtpDelivery,
    SmtpAttachmentDelivery,
    GmailDelivery,
    GmailAttachmentDelivery,
    GmailOAuth,
    GooglePeopleAddressBook,
    CardDavAddressBook,
}

enum MailSettingsProfileV1 {
    Imap {
        port: u16,
        smtp: Option<MailSmtpFixtureSettingsV1>,
    },
    Gmail(MailGmailFixtureSettingsV1),
    GooglePeopleAddressBook(MailGmailFixtureSettingsV1),
    CardDavAddressBook {
        imap_port: u16,
        carddav_port: u16,
        ca_certificate_pem: String,
    },
}

pub(super) fn installed_communications_mail_release(root: &Path) -> InstalledSignedBundle {
    let mut artifacts = communications_release_artifacts();
    artifacts.push(mail_release_artifact());
    InstalledSignedBundle::install(root, &artifacts)
        .expect("install signed Communications and Mail release")
}

pub(super) fn mail_release_artifact() -> SignedRuntimeArtifact {
    SignedRuntimeArtifact::new(
        MAIL_RELEASE_ARTIFACT_ID,
        mail_binary(),
        mail_module_descriptor_v1("managed-mail-live").encode_to_vec(),
    )
    .with_settings_schema(mail_settings_schema_bytes_v2())
}

pub(super) fn seed_mail_vault(vault_dir: &Path) -> SeededGmailCredentialBindingV1 {
    let key = FileWrappingKeyProvider::new(&vault_dir.join("platform-wrapping-key.bin"))
        .load_or_create()
        .expect("open Vault wrapping key");
    let store = VaultStore::open(
        &vault_dir.join("vault.db"),
        &vault_dir.join("vault.anchor"),
        &key,
    )
    .expect("open initialized Vault");
    fn store_basic_secret(
        store: &VaultStore,
        purpose: MailCredentialPurpose,
        secret: &[u8],
    ) -> SecretRecordId {
        let request = VaultPurposeRequestV1::new(
            purpose.as_str().to_owned(),
            MAIL_ACCOUNT_ID.to_owned(),
            vec![SecretClassV1::ProviderCredential],
            vec![
                VaultActionV1::Resolve,
                VaultActionV1::Retire,
                VaultActionV1::Delete,
            ],
            MAIL_CREDENTIAL_LEASE_TTL_SECONDS,
        )
        .expect("Mail credential purpose");
        let scope = SecretRecordScope::new(
            MAIL_OWNER_ID.to_owned(),
            &request,
            SecretClassV1::ProviderCredential,
            1,
        )
        .expect("Mail secret scope");
        store
            .store_secret(&scope, secret)
            .expect("store Mail test credential")
    }
    let imap_password_record_id = store_basic_secret(
        &store,
        MailCredentialPurpose::ImapPassword,
        b"managed-mail-imap-password",
    );
    let smtp_password_record_id = store_basic_secret(
        &store,
        MailCredentialPurpose::SmtpPassword,
        b"managed-mail-smtp-password",
    );
    let access_token_record_id = store_mail_test_secret(
        &store,
        MailCredentialPurpose::GmailAccessToken,
        SecretClassV1::ProviderCredential,
        b"managed-mail-gmail-access-token",
    );
    let refresh_credential_record_id = store_mail_test_secret(
        &store,
        MailCredentialPurpose::GmailRefreshCredential,
        SecretClassV1::OAuthRefreshCredential,
        b"managed-mail-gmail-refresh-credential",
    );
    let _carddav_password_record_id = store_basic_secret(
        &store,
        MailCredentialPurpose::IcloudCardDavPassword,
        b"managed-mail-carddav-password",
    );
    SeededGmailCredentialBindingV1 {
        imap_password_record_id,
        smtp_password_record_id,
        access_token_record_id,
        refresh_credential_record_id,
    }
}

pub(super) fn rotate_basic_mail_vault(vault_dir: &Path, seeded: &SeededGmailCredentialBindingV1) {
    let key = FileWrappingKeyProvider::new(&vault_dir.join("platform-wrapping-key.bin"))
        .load_or_create()
        .expect("open Vault wrapping key");
    let store = VaultStore::open(
        &vault_dir.join("vault.db"),
        &vault_dir.join("vault.anchor"),
        &key,
    )
    .expect("open initialized Vault");
    for (purpose, record_id, secret) in [
        (
            MailCredentialPurpose::ImapPassword,
            seeded.imap_password_record_id.clone(),
            b"managed-mail-imap-password".as_slice(),
        ),
        (
            MailCredentialPurpose::SmtpPassword,
            seeded.smtp_password_record_id.clone(),
            b"managed-mail-smtp-password".as_slice(),
        ),
    ] {
        let request = VaultPurposeRequestV1::new(
            purpose.as_str().to_owned(),
            MAIL_ACCOUNT_ID.to_owned(),
            vec![SecretClassV1::ProviderCredential],
            vec![VaultActionV1::Resolve],
            MAIL_CREDENTIAL_LEASE_TTL_SECONDS,
        )
        .expect("Mail credential purpose");
        let prior_scope = SecretRecordScope::new(
            MAIL_OWNER_ID.to_owned(),
            &request,
            SecretClassV1::ProviderCredential,
            1,
        )
        .expect("prior Mail credential scope");
        let next_scope = SecretRecordScope::new(
            MAIL_OWNER_ID.to_owned(),
            &request,
            SecretClassV1::ProviderCredential,
            2,
        )
        .expect("next Mail credential scope");
        store
            .replace_secret(&record_id, &prior_scope, &next_scope, secret)
            .expect("rotate Mail test credential");
    }
}

fn store_mail_test_secret(
    store: &VaultStore,
    purpose: MailCredentialPurpose,
    secret_class: SecretClassV1,
    secret: &[u8],
) -> [u8; 16] {
    let request = VaultPurposeRequestV1::new(
        purpose.as_str().to_owned(),
        MAIL_ACCOUNT_ID.to_owned(),
        vec![secret_class],
        vec![
            VaultActionV1::Resolve,
            VaultActionV1::Retire,
            VaultActionV1::Delete,
        ],
        MAIL_CREDENTIAL_LEASE_TTL_SECONDS,
    )
    .expect("Mail test credential purpose");
    let scope = SecretRecordScope::new(MAIL_OWNER_ID.to_owned(), &request, secret_class, 1)
        .expect("Mail test credential scope");
    *store
        .store_secret(&scope, secret)
        .expect("store Mail test credential")
        .as_bytes()
}

pub(super) fn admit_mail_runtime(store: &SqliteControlStore) -> AdmittedMailRuntime {
    admit_mail_runtime_profile(store, MailAdmissionProfileV1::ImapSync)
}

pub(super) fn admit_mail_delivery_runtime(store: &SqliteControlStore) -> AdmittedMailRuntime {
    admit_mail_runtime_profile(store, MailAdmissionProfileV1::SmtpDelivery)
}

pub(super) fn admit_mail_account_credential_runtime(
    store: &SqliteControlStore,
) -> AdmittedMailRuntime {
    admit_mail_runtime_profile(store, MailAdmissionProfileV1::AccountCredentialLifecycle)
}

pub(super) fn admit_mail_attachment_delivery_runtime(
    store: &SqliteControlStore,
) -> AdmittedMailRuntime {
    admit_mail_runtime_profile(store, MailAdmissionProfileV1::SmtpAttachmentDelivery)
}

pub(super) fn admit_mail_gmail_delivery_runtime(store: &SqliteControlStore) -> AdmittedMailRuntime {
    admit_mail_runtime_profile(store, MailAdmissionProfileV1::GmailDelivery)
}

pub(super) fn admit_mail_gmail_attachment_delivery_runtime(
    store: &SqliteControlStore,
) -> AdmittedMailRuntime {
    admit_mail_runtime_profile(store, MailAdmissionProfileV1::GmailAttachmentDelivery)
}

pub(super) fn admit_mail_gmail_oauth_runtime(store: &SqliteControlStore) -> AdmittedMailRuntime {
    admit_mail_runtime_profile(store, MailAdmissionProfileV1::GmailOAuth)
}

pub(super) fn admit_mail_google_people_runtime(store: &SqliteControlStore) -> AdmittedMailRuntime {
    admit_mail_runtime_profile(store, MailAdmissionProfileV1::GooglePeopleAddressBook)
}

pub(super) fn admit_mail_carddav_runtime(store: &SqliteControlStore) -> AdmittedMailRuntime {
    admit_mail_runtime_profile(store, MailAdmissionProfileV1::CardDavAddressBook)
}

fn admit_mail_runtime_profile(
    store: &SqliteControlStore,
    profile: MailAdmissionProfileV1,
) -> AdmittedMailRuntime {
    let descriptor = mail_module_descriptor_v1("managed-mail-live");
    let descriptor_bytes = descriptor.encode_to_vec();
    let registration = crate::modules::registration::registry::register(store, &descriptor_bytes)
        .expect("register exact Mail descriptor");
    let mut capability_ids = match profile {
        MailAdmissionProfileV1::ImapSync => vec![
            MAIL_ATTACHMENT_ANCHOR_CONSUME_CAPABILITY_ID.to_owned(),
            MAIL_ATTACHMENT_BLOB_ADMISSION_PUBLISH_CAPABILITY_ID.to_owned(),
            MAIL_ATTACHMENT_SCAN_CANDIDATE_PUBLISH_CAPABILITY_ID.to_owned(),
            MAIL_BLOB_CAPABILITY_ID.to_owned(),
            MAIL_COMMUNICATION_OBSERVED_PUBLISH_CAPABILITY_ID.to_owned(),
            MAIL_IMAP_CREDENTIALS_CAPABILITY_ID.to_owned(),
            MAIL_STORAGE_CAPABILITY_ID.to_owned(),
            MailClientContractV1::OperationalQuery
                .capability_id()
                .to_owned(),
            MailClientContractV1::MessageFlagCommand
                .capability_id()
                .to_owned(),
            MailClientContractV1::MessageFlagQuery
                .capability_id()
                .to_owned(),
            MailClientContractV1::MessageLocationCommand
                .capability_id()
                .to_owned(),
            MailClientContractV1::MessageLocationQuery
                .capability_id()
                .to_owned(),
            MailClientContractV1::MessagePermanentDeleteCommand
                .capability_id()
                .to_owned(),
            MailClientContractV1::MessagePermanentDeleteQuery
                .capability_id()
                .to_owned(),
            MailClientContractV1::SyncHealthQuery
                .capability_id()
                .to_owned(),
            MailClientContractV1::CompositionCommand
                .capability_id()
                .to_owned(),
            MailClientContractV1::CompositionQuery
                .capability_id()
                .to_owned(),
            MailClientContractV1::Sync.capability_id().to_owned(),
        ],
        MailAdmissionProfileV1::AccountCredentialLifecycle => vec![
            MAIL_COMMUNICATION_OBSERVED_PUBLISH_CAPABILITY_ID.to_owned(),
            MAIL_GMAIL_CREDENTIAL_LIFECYCLE_CAPABILITY_ID.to_owned(),
            MAIL_GMAIL_REFRESH_CREDENTIAL_LIFECYCLE_CAPABILITY_ID.to_owned(),
            MAIL_IMAP_CREDENTIALS_CAPABILITY_ID.to_owned(),
            MAIL_IMAP_CREDENTIAL_LIFECYCLE_CAPABILITY_ID.to_owned(),
            MAIL_SMTP_CREDENTIALS_CAPABILITY_ID.to_owned(),
            MAIL_SMTP_CREDENTIAL_LIFECYCLE_CAPABILITY_ID.to_owned(),
            MAIL_STORAGE_CAPABILITY_ID.to_owned(),
            MailClientContractV1::AccountCredentialBind
                .capability_id()
                .to_owned(),
            MailClientContractV1::AccountQuery
                .capability_id()
                .to_owned(),
            MailClientContractV1::AccountRetire
                .capability_id()
                .to_owned(),
            MailClientContractV1::AccountDelete
                .capability_id()
                .to_owned(),
            MailClientContractV1::AccountLifecycleRetry
                .capability_id()
                .to_owned(),
            MailClientContractV1::AccountLifecycleQuery
                .capability_id()
                .to_owned(),
            MailClientContractV1::Delivery.capability_id().to_owned(),
            MailClientContractV1::DeliveryQuery
                .capability_id()
                .to_owned(),
            MailClientContractV1::Sync.capability_id().to_owned(),
        ],
        MailAdmissionProfileV1::SmtpDelivery => vec![
            MAIL_COMMUNICATION_OBSERVED_PUBLISH_CAPABILITY_ID.to_owned(),
            MAIL_IMAP_CREDENTIALS_CAPABILITY_ID.to_owned(),
            MAIL_SMTP_CREDENTIALS_CAPABILITY_ID.to_owned(),
            MAIL_STORAGE_CAPABILITY_ID.to_owned(),
            MailClientContractV1::Delivery.capability_id().to_owned(),
            MailClientContractV1::DeliveryQuery
                .capability_id()
                .to_owned(),
        ],
        MailAdmissionProfileV1::SmtpAttachmentDelivery => vec![
            MAIL_ATTACHMENT_ANCHOR_CONSUME_CAPABILITY_ID.to_owned(),
            MAIL_ATTACHMENT_BLOB_ADMISSION_PUBLISH_CAPABILITY_ID.to_owned(),
            MAIL_ATTACHMENT_SAFETY_STATE_CONSUME_CAPABILITY_ID.to_owned(),
            MAIL_ATTACHMENT_SCAN_CANDIDATE_PUBLISH_CAPABILITY_ID.to_owned(),
            MAIL_BLOB_CAPABILITY_ID.to_owned(),
            MAIL_COMMUNICATION_OBSERVED_PUBLISH_CAPABILITY_ID.to_owned(),
            MAIL_IMAP_CREDENTIALS_CAPABILITY_ID.to_owned(),
            MAIL_SMTP_CREDENTIALS_CAPABILITY_ID.to_owned(),
            MAIL_STORAGE_CAPABILITY_ID.to_owned(),
            MailClientContractV1::Delivery.capability_id().to_owned(),
            MailClientContractV1::DeliveryQuery
                .capability_id()
                .to_owned(),
            MailClientContractV1::Sync.capability_id().to_owned(),
        ],
        MailAdmissionProfileV1::GmailDelivery => vec![
            MAIL_COMMUNICATION_OBSERVED_PUBLISH_CAPABILITY_ID.to_owned(),
            MAIL_GMAIL_CREDENTIALS_CAPABILITY_ID.to_owned(),
            MAIL_STORAGE_CAPABILITY_ID.to_owned(),
            MailClientContractV1::Delivery.capability_id().to_owned(),
            MailClientContractV1::DeliveryQuery
                .capability_id()
                .to_owned(),
        ],
        MailAdmissionProfileV1::GmailAttachmentDelivery => vec![
            MAIL_ATTACHMENT_ANCHOR_CONSUME_CAPABILITY_ID.to_owned(),
            MAIL_ATTACHMENT_BLOB_ADMISSION_PUBLISH_CAPABILITY_ID.to_owned(),
            MAIL_ATTACHMENT_SAFETY_STATE_CONSUME_CAPABILITY_ID.to_owned(),
            MAIL_ATTACHMENT_SCAN_CANDIDATE_PUBLISH_CAPABILITY_ID.to_owned(),
            MAIL_BLOB_CAPABILITY_ID.to_owned(),
            MAIL_COMMUNICATION_OBSERVED_PUBLISH_CAPABILITY_ID.to_owned(),
            MAIL_GMAIL_CREDENTIALS_CAPABILITY_ID.to_owned(),
            MAIL_STORAGE_CAPABILITY_ID.to_owned(),
            MailClientContractV1::Delivery.capability_id().to_owned(),
            MailClientContractV1::DeliveryQuery
                .capability_id()
                .to_owned(),
            MailClientContractV1::SyncHealthQuery
                .capability_id()
                .to_owned(),
            MailClientContractV1::Sync.capability_id().to_owned(),
        ],
        MailAdmissionProfileV1::GmailOAuth => vec![
            MAIL_COMMUNICATION_OBSERVED_PUBLISH_CAPABILITY_ID.to_owned(),
            MAIL_GMAIL_OAUTH_REFRESH_CREDENTIALS_CAPABILITY_ID.to_owned(),
            MAIL_GMAIL_OAUTH_SETUP_CREDENTIALS_CAPABILITY_ID.to_owned(),
            MAIL_STORAGE_CAPABILITY_ID.to_owned(),
            MailClientContractV1::GmailOAuthComplete
                .capability_id()
                .to_owned(),
            MailClientContractV1::GmailOAuthQuery
                .capability_id()
                .to_owned(),
            MailClientContractV1::GmailOAuthRefresh
                .capability_id()
                .to_owned(),
            MailClientContractV1::GmailOAuthStart
                .capability_id()
                .to_owned(),
        ],
        MailAdmissionProfileV1::GooglePeopleAddressBook => vec![
            CONTACT_MAIL_SYNC_SOURCE_BLOB_TARGET_CAPABILITY_ID_V1.to_owned(),
            MAIL_ADDRESS_BOOK_CAPABILITY_ID_V1.to_owned(),
            MAIL_GMAIL_CREDENTIALS_CAPABILITY_ID.to_owned(),
            MAIL_STORAGE_CAPABILITY_ID.to_owned(),
        ],
        MailAdmissionProfileV1::CardDavAddressBook => vec![
            MAIL_ADDRESS_BOOK_CAPABILITY_ID_V1.to_owned(),
            MAIL_IMAP_CREDENTIALS_CAPABILITY_ID.to_owned(),
            MAIL_STORAGE_CAPABILITY_ID.to_owned(),
        ],
    };
    capability_ids.push(MAIL_DELIVERY_INTENT_TARGET_CAPABILITY_ID_V1.to_owned());
    capability_ids.push(MAIL_RETAINED_EVIDENCE_REPLAY_CAPABILITY_ID.to_owned());
    capability_ids.sort();
    crate::modules::registration::registry::approve_after_owner_authorization(
        store,
        registration.registration_id(),
        &capability_ids,
    )
    .expect("approve exact Mail profile capabilities");
    let schema = mail_settings_schema_bytes_v2();
    crate::modules::settings::schema::admit(
        store,
        registration.registration_id(),
        &descriptor_bytes,
        &schema,
    )
    .expect("admit exact Mail Settings schema");
    store
        .record_bundled_managed_launch_binding(&BundledManagedLaunchBinding::new(
            registration.registration_id(),
            1,
            "makosh-managed-runtime-conformance",
            MAIL_RELEASE_ARTIFACT_ID,
            Sha256::digest(std::fs::read(mail_binary()).expect("Mail runtime binary bytes")).into(),
            Sha256::digest(&descriptor_bytes).into(),
            Some(Sha256::digest(&schema).into()),
        ))
        .expect("record Mail release binding");
    let bundle = mail_runtime_storage_bundle_v1().expect("compose managed Mail Storage bundle");
    let bundle_revision = bundle.revision;
    let bundle = bundle.encode_to_vec();
    store
        .record_platform_storage_bundle(
            &PlatformStorageBundleV1::new(
                MAIL_OWNER_ID,
                u64::from(bundle_revision),
                Sha256::digest(&bundle).into(),
                bundle,
            )
            .expect("record Mail Storage bundle"),
        )
        .expect("persist Mail Storage bundle");
    AdmittedMailRuntime {
        registration_id: registration.registration_id().to_owned(),
        capability_ids,
    }
}

pub(super) fn prepare_mail_runtime(
    supervisor: &ManagedRuntimeSupervisor,
    store: &SqliteControlStore,
    admitted: AdmittedMailRuntime,
) -> AdmittedMailRuntime {
    let reservation = managed_launch::reserve(supervisor, store, &admitted.registration_id)
        .expect("reserve Mail managed launch");
    let runtime_instance_id = reservation.runtime_instance_id().to_owned();
    let runtime_generation = reservation.runtime_generation();
    let bundle_revision = mail_runtime_storage_bundle_v1()
        .expect("compose managed Mail Storage bundle")
        .revision;
    let bundle = store
        .platform_storage_bundle(MAIL_OWNER_ID, u64::from(bundle_revision))
        .expect("read Mail Storage bundle")
        .expect("Mail Storage bundle");
    let binding = issue_managed(
        store,
        &admitted.registration_id,
        &runtime_instance_id,
        runtime_generation,
        MAIL_STORAGE_CAPABILITY_ID,
        StorageBindingIssueV1::new(1, 1, u64::from(bundle_revision), *bundle.digest())
            .expect("Mail Storage binding issue"),
    )
    .expect("issue Mail Storage binding");
    crate::platform::storage::provisioning::apply_reserved_binding(supervisor, store, &binding)
        .expect("provision Mail Storage binding");
    admitted
}

pub(super) fn start_mail_runtime(
    supervisor: &ManagedRuntimeSupervisor,
    store: &SqliteControlStore,
    kernel_data: &Path,
    runtime_dir: &Path,
    admitted: AdmittedMailRuntime,
    imap_port: u16,
) -> StartedMailRuntime {
    seed_basic_mail_bindings(false);
    start_mail_runtime_with_settings(
        supervisor,
        store,
        kernel_data,
        runtime_dir,
        admitted,
        MailSettingsProfileV1::Imap {
            port: imap_port,
            smtp: None,
        },
    )
}

pub(super) fn start_mail_delivery_runtime(
    supervisor: &ManagedRuntimeSupervisor,
    store: &SqliteControlStore,
    kernel_data: &Path,
    runtime_dir: &Path,
    admitted: AdmittedMailRuntime,
    imap_port: u16,
    smtp: MailSmtpFixtureSettingsV1,
) -> StartedMailRuntime {
    seed_basic_mail_bindings(true);
    start_mail_runtime_with_settings(
        supervisor,
        store,
        kernel_data,
        runtime_dir,
        admitted,
        MailSettingsProfileV1::Imap {
            port: imap_port,
            smtp: Some(smtp),
        },
    )
}

pub(super) fn restart_mail_delivery_runtime(
    supervisor: &ManagedRuntimeSupervisor,
    store: &SqliteControlStore,
    kernel_data: &Path,
    runtime_dir: &Path,
    predecessor: StartedMailRuntime,
    imap_port: u16,
    smtp: MailSmtpFixtureSettingsV1,
) -> StartedMailRuntime {
    let predecessor_generation = predecessor.runtime_generation;
    let predecessor_binding = store
        .platform_storage_binding(&predecessor.registration_id, MAIL_STORAGE_CAPABILITY_ID)
        .expect("read predecessor Mail Storage binding")
        .expect("predecessor Mail Storage binding");
    let issue = storage_successor::issue_after(&predecessor_binding)
        .expect("derive Mail successor storage fences");
    let (_, binding) = storage_successor::reserve(
        supervisor,
        store,
        &predecessor.registration_id,
        MAIL_STORAGE_CAPABILITY_ID,
        issue,
    )
    .expect("reserve successor Mail launch and Storage binding");
    crate::platform::storage::provisioning::apply_reserved_binding(supervisor, store, &binding)
        .expect("provision successor Mail Storage binding");
    let successor = start_mail_runtime_with_settings(
        supervisor,
        store,
        kernel_data,
        runtime_dir,
        AdmittedMailRuntime {
            registration_id: predecessor.registration_id,
            capability_ids: predecessor.capability_ids,
        },
        MailSettingsProfileV1::Imap {
            port: imap_port,
            smtp: Some(smtp),
        },
    );
    assert_eq!(
        successor.runtime_generation,
        predecessor_generation + 1,
        "Mail restart must use the next managed runtime generation",
    );
    successor
}

pub(super) fn restart_mail_runtime_without_smtp(
    supervisor: &ManagedRuntimeSupervisor,
    store: &SqliteControlStore,
    kernel_data: &Path,
    runtime_dir: &Path,
    predecessor: StartedMailRuntime,
    imap_port: u16,
) -> StartedMailRuntime {
    restart_mail_runtime_without_smtp_for_human_owner(
        supervisor,
        store,
        kernel_data,
        runtime_dir,
        predecessor,
        imap_port,
        "owner-1",
    )
}

#[allow(clippy::too_many_arguments)]
pub(super) fn restart_mail_runtime_without_smtp_for_human_owner(
    supervisor: &ManagedRuntimeSupervisor,
    store: &SqliteControlStore,
    kernel_data: &Path,
    runtime_dir: &Path,
    predecessor: StartedMailRuntime,
    imap_port: u16,
    logical_human_owner_id: &str,
) -> StartedMailRuntime {
    let predecessor_generation = predecessor.runtime_generation;
    let predecessor_binding = store
        .platform_storage_binding(&predecessor.registration_id, MAIL_STORAGE_CAPABILITY_ID)
        .expect("read predecessor Mail Storage binding")
        .expect("predecessor Mail Storage binding");
    let issue = storage_successor::issue_after(&predecessor_binding)
        .expect("derive Mail successor storage fences");
    let (_, binding) = storage_successor::reserve(
        supervisor,
        store,
        &predecessor.registration_id,
        MAIL_STORAGE_CAPABILITY_ID,
        issue,
    )
    .expect("reserve successor Mail launch and Storage binding");
    crate::platform::storage::provisioning::apply_reserved_binding(supervisor, store, &binding)
        .expect("provision successor Mail Storage binding");
    let successor = start_mail_runtime_with_settings_for_human_owner(
        supervisor,
        store,
        kernel_data,
        runtime_dir,
        AdmittedMailRuntime {
            registration_id: predecessor.registration_id,
            capability_ids: predecessor.capability_ids,
        },
        MailSettingsProfileV1::Imap {
            port: imap_port,
            smtp: None,
        },
        logical_human_owner_id,
    );
    assert_eq!(
        successor.runtime_generation,
        predecessor_generation + 1,
        "Mail restart must use the next managed runtime generation",
    );
    successor
}

pub(super) fn current_mail_runtime(
    store: &SqliteControlStore,
    predecessor: &StartedMailRuntime,
) -> StartedMailRuntime {
    let binding = store
        .platform_storage_binding(&predecessor.registration_id, MAIL_STORAGE_CAPABILITY_ID)
        .expect("read current Mail Storage binding")
        .expect("current Mail Storage binding");
    let registration = store
        .module_registration(&predecessor.registration_id)
        .expect("read current Mail registration")
        .expect("current Mail registration");
    StartedMailRuntime {
        registration_id: predecessor.registration_id.clone(),
        runtime_instance_id: binding.runtime_instance_id().to_owned(),
        runtime_generation: binding.runtime_generation(),
        grant_epoch: registration.grant_epoch(),
        capability_ids: predecessor.capability_ids.clone(),
    }
}

pub(super) fn start_mail_gmail_delivery_runtime(
    supervisor: &ManagedRuntimeSupervisor,
    store: &SqliteControlStore,
    kernel_data: &Path,
    runtime_dir: &Path,
    admitted: AdmittedMailRuntime,
    gmail: MailGmailFixtureSettingsV1,
) -> StartedMailRuntime {
    start_mail_runtime_with_settings(
        supervisor,
        store,
        kernel_data,
        runtime_dir,
        admitted,
        MailSettingsProfileV1::Gmail(gmail),
    )
}

pub(super) fn start_mail_google_people_runtime(
    supervisor: &ManagedRuntimeSupervisor,
    store: &SqliteControlStore,
    kernel_data: &Path,
    runtime_dir: &Path,
    admitted: AdmittedMailRuntime,
    gmail: MailGmailFixtureSettingsV1,
) -> StartedMailRuntime {
    start_mail_runtime_with_settings(
        supervisor,
        store,
        kernel_data,
        runtime_dir,
        admitted,
        MailSettingsProfileV1::GooglePeopleAddressBook(gmail),
    )
}

pub(super) fn start_mail_carddav_runtime(
    supervisor: &ManagedRuntimeSupervisor,
    store: &SqliteControlStore,
    kernel_data: &Path,
    runtime_dir: &Path,
    admitted: AdmittedMailRuntime,
    carddav: MailCardDavFixtureSettingsV1,
) -> StartedMailRuntime {
    seed_basic_mail_bindings(false);
    seed_carddav_binding();
    start_mail_runtime_with_settings(
        supervisor,
        store,
        kernel_data,
        runtime_dir,
        admitted,
        MailSettingsProfileV1::CardDavAddressBook {
            imap_port: carddav.imap_port,
            carddav_port: carddav.carddav_port,
            ca_certificate_pem: carddav.ca_certificate_pem,
        },
    )
}

fn seed_carddav_binding() {
    let runtime = tokio::runtime::Runtime::new().expect("CardDAV binding runtime");
    runtime.block_on(async {
        let durable = super::mail_event_flow::connect_postgres().await;
        if durable
            .account_credential_binding(
                MAIL_ACCOUNT_ID,
                MailCredentialPurposeV1::IcloudCardDavPassword,
            )
            .await
            .expect("query CardDAV credential binding")
            .is_none()
        {
            durable
                .bind_account_credential(
                    &MailBindCredentialRequestV1 {
                        connection_id: MAIL_ACCOUNT_ID.to_owned(),
                        purpose: MailCredentialPurposeV1::IcloudCardDavPassword,
                        expected_binding_revision: 0,
                        credential_revision: 1,
                    },
                    MAIL_ACCOUNT_ID,
                    1,
                )
                .await
                .expect("seed CardDAV credential binding");
        }
    });
}

fn seed_basic_mail_bindings(include_smtp: bool) {
    let runtime = tokio::runtime::Runtime::new().expect("Mail binding runtime");
    runtime.block_on(async {
        let durable = super::mail_event_flow::connect_postgres().await;
        let purposes = if include_smtp {
            vec![
                MailCredentialPurposeV1::ImapPassword,
                MailCredentialPurposeV1::SmtpPassword,
            ]
        } else {
            vec![MailCredentialPurposeV1::ImapPassword]
        };
        for purpose in purposes {
            if durable
                .account_credential_binding(MAIL_ACCOUNT_ID, purpose)
                .await
                .expect("query Mail credential binding")
                .is_none()
            {
                durable
                    .bind_account_credential(
                        &MailBindCredentialRequestV1 {
                            connection_id: MAIL_ACCOUNT_ID.to_owned(),
                            purpose,
                            expected_binding_revision: 0,
                            credential_revision: 1,
                        },
                        MAIL_ACCOUNT_ID,
                        1,
                    )
                    .await
                    .expect("seed Mail credential binding");
            }
        }
    });
}

#[allow(clippy::too_many_arguments)]
fn start_mail_runtime_with_settings(
    supervisor: &ManagedRuntimeSupervisor,
    store: &SqliteControlStore,
    kernel_data: &Path,
    runtime_dir: &Path,
    admitted: AdmittedMailRuntime,
    settings: MailSettingsProfileV1,
) -> StartedMailRuntime {
    start_mail_runtime_with_settings_for_human_owner(
        supervisor,
        store,
        kernel_data,
        runtime_dir,
        admitted,
        settings,
        "owner-1",
    )
}

#[allow(clippy::too_many_arguments)]
fn start_mail_runtime_with_settings_for_human_owner(
    supervisor: &ManagedRuntimeSupervisor,
    store: &SqliteControlStore,
    kernel_data: &Path,
    runtime_dir: &Path,
    admitted: AdmittedMailRuntime,
    settings: MailSettingsProfileV1,
    logical_human_owner_id: &str,
) -> StartedMailRuntime {
    let reservation = managed_launch::load(supervisor, store, &admitted.registration_id)
        .expect("load Mail managed launch reservation");
    let runtime_instance_id = reservation.runtime_instance_id().to_owned();
    let runtime_generation = reservation.runtime_generation();
    let grant_epoch = reservation.grant_epoch();
    let binding = store
        .platform_storage_binding(&admitted.registration_id, MAIL_STORAGE_CAPABILITY_ID)
        .expect("read Mail Storage binding")
        .expect("Mail Storage binding");
    let topology =
        crate::platform::storage::topology::current(store).expect("read Storage topology");
    let vault = vault_status::read_current(store, &supervisor.relay_port())
        .expect("read live Vault status");
    let storage = crate::platform::storage::topology::to_managed_runtime_configuration(
        &topology,
        &binding,
        store.snapshot().instance_id(),
        vault.runtime_generation(),
        vault.hpke_public_key_x25519(),
    )
    .expect("build Mail Storage configuration");
    let events = store
        .platform_event_hub_topology()
        .expect("read Event Hub topology")
        .expect("Event Hub topology");
    let configuration = makosh_runtime_protocol::v1::ManagedIntegrationRuntimeConfigurationV1 {
        major: 1,
        logical_owner_id: MAIL_OWNER_ID.to_owned(),
        registration_id: admitted.registration_id.clone(),
        runtime_instance_id: runtime_instance_id.clone(),
        runtime_generation,
        grant_epoch,
        storage: Some(storage),
        event_hub_endpoint: events.nats_endpoint().to_owned(),
        event_credential_revision: events.credential_revision(),
        configuration_instance_id: MAIL_ACCOUNT_ID.to_owned(),
        runtime_artifacts: Vec::new(),
        integration_state_root: None,
        configuration_instances: Vec::new(),
        logical_human_owner_id: logical_human_owner_id.to_owned(),
    };
    managed_launch::start_reserved_integration(
        supervisor,
        kernel_data,
        runtime_dir,
        reservation,
        managed_launch::ManagedIntegrationLaunchConfiguration {
            runtime: configuration,
            settings_snapshot_bytes: current_mail_settings_snapshot(
                store,
                &admitted.registration_id,
                settings,
            ),
            granted_capability_ids: &admitted.capability_ids,
        },
    )
    .expect("start managed Mail integration");
    supervisor
        .wait_until_ready(&admitted.registration_id)
        .unwrap_or_else(|error| {
            panic!(
                "wait for managed Mail readiness: {error}; last_failure={:?}",
                supervisor.last_failure(&admitted.registration_id)
            )
        });
    StartedMailRuntime {
        registration_id: admitted.registration_id,
        runtime_instance_id,
        runtime_generation,
        grant_epoch,
        capability_ids: admitted.capability_ids,
    }
}

pub(super) fn mail_delivery_settings_snapshot(
    registration_id: &str,
    imap_port: u16,
    smtp: MailSmtpFixtureSettingsV1,
    revision: u64,
) -> makosh_runtime_protocol::v1::SettingsSnapshotV1 {
    mail_settings_snapshot(
        MailSettingsProfileV1::Imap {
            port: imap_port,
            smtp: Some(smtp),
        },
        revision,
        registration_id,
    )
}

fn mail_settings_snapshot(
    profile: MailSettingsProfileV1,
    revision: u64,
    target_id: &str,
) -> makosh_runtime_protocol::v1::SettingsSnapshotV1 {
    use makosh_runtime_protocol::v1::{
        SettingValueV1, SettingsValueEntryV1, setting_value_v1::Value,
    };

    fn entry(setting_id: &str, value: Value) -> SettingsValueEntryV1 {
        SettingsValueEntryV1 {
            setting_id: setting_id.to_owned(),
            value: Some(SettingValueV1 { value: Some(value) }),
        }
    }

    let mut values = vec![
        entry(
            "mail.connection_id",
            Value::StringValue(MAIL_ACCOUNT_ID.to_owned()),
        ),
        entry("mail.sync.window", Value::UnsignedIntegerValue(1)),
        entry("mail.sync.windows", Value::UnsignedIntegerValue(2)),
    ];
    let google_people_enabled =
        matches!(&profile, MailSettingsProfileV1::GooglePeopleAddressBook(_));
    match profile {
        MailSettingsProfileV1::Imap { port, smtp } => {
            values.extend([
                entry("mail.imap.host", Value::StringValue("localhost".to_owned())),
                entry(
                    "mail.imap.port",
                    Value::UnsignedIntegerValue(u64::from(port)),
                ),
                entry(
                    "mail.imap.username",
                    Value::StringValue("owner@example.test".to_owned()),
                ),
                entry("mail.inbound.kind", Value::StringValue("imap".to_owned())),
                entry("mail.smtp.enabled", Value::BooleanValue(smtp.is_some())),
            ]);
            if let Some(smtp) = smtp {
                values.extend([
                    entry(
                        "mail.smtp.ca_certificate_pem",
                        Value::StringValue(smtp.ca_certificate_pem),
                    ),
                    entry(
                        "mail.smtp.from_address",
                        Value::StringValue("owner@example.test".to_owned()),
                    ),
                    entry("mail.smtp.host", Value::StringValue("localhost".to_owned())),
                    entry(
                        "mail.smtp.port",
                        Value::UnsignedIntegerValue(u64::from(smtp.port)),
                    ),
                    entry(
                        "mail.smtp.username",
                        Value::StringValue("owner@example.test".to_owned()),
                    ),
                ]);
            }
        }
        MailSettingsProfileV1::Gmail(gmail)
        | MailSettingsProfileV1::GooglePeopleAddressBook(gmail) => {
            values.extend([
                entry(
                    "mail.gmail.api_host",
                    Value::StringValue("localhost".to_owned()),
                ),
                entry(
                    "mail.gmail.api_port",
                    Value::UnsignedIntegerValue(u64::from(gmail.port)),
                ),
                entry(
                    "mail.gmail.ca_certificate_pem",
                    Value::StringValue(gmail.ca_certificate_pem.clone()),
                ),
                entry(
                    "mail.gmail.from_address",
                    Value::StringValue("owner@example.test".to_owned()),
                ),
                entry("mail.gmail.user_id", Value::StringValue("me".to_owned())),
                entry("mail.inbound.kind", Value::StringValue("gmail".to_owned())),
                entry("mail.smtp.enabled", Value::BooleanValue(false)),
            ]);
            if google_people_enabled {
                values.extend([
                    entry(
                        "mail.address_book.google_people_ca_certificate_pem",
                        Value::StringValue(gmail.ca_certificate_pem.clone()),
                    ),
                    entry(
                        "mail.address_book.google_people_host",
                        Value::StringValue("localhost".to_owned()),
                    ),
                    entry(
                        "mail.address_book.google_people_port",
                        Value::UnsignedIntegerValue(u64::from(gmail.port)),
                    ),
                    entry(
                        "mail.address_book.provider",
                        Value::StringValue("google_people".to_owned()),
                    ),
                ]);
            }
            if let Some(oauth) = gmail.oauth {
                values.extend([
                    entry(
                        "mail.gmail.oauth.authorization_ca_certificate_pem",
                        Value::StringValue(oauth.ca_certificate_pem.clone()),
                    ),
                    entry(
                        "mail.gmail.oauth.authorization_host",
                        Value::StringValue(oauth.host.clone()),
                    ),
                    entry(
                        "mail.gmail.oauth.authorization_path",
                        Value::StringValue("/authorize".to_owned()),
                    ),
                    entry(
                        "mail.gmail.oauth.authorization_port",
                        Value::UnsignedIntegerValue(u64::from(oauth.port)),
                    ),
                    entry(
                        "mail.gmail.oauth.client_id",
                        Value::StringValue("managed-mail-gmail-client".to_owned()),
                    ),
                    entry(
                        "mail.gmail.oauth.redirect_uri",
                        Value::StringValue("https://127.0.0.1/oauth/callback".to_owned()),
                    ),
                    entry(
                        "mail.gmail.oauth.token_ca_certificate_pem",
                        Value::StringValue(oauth.ca_certificate_pem),
                    ),
                    entry(
                        "mail.gmail.oauth.token_host",
                        Value::StringValue(oauth.host.clone()),
                    ),
                    entry(
                        "mail.gmail.oauth.token_path",
                        Value::StringValue("/token".to_owned()),
                    ),
                    entry(
                        "mail.gmail.oauth.token_port",
                        Value::UnsignedIntegerValue(u64::from(oauth.port)),
                    ),
                ]);
            }
        }
        MailSettingsProfileV1::CardDavAddressBook {
            imap_port,
            carddav_port,
            ca_certificate_pem,
        } => {
            values.extend([
                entry(
                    "mail.address_book.carddav_base_path",
                    Value::StringValue("/".to_owned()),
                ),
                entry(
                    "mail.address_book.carddav_ca_certificate_pem",
                    Value::StringValue(ca_certificate_pem),
                ),
                entry(
                    "mail.address_book.carddav_host",
                    Value::StringValue("localhost".to_owned()),
                ),
                entry(
                    "mail.address_book.carddav_port",
                    Value::UnsignedIntegerValue(u64::from(carddav_port)),
                ),
                entry(
                    "mail.address_book.carddav_username",
                    Value::StringValue("owner@example.test".to_owned()),
                ),
                entry(
                    "mail.address_book.provider",
                    Value::StringValue("icloud_carddav".to_owned()),
                ),
                entry("mail.imap.host", Value::StringValue("localhost".to_owned())),
                entry(
                    "mail.imap.port",
                    Value::UnsignedIntegerValue(u64::from(imap_port)),
                ),
                entry(
                    "mail.imap.username",
                    Value::StringValue("owner@example.test".to_owned()),
                ),
                entry("mail.inbound.kind", Value::StringValue("imap".to_owned())),
                entry("mail.smtp.enabled", Value::BooleanValue(false)),
            ]);
        }
    }
    values.sort_by(|left, right| left.setting_id.cmp(&right.setting_id));
    makosh_runtime_protocol::v1::SettingsSnapshotV1 {
        target_id: target_id.to_owned(),
        revision,
        values,
    }
}

fn current_mail_settings_snapshot(
    store: &SqliteControlStore,
    registration_id: &str,
    profile: MailSettingsProfileV1,
) -> Vec<u8> {
    let binding = store
        .settings_schema_binding(registration_id)
        .expect("read Mail Settings binding")
        .expect("Mail Settings binding");
    if binding.desired_revision() == 0 {
        let snapshot = mail_settings_snapshot(profile, 1, registration_id).encode_to_vec();
        crate::modules::settings::mutation::commit_after_owner_authorization(
            store,
            registration_id,
            0,
            &snapshot,
        )
        .expect("commit initial Mail Settings");
        for acknowledgement in [
            crate::modules::settings::application::ApplyAcknowledgement::ValidationAccepted,
            crate::modules::settings::application::ApplyAcknowledgement::ApplyStarted,
            crate::modules::settings::application::ApplyAcknowledgement::RuntimeApplied,
        ] {
            crate::modules::settings::application::acknowledge(
                store,
                registration_id,
                1,
                acknowledgement,
            )
            .expect("admit initial Mail Settings state");
        }
        return snapshot;
    }
    let (revision, snapshot) = store
        .desired_settings_snapshot(registration_id)
        .expect("read desired Mail Settings")
        .expect("desired Mail Settings");
    assert_eq!(revision, binding.effective_revision());
    snapshot
}

fn mail_binary() -> PathBuf {
    binary("MAKOSH_MAIL_RUNTIME_BIN")
}
