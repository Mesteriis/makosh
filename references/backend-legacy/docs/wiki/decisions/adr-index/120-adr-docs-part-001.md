---
chunk_id: 120-adr-docs-part-001
batch_id: batch-20260628T214902
group: docs
role: adr
source_status: pending
source_count: 25
generated_by: code-wiki-ru
---

# 120-adr-docs-part-001 — docs/adr

- Target index: [[decisions/adr-index]]
- Batch: `batch-20260628T214902`
- Source files: `25`

## Резюме

Создать страницу-индекс `decisions/adr-index.md` в русской Obsidian-wiki. Страница перечисляет все архитектурные решения (ADR‑0001 … ADR‑0025) с сохранением оригинальных английских названий и добавлением краткого русского описания, непосредственно опирающегося на текст каждого вложенного ADR. Индекс позволяет быстро ориентироваться в принятых/предложенных архитектурных решениях.

## Предложенные страницы

- `decisions/adr-index.md`

```markdown
# Индекс архитектурных решений (ADR)

В этом разделе собраны архитектурные решения проекта Макошь.
Все решения на данный момент имеют статус **Proposed**.

- ``ADR-0001 Event Sourcing as System Spine` (`adr-0001-event-sourcing-as-system-spine`)`
  Значимые изменения фиксируются как канонические события, на основе которых строятся проекции, графовые связи, индексы и временные шкалы.

- ``ADR-0002 Rust Backend` (`adr-0002-rust-backend`)`
  Бэкенд реализуется на Rust для координации ингеста, индексации, локального хранения, адаптеров провайдеров, инструментов агентов и десктопной интеграции.

- ``ADR-0003 SvelteKit Frontend` (`adr-0003-sveltekit-frontend`)`
  Фронтенд на SvelteKit обеспечивает плотные десктопные рабочие процессы, реактивное состояние, палитру команд и будущую веб-переносимость.

- ``ADR-0004 Tauri Desktop Shell` (`adr-0004-tauri-desktop-shell`)`
  Tauri используется как десктопная оболочка для локальной интеграции с файлами, сервисами, хранилищем секретов, уведомлениями и возможными мостами к провайдерам.

- ``ADR-0005 PostgreSQL Primary Store` (`adr-0005-postgresql-primary-store`)`
  PostgreSQL — основное локальное хранилище, объединяющее события, сущности, графоподобные связи, JSONB-полезную нагрузку, миграции и смещения проекций.

- ``ADR-0006 Tantivy Full Text Search` (`adr-0006-tantivy-full-text-search`)`
  Tantivy обеспечивает локальный полнотекстовый поиск по сообщениям, документам, задачам, контактам и проектам без облачных зависимостей.

- ``ADR-0007 Replaceable Vector Search` (`adr-0007-replaceable-vector-search`)`
  Векторный поиск определён за сменяемым интерфейсом; эмбеддинги и индексы рассматриваются как производные артефакты, допускающие перестроение.

- ``ADR-0008 Knowledge Graph First` (`adr-0008-knowledge-graph-first`)`
  Граф знаний — первоклассный архитектурный компонент, хранящий долгосрочные связи между людьми, организациями, проектами, документами, сообщениями, задачами и решениями с указанием происхождения и уверенности.

- ``ADR-0009 Local AI Through Ollama` (`adr-0009-local-ai-through-ollama`)`
  Ollama используется как начальная граница локального ИИ, чтобы инференс работал по умолчанию без обязательных обращений к облачным моделям.

- ``ADR-0010 Specialized Agent System` (`adr-0010-specialized-agent-system`)`
  Вместо одного универсального ассистента вводятся специализированные агенты: HESTIA, MAKOSH, MNEMOSYNE, ATHENA и HEPHAESTUS, с разделёнными обязанностями и разрешениями.

- ``ADR-0011 Plugin Architecture` (`adr-0011-plugin-architecture`)`
  Архитектура плагинов с манифестами, явными возможностями и ограниченным доступом во время выполнения позволяет развивать интеграции вне ядра.

- ``ADR-0012 OpenTelemetry Observability` (`adr-0012-opentelemetry-observability`)`
  OpenTelemetry обеспечивает трассы, метрики и структурированную наблюдаемость, исключая утечку тел сообщений, секретов и приватного содержимого документов.

- ``ADR-0013 Local First Data Ownership` (`adr-0013-local-first-data-ownership`)`
  Локальное хранение и работа — режим по умолчанию; облачные сервисы являются опциональными интеграциями, а не обязательной инфраструктурой.

- ``ADR-0014 Canonical Event Envelope` (`adr-0014-canonical-event-envelope`)`
  Определена каноническая оболочка события с полями: event ID, тип, версия схемы, таймстемпы, источник, актор, субъект, полезная нагрузка, происхождение, causation и correlation ID.

- ``ADR-0015 Command Query Separation` (`adr-0015-command-query-separation`)`
  Команды (изменяющие состояние) и запросы (читающие модели) разделены на границе приложения; это упрощает валидацию, тестирование и ограничение прав агентов.

- ``ADR-0016 Secrets and Encryption Boundary` (`adr-0016-secrets-and-encryption-boundary`)`
  Секреты (токены провайдеров, пароли, приватные ключи) хранятся вне обычных таблиц приложения и доступны через OS‑backed secret store или зашифрованное хранилище.

- ``ADR-0017 Document Processing Pipeline` (`adr-0017-document-processing-pipeline`)`
  Асинхронный конвейер обработки документов, запускаемый событиями, отделяет приём файла от дорогостоящих шагов (OCR, извлечение, индексация) и позволяет повторять сбойные операции.

- ``ADR-0018 Provider Adapter Boundary` (`adr-0018-provider-adapter-boundary`)`
  Адаптеры провайдеров сохраняют сырые исходные записи и порождают нормализованные команды или события, изолируя особенности внешних API.

- ``ADR-0019 Contact Identity Resolution` (`adr-0019-contact-identity-resolution`)`
  Идентификация контактов моделируется как набор кандидатов с оценкой уверенности, с явными операциями слияния и разделения; автоматическое объединение для неоднозначных случаев запрещено.

- ``ADR-0020 Task Candidate Lifecycle` (`adr-0020-task-candidate-lifecycle`)`
  ИИ извлекает задачи-кандидаты из сообщений и документов, но активация задачи требует подтверждения пользователя или узко определённой политики.

- ``ADR-0021 Calendar as Event Source` (`adr-0021-calendar-as-event-source`)`
  Календари рассматриваются как источники событий (встречи, изменения расписания, участие), а не просто UI-виджеты; события встреч обогащают граф и поиск.

- ``ADR-0022 No Fine Tuning on Private Data` (`adr-0022-no-fine-tuning-on-private-data`)`
  Запрещён файн-тюнинг моделей на приватных данных; вместо этого используются граф, RAG, векторный поиск и структурированная память с возможностью ссылаться на источники.

- ``ADR-0023 Rebuildable Projections` (`adr-0023-rebuildable-projections`)`
  Проекции и индексы рассматриваются как перестраиваемые из канонических событий, сырых записей и артефактов документов; фиксируются версии источников для перестроения.

- ``ADR-0024 Idempotent Imports` (`adr-0024-idempotent-imports`)`
  Весь импорт идемпотентен: используются идентификаторы провайдера, отпечатки контента, идентификаторы партий импорта и специфичные для источника чекпоинты.

- ``ADR-0025 Keyboard First Command Palette` (`adr-0025-keyboard-first-command-palette`)`
  Навигация с клавиатуры и палитра команд — основной UI-паттерн для обеспечения скорости повседневных рабочих процессов технического пользователя.
```

## Покрытие источников

| Исходный файл | Использованные факты |
|---|---|
| `docs/adr/ADR-0001-event-sourcing-as-system-spine.md` | Название, статус, суть решения (канонические события для проекций, графов, индексов, таймлайнов). |
| `docs/adr/ADR-0002-rust-backend.md` | Название, статус, решение о Rust для бэкенда, интеграция с Tauri и Tantivy. |
| `docs/adr/ADR-0003-sveltekit-frontend.md` | Название, статус, решение о SvelteKit, поддержка десктопных рабочих процессов, командной палитры. |
| `docs/adr/ADR-0004-tauri-desktop-shell.md` | Название, статус, использование Tauri для десктопной интеграции, общий технологический стек с Rust. |
| `docs/adr/ADR-0005-postgresql-primary-store.md` | Название, статус, PostgreSQL как основное хранилище, поддержка JSONB, событий, графовых связей. |
| `docs/adr/ADR-0006-tantivy-full-text-search.md` | Название, статус, Tantivy для локального полнотекстового поиска, тесная интеграция с Rust. |
| `docs/adr/ADR-0007-replaceable-vector-search.md` | Название, статус, сменяемый интерфейс векторного поиска, эмбеддинги/индексы как производные артефакты. |
| `docs/adr/ADR-0008-knowledge-graph-first.md` | Название, статус, граф знаний как первоклассный компонент, связи с provenance и confidence. |
| `docs/adr/ADR-0009-local-ai-through-ollama.md` | Название, статус, Ollama как локальная граница ИИ, опциональность облачных моделей. |
| `docs/adr/ADR-0010-specialized-agent-system.md` | Название, статус, специализированные агенты вместо одного ассистента, роли и разрешения. |
| `docs/adr/ADR-0011-plugin-architecture.md` | Название, статус, архитектура плагинов с манифестами и ограниченным доступом. |
| `docs/adr/ADR-0012-opentelemetry-observability.md` | Название, статус, OpenTelemetry для наблюдаемости, исключение приватного контента. |
| `docs/adr/ADR-0013-local-first-data-ownership.md` | Название, статус, локальное хранение по умолчанию, облачные сервисы как опция. |
| `docs/adr/ADR-0014-canonical-event-envelope.md` | Название, статус, поля канонического конверта события (ID, тип, версия, таймстемпы, источник, актор, субъект, payload, provenance, causation, correlation). |
| `docs/adr/ADR-0015-command-query-separation.md` | Название, статус, разделение команд и запросов на границе приложения. |
| `docs/adr/ADR-0016-secrets-and-encryption-boundary.md` | Название, статус, секреты вне таблиц приложения, OS-backed secret store или encrypted vault. |
| `docs/adr/ADR-0017-document-processing-pipeline.md` | Название, статус, асинхронный конвейер, повторные попытки, видимые состояния. |
| `docs/adr/ADR-0018-provider-adapter-boundary.md` | Название, статус, адаптеры провайдеров, сохранение сырых записей, нормализованные команды/события. |
| `docs/adr/ADR-0019-contact-identity-resolution.md` | Название, статус, разрешение идентичности с confidence, запрет автоматического слияния в неоднозначных случаях. |
| `docs/adr/ADR-0020-task-candidate-lifecycle.md` | Название, статус, задачи-кандидаты от ИИ, обязательное подтверждение пользователем. |
| `docs/adr/ADR-0021-calendar-as-event-source.md` | Название, статус, календари как источники событий, обогащение графа и поиска. |
| `docs/adr/ADR-0022-no-fine-tuning-on-private-data.md` | Название, статус, отказ от файн-тюнинга, использование графа, RAG, векторного поиска. |
| `docs/adr/ADR-0023-rebuildable-projections.md` | Название, статус, проекции и индексы перестраиваемы из канонических событий и сырых записей. |
| `docs/adr/ADR-0024-idempotent-imports.md` | Название, статус, идемпотентный импорт с использованием provider ID, fingerprints, batch ID, чекпоинтов. |
| `docs/adr/ADR-0025-keyboard-first-command-palette.md` | Название, статус, клавиатурная навигация и командная палитра как первичный UI-паттерн. |

## Исходные файлы

- [`docs/adr/ADR-0001-event-sourcing-as-system-spine.md`](../../../adr/ADR-0001-event-sourcing-as-system-spine.md)
- [`docs/adr/ADR-0002-rust-backend.md`](../../../adr/ADR-0002-rust-backend.md)
- [`docs/adr/ADR-0003-sveltekit-frontend.md`](../../../adr/ADR-0003-sveltekit-frontend.md)
- [`docs/adr/ADR-0004-tauri-desktop-shell.md`](../../../adr/ADR-0004-tauri-desktop-shell.md)
- [`docs/adr/ADR-0005-postgresql-primary-store.md`](../../../adr/ADR-0005-postgresql-primary-store.md)
- [`docs/adr/ADR-0006-tantivy-full-text-search.md`](../../../adr/ADR-0006-tantivy-full-text-search.md)
- [`docs/adr/ADR-0007-replaceable-vector-search.md`](../../../adr/ADR-0007-replaceable-vector-search.md)
- [`docs/adr/ADR-0008-knowledge-graph-first.md`](../../../adr/ADR-0008-knowledge-graph-first.md)
- [`docs/adr/ADR-0009-local-ai-through-ollama.md`](../../../adr/ADR-0009-local-ai-through-ollama.md)
- [`docs/adr/ADR-0010-specialized-agent-system.md`](../../../adr/ADR-0010-specialized-agent-system.md)
- [`docs/adr/ADR-0011-plugin-architecture.md`](../../../adr/ADR-0011-plugin-architecture.md)
- [`docs/adr/ADR-0012-opentelemetry-observability.md`](../../../adr/ADR-0012-opentelemetry-observability.md)
- [`docs/adr/ADR-0013-local-first-data-ownership.md`](../../../adr/ADR-0013-local-first-data-ownership.md)
- [`docs/adr/ADR-0014-canonical-event-envelope.md`](../../../adr/ADR-0014-canonical-event-envelope.md)
- [`docs/adr/ADR-0015-command-query-separation.md`](../../../adr/ADR-0015-command-query-separation.md)
- [`docs/adr/ADR-0016-secrets-and-encryption-boundary.md`](../../../adr/ADR-0016-secrets-and-encryption-boundary.md)
- [`docs/adr/ADR-0017-document-processing-pipeline.md`](../../../adr/ADR-0017-document-processing-pipeline.md)
- [`docs/adr/ADR-0018-provider-adapter-boundary.md`](../../../adr/ADR-0018-provider-adapter-boundary.md)
- [`docs/adr/ADR-0019-contact-identity-resolution.md`](../../../adr/ADR-0019-contact-identity-resolution.md)
- [`docs/adr/ADR-0020-task-candidate-lifecycle.md`](../../../adr/ADR-0020-task-candidate-lifecycle.md)
- [`docs/adr/ADR-0021-calendar-as-event-source.md`](../../../adr/ADR-0021-calendar-as-event-source.md)
- [`docs/adr/ADR-0022-no-fine-tuning-on-private-data.md`](../../../adr/ADR-0022-no-fine-tuning-on-private-data.md)
- [`docs/adr/ADR-0023-rebuildable-projections.md`](../../../adr/ADR-0023-rebuildable-projections.md)
- [`docs/adr/ADR-0024-idempotent-imports.md`](../../../adr/ADR-0024-idempotent-imports.md)
- [`docs/adr/ADR-0025-keyboard-first-command-palette.md`](../../../adr/ADR-0025-keyboard-first-command-palette.md)

## Кандидаты на drift

Из предоставленного контекста расхождений между кодом, документацией и ADR не выявлено. Все ADR имеют статус **Proposed**, что корректно отражено в индексной странице. Отсутствующие или противоречащие ссылки не обнаружены.
