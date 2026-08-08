---
chunk_id: 116-doc-docs-part-007
batch_id: batch-20260628T214902
group: docs
role: doc
source_status: pending
source_count: 16
generated_by: code-wiki-ru
---

# 116-doc-docs-part-007 — docs/doc

- Target index: [[operations/documentation-map]]
- Batch: `batch-20260628T214902`
- Source files: `16`

## Резюме

Необходимо создать страницу `operations/documentation-map.md` в русской Obsidian‑вики.
Она даст обзор доступной документации по интеграционным провайдерам Макошь (WhatsApp, Yandex Telemost, Zoom), поможет ориентироваться в наборе документов и понимать текущий статус каждого провайдера.
Страница строится исключительно на основе встроенных исходных файлов секции `docs/integrations/*`.

---

## Предложенные страницы

#### `operations/documentation-map.md`

```markdown
# Карта документации операций

Эта страница содержит обзор документации по интеграционным провайдерам Макошь.
Исходные документы расположены в `docs/integrations/`.

---

## WhatsApp

WhatsApp – integration‑провайдер, поставляющий source evidence в домен Communications.
Он не является самостоятельным доменом.

### Основные документы

- **[WhatsApp Full Functionality Target](/docs/integrations/whatsapp/full-functionality-target.md)**
  Целевая спецификация полной функциональности WhatsApp в Макошь (от 2026‑06‑24).
  Описывает модель провайдера, инварианты, формы провайдера (`whatsapp_web_companion`, `whatsapp_native_md`, `whatsapp_business_cloud`) и функциональную матрицу (аккаунты, диалоги, сообщения, реакции, медиа и т.д.).

- **[WhatsApp Gap Analysis](/docs/integrations/whatsapp/gap-analysis.md)**
  Анализ пробелов (от 2026‑06‑17).
  Оценивает уровень реализации каждой возможности по меткам `IMPLEMENTED`, `PARTIAL`, `MISSING`, `UNSUPPORTED` и т.п.
  Содержит приоритетные рекомендации (P0 – документация, P1 – read‑only evidence, P2 – provider‑write parity).

- **[WhatsApp Implementation Plan](/docs/integrations/whatsapp/implementation-plan.md)**
  План реализации (от 2026‑06‑24).
  Фазы: P0 – закрытие документационных вопросов, P1 – контракт provider runtime (включая трейт `WhatsAppProviderRuntime`), P2 – спайк сторонней Rust‑библиотеки.

- **[WhatsApp Live Smoke Checklist](/docs/integrations/whatsapp/live-smoke-checklist.md)**
  Чеклист ручного smoke‑тестирования локального runtime (от 2026‑06‑26).
  Используется только для owner‑visible локальной валидации. Включает проверки границ runtime, WebView companion, авторизации, read/write‑путей, редкации секретов и сценариев восстановления.

- **[WhatsApp Modules](/docs/integrations/whatsapp/modules.md)**
  Карта целевых модулей backend и frontend (target module map, от 2026‑06‑17).
  Задаёт структуру модулей, их назначение и статус (`MISSING` / `PARTIAL`). Содержит правила именования и границы модулей.

- **[WhatsApp Rust Provider Research](/docs/integrations/whatsapp/rust-provider-research.md)**
  Исследование Rust‑библиотек для нативного WhatsApp‑провайдера (от 2026‑06‑26).
  После неудачного compile‑спайка `whatsapp-rust 0.6.0` выбран `wa-rs 0.2.0` с отключённым SDK SQLite‑хранилищем. Рекомендована строгая граница адаптера.

- **[WhatsApp Implementation Status](/docs/integrations/whatsapp/status.md)**
  Сводный статус реализации (от 2026‑06‑27).
  На момент документа: 67 имплементированных чекпоинтов, domain closure не достигнут.
  Перечислены live‑блокеры: ручные smoke‑артефакты, недостающие safe write API, WebView live smoke, Business Cloud smoke.
  Детально описано, что уже работает (provider/account model, host‑vault boundary, runtime health, capability contract, fixture runtime, command outbox, message ingestion/projection, realtime, frontend workbench, native‑md compile boundary и др.).

### Текущий статус (кратко)

- Closure заблокирован: отсутствуют валидированные live‑smoke‑артефакты для `whatsapp_native_md`, `whatsapp_web_companion` и `whatsapp_business_cloud`.
- Часть возможностей (`archive`, `unarchive`, `mute`, `unmute`, `pin`, `unpin`, `mark_unread`, `join_group`, `publish_status`) ещё не подтверждена безопасными provider API и smoke‑тестами.

---

## Yandex Telemost

Yandex Telemost – integration‑провайдер для конференций.
Поставляет conference metadata, join links, локальные записи и speaker‑timeline‑подсказки.

### Основные документы

- **[Yandex Telemost README](/docs/integrations/yandex-telemost/README.md)**
  Обзор провайдера, foundation scope, provider kind (`yandex_telemost_user`), secret purpose (`yandex_telemost_oauth_token`), структура локальных записей и навигация (статус `FOUNDATION_PATCH_APPLIED`, от 2026‑06‑28).

- **[Yandex Telemost API](/docs/integrations/yandex-telemost/api.md)**
  Backend routes, Tauri‑команды, схемы запросов для аккаунтов, конференций, WebView‑манифеста, локальной записи и транскрипции. Описаны политики retention и автоматический вызов транскрайбера.

- **[Yandex Telemost Architecture](/docs/integrations/yandex-telemost/architecture.md)**
  Архитектурные границы, inbound/outbound потоки, модель видимого WebView (подсказки динамика помечены как `hint_not_truth`), модель локального рекордера на `ffmpeg`, список событий и secret policy.

- **[Yandex Telemost Implementation Plan](/docs/integrations/yandex-telemost/implementation-plan.md)**
  План по стадиям: Foundation, Conference API, Desktop companion, Provider-neutral projection, Transcription workflow. Явно перечислены non‑goals.

- **[Yandex Telemost Live Smoke Checklist](/docs/integrations/yandex-telemost/live-smoke-checklist.md)**
  Чеклист ручного smoke‑тестирования backend и desktop компонентов при наличии HostVault, ffmpeg и OAuth‑токена.

- **[Yandex Telemost Local Recording Contract](/docs/integrations/yandex-telemost/local-recording.md)**
  Спецификация локального рекордера: consent gate, раскладка файлов (`audio.mp3`, `speaker-timeline.jsonl`, `speaker-timeline.txt`), формат speaker‑timeline‑подсказок, платформенная стратегия (Linux/macOS/Windows), переменные окружения, retention‑политика и политика проекции.

- **[Yandex Telemost Modules](/docs/integrations/yandex-telemost/modules.md)**
  Структура модулей backend (`integrations/yandex_telemost/`), desktop runtime (`yandex_telemost_companion.rs`) и frontend.

- **[Yandex Telemost Status](/docs/integrations/yandex-telemost/status.md)**
  Статус `FOUNDATION_PATCH_APPLIED` (от 2026‑06‑28). Известных follow‑up gaps нет. Приведены конкретные тесты для валидации.

### Текущий статус (кратко)

- Foundation применён, пробелов не выявлено.
- Неподдерживаемые сценарии (hidden capture, automatic join и т.п.) зафиксированы как явные non‑goals.

---

## Zoom

Zoom – integration‑провайдер для встреч, записей и транскриптов.

### Основные документы

- **[Zoom README](/docs/integrations/zoom/README.md)**
  Обзор провайдера, foundation scope, provider kinds (`zoom_user`, `zoom_server_to_server`), secret purposes, текущий scope доступных/неподдерживаемых возможностей и навигация (статус `FOUNDATION_IMPLEMENTED`, от 2026‑06‑28).

Дополнительные документы (полный текст не включён в данный контекстный пакет, но они перечислены в `README`):

- `architecture.md` – архитектура
- `api.md` – API‑поверхность
- `modules.md` – модули
- `status.md` – статус реализации
- `gap-analysis.md` – анализ пробелов
- `blockers.md` – блокеры
- `implementation-plan.md` – план реализации
- `fixture-test-matrix.md` – матрица fixture‑тестов
- `live-smoke-checklist.md` – чеклист smoke‑тестирования
- `provider-runtime-research.md` – исследование рантайма провайдера
- [ADR‑0102 Zoom Provider Runtime Boundary](/docs/adr/ADR-0102-zoom-provider-runtime-boundary.md)

### Текущий статус (кратко)

- Foundation реализован: fixture‑аккаунты, OAuth/S2S авторизация, webhook‑менеджмент, синхронизация записей, загрузка медиа и транскриптов, edge‑proxy, retention, аудит, интеграция с Communications `calls/meetings`.
- Owner‑visible политики конфиденциальности управляют загрузкой удалённых записей и транскриптов.
- ADR‑0102 принят.

---

## Примечание

Данная карта отражает состояние документации на даты, указанные в каждом документе.
Для актуальной информации обращайтесь непосредственно к исходным файлам.
```

---

## Покрытие источников

| Исходный файл | Покрытые факты |
|---|---|
| `docs/integrations/whatsapp/full-functionality-target.md` | Цель полной функциональности WhatsApp, инвариант «канал не домен», формы провайдера, функциональная матрица (аккаунты, сессии, диалоги, сообщения, реакции, медиа), текущий fixture‑фундамент. |
| `docs/integrations/whatsapp/gap-analysis.md` | Метки статуса (`IMPLEMENTED`, `PARTIAL`, `MISSING`, `UNSUPPORTED`), оценка возможностей, приоритетные рекомендации P0‑P2. |
| `docs/integrations/whatsapp/implementation-plan.md` | Фазы P0–P2, trait `WhatsAppProviderRuntime`, структура backend‑модулей, приемка для фазы P1. |
| `docs/integrations/whatsapp/live-smoke-checklist.md` | Чеклист ручной валидации, evidence‑артефакт, проверки границ runtime, WebView companion, авторизации, read/write‑путей, редкации. |
| `docs/integrations/whatsapp/modules.md` | Карта целевых модулей backend/frontend, их статусы (`MISSING`/`PARTIAL`), правила именования. |
| `docs/integrations/whatsapp/rust-provider-research.md` | Результаты спайка Rust‑библиотек, кандидаты (`whatsapp-rust`, `wa-rs`), рекомендованная архитектура адаптера. |
| `docs/integrations/whatsapp/status.md` | Текущий статус (67 чекпоинтов, closure заблокирован), live‑блокеры, перечень уже реализованных компонентов. |
| `docs/integrations/yandex-telemost/README.md` | Статус `FOUNDATION_PATCH_APPLIED`, provider kind, secret purpose, foundation scope, структура локальных записей, навигация. |
| `docs/integrations/yandex-telemost/api.md` | Backend routes, Tauri‑команды, схемы для аккаунтов, конференций, WebView, записи и транскрипции, retention‑политика. |
| `docs/integrations/yandex-telemost/architecture.md` | Границы, inbound/outbound потоки, модель WebView (hint_not_truth), модель локального рекордера, список событий, secret policy. |
| `docs/integrations/yandex-telemost/implementation-plan.md` | Стадии Foundation, Conference API, Desktop companion, Projection, Transcription workflow, non‑goals. |
| `docs/integrations/yandex-telemost/live-smoke-checklist.md` | Предусловия, шаги backend и desktop smoke‑тестирования. |
| `docs/integrations/yandex-telemost/local-recording.md` | Consent gate, layout выходных файлов, формат speaker‑timeline‑подсказок, платформенная стратегия, переменные окружения, политики retention и проекции. |
| `docs/integrations/yandex-telemost/modules.md` | Структура модулей backend, desktop runtime и frontend. |
| `docs/integrations/yandex-telemost/status.md` | Статус `FOUNDATION_PATCH_APPLIED`, команды валидации, отсутствие follow‑up gaps. |
| `docs/integrations/zoom/README.md` | Статус `FOUNDATION_IMPLEMENTED`, foundation scope, provider kinds, secret purposes, текущий scope, навигация к остальным документам (перечислены). |

---

## Исходные файлы

- [`docs/integrations/whatsapp/full-functionality-target.md`](../../../integrations/whatsapp/full-functionality-target.md)
- [`docs/integrations/whatsapp/gap-analysis.md`](../../../integrations/whatsapp/gap-analysis.md)
- [`docs/integrations/whatsapp/implementation-plan.md`](../../../integrations/whatsapp/implementation-plan.md)
- [`docs/integrations/whatsapp/live-smoke-checklist.md`](../../../integrations/whatsapp/live-smoke-checklist.md)
- [`docs/integrations/whatsapp/modules.md`](../../../integrations/whatsapp/modules.md)
- [`docs/integrations/whatsapp/rust-provider-research.md`](../../../integrations/whatsapp/rust-provider-research.md)
- [`docs/integrations/whatsapp/status.md`](../../../integrations/whatsapp/status.md)
- [`docs/integrations/yandex-telemost/README.md`](../../../integrations/yandex-telemost/README.md)
- [`docs/integrations/yandex-telemost/api.md`](../../../integrations/yandex-telemost/api.md)
- [`docs/integrations/yandex-telemost/architecture.md`](../../../integrations/yandex-telemost/architecture.md)
- [`docs/integrations/yandex-telemost/implementation-plan.md`](../../../integrations/yandex-telemost/implementation-plan.md)
- [`docs/integrations/yandex-telemost/live-smoke-checklist.md`](../../../integrations/yandex-telemost/live-smoke-checklist.md)
- [`docs/integrations/yandex-telemost/local-recording.md`](../../../integrations/yandex-telemost/local-recording.md)
- [`docs/integrations/yandex-telemost/modules.md`](../../../integrations/yandex-telemost/modules.md)
- [`docs/integrations/yandex-telemost/status.md`](../../../integrations/yandex-telemost/status.md)
- [`docs/integrations/zoom/README.md`](../../../integrations/zoom/README.md)

## Кандидаты на drift

1. **WhatsApp Gap Analysis vs. Implementation Status**
   `gap-analysis.md` (от 2026‑06‑17) помечает почти все возможности как `MISSING`, в то время как `status.md` (от 2026‑06‑27) фиксирует 67 имплементированных чекпоинтов.
   Это расхождение во времени: gap‑анализ не обновлён после прогресса, отражённого в `status.md`.
   Рекомендуется актуализировать `gap-analysis.md` в соответствии с текущим состоянием.

2. **WhatsApp Modules vs. Implementation Status**
   `modules.md` (target module map) всё ещё показывает большинство модулей как `MISSING`, хотя `status.md` подтверждает частичную реализацию многих из них (например, provider/account model, runtime health, capability contract, fixture runtime, command outbox, message projection и др.).
   Возможно, `modules.md` намеренно является целевой картой, а не отчётом о состоянии, но это может вводить в заблуждение. Стоит либо обновить статусы модулей, либо явно указать, что документ отражает целевое состояние.

3. Иных расхождений в предоставленном контексте не обнаружено.
