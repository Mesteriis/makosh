# AI Compose Edge Panels Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Redesign the mail compose modal into a stable rich editor with Samsung-edge-style side handles for AI actions and context tools.

**Architecture:** Keep compose as a Communications surface. `MailWorkspace.vue` owns open/closed UI state, tiny presentation helpers live beside mail components, and CSS renders side panels behind the foreground compose card without changing send/draft APIs.

**Tech Stack:** Vue 3 `<script setup>`, existing Макошь `Dialog`, `Icon`, `RichTextEditor`, CSS in `communicationDomainElements.css`, Vitest boundary tests.

## Global Constraints

- First plan item is already complete: commit `4322e2a63 feat: stabilize provider setup and mail workspace`.
- No provider-specific business cache roots.
- AI results must not silently mutate the draft.
- Account ids, raw provider ids, and backend refs must not be normal user-facing compose labels.
- User-facing failures should use global notifications; inline compose errors only for fixable fields.
- Side panel handles belong to the panels and move with them.
- Open panels must not cover the editor, account selector, address fields, toolbar, or footer buttons.

---

### Task 1: Commit Current Workspace

**Files:**
- Already committed current staged workspace.

**Interfaces:**
- Produces: clean baseline commit `4322e2a63` on `main`.

- [x] **Step 1: Stage current non-ignored workspace**

Run:

```bash
git add backend frontend scripts docs/superpowers/specs/2026-07-07-ai-compose-edge-panels-design.md
```

- [x] **Step 2: Validate staged content**

Run:

```bash
git diff --cached --check
gitleaks protect --staged --redact --verbose
make architecture-check
make frontend-lint
make backend-fmt-check
make backend-clippy
```

Expected: all commands pass.

- [x] **Step 3: Commit**

Run:

```bash
git commit -m "feat: stabilize provider setup and mail workspace"
```

Expected: commit `4322e2a63`.

### Task 2: Add Compose Panel Presentation Model

**Files:**
- Modify: `frontend/src/domains/communications/components/mail/mailComposeOptions.ts`

**Interfaces:**
- Consumes: `CommunicationAccountOption` from `frontend/src/domains/communications/types/communications.ts`.
- Produces:
  - `type ComposeEdgePanelId = 'ai' | 'context'`
  - `type ComposeEdgePanelAction`
  - `type ComposeEdgePanelSection`
  - `composeAiPanelActions(): ComposeEdgePanelAction[]`
  - `composeContextPanelSections(accounts: readonly CommunicationAccountOption[]): ComposeEdgePanelSection[]`

- [x] **Step 1: Add failing boundary expectations**

Modify `frontend/src/domains/communications/views/CommunicationsWorkspaceSurface.boundary.test.ts` to assert these strings:

```ts
expect(mailComposeOptionsSource).toContain('ComposeEdgePanelId')
expect(mailComposeOptionsSource).toContain('composeAiPanelActions')
expect(mailComposeOptionsSource).toContain('composeContextPanelSections')
expect(mailWorkspaceSource).not.toContain('{{ account.account_id }}')
```

Add `mailComposeOptionsSource` by reading `../components/mail/mailComposeOptions.ts`.

- [x] **Step 2: Run test to verify it fails**

Run:

```bash
cd frontend && pnpm test:unit -- CommunicationsWorkspaceSurface.boundary.test.ts
```

Result: failed until helper exports existed.

- [x] **Step 3: Implement presentation exports**

Add to `mailComposeOptions.ts`:

```ts
export type ComposeEdgePanelId = 'ai' | 'context'

export type ComposeEdgePanelAction = {
  id: string
  label: string
  icon: string
  description: string
  disabled?: boolean
}

export type ComposeEdgePanelSection = {
  id: string
  title: string
  icon: string
  items: string[]
}

export function composeAiPanelActions(): ComposeEdgePanelAction[] {
  return [
    { id: 'prompt', label: 'Prompt to email', icon: 'tabler:sparkles', description: 'Draft from intent' },
    { id: 'rewrite', label: 'Rewrite draft', icon: 'tabler:refresh-dot', description: 'Keep meaning, improve shape' },
    { id: 'tone', label: 'Adjust tone', icon: 'tabler:mood-smile', description: 'Make it warmer, firmer, or shorter' },
    { id: 'translate', label: 'Translate', icon: 'tabler:language', description: 'Prepare another language version' },
    { id: 'correct', label: 'Autocorrect', icon: 'tabler:writing-sign', description: 'Fix typos and grammar' },
  ]
}

export function composeContextPanelSections(
  accounts: readonly CommunicationAccountOption[]
): ComposeEdgePanelSection[] {
  const senders = accounts
    .filter((account) => account.can_send)
    .map((account) => account.email || account.label)
  return [
    { id: 'templates', title: 'Templates', icon: 'tabler:template', items: ['Quick reply', 'Follow-up', 'Intro'] },
    { id: 'signatures', title: 'Signatures', icon: 'tabler:signature', items: ['Default signature', 'Short signature'] },
    { id: 'recipients', title: 'Recipient review', icon: 'tabler:users', items: senders.length > 0 ? senders : ['Select sender'] },
    { id: 'safety', title: 'Safety checks', icon: 'tabler:shield-check', items: ['Sender available', 'Address fields checked'] },
  ]
}
```

- [x] **Step 4: Run test to verify it passes**

Run:

```bash
cd frontend && pnpm test:unit -- CommunicationsWorkspaceSurface.boundary.test.ts
```

Result: PASS.

### Task 3: Wire Edge Panel State Into MailWorkspace

**Files:**
- Modify: `frontend/src/domains/communications/components/mail/MailWorkspace.vue`

**Interfaces:**
- Consumes:
  - `ComposeEdgePanelId`
  - `composeAiPanelActions()`
  - `composeContextPanelSections(composeAccountOptions.value)`
- Produces:
  - `activeComposePanel`
  - `toggleComposeEdgePanel(panelId: ComposeEdgePanelId)`
  - `closeComposeEdgePanels()`

- [x] **Step 1: Add failing boundary expectations**

Add expectations to `CommunicationsWorkspaceSurface.boundary.test.ts`:

```ts
expect(mailWorkspaceSource).toContain('activeComposePanel')
expect(mailWorkspaceSource).toContain('toggleComposeEdgePanel')
expect(mailWorkspaceSource).toContain('closeComposeEdgePanels')
expect(mailWorkspaceSource).toContain('compose-edge-panel--left')
expect(mailWorkspaceSource).toContain('compose-edge-panel--right')
```

- [x] **Step 2: Run test to verify it fails**

Run:

```bash
cd frontend && pnpm test:unit -- CommunicationsWorkspaceSurface.boundary.test.ts
```

Result: failure until state and markup existed.

- [x] **Step 3: Implement state**

Add imports:

```ts
import {
  composeAccountOptionSignature,
  composeAiPanelActions,
  composeContextPanelSections,
  sendCapableComposeAccounts,
  type ComposeEdgePanelId
} from './mailComposeOptions'
```

Add state:

```ts
const activeComposePanel = ref<ComposeEdgePanelId | null>(null)
const composeAiActions = computed(() => composeAiPanelActions())
const composeContextSections = computed(() => composeContextPanelSections(composeAccountOptions.value))

function toggleComposeEdgePanel(panelId: ComposeEdgePanelId): void {
  activeComposePanel.value = activeComposePanel.value === panelId ? null : panelId
}

function closeComposeEdgePanels(): void {
  activeComposePanel.value = null
}
```

Update `handleComposeDialogOpenChange`:

```ts
function handleComposeDialogOpenChange(open: boolean): void {
  if (!open) {
    closeComposeEdgePanels()
    emit('close-compose')
  }
}
```

- [x] **Step 4: Run test to verify state strings pass**

Run:

```bash
cd frontend && pnpm test:unit -- CommunicationsWorkspaceSurface.boundary.test.ts
```

Result: PASS after state and markup landed.

### Task 4: Render Foreground Compose Card With Background Wings

**Files:**
- Modify: `frontend/src/domains/communications/components/mail/MailWorkspace.vue`
- Modify: `frontend/src/domains/communications/components/communicationDomainElements.css`

**Interfaces:**
- Consumes: `activeComposePanel`, `composeAiActions`, `composeContextSections`.
- Produces:
  - `.mail-compose-stage`
  - `.mail-compose-card`
  - `.compose-edge-panel`
  - `.compose-edge-panel__handle`

- [x] **Step 1: Wrap compose content**

Inside `Dialog`, wrap current `section.mail-compose-panel` and footer content in:

```vue
<section class="mail-compose-stage" :data-active-panel="activeComposePanel ?? 'none'">
  <aside class="compose-edge-panel compose-edge-panel--left" :class="{ 'is-open': activeComposePanel === 'ai' }">
    <button type="button" class="compose-edge-panel__handle" :aria-expanded="activeComposePanel === 'ai'" @click="toggleComposeEdgePanel('ai')">
      <Icon icon="tabler:sparkles" size="1rem" />
      <span>{{ t('AI') }}</span>
    </button>
    <div class="compose-edge-panel__surface">
      <button v-for="action in composeAiActions" :key="action.id" type="button" class="compose-edge-panel__action">
        <Icon :icon="action.icon" size="1rem" />
        <span>{{ t(action.label) }}</span>
        <small>{{ t(action.description) }}</small>
      </button>
    </div>
  </aside>
  <section v-if="composeForm" class="mail-compose-panel mail-compose-card" :aria-label="composeTitle">
    <!-- existing fields -->
  </section>
  <aside class="compose-edge-panel compose-edge-panel--right" :class="{ 'is-open': activeComposePanel === 'context' }">
    <button type="button" class="compose-edge-panel__handle" :aria-expanded="activeComposePanel === 'context'" @click="toggleComposeEdgePanel('context')">
      <Icon icon="tabler:layout-sidebar-right" size="1rem" />
      <span>{{ t('Context') }}</span>
    </button>
    <div class="compose-edge-panel__surface">
      <section v-for="section in composeContextSections" :key="section.id" class="compose-edge-panel__section">
        <h3><Icon :icon="section.icon" size="1rem" /> {{ t(section.title) }}</h3>
        <p v-for="item in section.items" :key="item">{{ item }}</p>
      </section>
    </div>
  </aside>
</section>
```

- [x] **Step 2: Add CSS wing layout**

Add CSS:

```css
.mail-compose-stage {
  position: relative;
  display: grid;
  min-width: 0;
  isolation: isolate;
}

.mail-compose-card {
  position: relative;
  z-index: 2;
}

.compose-edge-panel {
  position: absolute;
  top: var(--h-spacing-4);
  bottom: var(--h-spacing-4);
  z-index: 1;
  width: min(280px, 28vw);
  pointer-events: none;
  transition: transform var(--h-motion-default), opacity var(--h-motion-default);
}

.compose-edge-panel--left {
  left: 0;
  transform: translateX(-32px);
}

.compose-edge-panel--right {
  right: 0;
  transform: translateX(32px);
}

.compose-edge-panel.is-open {
  pointer-events: auto;
  opacity: 1;
}

.compose-edge-panel--left.is-open {
  transform: translateX(calc(-100% + 18px));
}

.compose-edge-panel--right.is-open {
  transform: translateX(calc(100% - 18px));
}

.compose-edge-panel__handle {
  position: absolute;
  top: var(--h-spacing-5);
  display: inline-flex;
  align-items: center;
  gap: var(--h-spacing-1);
  min-height: var(--h-control-height-sm);
  border: 1px solid var(--h-color-border);
  border-radius: var(--h-radius-md);
  background: var(--h-color-surface-raised);
  color: var(--h-color-text-strong);
  box-shadow: var(--h-shadow-sm);
  cursor: pointer;
  pointer-events: auto;
  padding: 0 var(--h-spacing-2);
}
```

Continue CSS with left/right handle offsets, `.compose-edge-panel__surface`, actions, and compact media query.

- [x] **Step 3: Run frontend lint**

Run:

```bash
make frontend-lint
```

Result: PASS via `pnpm lint:ox`, `pnpm lint:styles`, `pnpm lint:srp`, and `pnpm lint:vue-boundaries`.

### Task 5: Validate And Commit Compose Slice

**Files:**
- Modified frontend compose files.

**Interfaces:**
- Produces: a second commit after the baseline commit.

- [x] **Step 1: Run targeted checks**

Run:

```bash
cd frontend && pnpm test:unit -- CommunicationsWorkspaceSurface.boundary.test.ts
make frontend-lint
git diff --check
gitleaks protect --staged --redact --verbose
```

Expected: all commands pass.

- [x] **Step 2: Commit**

Run:

```bash
git add frontend/src/domains/communications/components/mail/MailWorkspace.vue \
  frontend/src/domains/communications/components/mail/mailComposeOptions.ts \
  frontend/src/domains/communications/components/communicationDomainElements.css \
  frontend/src/domains/communications/views/CommunicationsWorkspaceSurface.boundary.test.ts \
  docs/superpowers/plans/2026-07-07-ai-compose-edge-panels.md
git commit -m "feat: add intelligent compose edge panels"
```

Expected: commit succeeds with repository hooks passing.

## Self-Review

- Spec coverage: panel ownership, handles, no internal ids, account selector continuity, explicit AI actions, responsive fallback, and validation all map to tasks 2-5.
- Placeholder scan: no `TBD`, `TODO`, `fill in`, or fake implementation text.
- Type consistency: `ComposeEdgePanelId`, `ComposeEdgePanelAction`, and `ComposeEdgePanelSection` are defined in Task 2 and consumed in Tasks 3-4.
