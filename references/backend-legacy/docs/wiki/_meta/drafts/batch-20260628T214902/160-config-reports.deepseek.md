### Summary / Резюме

Страница `operations/configuration.md` обновляется описанием текущей конфигурации мониторинга производительности тестов: фиксируется формат JSON-отчётов, метрики, источники данных и примеры реальных замеров. Документация строится исключительно на основе встроенных файлов `reports/test-performance/backend-full.json` и `reports/test-performance/unit.json`. Страница объясняет, какие поля формируются в результате прогона тестов и как интерпретировать ключевые показатели (общее время, перцентили, flaky-тесты, самые медленные кейсы). Значения конфигурационных ключей или переменных окружения, отвечающих за генерацию отчётов, в предоставленном контексте не раскрыты – документируется только наблюдаемый результат.

### Proposed pages / Предлагаемые страницы

#### `operations/configuration.md`

```markdown
# Конфигурация мониторинга производительности тестов

В проекте настроен сбор метрик производительности тестовых наборов. Результаты сохраняются в JSON-файлы в директории `reports/test-performance/`. Ниже описан формат отчётов и фактические значения из последних запусков.

## Источник данных

Каждый отчёт генерируется на основе JUnit XML-результатов тестового фреймворка. Поле `source` указывает путь к этому файлу:

```json
"source": "target/nextest/default/junit.xml"
```

Путь свидетельствует о том, что для запуска тестов используется **cargo nextest**, а результаты экспортируются в стандартный JUnit-формат. Детали конфигурации самого nextest (файл `.config/nextest.toml` или переменные окружения) в данном контексте не подтверждены.

## Структура отчёта

Каждый JSON-отчёт содержит следующие поля:

| Поле | Тип | Описание |
|---|---|---|
| `suite` | string | Идентификатор набора тестов (например, `"backend-full"` или `"unit"`). |
| `generatedAt` | string (ISO 8601) | Временная метка создания отчёта. |
| `source` | string | Путь к исходному JUnit XML-файлу. |
| `totalTests` | number | Общее количество тестов в наборе. |
| `failedTests` | number | Количество упавших тестов (в представленных отчётах – 0). |
| `flakyTests` | array of strings | Список идентификаторов тестов, помеченных как нестабильные (flaky). |
| `totalSeconds` | number | Суммарное время выполнения всех тестов (в секундах). |
| `averageSeconds` | number | Среднее время выполнения одного теста. |
| `p95Seconds` | number | 95-й перцентиль времени выполнения. |
| `p99Seconds` | number | 99-й перцентиль времени выполнения. |
| `slowest` | array of objects | Список десяти самых медленных тестов с детализацией. |

### Поля объекта в `slowest`

| Поле | Тип | Описание |
|---|---|---|
| `id` | string | Полный идентификатор теста (включает модуль и функцию). |
| `timeSeconds` | number | Время выполнения теста в секундах. |
| `failed` | boolean | Упал ли тест (во всех представленных случаях `false`). |
| `flaky` | boolean | Является ли тест flaky (во всех представленных случаях `false`). |

## Фактические метрики последних запусков

### Набор `backend-full`

- **Дата генерации**: 2026-06-28T18:53:54.434Z
- **Тестов всего**: 1401
- **Упавших**: 0
- **Flaky-тесты**:
  - `makosh-backend::event_platform::event_outbox_dispatcher_publishes_pending_events_to_nats`
- **Суммарное время**: 6710.525 с
- **Среднее время**: 4.79 с
- **P95**: 12.795 с, **P99**: 20.413 с
- **Самые медленные тесты** (топ-10):
  1. `…graph_api::search::graph_summary_returns_empty_state_for_empty_database` – 34.693 с
  2. `…tasks::task_checklist_against_postgres` – 31.357 с
  3. `…graph_api::neighborhood::graph_neighborhood_caps_depth_one_edges_nodes_and_evidence` – 29.265 с
  4. `…tasks_api::mutations::task_post_subtask` – 27.194 с
  5. `…graph_api::neighborhood::graph_neighborhood_caps_evidence_for_returned_edges` – 26.858 с
  6. `…tasks_api::mutations::task_post_relation` – 26.622 с
  7. `…whatsapp::whatsapp_runtime_bridge_participant_reconciles_join_group_command_with_live_provenance` – 26.523 с
  8. `…whatsapp::whatsapp_runtime_bridge_presence_and_call_record_live_observed_source_in_raw_provenance` – 26.095 с
  9. `…graph_api::neighborhood::graph_neighborhood_returns_selected_node_neighbors_edges_and_evidence` – 25.841 с
  10. `…graph_api::search::graph_nodes_returns_connected_picker_nodes_first` – 24.99 с

### Набор `unit`

- **Дата генерации**: 2026-06-27T23:18:55.999Z
- **Тестов всего**: 277
- **Упавших**: 0
- **Flaky-тесты**: отсутствуют
- **Суммарное время**: 200.45 с
- **Среднее время**: 0.724 с
- **P95**: 6.467 с, **P99**: 9.608 с
- **Самые медленные тесты** (топ-10):
  1. `…telegram::runtime::manager::participants::sync_provider_roster_snapshots_appends_leave_reconciliation_after_absence_update` – 10.343 с
  2. `…telegram::runtime::manager::chat_events::publish_chat_unread_event_reconciles_mark_read_command_and_emits_events` – 9.797 с
  3. `…telegram::runtime::manager::message_events::publish_message_edited_event_skips_without_projected_message` – 9.608 с
  4. `…telegram::runtime::manager::message_events::publish_message_created_event_publishes_signal_hub_raw_signal_instead_of_legacy_event` – 9.447 с
  5. `…telegram::runtime::manager::message_events::publish_reaction_changed_event_skips_without_projected_message` – 9.127 с
  6. `…telegram::runtime::manager::chat_events::publish_chat_position_event_reconciles_folder_add_and_remove_commands` – 9.061 с
  7. `…telegram::runtime::manager::realtime_events::telegram_runtime_event_bridge_skips_broadcast_when_runtime_paused` – 9.033 с
  8. `…telegram::runtime::manager::realtime_events::typing_tests::publish_command_reconciled_events_appends_status_and_reconciled_records` – 8.961 с
  9. `…telegram::runtime::manager::topic_events::publish_topic_event_reconciles_topic_close_and_appends_runtime_events` – 8.746 с
  10. `…telegram::runtime::manager::message_events::publish_message_content_updated_event_skips_without_projected_message` – 8.677 с

## Примечания

- Значения `failed` в списке `slowest` всегда `false`, так как отчёт формируется только для успешных и flaky-тестов; упавшие тесты (если бы они были) отображались бы в поле `failedTests` и, вероятно, прерывали бы генерацию отчёта. Это предположение не подтверждено предоставленным контекстом.
- Конкретный механизм пометки теста как flaky (порог повторяемости, ручная аннотация, конфигурация nextest) в данном контексте не раскрыт.
- Путь `target/nextest/default/junit.xml` указывает на использование профиля nextest по умолчанию (`default`). Наличие других профилей или кастомной конфигурации не подтверждено.
```

### Source coverage / Покрытие источников

- **`reports/test-performance/backend-full.json`** (2328 символов, не обрезан)
  - Структура отчёта (`suite`, `generatedAt`, `source`, `totalTests`, `failedTests`, `flakyTests`, `totalSeconds`, `averageSeconds`, `p95Seconds`, `p99Seconds`, `slowest`).
  - Конкретное значение `source`: `target/nextest/default/junit.xml`.
  - Фактические метрики для набора `backend-full`: 1401 тест, 0 упавших, 1 flaky-тест, суммарное время 6710.525 с, среднее 4.79 с, перцентили 12.795 с / 20.413 с.
  - Список 10 самых медленных тестов с идентификаторами и временем.
  - Идентификатор flaky-теста: `makosh-backend::event_platform::event_outbox_dispatcher_publishes_pending_events_to_nats`.

- **`reports/test-performance/unit.json`** (2876 символов, не обрезан)
  - Аналогичная структура отчёта для набора `unit`.
  - Фактические метрики: 277 тестов, 0 упавших, flaky отсутствуют, суммарное время 200.45 с, среднее 0.724 с, перцентили 6.467 с / 9.608 с.
  - Список 10 самых медленных тестов с идентификаторами и временем.

### Drift candidates / Кандидаты на drift

Из предоставленного контекста расхождения между кодом, документацией и ADR не видны. Оба отчёта имеют идентичную структуру, `failedTests: 0` соответствует отсутствию упавших тестов. Единственный зафиксированный flaky-тест (`event_outbox_dispatcher_publishes_pending_events_to_nats`) присутствует только в наборе `backend-full` и не дублируется в `unit`, что ожидаемо, так как это разные наборы. Признаков drift не обнаружено.
