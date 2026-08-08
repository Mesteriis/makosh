---
chunk_id: 123-adr-docs-part-004
batch_id: batch-20260628T214902
group: docs
role: adr
source_status: pending
source_count: 23
generated_by: code-wiki-ru
---

# 123-adr-docs-part-004 — docs/adr

- Target index: [[decisions/adr-index]]
- Batch: `batch-20260628T214902`
- Source files: `23`

## Резюме

Необходимо создать (или дополнить) страницу индекса архитектурных решений (ADR) в русской Obsidian‑вики по пути `decisions/adr-index.md`. В данном чанке доступны ADR‑0077 — ADR‑0099; для каждого из них в индекс добавляется запись со статусом, датой, краткой характеристикой и ссылкой на соответствующий файл ADR. Это обеспечит навигацию по обновлённому корпусу документов.

## Предложенные страницы

#### `decisions/adr-index.md`

```markdown
# Индекс ADR

Здесь собраны архитектурные решения (ADR) проекта Макошь, доступные в текущем контексте (ADR-0077 – ADR-0099).
Для каждого решения указан идентификатор, статус, дата принятия и краткое описание на основе исходного текста ADR.

| ADR | Статус | Дата | Что решено |
|-----|--------|------|------------|
| ``ADR-0077` (`ADR-0077-i18n-russian-english\`)` | Accepted | 2026-06-08 | Добавлен i18n для русского и английского интерфейса через JSON‑словари и Svelte‑стор. |
| ``ADR-0078` (`ADR-0078-frontend-component-decomposition\`)` | Superseded by ADR-0093 | 2026-06-09 | Декомпозиция SPA‑страниц и виджетов в SvelteKit. |
| ``ADR-0079` (`ADR-0079-script-logic-decomposition\`)` | Superseded by ADR-0093 | 2026-06-09 | Вынос сервисов, «умных» страниц и конфигурации из `+page.svelte` в SvelteKit. |
| ``ADR-0080` (`ADR-0080-mail-background-sync-progress-local-trash\`)` | Accepted | 2026-06-10 | Фоновый синхронизатор почты, прогресс‑индикаторы и локальная корзина (без удаления писем у провайдера). |
| ``ADR-0081` (`ADR-0081-opt-in-omniroute-ai-runtime\`)` | Proposed | — | Добавлен опциональный AI‑провайдер OmniRoute с явным согласием пользователя; Ollama остаётся умолчанием. |
| ``ADR-0082` (`ADR-0082-ai-settings-control-center\`)` | Proposed | — | Центр управления AI‑настройками с поддержкой разных типов провайдеров (built‑in, CLI, API), роутингом моделей и версионированными шаблонами промптов. |
| ``ADR-0083` (`ADR-0083-telegram-live-user-client-runtime\`)` | Proposed | — | Первый срез живого Telegram‑клиента через TDLib‑рантайм для синхронизации чатов, истории, отправки сообщений и медиа. |
| ``ADR-0084` (`ADR-0084-persona-intelligence-system\`)` | Proposed | — | Переход от контактной модели к представлению «Persona» как цифрового образа субъекта, включая отношения, память, таймлайн и досье. |
| ``ADR-0085` (`ADR-0085-communication-spine-and-contradiction-engine\`)` | Proposed | — | Коммуникации объявлены основным каналом поступления доказательств; вводится Consistency / Contradiction Engine (Polygraph). |
| ``ADR-0086` (`ADR-0086-first-class-relationship-persistence\`)` | Proposed | — | Введено долговременное хранение отношений (`relationships`) с источниками доказательств, отдельно от графовых проекций. |
| ``ADR-0087` (`ADR-0087-contradiction-observation-persistence\`)` | Proposed | — | Реализовано сохранение и рецензирование наблюдений о противоречиях (Contradiction Observation). |
| ``ADR-0088` (`ADR-0088-obligation-persistence\`)` | Proposed | — | Обязательства (Obligations) выделены в отдельную сущность с доказательствами, без автоматического создания задач. |
| ``ADR-0089` (`ADR-0089-decision-persistence\`)` | Proposed | — | Решения (Decisions) стали durable‑записями с обоснованием, альтернативами и влиянием на другие домены. |
| ``ADR-0090` (`ADR-0090-persona-native-compatibility-api-bridge\`)` | Proposed | — | Создан API‑мост `/api/v1/personas/*` для работы в терминах целевой модели Persona, сохраняя совместимость с `persons`. |
| ``ADR-0091` (`ADR-0091-telegram-production-client-capability-model\`)` | Proposed | — | Модель возможностей Telegram‑клиента: каждое действие классифицировано (available / blocked / degraded / planned / unsupported) с учётом политик безопасности. |
| ``ADR-0092` (`ADR-0092-mail-provider-capability-tiers\`)` | Proposed | 2026-06-13 | Почтовые провайдеры разбиты на уровни возможностей (Native API, IMAP/SMTP, POP3, Exchange legacy, Proton Bridge). |
| ``ADR-0093` (`ADR-0093-frontend-platform-migration-to-vue-3\`)` | Accepted | 2026-06-14 | Миграция фронтенда с SvelteKit на Vue 3 + TypeScript + Vite + Tauri 2, с Domain‑Driven Design и TanStack Query. |
| ``ADR-0094` (`ADR-0094-telegram-base-domain-completion-boundary\`)` | Superseded by ADR-0097 | 2026-06-18 | Определял границы завершения базового домена Telegram; заменён ADR‑0097. |
| ``ADR-0095` (`ADR-0095-event-driven-domain-communication-and-dlq\`)` | Accepted | — | Всё междоменное взаимодействие переведено на события с Dead Letter Queue и идемпотентностью потребителей. |
| ``ADR-0096` (`ADR-0096-canonical-evidence-review-and-context-packs\`)` | Accepted | — | Введены каноническое хранилище свидетельств (Observation Platform), единый инбокс рецензирования (Review) и контекстные пакеты (Context Packs). |
| ``ADR-0097` (`ADR-0097-communications-channel-domains-to-integrations\`)` | Accepted | 2026-06-20 | Почта, Telegram, WhatsApp переопределены как интеграции, а не домены; единый домен Communications. |
| ``ADR-0098` (`ADR-0098-provider-neutral-communications-api-and-strict-boundaries\`)` | Accepted | 2026-06-21 | API для Communications стал provider‑neutral; введены строгие архитектурные границы между слоями. |
| ``ADR-0099` (`ADR-0099-signal-hub-event-platform\`)` | Accepted | 2026-06-22 | Signal Hub — центральный домен управления источниками сигналов и событийной платформой (PostgreSQL + NATS JetStream + SSE + Protobuf). |

> **Примечание:** В данный чанк попали только ADR‑0077 – ADR‑0099. Более ранние ADR (0001‑0076) находятся в других частях и не отражены в этом индексе.
```

## Покрытие источников

Каждый из перечисленных ADR‑файлов представлен в индексе следующими фактами, извлечёнными непосредственно из его текста:

- **ADR‑0077**: заголовок, статус (`Accepted`), дата (`2026-06-08`), решение об i18n с JSON‑словарями и Svelte‑стором.
- **ADR‑0078**: заголовок, статус (`Superseded by ADR‑0093`), дата, факт декомпозиции компонентов Svelte.
- **ADR‑0079**: заголовок, статус (`Superseded by ADR‑0093`), дата, факт выноса сервисов и конфигурации.
- **ADR‑0080**: заголовок, статус, дата, решение о фоновой синхронизации почты и локальной корзине.
- **ADR‑0081**: заголовок, статус (`Proposed`), решение об опциональном OmniRoute‑провайдере.
- **ADR‑0082**: заголовок, статус, решение о центре управления AI.
- **ADR‑0083**: заголовок, статус, решение о живом Telegram‑клиенте.
- **ADR‑0084**: заголовок, статус, решение о системе Persona Intelligence.
- **ADR‑0085**: заголовок, статус, решение о Communication Spine и движке противоречий.
- **ADR‑0086**: заголовок, статус, решение о постоянном хранении отношений.
- **ADR‑0087**: заголовок, статус, решение о сохранении наблюдений противоречий.
- **ADR‑0088**: заголовок, статус, решение о хранении обязательств.
- **ADR‑0089**: заголовок, статус, решение о хранении решений.
- **ADR‑0090**: заголовок, статус, решение об API‑мосте Persona.
- **ADR‑0091**: заголовок, статус, решение о модели возможностей Telegram (источник обрезан, но начальное описание покрыто).
- **ADR‑0092**: заголовок, статус, дата, решение о уровнях возможностей почтовых провайдеров.
- **ADR‑0093**: заголовок, статус, дата, решение о миграции на Vue 3.
- **ADR‑0094**: заголовок, статус (`Superseded by ADR‑0097`), дата, факт замены.
- **ADR‑0095**: заголовок, статус, решение о событийной коммуникации и DLQ.
- **ADR‑0096**: заголовок, статус, решение о канонических свидетельствах, инбоксе и контекстных пакетах.
- **ADR‑0097**: заголовок, статус, дата, решение о переводе каналов в интеграции.
- **ADR‑0098**: заголовок, статус, дата, решение о provider‑neutral API и строгих границах.
- **ADR‑0099**: заголовок, статус, дата, решение о Signal Hub и событийной платформе.

## Исходные файлы

- [`docs/adr/ADR-0077-i18n-russian-english.md`](../../../adr/ADR-0077-i18n-russian-english.md)
- [`docs/adr/ADR-0078-frontend-component-decomposition.md`](../../../adr/ADR-0078-frontend-component-decomposition.md)
- [`docs/adr/ADR-0079-script-logic-decomposition.md`](../../../adr/ADR-0079-script-logic-decomposition.md)
- [`docs/adr/ADR-0080-mail-background-sync-progress-local-trash.md`](../../../adr/ADR-0080-mail-background-sync-progress-local-trash.md)
- [`docs/adr/ADR-0081-opt-in-omniroute-ai-runtime.md`](../../../adr/ADR-0081-opt-in-omniroute-ai-runtime.md)
- [`docs/adr/ADR-0082-ai-settings-control-center.md`](../../../adr/ADR-0082-ai-settings-control-center.md)
- [`docs/adr/ADR-0083-telegram-live-user-client-runtime.md`](../../../adr/ADR-0083-telegram-live-user-client-runtime.md)
- [`docs/adr/ADR-0084-persona-intelligence-system.md`](../../../adr/ADR-0084-persona-intelligence-system.md)
- [`docs/adr/ADR-0085-communication-spine-and-contradiction-engine.md`](../../../adr/ADR-0085-communication-spine-and-contradiction-engine.md)
- [`docs/adr/ADR-0086-first-class-relationship-persistence.md`](../../../adr/ADR-0086-first-class-relationship-persistence.md)
- [`docs/adr/ADR-0087-contradiction-observation-persistence.md`](../../../adr/ADR-0087-contradiction-observation-persistence.md)
- [`docs/adr/ADR-0088-obligation-persistence.md`](../../../adr/ADR-0088-obligation-persistence.md)
- [`docs/adr/ADR-0089-decision-persistence.md`](../../../adr/ADR-0089-decision-persistence.md)
- [`docs/adr/ADR-0090-persona-native-compatibility-api-bridge.md`](../../../adr/ADR-0090-persona-native-compatibility-api-bridge.md)
- [`docs/adr/ADR-0091-telegram-production-client-capability-model.md`](../../../adr/ADR-0091-telegram-production-client-capability-model.md)
- [`docs/adr/ADR-0092-mail-provider-capability-tiers.md`](../../../adr/ADR-0092-mail-provider-capability-tiers.md)
- [`docs/adr/ADR-0093-frontend-platform-migration-to-vue-3.md`](../../../adr/ADR-0093-frontend-platform-migration-to-vue-3.md)
- [`docs/adr/ADR-0094-telegram-base-domain-completion-boundary.md`](../../../adr/ADR-0094-telegram-base-domain-completion-boundary.md)
- [`docs/adr/ADR-0095-event-driven-domain-communication-and-dlq.md`](../../../adr/ADR-0095-event-driven-domain-communication-and-dlq.md)
- [`docs/adr/ADR-0096-canonical-evidence-review-and-context-packs.md`](../../../adr/ADR-0096-canonical-evidence-review-and-context-packs.md)
- [`docs/adr/ADR-0097-communications-channel-domains-to-integrations.md`](../../../adr/ADR-0097-communications-channel-domains-to-integrations.md)
- [`docs/adr/ADR-0098-provider-neutral-communications-api-and-strict-boundaries.md`](../../../adr/ADR-0098-provider-neutral-communications-api-and-strict-boundaries.md)
- [`docs/adr/ADR-0099-signal-hub-event-platform.md`](../../../adr/ADR-0099-signal-hub-event-platform.md)

## Кандидаты на drift

На основании исключительно встроенных исходных файлов расхождений между кодом (недоступен в контексте) и текстами ADR не выявлено.
Среди самих ADR статусы замещения (`Superseded`) отражены корректно:

- ADR‑0078 и ADR‑0079 явно помечены как заменённые ADR‑0093;
- ADR‑0094 явно помечен как заменённый ADR‑0097;
- ADR‑0097, в свою очередь, уточнён ADR‑0098, что также отмечено в документе.

Иных признаков дрейфа документации в предоставленном чанке не обнаружено.
