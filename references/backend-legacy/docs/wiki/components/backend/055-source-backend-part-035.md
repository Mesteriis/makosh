---
chunk_id: 055-source-backend-part-035
batch_id: batch-20260628T214902
group: backend
role: source
source_status: pending
source_count: 25
generated_by: code-wiki-ru
---

# 055-source-backend-part-035 — backend/source

- Target index: [[components/backend]]
- Batch: `batch-20260628T214902`
- Source files: `25`

## Резюме

Добавляется (или обновляется) страница `components/backend.md` русской wiki на основе анализа исходного кода пяти движков (`engines`): автоматизации (`automation`), аналитики звонков (`call_intelligence`), поиска противоречий (`consistency`), контекстных пакетов (`context_packs`) и обогащения (`enrichment`). Каждый движок описывается через его ответственность, ключевые модели данных, основные операции и источники.

## Предложенные страницы

#### `components/backend.md`

```markdown
# Компоненты бэкенда

> Страница описывает движки обработки (`engines`), доступные в бэкенде Макошь.
> Информация основана на коде из `backend/src/engines/`.
> Смежные подсистемы (API-слой, планировщики, платформенные сервисы) здесь не описаны – их покрывают отдельные страницы.

---

## Automation (автоматизация)

**Путь:** `backend/src/engines/automation/`

Управляет шаблонами сообщений (`automation_templates`) и политиками отправки (`automation_policies`) для Telegram-автоматизаций.

### Модели

- **`AutomationTemplate`** – шаблон сообщения:
  - `template_id`, `name`, `body_template`, `required_variables` (список строк).
- **`AutomationPolicy`** – политика отправки:
  - `policy_id`, `template_id`, `name`, `enabled`, `account_id`, `allowed_chat_ids` (JSON-массив), `trigger_kind`, `max_sends_per_hour`, `quiet_hours` (JSON), `expires_at`, `conditions` (JSON).
- **`TelegramSendDryRunRequest`** – запрос на пробную отправку:
  - `command_id`, `policy_id`, `provider_chat_id`, `variables` (JSON), `source_context` (JSON).

### Основные операции (`AutomationStore`)

| Метод | Назначение |
|---|---|
| `upsert_template` | Вставка/обновление шаблона, с валидацией и записью наблюдения |
| `upsert_policy` | Вставка/обновление политики, аналогично с наблюдением |
| `list_templates` | Список всех шаблонов (сортировка `updated_at DESC, template_id ASC`) |
| `list_policies` | Список всех политик (аналогично) |
| `dry_run_send` | Пробная отправка (делегирует в `super::dry_run::dry_run_send`) |
| `policy_with_template` | Получение политики вместе с привязанным шаблоном через `JOIN` |

### Валидация (`validation`)

- Все строковые поля обязательны (trim + non‑empty).
- `max_sends_per_hour > 0`.
- `allowed_chat_ids` не пуст.
- `quiet_hours` и `conditions` должны быть JSON-объектами.
- Имена переменных шаблона (`required_variables`) – только ASCII буквы, цифры и `_`.
- Для `TelegramSendDryRunRequest`: все поля непустые, `variables` и `source_context` – JSON-объекты.

> **Не подтверждено контекстом:** детали фактической отправки (реализация `dry_run_send`), логика триггеров `trigger_kind`.

---

## Call Intelligence (аналитика звонков)

**Путь:** `backend/src/engines/call_intelligence/`

Строит план обработки (`CallIntelligencePipelinePlan`) на основе манифеста бандла звонка (`CallBundleManifest`).

### Модели

- **`CallIntelligenceArtifactRequirement`** – требование к артефакту:
  - `kind` (имя артефакта), `required` (обязателен), `purpose` (назначение).
- **`CallIntelligenceStep`** – шаг пайплайна:
  - `step_id`, `title`, `input_artifacts`, `output_artifacts`, `source_of_truth_policy` (политика доверия к источнику).
- **`CallIntelligencePipelinePlan`** – полный план:
  - `bundle_id`, `requirements` (вектор требований), `steps` (вектор шагов).
- **`CallIntelligenceOutputCandidate`** – результат кандидата:
  - `candidate_kind`, `title`, `confidence`, `evidence`.

### План по умолчанию (`plan_from_bundle`)

**Требования к артефактам:**

| Артефакт | Обязательный | Назначение |
|---|---|---|
| `audio.mp3` | да | транскрипция и диаризация |
| `speaker-hints.jsonl` | нет | warm‑start числа спикеров и меток |
| `screenshots` | нет | screen intelligence, OCR, визуальные доказательства |
| `chat.json` | нет | доказательства из чата встречи, ссылки/файлы |

**Шаги обработки (в порядке выполнения):**

1. **`transcribe`** – транскрипция MP3 → `transcript.json`, `transcript.md`
2. **`diarize`** – диаризация спикеров → `speaker-timeline.json`
3. **`identify_speakers`** – слияние идентичностей → `speaker-identities.json`
4. **`topics`** – построение временной шкалы тем → `topics.json`
5. **`decisions`** – обнаружение решений → `decisions.json`
6. **`actions`** – обнаружение задач (action items) → `tasks.json`
7. **`screen_intelligence`** – анализ скриншотов и OCR → `ocr/`, `visual-evidence.json`
8. **`knowledge`** – извлечение знаний встречи → `knowledge.json`, `summary.md`
9. **`radar`** – проекция важных находок на Radar → `radar-signals.json`

Каждый шаг содержит политику достоверности (например, `candidate_not_domain_truth`, `review_required_before_promotion`), указывающую, что результат является кандидатом, а не истиной.

---

## Consistency (поиск противоречий)

**Путь:** `backend/src/engines/consistency/`

Обнаруживает противоречия между уже принятыми фактами о персонах (`AcceptedClaim`) и новыми заявлениями, извлечёнными из доказательств (`NewEvidenceClaim`).

### Модели

- **`AcceptedClaim`** – принятое утверждение:
  - `subject_id`, `claim_type`, `value`, `source_kind` (enum), `source_id`, `confidence`.
- **`NewEvidenceClaim`** – новое утверждение из доказательств (аналогичные поля).
- **`EvidenceClaimExtractionInput`** – входные данные для извлечения утверждений:
  - `subject_id`, `source_kind`, `source_id`, `text`, `confidence`.
- **`NewContradictionObservation`** – найденное противоречие:
  - `old_source_kind/id`, `new_source_kind/id`, `affected_entities`, `conflict_type`, `old_claim`, `new_claim`, `confidence`, `severity`, `review_state`, `metadata`.
- **`ContradictionObservation`** – сохранённое противоречие (добавлены `observation_id`, `reviewed_by`, `reviewed_at`, `resolution`, `created_at`, `updated_at`).
- Перечисления:
  - **`ContradictionSourceKind`**: `Communication`, `Document`, `Event`, `Memory`, `Knowledge`, `Decision`, `Obligation`, `Task`, `Relationship`.
  - **`ContradictionSeverity`**: `Low`, `Medium`, `High`, `Critical`.
  - **`ContradictionReviewState`**: `Suggested`, `UserConfirmed`, `UserRejected`.

### Логика обнаружения (`ConsistencyEngine`)

- Извлечение утверждений:
  - `extract_evidence_claims` – построчно разбирает текст: структурированные строки (`claim_type: value` или `claim_type=value`) или естественно‑языковые паттерны для `location` и `status`.
  - Поддерживаемые детерминированные типы: `location`, `status`.
  - Имя типа нормализуется (нижний регистр, пробелы заменяются на `_`).
- Поиск противоречий:
  - Сравнение всех пар `(AcceptedClaim, NewEvidenceClaim)`.
  - Совпадение должны: одинаковый `subject_id` и `claim_type`.
  - Значения нормализуются (разбивка по пробелам, склеивание, нижний регистр). Если они различаются – создаётся противоречие.
  - `confidence` наблюдения = минимум из двух confidence.
  - `severity` по уровням: `≥0.95` → Critical, `≥0.9` → High, `≥0.7` → Medium, иначе Low.
  - `review_state` всегда `Suggested`.

### Хранилище (`ContradictionObservationStore`)

- `upsert` – идемпотентное сохранение наблюдения (детерминированный `observation_id` на основе полей).
- `list_open` – список открытых (review_state = `suggested`) с лимитом 1–100.
- `refresh_deterministic_observations` – автоматический цикл обновления.
- `set_review_state` / `set_review_state_with_observation` – пользовательская проверка (подтверждение/отклонение).

### Процесс обновления (`refresh_deterministic_observations`)

1. Собирает источники:
   - Активные факты о персонах (`person_facts` с `is_active = true` и непустым email) – источник `Memory`.
   - Недавние email‑сообщения (`communication_messages`) – `Communication`.
   - Канальные сообщения (Telegram, WhatsApp) с привязкой через `person_identities` – `Communication`.
   - Документы (`documents` с непустым `extracted_text`) – `Document`.
   - Заметки встреч (`meeting_notes` через `event_participants`) – `Event`.
   - Транскрипты звонков (`call_transcripts` со статусом `succeeded`, привязка через Telegram‑identity) – `Communication`.
2. Для каждого факта и каждого доказательства, где совпадает email/персона, извлекает утверждения и детектирует противоречия.
3. Сохраняет найденные противоречия через `upsert`.

Константы:
- `MAX_REFRESH_LIMIT = 100`
- `MIN_REFRESH_LIMIT = 1`
- `STRUCTURED_EVIDENCE_CLAIM_CONFIDENCE = 0.8`

---

## Context Packs (контекстные пакеты)

**Путь:** `backend/src/engines/context_packs/`

Управляет агрегированными контекстными данными для конкретных субъектов (персона, встреча, задача и т.д.) и их источниками.

### Модели

- **`ContextPack`** – пакет:
  - `context_pack_id`, `kind`, `subject_id`, `content` (JSON), `metadata` (JSON), `rebuildable`, `built_at`, `updated_at`.
- **`NewContextPack`** – конструктор нового пакета (builder).
- **`ContextPackSource`** – источник данных для пакета:
  - `context_pack_id`, `source_kind`, `source_id`, `role`, `metadata` (JSON), `created_at`.
- **`NewContextPackSource`** – конструктор источника.
- Перечисления:
  - **`ContextPackKind`**: `Persona`, `Meeting`, `Task`, `Calendar`, `Project`.
  - **`ContextPackSourceKind`**: `Observation`, `DomainEntity`, `Knowledge`, `Relationship`, `Decision`, `Task`, `Obligation`, `Document`, `CalendarEvent`, `Project`.

### Хранилище (`ContextPackStore`)

- `get(kind, subject_id)` – получение одного пакета.
- `exists(kind, subject_id)` – проверка существования.
- `upsert_with_sources(pack, sources)` – транзакционное создание/обновление пакета и замена источников.
- `list_sources(context_pack_id)` – список источников пакета.

Особенности:
- `context_pack_id` = `format!("context_pack:v1:{:x}", SHA256(kind.as_str() + "\n" + subject_id.trim()))`
- Конфликт при upsert разрешается по уникальной паре `(kind, subject_id)`.
- При upsert старые источники удаляются, новые вставляются.
- Обязательная валидация: `subject_id` не пуст, `content` и `metadata` – JSON‑объекты, список источников не пуст.

---

## Enrichment (обогащение)

**Путь:** `backend/src/engines/enrichment/`

Предоставляет методы для создания предложений по обогащению данных персон.

### Основные методы (`EnrichmentEngine`)

- **`persona_favorite_preference(person_id, is_favorite)`**
  - Если `is_favorite == true`, возвращает `PreferenceDraft`:
    - `preference_type = "ui:favorite"`, `value = "true"`, `confidence = 1.0`.
- **`persona_observation_candidate(person_id, source, extracted_claim, data, confidence)`**
  - Создаёт `EnrichmentCandidateDraft` с полями:
    - `entity_kind = "persona"`, `entity_id = person_id`.
    - `source`, `extracted_claim`, `data` (JSON‑объект), `confidence`, `review_state = "pending"`, `freshness = "current"`.
  - В `data` добавляется объект `_enrichment` с метаинформацией (`affected_entity_kind/id`, `extracted_claim`, `source`, `review_state`, `freshness`, `conflict_marker`).
  - `conflict_marker` извлекается из поля `conflict_marker` или `conflict` в `data` (булево, по умолчанию `false`).
  - Валидирует, что строки не пусты и `confidence ∈ [0.0, 1.0]`.
```

## Покрытие источников

| Файл | Использованные факты |
|---|---|
| `backend/src/engines/automation/store.rs` | Структура `AutomationStore`, методы `upsert_template`, `upsert_policy`, `list_templates`, `list_policies`, `dry_run_send`, `policy_with_template`, SQL‑запросы, фиксация наблюдений (`capture_*_observation`) |
| `backend/src/engines/automation/validation.rs` | Валидация `NewAutomationTemplate`, `NewAutomationPolicy`, `TelegramSendDryRunRequest`: обязательные поля, `max_sends_per_hour > 0`, `allowed_chat_ids` не пуст, `quiet_hours`/`conditions` – JSON‑объекты, имена переменных – ASCII альфанумерик и `_` |
| `backend/src/engines/call_intelligence/engine.rs` | `CallIntelligenceEngine::plan_from_bundle`, требования к артефактам (audio.mp3 обязателен, speaker‑hints/screenshots/chat опциональны), 9 шагов пайплайна с входными/выходными артефактами и политиками достоверности |
| `backend/src/engines/call_intelligence/mod.rs` | Публичный экспорт `CallIntelligenceEngine`, `CallIntelligenceArtifactRequirement`, `CallIntelligenceOutputCandidate`, `CallIntelligencePipelinePlan`, `CallIntelligenceStep` |
| `backend/src/engines/call_intelligence/models.rs` | Структуры `CallIntelligenceArtifactRequirement`, `CallIntelligenceStep`, `CallIntelligencePipelinePlan`, `CallIntelligenceOutputCandidate` с полями |
| `backend/src/engines/consistency.rs` | Реэкспорт `ConsistencyEngine`, `ConsistencyError`, `contradiction_observation_id`, моделей, `ContradictionObservationStore` |
| `backend/src/engines/consistency/constants.rs` | Константы `MAX_REFRESH_LIMIT`, `MIN_REFRESH_LIMIT`, `STRUCTURED_EVIDENCE_CLAIM_CONFIDENCE` |
| `backend/src/engines/consistency/engine.rs` | Методы `extract_evidence_claims`, `detect_claim_contradictions`, `detect_evidence_contradictions`, логика сравнения: совпадение `subject_id`/`claim_type`, нормализация значений, вычисление confidence и severity, review_state=Suggested |
| `backend/src/engines/consistency/errors.rs` | Варианты `ConsistencyError`: `EmptyField`, `InvalidConfidence`, `UnknownSourceKind`, `UnknownSeverity`, `UnknownReviewState`, `ObservationNotFound`, `Sqlx`, `ObservationStore`, `InvalidJsonObject`, `InvalidJsonArrayOrObject` |
| `backend/src/engines/consistency/evidence.rs` | Структуры доказательств (`ActivePersonFactClaim`, `MessageEvidence`, `ChannelMessageEvidence`, `DocumentEvidence`, `MeetingNoteEvidence`, `CallTranscriptEvidence`), их row‑конвертеры, `link_consistency_entity_in_transaction` |
| `backend/src/engines/consistency/helpers.rs` | `contradiction_observation_id` (детерминированный формат), `claim_text`, `normalize_claim_value`, `severity_for_confidence` (пороги), `contradiction_metadata` |
| `backend/src/engines/consistency/models.rs` | Перечисления `ContradictionSourceKind`, `ContradictionSeverity`, `ContradictionReviewState` (со строковыми представлениями и `parse`), структуры `AcceptedClaim`, `NewEvidenceClaim`, `EvidenceClaimExtractionInput`, `NewContradictionObservation`, `ContradictionObservation` |
| `backend/src/engines/consistency/parsing.rs` | Разбор строк утверждений: структурированные (`:`/`=`, только `location`/`status`) и естественно‑языковые шаблоны |
| `backend/src/engines/consistency/rows.rs` | Конвертация строк БД в `ContradictionObservation`, парсинг строковых перечислений |
| `backend/src/engines/consistency/store.rs` | Публичный API `ContradictionObservationStore`: `refresh_deterministic_observations`, `upsert`, `list_open`, `set_review_state`, `set_review_state_with_observation` |
| `backend/src/engines/consistency/store/observations.rs` | SQL upsert с детерминированным ID, `list_open` с фильтром review_state='suggested' и clamped limit, `link_contradiction_observation_in_transaction`, захват evidence‑observation |
| `backend/src/engines/consistency/store/refresh.rs` | Цикл обновления: сбор фактов и доказательств (person_facts, messages, channel_messages, documents, meeting_notes, call_transcripts), формирование `AcceptedClaim`/`EvidenceClaimExtractionInput`, вызов `detect_evidence_contradictions` и `upsert` |
| `backend/src/engines/consistency/store/review.rs` | SQL обновления review_state, опциональное связывание review‑observation |
| `backend/src/engines/consistency/store/sources.rs` | SQL‑запросы для источников: `active_person_fact_claims` (person_facts + persons с email), `recent_message_evidence`, `recent_channel_message_evidence` (через identities для telegram/whatsapp), `recent_document_evidence`, `recent_meeting_note_evidence`, `recent_call_transcript_evidence` |
| `backend/src/engines/consistency/validation.rs` | `validate_non_empty`, `validate_confidence`, `validate_json_object`, `validate_json_array_or_object`, `validate_refresh_limit` (clamp 1..100) |
| `backend/src/engines/context_packs/errors.rs` | `ContextPackStoreError` с вариантами `EmptyField`, `MissingSources`, `UnknownContextPackKind`, `UnknownContextPackSourceKind`, `InvalidJsonObject`, `Sqlx`, `Json` |
| `backend/src/engines/context_packs/mod.rs` | Публичный экспорт `ContextPackStore`, `ContextPack`, `ContextPackKind`, `ContextPackSource`, `ContextPackSourceKind`, `NewContextPack`, `NewContextPackSource` |
| `backend/src/engines/context_packs/models.rs` | Перечисления `ContextPackKind`, `ContextPackSourceKind`, структуры `ContextPack`, `NewContextPack`, `ContextPackSource`, `NewContextPackSource`, валидация (`validate_context_pack_with_sources`, `validate_non_empty`, `validate_json_object`) |
| `backend/src/engines/context_packs/store.rs` | `ContextPackStore` с методами `get`, `exists`, `upsert_with_sources` (транзакционное upsert + замена источников), `list_sources`, формирование `context_pack_id` через SHA‑256 |
| `backend/src/engines/enrichment/engine.rs` | `EnrichmentEngine` с методами `persona_favorite_preference` и `persona_observation_candidate`, структура возврата с `_enrichment`, валидация |

## Исходные файлы

- [`backend/src/engines/automation/store.rs`](../../../../backend/src/engines/automation/store.rs)
- [`backend/src/engines/automation/validation.rs`](../../../../backend/src/engines/automation/validation.rs)
- [`backend/src/engines/call_intelligence/engine.rs`](../../../../backend/src/engines/call_intelligence/engine.rs)
- [`backend/src/engines/call_intelligence/mod.rs`](../../../../backend/src/engines/call_intelligence/mod.rs)
- [`backend/src/engines/call_intelligence/models.rs`](../../../../backend/src/engines/call_intelligence/models.rs)
- [`backend/src/engines/consistency.rs`](../../../../backend/src/engines/consistency.rs)
- [`backend/src/engines/consistency/constants.rs`](../../../../backend/src/engines/consistency/constants.rs)
- [`backend/src/engines/consistency/engine.rs`](../../../../backend/src/engines/consistency/engine.rs)
- [`backend/src/engines/consistency/errors.rs`](../../../../backend/src/engines/consistency/errors.rs)
- [`backend/src/engines/consistency/evidence.rs`](../../../../backend/src/engines/consistency/evidence.rs)
- [`backend/src/engines/consistency/helpers.rs`](../../../../backend/src/engines/consistency/helpers.rs)
- [`backend/src/engines/consistency/models.rs`](../../../../backend/src/engines/consistency/models.rs)
- [`backend/src/engines/consistency/parsing.rs`](../../../../backend/src/engines/consistency/parsing.rs)
- [`backend/src/engines/consistency/rows.rs`](../../../../backend/src/engines/consistency/rows.rs)
- [`backend/src/engines/consistency/store.rs`](../../../../backend/src/engines/consistency/store.rs)
- [`backend/src/engines/consistency/store/observations.rs`](../../../../backend/src/engines/consistency/store/observations.rs)
- [`backend/src/engines/consistency/store/refresh.rs`](../../../../backend/src/engines/consistency/store/refresh.rs)
- [`backend/src/engines/consistency/store/review.rs`](../../../../backend/src/engines/consistency/store/review.rs)
- [`backend/src/engines/consistency/store/sources.rs`](../../../../backend/src/engines/consistency/store/sources.rs)
- [`backend/src/engines/consistency/validation.rs`](../../../../backend/src/engines/consistency/validation.rs)
- [`backend/src/engines/context_packs/errors.rs`](../../../../backend/src/engines/context_packs/errors.rs)
- [`backend/src/engines/context_packs/mod.rs`](../../../../backend/src/engines/context_packs/mod.rs)
- [`backend/src/engines/context_packs/models.rs`](../../../../backend/src/engines/context_packs/models.rs)
- [`backend/src/engines/context_packs/store.rs`](../../../../backend/src/engines/context_packs/store.rs)
- [`backend/src/engines/enrichment/engine.rs`](../../../../backend/src/engines/enrichment/engine.rs)

## Кандидаты на drift

На основе предоставленного контекста (только исходные файлы бэкенда) видимых расхождений между кодом и документацией или ADR не обнаружено, поскольку существующая wiki-документация для этих движков не была включена в контекст. Внутри самих исходных файлов противоречий не выявлено.
