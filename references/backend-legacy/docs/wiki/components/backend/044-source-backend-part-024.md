---
chunk_id: 044-source-backend-part-024
batch_id: batch-20260628T214902
group: backend
role: source
source_status: pending
source_count: 25
generated_by: code-wiki-ru
---

# 044-source-backend-part-024 — backend/source

- Target index: [[components/backend]]
- Batch: `batch-20260628T214902`
- Source files: `25`

## Резюме

Создать/обновить страницу `components/backend.md` в русской wiki Макошь. Добавить описание сервисов домена решений, импорта документов, классификации вложений и конвейера обработки документов, опираясь на исходные файлы из этого набора контекста. Все утверждения о поведении подтверждены исходным кодом.

## Предложенные страницы

**Wiki page: `components/backend.md`**

```markdown
# Backend Components

## Decisions Service

### DecisionCommandService

`DecisionCommandService` предоставляет команду ручного ревью решения.

- **review_manual** — обновляет состояние ревью (`review_state`) у решения:
  - Создаёт observation типа `REVIEW_TRANSITION` с источником `Manual`, привязывая идентификатор решения и выбранное состояние.
  - Вызывает `DecisionStore::set_review_state_with_observation`, передавая observation_id и метаданные.
  - Возвращает обновлённое решение.

### DecisionStore

Хранилище решений (`DecisionStore`) работает поверх `PgPool` и предоставляет следующие методы:

- **upsert_with_evidence** — вставляет или обновляет решение, его свидетельства (evidence) и затронутые сущности (impacted entities) в одной транзакции. Перед вставкой проверяет, что все переданные observation_id существуют в таблице observations.
- **list_for_entity** — возвращает список решений, связанных с указанной сущностью (`entity_kind`, `entity_id`), включая решения, где сущность является `decided_by_entity` или присутствует в `decision_impacted_entities`. Ограничение `limit` зажимается в диапазон [1, 100].
- **list_by_review_state** — фильтрует решения по состоянию ревью. Лимит аналогично ограничен [1, 100].
- **set_review_state_with_observation** — обновляет `review_state` решения и связывает observation перехода ревью (через `link_decision_review_transition_in_transaction`). Если observation_id не передан, связывание не выполняется.

### Validation (валидация решений)

Функции валидации в модуле:

- **validate_decision_with_evidence** — проверяет валидность решения, требует хотя бы одного свидетельства, валидирует каждое свидетельство и затронутую сущность.
- **preserve_existing_review_state** — читает текущее `review_state` решения. Если существующее состояние не `Suggested`, перезаписывает им переданное новое решение, чтобы предотвратить случайное изменение статуса ревью.
- **validate_refresh_limit** — проверяет, что лимит лежит в пределах [MIN_REFRESH_LIMIT..MAX_REFRESH_LIMIT] (точные значения констант не входят в предоставленный контекст).
- **validate_non_empty** — проверяет, что строка непустая после trim.
- **validate_score** — проверяет, что значение `f64` находится в диапазоне [0.0, 1.0].
- **validate_json_object** / **validate_json_array** — проверяют, что `serde_json::Value` является объектом/массивом.

## Document Import (Documents Core)

### DocumentImportStore

Хранилище импорта документов (`DocumentImportStore`) реализует:

- **import_document** — начинает транзакцию и вызывает `import_document_in_transaction`.
- **import_document_and_enqueue_processing** — импортирует документ, затем помещает документ в очередь на обработку (`DocumentProcessingStore::enqueue_for_document`).
- Внутри транзакции создаётся observation типа `DOCUMENT` через `ObservationStore`, затем выполняется upsert в таблицу `documents`:
  - INSERT … ON CONFLICT (document_id) DO UPDATE SET …
  - При обновлении требует, чтобы `document_kind` не менялся (иначе ошибка `DocumentKindChange`). Если строка не возвращена INSERT (например, конфликт по kind), возвращается ошибка `UpsertSkipped`.
  - После успешного импорта связывает observation с сущностью документа через `link_document_entity_in_transaction`.
  - Если предоставлен `source_observation_id`, дополнительно связывает его с документом.

### NewDocumentImport (модель импорта)

`NewDocumentImport` может быть создана двумя способами:

- **markdown** — автоматически извлекает текст из Markdown-содержимого (удаляя ATX-заголовки) и генерирует локальный fingerprint (FNV-1a) для идемпотентности.
- **pdf_metadata** — задаёт `document_kind = "pdf"`, оставляет `extracted_text` пустым, принимает внешний fingerprint.

### Markdown extraction (извлечение текста)

Функция `extract_markdown_text` обрабатывает строки Markdown: если строка является ATX-заголовком (от 1 до 6 символов `#`, за которыми следует пробел), заголовочный маркер отбрасывается, остаётся текст заголовка. Остальные строки сохраняются как есть.

### Fingerprint

`local_markdown_fingerprint` вычисляет 64‑битный FNV-1a хеш от `extracted_text` и формирует строку вида `local-v1:markdown:{hash:016x}`. Не является криптографической проверкой контента.

### Validation (валидация импорта)

**validate_document_import** проверяет, что все обязательные поля (`document_id`, `document_kind`, `title`, `source_fingerprint`) непустые. Поддерживает только:

- `document_kind = "markdown"` — требует непустой `extracted_text` (после trim).
- `document_kind = "pdf"` — `extracted_text` может быть пустым.
- Любой другой kind вызывает ошибку `InvalidDocumentKind`.

### Error types

- `DocumentImportError` — ошибки: пустое поле, неверный document_kind, попытка смены kind, upsert skipped, обёртки для SQLx и ObservationStore.
- `DocumentImportWithProcessingError` — комбинирует `DocumentImportError` и `DocumentProcessingError`.

## Attachment Intelligence (интеллект вложений)

### AttachmentIntelligenceService

Сервис классификации вложений (`AttachmentIntelligenceService`) предоставляет единственный метод:

- **classify** — принимает `AttachmentIntelligenceInput` (attachment_id, filename, content_type, size_bytes) и возвращает `AttachmentClassification`:
  - **category** — определяется по имени файла и content-type (см. ниже).
  - **is_executable** — проверяется по списку исполняемых content-type и расширений.
  - **is_archive** — проверяется по списку архивных типов и расширений.
  - **is_document** — проверяется по списку документных типов и расширений.
  - **risk_level**:
    - `High`, если файл определён как исполняемый.
    - `Medium`, если файл является архивом.
    - `Safe` в остальных случаях.
  - **summary** — строка вида `"filename (size MB) - category"`.

### Классификация по имени и типу

Функция `classify_by_name_and_type` применяет эвристики:

- Имя содержит `"invoice"`, `"factura"`, `"receipt"` → категория `Invoice`.
- `"contract"`, `"agreement"`, `"nda"` → `Contract`.
- `"certificate"`, `"cert"` → `Certificate`.
- `"tax"`, `"hacienda"`, `"aeat"` → `TaxDocument`.
- `"passport"`, `"dni"`, `"nie"` → `IdentityDocument`.
- `"report"` → `Report`. Также `application/pdf` без других совпадений засчитывается как `Report`.
- `"presentation"` или расширение `.pptx`, `.ppt` → `Presentation`.
- Расширения `.xlsx`, `.xls`, `.csv` → `Spreadsheet`.
- Расширения исходного кода `.rs`, `.py`, `.js`, `.ts`, `.go`, `.java`, `.c`, `.cpp` → `SourceCode`.
- Архивные расширения `.zip`, `.rar`, `.7z`, `.tar.gz`, `.tar` → `Archive`.
- Если content-type начинается с `"image/"`:
  - При наличии `"screenshot"` или `"screen"` → `Screenshot`.
  - Иначе → `Image`.
- Всё остальное → `Unknown`.

### Определение типов файлов (file_kinds)

**is_executable_type**: проверяет content-type (например `application/x-msdownload`) или расширения (`.exe`, `.dll`, `.sh`, `.bat`, `.cmd`, `.msi`, `.app`, `.bin`).

**is_archive_type**: content-type (например `application/zip`, `application/x-rar-compressed`) или расширения (`.zip`, `.rar`, `.7z`, `.tar`, `.gz`, `.bz2`, `.xz`, `.tar.gz`).

**is_document_type**: content-type (например `application/pdf`, `application/msword`, `text/plain`, `text/markdown`) или расширения (`.pdf`, `.doc`, `.docx`, `.xls`, `.xlsx`, `.txt`, `.md`, `.csv`).

### Модели

- `AttachmentCategory` — enum с 14 вариантами от `Invoice` до `Unknown`.
- `RiskLevel` — пять уровней: `Safe`, `Low`, `Medium`, `High`, `Critical` (в текущей логике используются только Safe/Medium/High).
- `AttachmentIntelligenceError` — содержит только вариант `NotFound` (в предоставленном коде не используется).
- `AttachmentIntelligenceInput` — вход классификатора: attachment_id, опциональный filename, content_type, size_bytes.
- `AttachmentClassification` — результат классификации.

### Тесты (примеры)

- Файл `Invoice_2026_001.pdf` классифицируется как `invoice`.
- Файл `NDA_Acme_Corp.docx` — как `contract`.
- `documents.zip` — как `archive`, уровень риска `medium`.
- `setup.exe` — как исполняемый, риск `high`.
- Файл `photo.jpg` — как `image`, риск `safe`.
- `main.rs` — как `source_code`.

## Document Processing (обработка документов)

### DocumentProcessingStore

Хранилище управления обработкой документов (`DocumentProcessingStore`) отвечает за задания и артефакты. В предоставленном контексте видны ключевые операции:

- **ensure_document_exists** — проверяет, что документ с указанным ID существует.
- **document_for_id** / **document_record_by_id** — читает запись документа (document_kind, extracted_text) из таблицы `documents`.
- **upsert_artifact** — сохраняет текстовый артефакт обработки в таблицу `document_artifacts`:
  - Вычисляет SHA-256 хеш контента.
  - Идентификатор артефакта строится с префиксом `document_artifact:v1:`.
  - ON CONFLICT по (document_id, artifact_kind) обновляет содержимое и job_id.

### Идентификаторы (IDs)

- `job_id(document_id, step)` — формат: `document_processing_job:v1:{document_id}:{step}`.
- `artifact_id(document_id, artifact_kind)` — формат: `document_artifact:v1:{document_id}:{artifact_kind}`.

### Константы

- Лимиты списков: по умолчанию 50, минимум 1, максимум 100.
- Максимальное количество попыток выполнения задания: `DEFAULT_MAX_ATTEMPTS = 3`.
- Префиксы для событий повторной обработки: `RETRY_EVENT_TYPE = "document_processing.retry_requested"`, `RETRY_SOURCE_KIND = "document_processing_retry"`.

### Ошибки

`DocumentProcessingError` включает варианты: `InvalidLimit`, `EmptyField`, `JobNotFound`, `RetryRequiresFailedJob`, `RetryCommandConflict`, `DocumentNotFound`, `InvalidStep`, `InvalidStatus`, `InvalidArtifactKind`, `MissingSourceText`, `OcrBackendUnavailable`, а также обёртки для `EventEnvelopeError`, `EventStoreError`, `ObservationStoreError` и `sqlx::Error`.

### Связывание с наблюдениями (evidence)

Модуль `evidence` предоставляет функцию `link_document_processing_entity_in_transaction`, которая делегирует в платформенный метод `link_domain_entity_in_transaction` с фиксированным доменом `"documents"`.

### Публичный интерфейс модуля

Модуль `processing` реэкспортирует:
- `DocumentProcessingCommandService` (командный сервис; реализация не включена в данный контекст).
- `DocumentProcessingStore` (хранилище).
- Модели: `DocumentProcessingJob`, `DocumentProcessingRecord`, `DocumentProcessingArtifact`, `DocumentProcessingRetryCommand`, `DocumentProcessingRunReport`, `DocumentProcessingStatus`, `DocumentProcessingStep`.
- Типы ошибок.
```

## Покрытие источников

| Source file | Facts covered |
|---|---|
| `backend/src/domains/decisions/service.rs` | `DecisionCommandService` создаётся из `PgPool`; метод `review_manual` создаёт observation REVIEW_TRANSITION через `ObservationStore`, затем вызывает `DecisionStore::set_review_state_with_observation`. |
| `backend/src/domains/decisions/store.rs` | `DecisionStore::upsert_with_evidence` (вставка/обновление решения, evidence, impacted entities, проверка существования observations); `list_for_entity` (соединение по decided_by_entity и impacted_entities); `list_by_review_state`; `set_review_state_with_observation` (обновление review_state и связывание observation). |
| `backend/src/domains/decisions/validation.rs` | Функции валидации: `validate_decision_with_evidence`, `preserve_existing_review_state`, `validate_non_empty`, `validate_score`, `validate_json_object`, `validate_json_array`, `validate_refresh_limit`. |
| `backend/src/domains/documents/attachment_intelligence.rs` | `AttachmentIntelligenceService::classify` – логика: определение category через classify_by_name_and_type, флаги is_executable/is_archive/is_document, уровень риска, summary. |
| `backend/src/domains/documents/attachment_intelligence/classification.rs` | `classify_by_name_and_type` – эвристики по ключевым словам в имени файла и content-type; логика отнесения к `SourceCode` и `Archive` по расширениям. |
| `backend/src/domains/documents/attachment_intelligence/file_kinds.rs` | `is_executable_type`, `is_archive_type`, `is_document_type` – проверка по спискам content-type и расширений. |
| `backend/src/domains/documents/attachment_intelligence/models.rs` | Перечисления `AttachmentCategory`, `RiskLevel` и их строковые представления; структуры `AttachmentIntelligenceInput`, `AttachmentClassification`; ошибка `AttachmentIntelligenceError`. |
| `backend/src/domains/documents/attachment_intelligence/tests.rs` | Тесты классификации: invoice, contract, archive, executable high risk, safe image, source code. |
| `backend/src/domains/documents/core.rs` | Публичный API модуля: реэкспорт `DocumentImportStore`, `NewDocumentImport`, `ImportedDocument`, `ImportedDocumentWithProcessing`, ошибок. |
| `backend/src/domains/documents/core/errors.rs` | `DocumentImportError` (EmptyField, InvalidDocumentKind, DocumentKindChange, UpsertSkipped, Sqlx, ObservationStore); `DocumentImportWithProcessingError`. |
| `backend/src/domains/documents/core/evidence.rs` | `link_document_entity_in_transaction` – обёртка над платформенным `link_domain_entity_in_transaction` с domain="documents", entity_kind="document". |
| `backend/src/domains/documents/core/fingerprint.rs` | `local_markdown_fingerprint` – FNV-1a с константами, формат `local-v1:markdown:{hash:016x}`. |
| `backend/src/domains/documents/core/markdown.rs` | `extract_markdown_text` – удаление ATX-заголовков (1–6 `#` с пробелом) и сбор оставшегося текста. |
| `backend/src/domains/documents/core/models.rs` | `NewDocumentImport` с конструкторами `markdown` и `pdf_metadata`; `ImportedDocument`, `ImportedDocumentWithProcessing`; константы `DOCUMENT_KIND_MARKDOWN`, `DOCUMENT_KIND_PDF`. |
| `backend/src/domains/documents/core/rows.rs` | `row_to_imported_document` – маппинг строки PgRow в `ImportedDocument`. |
| `backend/src/domains/documents/core/store.rs` | `DocumentImportStore::import_document`, `import_document_and_enqueue_processing`; логика создания observation DOCUMENT, upsert в `documents` с проверкой неизменности document_kind, связывание с observation. |
| `backend/src/domains/documents/core/validation.rs` | `validate_document_import` – проверка непустых полей, допустимых значений document_kind. |
| `backend/src/domains/documents/mod.rs` | Структура модуля: `attachment_intelligence`, `core`, `processing`. |
| `backend/src/domains/documents/processing.rs` | Публичный API `processing` – реэкспорт важнейших типов. |
| `backend/src/domains/documents/processing/artifacts.rs` | `DocumentProcessingStore::upsert_artifact` – сохранение текстового артефакта с SHA-256 хешем и конфликтным разрешением. |
| `backend/src/domains/documents/processing/constants.rs` | Константы: лимиты списков, максимальное число попыток, префиксы job/artifact/retry, тип события retry. |
| `backend/src/domains/documents/processing/documents.rs` | Методы `ensure_document_exists`, `document_for_id`, `document_record_by_id`; структура `DocumentRecord`. |
| `backend/src/domains/documents/processing/errors.rs` | `DocumentProcessingError` с вариантами InvalidLimit, EmptyField, JobNotFound, RetryRequiresFailedJob, OcrBackendUnavailable и обёртками. |
| `backend/src/domains/documents/processing/evidence.rs` | `link_document_processing_entity_in_transaction` – связывание observation с сущностями домена "documents". |
| `backend/src/domains/documents/processing/ids.rs` | Формирование идентификаторов `job_id` и `artifact_id`. |

## Исходные файлы

- [`backend/src/domains/decisions/service.rs`](../../../../backend/src/domains/decisions/service.rs)
- [`backend/src/domains/decisions/store.rs`](../../../../backend/src/domains/decisions/store.rs)
- [`backend/src/domains/decisions/validation.rs`](../../../../backend/src/domains/decisions/validation.rs)
- [`backend/src/domains/documents/attachment_intelligence.rs`](../../../../backend/src/domains/documents/attachment_intelligence.rs)
- [`backend/src/domains/documents/attachment_intelligence/classification.rs`](../../../../backend/src/domains/documents/attachment_intelligence/classification.rs)
- [`backend/src/domains/documents/attachment_intelligence/file_kinds.rs`](../../../../backend/src/domains/documents/attachment_intelligence/file_kinds.rs)
- [`backend/src/domains/documents/attachment_intelligence/models.rs`](../../../../backend/src/domains/documents/attachment_intelligence/models.rs)
- [`backend/src/domains/documents/attachment_intelligence/tests.rs`](../../../../backend/src/domains/documents/attachment_intelligence/tests.rs)
- [`backend/src/domains/documents/core.rs`](../../../../backend/src/domains/documents/core.rs)
- [`backend/src/domains/documents/core/errors.rs`](../../../../backend/src/domains/documents/core/errors.rs)
- [`backend/src/domains/documents/core/evidence.rs`](../../../../backend/src/domains/documents/core/evidence.rs)
- [`backend/src/domains/documents/core/fingerprint.rs`](../../../../backend/src/domains/documents/core/fingerprint.rs)
- [`backend/src/domains/documents/core/markdown.rs`](../../../../backend/src/domains/documents/core/markdown.rs)
- [`backend/src/domains/documents/core/models.rs`](../../../../backend/src/domains/documents/core/models.rs)
- [`backend/src/domains/documents/core/rows.rs`](../../../../backend/src/domains/documents/core/rows.rs)
- [`backend/src/domains/documents/core/store.rs`](../../../../backend/src/domains/documents/core/store.rs)
- [`backend/src/domains/documents/core/validation.rs`](../../../../backend/src/domains/documents/core/validation.rs)
- [`backend/src/domains/documents/mod.rs`](../../../../backend/src/domains/documents/mod.rs)
- [`backend/src/domains/documents/processing.rs`](../../../../backend/src/domains/documents/processing.rs)
- [`backend/src/domains/documents/processing/artifacts.rs`](../../../../backend/src/domains/documents/processing/artifacts.rs)
- [`backend/src/domains/documents/processing/constants.rs`](../../../../backend/src/domains/documents/processing/constants.rs)
- [`backend/src/domains/documents/processing/documents.rs`](../../../../backend/src/domains/documents/processing/documents.rs)
- [`backend/src/domains/documents/processing/errors.rs`](../../../../backend/src/domains/documents/processing/errors.rs)
- [`backend/src/domains/documents/processing/evidence.rs`](../../../../backend/src/domains/documents/processing/evidence.rs)
- [`backend/src/domains/documents/processing/ids.rs`](../../../../backend/src/domains/documents/processing/ids.rs)

## Кандидаты на drift

- **Недостающие определения типов в decisions**: `DecisionReviewState`, `DecisionEntityKind`, `NewDecisionEvidence`, `NewDecisionImpactedEntity`, `DecisionStoreError` используются, но не определены в предоставленных файлах. Любое изменение их полей или вариантов может нарушить документированную здесь логику, но из данного контекста это не подтверждено.
- **Функция `preserve_existing_review_state`**: объявлена, но в данном наборе нет вызовов. Возможен drift: если она не используется, описание её поведения может быть неактуальным.
- **`AttachmentIntelligenceError::NotFound`**: определён, но не используется в предоставленном коде `attachment_intelligence`. Потенциально мёртвый код или используется в других модулях, не включённых в контекст.
- **`DocumentProcessingError::OcrBackendUnavailable`**: вариант ошибки, но никакой OCR-логики в предоставленных файлах нет. Если OCR удалён или переименован, этот вариант станет drift-ом.
- **Константы `MIN_REFRESH_LIMIT`, `MAX_REFRESH_LIMIT`**: используются в валидации решений, но их значения не видны. Любое изменение этих значений сделает документированный диапазон невалидным.
- **`DocumentProcessingCommandService`**: объявлен в `processing.rs` как публичный, но реализация не включена в чанк. Документация его поведения не может быть проверена по данному контексту.
- **Структуры `DocumentProcessingJob`, `DocumentProcessingStep`, и др.**: реэкспортируются, но их точные поля и состояния не видны. Документированные здесь ссылки на них опираются только на публичный API, что может не совпадать с реальным контрактом.
