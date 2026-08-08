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

- Chunk ID / ID чанка: `009-doc-supergoal-part-002`
- Group / Группа: `.supergoal`
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

### `.supergoal/makosh-frontend-migration-vue-3-WzENWm/phases/phase-10.md`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/.supergoal/makosh-frontend-migration-vue-3-WzENWm/phases/phase-10.md`
- Size bytes / Размер в байтах: `2196`
- Included characters / Включено символов: `2184`
- Truncated / Обрезано: `no`

```markdown
SUPERGOAL_PHASE_START
Phase: 10 of 15 — Knowledge & Review
Task: Port Knowledge graph and Review (polygraph) domain pages to Vue
Mandatory commands: cd frontend && pnpm build
Acceptance criteria: 6
Evidence required: build output, knowledge and review domain file listings
Depends on phases: 1, 2, 3

## Why

Knowledge graph (using Vue Flow) and Review (polygraph/contradictions) are intelligence-focused domains. Knowledge graph validates Vue Flow integration for graph visualization.

## Work

1. **Create knowledge domain** under `frontend/src/domains/knowledge/`:
   - Types, API (graph, contradictions), queries (useGraphQuery, useContradictionsQuery)
   - Stores (Pinia for UI state: selected node, active tab)
   - Components:
     - KnowledgeGraphCanvas.vue — Vue Flow graph visualization
     - KnowledgeNodeInspector.vue — node detail panel
     - KnowledgePolygraphReview.vue — contradiction observations list
   - Views/KnowledgePage.vue — main page
   - Routes

2. **Create review domain** under `frontend/src/domains/review/`:
   - Types, API, queries (useObligationsQuery, useDecisionsQuery)
   - Components: ReviewObligations, ReviewDecisions
   - Views/ReviewPage.vue — main page
   - Routes

3. **Register routes** for `/knowledge` and `/review`

4. **Verify:**
   - Build passes
   - Knowledge graph canvas renders with Vue Flow
   - Review page lists obligations and decisions

## Acceptance criteria

- [ ] AC1: Knowledge graph canvas renders using Vue Flow
- [ ] AC2: Node inspector shows selected node details
- [ ] AC3: Polygraph review lists contradiction observations from API
- [ ] AC4: Review page lists obligations and decisions with status
- [ ] AC5: Graph nodes/edges render with correct styling
- [ ] AC6: `cd frontend && pnpm build` exits 0

## Mandatory commands

- `cd frontend && pnpm build`

## Evidence required in transcript

- Build output

## Notes

- Reference `frontend/src-svelte/lib/pages/knowledge/` and `frontend/src-svelte/lib/pages/review/`
- Vue Flow replaces the Svelte-based graph implementation
- Polygraph is the user-facing name for the Consistency/Contradiction Engine per ADR-0085
- Keep graph canvas component under 500 lines
```

### `.supergoal/makosh-frontend-migration-vue-3-WzENWm/phases/phase-11.md`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/.supergoal/makosh-frontend-migration-vue-3-WzENWm/phases/phase-11.md`
- Size bytes / Размер в байтах: `1676`
- Included characters / Включено символов: `1674`
- Truncated / Обрезано: `no`

```markdown
SUPERGOAL_PHASE_START
Phase: 11 of 15 — Agents & Timeline
Task: Port Agents and Timeline domain pages to Vue
Mandatory commands: cd frontend && pnpm build
Acceptance criteria: 5
Evidence required: build output, agents and timeline domain file listings
Depends on phases: 1, 2, 3

## Why

Agents and Timeline are the remaining smaller domain views. Agents shows AI runtime status; Timeline shows activity stream. Validates TanStack Virtual for timeline items.

## Work

1. **Create agents domain** under `frontend/src/domains/agents/`:
   - Types, API, queries, stores (Pinia for UI state)
   - Components: AgentsDetail, AgentsGrid, AgentsRail, AgentsRuntimeMetrics, AgentsWorkflows
   - Views/AgentsPage.vue
   - Routes

2. **Create timeline domain** under `frontend/src/domains/timeline/`:
   - Types, API, queries, stores
   - Components: TimelineStream (with TanStack Virtual), TimelineFilters
   - Views/TimelinePage.vue
   - Routes

3. **Register routes** for `/agents` and `/timeline`

4. **Verify:**
   - Build passes
   - Agents page renders
   - Timeline renders with virtual scrolling

## Acceptance criteria

- [ ] AC1: Agents page renders with grid, detail, rail, metrics, workflows
- [ ] AC2: Timeline page renders with stream and filters
- [ ] AC3: Timeline items use TanStack Virtual for virtualization
- [ ] AC4: Timeline filters affect displayed items
- [ ] AC5: `cd frontend && pnpm build` exits 0

## Mandatory commands

- `cd frontend && pnpm build`

## Evidence required in transcript

- Build output

## Notes

- Reference `frontend/src-svelte/lib/pages/agents/` and `frontend/src-svelte/lib/pages/timeline/`
- Use TanStack Virtual for timeline stream
```

### `.supergoal/makosh-frontend-migration-vue-3-WzENWm/phases/phase-12.md`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/.supergoal/makosh-frontend-migration-vue-3-WzENWm/phases/phase-12.md`
- Size bytes / Размер в байтах: `4693`
- Included characters / Включено символов: `4631`
- Truncated / Обрезано: `no`

```markdown
SUPERGOAL_PHASE_START
Phase: 12 of 15 — Communications/Mail
Task: Port the Communications (mail) domain — the most complex domain — to Vue
Mandatory commands: cd frontend && pnpm build
Acceptance criteria: 9
Evidence required: build output, communications domain file listing
Depends on phases: 1, 2, 3

## Why

Communications (mail) is the most complex domain with mail list, message viewer with tabs, compose drawer, draft strip, health strip, account wizard, context inspector, and conversation list. It's the core ingestion spine of Макошь.

## Work

1. **Create communications domain** under `frontend/src/domains/communications/`:
   - `types/` — Communication types (ComposeFormModel, MailAccountOption, CommunicationListMessage, RenderedMessageContent, etc.)
   - `api/` — API functions (loadMailList, loadMessage, sendMessage, saveDraft, deleteDraft, loadConversations, loadContext, etc.)
   - `queries/` — TanStack Query hooks:
     - `useMailListQuery(accountId, folder)` — mail list with pagination
     - `useMessageQuery(messageId)` — single message detail
     - `useConversationsQuery(threadId)` — conversation thread
     - `useDraftsQuery()` — draft list
     - `useMailboxHealthQuery()` — mailbox health status
     - `useAccountOptionsQuery()` — mail account options
     - Mutations: useSendMailMutation, useSaveDraftMutation, useDeleteDraftMutation
   - `stores/` — Pinia stores for UI state only:
     - Selected communication, compose form state, drawer open state, active tab, send review state
   - `components/`:
     - MailList.vue — virtualized mail list (TanStack Virtual + TanStack Table)
     - MailListItem.vue — single mail row
     - MailViewer.vue — message detail with tabs
     - MessageBodyTab.vue — rendered HTML body in sandboxed iframe
     - MessageHeadersTab.vue — headers display
     - MessageAttachmentsTab.vue — attachment list
     - MessageRelatedTab.vue — related messages
     - MessageTimelineTab.vue — timeline for this message
     - CommunicationsContextInspector.vue — context analysis panel
     - CommunicationsContextRail.vue — context sidebar
     - CommunicationsConversationList.vue — conversation thread
     - ComposeDrawer.vue — compose/reply/forward drawer with TipTap editor
     - DraftStrip.vue — draft management strip
     - HealthStrip.vue — mailbox health indicator
     - AccountSetupModal.vue — account wizard modal
   - `views/`:
     - CommunicationsPage.vue — main page with widget layout
     - CommunicationsEmptyPage.vue — empty section placeholder
   - `routes/`

2. **Port compose functionality:**
   - Study existing compose in `frontend/src-svelte/lib/services/communications/compose.ts`
   - TipTap editor for rich text body
   - Support compose/reply/forward modes
   - Draft auto-save and management
   - Send with review step

3. **Port message rendering:**
   - Study `frontend/src-svelte/lib/services/communications/rendering.ts`
   - Render HTML email bodies in sandboxed iframe
   - Support text/plain fallback

4. **Register route** for `/communications`

5. **Verify:**
   - Build passes
   - Mail list loads with real data
   - Message viewer renders HTML content
   - Compose drawer opens and can save drafts
   - Draft strip shows managed drafts
   - Health strip shows mailbox status

## Acceptance criteria

- [ ] AC1: Mail list renders with virtual scrolling from API data
- [ ] AC2: Message viewer renders HTML content in sandboxed iframe
- [ ] AC3: Compose drawer supports compose/reply/forward modes with TipTap editor
- [ ] AC4: Draft strip shows and manages drafts (save, delete, open for editing)
- [ ] AC5: Account wizard modal renders provider selection flow
- [ ] AC6: Health strip shows mailbox health status from API
- [ ] AC7: Conversation list renders thread messages
- [ ] AC8: Context inspector shows AI analysis of selected message
- [ ] AC9: `cd frontend && pnpm build` exits 0

## Mandatory commands

- `cd frontend && pnpm build`

## Evidence required in transcript

- Build output
- List of communications domain files

## Notes

- Reference `frontend/src-svelte/lib/pages/communications/` and `frontend/src-svelte/lib/services/communications/`
- This is the MOST COMPLEX domain — expect god-file risks. Enforce 500-line limit
- Decompose if any file approaches 500 lines
- Mail list virtualization is critical for performance (TanStack Virtual)
- TipTap is used for the compose editor
- HTML email rendering must use sandboxed iframe (srcdoc approach)
- Server state goes through TanStack Query, UI state through Pinia
- Compose form state is UI state (Pinia), email send is server mutation (TanStack Query)
```

### `.supergoal/makosh-frontend-migration-vue-3-WzENWm/phases/phase-13.md`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/.supergoal/makosh-frontend-migration-vue-3-WzENWm/phases/phase-13.md`
- Size bytes / Размер в байтах: `2837`
- Included characters / Включено символов: `2813`
- Truncated / Обрезано: `no`

```markdown
SUPERGOAL_PHASE_START
Phase: 13 of 15 — Telegram & WhatsApp
Task: Port Telegram and WhatsApp messaging domain pages to Vue
Mandatory commands: cd frontend && pnpm build
Acceptance criteria: 7
Evidence required: build output, telegram and whatsapp domain file listings
Depends on phases: 1, 2, 3

## Why

Telegram and WhatsApp are messaging sub-sections of the Communications view. They share message thread UI patterns and validate real-time-first approach for chat interfaces.

## Work

1. **Create telegram domain** under `frontend/src/domains/telegram/`:
   - Types, API, queries (TanStack Query hooks for chat list, messages, status)
   - Stores (Pinia for UI state: selected chat, active thread)
   - Components:
     - TelegramChatList.vue — virtualized chat list (TanStack Virtual)
     - TelegramMessageThread.vue — message thread display
     - TelegramRail.vue — chat details rail
     - TelegramCommandHeader.vue — command input header
     - TelegramActionRail.vue — action buttons rail
     - TelegramStatusMessages.vue — system status messages
   - Views/TelegramPage.vue — main page
   - Routes

2. **Create whatsapp domain** under `frontend/src/domains/whatsapp/`:
   - Types, API, queries, stores
   - Components:
     - WhatsAppSessionList.vue — session list with virtualization
     - WhatsAppMessageThread.vue — message thread
     - WhatsAppRail.vue — session details rail
   - Views/WhatsAppPage.vue — main page
   - Routes

3. **Update communications navigation:**
   - The Communications route handles sub-navigation for mail/telegram/whatsapp sections
   - These domains render as sub-views of the communications workspace

4. **Register routes** for `/communications/telegram` and `/communications/whatsapp`

5. **Verify:**
   - Build passes
   - Telegram chat list renders with virtual scrolling
   - WhatsApp session list renders

## Acceptance criteria

- [ ] AC1: Telegram chat list renders with virtual scrolling
- [ ] AC2: Telegram message thread renders messages with correct formatting
- [ ] AC3: Telegram rail shows chat details and metadata
- [ ] AC4: WhatsApp session list renders with virtualization
- [ ] AC5: WhatsApp message thread renders messages
- [ ] AC6: Both domains load data from API via TanStack Query
- [ ] AC7: `cd frontend && pnpm build` exits 0

## Mandatory commands

- `cd frontend && pnpm build`

## Evidence required in transcript

- Build output

## Notes

- Reference `frontend/src-svelte/lib/pages/telegram/` and `frontend/src-svelte/lib/pages/whatsapp/`
- Use TanStack Virtual for chat/session list virtualization
- These are communication sub-sections accessed via the Communications view
- Telegram has more complex UI (chat list + thread + rail + command header + action rail + status messages)
- WhatsApp is simpler (session list + thread + rail)
```

### `.supergoal/makosh-frontend-migration-vue-3-WzENWm/phases/phase-14.md`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/.supergoal/makosh-frontend-migration-vue-3-WzENWm/phases/phase-14.md`
- Size bytes / Размер в байтах: `7831`
- Included characters / Включено символов: `7787`
- Truncated / Обрезано: `no`

```markdown
SUPERGOAL_PHASE_START
Phase: 14 of 15 — Polish & Harden
Task: Catch what earlier phases missed — UX copy, states, edges, security, a11y, perf, animations, and regression sweep
Mandatory commands: cd frontend && pnpm build, cd frontend && pnpm lint (if configured)
Acceptance criteria: 9
Evidence required: one paragraph per sub-pass, bundle size analysis, final screenshots
Depends on phases: 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13

## Why

Earlier phases were focused on shipping behavior — getting each domain ported and rendering. This phase enforces that every aspect is production-quality: UX copy, empty/loading/error/unauthorized states, edge cases, security, accessibility, performance, animations, and a full regression sweep against the existing Макошь SvelteKit app.

## Work

### Sub-pass 1: UX & Copy

- Audit every visible string across all 14 ported domains
- Remove any debug placeholders, TODO stubs, or lorem-ipsum content
- Verify all strings use the i18n system (`useI18n().t()`) — no hardcoded Russian or English strings
- Check that error messages are user-friendly, not raw API errors or stack traces
- Verify button labels, tooltips, aria-labels, empty-state messages, and toast notifications
- Run: `grep -r "TODO\|FIXME\|HACK\|XXX\|debug\|placeholder\|lorem\|stub" frontend/src/ --include="*.vue" --include="*.ts"` — review each match

### Sub-pass 2: States

- For EVERY domain surface (14 domains), verify these states:
  - **Loading**: Skeleton or spinner shown during TanStack Query `isPending`
  - **Empty**: Meaningful empty state message with icon when data array is empty
  - **Error**: Error state with retry button when TanStack Query `isError`
  - **Unauthorized**: Proper handling when API returns 401/403 (redirect to auth or show message)
- Create or update a `frontend/src/domains/<domain>/components/States.vue` pattern if needed
- Verify optimistic updates in TanStack Query mutations handle rollback gracefully

### Sub-pass 3: Edges

- Test with:
  - Empty inputs (empty strings, zero-length arrays)
  - Very long inputs (1000+ character strings, long names)
  - Special characters (Unicode, emoji, HTML injection attempts, SQL-like patterns)
  - Slow network (simulate via browser DevTools throttling)
  - Rapid repeated clicks (debounce/throttle on save/submit buttons)
- Verify forms disable submit button while mutation is pending
- Verify file uploads handle cancellation, large files, and wrong file types

### Sub-pass 4: Security

- Verify `X-Макошь-Secret` header is present in ALL API calls (not just get/post — every method)
- Check that no secrets, tokens, or private data appear in:
  - Compiled bundle (`grep -r "password\|secret\|token\|api_key" frontend/dist/ --include="*.js"` after build)
  - Console.log statements in production
  - Error messages exposed to user
- Verify input validation/sanitization on all user-input forms
- Check that sandboxed iframe for message viewer (Communications) has proper `sandbox` attribute
- Verify CSP is not wide-open (check tauri.conf.json and any meta tags)

### Sub-pass 5: A11y

- Keyboard navigation:
  - Tab through all interactive elements — focus order must be logical
  - All dialogs trap focus while open
  - Escape closes dialogs, drawers, popovers
  - Enter/Space activates buttons and links
- Focus management:
  - Focus returns to trigger element after dialog/drawer closes
  - Route changes focus to main content area or h1
- Screen reader:
  - All images have `alt` text or `aria-hidden="true"`
  - All interactive elements have accessible names
  - ARIA live regions for dynamic content updates (toast notifications, SSE updates)
  - Proper heading hierarchy (h1 → h2 → h3, no skipping)
- Contrast: verify all text/background combinations meet WCAG AA (4.5:1 for normal text, 3:1 for large)

### Sub-pass 6: Performance

- **Virtual scrolling**: Verify TanStack Virtual is used for ALL large collections:
  - Communications mail list
  - Telegram chat list, WhatsApp session list
  - Documents list, Notes list
  - Tasks list
  - Timeline stream
  - Personas list
  - Projects list
- **Bundle size analysis**: Run `cd frontend && pnpm build && du -sh dist/` and `ls -lh dist/assets/`
- **No N+1 queries**: Verify TanStack Query patterns — no queries inside loops
- **Lazy loading**: Route-level code splitting via Vue Router dynamic imports
- **Image optimization**: Verify images are properly sized and lazy-loaded

### Sub-pass 7: Diff Review

- Search for and clean up:
  - Stray `console.log()` statements (use `console.debug()` or a logger utility instead)
  - `TODO` comments left from migration phases
  - Unused imports (run `eslint --rule 'unused-imports/no-unused-imports: error'` or similar)
  - Dead code/commented-out blocks
  - Duplicate type definitions
- Run `cd frontend && pnpm build` to confirm no warnings

### Sub-pass 8: Regression Sweep

- Full build: `cd frontend && pnpm build` — confirm exits 0, no warnings
- Visual comparison: Open both the new Vue app and the existing SvelteKit app (from `frontend/src-svelte/` if still runnable, or reference screenshots)
- Compare key surfaces side-by-side:
  - Home dashboard layout and widget rendering
  - Settings panel appearance
  - Mail list and message viewer
  - Persona detail view
  - Knowledge graph
- Verify no visual regressions: colors, spacing, typography, shadows, border radii, transitions

### Sub-pass 9: Animation

- Verify workspace transitions (route changes) animate smoothly via Motion library
- Panel animations (sidebar open/close, drawer slide-in) are smooth
- Micro-interactions:
  - Button hover/active states
  - Checkbox/switch toggle animations
  - List item hover effects
  - Dialog open/close transitions
- Verify animations respect `prefers-reduced-motion` — disable animations when user prefers reduced motion
- Ensure animations are not too slow (max 300ms for UI transitions)

## Acceptance criteria

- [ ] AC1: UX audit complete — no debug placeholders, all strings through i18n
- [ ] AC2: All 14 domains verified for loading/empty/error/unauthorized states
- [ ] AC3: Edge case testing completed — empty inputs, long inputs, special chars, slow network
- [ ] AC4: Security audit passed — X-Макошь-Secret in all calls, no secrets in bundle
- [ ] AC5: A11y audit passed — keyboard nav, focus management, headings, contrast ≥ AA
- [ ] AC6: Performance verified — virtual scrolling on all large collections, bundle size acceptable
- [ ] AC7: Diff review clean — no console.log, no TODOs, no unused imports
- [ ] AC8: Regression sweep passed — visual comparison confirms no regressions
- [ ] AC9: Animation pass — transitions smooth, respects prefers-reduced-motion

## Mandatory commands

- `cd frontend && pnpm build`
- `grep -r "TODO\|FIXME\|HACK\|XXX\|console\.log" frontend/src/ --include="*.vue" --include="*.ts"` (review output, clean up)
- `cd frontend && du -sh dist/` and `ls -lh dist/assets/` (bundle size check)

## Evidence required in transcript

- One paragraph per sub-pass — what was checked, what was found, what was fixed
- Bundle size analysis output
- Screenshots of 3 key surfaces: Home, Settings, Communications (or relevant domain)
- List of any remaining known issues or deferred work

## Notes

- This phase is intentionally manual-intensive. Each sub-pass requires human judgment, not just automated checks.
- If any sub-pass reveals systemic issues (e.g., all domains missing empty states), create a shared component fix rather than fixing each domain individually.
- The regression sweep should use the same test data/account for both old and new apps.
- Document any visual differences found and whether they are acceptable regressions or improvements.
- Proceed to Phase 15 (Cutover) only when all 9 acceptance criteria are satisfied.
```

### `.supergoal/makosh-frontend-migration-vue-3-WzENWm/phases/phase-15.md`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/.supergoal/makosh-frontend-migration-vue-3-WzENWm/phases/phase-15.md`
- Size bytes / Размер в байтах: `6442`
- Included characters / Включено символов: `6394`
- Truncated / Обрезано: `no`

````markdown
SUPERGOAL_PHASE_START
Phase: 15 of 15 — Cutover
Task: Remove all SvelteKit code, update Tauri configuration, update Makefile/CI, and run full validation gate
Mandatory commands: cd frontend && pnpm build, cd frontend && pnpm lint, cd frontend && pnpm test:unit, make validate
Acceptance criteria: 8
Evidence required: build output, test output, file listing confirming no .svelte files, package.json confirming no svelte deps, tauri.conf.json
Depends on phases: 14

## Why

Final phase. All 14 domains are ported, polished, and hardened. Now remove all SvelteKit artifacts, update Tauri config to point to the Vue build, update Makefile/CI targets, and run the full validation gate to confirm the migration is complete.

## Work

### 1. Remove SvelteKit code

- Delete `frontend/src-svelte/` directory entirely (the backup of old SvelteKit code)
- Delete `frontend/static-svelte/` if it exists
- Delete `frontend/svelte.config.js`
- Delete `frontend/.svelte-kit/` if present (generated by SvelteKit)
- Remove SvelteKit-related files from `frontend/.gitignore`
- Run `find frontend/ -name "*.svelte" -not -path "*/node_modules/*"` to confirm zero remaining
- Remove any Svelte-related references in CI/CD config (`.github/workflows/`)

### 2. Remove SvelteKit dependencies from package.json

- Remove from `frontend/package.json`:
  - `@sveltejs/kit`
  - `@sveltejs/adapter-static`
  - `svelte`
  - `@sveltejs/vite-plugin-svelte`
  - `svelte-check`
  - `@iconify/svelte` (replaced by `@iconify/vue`)
  - Any other `@sveltejs/*` or `svelte-*` packages
- Update `package.json` scripts:
  - Replace `"dev": "vite dev"` → `"dev": "vite"` (Vue standard)
  - Replace `"build": "vite build"` (keep as is, already works)
  - Replace `"preview": "vite preview"` (keep as is)
  - Replace `"lint:ts": "svelte-check"` → `"lint:ts": "vue-tsc --noEmit"`
  - Add `"lint:eslint": "eslint src/ --ext .vue,.ts"` if eslint configured
  - Update `"lint"` to combine lint:ts + lint:eslint
  - Remove any svelte-check references
- Run `pnpm install` to clean up lockfile

### 3. Update Tauri configuration

- Verify `frontend/src-tauri/tauri.conf.json`:
  - `build.frontendDist` is `"../build"` (or `"../dist"`) — the Vue build output directory
  - `build.beforeDevCommand` is `"pnpm dev"` (Vue standard)
  - `build.beforeBuildCommand` is `"pnpm build"` (Vue standard)
  - Remove any SvelteKit-specific `devUrl` overrides if present
- Update `frontend/src-tauri/Cargo.toml` if any Tauri plugin dependencies need updating
- Run `cd frontend/src-tauri && cargo check` to verify Tauri Rust side compiles

### 4. Update configuration files

- **vite.config.ts**: Remove any remaining SvelteKit adapter references. Ensure Vue plugin is the primary plugin.
- **tsconfig.json**: Remove any `extends` paths referencing `.svelte-kit/`. Add Vue-specific compiler options:
  ```json
  "compilerOptions": {
    "jsx": "preserve",
    "types": ["vite/client"]
  }
  ```
- **frontend/README.md**: Update to reflect Vue 3 project, not SvelteKit
- **frontend/.gitignore**: Clean up — remove svelte-kit entries, keep Vue/dist entries

### 5. Update Makefile

- Update root `Makefile` frontend targets:
  - `frontend-lint` — no longer calls svelte-check, uses vue-tsc + eslint
  - `frontend-build` — should point to `cd frontend && pnpm build`
  - `frontend-test` — runs `cd frontend && pnpm test:unit`
  - `frontend-check` — runs lint + build + test
- Update `make validate` to include `make frontend-check` or equivalent
- Update any CI-related targets

### 6. Update CI/CD

- Update `.github/workflows/` files:
  - Replace `svelte-check` with `vue-tsc --noEmit`
  - Update any SvelteKit-specific actions or caches
  - Verify pnpm cache keys are still valid

### 7. Full validation gate

Run the following in sequence and confirm each exits 0:

1. `cd frontend && pnpm install` — clean install
2. `cd frontend && pnpm build` — Vue build
3. `cd frontend && pnpm lint` — type check + eslint
4. `cd frontend && pnpm test:unit` — unit tests
5. `cd frontend && npx vue-tsc --noEmit` — explicit type check
6. `cd frontend/src-tauri && cargo check` — Tauri Rust side
7. `make validate` — repo root full validation gate

### 8. Final verification

- `find frontend/ -name "*.svelte" -not -path "*/node_modules/*"` — must return empty
- `jq '.devDependencies + .dependencies' frontend/package.json | grep -E "svelte|@sveltejs"` — must return empty
- Confirm `frontend/src-svelte/` no longer exists
- Confirm frontend build output exists and is served by Tauri

## Acceptance criteria

- [ ] AC1: No `.svelte` files exist anywhere in `frontend/` (excluding node_modules)
- [ ] AC2: No `svelte` or `@sveltejs/*` dependencies in `frontend/package.json`
- [ ] AC3: `cd frontend && pnpm build` exits 0
- [ ] AC4: `cd frontend && pnpm lint` passes (vue-tsc + eslint)
- [ ] AC5: `cd frontend && pnpm test:unit` passes
- [ ] AC6: `frontend/src-tauri/tauri.conf.json` `frontendDist` points to correct Vue build output
- [ ] AC7: Full `make validate` passes from repo root
- [ ] AC8: `frontend/src-svelte/` directory no longer exists

## Mandatory commands

- `cd frontend && pnpm build`
- `cd frontend && pnpm lint`
- `cd frontend && pnpm test:unit`
- `find frontend/ -name "*.svelte" -not -path "*/node_modules/*"`
- `jq '.devDependencies + .dependencies' frontend/package.json | grep -E "svelte|@sveltejs"` (or equivalent grep)
- `make validate` (from repo root)

## Evidence required in transcript

- Build output — last 10 lines
- Test output — last 10 lines
- `find` command output confirming no .svelte files remain
- package.json snippet confirming no svelte dependencies
- tauri.conf.json snippet showing correct frontendDist
- `make validate` output — last 15 lines

## Notes

- This is the final phase. After this, the migration is COMPLETE.
- Do NOT skip the `make validate` step — it's the repo-wide gate.
- If `pnpm lint` fails due to new lint rules, fix the issues rather than lowering the bar.
- If `pnpm test:unit` fails, ensure the test suite is updated for Vue components — existing vitest tests from the SvelteKit era should still work for pure TypeScript logic, but component tests may need updating.
- If `cargo check` in the Tauri crate fails, it's likely a dependency version issue — run `cd frontend/src-tauri && cargo update` first.
- After this phase, the repository is fully migrated to Vue 3 + TypeScript + Vite + Tauri 2.
````

### `.supergoal/makosh-frontend-migration-vue-3-WzENWm/phases/phase-2.md`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/.supergoal/makosh-frontend-migration-vue-3-WzENWm/phases/phase-2.md`
- Size bytes / Размер в байтах: `5524`
- Included characters / Включено символов: `5508`
- Truncated / Обрезано: `no`

```markdown
SUPERGOAL_PHASE_START
Phase: 2 of 15 — App Shell
Task: Port sidebar, topbar, workspace layout, notifications drawer, layout editor to Vue
Mandatory commands: cd frontend && pnpm build
Acceptance criteria: 10
Evidence required: build output, route config, shell component listing
Depends on phases: 1

## Why

The app shell (sidebar, topbar, workspace layout) is the container for all domain views. Every domain renders inside it. Must be ported before any domain can render.

## Work

1. **Create Vue Router config with all route definitions:**
   - Create `frontend/src/app/router.ts`
   - Define routes for all 16 views: home, communications, persons, projects, tasks, calendar, documents, notes, knowledge, review, settings, agents, organizations, timeline, telegram, whatsapp
   - Each route maps to a placeholder component initially (actual domain components added in later phases)
   - Use Vue Router's history mode (hash mode for Tauri compatibility)

2. **Port Pinia stores from existing Svelte stores:**
   - Study existing Svelte stores in `frontend/src-svelte/lib/stores/`:
     - `navigation.ts` — activeView, activeCommunicationSection, isSidebarRail, expandedSidebarGroupIds, isUserMenuOpen, shellViewClass
     - `theme.ts` — shellThemeClass
     - `sidebar.ts` — sidebarRootEntries
     - `notifications.ts` — notificationItems, notificationCount, toggleNotificationsDrawer, openNotificationTarget
     - `layoutEditor.ts` — isLayoutEditing, activeWidgetById, visibleWidgetIds, layoutDraft, addableWidgetsForCurrentView, isWidgetDrawerOpen, selectedLayoutWidget, widgetGridValue, etc.
   - Create Pinia stores in `frontend/src/shared/stores/`:
     - `navigation.ts`, `theme.ts`, `sidebar.ts`, `notifications.ts`, `layoutEditor.ts`
   - Each store uses `defineStore` with Composition API
   - Server-derived state must NOT be in Pinia (only UI state)

3. **Port Sidebar component:**
   - Study `frontend/src-svelte/lib/components/shell/Sidebar.svelte`
   - Create `frontend/src/app/shell/Sidebar.vue`
   - Navigation groups with expand/collapse
   - Sidebar rail mode (compact/expanded toggle)
   - Active state highlighting
   - Settings button at bottom
   - Iconify icons for navigation items

4. **Port Topbar component:**
   - Study `frontend/src-svelte/lib/components/shell/Topbar.svelte`
   - Create `frontend/src/app/shell/Topbar.vue`
   - View title and subtitle display
   - Notification bell with count badge
   - User menu dropdown
   - Layout editing toggle button
   - Locale switch button
   - Exit button

5. **Port NotificationsDrawer component:**
   - Study existing notification drawer
   - Create `frontend/src/app/shell/NotificationsDrawer.vue`
   - Slide-in drawer with notification list
   - Click to navigate to notification target
   - Empty state

6. **Port LayoutEditor controls:**
   - Study `frontend/src-svelte/lib/components/shared/LayoutEditControls.svelte`
   - Create `frontend/src/app/shell/LayoutEditControls.vue`
   - Add widget, cancel, reset, save buttons
   - Widget settings drawer
   - Add widget drawer

7. **Create AppShell layout component:**
   - Create `frontend/src/app/shell/AppShell.vue`
   - Contains Sidebar (left), workspace area (center with Topbar at top), notifications drawer (overlay)
   - Handles sidebar rail mode and vault gate mode
   - Renders `<router-view>` in workspace area
   - Includes vault onboarding gate placeholder

8. **Update App.vue:**
   - Update `frontend/src/app/App.vue` to use AppShell wrapping `<router-view>`

9. **Port shell CSS:**
   - Study `frontend/src-svelte/lib/styles/shell.css`, `shellTheme.css`, `app.css`, `panels.css`
   - Create Tailwind-based shell styling that replicates the existing visual appearance
   - Use the Tailwind theme tokens from Phase 1

10. **Verify:**
    - `pnpm build` passes
    - Run `pnpm dev`, open browser, verify shell renders with sidebar, topbar, workspace
    - Test sidebar navigation between placeholder views
    - Test sidebar rail toggle
    - Test notification drawer open/close
    - Test layout editing mode toggle

## Acceptance criteria (all must pass)

- [ ] AC1: AppShell renders Sidebar, Topbar, and workspace router-view area
- [ ] AC2: Sidebar navigation switches between placeholder route views
- [ ] AC3: Topbar shows view title and notification count badge
- [ ] AC4: NotificationsDrawer opens/closes with slide animation
- [ ] AC5: User menu opens/closes
- [ ] AC6: Layout editing mode toggle works
- [ ] AC7: Sidebar rail mode (compact/expanded) toggle works
- [ ] AC8: All CSS/styling matches existing Макошь visual identity (compare with screenshots)
- [ ] AC9: `cd frontend && pnpm build` exits 0
- [ ] AC10: Vue Router config defines all 16 route paths

## Mandatory commands

- `cd frontend && pnpm build`

## Evidence required in transcript

- Build output last 10 lines
- Vue Router config showing all defined routes
- List of created Pinia store files
- List of shell component files

## Notes

- Reference the existing SvelteKit code in `frontend/src-svelte/` for exact component behavior
- Do NOT port vault onboarding or VaultOnboarding component in this phase — it will be ported later or kept as a placeholder
- Use `@iconify/vue` for all icons (replacing `@iconify/svelte`)
- Pinia stores must NOT contain server-derived data — only transient UI state
- Theme store should read from localStorage or backend settings (same as existing)
- The layout editor widget settings and add-widget drawers can be simplified stubs in this phase
```

### `.supergoal/makosh-frontend-migration-vue-3-WzENWm/phases/phase-3.md`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/.supergoal/makosh-frontend-migration-vue-3-WzENWm/phases/phase-3.md`
- Size bytes / Размер в байтах: `3161`
- Included characters / Включено символов: `3155`
- Truncated / Обрезано: `no`

```markdown
SUPERGOAL_PHASE_START
Phase: 3 of 15 — Shared UI Primitives
Task: Initialize shadcn-vue components and build Level 1 shared UI primitives
Mandatory commands: cd frontend && pnpm build
Acceptance criteria: 8
Evidence required: build output, file listing of shared/ui/
Depends on phases: 1

## Why

UI primitives (Button, Input, Dialog, Dropdown, etc.) are used by every domain. Porting them early ensures consistency, avoids duplication, and establishes the Макошь-styled component library.

## Work

1. **Initialize shadcn-vue:**
   - Run `npx shadcn-vue init` to set up the components system
   - Configure to use the Макошь Tailwind theme
   - Components go to `frontend/src/shared/ui/`

2. **Add shadcn-vue components:**
   - Add each component via `npx shadcn-vue add <component>` or by manually creating them
   - Required components: Button, Input, Dialog, DropdownMenu, Select, Switch, Tabs, Card, Badge, Avatar, Tooltip, Popover, Command (for palette), Sheet (for drawers), Separator, ScrollArea, Skeleton, Progress, Toast, Label, Textarea, Form

3. **Customize components for Макошь visual identity:**
   - Modify each component's styling to use Макошь theme tokens (colors, spacing, typography, border radius)
   - Ensure components match the existing Макошь visual style, NOT default shadcn look
   - Key customizations:
     - Button: match existing Макошь button states (default, hover, active, disabled)
     - Input: match existing input styling (background, border, focus ring)
     - Dialog: match existing modal/drawer styling
     - Tabs: match existing tab styling

4. **Create barrel export:**
   - Create `frontend/src/shared/ui/index.ts` that exports all components

5. **Add Iconify Vue integration:**
   - Ensure `@iconify/vue` is installed and an `<IconifyIcon>` component is available
   - Create `frontend/src/shared/ui/Icon.vue` — wrapper for consistent icon usage

6. **Verify:**
   - Build passes
   - Each component renders correctly in a test view

## Acceptance criteria (all must pass)

- [ ] AC1: All required shadcn-vue components exist in `frontend/src/shared/ui/`
- [ ] AC2: Components use Макошь theme tokens (colors, spacing, typography)
- [ ] AC3: Button supports all variants (default, secondary, outline, ghost, destructive)
- [ ] AC4: Dialog opens/closes with smooth animation
- [ ] AC5: DropdownMenu opens on click with correct positioning
- [ ] AC6: Tooltip shows on hover with correct positioning
- [ ] AC7: `cd frontend && pnpm build` exits 0
- [ ] AC8: Icon wrapper works and renders Iconify icons correctly

## Mandatory commands

- `cd frontend && pnpm build`

## Evidence required in transcript

- Build output last 10 lines
- List of files in `frontend/src/shared/ui/`
- Brief note on which components were customized from defaults

## Notes

- shadcn-vue components become project-owned code — modify them freely
- Do NOT depend on shadcn-vue's default styling; always customize to Макошь theme
- Icon wrapper should accept icon name string (e.g., "tabler:mail") and render the correct Iconify icon
- If `npx shadcn-vue add` has issues, create the component files manually based on shadcn-vue source
```

### `.supergoal/makosh-frontend-migration-vue-3-WzENWm/phases/phase-4.md`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/.supergoal/makosh-frontend-migration-vue-3-WzENWm/phases/phase-4.md`
- Size bytes / Размер в байтах: `2665`
- Included characters / Включено символов: `2637`
- Truncated / Обрезано: `no`

```markdown
SUPERGOAL_PHASE_START
Phase: 4 of 15 — Settings Domain
Task: Port the Settings page with all panels to Vue
Mandatory commands: cd frontend && pnpm build
Acceptance criteria: 7
Evidence required: build output, settings domain file listing
Depends on phases: 1, 2, 3

## Why

Settings is the simplest domain and serves as the first real domain port to validate the architecture pattern (domain structure, TanStack Query, Pinia boundary, component hierarchy).

## Work

1. **Create settings domain structure** under `frontend/src/domains/settings/`:
   - `types/settings.ts` — TypeScript types for settings
   - `api/settings.ts` — API functions (getSettings, updateSettings, etc.)
   - `queries/useSettingsQuery.ts` — TanStack Query hooks
   - `stores/settings.ts` — Pinia store for UI state only (active section, action messages)
   - `views/SettingsPage.vue` — main settings page
   - `components/` — settings-specific components
   - `routes/index.ts` — route definitions for settings

2. **Port each settings panel:**
   - AppearanceSettings — theme toggle (light/dark)
   - LanguageSettings — locale switch (en/ru)
   - IntegrationsSettings — connected accounts list
   - SidebarSettings — sidebar configuration
   - ApplicationSettings — app-level settings
   - AI Settings panels (OverviewPanel, AIApiProvidersPanel, AIBuiltInProvidersPanel, AICliProvidersPanel, AIModelRoutingPanel, AIPromptStudioPanel, AIRunsHealthPanel, AISettingsControlCenter, AISettingsHeader, AISettingsRail, AISettingsStatus, AISettingsTabs)

3. **Register route** in `frontend/src/app/router.ts`

4. **Verify:**
   - Build passes
   - Each settings panel renders with correct data from API

## Acceptance criteria

- [ ] AC1: All settings panels render with correct data from backend API via TanStack Query
- [ ] AC2: Theme toggle (light/dark) works and persists across reload
- [ ] AC3: Language switch (en/ru) works and persists — i18n store updates reactively
- [ ] AC4: Integrations settings shows connected accounts list
- [ ] AC5: AI settings panels render
- [ ] AC6: `cd frontend && pnpm build` exits 0
- [ ] AC7: Settings page is accessible via route `/settings`

## Mandatory commands

- `cd frontend && pnpm build`

## Evidence required in transcript

- Build output
- List of settings domain files
- Note on which API endpoints are used

## Notes

- Reference Svelte implementations in `frontend/src-svelte/lib/pages/settings/` and `frontend/src-svelte/lib/stores/settings.ts`
- API endpoints: study `frontend/src-svelte/lib/api/endpoints/settings.ts` and `frontend/src-svelte/lib/services/settings.ts`
- Keep each Vue component under 500 lines
```

### `.supergoal/makosh-frontend-migration-vue-3-WzENWm/phases/phase-5.md`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/.supergoal/makosh-frontend-migration-vue-3-WzENWm/phases/phase-5.md`
- Size bytes / Размер в байтах: `1620`
- Included characters / Включено символов: `1604`
- Truncated / Обрезано: `no`

```markdown
SUPERGOAL_PHASE_START
Phase: 5 of 15 — Home Dashboard
Task: Port the Home dashboard page with all widgets to Vue
Mandatory commands: cd frontend && pnpm build
Acceptance criteria: 5
Evidence required: build output, home domain file listing
Depends on phases: 1, 2, 3

## Why

Home dashboard is the default landing view. It validates the widget-based workspace pattern and serves as reference for other domain views.

## Work

1. **Create home domain structure** under `frontend/src/domains/home/`:
   - `types/`, `api/`, `queries/`, `stores/`, `components/`, `views/`, `routes/`

2. **Port HomePage** and all widget components:
   - HomeActiveProjects — active projects list
   - HomeMetrics — key metrics display
   - HomePeopleTalked — recent contacts
   - HomePriorities — priority items
   - HomeSystemStatus — system status indicators
   - HomeUpcoming — upcoming events/tasks
   - HomeWhatsNew — what's new feed

3. **Register route** for `/home`

4. **Verify:**
   - Build passes
   - Home page renders all widgets with real API data

## Acceptance criteria

- [ ] AC1: Home page renders all 7 widget types in correct layout
- [ ] AC2: Widgets show real data from API (not mock data)
- [ ] AC3: Widget layout responds to workspace resizing
- [ ] AC4: `cd frontend && pnpm build` exits 0
- [ ] AC5: Empty state renders when no data is available

## Mandatory commands

- `cd frontend && pnpm build`

## Evidence required in transcript

- Build output

## Notes

- Reference `frontend/src-svelte/lib/pages/home/`
- Widget components use TanStack Query for data
- Keep each component under 500 lines
```

### `.supergoal/makosh-frontend-migration-vue-3-WzENWm/phases/phase-6.md`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/.supergoal/makosh-frontend-migration-vue-3-WzENWm/phases/phase-6.md`
- Size bytes / Размер в байтах: `2519`
- Included characters / Включено символов: `2505`
- Truncated / Обрезано: `no`

```markdown
SUPERGOAL_PHASE_START
Phase: 6 of 15 — Personas & Organizations
Task: Port Personas (people) and Organizations domain pages to Vue
Mandatory commands: cd frontend && pnpm build
Acceptance criteria: 6
Evidence required: build output, personas and organizations domain file listings
Depends on phases: 1, 2, 3

## Why

Personas and Organizations are related entity domains that share UI patterns (list + detail, identity review, relationship review). Porting together reduces overhead and reinforces the domain-driven pattern.

## Work

1. **Create personas domain** under `frontend/src/domains/personas/`:
   - `types/persona.ts` — Persona types (PersonaType, Identity, Relationship, etc.)
   - `api/personas.ts` — API functions for person CRUD, identity review, relationship review
   - `queries/usePersonasQuery.ts` — TanStack Query hooks
   - `stores/personas.ts` — Pinia store for UI state (selected persona, active tab)
   - `components/` — PersonsList, PersonsDetail, PersonsIdentityReview, PersonsRelationshipReview, PersonsIdentityTraceReview
   - `views/PersonsPage.vue` — main persons page with widget layout
   - `routes/index.ts`

2. **Create organizations domain** under `frontend/src/domains/organizations/`:
   - Same structure as personas
   - Components: OrganizationsDashboard, OrganizationsHero, OrganizationsRail

3. **Register routes** for `/persons` and `/organizations`

4. **Verify:**
   - Build passes
   - Personas list renders with data
   - Detail view shows identity, relationships, communication history
   - Organizations page renders

## Acceptance criteria

- [ ] AC1: Personas list renders with virtual scrolling from API data
- [ ] AC2: Persona detail view shows identity, relationships, and communication
- [ ] AC3: Identity review panel renders with reviewable data
- [ ] AC4: Relationship review panel renders with suggestions
- [ ] AC5: Organizations page renders with dashboard, hero, rail widgets
- [ ] AC6: `cd frontend && pnpm build` exits 0

## Mandatory commands

- `cd frontend && pnpm build`

## Evidence required in transcript

- Build output
- List of personas domain files
- List of organizations domain files

## Notes

- Reference `frontend/src-svelte/lib/pages/persons/` and `frontend/src-svelte/lib/pages/organizations/`
- Personas terminology per ADR-0084 (not "contacts")
- API endpoints: study `frontend/src-svelte/lib/api/endpoints/persons.ts` and organizations
- Persona types: PersonaType = 'human' | 'ai_agent' | 'organization_proxy' | 'system'
```

### `.supergoal/makosh-frontend-migration-vue-3-WzENWm/phases/phase-7.md`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/.supergoal/makosh-frontend-migration-vue-3-WzENWm/phases/phase-7.md`
- Size bytes / Размер в байтах: `1755`
- Included characters / Включено символов: `1753`
- Truncated / Обрезано: `no`

```markdown
SUPERGOAL_PHASE_START
Phase: 7 of 15 — Projects & Tasks
Task: Port Projects and Tasks domain pages to Vue
Mandatory commands: cd frontend && pnpm build
Acceptance criteria: 5
Evidence required: build output, projects and tasks domain file listings
Depends on phases: 1, 2, 3

## Why

Projects and Tasks are related domains sharing UI patterns for obligation and decision review. Porting together ensures consistency in how task candidates, obligations, and decisions are displayed.

## Work

1. **Create projects domain** under `frontend/src/domains/projects/`:
   - Types, API, queries, stores, components (ProjectsDashboard, ProjectsHero, ProjectsRail), views/ProjectsPage, routes

2. **Create tasks domain** under `frontend/src/domains/tasks/`:
   - Types, API, queries, stores, components (TaskList with TanStack Virtual), views/TasksPage, routes
   - Task list must use TanStack Virtual for virtualization
   - Include obligation/decision review panel integration

3. **Register routes** for `/projects` and `/tasks`

4. **Verify:**
   - Build passes
   - Projects dashboard renders
   - Tasks list renders with virtual scrolling

## Acceptance criteria

- [ ] AC1: Projects page renders with dashboard, hero, rail widgets
- [ ] AC2: Tasks list renders with virtual scrolling (TanStack Virtual)
- [ ] AC3: Task items show correct status, priority, and metadata from API
- [ ] AC4: Obligation/decision review panel renders within tasks
- [ ] AC5: `cd frontend && pnpm build` exits 0

## Mandatory commands

- `cd frontend && pnpm build`

## Evidence required in transcript

- Build output

## Notes

- Reference `frontend/src-svelte/lib/pages/projects/` and `frontend/src-svelte/lib/pages/tasks/`
- Use TanStack Virtual for task list virtualization
```

### `.supergoal/makosh-frontend-migration-vue-3-WzENWm/phases/phase-8.md`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/.supergoal/makosh-frontend-migration-vue-3-WzENWm/phases/phase-8.md`
- Size bytes / Размер в байтах: `1229`
- Included characters / Включено символов: `1227`
- Truncated / Обрезано: `no`

```markdown
SUPERGOAL_PHASE_START
Phase: 8 of 15 — Calendar Domain
Task: Port Calendar domain page to Vue
Mandatory commands: cd frontend && pnpm build
Acceptance criteria: 4
Evidence required: build output, calendar domain file listing
Depends on phases: 1, 2, 3

## Why

Calendar is a self-contained domain with events from multiple providers. Porting it validates the domain pattern for date/time-intensive views.

## Work

1. **Create calendar domain** under `frontend/src/domains/calendar/`:
   - Types, API, queries (useCalendarEventsQuery), stores, components, views/CalendarPage, routes
   - Calendar event display with date formatting via date-fns

2. **Register route** for `/calendar`

3. **Verify:**
   - Build passes
   - Calendar renders with events

## Acceptance criteria

- [ ] AC1: Calendar page renders with events from API
- [ ] AC2: Events show correct date/time info formatted via date-fns
- [ ] AC3: Empty state renders when no events
- [ ] AC4: `cd frontend && pnpm build` exits 0

## Mandatory commands

- `cd frontend && pnpm build`

## Evidence required in transcript

- Build output

## Notes

- Reference `frontend/src-svelte/lib/pages/calendar/` and Svelte CalendarPage
- Use date-fns for all date formatting
```

### `.supergoal/makosh-frontend-migration-vue-3-WzENWm/phases/phase-9.md`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/.supergoal/makosh-frontend-migration-vue-3-WzENWm/phases/phase-9.md`
- Size bytes / Размер в байтах: `1814`
- Included characters / Включено символов: `1808`
- Truncated / Обрезано: `no`

```markdown
SUPERGOAL_PHASE_START
Phase: 9 of 15 — Documents & Notes
Task: Port Documents and Notes domain pages to Vue
Mandatory commands: cd frontend && pnpm build
Acceptance criteria: 5
Evidence required: build output, documents and notes domain file listings
Depends on phases: 1, 2, 3

## Why

Documents and Notes are content-focused domains with similar UI patterns (lists with source filters, insights panels, virtual scrolling). Porting together validates pattern reuse.

## Work

1. **Create documents domain** under `frontend/src/domains/documents/`:
   - Types, API, queries (TanStack Query hooks), stores (Pinia for UI state only)
   - Views/DocumentsPage.vue — main page
   - Components: DocumentsList (with TanStack Virtual), DocumentsNavigation, DocumentsSourceCards, DocumentsInsights, DocumentsProcessingJobs
   - Routes

2. **Create notes domain** under `frontend/src/domains/notes/`:
   - Types, API, queries, stores
   - Views/NotesPage.vue — main page
   - Components: NotesList (with TanStack Virtual), NotesSourceFilters, NotesInsights
   - Routes

3. **Register routes** for `/documents` and `/notes`

4. **Verify:**
   - Build passes
   - Documents and Notes pages render with data

## Acceptance criteria

- [ ] AC1: Documents page renders with list, navigation, source cards, insights
- [ ] AC2: Documents list uses TanStack Virtual for scrolling
- [ ] AC3: Notes page renders with list, source filters, insights
- [ ] AC4: Notes list uses TanStack Virtual for scrolling
- [ ] AC5: `cd frontend && pnpm build` exits 0

## Mandatory commands

- `cd frontend && pnpm build`

## Evidence required in transcript

- Build output

## Notes

- Reference `frontend/src-svelte/lib/pages/documents/` and `frontend/src-svelte/lib/pages/notes/`
- Both domains use TanStack Virtual for list virtualization
```

### `.supergoal/makosh-frontend-migration-vue-3-WzENWm/repo-map.md`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/.supergoal/makosh-frontend-migration-vue-3-WzENWm/repo-map.md`
- Size bytes / Размер в байтах: `2989`
- Included characters / Включено символов: `2979`
- Truncated / Обрезано: `no`

```markdown
# Repo map

_Generated 2026-06-14 13:26:32_

## Top-level layout
- AGENTS.md
- backend
- Cargo.lock
- Cargo.toml
- CODE_OF_CONDUCT.md
- CONTRIBUTING.md
- crates
- design-qa.md
- docker
- docs
- frontend
- IMPLEMENTATION_STATUS.md
- LICENSE
- MAIL_WORKING_STATE.md
- Makefile
- plans
- README.md
- scripts
- SECURITY.md
- target
- tmp

## Source directories (depth 2)
## File counts (top extensions)
- `.rs`: 1024 files
- `.md`: 229 files
- `.ts`: 164 files
- `.svelte`: 121 files
- `.sql`: 74 files
- `.png`: 50 files
- `.css`: 47 files
- `.json`: 6 files
- `.toml`: 4 files
- `.mjs`: 4 files

## Largest source files (top 15 by line count)
- `frontend/src-tauri/icons/icon.icns` (6809 lines)
- `backend/tests/email_account_setup.rs` (2026 lines)
- `backend/tests/telegram.rs` (1996 lines)
- `backend/tests/calendar_api.rs` (1546 lines)
- `backend/tests/persons.rs` (1316 lines)
- `backend/tests/messages.rs` (1305 lines)
- `backend/tests/consistency_contradiction.rs` (1147 lines)
- `backend/tests/v1_communications_api.rs` (1060 lines)
- `backend/tests/persons_api.rs` (953 lines)
- `backend/tests/calendar.rs` (925 lines)
- `backend/tests/graph_projection.rs` (877 lines)
- `backend/tests/task_candidates.rs` (845 lines)
- `backend/tests/person_identity.rs` (806 lines)
- `backend/tests/communication_ingestion.rs` (803 lines)
- `backend/tests/tasks_api.rs` (770 lines)

## Test surface
- Directories named `test`: 4
- Directories named `tests`: 201
- Directories named `spec`: 1
- Directories named `specs`: 1
- Test files (by name pattern): 40

## Notable config / infra
- `.github/workflows`

## Recent activity (last 10 commits)
- `662de2dd` 2026-06-14 Remove obsolete documentation and cleanup repository
- `c284c9d0` 2026-06-14 Rework Макошь foundation docs and backend alignment
- `9e8a631c` 2026-06-14 refactor: split document processing boundary
- `4c82f71f` 2026-06-14 refactor: split decisions boundary
- `219ebe8f` 2026-06-14 refactor: split mail messages boundary
- `9c1e1ada` 2026-06-14 refactor: split task candidates boundary
- `1bc31a45` 2026-06-14 refactor: split platform settings boundary
- `f65a2044` 2026-06-14 refactor: split consistency engine boundary
- `4298e421` 2026-06-13 refactor: split app router boundary
- `7447adbc` 2026-06-13 refactor: split persons handlers boundary

## Files churned in last 20 commits (top 10)
- `IMPLEMENTATION_STATUS.md` (20×)
- `docs/superpowers/plans/2026-06-13-documentation-alignment-architecture-gates.md` (7×)
- `backend/src/platform/settings/definitions.rs` (2×)
- `backend/src/integrations/telegram/tdjson/qr_login.rs` (2×)
- `backend/src/integrations/telegram/tdjson/qr_login_support.rs` (2×)
- `backend/src/integrations/telegram/tdjson/parsing.rs` (2×)
- `backend/src/integrations/telegram/tdjson.rs` (2×)
- `backend/src/integrations/telegram/runtime/manager.rs` (2×)
- `backend/src/integrations/telegram/runtime/actor.rs` (2×)
- `backend/src/integrations/telegram/client/models.rs` (2×)

_End repo map._
```

### `.supergoal/makosh-frontend-migration-vue-3-WzENWm/tools.md`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/.supergoal/makosh-frontend-migration-vue-3-WzENWm/tools.md`
- Size bytes / Размер в байтах: `827`
- Included characters / Включено символов: `825`
- Truncated / Обрезано: `no`

```markdown
# Available Tools

## Detected tools (Stage 0)

- **codebase_search**: Available (semantic code search)
- **search_files**: Available (regex file search)
- **list_files**: Available
- **read_file**: Available
- **execute_command**: Available
- **write_to_file**: Available
- **apply_diff**: Available
- **ask_followup_question**: Available
- **new_task**: Available (subagent dispatch)
- **skill**: Available

## Not available
- **WebSearch**: NOT available
- **WebFetch**: NOT available
- **Context7**: NOT available
- **MCP clients**: NOT detected

## Implications
- Web research is skipped — rely on training-cutoff knowledge of Vue 3, TanStack, Tailwind, shadcn-vue
- All planning evidence comes from local recon + codebase_search + file reading
- Subagent dispatch (new_task) available for parallel work within phases
```
