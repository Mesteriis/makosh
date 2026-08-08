# Задача для DeepSeek: обновить русскую Obsidian wiki

## Safety instructions / Инструкции безопасности

- Do not print, infer, summarize, or request secrets. / Не печатай, не выводи, не пересказывай и не запрашивай секреты.
- Treat `.env`, credential, token, key, certificate, and private paths as redacted even if referenced. / Считай `.env`, учетные данные, токены, ключи, сертификаты и приватные пути редактированными.
- Keep code identifiers, file paths, commands, package names, API names, and ADR titles exactly as written. / Сохраняй идентификаторы кода, пути, команды, имена пакетов, API и названия ADR без изменений.
- Write wiki prose in Russian and keep Markdown Obsidian-compatible. / Пиши текст wiki на русском и сохраняй совместимость с Obsidian Markdown.
- Do not invent source facts. If the context is insufficient, state that explicitly. / Не выдумывай факты об исходниках. Если контекста недостаточно, напиши это явно.
- Every behavioral statement in proposed wiki pages must be directly supported by the embedded source text. / Каждое утверждение о поведении в предлагаемых wiki-страницах должно напрямую подтверждаться встроенным текстом исходников.
- Do not infer semantics for profiles, flags, annotations, environment variables, or framework conventions unless this context pack explicitly defines them. / Не выводи семантику профилей, флагов, аннотаций, переменных окружения или framework-конвенций, если этот context pack явно её не определяет.
- Do not add external background knowledge about tools, frameworks, or CLIs. / Не добавляй внешние справочные знания об инструментах, framework или CLI.
- When only a command or config value is visible, document only the literal command or value. For deeper meaning, write only that it is not confirmed by this context. / Когда видна только команда или значение конфигурации, документируй только буквальную команду или значение. Для более глубокого смысла пиши только, что он не подтвержден этим контекстом.
- Do not name likely related files unless they are embedded in this context pack. / Не называй вероятные связанные файлы, если они не встроены в этот context pack.
- Use only the embedded Source Files section below. Do not call tools, read files, inspect the filesystem, or access MCP/web resources. / Используй только встроенный ниже раздел Source Files. Не вызывай tools, не читай файлы, не инспектируй файловую систему и не обращайся к MCP/web ресурсам.
- If a referenced path or wiki page is not embedded in this context pack, report insufficient context instead of trying to open it. / Если упомянутый путь или wiki-страница не встроены в этот context pack, укажи недостаток контекста вместо попытки открыть файл.

## Chunk details / Детали чанка

- Chunk ID / ID чанка: `080-test-backend-part-003`
- Group / Группа: `backend`
- Role / Роль: `test`
- Status / Статус: `pending`
- Repository / Репозиторий: `/Users/avm/projects/Personal/makosh`
- Wiki path / Путь wiki: `/Users/avm/projects/Personal/makosh/docs/wiki`
- Metadata path / Путь metadata: `/Users/avm/projects/Personal/makosh/docs/wiki/_meta`
- Plan generated at / План создан: `2026-06-28T19:48:55Z`
- Per-file source limit / Лимит источника на файл: `12000` characters

## Target pages / Целевые страницы

- `operations/backend-tests.md`

## Required Output / Требуемый результат

Return one Markdown response with these sections and no extra wrapper text. / Верни один Markdown-ответ с этими разделами и без дополнительной обертки.

### Summary / Резюме

Briefly describe what should change in the Russian wiki and why. / Кратко опиши, что нужно изменить в русской wiki и почему.

### Proposed pages / Предлагаемые страницы

For each target page, provide the wiki-relative path and full proposed Obsidian-compatible Markdown content. / Для каждой целевой страницы укажи путь относительно wiki и полный предложенный Markdown, совместимый с Obsidian.

### Source coverage / Покрытие источников

List each source file and the facts from it that the proposed pages cover. / Перечисли каждый исходный файл и факты из него, покрытые предложенными страницами.

### Drift candidates / Кандидаты на drift

List possible code/docs/ADR drift found in this chunk, or state that none is visible from the provided context. / Перечисли возможные расхождения кода, документации и ADR в этом чанке либо укажи, что из данного контекста они не видны.

## Source Files / Исходные файлы

### `backend/tests/communication_ingestion/credential_reader.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/tests/communication_ingestion/credential_reader.rs`
- Size bytes / Размер в байтах: `9314`
- Included characters / Включено символов: `9314`
- Truncated / Обрезано: `no`

```rust
use crate::support::*;

#[tokio::test]
async fn provider_credential_reader_resolves_bound_account_secret_against_postgres() {
    let Some(database) = connect_database("provider credential reader test fixture database").await
    else {
        return;
    };

    let pool = database.pool().expect("configured pool").clone();
    let communication_store = CommunicationIngestionStore::new(pool.clone());
    let secret_binding_store =
        makosh_hub_backend::domains::communications::core::CommunicationProviderSecretBindingStore::new(
            pool.clone(),
        );
    let secret_store = SecretReferenceStore::new(pool);
    let suffix = unique_suffix();
    let account_id = format!("acct_credential_reader_{suffix}");
    let secret_ref = format!("secret:test:credential-reader:{suffix}");
    let mut resolver = InMemorySecretResolver::new();

    communication_store
        .upsert_provider_account(&NewProviderAccount::new(
            &account_id,
            EmailProviderKind::Gmail,
            "Gmail credential reader",
            format!("credential-reader-{suffix}@example.com"),
        ))
        .await
        .expect("store provider account");
    secret_store
        .upsert_secret_reference(&NewSecretReference::new(
            &secret_ref,
            SecretKind::OauthToken,
            SecretStoreKind::TestDouble,
            "Gmail test credential",
        ))
        .await
        .expect("store secret reference");
    communication_store
        .bind_provider_account_secret(&NewProviderAccountSecretBinding::new(
            &account_id,
            ProviderAccountSecretPurpose::OauthToken,
            &secret_ref,
        ))
        .await
        .expect("bind account secret");
    resolver
        .insert(&secret_ref, "test-only-gmail-runtime-value")
        .expect("insert in-memory runtime value");

    let reader = ProviderCredentialReader::new(secret_binding_store, secret_store, &resolver);
    let credential = reader
        .read(&account_id, ProviderAccountSecretPurpose::OauthToken)
        .await
        .expect("read provider credential");

    assert_eq!(credential.binding.account_id, account_id);
    assert_eq!(
        credential.binding.secret_purpose,
        ProviderAccountSecretPurpose::OauthToken
    );
    assert_eq!(credential.reference.secret_ref, secret_ref);
    assert_eq!(credential.reference.secret_kind, SecretKind::OauthToken);
    assert_eq!(
        credential.secret.expose_for_runtime(),
        "test-only-gmail-runtime-value"
    );
    assert!(!format!("{credential:?}").contains("test-only-gmail-runtime-value"));
}

#[tokio::test]
async fn provider_credential_reader_reports_missing_binding_against_postgres() {
    let Some(database) =
        connect_database("missing provider credential binding test fixture database").await
    else {
        return;
    };

    let pool = database.pool().expect("configured pool").clone();
    let communication_store = CommunicationIngestionStore::new(pool.clone());
    let secret_binding_store =
        makosh_hub_backend::domains::communications::core::CommunicationProviderSecretBindingStore::new(
            pool.clone(),
        );
    let secret_store = SecretReferenceStore::new(pool);
    let suffix = unique_suffix();
    let account_id = format!("acct_missing_credential_binding_{suffix}");
    let resolver = InMemorySecretResolver::new();

    communication_store
        .upsert_provider_account(&NewProviderAccount::new(
            &account_id,
            EmailProviderKind::Icloud,
            "iCloud missing credential binding",
            format!("missing-credential-binding-{suffix}@icloud.com"),
        ))
        .await
        .expect("store provider account");

    let reader = ProviderCredentialReader::new(secret_binding_store, secret_store, &resolver);
    let error = reader
        .read(&account_id, ProviderAccountSecretPurpose::ImapPassword)
        .await
        .expect_err("missing credential binding should fail");

    match error {
        ProviderCredentialError::MissingBinding {
            account_id: error_account_id,
            secret_purpose,
        } => {
            assert_eq!(error_account_id, account_id);
            assert_eq!(secret_purpose, ProviderAccountSecretPurpose::ImapPassword);
        }
        other => panic!("unexpected provider credential error: {other:?}"),
    }
}

#[tokio::test]
async fn provider_credential_reader_propagates_resolver_failures_against_postgres() {
    let Some(database) =
        connect_database("provider credential resolver failure test fixture database").await
    else {
        return;
    };

    let pool = database.pool().expect("configured pool").clone();
    let communication_store = CommunicationIngestionStore::new(pool.clone());
    let secret_binding_store =
        makosh_hub_backend::domains::communications::core::CommunicationProviderSecretBindingStore::new(
            pool.clone(),
        );
    let secret_store = SecretReferenceStore::new(pool);
    let suffix = unique_suffix();
    let account_id = format!("acct_resolver_failure_{suffix}");
    let secret_ref = format!("secret:os-keychain:resolver-failure:{suffix}");
    let mut resolver = InMemorySecretResolver::new();

    communication_store
        .upsert_provider_account(&NewProviderAccount::new(
            &account_id,
            EmailProviderKind::Imap,
            "IMAP resolver failure",
            format!("resolver-failure-{suffix}@example.net"),
        ))
        .await
        .expect("store provider account");
    secret_store
        .upsert_secret_reference(&NewSecretReference::new(
            &secret_ref,
            SecretKind::Password,
            SecretStoreKind::OsKeychain,
            "IMAP keychain credential",
        ))
        .await
        .expect("store secret reference");
    communication_store
        .bind_provider_account_secret(&NewProviderAccountSecretBinding::new(
            &account_id,
            ProviderAccountSecretPurpose::ImapPassword,
            &secret_ref,
        ))
        .await
        .expect("bind account secret");
    resolver
        .insert(&secret_ref, "test-only-imap-runtime-value")
        .expect("insert in-memory runtime value");

    let reader = ProviderCredentialReader::new(secret_binding_store, secret_store, &resolver);
    let error = reader
        .read(&account_id, ProviderAccountSecretPurpose::ImapPassword)
        .await
        .expect_err("unsupported resolver store kind should fail");

    match error {
        ProviderCredentialError::SecretResolution(SecretResolutionError::UnsupportedStoreKind(
            store_kind,
        )) => assert_eq!(store_kind, "os_keychain"),
        other => panic!("unexpected provider credential error: {other:?}"),
    }
}

#[tokio::test]
async fn provider_credential_reader_rejects_incompatible_secret_kind_against_postgres() {
    let Some(database) =
        connect_database("incompatible provider credential kind test fixture database").await
    else {
        return;
    };

    let pool = database.pool().expect("configured pool").clone();
    let communication_store = CommunicationIngestionStore::new(pool.clone());
    let secret_binding_store =
        makosh_hub_backend::domains::communications::core::CommunicationProviderSecretBindingStore::new(
            pool.clone(),
        );
    let secret_store = SecretReferenceStore::new(pool);
    let suffix = unique_suffix();
    let account_id = format!("acct_incompatible_credential_kind_{suffix}");
    let secret_ref = format!("secret:test:incompatible-kind:{suffix}");
    let resolver = InMemorySecretResolver::new();

    communication_store
        .upsert_provider_account(&NewProviderAccount::new(
            &account_id,
            EmailProviderKind::Gmail,
            "Gmail incompatible credential kind",
            format!("incompatible-kind-{suffix}@example.com"),
        ))
        .await
        .expect("store provider account");
    secret_store
        .upsert_secret_reference(&NewSecretReference::new(
            &secret_ref,
            SecretKind::Password,
            SecretStoreKind::TestDouble,
            "Wrong Gmail credential kind",
        ))
        .await
        .expect("store incompatible secret reference");
    communication_store
        .bind_provider_account_secret(&NewProviderAccountSecretBinding::new(
            &account_id,
            ProviderAccountSecretPurpose::OauthToken,
            &secret_ref,
        ))
        .await
        .expect("bind incompatible account secret");

    let reader = ProviderCredentialReader::new(secret_binding_store, secret_store, &resolver);
    let error = reader
        .read(&account_id, ProviderAccountSecretPurpose::OauthToken)
        .await
        .expect_err("incompatible credential kind should fail");

    match error {
        ProviderCredentialError::IncompatibleSecretKind {
            secret_ref: error_secret_ref,
            secret_purpose,
            secret_kind,
        } => {
            assert_eq!(error_secret_ref, secret_ref);
            assert_eq!(secret_purpose, ProviderAccountSecretPurpose::OauthToken);
            assert_eq!(secret_kind, SecretKind::Password);
        }
        other => panic!("unexpected provider credential error: {other:?}"),
    }
}
```

### `backend/tests/communication_ingestion/raw_records.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/tests/communication_ingestion/raw_records.rs`
- Size bytes / Размер в байтах: `4292`
- Included characters / Включено символов: `4292`
- Truncated / Обрезано: `no`

```rust
use crate::support::*;
use sqlx::Row;

#[tokio::test]
async fn communication_ingestion_records_raw_sources_idempotently_against_postgres() {
    let Some(database) = connect_database("communication raw source test fixture database").await
    else {
        return;
    };

    let pool = database.pool().expect("configured pool").clone();
    let store = CommunicationIngestionStore::new(pool.clone());
    let suffix = unique_suffix();
    let account_id = format!("acct_raw_{suffix}");
    let provider_record_id = format!("gmail-message-{suffix}");

    store
        .upsert_provider_account(&NewProviderAccount::new(
            &account_id,
            EmailProviderKind::Gmail,
            "Gmail raw source test",
            format!("raw-{suffix}@example.com"),
        ))
        .await
        .expect("store provider account");

    let first = store
        .record_raw_source(
            &NewRawCommunicationRecord::new(
                format!("raw_{suffix}"),
                &account_id,
                "email_message",
                &provider_record_id,
                format!("sha256:{suffix}"),
                format!("batch_{suffix}"),
                json!({"id": provider_record_id, "provider": "gmail"}),
            )
            .occurred_at(Utc::now())
            .provenance(json!({"source": "gmail-api"})),
        )
        .await
        .expect("record raw source");

    let duplicate = store
        .record_raw_source(
            &NewRawCommunicationRecord::new(
                format!("raw_duplicate_{suffix}"),
                &account_id,
                "email_message",
                &provider_record_id,
                format!("sha256:different-{suffix}"),
                format!("batch_{suffix}"),
                json!({"id": provider_record_id, "provider": "gmail", "changed": true}),
            )
            .provenance(json!({"source": "retry"})),
        )
        .await
        .expect("record duplicate raw source");

    assert_eq!(duplicate.raw_record_id, first.raw_record_id);
    assert_eq!(duplicate.observation_id, first.observation_id);
    assert_eq!(duplicate.payload, first.payload);

    let count = sqlx::query_scalar::<_, i64>(
        r#"
        SELECT count(*)
        FROM communication_raw_records
        WHERE account_id = $1
          AND record_kind = 'email_message'
          AND provider_record_id = $2
        "#,
    )
    .bind(&account_id)
    .bind(&provider_record_id)
    .fetch_one(&pool)
    .await
    .expect("raw record count");
    assert_eq!(count, 1);

    let observation = sqlx::query(
        r#"
        SELECT
            observation.observation_id,
            kind.code AS kind_code,
            observation.source_ref,
            observation.provenance
        FROM observations observation
        JOIN observation_kind_definitions kind
          ON kind.kind_definition_id = observation.kind_definition_id
        WHERE observation.observation_id = $1
        "#,
    )
    .bind(&first.observation_id)
    .fetch_one(&pool)
    .await
    .expect("canonical raw record observation");

    let kind_code: String = observation.try_get("kind_code").expect("kind code");
    let source_ref: String = observation.try_get("source_ref").expect("source ref");
    let provenance: serde_json::Value = observation.try_get("provenance").expect("provenance");
    assert_eq!(kind_code, "COMMUNICATION_MESSAGE");
    assert_eq!(
        source_ref,
        format!("communication://{account_id}/email_message/{provider_record_id}")
    );
    assert_eq!(provenance["communication_raw_record"], json!(true));
    assert_eq!(provenance["raw_record_id"], json!(first.raw_record_id));

    let observation_count = sqlx::query_scalar::<_, i64>(
        r#"
        SELECT count(*)
        FROM observations
        WHERE source_ref = $1
        "#,
    )
    .bind(&source_ref)
    .fetch_one(&pool)
    .await
    .expect("raw record observation count");
    assert_eq!(observation_count, 1);

    let mutation = sqlx::query(
        "UPDATE communication_raw_records SET payload = '{}'::jsonb WHERE raw_record_id = $1",
    )
    .bind(&first.raw_record_id)
    .execute(&pool)
    .await;
    assert!(
        mutation.is_err(),
        "raw provider records must be append-only"
    );
}
```

### `backend/tests/communication_ingestion/secret_bindings.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/tests/communication_ingestion/secret_bindings.rs`
- Size bytes / Размер в байтах: `9777`
- Included characters / Включено символов: `9777`
- Truncated / Обрезано: `no`

```rust
use crate::support::*;

#[tokio::test]
async fn communication_ingestion_binds_provider_accounts_to_secret_refs_against_postgres() {
    let Some(database) =
        connect_database("communication secret binding test fixture database").await
    else {
        return;
    };

    let pool = database.pool().expect("configured pool").clone();
    let communication_store = CommunicationIngestionStore::new(pool.clone());
    let secret_store = SecretReferenceStore::new(pool);
    let suffix = unique_suffix();
    let gmail_account_id = format!("acct_gmail_secret_{suffix}");
    let icloud_account_id = format!("acct_icloud_secret_{suffix}");
    let imap_account_id = format!("acct_imap_secret_{suffix}");

    communication_store
        .upsert_provider_account(&NewProviderAccount::new(
            &gmail_account_id,
            EmailProviderKind::Gmail,
            "Gmail secret binding",
            format!("gmail-secret-{suffix}@example.com"),
        ))
        .await
        .expect("store gmail account");
    communication_store
        .upsert_provider_account(&NewProviderAccount::new(
            &icloud_account_id,
            EmailProviderKind::Icloud,
            "iCloud secret binding",
            format!("icloud-secret-{suffix}@icloud.com"),
        ))
        .await
        .expect("store icloud account");
    communication_store
        .upsert_provider_account(&NewProviderAccount::new(
            &imap_account_id,
            EmailProviderKind::Imap,
            "IMAP secret binding",
            format!("imap-secret-{suffix}@example.net"),
        ))
        .await
        .expect("store imap account");

    let gmail_secret_ref = format!("secret:gmail:oauth:{suffix}");
    let icloud_secret_ref = format!("secret:icloud:app-password:{suffix}");
    let imap_secret_ref = format!("secret:imap:password:{suffix}");

    secret_store
        .upsert_secret_reference(&NewSecretReference::new(
            &gmail_secret_ref,
            SecretKind::OauthToken,
            SecretStoreKind::OsKeychain,
            "Gmail OAuth credential",
        ))
        .await
        .expect("store gmail secret reference");
    secret_store
        .upsert_secret_reference(&NewSecretReference::new(
            &icloud_secret_ref,
            SecretKind::AppPassword,
            SecretStoreKind::OsKeychain,
            "iCloud app-specific password",
        ))
        .await
        .expect("store icloud secret reference");
    secret_store
        .upsert_secret_reference(&NewSecretReference::new(
            &imap_secret_ref,
            SecretKind::Password,
            SecretStoreKind::OsKeychain,
            "Generic IMAP password",
        ))
        .await
        .expect("store imap secret reference");

    let gmail_binding = communication_store
        .bind_provider_account_secret(&NewProviderAccountSecretBinding::new(
            &gmail_account_id,
            ProviderAccountSecretPurpose::OauthToken,
            &gmail_secret_ref,
        ))
        .await
        .expect("bind gmail oauth secret");
    let icloud_binding = communication_store
        .bind_provider_account_secret(&NewProviderAccountSecretBinding::new(
            &icloud_account_id,
            ProviderAccountSecretPurpose::ImapPassword,
            &icloud_secret_ref,
        ))
        .await
        .expect("bind icloud imap secret");
    let imap_binding = communication_store
        .bind_provider_account_secret(&NewProviderAccountSecretBinding::new(
            &imap_account_id,
            ProviderAccountSecretPurpose::ImapPassword,
            &imap_secret_ref,
        ))
        .await
        .expect("bind generic imap secret");

    assert_eq!(gmail_binding.secret_ref, gmail_secret_ref);
    assert_eq!(icloud_binding.secret_ref, icloud_secret_ref);
    assert_eq!(imap_binding.secret_ref, imap_secret_ref);

    let gmail_bindings = communication_store
        .provider_account_secret_bindings(&gmail_account_id)
        .await
        .expect("load gmail secret bindings");
    assert_eq!(gmail_bindings, vec![gmail_binding]);
}

#[tokio::test]
async fn communication_ingestion_scopes_secret_refs_by_provider_account_against_postgres() {
    let Some(database) =
        connect_database("multi-account secret binding test fixture database").await
    else {
        return;
    };

    let pool = database.pool().expect("configured pool").clone();
    let communication_store = CommunicationIngestionStore::new(pool.clone());
    let secret_store = SecretReferenceStore::new(pool);
    let suffix = unique_suffix();
    let accounts = [
        (
            format!("acct_multi_gmail_a_{suffix}"),
            EmailProviderKind::Gmail,
            "Gmail work",
            format!("gmail-work-{suffix}@example.com"),
            ProviderAccountSecretPurpose::OauthToken,
            SecretKind::OauthToken,
            format!("secret:test:gmail:work:{suffix}"),
            "fake-gmail-work-runtime-secret",
        ),
        (
            format!("acct_multi_gmail_b_{suffix}"),
            EmailProviderKind::Gmail,
            "Gmail personal",
            format!("gmail-personal-{suffix}@example.com"),
            ProviderAccountSecretPurpose::OauthToken,
            SecretKind::OauthToken,
            format!("secret:test:gmail:personal:{suffix}"),
            "fake-gmail-personal-runtime-secret",
        ),
        (
            format!("acct_multi_icloud_a_{suffix}"),
            EmailProviderKind::Icloud,
            "iCloud work",
            format!("icloud-work-{suffix}@icloud.com"),
            ProviderAccountSecretPurpose::ImapPassword,
            SecretKind::AppPassword,
            format!("secret:test:icloud:work:{suffix}"),
            "fake-icloud-work-runtime-secret",
        ),
        (
            format!("acct_multi_icloud_b_{suffix}"),
            EmailProviderKind::Icloud,
            "iCloud personal",
            format!("icloud-personal-{suffix}@icloud.com"),
            ProviderAccountSecretPurpose::ImapPassword,
            SecretKind::AppPassword,
            format!("secret:test:icloud:personal:{suffix}"),
            "fake-icloud-personal-runtime-secret",
        ),
    ];
    let mut resolver = InMemorySecretResolver::new();

    for (
        account_id,
        provider_kind,
        display_name,
        external_account_id,
        secret_purpose,
        secret_kind,
        secret_ref,
        runtime_value,
    ) in &accounts
    {
        communication_store
            .upsert_provider_account(&NewProviderAccount::new(
                account_id.as_str(),
                *provider_kind,
                *display_name,
                external_account_id.as_str(),
            ))
            .await
            .expect("store provider account");
        secret_store
            .upsert_secret_reference(&NewSecretReference::new(
                secret_ref.as_str(),
                *secret_kind,
                SecretStoreKind::TestDouble,
                format!("{display_name} credential"),
            ))
            .await
            .expect("store secret reference");
        communication_store
            .bind_provider_account_secret(&NewProviderAccountSecretBinding::new(
                account_id.as_str(),
                *secret_purpose,
                secret_ref.as_str(),
            ))
            .await
            .expect("bind account secret");
        resolver
            .insert(secret_ref.as_str(), *runtime_value)
            .expect("insert in-memory runtime secret");
    }

    for (
        account_id,
        _provider_kind,
        _display_name,
        _external_account_id,
        secret_purpose,
        _secret_kind,
        secret_ref,
        runtime_value,
    ) in &accounts
    {
        let binding = communication_store
            .provider_account_secret_binding(account_id.as_str(), *secret_purpose)
            .await
            .expect("load account secret binding")
            .expect("account secret binding exists");
        assert_eq!(binding.account_id, *account_id);
        assert_eq!(binding.secret_ref, *secret_ref);

        let reference = secret_store
            .secret_reference(&binding.secret_ref)
            .await
            .expect("load secret reference")
            .expect("secret reference exists");
        let resolved = resolver
            .resolve(&reference)
            .await
            .expect("resolve account-scoped secret");
        assert_eq!(resolved.expose_for_runtime(), *runtime_value);
    }

    let first_gmail_binding = communication_store
        .provider_account_secret_binding(&accounts[0].0, ProviderAccountSecretPurpose::OauthToken)
        .await
        .expect("load first gmail binding")
        .expect("first gmail binding exists");
    let second_gmail_binding = communication_store
        .provider_account_secret_binding(&accounts[1].0, ProviderAccountSecretPurpose::OauthToken)
        .await
        .expect("load second gmail binding")
        .expect("second gmail binding exists");
    assert_ne!(
        first_gmail_binding.secret_ref,
        second_gmail_binding.secret_ref
    );

    let first_icloud_binding = communication_store
        .provider_account_secret_binding(&accounts[2].0, ProviderAccountSecretPurpose::ImapPassword)
        .await
        .expect("load first icloud binding")
        .expect("first icloud binding exists");
    let second_icloud_binding = communication_store
        .provider_account_secret_binding(&accounts[3].0, ProviderAccountSecretPurpose::ImapPassword)
        .await
        .expect("load second icloud binding")
        .expect("second icloud binding exists");
    assert_ne!(
        first_icloud_binding.secret_ref,
        second_icloud_binding.secret_ref
    );
}
```

### `backend/tests/communication_ingestion/support.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/tests/communication_ingestion/support.rs`
- Size bytes / Размер в байтах: `1384`
- Included characters / Включено символов: `1384`
- Truncated / Обрезано: `no`

```rust
use std::time::{SystemTime, UNIX_EPOCH};
use testkit::context::TestContext;

pub(crate) use chrono::Utc;
pub(crate) use makosh_hub_backend::domains::communications::core::{
    CommunicationIngestionStore, EmailProviderKind, NewIngestionCheckpoint, NewProviderAccount,
    NewProviderAccountSecretBinding, NewRawCommunicationRecord, ProviderAccountSecretPurpose,
    ProviderCredentialError, ProviderCredentialReader,
};
pub(crate) use makosh_hub_backend::platform::secrets::{
    InMemorySecretResolver, NewSecretReference, SecretKind, SecretReferenceStore,
    SecretResolutionError, SecretResolver, SecretStoreKind,
};
pub(crate) use makosh_hub_backend::platform::storage::Database;
pub(crate) use serde_json::json;

pub(crate) async fn test_database_url(test_name: &str) -> Option<String> {
    let _ = test_name;
    let test_context = TestContext::new().await;
    let database_url = test_context.connection_string();
    Some(database_url)
}

pub(crate) async fn connect_database(test_name: &str) -> Option<Database> {
    let database_url = test_database_url(test_name).await?;
    Some(
        Database::connect(Some(&database_url))
            .await
            .expect("database connection"),
    )
}

pub(crate) fn unique_suffix() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock after unix epoch")
        .as_nanos()
}
```

### `backend/tests/communication_ingestion_architecture.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/tests/communication_ingestion_architecture.rs`
- Size bytes / Размер в байтах: `2094`
- Included characters / Включено символов: `2094`
- Truncated / Обрезано: `no`

```rust
use std::fs;
use std::path::{Path, PathBuf};

const MAX_TEST_FILE_LINES: usize = 700;

#[test]
fn communication_ingestion_tests_stay_below_architecture_line_limit() {
    let tests_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests");

    let mut violations = Vec::new();
    collect_communication_ingestion_test_violations(&tests_dir, &mut violations);

    assert!(
        violations.is_empty(),
        "communication ingestion test files exceed {MAX_TEST_FILE_LINES} lines:\n{}",
        violations.join("\n")
    );
}

fn collect_communication_ingestion_test_violations(root: &Path, violations: &mut Vec<String>) {
    let entries = fs::read_dir(root)
        .unwrap_or_else(|error| panic!("failed to read test directory {root:?}: {error}"));

    for entry in entries {
        let entry = entry.expect("failed to read test directory entry");
        let path = entry.path();
        if path.is_dir() {
            collect_communication_ingestion_test_violations(&path, violations);
            continue;
        }
        if !is_communication_ingestion_test_file(&path) {
            continue;
        }

        let content = fs::read_to_string(&path)
            .unwrap_or_else(|error| panic!("failed to read {path:?}: {error}"));
        let line_count = content.lines().count();
        if line_count > MAX_TEST_FILE_LINES {
            violations.push(format!("{}: {line_count}", path.display()));
        }
    }
}

fn is_communication_ingestion_test_file(path: &Path) -> bool {
    let file_name = path
        .file_name()
        .and_then(|file_name| file_name.to_str())
        .unwrap_or_default();
    if file_name == "communication_ingestion.rs"
        || file_name == "communication_ingestion_architecture.rs"
    {
        return true;
    }

    path.components().any(|component| {
        component
            .as_os_str()
            .to_str()
            .is_some_and(|value| value == "communication_ingestion")
    }) && path
        .extension()
        .and_then(|extension| extension.to_str())
        .is_some_and(|extension| extension == "rs")
}
```

### `backend/tests/communications_architecture_target.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/tests/communications_architecture_target.rs`
- Size bytes / Размер в байтах: `64797`
- Included characters / Включено символов: `12000`
- Truncated / Обрезано: `yes`

```rust
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
        "pub mod sync;",
        "pub mod rfc822;",
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

    let frontend_router = read(root.join("frontend/src/app/router.ts"));
    assert!(
        !frontend_router.contains("'/telegram'") && !frontend_router.contains("\"/telegram\""),
        "Telegram must not remain a top-level frontend route"
    );
    assert!(
        !frontend_router.contains("'/whatsapp'") && !frontend_router.contains("\"/whatsapp\""),
        "WhatsApp must not remain a top-level frontend route"
    );

    let frontend_communications_domain =
        read_all_sources(root.join("frontend/src/domains/communications"));
    let frontend_integration_runtime = read_all_sources(root.join("frontend/src/integrations"));
    let frontend_platform_bootstrap =
        read_all_sources(root.join("frontend/src/platform/bootstrap"));
    let frontend_layout_scopes = read(root.join("frontend/src/shared/stores/layoutEditor.ts"));
    let legacy_telegram_key = format!("['{}'", "telegram");
    let legacy_whatsapp_key = format!("['{}'", "whatsapp");
    assert!(
        !frontend_communications_domain.contains(&legacy_telegram_key),
        "user-facing communication caches must not use provider-rooted Telegram query keys"
    );
    assert!(
        !frontend_communications_domain.contains(&legacy_whatsapp_key),
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
    let telegram_handler_facade = read(root.join("backend/src/app/handlers/telegram.rs"));
    let whatsapp_handler = read(root.join("backend/src/app/handlers/whatsapp.rs"));
    let all_handler_sources = read_all_sources(root.join("backend/src/app/handlers"));
    let app_sources = read_all_sources(root.join("backend/src/app"));
    let provider_runtime_handler_sources =
        read_all_sources(root.join("backend/src/app/provider_runtime_handlers"));

    let telegram_handler_sources = read_all_sources(telegram_handler_root);
    assert!(
        telegram_handler_sources.trim().is_empty(),
        "backend/src/app/handlers/telegram must not contain provider runtime/store implementation files"
    );
    assert!(
        telegram_handler_facade.contains("provider_runtime_handlers::telegram")
            && whatsapp_handler.contains("provider_runtime_handlers::whatsapp"),
        "messaging app handlers must be thin facades over the provider runtime composition root"
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
        "crate::integrations::telegram::client",
        "crate::integrations::whatsapp::client",
```
_Source file truncated after 12000 characters. / Исходный файл обрезан после 12000 символов._

### `backend/tests/communications_connectrpc.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/tests/communications_connectrpc.rs`
- Size bytes / Размер в байтах: `44239`
- Included characters / Включено символов: `12000`
- Truncated / Обрезано: `yes`

```rust
use axum::body::to_bytes;
use axum::http::StatusCode;
use chrono::Utc;
use makosh_hub_backend::domains::communications::core::{
    CommunicationIngestionStore, EmailProviderKind, NewProviderAccount, NewRawCommunicationRecord,
};
use makosh_hub_backend::domains::communications::drafts::{
    CommunicationDraftStore, DraftStatus, NewCommunicationDraft,
};
use makosh_hub_backend::domains::communications::messages::{
    MessageProjectionStore, project_raw_email_message,
};
use makosh_hub_backend::domains::communications::outbox::{
    CommunicationOutboxStatus, CommunicationOutboxStore, NewCommunicationOutboxItem,
};
use makosh_hub_backend::domains::communications::storage::{
    AttachmentSafetyScanReport, AttachmentSafetyScanStatus, CommunicationAttachmentDisposition,
    CommunicationStorageStore, LocalCommunicationBlobStore, NewCommunicationAttachment,
    NewCommunicationBlob,
};
use makosh_hub_backend::workflows::mail_background_sync::DEFAULT_MAIL_SYNC_BLOB_ROOT;
use serde_json::{Value, json};
use testkit::app::{TestApp, post_json};
use tower::ServiceExt;

#[tokio::test]
async fn communications_connect_api_requires_local_api_secret() {
    let app = TestApp::new().await;
    let router = app.clone_router();

    let forbidden_response = router
        .clone()
        .oneshot(
            axum::http::Request::builder()
                .method("POST")
                .uri("/makosh.communications.v1.CommunicationsService/ListMessages")
                .header("content-type", "application/json")
                .body(axum::body::Body::from("{}"))
                .expect("connect request without secret"),
        )
        .await
        .expect("connect response without secret");
    assert_eq!(forbidden_response.status(), StatusCode::FORBIDDEN);

    let allowed_response = router
        .oneshot(post_json(
            "/makosh.communications.v1.CommunicationsService/ListMessages",
            json!({}),
        ))
        .await
        .expect("connect response with secret");
    assert_eq!(allowed_response.status(), StatusCode::OK);
}

#[tokio::test]
async fn communications_connect_api_exposes_provider_neutral_queries_and_send() {
    let app = TestApp::new().await;
    let pool = app.context().pool().clone();
    let router = app.clone_router();
    let ingestion = CommunicationIngestionStore::new(pool.clone());
    let message_store = MessageProjectionStore::new(pool.clone());
    let draft_store = CommunicationDraftStore::new(pool.clone());
    let outbox_store = CommunicationOutboxStore::new(pool);

    ingestion
        .upsert_provider_account(&NewProviderAccount::new(
            "acct-connectrpc-mail",
            EmailProviderKind::Gmail,
            "ConnectRPC Mail",
            "connectrpc@example.com",
        ))
        .await
        .expect("store provider account");
    let raw = ingestion
        .record_raw_source(
            &NewRawCommunicationRecord::new(
                "raw-connectrpc-message",
                "acct-connectrpc-mail",
                "email_message",
                "provider-connectrpc-message",
                "sha256:connectrpc-message",
                "batch-connectrpc-message",
                json!({
                    "subject": "ConnectRPC Thread",
                    "from": "alice@example.com",
                    "to": ["bob@example.com"],
                    "body_text": "ConnectRPC message body"
                }),
            )
            .occurred_at(Utc::now())
            .provenance(json!({"source": "communications_connectrpc_test"})),
        )
        .await
        .expect("record raw message");
    let projected = project_raw_email_message(&message_store, &raw)
        .await
        .expect("project raw message");
    let raw_newsletter = ingestion
        .record_raw_source(
            &NewRawCommunicationRecord::new(
                "raw-connectrpc-message-2",
                "acct-connectrpc-mail",
                "email_message",
                "provider-connectrpc-message-2",
                "sha256:connectrpc-message-2",
                "batch-connectrpc-message",
                json!({
                    "subject": "ConnectRPC Thread",
                    "from": "alice@example.com",
                    "to": ["bob@example.com"],
                    "body_text": "ConnectRPC newsletter body with unsubscribe link"
                }),
            )
            .occurred_at(Utc::now())
            .provenance(json!({"source": "communications_connectrpc_test"})),
        )
        .await
        .expect("record second raw message");
    project_raw_email_message(&message_store, &raw_newsletter)
        .await
        .expect("project second raw message");
    let seeded_attachment = seed_connectrpc_attachment(
        &app.context().pool().clone(),
        &raw.raw_record_id,
        &projected.message_id,
        "connectrpc-note.txt",
        "text/plain",
        b"Hola equipo\n",
    )
    .await;
    let seeded_pdf_attachment = seed_connectrpc_attachment(
        &app.context().pool().clone(),
        &raw.raw_record_id,
        &projected.message_id,
        "connectrpc-spec.pdf",
        "application/pdf",
        b"%PDF-1.4\n",
    )
    .await;

    draft_store
        .upsert(&NewCommunicationDraft {
            draft_id: "draft-connectrpc-1".to_owned(),
            account_id: "acct-connectrpc-mail".to_owned(),
            persona_id: None,
            to_recipients: vec!["draft@example.com".to_owned()],
            cc_recipients: Vec::new(),
            bcc_recipients: Vec::new(),
            subject: "ConnectRPC Draft".to_owned(),
            body_text: "Draft body".to_owned(),
            body_html: None,
            in_reply_to: None,
            references: Vec::new(),
            status: DraftStatus::Draft,
            scheduled_send_at: None,
            metadata: json!({"origin":"connectrpc_test"}),
        })
        .await
        .expect("store draft");
    outbox_store
        .enqueue(&NewCommunicationOutboxItem {
            outbox_id: "outbox-connectrpc-1".to_owned(),
            account_id: "acct-connectrpc-mail".to_owned(),
            draft_id: None,
            to_recipients: vec!["queued@example.com".to_owned()],
            cc_recipients: Vec::new(),
            bcc_recipients: Vec::new(),
            subject: "Queued ConnectRPC".to_owned(),
            body_text: "Queued body".to_owned(),
            body_html: None,
            status: CommunicationOutboxStatus::Queued,
            scheduled_send_at: None,
            undo_deadline_at: Some(Utc::now() + chrono::Duration::minutes(5)),
            metadata: json!({"seeded": true}),
        })
        .await
        .expect("store outbox");

    let list_messages = response_json(
        router
            .clone()
            .oneshot(post_json(
                "/makosh.communications.v1.CommunicationsService/ListMessages",
                json!({
                    "account_id": "acct-connectrpc-mail",
                    "limit": 10
                }),
            ))
            .await
            .expect("list messages response"),
    )
    .await;
    assert!(list_messages["items"].as_array().is_some_and(|items| {
        items
            .iter()
            .any(|item| item["messageId"] == projected.message_id)
    }));
    assert!(
        list_messages["items"]
            .as_array()
            .is_some_and(|items| items.iter().any(|item| item["workflowState"] == "new"))
    );

    let get_message = response_json(
        router
            .clone()
            .oneshot(post_json(
                "/makosh.communications.v1.CommunicationsService/GetMessage",
                json!({
                    "message_id": projected.message_id
                }),
            ))
            .await
            .expect("get message response"),
    )
    .await;
    assert_eq!(get_message["item"]["subject"], "ConnectRPC Thread");
    assert_eq!(
        get_message["attachments"][0]["attachmentId"],
        seeded_attachment.attachment_id
    );

    let transitioned_message = response_json(
        router
            .clone()
            .oneshot(post_json(
                "/makosh.communications.v1.CommunicationsService/TransitionMessageWorkflowState",
                json!({
                    "message_id": projected.message_id,
                    "workflow_state": "reviewed"
                }),
            ))
            .await
            .expect("transition workflow state response"),
    )
    .await;
    assert_eq!(transitioned_message["messageId"], projected.message_id);
    assert_eq!(transitioned_message["workflowState"], "reviewed");
    assert_eq!(transitioned_message["previousState"], "new");

    let trashed_message = response_json(
        router
            .clone()
            .oneshot(post_json(
                "/makosh.communications.v1.CommunicationsService/TrashMessage",
                json!({
                    "message_id": projected.message_id
                }),
            ))
            .await
            .expect("trash message response"),
    )
    .await;
    assert_eq!(trashed_message["messageId"], projected.message_id);
    assert_eq!(trashed_message["localState"], "trash");

    let restored_message = response_json(
        router
            .clone()
            .oneshot(post_json(
                "/makosh.communications.v1.CommunicationsService/RestoreMessage",
                json!({
                    "message_id": projected.message_id
                }),
            ))
            .await
            .expect("restore message response"),
    )
    .await;
    assert_eq!(restored_message["messageId"], projected.message_id);
    assert_eq!(restored_message["localState"], "active");

    let marked_read = response_json(
        router
            .clone()
            .oneshot(post_json(
                "/makosh.communications.v1.CommunicationsService/MarkMessageRead",
                json!({
                    "message_id": projected.message_id
                }),
            ))
            .await
            .expect("mark message read response"),
    )
    .await;
    assert_eq!(marked_read["messageId"], projected.message_id);
    assert_eq!(marked_read["markedRead"], true);
    assert_eq!(marked_read["workflowState"], "reviewed");

    let deleted_from_provider = response_json(
        router
            .clone()
            .oneshot(post_json(
                "/makosh.communications.v1.CommunicationsService/DeleteMessageFromProvider",
                json!({
                    "message_id": projected.message_id
                }),
            ))
            .await
            .expect("delete message from provider response"),
    )
    .await;
    assert_eq!(deleted_from_provider["messageId"], projected.message_id);
    assert_eq!(deleted_from_provider["deleted"], true);
    assert_eq!(deleted_from_provider["localState"], "trash");

    let restored_after_provider_delete = response_json(
        router
            .clone()
            .oneshot(post_json(
                "/makosh.communications.v1.CommunicationsService/RestoreMessage",
                json!({
                    "message_id": projected.message_id
                }),
            ))
            .await
            .expect("restore message after provider delete response"),
    )
    .await;
    assert_eq!(restored_after_provider_delete["localState"], "active");

    let bulk_action = response_json(
        router
            .clone()
            .oneshot(post_json(
                "/makosh.communications.v1.CommunicationsService/BulkMessageAction",
                json!({
                    "action": "trash",
                    "message_ids": [projected.message_id]
                }),
            ))
            .await
            .expect("bulk message action response"),
    )
    .await;
    assert_eq!(bulk_action["action"], "trash");
    assert_eq!(bulk_action["updatedCount"], 1);

    let restored_after_bu
```
_Source file truncated after 12000 characters. / Исходный файл обрезан после 12000 символов._

### `backend/tests/config.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/tests/config.rs`
- Size bytes / Размер в байтах: `11566`
- Included characters / Включено символов: `11566`
- Truncated / Обрезано: `no`

```rust
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::path::Path;

use makosh_hub_backend::platform::config::{AiRuntimeProvider, AppConfig, ConfigError};

#[test]
fn default_config_binds_to_localhost_without_database_url() {
    let config = AppConfig::default();

    assert_eq!(
        config.http_addr(),
        SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 8080)
    );
    assert_eq!(config.service_name(), "makosh-backend");
    assert_eq!(config.database_url(), None);
    assert_eq!(config.local_api_secret(), None);
    assert_eq!(config.secret_vault_path(), None);
    assert_eq!(config.secret_vault_key(), None);
    assert_eq!(config.tdjson_path(), None);
    assert!(config.zoom_token_maintenance_scheduler_enabled());
    assert!(config.zoom_recording_sync_scheduler_enabled());
    assert!(config.zoom_retention_cleanup_scheduler_enabled());
}

#[test]
fn config_from_pairs_overrides_http_addr_database_url_and_local_api_secret() {
    let config = AppConfig::from_pairs([
        ("MAKOSH_HTTP_ADDR", "127.0.0.1:9090"),
        (
            "DATABASE_URL",
            "postgres://makosh:local-dev-password@postgres:5432/makosh_hub",
        ),
        ("MAKOSH_LOCAL_API_SECRET", "local-dev-api-secret"),
    ])
    .expect("valid config");

    assert_eq!(
        config.http_addr(),
        SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 9090)
    );
    assert_eq!(
        config.database_url(),
        Some("postgres://makosh:local-dev-password@postgres:5432/makosh_hub")
    );
    assert_eq!(config.local_api_secret(), Some("local-dev-api-secret"));
}

#[test]
fn config_from_pairs_accepts_secret_vault_path_and_key() {
    let config = AppConfig::from_pairs([
        (
            "MAKOSH_SECRET_VAULT_PATH",
            "docker/data/secrets/makosh.vault.json",
        ),
        ("MAKOSH_SECRET_VAULT_KEY", "local-vault-key"),
    ])
    .expect("valid secret vault config");

    assert_eq!(
        config.secret_vault_path(),
        Some(Path::new("docker/data/secrets/makosh.vault.json"))
    );
    assert_eq!(
        config
            .secret_vault_key()
            .expect("vault key")
            .expose_for_runtime(),
        "local-vault-key"
    );
    assert_eq!(
        format!("{:?}", config.secret_vault_key().expect("vault key")),
        "ResolvedSecret { value: \"<redacted>\" }"
    );
}

#[test]
fn config_from_pairs_accepts_ollama_runtime_overrides() {
    let config = AppConfig::from_pairs([
        ("MAKOSH_OLLAMA_BASE_URL", "http://192.168.1.2:11434"),
        ("MAKOSH_OLLAMA_CHAT_MODEL", "qwen3:4b"),
        ("MAKOSH_OLLAMA_EMBED_MODEL", "qwen3-embedding:4b"),
        ("MAKOSH_OLLAMA_TIMEOUT_SECONDS", "120"),
    ])
    .expect("valid Ollama config");

    assert_eq!(config.ollama_base_url(), "http://192.168.1.2:11434");
    assert_eq!(config.ollama_chat_model(), "qwen3:4b");
    assert_eq!(config.ollama_embed_model(), "qwen3-embedding:4b");
    assert_eq!(config.ollama_timeout_seconds(), 120);
}

#[test]
fn config_from_pairs_accepts_omniroute_runtime_overrides_without_printing_key() {
    let config = AppConfig::from_pairs([
        ("MAKOSH_AI_PROVIDER", "omniroute"),
        ("MAKOSH_OMNIROUTE_BASE_URL", "https://ai.sh-inc.ru/v1/"),
        ("MAKOSH_OMNIROUTE_CHAT_MODEL", "codex/gpt-5.5"),
        (
            "MAKOSH_OMNIROUTE_EMBED_MODEL",
            "openai-compatible-chat-ollama-pve/qwen3-embedding:4b",
        ),
        ("MAKOSH_OMNIROUTE_TIMEOUT_SECONDS", "90"),
        ("MAKOSH_OMNIROUTE_API_KEY", "omniroute-test-key"),
    ])
    .expect("valid OmniRoute config");

    assert_eq!(config.ai_provider(), AiRuntimeProvider::OmniRoute);
    assert_eq!(config.omniroute_base_url(), "https://ai.sh-inc.ru/v1");
    assert_eq!(config.omniroute_chat_model(), "codex/gpt-5.5");
    assert_eq!(
        config.omniroute_embed_model(),
        "openai-compatible-chat-ollama-pve/qwen3-embedding:4b"
    );
    assert_eq!(config.omniroute_timeout_seconds(), 90);
    assert_eq!(
        config
            .omniroute_api_key()
            .expect("OmniRoute API key")
            .expose_for_runtime(),
        "omniroute-test-key"
    );
    assert_eq!(
        format!(
            "{:?}",
            config.omniroute_api_key().expect("OmniRoute API key")
        ),
        "ResolvedSecret { value: \"<redacted>\" }"
    );
}

#[test]
fn config_from_pairs_accepts_tdjson_runtime_path() {
    let config =
        AppConfig::from_pairs([("MAKOSH_TDJSON_PATH", "/opt/homebrew/lib/libtdjson.dylib")])
            .expect("valid TDLib JSON runtime config");

    assert_eq!(
        config.tdjson_path(),
        Some(Path::new("/opt/homebrew/lib/libtdjson.dylib"))
    );
}

#[test]
fn config_from_pairs_accepts_telegram_app_credentials() {
    let config = AppConfig::from_pairs([
        ("MAKOSH_TELEGRAM_API_ID", "12345"),
        ("MAKOSH_TELEGRAM_API_HASH", "telegram-api-hash"),
    ])
    .expect("valid Telegram app credential config");

    assert_eq!(config.telegram_api_id(), Some(12345));
    assert_eq!(
        config
            .telegram_api_hash()
            .expect("Telegram API hash")
            .expose_for_runtime(),
        "telegram-api-hash"
    );
    assert_eq!(
        format!(
            "{:?}",
            config.telegram_api_hash().expect("Telegram API hash")
        ),
        "ResolvedSecret { value: \"<redacted>\" }"
    );
}

#[test]
fn config_from_pairs_accepts_zoom_token_maintenance_scheduler_toggle() {
    let config =
        AppConfig::from_pairs([("MAKOSH_ZOOM_TOKEN_MAINTENANCE_SCHEDULER_ENABLED", "false")])
            .expect("valid Zoom token maintenance scheduler config");

    assert!(!config.zoom_token_maintenance_scheduler_enabled());
}

#[test]
fn config_from_pairs_accepts_zoom_recording_sync_scheduler_toggle() {
    let config = AppConfig::from_pairs([("MAKOSH_ZOOM_RECORDING_SYNC_SCHEDULER_ENABLED", "false")])
        .expect("valid Zoom recording sync scheduler config");

    assert!(!config.zoom_recording_sync_scheduler_enabled());
}

#[test]
fn config_from_pairs_accepts_zoom_retention_cleanup_scheduler_toggle() {
    let config =
        AppConfig::from_pairs([("MAKOSH_ZOOM_RETENTION_CLEANUP_SCHEDULER_ENABLED", "false")])
            .expect("valid Zoom retention cleanup scheduler config");

    assert!(!config.zoom_retention_cleanup_scheduler_enabled());
}

#[test]
fn default_config_uses_local_ollama_and_qwen_models() {
    let config = AppConfig::default();

    assert_eq!(config.ai_provider(), AiRuntimeProvider::Ollama);
    assert_eq!(config.ollama_base_url(), "http://127.0.0.1:11434");
    assert_eq!(config.ollama_chat_model(), "qwen3:4b");
    assert_eq!(config.ollama_embed_model(), "qwen3-embedding:4b");
    assert_eq!(config.ollama_timeout_seconds(), 120);
    assert_eq!(config.omniroute_base_url(), "https://ai.sh-inc.ru/v1");
    assert_eq!(config.omniroute_chat_model(), "codex/gpt-5.5");
    assert_eq!(
        config.omniroute_embed_model(),
        "openai-compatible-chat-ollama-pve/qwen3-embedding:4b"
    );
    assert_eq!(config.omniroute_timeout_seconds(), 120);
    assert_eq!(config.omniroute_api_key(), None);
}

#[test]
fn config_from_pairs_rejects_invalid_http_addr() {
    let error = AppConfig::from_pairs([("MAKOSH_HTTP_ADDR", "not-a-socket")])
        .expect_err("invalid socket address must fail");

    assert!(matches!(error, ConfigError::InvalidHttpAddr { .. }));
}

#[test]
fn config_from_pairs_rejects_empty_database_url() {
    let error =
        AppConfig::from_pairs([("DATABASE_URL", "   ")]).expect_err("empty database URL must fail");

    assert!(matches!(error, ConfigError::EmptyDatabaseUrl));
}

#[test]
fn config_from_pairs_rejects_empty_local_api_secret() {
    let error = AppConfig::from_pairs([("MAKOSH_LOCAL_API_SECRET", "   ")])
        .expect_err("empty local API secret must fail");

    assert!(matches!(error, ConfigError::EmptyLocalApiSecret));
}

#[test]
fn config_from_pairs_rejects_empty_secret_vault_path() {
    let error = AppConfig::from_pairs([("MAKOSH_SECRET_VAULT_PATH", "   ")])
        .expect_err("empty secret vault path must fail");

    assert!(matches!(error, ConfigError::EmptySecretVaultPath));
}

#[test]
fn config_from_pairs_rejects_empty_secret_vault_key() {
    let error = AppConfig::from_pairs([("MAKOSH_SECRET_VAULT_KEY", "   ")])
        .expect_err("empty secret vault key must fail");

    assert!(matches!(error, ConfigError::EmptySecretVaultKey));
}

#[test]
fn config_from_pairs_rejects_empty_tdjson_path() {
    let error = AppConfig::from_pairs([("MAKOSH_TDJSON_PATH", "   ")])
        .expect_err("empty TDLib JSON runtime path must fail");

    assert!(matches!(error, ConfigError::EmptyTdjsonPath));
}

#[test]
fn config_from_pairs_rejects_invalid_telegram_app_credentials() {
    let error = AppConfig::from_pairs([("MAKOSH_TELEGRAM_API_ID", "0")])
        .expect_err("zero Telegram API ID must fail");
    assert!(matches!(error, ConfigError::InvalidTelegramApiId { .. }));

    let error = AppConfig::from_pairs([("MAKOSH_TELEGRAM_API_ID", "not-a-number")])
        .expect_err("non-numeric Telegram API ID must fail");
    assert!(matches!(error, ConfigError::InvalidTelegramApiId { .. }));

    let error = AppConfig::from_pairs([("MAKOSH_TELEGRAM_API_HASH", "   ")])
        .expect_err("empty Telegram API hash must fail");
    assert!(matches!(error, ConfigError::EmptyTelegramApiHash));
}

#[test]
fn config_from_pairs_rejects_invalid_ollama_values() {
    let error = AppConfig::from_pairs([("MAKOSH_OLLAMA_BASE_URL", "   ")])
        .expect_err("empty Ollama base URL must fail");
    assert!(matches!(error, ConfigError::EmptyOllamaBaseUrl));

    let error = AppConfig::from_pairs([("MAKOSH_OLLAMA_CHAT_MODEL", "   ")])
        .expect_err("empty Ollama chat model must fail");
    assert!(matches!(error, ConfigError::EmptyOllamaChatModel));

    let error = AppConfig::from_pairs([("MAKOSH_OLLAMA_EMBED_MODEL", "   ")])
        .expect_err("empty Ollama embed model must fail");
    assert!(matches!(error, ConfigError::EmptyOllamaEmbedModel));

    let error = AppConfig::from_pairs([("MAKOSH_OLLAMA_TIMEOUT_SECONDS", "0")])
        .expect_err("zero Ollama timeout must fail");
    assert!(matches!(error, ConfigError::InvalidOllamaTimeout { .. }));
}

#[test]
fn config_from_pairs_rejects_invalid_omniroute_values() {
    let error = AppConfig::from_pairs([("MAKOSH_AI_PROVIDER", "cloudy")])
        .expect_err("unknown AI provider must fail");
    assert!(matches!(error, ConfigError::InvalidAiProvider { .. }));

    let error = AppConfig::from_pairs([("MAKOSH_OMNIROUTE_BASE_URL", "   ")])
        .expect_err("empty OmniRoute base URL must fail");
    assert!(matches!(error, ConfigError::EmptyOmniRouteBaseUrl));

    let error = AppConfig::from_pairs([("MAKOSH_OMNIROUTE_CHAT_MODEL", "   ")])
        .expect_err("empty OmniRoute chat model must fail");
    assert!(matches!(error, ConfigError::EmptyOmniRouteChatModel));

    let error = AppConfig::from_pairs([("MAKOSH_OMNIROUTE_EMBED_MODEL", "   ")])
        .expect_err("empty OmniRoute embed model must fail");
    assert!(matches!(error, ConfigError::EmptyOmniRouteEmbedModel));

    let error = AppConfig::from_pairs([("MAKOSH_OMNIROUTE_TIMEOUT_SECONDS", "0")])
        .expect_err("zero OmniRoute timeout must fail");
    assert!(matches!(error, ConfigError::InvalidOmniRouteTimeout { .. }));

    let error = AppConfig::from_pairs([("MAKOSH_OMNIROUTE_API_KEY", "   ")])
        .expect_err("empty OmniRoute API key must fail");
    assert!(matches!(error, ConfigError::EmptyOmniRouteApiKey));
}
```

### `backend/tests/consistency_contradiction.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/tests/consistency_contradiction.rs`
- Size bytes / Размер в байтах: `482`
- Included characters / Включено символов: `482`
- Truncated / Обрезано: `no`

```rust
#[path = "consistency_contradiction/engine.rs"]
mod engine;
#[path = "consistency_contradiction/observation_store.rs"]
mod observation_store;
#[path = "consistency_contradiction/refresh_event_call.rs"]
mod refresh_event_call;
#[path = "consistency_contradiction/refresh_message_document.rs"]
mod refresh_message_document;
#[path = "consistency_contradiction/refresh_provider_messages.rs"]
mod refresh_provider_messages;
#[path = "consistency_contradiction/support.rs"]
mod support;
```

### `backend/tests/consistency_contradiction/engine.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/tests/consistency_contradiction/engine.rs`
- Size bytes / Размер в байтах: `7515`
- Included characters / Включено символов: `7515`
- Truncated / Обрезано: `no`

```rust
use makosh_hub_backend::engines::consistency::{
    AcceptedClaim, ConsistencyEngine, ContradictionReviewState, ContradictionSeverity,
    ContradictionSourceKind, EvidenceClaimExtractionInput, NewEvidenceClaim,
};
use serde_json::json;

#[test]
fn consistency_engine_detects_direct_claim_contradiction_from_structured_claims() {
    let accepted = AcceptedClaim {
        subject_id: "person:v1:email:alex@example.com".to_owned(),
        claim_type: "location".to_owned(),
        value: "Berlin".to_owned(),
        source_kind: ContradictionSourceKind::Memory,
        source_id: "person_fact:location:alex".to_owned(),
        confidence: 0.95,
    };
    let new_claim = NewEvidenceClaim {
        subject_id: "person:v1:email:alex@example.com".to_owned(),
        claim_type: "location".to_owned(),
        value: "Madrid".to_owned(),
        source_kind: ContradictionSourceKind::Communication,
        source_id: "message:location-update".to_owned(),
        confidence: 0.87,
    };

    let observations = ConsistencyEngine::detect_claim_contradictions(&[accepted], &[new_claim])
        .expect("detect contradictions");

    assert_eq!(observations.len(), 1);
    let observation = &observations[0];
    assert_eq!(observation.old_source_kind, ContradictionSourceKind::Memory);
    assert_eq!(observation.old_source_id, "person_fact:location:alex");
    assert_eq!(
        observation.new_source_kind,
        ContradictionSourceKind::Communication
    );
    assert_eq!(observation.new_source_id, "message:location-update");
    assert_eq!(observation.conflict_type, "direct_contradiction");
    assert_eq!(observation.old_claim, "location=Berlin");
    assert_eq!(observation.new_claim, "location=Madrid");
    assert_eq!(observation.confidence, 0.87);
    assert_eq!(observation.severity, ContradictionSeverity::Medium);
    assert_eq!(
        observation.review_state,
        ContradictionReviewState::Suggested
    );
    assert_eq!(
        observation.affected_entities,
        json!([{"entity_kind": "subject", "entity_id": "person:v1:email:alex@example.com"}])
    );
}

#[test]
fn consistency_engine_ignores_matching_claims_after_normalization() {
    let accepted = AcceptedClaim {
        subject_id: "project:v1:makosh".to_owned(),
        claim_type: "status".to_owned(),
        value: " Active ".to_owned(),
        source_kind: ContradictionSourceKind::Knowledge,
        source_id: "knowledge:project-status".to_owned(),
        confidence: 0.9,
    };
    let new_claim = NewEvidenceClaim {
        subject_id: "project:v1:makosh".to_owned(),
        claim_type: "status".to_owned(),
        value: "active".to_owned(),
        source_kind: ContradictionSourceKind::Communication,
        source_id: "message:project-status".to_owned(),
        confidence: 0.8,
    };

    let observations = ConsistencyEngine::detect_claim_contradictions(&[accepted], &[new_claim])
        .expect("detect contradictions");

    assert_eq!(observations, Vec::new());
}

#[test]
fn consistency_engine_extracts_structured_claims_from_communication_evidence() {
    let input = EvidenceClaimExtractionInput {
        subject_id: "person:v1:email:alex@example.com".to_owned(),
        source_kind: ContradictionSourceKind::Communication,
        source_id: "message:claim-extraction".to_owned(),
        text: "Location: Madrid\nStatus = active\nNotes without claim\nEmpty:".to_owned(),
        confidence: 0.81,
    };

    let claims =
        ConsistencyEngine::extract_evidence_claims(&input).expect("extract evidence claims");

    assert_eq!(
        claims,
        vec![
            NewEvidenceClaim {
                subject_id: "person:v1:email:alex@example.com".to_owned(),
                claim_type: "location".to_owned(),
                value: "Madrid".to_owned(),
                source_kind: ContradictionSourceKind::Communication,
                source_id: "message:claim-extraction".to_owned(),
                confidence: 0.81,
            },
            NewEvidenceClaim {
                subject_id: "person:v1:email:alex@example.com".to_owned(),
                claim_type: "status".to_owned(),
                value: "active".to_owned(),
                source_kind: ContradictionSourceKind::Communication,
                source_id: "message:claim-extraction".to_owned(),
                confidence: 0.81,
            },
        ]
    );
}

#[test]
fn consistency_engine_extracts_deterministic_natural_language_claims_from_evidence() {
    let input = EvidenceClaimExtractionInput {
        subject_id: "person:v1:email:alex@example.com".to_owned(),
        source_kind: ContradictionSourceKind::Communication,
        source_id: "message:natural-language-claim-extraction".to_owned(),
        text: "Quick update: I am now in Madrid.\nThe project status is blocked.".to_owned(),
        confidence: 0.79,
    };

    let claims =
        ConsistencyEngine::extract_evidence_claims(&input).expect("extract evidence claims");

    assert_eq!(
        claims,
        vec![
            NewEvidenceClaim {
                subject_id: "person:v1:email:alex@example.com".to_owned(),
                claim_type: "location".to_owned(),
                value: "Madrid".to_owned(),
                source_kind: ContradictionSourceKind::Communication,
                source_id: "message:natural-language-claim-extraction".to_owned(),
                confidence: 0.79,
            },
            NewEvidenceClaim {
                subject_id: "person:v1:email:alex@example.com".to_owned(),
                claim_type: "status".to_owned(),
                value: "blocked".to_owned(),
                source_kind: ContradictionSourceKind::Communication,
                source_id: "message:natural-language-claim-extraction".to_owned(),
                confidence: 0.79,
            },
        ]
    );
}

#[test]
fn consistency_engine_detects_document_evidence_contradiction_after_claim_extraction() {
    let accepted = AcceptedClaim {
        subject_id: "project:v1:makosh".to_owned(),
        claim_type: "status".to_owned(),
        value: "green".to_owned(),
        source_kind: ContradictionSourceKind::Memory,
        source_id: "memory:project-status".to_owned(),
        confidence: 0.92,
    };
    let document = EvidenceClaimExtractionInput {
        subject_id: "project:v1:makosh".to_owned(),
        source_kind: ContradictionSourceKind::Document,
        source_id: "document:weekly-report".to_owned(),
        text: "Status: blocked".to_owned(),
        confidence: 0.84,
    };

    let observations = ConsistencyEngine::detect_evidence_contradictions(&[accepted], &[document])
        .expect("detect evidence contradictions");

    assert_eq!(observations.len(), 1);
    let observation = &observations[0];
    assert_eq!(observation.old_source_kind, ContradictionSourceKind::Memory);
    assert_eq!(observation.old_source_id, "memory:project-status");
    assert_eq!(
        observation.new_source_kind,
        ContradictionSourceKind::Document
    );
    assert_eq!(observation.new_source_id, "document:weekly-report");
    assert_eq!(observation.old_claim, "status=green");
    assert_eq!(observation.new_claim, "status=blocked");
    assert_eq!(observation.confidence, 0.84);
    assert_eq!(observation.severity, ContradictionSeverity::Medium);
    assert_eq!(
        observation.metadata,
        json!({
            "detector": "structured_evidence_claim",
            "claim_type": "status",
            "source_kind": "document"
        })
    );
}
```

### `backend/tests/consistency_contradiction/observation_store.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/tests/consistency_contradiction/observation_store.rs`
- Size bytes / Размер в байтах: `3529`
- Included characters / Включено символов: `3529`
- Truncated / Обрезано: `no`

```rust
use makosh_hub_backend::engines::consistency::{
    ContradictionObservationStore, ContradictionReviewState, ContradictionSeverity,
    ContradictionSourceKind, NewContradictionObservation,
};
use serde_json::json;

use super::support::{live_consistency_pool, unique_suffix};

#[tokio::test]
async fn contradiction_observation_store_upserts_reviewable_observation_against_postgres() {
    let Some(pool) = live_consistency_pool("contradiction observation").await else {
        return;
    };
    let store = ContradictionObservationStore::new(pool);
    let suffix = unique_suffix();
    let observation = NewContradictionObservation {
        old_source_kind: ContradictionSourceKind::Memory,
        old_source_id: format!("memory:budget:{suffix}"),
        new_source_kind: ContradictionSourceKind::Communication,
        new_source_id: format!("message:budget:{suffix}"),
        affected_entities: json!([
            {"entity_kind": "project", "entity_id": format!("project:v1:{suffix}")}
        ]),
        conflict_type: "direct_contradiction".to_owned(),
        old_claim: "budget=approved".to_owned(),
        new_claim: "budget=rejected".to_owned(),
        confidence: 0.88,
        severity: ContradictionSeverity::High,
        review_state: ContradictionReviewState::Suggested,
        metadata: json!({"detector": "structured_claim_test"}),
    };

    let first = store
        .upsert(&observation)
        .await
        .expect("first contradiction upsert");
    let second = store
        .upsert(&observation)
        .await
        .expect("idempotent contradiction upsert");

    assert_eq!(first.observation_id, second.observation_id);
    assert_eq!(first.review_state, ContradictionReviewState::Suggested);
    assert_eq!(first.severity, ContradictionSeverity::High);
    assert_eq!(first.confidence, 0.88);

    let open = store.list_open(20).await.expect("open contradictions");
    assert!(
        open.iter()
            .any(|item| item.observation_id == first.observation_id)
    );

    let reviewed = store
        .set_review_state(
            &first.observation_id,
            ContradictionReviewState::UserConfirmed,
            "test-reviewer",
            Some("confirmed contradiction"),
        )
        .await
        .expect("review contradiction");

    assert_eq!(
        reviewed.review_state,
        ContradictionReviewState::UserConfirmed
    );
    assert_eq!(reviewed.reviewed_by.as_deref(), Some("test-reviewer"));
    assert_eq!(
        reviewed.resolution.as_deref(),
        Some("confirmed contradiction")
    );
}

#[test]
fn contradiction_observation_rejects_invalid_confidence_before_database_write() {
    let observation = NewContradictionObservation {
        old_source_kind: ContradictionSourceKind::Memory,
        old_source_id: "memory:invalid".to_owned(),
        new_source_kind: ContradictionSourceKind::Communication,
        new_source_id: "message:invalid".to_owned(),
        affected_entities: json!([]),
        conflict_type: "direct_contradiction".to_owned(),
        old_claim: "status=active".to_owned(),
        new_claim: "status=archived".to_owned(),
        confidence: 1.2,
        severity: ContradictionSeverity::Medium,
        review_state: ContradictionReviewState::Suggested,
        metadata: json!({}),
    };

    let error = observation
        .validate()
        .expect_err("invalid confidence must be rejected");

    assert_eq!(
        error.to_string(),
        "confidence must be between 0.0 and 1.0: 1.2"
    );
}
```

### `backend/tests/consistency_contradiction/refresh_event_call.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/tests/consistency_contradiction/refresh_event_call.rs`
- Size bytes / Размер в байтах: `9000`
- Included characters / Включено символов: `9000`
- Truncated / Обрезано: `no`

```rust
use chrono::{Duration, Utc};
use makosh_hub_backend::domains::calendar::events::{CalendarEventStore, NewCalendarEvent};
use makosh_hub_backend::domains::calendar::meetings::MeetingNoteStore;
use makosh_hub_backend::domains::communications::core::{
    CommunicationIngestionStore, EmailProviderKind, NewProviderAccount,
};
use makosh_hub_backend::domains::persons::api::PersonProjectionStore;
use makosh_hub_backend::engines::consistency::{
    ContradictionObservationStore, ContradictionSeverity, ContradictionSourceKind,
};
use makosh_hub_backend::platform::calls::{
    CallDirection, CallIntelligenceStore, CallState, NewCallTranscript, NewTelegramCall,
    TranscriptStatus,
};
use serde_json::json;

use super::support::{live_consistency_pool, unique_suffix};

#[tokio::test]
async fn contradiction_refresh_detects_meeting_note_claim_against_active_person_fact_without_overwriting_memory()
 {
    let Some(pool) = live_consistency_pool("contradiction meeting note refresh").await else {
        return;
    };
    let store = ContradictionObservationStore::new(pool.clone());
    let suffix = unique_suffix();
    let email_address = format!("polygraph-meeting-{suffix}@example.com");
    let person = PersonProjectionStore::new(pool.clone())
        .upsert_email_person(&email_address)
        .await
        .expect("person");
    let fact_id: String = sqlx::query_scalar(
        r#"
        INSERT INTO person_facts (person_id, fact_type, value, source, confidence)
        VALUES ($1, 'location', 'Berlin', 'manual', 0.93)
        RETURNING id::text
        "#,
    )
    .bind(&person.person_id)
    .fetch_one(&pool)
    .await
    .expect("person fact");
    let start_at = Utc::now();
    let event = CalendarEventStore::new(pool.clone())
        .create(&NewCalendarEvent {
            title: format!("Polygraph meeting {suffix}"),
            description: Some("Meeting evidence for consistency refresh".to_owned()),
            start_at,
            end_at: start_at + Duration::minutes(30),
            event_type: Some("meeting".to_owned()),
            ..NewCalendarEvent::default()
        })
        .await
        .expect("calendar event");
    sqlx::query(
        r#"
        INSERT INTO event_participants (event_id, email, display_name, role, person_id)
        VALUES ($1, $2, 'Polygraph Participant', 'attendee', $3)
        "#,
    )
    .bind(&event.event_id)
    .bind(&email_address)
    .bind(&person.person_id)
    .execute(&pool)
    .await
    .expect("event participant");
    let note = MeetingNoteStore::new(pool.clone())
        .create(
            &event.event_id,
            "Location: Madrid",
            Some("markdown"),
            Some("manual"),
        )
        .await
        .expect("meeting note");

    let refreshed = store
        .refresh_deterministic_observations(100)
        .await
        .expect("refresh contradictions");
    assert!(refreshed >= 1);

    let open = store.list_open(100).await.expect("open contradictions");
    let observation = open
        .iter()
        .find(|item| item.old_source_id == fact_id && item.new_source_id == note.id)
        .expect("meeting note claim should contradict active remembered person fact");

    assert_eq!(observation.old_source_kind, ContradictionSourceKind::Memory);
    assert_eq!(observation.new_source_kind, ContradictionSourceKind::Event);
    assert_eq!(observation.conflict_type, "direct_contradiction");
    assert_eq!(observation.old_claim, "location=Berlin");
    assert_eq!(observation.new_claim, "location=Madrid");
    assert_eq!(observation.confidence, 0.8);
    assert_eq!(observation.severity, ContradictionSeverity::Medium);
    assert_eq!(
        observation.affected_entities,
        json!([{"entity_kind": "subject", "entity_id": person.person_id}])
    );
    assert_eq!(
        observation.metadata,
        json!({
            "detector": "structured_evidence_claim",
            "claim_type": "location",
            "source_kind": "event"
        })
    );

    let remembered_value: String =
        sqlx::query_scalar("SELECT value FROM person_facts WHERE id::text = $1")
            .bind(&fact_id)
            .fetch_one(&pool)
            .await
            .expect("remembered value");
    assert_eq!(remembered_value, "Berlin");
}

#[tokio::test]
async fn contradiction_refresh_detects_call_transcript_claim_against_active_person_fact_without_overwriting_memory()
 {
    let Some(pool) = live_consistency_pool("contradiction call transcript refresh").await else {
        return;
    };
    let store = ContradictionObservationStore::new(pool.clone());
    let suffix = unique_suffix();
    let email_address = format!("polygraph-call-{suffix}@example.com");
    let provider_chat_id = format!("telegram-chat-{suffix}");
    let person = PersonProjectionStore::new(pool.clone())
        .upsert_email_person(&email_address)
        .await
        .expect("person");
    sqlx::query(
        r#"
        INSERT INTO person_identities (person_id, identity_type, identity_value, source, confidence, status)
        VALUES ($1, 'telegram', $2, 'test', 1.0, 'active')
        "#,
    )
    .bind(&person.person_id)
    .bind(&provider_chat_id)
    .execute(&pool)
    .await
    .expect("telegram identity");
    let fact_id: String = sqlx::query_scalar(
        r#"
        INSERT INTO person_facts (person_id, fact_type, value, source, confidence)
        VALUES ($1, 'location', 'Berlin', 'manual', 0.93)
        RETURNING id::text
        "#,
    )
    .bind(&person.person_id)
    .fetch_one(&pool)
    .await
    .expect("person fact");
    let account_id = format!("acct_polygraph_call_{suffix}");
    CommunicationIngestionStore::new(pool.clone())
        .upsert_provider_account(&NewProviderAccount::new(
            &account_id,
            EmailProviderKind::Gmail,
            "Polygraph Call Compatibility Account",
            format!("polygraph-call-{suffix}@example.com"),
        ))
        .await
        .expect("provider account");
    let call_store = CallIntelligenceStore::new(pool.clone());
    let call_id = format!("call:polygraph:{suffix}");
    call_store
        .upsert_call(&NewTelegramCall {
            call_id: call_id.clone(),
            account_id: account_id.clone(),
            provider_call_id: format!("provider-call-{suffix}"),
            provider_chat_id: provider_chat_id.clone(),
            direction: CallDirection::Incoming,
            call_state: CallState::Ended,
            started_at: Some(Utc::now()),
            ended_at: Some(Utc::now()),
            transcription_policy_id: None,
            metadata: json!({"source": "polygraph_test"}),
        })
        .await
        .expect("telegram call");
    let transcript_id = format!("call-transcript-polygraph-{suffix}");
    call_store
        .upsert_transcript(&NewCallTranscript {
            transcript_id: transcript_id.clone(),
            call_id,
            account_id,
            provider_chat_id,
            transcript_status: TranscriptStatus::Succeeded,
            stt_provider: "fixture-stt".to_owned(),
            source_audio_ref: Some(format!("audio-polygraph-{suffix}")),
            language_code: Some("en".to_owned()),
            transcript_text: "Location: Madrid".to_owned(),
            segments: json!([]),
            provenance: json!({"source": "polygraph_test"}),
        })
        .await
        .expect("call transcript");

    let refreshed = store
        .refresh_deterministic_observations(100)
        .await
        .expect("refresh contradictions");
    assert!(refreshed >= 1);

    let open = store.list_open(100).await.expect("open contradictions");
    let observation = open
        .iter()
        .find(|item| item.old_source_id == fact_id && item.new_source_id == transcript_id)
        .expect("call transcript claim should contradict active remembered person fact");

    assert_eq!(observation.old_source_kind, ContradictionSourceKind::Memory);
    assert_eq!(
        observation.new_source_kind,
        ContradictionSourceKind::Communication
    );
    assert_eq!(observation.conflict_type, "direct_contradiction");
    assert_eq!(observation.old_claim, "location=Berlin");
    assert_eq!(observation.new_claim, "location=Madrid");
    assert_eq!(observation.confidence, 0.8);
    assert_eq!(observation.severity, ContradictionSeverity::Medium);
    assert_eq!(
        observation.affected_entities,
        json!([{"entity_kind": "subject", "entity_id": person.person_id}])
    );
    assert_eq!(
        observation.metadata,
        json!({
            "detector": "structured_evidence_claim",
            "claim_type": "location",
            "source_kind": "communication"
        })
    );

    let remembered_value: String =
        sqlx::query_scalar("SELECT value FROM person_facts WHERE id::text = $1")
            .bind(&fact_id)
            .fetch_one(&pool)
            .await
            .expect("remembered value");
    assert_eq!(remembered_value, "Berlin");
}
```

### `backend/tests/consistency_contradiction/refresh_message_document.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/tests/consistency_contradiction/refresh_message_document.rs`
- Size bytes / Размер в байтах: `8419`
- Included characters / Включено символов: `8419`
- Truncated / Обрезано: `no`

```rust
use makosh_hub_backend::domains::documents::core::{DocumentImportStore, NewDocumentImport};
use makosh_hub_backend::domains::persons::api::PersonProjectionStore;
use makosh_hub_backend::engines::consistency::{
    ContradictionObservationStore, ContradictionSeverity, ContradictionSourceKind,
};
use serde_json::json;

use super::support::{live_consistency_pool, seed_message, unique_suffix};

#[tokio::test]
async fn contradiction_refresh_detects_message_claim_against_active_person_fact_without_overwriting_memory()
 {
    let Some(pool) = live_consistency_pool("contradiction refresh").await else {
        return;
    };
    let store = ContradictionObservationStore::new(pool.clone());
    let suffix = unique_suffix();
    let sender = format!("polygraph-{suffix}@example.com");
    let person = PersonProjectionStore::new(pool.clone())
        .upsert_email_person(&sender)
        .await
        .expect("person");
    let fact_id: String = sqlx::query_scalar(
        r#"
        INSERT INTO person_facts (person_id, fact_type, value, source, confidence)
        VALUES ($1, 'location', 'Berlin', 'manual', 0.93)
        RETURNING id::text
        "#,
    )
    .bind(&person.person_id)
    .fetch_one(&pool)
    .await
    .expect("person fact");
    let message_id = seed_message(
        &pool,
        suffix,
        &sender,
        &[format!("owner-{suffix}@example.com")],
        &format!("provider-polygraph-{suffix}"),
        &format!("Location update {suffix}"),
        "Location: Madrid",
    )
    .await;

    let refreshed = store
        .refresh_deterministic_observations(100)
        .await
        .expect("refresh contradictions");
    assert!(refreshed >= 1);

    let open = store.list_open(100).await.expect("open contradictions");
    let observation = open
        .iter()
        .find(|item| item.old_source_id == fact_id && item.new_source_id == message_id)
        .expect("message claim should contradict active remembered person fact");

    assert_eq!(observation.old_source_kind, ContradictionSourceKind::Memory);
    assert_eq!(
        observation.new_source_kind,
        ContradictionSourceKind::Communication
    );
    assert_eq!(observation.conflict_type, "direct_contradiction");
    assert_eq!(observation.old_claim, "location=Berlin");
    assert_eq!(observation.new_claim, "location=Madrid");
    assert_eq!(observation.confidence, 0.8);
    assert_eq!(observation.severity, ContradictionSeverity::Medium);
    assert_eq!(
        observation.affected_entities,
        json!([{"entity_kind": "subject", "entity_id": person.person_id}])
    );
    assert_eq!(
        observation.metadata,
        json!({
            "detector": "structured_evidence_claim",
            "claim_type": "location",
            "source_kind": "communication"
        })
    );

    let remembered_value: String =
        sqlx::query_scalar("SELECT value FROM person_facts WHERE id::text = $1")
            .bind(&fact_id)
            .fetch_one(&pool)
            .await
            .expect("remembered value");
    assert_eq!(remembered_value, "Berlin");
}

#[tokio::test]
async fn contradiction_refresh_detects_natural_language_message_claim_against_active_person_fact_without_overwriting_memory()
 {
    let Some(pool) = live_consistency_pool("contradiction natural-language refresh").await else {
        return;
    };
    let store = ContradictionObservationStore::new(pool.clone());
    let suffix = unique_suffix();
    let sender = format!("polygraph-natural-language-{suffix}@example.com");
    let person = PersonProjectionStore::new(pool.clone())
        .upsert_email_person(&sender)
        .await
        .expect("person");
    let fact_id: String = sqlx::query_scalar(
        r#"
        INSERT INTO person_facts (person_id, fact_type, value, source, confidence)
        VALUES ($1, 'location', 'Berlin', 'manual', 0.93)
        RETURNING id::text
        "#,
    )
    .bind(&person.person_id)
    .fetch_one(&pool)
    .await
    .expect("person fact");
    let message_id = seed_message(
        &pool,
        suffix,
        &sender,
        &[format!("owner-natural-language-{suffix}@example.com")],
        &format!("provider-polygraph-natural-language-{suffix}"),
        &format!("Natural language location update {suffix}"),
        "Quick update: I am now in Madrid.",
    )
    .await;

    let refreshed = store
        .refresh_deterministic_observations(100)
        .await
        .expect("refresh contradictions");
    assert!(refreshed >= 1);

    let open = store.list_open(100).await.expect("open contradictions");
    let observation = open
        .iter()
        .find(|item| item.old_source_id == fact_id && item.new_source_id == message_id)
        .expect("natural-language message claim should contradict active remembered person fact");

    assert_eq!(observation.old_source_kind, ContradictionSourceKind::Memory);
    assert_eq!(
        observation.new_source_kind,
        ContradictionSourceKind::Communication
    );
    assert_eq!(observation.conflict_type, "direct_contradiction");
    assert_eq!(observation.old_claim, "location=Berlin");
    assert_eq!(observation.new_claim, "location=Madrid");
    assert_eq!(observation.confidence, 0.8);
    assert_eq!(observation.severity, ContradictionSeverity::Medium);

    let remembered_value: String =
        sqlx::query_scalar("SELECT value FROM person_facts WHERE id::text = $1")
            .bind(&fact_id)
            .fetch_one(&pool)
            .await
            .expect("remembered value");
    assert_eq!(remembered_value, "Berlin");
}

#[tokio::test]
async fn contradiction_refresh_detects_document_claim_against_active_person_fact_without_overwriting_memory()
 {
    let Some(pool) = live_consistency_pool("contradiction document refresh").await else {
        return;
    };
    let store = ContradictionObservationStore::new(pool.clone());
    let suffix = unique_suffix();
    let email_address = format!("polygraph-document-{suffix}@example.com");
    let person = PersonProjectionStore::new(pool.clone())
        .upsert_email_person(&email_address)
        .await
        .expect("person");
    let fact_id: String = sqlx::query_scalar(
        r#"
        INSERT INTO person_facts (person_id, fact_type, value, source, confidence)
        VALUES ($1, 'location', 'Berlin', 'manual', 0.93)
        RETURNING id::text
        "#,
    )
    .bind(&person.person_id)
    .fetch_one(&pool)
    .await
    .expect("person fact");
    let document_id = format!("document_polygraph_{suffix}");
    DocumentImportStore::new(pool.clone())
        .import_document(&NewDocumentImport::markdown(
            &document_id,
            format!("Persona dossier {suffix}"),
            format!("# Persona dossier\nEmail: {email_address}\nLocation: Madrid"),
        ))
        .await
        .expect("document import");

    let refreshed = store
        .refresh_deterministic_observations(100)
        .await
        .expect("refresh contradictions");
    assert!(refreshed >= 1);

    let open = store.list_open(100).await.expect("open contradictions");
    let observation = open
        .iter()
        .find(|item| item.old_source_id == fact_id && item.new_source_id == document_id)
        .expect("document claim should contradict active remembered person fact");

    assert_eq!(observation.old_source_kind, ContradictionSourceKind::Memory);
    assert_eq!(
        observation.new_source_kind,
        ContradictionSourceKind::Document
    );
    assert_eq!(observation.conflict_type, "direct_contradiction");
    assert_eq!(observation.old_claim, "location=Berlin");
    assert_eq!(observation.new_claim, "location=Madrid");
    assert_eq!(observation.confidence, 0.8);
    assert_eq!(observation.severity, ContradictionSeverity::Medium);
    assert_eq!(
        observation.affected_entities,
        json!([{"entity_kind": "subject", "entity_id": person.person_id}])
    );
    assert_eq!(
        observation.metadata,
        json!({
            "detector": "structured_evidence_claim",
            "claim_type": "location",
            "source_kind": "document"
        })
    );

    let remembered_value: String =
        sqlx::query_scalar("SELECT value FROM person_facts WHERE id::text = $1")
            .bind(&fact_id)
            .fetch_one(&pool)
            .await
            .expect("remembered value");
    assert_eq!(remembered_value, "Berlin");
}
```

### `backend/tests/consistency_contradiction/refresh_provider_messages.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/tests/consistency_contradiction/refresh_provider_messages.rs`
- Size bytes / Размер в байтах: `6284`
- Included characters / Включено символов: `6284`
- Truncated / Обрезано: `no`

```rust
use makosh_hub_backend::domains::persons::api::PersonProjectionStore;
use makosh_hub_backend::engines::consistency::{
    ContradictionObservationStore, ContradictionSeverity, ContradictionSourceKind,
};
use serde_json::json;

use super::support::{
    live_consistency_pool, seed_telegram_message, seed_whatsapp_message, unique_suffix,
};

#[tokio::test]
async fn contradiction_refresh_detects_telegram_message_claim_against_active_person_fact_without_overwriting_memory()
 {
    let Some(pool) = live_consistency_pool("contradiction telegram message refresh").await else {
        return;
    };
    let store = ContradictionObservationStore::new(pool.clone());
    let suffix = unique_suffix();
    let email_address = format!("polygraph-telegram-{suffix}@example.com");
    let sender_id = format!("telegram-sender-{suffix}");
    let person = PersonProjectionStore::new(pool.clone())
        .upsert_email_person(&email_address)
        .await
        .expect("person");
    sqlx::query(
        r#"
        INSERT INTO person_identities (person_id, identity_type, identity_value, source, confidence, status)
        VALUES ($1, 'telegram', $2, 'test', 1.0, 'active')
        "#,
    )
    .bind(&person.person_id)
    .bind(&sender_id)
    .execute(&pool)
    .await
    .expect("telegram identity");
    let fact_id: String = sqlx::query_scalar(
        r#"
        INSERT INTO person_facts (person_id, fact_type, value, source, confidence)
        VALUES ($1, 'location', 'Berlin', 'manual', 0.93)
        RETURNING id::text
        "#,
    )
    .bind(&person.person_id)
    .fetch_one(&pool)
    .await
    .expect("person fact");
    let message_id = seed_telegram_message(&pool, suffix, &sender_id, "Location: Madrid").await;

    let refreshed = store
        .refresh_deterministic_observations(100)
        .await
        .expect("refresh contradictions");
    assert!(refreshed >= 1);

    let open = store.list_open(100).await.expect("open contradictions");
    let observation = open
        .iter()
        .find(|item| item.old_source_id == fact_id && item.new_source_id == message_id)
        .expect("telegram message claim should contradict active remembered person fact");

    assert_eq!(observation.old_source_kind, ContradictionSourceKind::Memory);
    assert_eq!(
        observation.new_source_kind,
        ContradictionSourceKind::Communication
    );
    assert_eq!(observation.conflict_type, "direct_contradiction");
    assert_eq!(observation.old_claim, "location=Berlin");
    assert_eq!(observation.new_claim, "location=Madrid");
    assert_eq!(observation.confidence, 0.8);
    assert_eq!(observation.severity, ContradictionSeverity::Medium);
    assert_eq!(
        observation.affected_entities,
        json!([{"entity_kind": "subject", "entity_id": person.person_id}])
    );
    assert_eq!(
        observation.metadata,
        json!({
            "detector": "structured_evidence_claim",
            "claim_type": "location",
            "source_kind": "communication"
        })
    );

    let remembered_value: String =
        sqlx::query_scalar("SELECT value FROM person_facts WHERE id::text = $1")
            .bind(&fact_id)
            .fetch_one(&pool)
            .await
            .expect("remembered value");
    assert_eq!(remembered_value, "Berlin");
}

#[tokio::test]
async fn contradiction_refresh_detects_whatsapp_message_claim_against_active_person_fact_without_overwriting_memory()
 {
    let Some(pool) = live_consistency_pool("contradiction WhatsApp message refresh").await else {
        return;
    };
    let store = ContradictionObservationStore::new(pool.clone());
    let suffix = unique_suffix();
    let email_address = format!("polygraph-whatsapp-{suffix}@example.com");
    let sender_id = format!("whatsapp-sender-{suffix}");
    let person = PersonProjectionStore::new(pool.clone())
        .upsert_email_person(&email_address)
        .await
        .expect("person");
    sqlx::query(
        r#"
        INSERT INTO person_identities (person_id, identity_type, identity_value, source, confidence, status)
        VALUES ($1, 'whatsapp', $2, 'test', 1.0, 'active')
        "#,
    )
    .bind(&person.person_id)
    .bind(&sender_id)
    .execute(&pool)
    .await
    .expect("whatsapp identity");
    let fact_id: String = sqlx::query_scalar(
        r#"
        INSERT INTO person_facts (person_id, fact_type, value, source, confidence)
        VALUES ($1, 'location', 'Berlin', 'manual', 0.93)
        RETURNING id::text
        "#,
    )
    .bind(&person.person_id)
    .fetch_one(&pool)
    .await
    .expect("person fact");
    let message_id = seed_whatsapp_message(&pool, suffix, &sender_id, "Location: Madrid").await;

    let refreshed = store
        .refresh_deterministic_observations(100)
        .await
        .expect("refresh contradictions");
    assert!(refreshed >= 1);

    let open = store.list_open(100).await.expect("open contradictions");
    let observation = open
        .iter()
        .find(|item| item.old_source_id == fact_id && item.new_source_id == message_id)
        .expect("WhatsApp message claim should contradict active remembered person fact");

    assert_eq!(observation.old_source_kind, ContradictionSourceKind::Memory);
    assert_eq!(
        observation.new_source_kind,
        ContradictionSourceKind::Communication
    );
    assert_eq!(observation.conflict_type, "direct_contradiction");
    assert_eq!(observation.old_claim, "location=Berlin");
    assert_eq!(observation.new_claim, "location=Madrid");
    assert_eq!(observation.confidence, 0.8);
    assert_eq!(observation.severity, ContradictionSeverity::Medium);
    assert_eq!(
        observation.affected_entities,
        json!([{"entity_kind": "subject", "entity_id": person.person_id}])
    );
    assert_eq!(
        observation.metadata,
        json!({
            "detector": "structured_evidence_claim",
            "claim_type": "location",
            "source_kind": "communication"
        })
    );

    let remembered_value: String =
        sqlx::query_scalar("SELECT value FROM person_facts WHERE id::text = $1")
            .bind(&fact_id)
            .fetch_one(&pool)
            .await
            .expect("remembered value");
    assert_eq!(remembered_value, "Berlin");
}
```

### `backend/tests/consistency_contradiction/support.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/tests/consistency_contradiction/support.rs`
- Size bytes / Размер в байтах: `7375`
- Included characters / Включено символов: `7375`
- Truncated / Обрезано: `no`

```rust
#![allow(dead_code)]

use std::time::{SystemTime, UNIX_EPOCH};
use testkit::context::TestContext;

use chrono::Utc;
use makosh_hub_backend::domains::communications::core::{
    CommunicationIngestionStore, CommunicationProviderKind, EmailProviderKind, NewProviderAccount,
    NewRawCommunicationRecord,
};
use makosh_hub_backend::domains::communications::messages::{
    MessageProjectionStore, consume_accepted_signal_event, project_raw_email_message,
};
use makosh_hub_backend::domains::signal_hub::{
    dispatch_telegram_raw_signal, dispatch_whatsapp_raw_signal,
};
use makosh_hub_backend::platform::storage::Database;
use serde_json::json;
use sqlx::postgres::PgPool;

pub async fn live_consistency_pool(_test_name: &str) -> Option<PgPool> {
    let test_context = TestContext::new().await;
    let database_url = test_context.connection_string();

    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    Some(database.pool().expect("configured pool").clone())
}

pub fn unique_suffix() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock after unix epoch")
        .as_nanos()
}

pub async fn seed_message(
    pool: &PgPool,
    suffix: u128,
    sender: &str,
    recipients: &[String],
    provider_record_id: &str,
    subject: &str,
    body_text: &str,
) -> String {
    let account_id = format!("acct_polygraph_{suffix}");
    let ingestion_store = CommunicationIngestionStore::new(pool.clone());
    ingestion_store
        .upsert_provider_account(&NewProviderAccount::new(
            &account_id,
            EmailProviderKind::Gmail,
            "Polygraph Gmail",
            format!("polygraph-{suffix}@example.com"),
        ))
        .await
        .expect("provider account");

    let raw_record_id = format!("raw_polygraph_{suffix}_{provider_record_id}");
    let raw = ingestion_store
        .record_raw_source(
            &NewRawCommunicationRecord::new(
                &raw_record_id,
                &account_id,
                "email_message",
                provider_record_id,
                format!("sha256:polygraph:{suffix}:{provider_record_id}"),
                format!("batch-polygraph-{suffix}"),
                json!({
                    "subject": subject,
                    "from": sender,
                    "to": recipients,
                    "body_text": body_text,
                }),
            )
            .occurred_at(Utc::now())
            .provenance(json!({"source":"polygraph_test"})),
        )
        .await
        .expect("raw message");

    let message_store = MessageProjectionStore::new(pool.clone());
    project_raw_email_message(&message_store, &raw)
        .await
        .expect("project message")
        .message_id
}

pub async fn seed_telegram_message(
    pool: &PgPool,
    suffix: u128,
    sender_id: &str,
    body_text: &str,
) -> String {
    let account_id = format!("acct_polygraph_telegram_{suffix}");
    let provider_chat_id = format!("telegram-chat-{suffix}");
    let provider_message_id = format!("telegram-message-{suffix}");
    let ingestion_store = CommunicationIngestionStore::new(pool.clone());
    ingestion_store
        .upsert_provider_account(&NewProviderAccount::new(
            &account_id,
            CommunicationProviderKind::TelegramUser,
            "Polygraph Telegram",
            format!("polygraph-telegram-{suffix}"),
        ))
        .await
        .expect("provider account");

    let raw = ingestion_store
        .record_raw_source(
            &NewRawCommunicationRecord::new(
                format!("raw_polygraph_telegram_{suffix}"),
                &account_id,
                "telegram_message",
                &provider_message_id,
                format!("sha256:polygraph:telegram:{suffix}"),
                format!("batch-polygraph-telegram-{suffix}"),
                json!({
                    "provider_chat_id": provider_chat_id,
                    "chat_title": format!("Polygraph Telegram {suffix}"),
                    "chat_kind": "private",
                    "sender_id": sender_id,
                    "sender_display_name": "Polygraph Telegram Sender",
                    "text": body_text,
                    "delivery_state": "received",
                }),
            )
            .occurred_at(Utc::now())
            .provenance(json!({
                "source": "polygraph_test",
                "provider": "telegram",
                "provider_kind": "telegram_user",
                "account_id": account_id,
                "provider_chat_id": provider_chat_id,
            })),
        )
        .await
        .expect("raw telegram message");

    let accepted_event = dispatch_telegram_raw_signal(pool.clone(), &raw)
        .await
        .expect("dispatch telegram raw signal")
        .expect("accepted telegram signal");
    consume_accepted_signal_event(pool.clone(), &accepted_event)
        .await
        .expect("project accepted telegram signal")
        .expect("projected telegram message")
        .message_id
}

pub async fn seed_whatsapp_message(
    pool: &PgPool,
    suffix: u128,
    sender_id: &str,
    body_text: &str,
) -> String {
    let account_id = format!("acct_polygraph_whatsapp_{suffix}");
    let provider_chat_id = format!("whatsapp-chat-{suffix}");
    let provider_message_id = format!("whatsapp-message-{suffix}");
    let ingestion_store = CommunicationIngestionStore::new(pool.clone());
    ingestion_store
        .upsert_provider_account(&NewProviderAccount::new(
            &account_id,
            CommunicationProviderKind::WhatsappWeb,
            "Polygraph WhatsApp",
            format!("polygraph-whatsapp-{suffix}"),
        ))
        .await
        .expect("provider account");

    let raw = ingestion_store
        .record_raw_source(
            &NewRawCommunicationRecord::new(
                format!("raw_polygraph_whatsapp_{suffix}"),
                &account_id,
                "whatsapp_web_message",
                &provider_message_id,
                format!("sha256:polygraph:whatsapp:{suffix}"),
                format!("batch-polygraph-whatsapp-{suffix}"),
                json!({
                    "provider_chat_id": provider_chat_id,
                    "chat_title": format!("Polygraph WhatsApp {suffix}"),
                    "sender_id": sender_id,
                    "sender_display_name": "Polygraph WhatsApp Sender",
                    "text": body_text,
                    "delivery_state": "received",
                }),
            )
            .occurred_at(Utc::now())
            .provenance(json!({
                "source": "polygraph_test",
                "provider": "whatsapp",
                "provider_kind": "whatsapp_web",
                "account_id": account_id,
                "provider_chat_id": provider_chat_id,
            })),
        )
        .await
        .expect("raw WhatsApp message");

    let accepted_event = dispatch_whatsapp_raw_signal(pool.clone(), &raw)
        .await
        .expect("dispatch WhatsApp raw signal")
        .expect("accepted WhatsApp signal");
    consume_accepted_signal_event(pool.clone(), &accepted_event)
        .await
        .expect("project accepted WhatsApp signal")
        .expect("projected WhatsApp message")
        .message_id
}
```

### `backend/tests/consistency_contradiction_architecture.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/tests/consistency_contradiction_architecture.rs`
- Size bytes / Размер в байтах: `2114`
- Included characters / Включено символов: `2114`
- Truncated / Обрезано: `no`

```rust
use std::fs;
use std::path::{Path, PathBuf};

const MAX_TEST_FILE_LINES: usize = 700;

#[test]
fn consistency_contradiction_tests_stay_below_architecture_line_limit() {
    let tests_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests");

    let mut violations = Vec::new();
    collect_consistency_contradiction_test_violations(&tests_dir, &mut violations);

    assert!(
        violations.is_empty(),
        "consistency contradiction test files exceed {MAX_TEST_FILE_LINES} lines:\n{}",
        violations.join("\n")
    );
}

fn collect_consistency_contradiction_test_violations(root: &Path, violations: &mut Vec<String>) {
    let entries = fs::read_dir(root)
        .unwrap_or_else(|error| panic!("failed to read test directory {root:?}: {error}"));

    for entry in entries {
        let entry = entry.expect("failed to read test directory entry");
        let path = entry.path();
        if path.is_dir() {
            collect_consistency_contradiction_test_violations(&path, violations);
            continue;
        }
        if !is_consistency_contradiction_test_file(&path) {
            continue;
        }

        let content = fs::read_to_string(&path)
            .unwrap_or_else(|error| panic!("failed to read {path:?}: {error}"));
        let line_count = content.lines().count();
        if line_count > MAX_TEST_FILE_LINES {
            violations.push(format!("{}: {line_count}", path.display()));
        }
    }
}

fn is_consistency_contradiction_test_file(path: &Path) -> bool {
    let file_name = path
        .file_name()
        .and_then(|file_name| file_name.to_str())
        .unwrap_or_default();
    if file_name == "consistency_contradiction.rs"
        || file_name == "consistency_contradiction_architecture.rs"
    {
        return true;
    }

    path.components().any(|component| {
        component
            .as_os_str()
            .to_str()
            .is_some_and(|value| value == "consistency_contradiction")
    }) && path
        .extension()
        .and_then(|extension| extension.to_str())
        .is_some_and(|extension| extension == "rs")
}
```

### `backend/tests/context_packs.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/tests/context_packs.rs`
- Size bytes / Размер в байтах: `4875`
- Included characters / Включено символов: `4875`
- Truncated / Обрезано: `no`

```rust
use std::time::{SystemTime, UNIX_EPOCH};
use testkit::context::TestContext;

use chrono::{TimeZone, Utc};
use makosh_hub_backend::engines::context_packs::{
    ContextPackKind, ContextPackSourceKind, ContextPackStore, ContextPackStoreError,
    NewContextPack, NewContextPackSource,
};
use makosh_hub_backend::platform::observations::{
    NewObservation, ObservationOriginKind, ObservationStore,
};
use makosh_hub_backend::platform::storage::Database;
use serde_json::json;
use sqlx::postgres::{PgPool, PgPoolOptions};

#[tokio::test]
async fn context_pack_store_persists_derived_pack_with_explicit_sources_against_postgres() {
    let Some((_pool, observation_store, context_pack_store)) =
        live_context_pack_context("context pack sources").await
    else {
        return;
    };
    let suffix = unique_suffix();
    let observation = observation_store
        .capture(
            &NewObservation::new(
                "MEETING_TRANSCRIPT",
                ObservationOriginKind::VaultSource,
                Utc.with_ymd_and_hms(2026, 6, 18, 12, 0, 0).unwrap(),
                json!({
                    "meeting_id": format!("meeting:v1:{suffix}"),
                    "transcript": "Decision: prepare the NAS purchase context pack."
                }),
                format!("zoom://meeting/{suffix}/transcript"),
            )
            .vault_source_id(format!("vault_source:zoom:{suffix}"))
            .confidence(0.94),
        )
        .await
        .expect("capture meeting transcript");

    let stored = context_pack_store
        .upsert_with_sources(
            &NewContextPack::new(
                ContextPackKind::Meeting,
                format!("meeting:v1:{suffix}"),
                json!({
                    "summary": "Meeting context for NAS purchase decision.",
                    "open_items": ["prepare storage requirements"]
                }),
            )
            .metadata(json!({"builder": "contract-test"})),
            &[
                NewContextPackSource::new(
                    ContextPackSourceKind::Observation,
                    observation.observation_id.clone(),
                )
                .role("primary_evidence"),
                NewContextPackSource::new(
                    ContextPackSourceKind::DomainEntity,
                    format!("meeting:v1:{suffix}"),
                )
                .role("meeting"),
                NewContextPackSource::new(
                    ContextPackSourceKind::Knowledge,
                    format!("knowledge:v1:{suffix}"),
                )
                .role("background"),
            ],
        )
        .await
        .expect("upsert context pack");

    assert_eq!(stored.kind, ContextPackKind::Meeting);
    assert_eq!(stored.subject_id, format!("meeting:v1:{suffix}"));
    assert!(stored.rebuildable);
    assert_eq!(
        stored.content["summary"],
        json!("Meeting context for NAS purchase decision.")
    );

    let sources = context_pack_store
        .list_sources(&stored.context_pack_id)
        .await
        .expect("list context pack sources");
    assert_eq!(sources.len(), 3);
    assert!(sources.iter().any(|source| {
        source.source_kind == ContextPackSourceKind::Observation
            && source.source_id == observation.observation_id
    }));
}

#[tokio::test]
async fn context_pack_store_rejects_pack_without_sources_before_database_write() {
    let store = disconnected_context_pack_store();
    let error = store
        .upsert_with_sources(
            &NewContextPack::new(
                ContextPackKind::Persona,
                "person:v1:missing-sources",
                json!({"summary": "source-less context is not acceptable"}),
            ),
            &[],
        )
        .await
        .expect_err("context pack without sources must fail before database write");

    assert!(matches!(error, ContextPackStoreError::MissingSources));
}

async fn live_context_pack_context(
    _test_name: &str,
) -> Option<(PgPool, ObservationStore, ContextPackStore)> {
    let test_context = TestContext::new().await;
    let database_url = test_context.connection_string();

    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    let pool = database.pool().expect("configured pool").clone();
    Some((
        pool.clone(),
        ObservationStore::new(pool.clone()),
        ContextPackStore::new(pool),
    ))
}

fn disconnected_context_pack_store() -> ContextPackStore {
    let pool = PgPoolOptions::new()
        .connect_lazy("postgres://makosh:unused@127.0.0.1:1/makosh_hub")
        .expect("create lazy test pool");
    ContextPackStore::new(pool)
}

fn unique_suffix() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock before UNIX_EPOCH")
        .as_nanos()
}
```
