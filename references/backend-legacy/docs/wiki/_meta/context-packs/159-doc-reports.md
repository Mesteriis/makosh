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

- Chunk ID / ID чанка: `159-doc-reports`
- Group / Группа: `reports`
- Role / Роль: `doc`
- Status / Статус: `pending`
- Repository / Репозиторий: `/Users/avm/projects/Personal/makosh`
- Wiki path / Путь wiki: `/Users/avm/projects/Personal/makosh/docs/wiki`
- Metadata path / Путь metadata: `/Users/avm/projects/Personal/makosh/docs/wiki/_meta`
- Plan generated at / План создан: `2026-06-28T19:48:55Z`
- Per-file source limit / Лимит источника на файл: `12000` characters

## Target pages / Целевые страницы

- `operations/documentation-map.md`

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

### `reports/test-performance/2026-06-23-baseline.md`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/reports/test-performance/2026-06-23-baseline.md`
- Size bytes / Размер в байтах: `2398`
- Included characters / Включено символов: `2398`
- Truncated / Обрезано: `no`

```markdown
# Макошь test baseline - 2026-06-23

## Observed current state

- Backend test tree volume: `233` files under `backend/tests`, with `151` top-level Rust test targets at audit time.
- Frontend test/spec files discovered statically: `132`
- Rust `#[test]` / `#[tokio::test]` attributes discovered statically under `backend/src`, `backend/tests`, `crates/testkit`: `1121`
- Rust unit-test attributes discovered statically under `backend/src` and `crates/testkit/src`: `263`
- Testcontainers runtime exists already in `crates/testkit` for PostgreSQL and NATS.
- `cargo-nextest` wiring already existed only as a fallback path in `Makefile`, not as the default structured test system.

## Slowest backend suites observed before modernization

These durations were observed in real backend runs before this modernization pass:

1. `backend/tests/calendar_api.rs` - about `196-197s`
2. `backend/tests/persons_api.rs` - about `167s`
3. `backend/tests/v1_communications_api.rs` - about `128-143s`
4. `backend/tests/tasks_api.rs` - about `117-119s`
5. `backend/tests/organizations_api.rs` - about `100-107s`
6. `backend/tests/signal_hub.rs` - about `95-97s`
7. `backend/tests/tasks.rs` - about `94-97s`
8. `backend/tests/document_processing.rs` - about `72-76s`
9. `backend/tests/persons.rs` - about `70-76s`
10. `backend/tests/messages.rs` - about `56-68s`

## Heaviest compile/runtime hotspots

Observed repeatedly during backend test runs:

- `ring`
- `rustls`
- `rustls-webpki`
- `sqlx-core`
- `sqlx-postgres`
- `reqwest`
- `async-nats`
- `bollard`
- `testcontainers`
- `testcontainers-modules`

## Main bottlenecks

1. Full backend runs depend on a heavy integration surface and many top-level test targets.
2. Container-backed tests are expensive when the `makosh_test_session` harness is bypassed.
3. CI had no dedicated split for snapshots, integration, coverage, security, and nightly mutation runs.
4. Reports for slow tests and flaky tests were not generated automatically.
5. Local tool installation was undocumented and inconsistent.

## Safe modernization boundary

- Keep PostgreSQL as the main persistent store.
- Keep Docker development infra under `docker/` per `ADR-0032`.
- Keep backend full-suite execution behind the `crates/testkit` harness.
- Do not introduce Redis artificially into the test stack unless a real Макошь subsystem requires it and the architecture decision is explicit.
```

### `reports/test-performance/2026-06-23-testcontainers-audit.md`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/reports/test-performance/2026-06-23-testcontainers-audit.md`
- Size bytes / Размер в байтах: `1715`
- Included characters / Включено символов: `1715`
- Truncated / Обрезано: `no`

```markdown
# Testcontainers audit - 2026-06-23

## Verified current behavior

- `crates/testkit/src/context.rs` keeps PostgreSQL and NATS containers in `tokio::sync::OnceCell`.
- `crates/testkit/src/bin/makosh_test_session.rs` creates a single owned PostgreSQL session container for the full backend run and tears it down when the command exits.
- Individual tests create isolated databases inside the shared PostgreSQL container instead of starting a new PostgreSQL container per test.
- NATS is started lazily only for tests that request it through `TestContext::nats_server_url()` / `app_config_with_nats()`.

## Main risk found

Direct full-suite `cargo test` bypasses `makosh_test_session`. That removes the shared-session contract and is the main path that can leave extra Docker garbage behind after interrupts or failures.

## Changes in this modernization pass

1. Full backend test and coverage entry points are now routed through `scripts/test/run-nextest.sh` and `scripts/test/run-llvm-cov.sh`.
2. Repository guidance explicitly keeps `make backend-test` / `make backend-validate` as the safe harness entry points.
3. CI jobs are split so heavy container-backed lanes run separately from unit/snapshot lanes.

## Remaining optimization opportunities

1. Move more backend logic from integration targets into lib/unit tests where Docker is not needed.
2. Identify the slowest container-backed suites and merge repeated bootstraps inside the same target where practical.
3. Add per-target nextest grouping once there is enough measured data to justify serializing specific resource-heavy targets.
4. Keep Redis out of the test stack unless a real Макошь subsystem needs it and the architecture decision is explicit.
```

### `reports/test-performance/README.md`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/reports/test-performance/README.md`
- Size bytes / Размер в байтах: `552`
- Included characters / Включено символов: `552`
- Truncated / Обрезано: `no`

```markdown
# Test Performance Reports

This directory stores committed baseline measurements and human-readable optimization notes for Макошь test infrastructure.

Generated summaries from `scripts/test/analyze-nextest-junit.mjs` may also be written here during explicit test runs.

Current baseline:

- `2026-06-23-baseline.md`
- `2026-06-23-testcontainers-audit.md`
- `unit.md` / `unit.json` after a real `make test-unit`
- `backend-full.md` / `backend-full.json` after a real `make backend-test`

Current status matrix:

- `docs/development/testing/status.md`
```

### `reports/test-performance/backend-full.md`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/reports/test-performance/backend-full.md`
- Size bytes / Размер в байтах: `1334`
- Included characters / Включено символов: `1334`
- Truncated / Обрезано: `no`

```markdown
# backend-full nextest report

- Generated at: 2026-06-28T18:53:54.434Z
- Source JUnit: `target/nextest/default/junit.xml`
- Total tests: 1401
- Failed tests: 0
- Flaky tests: 1
- Total time: 6710.525s
- Average time: 4.79s
- p95: 12.795s
- p99: 20.413s

## Slowest tests

1. `makosh-backend::graph_api::search::graph_summary_returns_empty_state_for_empty_database` - 34.693s
2. `makosh-backend::tasks::task_checklist_against_postgres` - 31.357s
3. `makosh-backend::graph_api::neighborhood::graph_neighborhood_caps_depth_one_edges_nodes_and_evidence` - 29.265s
4. `makosh-backend::tasks_api::mutations::task_post_subtask` - 27.194s
5. `makosh-backend::graph_api::neighborhood::graph_neighborhood_caps_evidence_for_returned_edges` - 26.858s
6. `makosh-backend::tasks_api::mutations::task_post_relation` - 26.622s
7. `makosh-backend::whatsapp::whatsapp_runtime_bridge_participant_reconciles_join_group_command_with_live_provenance` - 26.523s
8. `makosh-backend::whatsapp::whatsapp_runtime_bridge_presence_and_call_record_live_observed_source_in_raw_provenance` - 26.095s
9. `makosh-backend::graph_api::neighborhood::graph_neighborhood_returns_selected_node_neighbors_edges_and_evidence` - 25.841s
10. `makosh-backend::graph_api::search::graph_nodes_returns_connected_picker_nodes_first` - 24.99s
```

### `reports/test-performance/unit.md`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/reports/test-performance/unit.md`
- Size bytes / Размер в байтах: `1996`
- Included characters / Включено символов: `1996`
- Truncated / Обрезано: `no`

```markdown
# unit nextest report

- Generated at: 2026-06-27T23:18:55.999Z
- Source JUnit: `target/nextest/default/junit.xml`
- Total tests: 277
- Failed tests: 0
- Flaky tests: none detected
- Total time: 200.45s
- Average time: 0.724s
- p95: 6.467s
- p99: 9.608s

## Slowest tests

1. `makosh-backend::integrations::telegram::runtime::manager::participants::participants_runtime_tests::sync_provider_roster_snapshots_appends_leave_reconciliation_after_absence_update` - 10.343s
2. `makosh-backend::integrations::telegram::runtime::manager::chat_events::tests::publish_chat_unread_event_reconciles_mark_read_command_and_emits_events` - 9.797s
3. `makosh-backend::integrations::telegram::runtime::manager::message_events::tests::publish_message_edited_event_skips_without_projected_message` - 9.608s
4. `makosh-backend::integrations::telegram::runtime::manager::message_events::tests::publish_message_created_event_publishes_signal_hub_raw_signal_instead_of_legacy_event` - 9.447s
5. `makosh-backend::integrations::telegram::runtime::manager::message_events::tests::publish_reaction_changed_event_skips_without_projected_message` - 9.127s
6. `makosh-backend::integrations::telegram::runtime::manager::chat_events::tests::publish_chat_position_event_reconciles_folder_add_and_remove_commands` - 9.061s
7. `makosh-backend::integrations::telegram::runtime::manager::realtime_events::tests::telegram_runtime_event_bridge_skips_broadcast_when_runtime_paused` - 9.033s
8. `makosh-backend::integrations::telegram::runtime::manager::realtime_events::typing_tests::publish_command_reconciled_events_appends_status_and_reconciled_records` - 8.961s
9. `makosh-backend::integrations::telegram::runtime::manager::topic_events::tests::publish_topic_event_reconciles_topic_close_and_appends_runtime_events` - 8.746s
10. `makosh-backend::integrations::telegram::runtime::manager::message_events::tests::publish_message_content_updated_event_skips_without_projected_message` - 8.677s
```
