---
chunk_id: 029-source-backend-part-009
batch_id: batch-20260628T214902
group: backend
role: source
source_status: pending
source_count: 25
generated_by: code-wiki-ru
---

# 029-source-backend-part-009 — backend/source

- Target index: [[components/backend]]
- Batch: `batch-20260628T214902`
- Source files: `25`

## Резюме

В русскую wiki добавляется страница `components/backend.md`, документирующая backend-компонент приложения `makosh` на основе предоставленных обработчиков. Страница описывает группу API-обработчиков, относящихся к коммуникациям (отправка, перевод, шаблоны, локальное состояние), рабочим действиям (workflow actions), проверке согласованности и решениям. Факты взяты строго из встроенных исходных файлов данного чанка.

## Предложенные страницы

#### `components/backend.md`

```markdown
# Backend

## Обзор

Backend-компонент `makosh` предоставляет REST API для работы с коммуникационными сообщениями, рабочими процессами, согласованностью данных и автоматизированными решениями. Ниже описаны обработчики, обнаруженные в данном срезе исходного кода.

## Обработчики API

### Коммуникации (`communications`)

#### `bilingual_reply_flow` — двуязычный ответ

- `POST /v1/bilingual_reply_flow/:message_id`
  - Принимает `reply_text_ru` (текст ответа на русском) и `tone` (тон письма).
  - Поддерживаемые тона: `formal`, `business`, `friendly`, `short`, `detailed`.
  - Максимальная длина текста ответа: `64 000` символов.
  - Определяет язык исходного сообщения через `MultilingualService::detect_language`.
  - Выполняет перевод исходного сообщения на русский и обратный перевод ответа на язык оригинала (или на английский, если язык не определён).
  - Каждый шаг перевода отправляет сигнал в AI‑хаб (события `bilingual_reply_inbound_translation` и `bilingual_reply_back_translation`).
  - Флаг `send_ready` устанавливается, только если оба перевода успешны.
  - Ответ включает оригинальный текст, переводы и черновик ответа с темой (префикс `Re:` добавляется при необходимости).

#### `forwarding` — ответ и пересылка

- `POST /v1/reply/:message_id`
  - Для WhatsApp: преобразует запрос, вызывает `post_whatsapp_command_reply`, возвращает ответ провайдера.
  - Для Telegram: вызывает `telegram_message_write_service().reply_to_message(...)`.
  - Для остальных каналов: формирует цитируемое тело ответа в стиле «On ..., ... wrote:» и возвращает локальный `SendResponse` со статусом `queued`.
- `POST /v1/forward/:message_id`
  - Аналогичная маршрутизация для WhatsApp (`post_whatsapp_command_forward`) и Telegram.
  - Для остальных: формирует тело пересылки с заголовком `--- Forwarded message ---` и возвращает предпросмотр.
- `POST /v1/reply_all/:message_id`
  - Принимает `ReplyAllRequest` (текст ответа, флаг цитирования).
  - Формирует тело ответа с цитированием через `build_reply_body`. Возвращает список получателей и тему `Re: ...`.
- `POST /v1/forward_eml/:message_id`
  - Принимает `ForwardEmlRequest` (список получателей).
  - Строит EML‑представление и возвращает его размер.
- `POST /v1/redirect/:message_id`
  - Принимает `RedirectRequest` (получатели `to`, `cc`, `bcc`, подтверждение `confirmed_provider_write`).
  - Требует `confirmed_provider_write: true` и хотя бы одного получателя.
  - Ставит перенаправленное сообщение в очередь через `CommunicationCommandService`.

#### `provider_send` — отправка письма

- `POST /v1/send`
  - Принимает `SendRequest` (получатели, тема, тело в текстовом и HTML‑виде, `in_reply_to`, `references`, настройки отложенной отправки и отмены).
  - Требует `confirmed_provider_write: true`.
  - Вызывает `send_email` с зависимостями `CommunicationSendDependencies`.
  - Возвращает `SendResponse` с идентификатором сообщения, статусом, транспортом и т.д.

#### `certificates` — сертификаты и проверки подлинности

- `GET /v1/certs` — список всех сертификатов.
- `POST /v1/cert` — создание/обновление сертификата.
  - Поля: `cert_id`, `owner_name`, `issuer`, `serial_number`, `fingerprint_sha256`, `valid_from`, `valid_until`, `cert_type`, `provider`, `storage_kind`, `storage_ref`, `trust_status`, `is_revoked`, `usage`, `linked_message_id`, `metadata`.
  - Для типов, провайдеров, хранилищ и статуса доверия заданы значения по умолчанию (например, `CertificateStorageKind::EncryptedVault`, `TrustStatus::Untrusted`).
- `GET /v1/certs/expiring?days=90` — сертификаты, истекающие в ближайшие N дней (по умолчанию 90).
- `GET /v1/signature_check/:message_id` — поиск цифровой подписи в теле сообщения.
- `GET /v1/spf_dkim/:message_id` — разбор заголовков аутентификации (SPF/DKIM) и оценка риска.

#### `extraction` — извлечение задач и заметок

- `POST /v1/extract_tasks/:message_id`
  - Использует `EmailExtractService` с опциональным AI‑рантаймом.
  - При наличии задач, извлечённых с помощью LLM, отправляет сигнал `message_task_extraction` в AI‑хаб.
- `POST /v1/extract_notes/:message_id`
  - Извлекает заметки без использования LLM (`EmailExtractService::new(None)`).

#### `multilingual` — многоязычные операции

- `GET /v1/detect_language/:message_id` — определяет язык сообщения.
- `POST /v1/translate/:message_id` — переводит тело сообщения на указанный язык.
  - При успехе отправляет сигнал `message_translation`.
- `POST /v1/translate_attachment/:attachment_id`
  - Принимает `target_language` и `source_text`.
  - Максимальная длина исходного текста: `64 000` символов (константа `MAX_ATTACHMENT_TRANSLATION_SOURCE_CHARS`).
  - Переводит извлечённый из вложения текст и отправляет сигнал `attachment_translation`.
- `POST /v1/translate_thread?account_id=...&subject=...`
  - Загружает все сообщения цепочки (до 50 по умолчанию), переводит каждое.
  - На каждое сообщение отправляет сигнал `thread_message_translation`.

Во всех переводах: если LLM не настроен — `reason: "no LLM configured"`; при ошибке выполнения — `reason: "translation runtime unavailable"`.

#### `local_state` — локальное состояние

- `POST /v1/imap_mark_read/:message_id` — помечает сообщение как прочитанное через IMAP.
- `POST /v1/imap_delete/:message_id` — перемещает сообщение в локальную корзину (статус `imap_delete_alias`).
- `POST /v1/message_trash/:message_id` — перемещает в корзину как удалённое пользователем (`user_deleted`).
- `POST /v1/message_restore/:message_id` — восстанавливает из локальной корзины.

### Шаблоны и статус (`templates_status`)

- `GET /v1/rich_templates` — список шаблонов.
- `POST /v1/rich_template` — создание/обновление шаблона.
  - Если `template_id` не передан, генерируется `mail_template:{timestamp}`.
  - Подстановки: `subject_template` (fallback на `content`), `body_template` (fallback на `content`).
- `DELETE /v1/rich_template/:template_id` — удаление шаблона.
- `POST /v1/render_template` — рендеринг шаблона с переданными переменными.
- `POST /v1/rich_template/mail_merge_preview` — предпросмотр слияния до 250 строк.
- `GET /v1/blockers` — список архитектурных блокировок.
- `GET /v1/status` — статус API (версия `1.0`), доступные поверхности (messages, persons, search, documents, account_setup) и статус хранилища (vault).
- Хранилище (vault):
  - `GET /v1/vault_status`
  - `POST /v1/vault/collect_entropy`
  - `POST /v1/vault/create`
  - `POST /v1/vault/unlock`
  - `POST /v1/vault/recovery_export`
  - `POST /v1/vault/recovery_import`

### Рабочие действия (`workflow_actions`)

- `POST /v1/workflow_action` — выполнение рабочего действия.
  - Принимает `WorkflowActionRequest` с `command_id`, `action`, опциональными `source` (сообщение‑источник) и `input`.
  - Поддерживаемые действия (`WorkflowActionKind`):
    - `Reply` — открыть окно ответа.
    - `CreateTask` — создать задачу.
    - `CreateNote` — создать заметку.
    - `CreateDocument` — создать документ.
    - `CreateEvent` — создать событие календаря (требует `starts_at` и `ends_at`).
    - `LinkDocument` — связать сообщение с документом.
    - `CreateContact` — создать контакт.
    - `Archive` — архивировать сообщение (с проверкой допустимости перехода состояния).
  - Каждое действие выполняется в транзакции, результат записывается как событие `workflow.action_executed` в хранилище событий.
  - Обеспечена идемпотентность: если событие с `event_id = "workflow_action:{command_id}"` уже существует, возвращается сохранённый результат.
  - Идентификатор актора извлекается из заголовка `x-makosh-actor-id` (по умолчанию `makosh-frontend`).

### Состояния рабочего процесса (`workflow_state`)

- `PUT /v1/message_workflow_state/:message_id` — переход в новое состояние `workflow_state` (с аудитом).
- `GET /v1/message_workflow_state_counts` — количество сообщений по состояниям (фильтры: `account_id`, `local_state`).
- `POST /v1/message_analyze/:message_id` — эвристический анализ сообщения.
  - Устанавливает AI‑состояние в `Processing`, затем вычисляет `importance_score`, категорию и структурированную сводку.
  - Если `importance_score >= 75` и текущее состояние `"new"`, автоматически переводит в `NeedsAction`.
  - После анализа AI‑состояние становится `Processed`.
  - Обновляет кандидатов на обзор знаний (`refresh_message_knowledge_candidates_into_review`).
  - Возвращает категорию, сводку, оценку важности и список причин (`evidence`).

### Согласованность (`consistency`)

- `GET /v1/contradictions?limit=50` — список открытых противоречий (лимит 1–100).
- `PUT /v1/contradiction_review/:observation_id`
  - Принимает `review_state` и опциональное `resolution`.
  - Фиксирует аудит, вызывает `ContradictionReviewService::review_manual`.

### Решения (`decisions`)

- `GET /v1/decisions` — список решений с фильтрами:
  - `review_state` (нельзя комбинировать с фильтрами по сущности).
  - `entity_kind` и `entity_id`.
- `PUT /v1/decision_review/:decision_id`
  - Принимает `review_state`.
  - Фиксирует аудит, вызывает `DecisionReviewApplicationService::review_manual`.
```

## Покрытие источников

- `backend/src/app/handlers/communications/sending/bilingual_reply_flow.rs` — эндпоинт `POST /v1/bilingual_reply_flow/:message_id`, константа `MAX_BILINGUAL_REPLY_TEXT_CHARS` (64 000), допустимые тона `BILINGUAL_REPLY_TONES`, нормализация тона, определение языка входящего сообщения, переводы с сигналами AI‑хаба, флаг `send_ready`, построение темы с `Re:`.
- `backend/src/app/handlers/communications/sending/certificates.rs` — эндпоинты `GET /v1/certs`, `POST /v1/cert`, `GET /v1/certs/expiring`, `GET /v1/signature_check/:message_id`, `GET /v1/spf_dkim/:message_id`; поля сертификатов, значения по умолчанию для типов и хранилищ, парсинг `cert_type`/`provider`/`storage_kind`/`trust_status`, разбор SPF/DKIM и оценка риска.
- `backend/src/app/handlers/communications/sending/extraction.rs` — эндпоинты `POST /v1/extract_tasks/:message_id` и `POST /v1/extract_notes/:message_id`; создание `EmailExtractService` с опциональным AI‑рантаймом; диспетчеризация сигнала `message_task_extraction` только при наличии LLM‑задач.
- `backend/src/app/handlers/communications/sending/forwarding.rs` — эндпоинты `POST /v1/reply/:message_id`, `POST /v1/forward/:message_id`, `POST /v1/reply_all/:message_id`, `POST /v1/forward_eml/:message_id`, `POST /v1/redirect/:message_id`; маршрутизация по каналу (WhatsApp, Telegram, локальный email), цитирование/пересылка для email, требование `confirmed_provider_write` для редиректа, постановка в очередь через `CommunicationCommandService`.
- `backend/src/app/handlers/communications/sending/local_state.rs` — эндпоинты `POST /v1/imap_mark_read/:message_id`, `POST /v1/imap_delete/:message_id`, `POST /v1/message_trash/:message_id`, `POST /v1/message_restore/:message_id`; использование `CommunicationCommandService` для изменения локального состояния.
- `backend/src/app/handlers/communications/sending/multilingual.rs` — эндпоинты `GET /v1/detect_language/:message_id`, `POST /v1/translate/:message_id`, `POST /v1/translate_attachment/:attachment_id`, `POST /v1/translate_thread`; константа `MAX_ATTACHMENT_TRANSLATION_SOURCE_CHARS` (64 000); отправка сигналов `message_translation`, `attachment_translation`, `thread_message_translation`; причины `"no LLM configured"` и `"translation runtime unavailable"`.
- `backend/src/app/handlers/communications/sending/provider_send.rs` — эндпоинт `POST /v1/send`; требование `confirmed_provider_write`, вызов `send_email`, маппинг ошибок `CommunicationSendError` → `ApiError`.
- `backend/src/app/handlers/communications/templates_status.rs` — эндпоинты шаблонов (CRUD, рендеринг, mail merge до 250 строк), блокировок (`GET /v1/blockers`), статуса (`GET /v1/status` с версией `1.0` и поверхностями), операций с хранилищем (`vault`).
- `backend/src/app/handlers/communications/workflow_actions/` (включая `actions/`, `handler.rs`, `models.rs`, `response.rs`, `source.rs`, `validation.rs`) — эндпоинт `POST /v1/workflow_action`; идемпотентность через event store, транзакционное выполнение действий (`WorkflowActionKind`: 8 видов), загрузка сообщения‑источника, валидация и извлечение заголовка `x-makosh-actor-id`, сохранение результата как события в `EventStore`.
- `backend/src/app/handlers/communications/workflow_state.rs` — эндпоинты `PUT /v1/message_workflow_state/:message_id`, `GET /v1/message_workflow_state_counts`, `POST /v1/message_analyze/:message_id`; эвристический анализ с автоматическим переходом в `NeedsAction` при high score, обновление кандидатов на обзор знаний, структура ответа анализа.
- `backend/src/app/handlers/consistency.rs` — эндпоинты `GET /v1/contradictions` и `PUT /v1/contradiction_review/:observation_id`; валидация лимита (1–100), использование `ContradictionReviewService`.
- `backend/src/app/handlers/decisions/handlers.rs` — эндпоинты `GET /v1/decisions` и `PUT /v1/decision_review/:decision_id`; валидация лимита (1–100), парсинг `review_state` и `entity_kind`, аудит.

## Исходные файлы

- [`backend/src/app/handlers/communications/sending/bilingual_reply_flow.rs`](../../../../backend/src/app/handlers/communications/sending/bilingual_reply_flow.rs)
- [`backend/src/app/handlers/communications/sending/certificates.rs`](../../../../backend/src/app/handlers/communications/sending/certificates.rs)
- [`backend/src/app/handlers/communications/sending/extraction.rs`](../../../../backend/src/app/handlers/communications/sending/extraction.rs)
- [`backend/src/app/handlers/communications/sending/forwarding.rs`](../../../../backend/src/app/handlers/communications/sending/forwarding.rs)
- [`backend/src/app/handlers/communications/sending/local_state.rs`](../../../../backend/src/app/handlers/communications/sending/local_state.rs)
- [`backend/src/app/handlers/communications/sending/multilingual.rs`](../../../../backend/src/app/handlers/communications/sending/multilingual.rs)
- [`backend/src/app/handlers/communications/sending/provider_send.rs`](../../../../backend/src/app/handlers/communications/sending/provider_send.rs)
- [`backend/src/app/handlers/communications/templates_status.rs`](../../../../backend/src/app/handlers/communications/templates_status.rs)
- [`backend/src/app/handlers/communications/workflow_actions.rs`](../../../../backend/src/app/handlers/communications/workflow_actions.rs)
- [`backend/src/app/handlers/communications/workflow_actions/actions.rs`](../../../../backend/src/app/handlers/communications/workflow_actions/actions.rs)
- [`backend/src/app/handlers/communications/workflow_actions/actions/archive.rs`](../../../../backend/src/app/handlers/communications/workflow_actions/actions/archive.rs)
- [`backend/src/app/handlers/communications/workflow_actions/actions/calendar.rs`](../../../../backend/src/app/handlers/communications/workflow_actions/actions/calendar.rs)
- [`backend/src/app/handlers/communications/workflow_actions/actions/documents.rs`](../../../../backend/src/app/handlers/communications/workflow_actions/actions/documents.rs)
- [`backend/src/app/handlers/communications/workflow_actions/actions/persons.rs`](../../../../backend/src/app/handlers/communications/workflow_actions/actions/persons.rs)
- [`backend/src/app/handlers/communications/workflow_actions/actions/reply.rs`](../../../../backend/src/app/handlers/communications/workflow_actions/actions/reply.rs)
- [`backend/src/app/handlers/communications/workflow_actions/actions/tasks.rs`](../../../../backend/src/app/handlers/communications/workflow_actions/actions/tasks.rs)
- [`backend/src/app/handlers/communications/workflow_actions/constants.rs`](../../../../backend/src/app/handlers/communications/workflow_actions/constants.rs)
- [`backend/src/app/handlers/communications/workflow_actions/handler.rs`](../../../../backend/src/app/handlers/communications/workflow_actions/handler.rs)
- [`backend/src/app/handlers/communications/workflow_actions/models.rs`](../../../../backend/src/app/handlers/communications/workflow_actions/models.rs)
- [`backend/src/app/handlers/communications/workflow_actions/response.rs`](../../../../backend/src/app/handlers/communications/workflow_actions/response.rs)
- [`backend/src/app/handlers/communications/workflow_actions/source.rs`](../../../../backend/src/app/handlers/communications/workflow_actions/source.rs)
- [`backend/src/app/handlers/communications/workflow_actions/validation.rs`](../../../../backend/src/app/handlers/communications/workflow_actions/validation.rs)
- [`backend/src/app/handlers/communications/workflow_state.rs`](../../../../backend/src/app/handlers/communications/workflow_state.rs)
- [`backend/src/app/handlers/consistency.rs`](../../../../backend/src/app/handlers/consistency.rs)
- [`backend/src/app/handlers/decisions/handlers.rs`](../../../../backend/src/app/handlers/decisions/handlers.rs)

## Кандидаты на drift

1. Файл `forwarding.rs` был обрезан после ~12 000 символов, из-за чего полная реализация некоторых эндпоинтов (например, `post_v1_redirect`) не видна. Нельзя подтвердить полную структуру ответа или дополнительные проверки.
2. Файл `multilingual.rs` был обрезан после ~12 000 символов, из-за чего цикл обработки сообщений цепочки (и часть сигналов) может быть неполным. Реальное поведение не может быть подтверждено полностью.
3. В `workflow_state.rs` объявлены структуры `ThreadListQuery`, `ThreadListResponse`, `ThreadMessagesQuery`, `ThreadMessagesResponse`, но обработчики для них не включены в данный чанк. Наличие соответствующих эндпоинтов не подтверждено предоставленным контекстом.
4. Некоторые обработчики ссылаются на сервисы и хранилища из других crate‑ов (например, `WhatsAppProviderCommandResponse`, `telegram_message_write_service`), определения которых не входят в чанк. Любые детали их поведения за пределами видимых вызовов не подтверждены.
5. Признаков расхождения между предоставленным кодом и существующей документацией не обнаружено, поскольку отсутствуют встроенные страницы wiki для сравнения.
