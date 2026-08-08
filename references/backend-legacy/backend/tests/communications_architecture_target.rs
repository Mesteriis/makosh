use std::fs;
use std::path::{Path, PathBuf};

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("backend has repository parent")
        .to_path_buf()
}

#[test]
fn channel_providers_are_not_product_domains_or_user_routes() {
    let root = repo_root();

    let backend_domains_dir = root.join("backend/src/domains");
    let frontend_domains_dir = root.join("frontend/src/domains");
    assert!(
        !backend_domains_dir.join("mail").exists(),
        "mail must not remain a backend domain"
    );
    assert!(
        root.join("backend/src/domains/communications").is_dir(),
        "communications must be the backend communication domain"
    );
    assert!(
        !frontend_domains_dir.join("telegram").exists(),
        "Telegram must not remain a frontend product domain"
    );
    assert!(
        !frontend_domains_dir.join("whatsapp").exists(),
        "WhatsApp must not remain a frontend product domain"
    );

    let domains_mod = read(root.join("backend/src/domains/mod.rs"));
    assert!(
        !domains_mod.contains("mod mail"),
        "backend domains mod must not export mail"
    );
    assert!(
        domains_mod.contains("mod communications"),
        "backend domains mod must export communications"
    );
    for legacy_mail_runtime_mod in [
        "pub mod accounts;",
        "pub mod background_sync;",
        "pub mod sync;",
        "pub mod rfc822;",
        "pub mod send;",
        "pub mod imap_write;",
    ] {
        assert!(
            !domains_mod.contains(legacy_mail_runtime_mod),
            "communications domain must not own mail runtime module {legacy_mail_runtime_mod}"
        );
    }
    let mail_integrations_mod = read(root.join("backend/src/integrations/mail/mod.rs"));
    for integration_mail_mod in [
        "pub mod accounts;",
        "pub mod sync_provider;",
        "pub mod read_state;",
        "pub mod outbox;",
        "pub mod send;",
        "pub mod imap_write;",
    ] {
        assert!(
            mail_integrations_mod.contains(integration_mail_mod),
            "mail integrations module must export {integration_mail_mod}"
        );
    }
    let workflows_mod = read(root.join("backend/src/workflows/mod.rs"));
    assert!(
        workflows_mod.contains("pub mod mail_background_sync;"),
        "mail background sync must be a workflow process manager, not an integration module"
    );

    let router_sources = read_all_sources(root.join("backend/src/app/router"));
    let legacy_mail_domain_import = format!("domains::{}", "mail");
    assert!(
        !router_sources.contains(&legacy_mail_domain_import),
        "router code must not import the old mail domain"
    );
    for legacy_prefix in [
        format!("\"/api/v1/{}", "telegram"),
        format!("\"/api/v1/{}", "whatsapp"),
        format!("\"/api/v1/{}", "email-accounts"),
    ] {
        assert!(
            !router_sources.contains(&legacy_prefix),
            "legacy user-facing provider route prefix remains: {legacy_prefix}"
        );
    }
    for integration_prefix in [
        "\"/api/v1/integrations/telegram",
        "\"/api/v1/integrations/whatsapp",
        "\"/api/v1/integrations/mail",
    ] {
        assert!(
            router_sources.contains(integration_prefix),
            "provider runtime/setup routes must live under integrations: {integration_prefix}"
        );
    }
    assert!(
        !router_sources.contains("\"/api/v1/integrations/whatsapp/messages\""),
        "WhatsApp message business reads must use provider-neutral Communications routes"
    );
    for forbidden_communication_prefix in ["mail", "telegram", "whatsapp"]
        .map(|provider| format!("\"/api/v1/communications/{provider}"))
    {
        assert!(
            !router_sources.contains(&forbidden_communication_prefix),
            "provider-specific communication route prefix remains: {forbidden_communication_prefix}"
        );
    }
    for removed_provider_communication_prefix in [
        "\"/api/v1/communications/provider-conversations",
        "\"/api/v1/communications/provider-messages",
        "\"/api/v1/communications/provider-web-messages",
    ] {
        assert!(
            !router_sources.contains(removed_provider_communication_prefix),
            "removed provider-shaped communication route still exists: {removed_provider_communication_prefix}"
        );
    }
    assert!(
        router_sources.contains("\"/api/v1/communications/")
            && !router_sources.contains(&format!("\"/api/v1/communications/{}/accounts", "mail")),
        "communications router must keep product routes under /api/v1/communications without resurrecting mail runtime setup paths"
    );
    for provider_neutral_communication_prefix in [
        "\"/api/v1/communications/messages",
        "\"/api/v1/communications/search",
    ] {
        assert!(
            router_sources.contains(provider_neutral_communication_prefix),
            "communications router must expose provider-neutral communication routes: {provider_neutral_communication_prefix}"
        );
    }

    let telegram_chats_handler =
        read(root.join("backend/src/app/provider_runtime_handlers/telegram/chats.rs"));
    let telegram_search_handler =
        read(root.join("backend/src/app/provider_runtime_handlers/telegram/search.rs"));
    assert!(
        telegram_chats_handler.contains("query.channel_kind.as_deref()")
            && telegram_search_handler.contains("query.channel_kind.as_deref()"),
        "provider-neutral conversation routes must honor channel_kind filtering for WhatsApp/Telegram conversation reads"
    );
    assert!(
        telegram_chats_handler.contains("includes_whatsapp_channel_kind")
            && telegram_chats_handler.contains("includes_telegram_channel_kind")
            && telegram_search_handler.contains("includes_whatsapp_channel_kind")
            && telegram_search_handler.contains("includes_telegram_channel_kind"),
        "provider-neutral conversation routes must not mix Telegram runtime rows into WhatsApp-filtered reads"
    );
    assert!(
        telegram_search_handler.contains("query.channel_kind.as_deref()")
            && telegram_search_handler.contains("search_channel_kinds("),
        "provider-neutral message/media search routes must honor channel_kind filtering for WhatsApp/Telegram reads"
    );

    let frontend_app_sources = read_all_sources(root.join("frontend/src/app"));
    assert!(
        !frontend_app_sources.contains("'/telegram'")
            && !frontend_app_sources.contains("\"/telegram\""),
        "Telegram must not remain a top-level frontend route"
    );
    assert!(
        !frontend_app_sources.contains("'/whatsapp'")
            && !frontend_app_sources.contains("\"/whatsapp\""),
        "WhatsApp must not remain a top-level frontend route"
    );

    let frontend_communications_domain =
        read_all_sources(root.join("frontend/src/domains/communications"));
    let frontend_communications_queries =
        read_all_sources(root.join("frontend/src/domains/communications/queries"));
    let frontend_integration_runtime = read_all_sources(root.join("frontend/src/integrations"));
    let frontend_platform_bootstrap =
        read_all_sources(root.join("frontend/src/platform/bootstrap"));
    let frontend_layout_scopes = read(root.join("frontend/src/shared/stores/layoutEditor.ts"));
    let legacy_telegram_key = format!("['{}'", "telegram");
    let legacy_whatsapp_key = format!("['{}'", "whatsapp");
    assert!(
        !frontend_communications_queries.contains(&legacy_telegram_key),
        "user-facing communication caches must not use provider-rooted Telegram query keys"
    );
    assert!(
        !frontend_communications_queries.contains(&legacy_whatsapp_key),
        "user-facing communication caches must not use provider-rooted WhatsApp query keys"
    );
    for forbidden_business_key in [
        "['integrations', 'telegram', 'messages'",
        "['integrations', 'telegram', 'chats'",
        "['integrations', 'whatsapp', 'messages'",
    ] {
        assert!(
            !frontend_integration_runtime.contains(forbidden_business_key),
            "provider business cache key must live under communications, not integrations: {forbidden_business_key}"
        );
    }
    assert!(
        frontend_communications_domain.contains("['communications', 'telegram', 'messages'")
            && frontend_communications_domain.contains("['communications', 'telegram', 'chats'")
            && frontend_communications_domain.contains("['communications', 'whatsapp', 'messages'"),
        "Telegram/WhatsApp business caches must be owned by the Communications domain"
    );
    assert!(
        !frontend_integration_runtime.contains("['communications', 'telegram'")
            && !frontend_integration_runtime.contains("['communications', 'whatsapp'")
            && !frontend_integration_runtime.contains("\"/api/v1/communications/messages")
            && !frontend_integration_runtime.contains("\"/api/v1/communications/conversations")
            && !frontend_integration_runtime.contains("\"/api/v1/communications/search")
            && !frontend_integration_runtime.contains("\"/api/v1/communications/topics"),
        "integration modules must not own Communications business cache keys or business routes"
    );
    assert!(
        frontend_platform_bootstrap
            .contains("domains/communications/queries/realtimeTelegramPatches")
            && frontend_platform_bootstrap
                .contains("domains/communications/queries/realtimeTelegramParticipantPatches")
            && frontend_platform_bootstrap
                .contains("integrations/telegram/queries/realtimeTelegramCommandPatches"),
        "platform realtime bootstrap must compose Communications business patching separately from Telegram integration runtime patching"
    );
    assert!(
        frontend_integration_runtime.contains("['integrations', 'telegram'"),
        "Telegram runtime query keys must be scoped under integrations"
    );
    assert!(
        frontend_integration_runtime.contains("['integrations', 'whatsapp'"),
        "WhatsApp runtime query keys must be scoped under integrations"
    );
    assert!(
        frontend_layout_scopes.contains("viewScope: ['communications', 'telegram']")
            && frontend_layout_scopes.contains("viewScope: ['communications', 'whatsapp']"),
        "communications workspace must keep Telegram and WhatsApp as communication filters/scopes"
    );
}

#[test]
fn app_messaging_handlers_are_thin() {
    let root = repo_root();
    let telegram_handler_root = root.join("backend/src/app/handlers/telegram");
    let router_support = read(root.join("backend/src/app/router/routes/support.rs"));
    let all_handler_sources = read_all_sources(root.join("backend/src/app/handlers"));
    let app_sources = read_all_sources(root.join("backend/src/app"));
    let provider_runtime_handler_sources =
        read_all_sources(root.join("backend/src/app/provider_runtime_handlers"));

    let telegram_handler_sources = read_all_sources(telegram_handler_root);
    assert!(
        telegram_handler_sources.trim().is_empty(),
        "backend/src/app/handlers/telegram must not contain provider runtime/store implementation files"
    );
    for facade in [
        "telegram.rs",
        "whatsapp.rs",
        "yandex_telemost.rs",
        "zoom.rs",
        "zulip.rs",
    ] {
        assert!(
            !root.join("backend/src/app/handlers").join(facade).exists(),
            "provider handler facade must not exist: {facade}"
        );
    }
    assert!(
        router_support.contains("provider_runtime_handlers::telegram")
            && router_support.contains("provider_runtime_handlers::whatsapp"),
        "router composition must import provider runtime handlers from their owner module"
    );
    for forbidden in [
        "telegram_store(",
        "whatsapp_store(",
        "crate::integrations::telegram::client::lifecycle",
    ] {
        assert!(
            !all_handler_sources.contains(forbidden),
            "app handlers must not call provider runtime/store helper directly: {forbidden}"
        );
    }
    for forbidden in [
        "telegram_store(",
        "whatsapp_store(",
        "TelegramStore::new",
        "WhatsappWebStore::new",
        "whatsapp_provider_runtime_mux(",
    ] {
        assert!(
            !app_sources.contains(forbidden),
            "backend/src/app must not own provider runtime composition or concrete stores: {forbidden}"
        );
    }
    for forbidden in [
        "telegram_runtime_store(",
        "whatsapp_runtime_store(",
        "TelegramStore::new",
        "WhatsappWebStore::new",
        "whatsapp_provider_runtime_mux(",
    ] {
        assert!(
            !provider_runtime_handler_sources.contains(forbidden),
            "provider runtime handlers may consume provider DTOs but must not compose provider runtimes or stores: {forbidden}"
        );
    }
}

#[test]
fn whatsapp_runtime_is_a_single_hidden_webview_trait_adapter() {
    let root = repo_root();
    let runtime_source = read_all_sources(root.join("backend/src/integrations/whatsapp/runtime"));
    let application_services =
        read(root.join("backend/src/application/provider_runtime_services.rs"));

    assert!(
        runtime_source.contains("pub trait WhatsAppProviderRuntime")
            && runtime_source.contains("impl WhatsAppProviderRuntime for WhatsappWebStore")
            && runtime_source.contains("WhatsAppProviderRuntimeShape::WebCompanion"),
        "WhatsApp must retain its typed runtime boundary with the WebView store as its only adapter"
    );
    assert!(
        application_services.contains("whatsapp_web_companion_runtime(")
            && !application_services.contains("whatsapp_provider_runtime_mux(")
            && !application_services.contains("native_md")
            && !application_services.contains("business_cloud"),
        "composition must instantiate only the hidden WebView adapter"
    );
    assert!(
        !runtime_source.contains("NativeMultiDevice")
            && !runtime_source.contains("BusinessCloud")
            && !runtime_source.contains("whatsapp_native_md")
            && !runtime_source.contains("whatsapp_business_cloud"),
        "the runtime contract must not retain inactive provider shapes"
    );
}

#[test]
fn whatsapp_removed_runtimes_have_no_binary_dependency_or_route_surface() {
    let root = repo_root();
    let backend_cargo = read(root.join("backend/Cargo.toml"));
    let router = read(root.join("backend/src/app/router/routes/messaging.rs"));
    let runtime = read(root.join("backend/src/integrations/whatsapp/runtime/mod.rs"));

    assert!(
        !root
            .join("backend/src/integrations/whatsapp/runtime/native_md.rs")
            .exists()
            && !root
                .join("backend/src/integrations/whatsapp/runtime/business_cloud.rs")
                .exists()
            && !root
                .join("backend/src/bin/makosh_whatsapp_business_cloud_edge_proxy.rs")
                .exists()
            && !root
                .join("backend/src/application/whatsapp_command_executor.rs")
                .exists(),
        "removed WhatsApp runtimes and their in-process command worker must not remain in the production tree"
    );
    assert!(
        !backend_cargo.contains("wa-rs")
            && !backend_cargo.contains("whatsapp-native-md-runtime")
            && !backend_cargo.contains("whatsapp-business-cloud-runtime")
            && !backend_cargo.contains("business-cloud-edge-proxy"),
        "the backend manifest must not build deleted WhatsApp runtimes"
    );
    assert!(
        !router.contains("runtime-bridge/business-cloud")
            && !runtime.contains("whatsapp_native_md")
            && !runtime.contains("whatsapp_business_cloud"),
        "HTTP routing and the runtime must expose only the hidden WebView path"
    );
}

#[test]
fn whatsapp_runtime_has_no_direct_domain_or_engine_write_boundary() {
    let root = repo_root();
    let whatsapp_integration_sources =
        read_all_sources(root.join("backend/src/integrations/whatsapp"));
    let whatsapp_application_sources = read_all_sources(root.join("backend/src/application"));
    let domain_engine_workflow_sources = read_all_sources(root.join("backend/src/domains"))
        + &read_all_sources(root.join("backend/src/engines"))
        + &read_all_sources(root.join("backend/src/workflows"));

    for forbidden in [
        "crate::domains::personas",
        "crate::domains::tasks",
        "crate::domains::documents",
        "crate::domains::memory",
        "crate::engines::search",
        "crate::engines::timeline",
        "crate::engines::consistency",
        "ReviewInbox",
        "TaskCandidate",
        "DecisionCandidate",
        "Obligation",
        "MemoryObservation",
    ] {
        assert!(
            !whatsapp_integration_sources.contains(forbidden),
            "WhatsApp runtime/integration must publish evidence/events, not write domain or engine state directly: {forbidden}"
        );
    }

    for forbidden in [
        "crate::integrations::whatsapp",
        "WhatsAppProviderRuntime",
        "WhatsappWebStore",
        "whatsapp_provider_runtime",
    ] {
        assert!(
            !domain_engine_workflow_sources.contains(forbidden),
            "domains/engines/workflows must not depend on WhatsApp runtime/provider implementation: {forbidden}"
        );
    }

    assert!(
        whatsapp_application_sources.contains("whatsapp_runtime_event_projection")
            && whatsapp_application_sources
                .contains("whatsapp_provider_observation_reconciliation"),
        "WhatsApp application layer must consume event-spine observations through projection/reconciliation workers"
    );
}

#[test]
fn canonical_communication_migration_exists_after_0148() {
    let root = repo_root();
    let migration =
        read(root.join("backend/migrations/0149_create_canonical_communication_tables.sql"));
    let canonical_migration_corpus = migration.clone()
        + &read(root.join("backend/migrations/0157_create_whatsapp_provider_write_commands.sql"));

    for table_name in [
        "communication_accounts",
        "communication_channels",
        "communication_identities",
        "communication_conversations",
        "communication_conversation_participants",
        "communication_message_versions",
        "communication_message_tombstones",
        "communication_message_reactions",
        "communication_message_refs",
        "communication_folders",
        "communication_drafts",
        "communication_outbox",
        "communication_provider_commands",
        "communication_sync_runs",
        "communication_sync_checkpoints",
        "communication_raw_payloads",
    ] {
        assert!(
            migration.contains(&format!("CREATE TABLE IF NOT EXISTS {table_name}")),
            "canonical migration must create {table_name}"
        );
    }

    for legacy_source in [
        "FROM communication_provider_accounts",
        "FROM telegram_message_versions",
        "FROM telegram_message_tombstones",
        "FROM telegram_message_reactions",
        "FROM telegram_message_reply_refs",
        "FROM telegram_message_forward_refs",
        "FROM mail_folders",
        "FROM mail_saved_searches",
        "FROM email_drafts",
        "FROM email_outbox_tracking",
        "FROM telegram_provider_write_commands",
        "FROM whatsapp_provider_write_commands",
        "FROM communication_mail_sync_runs",
        "FROM communication_ingestion_checkpoints",
        "FROM communication_raw_records",
    ] {
        assert!(
            canonical_migration_corpus.contains(legacy_source),
            "canonical migration must migrate legacy source {legacy_source}"
        );
    }
}

#[test]
fn whatsapp_docs_describe_the_single_hidden_webview_boundary() {
    let root = repo_root();
    let whatsapp_readme = read(root.join("docs/integrations/whatsapp/README.md"));
    let api_reference = read(root.join("docs/integrations/whatsapp/api.md"));
    let architecture = read(root.join("docs/integrations/whatsapp/architecture.md"));
    let adr = read(root.join("docs/adr/ADR-0182-whatsapp-hidden-webview-runtime-only.md"));

    assert!(
        whatsapp_readme.contains("ADR-0182")
            && whatsapp_readme.contains("hidden Tauri WebView")
            && whatsapp_readme.contains("cannot be selected, started or dispatched"),
        "WhatsApp README must describe the current single hidden-WebView policy"
    );
    assert!(
        api_reference.contains("whatsapp_web_companion")
            && api_reference.contains("no Business Cloud webhook/proxy routes"),
        "WhatsApp API reference must exclude retired provider surfaces"
    );
    assert!(
        architecture.contains("Hidden account-scoped Tauri WebView")
            && architecture.contains("No Native MD, Business Cloud"),
        "WhatsApp architecture must retain only the hidden WebView topology"
    );
    assert!(
        adr.contains("WhatsApp has exactly one provider runtime")
            && adr.contains("visible(false)")
            && adr.contains("fallback or alternate WhatsApp topology exists"),
        "the accepted ADR must make the hidden WebView runtime exclusive"
    );
}

#[test]
fn communication_runtime_core_uses_canonical_storage_tables() {
    let root = repo_root();
    for (path, forbidden) in [
        (
            "backend/src/domains/communications/ai_state.rs",
            vec!["mail_ai_states"],
        ),
        (
            "backend/src/domains/communications/drafts.rs",
            vec!["email_drafts"],
        ),
        (
            "backend/src/domains/communications/folders.rs",
            vec!["mail_folders", "mail_folder_messages"],
        ),
        (
            "backend/src/domains/communications/outbox.rs",
            vec!["email_outbox_tracking", "mail_read_receipts"],
        ),
        (
            "backend/src/domains/communications/templates.rs",
            vec!["email_templates"],
        ),
        (
            "backend/src/domains/communications/rules/store.rs",
            vec!["email_rules"],
        ),
        (
            "backend/src/domains/communications/finance.rs",
            vec!["email_invoices"],
        ),
        (
            "backend/src/domains/communications/legal.rs",
            vec!["email_legal_documents"],
        ),
        (
            "backend/src/domains/communications/personas.rs",
            vec!["email_personas"],
        ),
        (
            "backend/src/domains/communications/signatures/store.rs",
            vec!["email_certificates"],
        ),
        (
            "backend/src/domains/communications/read_receipts.rs",
            vec!["email_outbox_tracking", "mail_read_receipts"],
        ),
        (
            "backend/src/domains/communications/saved_searches.rs",
            vec!["mail_saved_searches"],
        ),
    ] {
        let source = read(root.join(path));
        for needle in forbidden {
            assert!(
                !source.contains(needle),
                "{path} must not read or write legacy storage table {needle}"
            );
        }
    }

    let read_receipts_migration =
        read(root.join("backend/migrations/0151_create_communication_read_receipts.sql"));
    assert!(
        read_receipts_migration.contains("CREATE TABLE IF NOT EXISTS communication_read_receipts"),
        "read receipt canonical migration must create communication_read_receipts"
    );
    assert!(
        read_receipts_migration.contains("FROM mail_read_receipts"),
        "read receipt canonical migration must backfill from mail_read_receipts"
    );

    let aux_migration =
        read(root.join("backend/migrations/0152_create_canonical_communication_aux_tables.sql"));
    for table_name in [
        "communication_rules",
        "communication_templates",
        "communication_personas",
        "communication_invoices",
        "communication_legal_documents",
        "communication_certificates",
    ] {
        assert!(
            aux_migration.contains(&format!("CREATE TABLE IF NOT EXISTS {table_name}")),
            "aux canonical migration must create {table_name}"
        );
    }
    for legacy_source in [
        "FROM email_rules",
        "FROM email_templates",
        "FROM email_personas",
        "FROM email_invoices",
        "FROM email_legal_documents",
        "FROM email_certificates",
    ] {
        assert!(
            aux_migration.contains(legacy_source),
            "aux canonical migration must backfill from {legacy_source}"
        );
    }
}

#[test]
fn communications_domain_uses_canonical_dto_names() {
    let root = repo_root();
    let backend_domain = read_all_sources(root.join("backend/src/domains/communications"));
    let frontend_domain = read_all_sources(root.join("frontend/src/domains/communications"));

    for legacy_name in [
        "EmailDraft",
        "NewEmailDraft",
        "EmailTemplate",
        "NewEmailTemplate",
        "EmailPersona",
        "NewEmailPersona",
        "EmailOutboxItem",
        "NewEmailOutboxItem",
        "EmailOutboxStatus",
        "EmailThread",
        "MailThreadSummary",
        "EmailSearchResponse",
        "SendEmailRequest",
        "SendEmailResponse",
        "EmailFinanceStore",
        "EmailExport",
    ] {
        assert!(
            !backend_domain.contains(legacy_name),
            "backend communications domain must not expose legacy DTO name {legacy_name}"
        );
        assert!(
            !frontend_domain.contains(legacy_name),
            "frontend communications domain must not expose legacy DTO name {legacy_name}"
        );
    }

    for canonical_name in [
        "CommunicationDraft",
        "CommunicationTemplate",
        "CommunicationPersona",
        "CommunicationOutboxItem",
        "CommunicationOutboxStatus",
        "CommunicationThread",
        "CommunicationThreadSummary",
        "CommunicationSearchResponse",
        "SendCommunicationRequest",
        "SendCommunicationResponse",
    ] {
        assert!(
            backend_domain.contains(canonical_name) || frontend_domain.contains(canonical_name),
            "communications domain must expose canonical DTO name {canonical_name}"
        );
    }
}

fn read(path: PathBuf) -> String {
    fs::read_to_string(&path).unwrap_or_else(|error| {
        panic!("failed to read {}: {error}", path.display());
    })
}

fn read_all_sources(root: PathBuf) -> String {
    let mut output = String::new();
    collect_sources(&root, &mut output);
    output
}

fn collect_sources(path: &Path, output: &mut String) {
    if path.is_file() {
        if matches!(
            path.extension().and_then(|value| value.to_str()),
            Some("rs" | "ts" | "vue" | "js" | "mjs")
        ) {
            output.push_str(&read(path.to_path_buf()));
            output.push('\n');
        }
        return;
    }

    let Ok(entries) = fs::read_dir(path) else {
        return;
    };
    for entry in entries.flatten() {
        collect_sources(&entry.path(), output);
    }
}
