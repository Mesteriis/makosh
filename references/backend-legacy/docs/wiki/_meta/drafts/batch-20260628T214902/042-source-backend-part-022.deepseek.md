### Summary / Резюме

Предоставленные исходные файлы (`backend/src/domains/communications/`) описывают подсистемы бэкенда, относящиеся к сохранённым поискам, полнотекстовому поиску, анализу SPF/DKIM/DMARC, электронным подписям, источникам писем и хранилищу вложений.
Страница `components/backend.md` должна быть дополнена структурированным описанием этих компонентов, чтобы отразить их состав, ключевые структуры данных и публичные интерфейсы, прямо подтверждённые исходным кодом.

### Proposed pages / Предлагаемые страницы

#### `components/backend.md`

```markdown
# Backend Components

## Communications Domain

Домен `backend/src/domains/communications/` объединяет подсистемы, связанные с обработкой писем, вложений и поиском.

### Saved Searches (`saved_searches.rs`)

**Основные структуры**

- `CommunicationSavedSearch` – сохранённый поиск:
  - поля: `saved_search_id`, `name`, `description`, `account_id`, `query`, `workflow_state`, `local_state`, `channel_kind`, `is_smart_folder`, `sort_order`, `message_count`, `created_at`, `updated_at`.
- `NewCommunicationSavedSearch` – входные данные для создания (`saved_search_id`, `name`, `description`, `account_id`, `query`, `workflow_state`, `local_state`, `channel_kind`, `is_smart_folder`, `sort_order`).
- `UpdateCommunicationSavedSearch` – данные для обновления (все поля опциональны).
- `CommunicationSavedSearchListPage` – страница списка с полями `items`, `next_cursor`, `has_more`.

**Хранилище**

`CommunicationSavedSearchStore` (обёртка над `PgPool`) предоставляет:

- `list(query: CommunicationSavedSearchListQuery<'_>) -> CommunicationSavedSearchListPage` – постраничный список с курсором (сортировка `is_smart_folder DESC, sort_order ASC, lower(name) ASC, updated_at DESC, saved_search_id ASC`). Ограничение числа записей `1..1000`.
- `create(input: NewCommunicationSavedSearch) -> CommunicationSavedSearch` – создание с записью события `mail.saved_search.created`.
- `create_with_observation(...)` – создание с привязкой к наблюдению (observation).
- `update(saved_search_id, update) -> Option<CommunicationSavedSearch>` – обновление с событием `mail.saved_search.updated`.
- `update_with_observation(...)` – обновление с привязкой к наблюдению.
- `delete(saved_search_id) -> bool` – удаление с событием `mail.saved_search.deleted`.
- `delete_with_observation(...)` – удаление с привязкой к наблюдению.

Операции выполняются в транзакции, проверяют каноничность `account_id`, сохраняют события через `EventStore` и линкуют сущности через `link_mail_entity_in_transaction` (из модуля `evidence` – не входит в данный контекст).

Значения `message_count` подгружаются через `load_message_counts_for_saved_searches` (модуль `saved_search_counts` – не входит в данный контекст).

**События**

- `mail.saved_search.created`
- `mail.saved_search.updated`
- `mail.saved_search.deleted`

### Full-Text Search (`search.rs`)

- `project_message_to_search_document(message: &ProjectedMessage) -> SearchDocument` — преобразует `ProjectedMessage` в документ для поискового индекса, используя `message_id` как `object_id`, `object_kind = "communication_message"`, заголовок в формате `[sender] subject` и тело `body_text`.
- `index_messages(index, store, limit) -> Result<usize, IndexEmailError>` — получает последние сообщения из `MessageProjectionStore` и индексирует их.
- `search_emails(index, query, limit) -> Result<Vec<SearchResult>, IndexEmailError>` — выполняет поиск и фильтрует результат по `object_kind == "communication_message"`.
- `IndexEmailError` — ошибка, оборачивающая `SearchError` и `MessageProjectionError`.

### SPF/DKIM/DMARC (`spf_dkim.rs`)

**Парсинг заголовков**

- `parse_auth_headers(raw_headers: &str) -> AuthResults` — разбирает строки, начинающиеся с `authentication-results:` или `received-spf:`, извлекает значения для `spf=`, `dkim=`, `dmarc=` и сопутствующие атрибуты (`smtp.mailfrom`, `d=`, `s=`, `header.from`, `p=`). Возвращает `AuthResults` с опциональными `SpfResult`, `DkimResult`, `DmarcResult` и списком исходных заголовков `raw_headers`.

**Оценка риска**

- `assess_auth_risk(auth: &AuthResults) -> SpfDkimReport` — формирует сводку:
  - `has_spf`, `has_dkim`, `has_dmarc` – наличие заголовков.
  - `spf_pass`, `dkim_pass`, `dmarc_pass` – пройдены ли проверки (результат `"pass"`).
  - `is_spoofed` – признак подделки, если хотя бы один из имеющихся заголовков не `pass`.
  - `risk_summary` – текстовое резюме.

**Тесты** подтверждают разбор и логику определения spoofed/clean.

### Signatures (`signatures/`)

**Публичный интерфейс модуля** (переэкспорт):

- `CertificateType` — перечисление: `Smime`, `Pgp`, `PdfSign`, `Cades`, `Xades`, `GostSign`, `Unknown`. Методы `as_str()` и `parse()`.
- `SignatureDetection` — результат детектирования со статусом `has_signature`, `signature_type`, `signer_info`, `is_valid`, `cert_expiry_warning`.
- `SignatureDetector` — детектор подписей в теле и заголовках письма.
- `CertificateRecord` — полная запись сертификата (все поля, включая `cert_type`, `provider`, `storage_kind`, `trust_status`, `is_revoked`, `usage`, `metadata`, таймстемпы).
- `NewCertificate` — данные для создания сертификата; метод `validate()` проверяет непустой `cert_id`.
- `CertificateProvider` — `Fnmt`, `Dnie`, `CryptoPro`, `Gost`, `AppleKeychain`, `Pkcs12`, `Yubikey`, `UsbToken`, `Other`.
- `CertificateStorageKind` — `OsKeychain`, `EncryptedVault`, `Pkcs12File`, `PfxFile`, `SmartCard`, `UsbToken`, `ExternalVault`.
- `CertificateStore` — хранилище сертификатов в PostgreSQL:
  - `upsert(cert: &NewCertificate) -> CertificateRecord` — вставка/обновление с `ON CONFLICT (cert_id) DO UPDATE`.
  - `list() -> Vec<CertificateRecord>` — полный список, сортировка по `COALESCE(valid_until, created_at) DESC`.
  - `expiring_soon(days: i64) -> Vec<CertificateRecord>` — сертификаты, у которых `valid_until` попадает в интервал `[now, now + days]` и `is_revoked = false`.
- `TrustStatus` — `Trusted`, `Untrusted`, `Expired`, `Revoked`, `PendingVerification`, `SelfSigned`.

**Детектирование**

`SignatureDetector::detect_in_message(body_text, headers)` проверяет:
- SMIME: наличие в заголовках `Content-Type: application/pkcs7-mime` или `application/x-pkcs7-signature`.
- PGP: наличие в теле `-----BEGIN PGP SIGNATURE-----` или `-----BEGIN PGP MESSAGE-----`.

Возвращает `SignatureDetection` с соответствующим `CertificateType`, поля `signer_info`, `is_valid`, `cert_expiry_warning` заполняются как `None`.

`SignatureDetector::check_expiry_warning(cert)` — если срок истекает через ≤90 дней или уже истёк, возвращает предупреждение.

**Тесты** проверяют корректность определения SMIME, PGP, отсутствия подписи, а также roundtrip-преобразование `CertificateType`.

### Sources (`sources.rs`)

- `FixtureCommunicationSourceMessage` — представление тестового письма с обязательными полями: `provider_record_id`, `subject`, `from`, `to`, `body_text`, `source_fingerprint`.
- `parse_fixture_email_messages(input: &str) -> Result<Vec<FixtureCommunicationSourceMessage>, FixtureEmailSourceError>` — парсит JSON-массив, валидирует каждое сообщение: обязательные поля должны быть непустыми, `to` должен содержать хотя бы одного получателя.
- `FixtureEmailSourceError` — варианты: `Json`, `EmptyField`, `EmptyRecipients`.

### Storage (`storage/`)

**Публичный интерфейс модуля** (переэкспорт):

- `LocalCommunicationBlobStore` (алиас `LocalCommunicationBlobPort`), `LocalCommunicationBlob`.
- `CommunicationStorageError`, `AttachmentSafetyScanError`.
- ID‑функция `new_communication_attachment_import_id`.
- Модели: `CommunicationAttachmentDisposition`, `ImportedCommunicationAttachment`, `NewCommunicationAttachment`, `NewCommunicationAttachmentImport`, `NewCommunicationBlob`, `StoredCommunicationAttachment`, `StoredCommunicationAttachmentWithBlob`, `StoredCommunicationBlob`.
- Сканеры: `AttachmentSafetyScanReport`, `AttachmentSafetyScanRequest`, `AttachmentSafetyScanStatus`, `AttachmentSafetyScanner`, `HeuristicAttachmentSafetyScanner`, `NoopAttachmentSafetyScanner`.
- `CommunicationStorageStore` (основное хранилище, также `CommunicationBlobMetadataPort`).

**Локальное блоб-хранилище (`blob_store.rs`)**

`LocalCommunicationBlobStore`:
- `new(root)` – рабочая директория.
- `put_blob(bytes) -> LocalCommunicationBlob` – сохраняет блоб по пути `sha256/xx/digest.blob` (первые два символа хеша как поддиректория). Файл записывается атомарно через временный файл, проверяется соответствие размера.
- `read_blob(storage_path) -> Vec<u8>` – читает файл с проверкой безопасности пути.
- `delete_blob(storage_path) -> bool` – удаляет файл и подчищает пустые родительские директории.
- SHA-256 вычисляется с префиксом `sha256:`.

**Константы (`constants.rs`)**
- `LOCAL_FS_STORAGE_KIND = "local_fs"`, `SHA256_PREFIX = "sha256:"`.

**Идентификаторы (`ids.rs`)**
- Формирование идентификаторов: `mail_blob_id(sha256)`, `mail_attachment_id(message_id, provider_attachment_id)`, `communication_attachment_import_id(seed)` с формой `prefix:v1:len:value`.

**Модели вложений и импортов (`models.rs`)**

- `NewCommunicationBlob` – валидирует `storage_kind`, `storage_path`, `sha256`, `size_bytes`, `content_type`.
- `NewCommunicationAttachmentImport` – описывает импортированное вложение; валидация всех полей, требует JSON-объект для `metadata`, а `scan_report` не должен содержать данные, если статус `NotScanned`.
- `ImportedCommunicationAttachment` – прочитанная из БД запись импорта (включает поля из `communication_attachment_imports` и `communication_mail_blobs`).
- `NewCommunicationAttachment` – вложение, связанное с письмом; поля: `message_id`, `raw_record_id`, `blob_id`, `provider_attachment_id`, `filename`, `content_type`, `size_bytes`, `sha256`, `disposition`, `scan_report`. Валидация всех полей.
- `CommunicationAttachmentDisposition` – `Attachment`, `Inline`, `Unknown`; методы `as_str()` и парсинг из строки.

**Сканер безопасности вложений (`scanner.rs`)**

`AttachmentSafetyScanStatus`: `NotScanned`, `Clean`, `Suspicious`, `Malicious`, `Failed`.

`AttachmentSafetyScanReport`:
- `status`, `engine`, `checked_at`, `summary`, `metadata` (JSON).
- Валидация: при статусе `NotScanned` поля `engine`, `checked_at`, `summary` должны быть `None`.

`AttachmentSafetyScanner` (trait) – метод `scan(request) -> AttachmentSafetyScanReport`.

`HeuristicAttachmentSafetyScanner`:
- Проверяет:
  1. Magic-байты исполняемых файлов (`MZ` или `\x7fELF`) → `Malicious`.
  2. «Активные» расширения (`.exe`, `.dll`, `.ps1`, `.vbs` и др.) → `Malicious`.
  3. Расширения документов с макросами (`.docm`, `.xlsm`, `.pptm` и др.) → `Suspicious`.
  4. Несовпадение MIME-типа и расширения для известных пар → `Suspicious`.
- Собирает причины (`reasons`), возвращает отчёт с `engine = "makosh_heuristic_v1"`, временем проверки, сводкой и метаданными.

`NoopAttachmentSafetyScanner` – всегда возвращает `NotScanned`.

**Строковые отображения (`rows.rs`)**
- `row_to_mail_blob`, `row_to_mail_attachment`, `row_to_mail_attachment_with_blob`, `row_to_imported_attachment` – маппинг `PgRow` в соответствующие структуры, включая парсинг `disposition`, `scan_status` из строк.

**Операции импорта (`imports.rs`, частично)**

`CommunicationStorageStore` (методы, зафиксированные в контексте):
- `upsert_imported_attachment(import)` – вставка/обновление записи импорта в `communication_attachment_imports` с join к `communication_mail_blobs`.
- `upsert_imported_attachment_with_observation(...)` – дополнительно линкует вложение к `observation` через `link_mail_entity_in_transaction`.
- `imported_attachment_by_id(attachment_id)` – поиск по `attachment_id`.
- `imported_attachment_by_blob_id(blob_id)` – поиск по `blob_id`.
- `list_imported_attachments(account_id, source_kind, limit)` – список импортов с лимитом `1..100`.
- `list_expired_imported_attachments(...)` – выборка записей с истёкшим `retention_policy.expires_at`.
- `blob_by_id(blob_id)` – поиск блоба в `communication_mail_blobs`.
- `remove_imported_attachment(...)` – удаление импорта и, если блоб больше не используется, удаление метаданных блоба.

**Ошибки (`errors.rs`)**
- `CommunicationStorageError` – охватывает `Sqlx`, `ObservationStoreError`, `Io`, ошибки валидации (`EmptyField`, `InvalidStorageKind`, `UnsafeStoragePath`, `InvalidSha256`, `NegativeSizeBytes`, `BlobTooLarge`, `BlobSizeMismatch`, `InvalidDisposition`, `InvalidScanStatus`, `NonObjectJson`, `InvalidNotScannedReport`).
- `AttachmentSafetyScanError` – ошибка сканера.

## Связи

- Модуль `service.rs` лишь реэкспортирует `super::command_service::*`; содержание `command_service` не подтверждено данным контекстом.
- Часть функциональности (`saved_search_counts`, `evidence`, `platform::events`, `platform::storage`) используется в представленных файлах, но их исходный код отсутствует в данном чанке – детальное описание этих зависимостей не может быть дано.
```

### Source coverage / Покрытие источников

| Файл | Факты, покрытые в странице |
|------|---------------------------|
| `backend/src/domains/communications/saved_searches.rs` (truncated) | Структуры `CommunicationSavedSearch`, `NewCommunicationSavedSearch`, `UpdateCommunicationSavedSearch`, `CommunicationSavedSearchListPage`; `CommunicationSavedSearchStore` с методами `list`, `create`, `create_with_observation`, `update`, `update_with_observation`, `delete`, `delete_with_observation`; события `mail.saved_search.{created,updated,deleted}`; использование транзакций, `EventStore`, `link_mail_entity_in_transaction`. |
| `backend/src/domains/communications/search.rs` | Функции `project_message_to_search_document`, `index_messages`, `search_emails`; структура `SearchDocument`, фильтр `object_kind == "communication_message"`; ошибка `IndexEmailError`. |
| `backend/src/domains/communications/service.rs` | Реэкспорт `super::command_service::*`. |
| `backend/src/domains/communications/signatures.rs` | Перечень публичных реэкспортов модуля signatures. |
| `backend/src/domains/communications/signatures/certificate_type.rs` | `CertificateType` с вариантами, `as_str()` и `parse()`. |
| `backend/src/domains/communications/signatures/detector.rs` | `SignatureDetection`, `SignatureDetector::detect_in_message` (детектирование SMIME/PGP), `SignatureDetector::check_expiry_warning`. |
| `backend/src/domains/communications/signatures/errors.rs` | `CertificateError` с вариантами `Sqlx` и `Invalid`. |
| `backend/src/domains/communications/signatures/models.rs` | `CertificateRecord`, `NewCertificate` и их поля; метод `validate`. |
| `backend/src/domains/communications/signatures/provider.rs` | `CertificateProvider` перечисление, методы `as_str`, `parse`. |
| `backend/src/domains/communications/signatures/rows.rs` | Константа `CERTIFICATE_COLUMNS`, функция `row_to_cert` с парсингом всех полей. |
| `backend/src/domains/communications/signatures/storage_kind.rs` | `CertificateStorageKind` перечисление. |
| `backend/src/domains/communications/signatures/store.rs` | `CertificateStore::new`, `upsert`, `list`, `expiring_soon` (SQL-запросы и логика). |
| `backend/src/domains/communications/signatures/tests.rs` | Тесты детектирования SMIME, PGP, отсутствия подписи; roundtrip `CertificateType`. |
| `backend/src/domains/communications/signatures/trust.rs` | `TrustStatus` перечисление, методы `as_str`, `parse`. |
| `backend/src/domains/communications/sources.rs` | `FixtureCommunicationSourceMessage`, `parse_fixture_email_messages`, валидация полей, `FixtureEmailSourceError`. |
| `backend/src/domains/communications/spf_dkim.rs` | `AuthResults`, `SpfResult`, `DkimResult`, `DmarcResult`; `parse_auth_headers` (парсинг заголовков); `SpfDkimReport` и `assess_auth_risk`; unit-тесты. |
| `backend/src/domains/communications/storage.rs` | Сводка публичных реэкспортов модуля storage. |
| `backend/src/domains/communications/storage/blob_store.rs` | `LocalCommunicationBlobStore` с методами `put_blob`, `read_blob`, `delete_blob`; структура `LocalCommunicationBlob`; атомарная запись, проверка размера, получение SHA-256. |
| `backend/src/domains/communications/storage/constants.rs` | Константы `LOCAL_FS_STORAGE_KIND`, `SHA256_PREFIX`. |
| `backend/src/domains/communications/storage/errors.rs` | `CommunicationStorageError` и все перечисленные варианты; `AttachmentSafetyScanError`. |
| `backend/src/domains/communications/storage/ids.rs` | Функции генерации `mail_blob_id`, `mail_attachment_id`, `communication_attachment_import_id`. |
| `backend/src/domains/communications/storage/imports.rs` (truncated) | Методы `CommunicationStorageStore`: `upsert_imported_attachment`, `upsert_imported_attachment_with_observation`, `imported_attachment_by_id`, `imported_attachment_by_blob_id`, `list_imported_attachments`, `list_expired_imported_attachments`, `blob_by_id`, `remove_imported_attachment`; SQL-запросы и логика связывания с observation. |
| `backend/src/domains/communications/storage/models.rs` (truncated) | `NewCommunicationBlob`, `StoredCommunicationBlob`, `NewCommunicationAttachmentImport`, `ImportedCommunicationAttachment`, `ImportedCommunicationAttachmentRemovalResult`, `NewCommunicationAttachment`, `StoredCommunicationAttachment`, `StoredCommunicationAttachmentWithBlob`, `CommunicationAttachmentDisposition`; методы `validate` и builder-паттерны. |
| `backend/src/domains/communications/storage/rows.rs` | Функции маппинга `PgRow`: `row_to_mail_blob`, `row_to_mail_attachment`, `row_to_mail_attachment_with_blob`, `row_to_imported_attachment`. |
| `backend/src/domains/communications/storage/scanner.rs` | `AttachmentSafetyScanReport`, `AttachmentSafetyScanStatus`, `HeuristicAttachmentSafetyScanner` с проверками, `NoopAttachmentSafetyScanner`; вспомогательные функции и тесты. |

### Drift candidates / Кандидаты на drift

Из предоставленного контекста расхождения между кодом, документацией и ADR не видны.
Некоторые модули (например, `evidence`, `platform::events`, `saved_search_counts`, `command_service`) используются в рассмотренных файлах, но их исходный код отсутствует в данном чанке; без их кода невозможно подтвердить или опровергнуть соответствие документации – эти зависимости помечены в вики-странице как неподтверждённые.
