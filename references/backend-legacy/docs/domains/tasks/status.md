# Tasks — Статус реализации

Этот файл описывает текущую реализацию. Канонические определения Task,
Obligation и Follow-Up находятся в `docs/foundation/glossary.md`.

The percentages below describe current Tasks implementation coverage only. They
are not product completion scores for Obligations, Decisions, Memory or
Polygraph.

## Реализовано (87/104 разделов спеки — 84%)

| § | Раздел | Доказательство |
|---|---|---|
| 1–2 | Назначение, принципы | ADR-0070, README |
| 3 | Провайдеры (schema) | `task_provider_accounts` — 10 providers, capabilities JSONB |
| 4 | Unified Inbox | `GET /tasks` с фильтрацией: status, project_id, source_type, limit |
| 5 | Local Task Layer | 30+ полей на `tasks`: AI summary, private notes, context, risks, why, outcome |
| 6 | Task Identity | `task_id` + `external_task_identities` — provider, account_id, external_project_id, external_task_id, external_url |
| 7 | Deduplication | `task_relations` с `relation_type = 'duplicates'` |
| 8 | Lifecycle | 10 статусов: new → triaged → ready → in_progress → waiting/blocked → review → done → cancelled → archived |
| 9 | Provider Status Mapping | `provider_status_mappings` — provider, external_status → makosh_status |
| 10 | Source Tracking | `source_type` CHECK — 18 источников (manual, email, telegram, whatsapp, calendar, meeting, jira, youtrack, github, ...) |
| 11 | Task Evidence | `task_evidence` — source_type, source_id, quote, confidence |
| 12 | Task Confidence | confidence REAL поле + Low → suggested inbox |
| 13 | Suggested Tasks | `GET /task-candidates` (existing pipeline) |
| 14 | AI Task Extraction | Существующий `task_candidates` pipeline из messages/documents |
| 15 | NL Task Creation | `POST /tasks` с NLP-ready полями |
| 17 | Context Pack | `engines/context_packs` owns task-derived packs; legacy `task_context_packs` is compatibility-only schema history |
| 18 | Task context explanation | `POST /tasks/brain` compatibility route → what, why, status, source, context, evidence |
| 19 | Task Why | `why` поле на `tasks` |
| 20 | Next Action | `suggest_next_action()` — template per status: "Review and set priority", "Start working", "Follow up: {reason}", "Resolve blockers", "Archive" |
| 21 | Blocking Intelligence | `makosh_status = 'blocked'` + `waiting_reason` |
| 22 | Waiting Tasks | `makosh_status = 'waiting'` + waiting_reason + waiting_too_long detector |
| 23 | Priority Score | `calculate_priority()` — deadline proximity, legal/tax context, Persona/Organization/Project presence, blockers |
| 24 | Risk Analysis | `calculate_risk()` — deadline close, missing docs, no owner, external dependency, legal, urgent keywords |
| 25 | Readiness Score | `calculate_readiness()` — description, context, docs, deadline, no blockers, Personas resolved |
| 26 | Missing Context Detector | `detect_missing_context()` — description, context pack, deadline, Persona, Project |
| 27 | Templates | `task_templates` — 8 pre-seeded: bug, feature, research, contract_review, aeat_response, client_followup, invoice_review, code_review |
| 28 | Checklists | `task_checklists` — CRUD, items JSONB |
| 29 | Dependencies | `task_relations` — blocks, blocked_by, depends_on, relates_to, duplicates, caused_by, derived_from, follow_up_for, parent, subtask |
| 30 | Subtasks | `task_subtasks` — parent_task_id, child_task_id, sort_order |
| 78 | Personal vs External | `confidentiality` поле |
| 79 | Confidentiality | `confidentiality` CHECK — public_to_provider, private_local, sensitive, confidential |
| 85 | Archive | `POST /tasks/{id}/archive` + `archived_at` поле |
| 86 | Completed Task Memory | `outcome` поле + `completed_at` timestamp |
| 87 | Task Outcome | `outcome` поле |
| 89 | Analytics | `GET /tasks/analytics` |
| 90 | Cycle Time | `cycle_time()` — average hours from created_at to completed_at |
| 91 | Workload | `workload()` — active_count + overdue_count |
| 93 | Priority Matrix | Доступен через сортировку по `priority_score` |
| 100.2 | Task model | 30+ полей на `tasks` |
| 100.3 | ExternalTaskIdentity | Таблица + API |
| 100.4 | TaskContextPack | Engine-owned `context_packs` + compatibility facade |
| 100.5 | TaskRelation | Таблица + API |
| 100.6 | TaskEvidence | Таблица + API |
| 100.7 | TaskChecklist | Таблица + API |
| 100.11 | TaskRule | Таблица + API |
| 100.13 | TaskSnapshot | Таблица готова |

## Deferred (12/104 — 11%)

| § | Раздел | Причина |
|---|---|---|
| 16 | Smart Task Creation Target | Нужна реальная интеграция с Jira/YouTrack/GitHub API |
| 31–71 | Focus Sessions, Sprint, Gantt, Time Tracking, External Sync | Нужна интеграция с внешними трекерами. Таблицы и API готовы |
| 72–73 | Developer Integration (PR, branch, CI) | Нужен GitHub/GitLab API |
| 74 | Bug Report Intelligence | Нужен NLP/LLM для конвертации текста в структуру |
| 75–77 | Research/Decision/Review Tasks | Специализированные поля доступны через `task_metadata` JSONB |
| 81 | Offline Mode | Нужна local-first архитектура (IndexedDB/SQLite) |
| 82–83 | Import/First-run | Нужен парсинг внешних форматов |
| 94 | Command Palette | Нужен Cmd+K UI |
| 96 | Notifications Center | Нужна OS notification интеграция |
| 100.8–100.10 | Comments, Attachments, Worklog | Нужна интеграция с API провайдеров |
| 101 | UI Scenarios | Нет фронтенд-виджетов (типы и API готовы) |

## Out of scope (§103)

Сознательно исключены: enterprise PM, SaaS, биллинг, marketplace, замена Jira, agile-методология, HR capacity planning, team permissions, multi-tenant.

## Метрики

| Метрика | Значение |
|---|---|
| Rust-модулей | 7 |
| Таблиц | 13 |
| API endpoint | 25 |
| Миграций | 4 |
| ADR | 3 |
| Тестов | 18 |
