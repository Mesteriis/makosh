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

- Chunk ID / ID чанка: `011-doc-AGENTS`
- Group / Группа: `AGENTS`
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

### `AGENTS.md`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/AGENTS.md`
- Size bytes / Размер в байтах: `17411`
- Included characters / Включено символов: `12000`
- Truncated / Обрезано: `yes`

````markdown
# AGENTS.md

Правила работы агентов в репозитории Макошь.

Эти правила обязательны для любых изменений в проекте. Репозиторий проектируется как долгосрочная local-first Personal Memory System, а не как MVP, CRM, почтовый клиент, task tracker, calendar app или note-taking app.

## 1. Роль агента

Агент работает как Principal Software Engineer / Software Architect.

Ожидаемое поведение:

- проверять факты по файлам репозитория;
- не выдумывать API, команды, зависимости, схемы и результаты тестов;
- делать минимальное корректное изменение;
- уважать архитектурные границы;
- явно сообщать, что было проверено;
- не писать реализацию, если задача относится к текущей documentation-first фазе.

## 2. Источники истины

При конфликте использовать такой порядок:

1. Текущий запрос пользователя.
2. Этот `AGENTS.md`.
3. ADR в `docs/adr/`.
4. Каноническая продуктовая и foundation-документация:
   `docs/product/master-spec.md`, `docs/foundation/`, `docs/domains/`,
   `docs/engines/`, `docs/workflows/`.
5. Архитектурная документация в `docs/architecture/`, `docs/ai/agents/`, `docs/ui/`.
6. Текущие файлы реализации как источник фактов о том, что уже реально
   реализовано.
7. Внешняя документация, только если она нужна и проверена.

Если запрос конфликтует с действующим ADR, нельзя тихо нарушать ADR. Нужно явно указать конфликт и сначала предложить новый ADR, который supersede или уточнит прежнее решение.

## 3. Жесткое следование ADR

Перед любым нетривиальным изменением агент обязан:

- прочитать релевантные ADR;
- назвать в плане, какие ADR влияют на работу;
- не вносить изменение, которое нарушает ADR;
- создать новый ADR до реализации, если появляется долгосрочное архитектурное решение;
- пометить старый ADR как `Superseded`, если решение заменяется.

Ключевые текущие ADR:

- `ADR-0001` - event sourcing is system spine.
- `ADR-0002` - Rust backend.
- `ADR-0004` - Tauri desktop shell.
- `ADR-0093` - Vue 3 frontend (supersedes ADR-0003).
- `ADR-0005` - PostgreSQL primary store.
- `ADR-0006` - Tantivy full text search.
- `ADR-0008` - knowledge graph first.
- `ADR-0009` - local AI through Ollama.
- `ADR-0022` - no fine-tuning on private data.
- `ADR-0026` - desktop-first responsive UI.
- `ADR-0031` - temporary desktop-only UI scope; no mobile UI design, implementation or validation until superseded.
- `ADR-0032` - Docker Compose development environment under `docker/`.
- `ADR-0041` - email provider ingestion foundation for Gmail, iCloud and generic IMAP.
- `ADR-0042` - provider credential secret references and resolver boundary.
- `ADR-0043` - Superseded by ADR-0055. Historical temporary provider-networking restriction.
- `ADR-0046` - persistent dev mail cache and blob storage; mail bytes/attachments live under `docker/data/mail/`, PostgreSQL stores metadata, references and attachment scan state.
- `ADR-0054` - application settings store; user-editable runtime/UI settings live in `application_settings`, while provider accounts remain domain records.
- `ADR-0055` - full email provider networking with read and write operations; supersedes ADR-0043. Read-only restriction retained only for automated integration tests.
- `ADR-0056` - local API simplified auth with router-level `X-Макошь-Secret`.
- `ADR-0076` - host vault on macOS; supersedes ADR-0044 and ADR-0053.
- `ADR-0077` - i18n with Russian and English interface via JSON dictionaries and Svelte stores; English strings serve as translation keys, ru.json provides Russian translations.
- `ADR-0084` - Persona Intelligence System; supersedes Contact/Person CRM framing.
- `ADR-0085` - Communication spine and Consistency / Contradiction Engine.
- `ADR-0086` - first-class Relationship persistence.
- `ADR-0087` - contradiction observation persistence.
- `ADR-0088` - obligation persistence.
- `ADR-0089` - decision persistence.

Superseded ADRs remain historical traceability records. Do not apply their
obsolete token, actor, Contact or database-vault requirements as current rules
when a newer ADR supersedes them.

## 3.1 Canonical Product Model

Макошь is a Personal Memory System for:

- Communications;
- Knowledge;
- Memory;
- Relationships;
- Projects;
- Documents;
- Decisions;
- Obligations;
- Context.

The central product value is context, not CRUD.

Communication is the primary ingestion spine:

```text
Communication -> Source Evidence -> Extracted Knowledge -> Memory -> Context
```

Tasks, decisions, projects, obligations, dossiers, timelines and search results
are built from evidence, events, graph links and reviewed memory.

Do not describe Макошь as:

- Email Client;
- CRM;
- Address Book;
- Contact Manager;
- Task Tracker;
- Calendar App;
- Note Taking App;
- generic Knowledge Base.

People are Personas, not contacts.

Required Persona concepts:

- one Owner Persona with `is_self = true`;
- `PersonaType`: `human`, `ai_agent`, `organization_proxy`, `system`;
- Identity, Relationships, Communication, Memory, Timeline, Dossier and Context;
- first-class Relationship semantics with source persona, target persona,
  relationship type, trust score and strength score.

Current implementation may still use compatibility names such as `persons`,
`person_id`, historical `contacts`, `health`, `watchtower` or `follow-up`.
Treat those as implementation compatibility labels. When docs and code differ,
record the gap in `docs/refactoring/implementation-alignment-plan.md` or a
follow-up plan before renaming code, routes or schemas.

Domains own durable entities. Engines are reusable mechanisms that produce
derived views, scores, candidates, observations and context. Do not duplicate
engine ownership inside a domain. Shared engines include Memory, Timeline,
Trust, Search, Enrichment, Obligation, Risk and Consistency / Contradiction
(Polygraph).

Polygraph is the user-facing alias for the Consistency / Contradiction Engine.
It detects evidence-backed contradictions and creates reviewable observations.
It must not automatically overwrite memory or label a person as dishonest.

## 4. Implementation Phase

The project has entered implementation with the Rust backend foundation. Agents may add scoped implementation code when it follows the current request, relevant ADR and existing architecture.

Allowed:

- documentation;
- ADR;
- architecture diagrams;
- roadmap and research notes;
- repository hygiene files;
- development infrastructure;
- tests and validation tooling;
- scoped backend/frontend/Tauri implementation that is covered by relevant validation.

Disallowed without an explicit user request and relevant ADR review:

- broad rewrites;
- generated app scaffolds that ignore repository conventions;
- database migrations;
- provider adapters;
- AI agent runtime code;
- domain model expansion;
- fake stub modules.

For implementation work, prefer TDD: write the failing test first, verify the failure, implement the smallest passing code, then run the configured validation.

After meaningful repository changes, run the relevant validation. Do not create
a git commit unless the user explicitly asks for a commit.

## 5. Required Workflow

For non-trivial work:

1. Inspect `git status --short`.
2. Inspect relevant files before editing.
3. Recall project memory if AgentMemory tools are available.
4. Read relevant ADR and architecture docs.
5. State a concise plan.
6. Edit only scoped files.
7. Validate with available commands.
8. Report changed files, validation and risks.

For trivial documentation edits, this workflow may be compressed, but validation must still be truthful.

## 6. Git Rules

- The repository must remain an active Git repository.
- Run `git status --short` before and after meaningful changes.
- Do not run destructive Git commands such as `git reset --hard` or `git checkout --` unless explicitly requested.
- Do not revert user changes.
- Do not commit unless the user explicitly asks for a commit.
- Do not mix unrelated changes into one task.
- Keep `.gitignore` aligned with the actual stack.

## 7. Linting and Validation Policy

Never claim validation passed unless the command was actually run.

Always discover the real toolchain from repository files before running stack-specific commands.

### Repository foundation validation

Use these checks while the repository is still in documentation/dev-infrastructure foundation mode:

```sh
find docs/adr -maxdepth 1 -type f -name 'ADR-*.md' | wc -l
find docs -type f -name '*.md' | wc -l
find . -path ./.git -prune -o -type f \
  ! -name '*.md' \
  ! -name '.gitignore' \
  ! -name 'Makefile' \
  ! -name 'Dockerfile' \
  ! -name '.env.example' \
  -print
```

If Markdown lint tooling is added later, run the configured command. Do not invent a markdown linter command when no config or dependency exists.

### Rust validation

When Rust implementation exists, prefer the configured commands. If no project-specific command exists, the default validation set is:

```sh
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all
```

For the current backend crate, use the repository Makefile targets:

```sh
make validate
make backend-fmt-check
make backend-clippy
make backend-test
make backend-validate
```

Run `make validate` before reporting broad backend or development-infrastructure work as complete. Use `make backend-validate` for targeted backend-only changes.
For full backend validation, prefer `make backend-test` or `make backend-validate` over direct `cargo test`. These targets run the `crates/testkit` session harness (`makosh_test_session`) that reuses the shared testcontainers PostgreSQL session correctly and cleans it up after the run. Direct full-suite `cargo test` bypasses that harness, can create excessive testcontainers churn, and may leave Docker container garbage behind after failures or interrupted runs.

### SvelteKit / TypeScript validation

Detect the package manager from lockfiles:

- `pnpm-lock.yaml` -> use `pnpm`
- `yarn.lock` -> use `yarn`
- `package-lock.json` -> use `npm`

Do not mix package managers.

Run configured scripts such as:

```sh
<package-manager> lint
<package-manager> check
<package-manager> test
<package-manager> build
```

Only run scripts that exist in `package.json`.

### Tauri validation

When Tauri exists, validate both sides:

- frontend checks from `package.json`;
- Rust checks in the Tauri crate;
- Tauri build/check command only if configured and practical for the task.

### Infrastructure and config validation

When relevant tooling exists, validate:

- SQL migrations;
- YAML;
- TOML;
- Docker/Compose files;
- shell scripts;
- OpenTelemetry config.

Use the repository-configured tool first. If no tool exists, report that validation was not available instead of fabricating coverage.

## 8. Architecture Constraints

- Local-first is mandatory.
- Knowledge graph and events are primary architecture concepts.
- Search indexes and embeddings are derived, rebuildable state.
- AI output is never source of truth.
- Private data must not be used for fine-tuning.
- Provider adapters must preserve raw source provenance. Full read-write provider networking is enabled per ADR-0055; read-only restriction applies only to automated tests.
- Agents and plugins must use capability-based permissions.
- Mobile UI is out of scope until `ADR-0031` is superseded.
- Docker development infrastructure must stay under `docker/` per `ADR-0032`.
- Protected local API endpoints must use the router-level shared secret guard
  from `ADR-0056`: `MAKOSH_LOCAL_API_SECRET` plus the `X-Макошь-Secret`
  request header.
- Local event API access must be recorded in append-only `api_audit_log` per `ADR-0039`; do not store tokens or secrets in audit records.
- API audit actor identity is the constant `makosh-frontend` unless a newer ADR
  changes the local actor model. Do not require `X-Макошь-Actor-Id`.
- Email ingestion provider accounts must support `gmail`, `icloud` and `imap` per `ADR-0041`; account config must not store OAuth tokens, app passwords or mailbox
````
_Source file truncated after 12000 characters. / Исходный файл обрезан после 12000 символов._
