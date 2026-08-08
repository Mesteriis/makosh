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

- Chunk ID / ID чанка: `136-other-frontend-part-009`
- Group / Группа: `frontend`
- Role / Роль: `other`
- Status / Статус: `pending`
- Repository / Репозиторий: `/Users/avm/projects/Personal/makosh`
- Wiki path / Путь wiki: `/Users/avm/projects/Personal/makosh/docs/wiki`
- Metadata path / Путь metadata: `/Users/avm/projects/Personal/makosh/docs/wiki/_meta`
- Plan generated at / План создан: `2026-06-28T19:48:55Z`
- Per-file source limit / Лимит источника на файл: `12000` characters

## Target pages / Целевые страницы

- `components/frontend.md`

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

### `frontend/src/domains/notes/components/NotesList.vue`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/domains/notes/components/NotesList.vue`
- Size bytes / Размер в байтах: `2345`
- Included characters / Включено символов: `2345`
- Truncated / Обрезано: `no`

```text
<script setup lang="ts">
import { ref, computed } from 'vue'
import { useVirtualizer } from '@tanstack/vue-virtual'
import { useI18n } from '../../../platform/i18n'
import Icon from '../../../shared/ui/Icon.vue'
import type { NoteItem } from '../types/notes'

const { t } = useI18n()

const props = defineProps<{
  notes: NoteItem[]
  searchQuery: string
}>()

const emit = defineEmits<{
  'update:search-query': [value: string]
}>()

const parentRef = ref<HTMLDivElement | null>(null)

const virtualOptions = computed(() => ({
  count: props.notes.length,
  getScrollElement: () => parentRef.value,
  estimateSize: () => 100,
  overscan: 5
}))

const virtualizer = useVirtualizer(virtualOptions)

const virtualItems = computed(() => virtualizer.value.getVirtualItems())
const totalSize = computed(() => virtualizer.value.getTotalSize())
</script>

<template>
  <div class="widget-frame notes-list-panel">
    <div class="notes-main-list">
      <label class="local-search">
        <Icon icon="tabler:search" :size="17" />
        <input
          :placeholder="t('Search notes...')"
          :value="searchQuery"
          @input="emit('update:search-query', ($event.target as HTMLInputElement).value)"
        />
      </label>
      <div ref="parentRef" class="notes-scroll-container">
        <div v-if="notes.length === 0" class="muted p-4">{{ t('No notes found') }}</div>
        <div v-else :style="{ height: `${totalSize}px` }">
          <article
            v-for="vitem in virtualItems"
            :key="String(vitem.key)"
            class="note-card"
            :style="{ transform: `translateY(${vitem.start}px)`, height: `${vitem.size}px` }"
          >
            <span class="round-icon">
              <Icon :icon="notes[vitem.index].icon" :size="22" />
            </span>
            <div>
              <strong>{{ notes[vitem.index].title }}</strong>
              <p>{{ notes[vitem.index].body }}</p>
              <div class="note-meta">
                <span>{{ notes[vitem.index].source }}</span>
                <em>{{ notes[vitem.index].tag }}</em>
                <time>{{ notes[vitem.index].time }}</time>
              </div>
            </div>
          </article>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.notes-scroll-container {
  flex: 1;
  overflow-y: auto;
}
</style>
```

### `frontend/src/domains/notes/components/NotesSourceFilters.vue`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/domains/notes/components/NotesSourceFilters.vue`
- Size bytes / Размер в байтах: `1300`
- Included characters / Включено символов: `1300`
- Truncated / Обрезано: `no`

```text
<script setup lang="ts">
import { useI18n } from '../../../platform/i18n'

const { t } = useI18n()

defineProps<{
  activeSources: string[]
  activeTags: string[]
}>()

const emit = defineEmits<{
  'toggle-source': [source: string]
  'toggle-tag': [tag: string]
}>()

const sources = ['Apple Notes', 'Obsidian', 'Gmail', 'Anytype', 'Outlook']
const tags = ['#project', '#research', '#meeting', '#idea', '#reference', '#partnership']
</script>

<template>
  <aside class="left-panels">
    <div class="widget-frame">
      <section class="panel info-card">
        <h2>{{ t('Sources') }}</h2>
        <label v-for="src in sources" :key="src" class="mini-check">
          <input
            type="checkbox"
            :checked="activeSources.includes(src)"
            @change="emit('toggle-source', src)"
          />
          {{ t(src) }}
        </label>
      </section>
    </div>
    <div class="widget-frame">
      <section class="panel info-card">
        <h2>{{ t('Tags') }}</h2>
        <label v-for="tag in tags" :key="tag" class="mini-check">
          <input
            type="checkbox"
            :checked="activeTags.includes(tag)"
            @change="emit('toggle-tag', tag)"
          />
          {{ t(tag) }}
        </label>
      </section>
    </div>
  </aside>
</template>
```

### `frontend/src/domains/notes/views/NotesPage.vue`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/domains/notes/views/NotesPage.vue`
- Size bytes / Размер в байтах: `2438`
- Included characters / Включено символов: `2438`
- Truncated / Обрезано: `no`

```text
<script setup lang="ts">
import { computed } from 'vue'
import { useI18n } from '../../../platform/i18n'
import Icon from '../../../shared/ui/Icon.vue'
import { useNotesStore } from '../stores/notes'
import { useNotesQuery } from '../queries/useNotesQuery'
import type { NoteItem } from '../types/notes'
import NotesSourceFilters from '../components/NotesSourceFilters.vue'
import NotesList from '../components/NotesList.vue'
import NotesInsights from '../components/NotesInsights.vue'

const { t } = useI18n()
const store = useNotesStore()

const { data: notesData } = useNotesQuery()

const fallbackNotes: NoteItem[] = [
  { title: 'Welcome to Notes', body: 'This is your personal notes workspace. Notes from connected sources will appear here.', source: 'Макошь', tag: '#reference', time: new Date().toISOString(), icon: 'tabler:notes' },
  { title: 'Meeting Notes Template', body: 'Use this template to capture key decisions, action items, and follow-ups from meetings.', source: 'Макошь', tag: '#meeting', time: new Date().toISOString(), icon: 'tabler:clipboard-list' },
  { title: 'Project Ideas', body: 'A collection of project ideas and brainstorming notes for future development.', source: 'Макошь', tag: '#idea', time: new Date().toISOString(), icon: 'tabler:lightbulb' },
  { title: 'Research Notes', body: 'Research findings and references organized by topic for easy retrieval.', source: 'Макошь', tag: '#research', time: new Date().toISOString(), icon: 'tabler:books' }
]

const notes = computed<NoteItem[]>(() => notesData.value?.items ?? fallbackNotes)
</script>

<template>
  <section class="notes-page">
    <div class="view-header">
      <div class="view-title-with-icon">
        <span class="hero-mark small"><Icon icon="tabler:notes" :size="28" /></span>
        <div>
          <h1>{{ t('Notes') }}</h1>
          <p>{{ t('All your notes from connected sources') }}</p>
        </div>
      </div>
    </div>
    <div class="notes-layout">
      <NotesSourceFilters
        :active-sources="store.activeSources"
        :active-tags="store.activeTags"
        @toggle-source="store.toggleSource"
        @toggle-tag="store.toggleTag"
      />
      <NotesList
        :notes="notes"
        :search-query="store.searchQuery"
        @update:search-query="store.setSearchQuery"
      />
      <aside class="stacked-rail">
        <NotesInsights />
      </aside>
    </div>
  </section>
</template>
```

### `frontend/src/domains/organizations/components/OrganizationsDetail.vue`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/domains/organizations/components/OrganizationsDetail.vue`
- Size bytes / Размер в байтах: `4552`
- Included characters / Включено символов: `4551`
- Truncated / Обрезано: `no`

```text
<script setup lang="ts">
import { useI18n } from '../../../platform/i18n'
import Icon from '../../../shared/ui/Icon.vue'
import type { Organization } from '../types/organization'

const { t } = useI18n()

defineProps<{
  selectedOrganization: Record<string, unknown> | null
  orgPeople: unknown[]
}>()
</script>

<template>
  <div class="widget-frame" data-widget-id="organizations-detail">
    <section class="panel org-detail-panel">
      <template v-if="selectedOrganization">
        <header>
          <span class="round-icon blue"><Icon icon="tabler:building" :size="26" /></span>
          <div>
            <h2>{{ selectedOrganization.display_name as string }}</h2>
            <em>{{ selectedOrganization.industry as string || t('Unknown industry') }}{{ selectedOrganization.country ? ` · ${selectedOrganization.country as string}` : '' }}</em>
          </div>
        </header>
        <div class="org-detail-grid">
          <div class="info-card">
            <h3>{{ t('Status') }}</h3>
            <span class="status-chip">{{ selectedOrganization.status as string }}</span>
            <span v-if="selectedOrganization.health_status" class="health-chip">{{ selectedOrganization.health_status as string }}</span>
            <span v-if="selectedOrganization.watchlist" class="health-chip important">{{ t('Watchlist') }}</span>
          </div>
          <div v-if="selectedOrganization.description" class="info-card">
            <h3>{{ t('About') }}</h3>
            <p>{{ selectedOrganization.description as string }}</p>
          </div>
          <div class="info-card">
            <h3>{{ t('Details') }}</h3>
            <div v-if="selectedOrganization.website" class="detail-row">
              <span>{{ t('Website') }}</span>
              <strong>{{ selectedOrganization.website as string }}</strong>
            </div>
            <div v-if="selectedOrganization.legal_name" class="detail-row">
              <span>{{ t('Legal name') }}</span>
              <strong>{{ selectedOrganization.legal_name as string }}</strong>
            </div>
            <div v-if="selectedOrganization.registration_number" class="detail-row">
              <span>{{ t('Registration') }}</span>
              <strong>{{ selectedOrganization.registration_number as string }}</strong>
            </div>
            <div v-if="selectedOrganization.vat" class="detail-row">
              <span>{{ t('VAT') }}</span>
              <strong>{{ selectedOrganization.vat as string }}</strong>
            </div>
            <div class="detail-row">
              <span>{{ t('Interactions') }}</span>
              <strong>{{ selectedOrganization.interaction_count as string }}</strong>
            </div>
            <div class="detail-row">
              <span>{{ t('Priority') }}</span>
              <strong>{{ selectedOrganization.priority as string || t('normal') }}</strong>
            </div>
          </div>
          <div v-if="orgPeople.length > 0" class="info-card">
            <h3>{{ t('Key People') }}</h3>
            <div v-for="person in orgPeople" :key="(person as Record<string, unknown>).person_id as string" class="person-mini">
              <span class="round-icon"><Icon icon="tabler:user" :size="16" /></span>
              <strong>{{ (person as Record<string, unknown>).display_name as string }}</strong>
              <small>{{ (person as Record<string, unknown>).email_address as string }}</small>
            </div>
          </div>
        </div>
      </template>
      <template v-else>
        <header>
          <span class="round-icon"><Icon icon="tabler:building-off" :size="26" /></span>
          <div>
            <h2>{{ t('No company selected') }}</h2>
            <em>{{ t('Select a company from the list') }}</em>
          </div>
        </header>
      </template>
    </section>
  </div>
</template>

<style scoped>
.org-detail-panel {
  padding: 12px;
}
.org-detail-panel header {
  display: grid;
  grid-template-columns: auto 1fr;
  align-items: center;
  gap: 12px;
  margin-bottom: 16px;
}
.org-detail-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(200px, 1fr));
  gap: 12px;
}
.detail-row {
  display: flex;
  justify-content: space-between;
  padding: 6px 0;
  border-top: 1px solid rgba(102, 189, 180, 0.08);
  font-size: 12px;
}
.detail-row span {
  color: var(--hh-color-text-muted);
}
.person-mini {
  display: grid;
  grid-template-columns: auto 1fr;
  gap: 8px;
  align-items: center;
  padding: 6px 0;
  border-top: 1px solid rgba(102, 189, 180, 0.08);
}
</style>
```

### `frontend/src/domains/organizations/components/OrganizationsList.vue`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/domains/organizations/components/OrganizationsList.vue`
- Size bytes / Размер в байтах: `2507`
- Included characters / Включено символов: `2503`
- Truncated / Обрезано: `no`

```text
<script setup lang="ts">
import { useI18n } from '../../../platform/i18n'
import Icon from '../../../shared/ui/Icon.vue'
import type { Organization } from '../types/organization'

const { t } = useI18n()

defineProps<{
  organizations: Organization[]
  selectedOrganizationId: string
  isOrganizationsLoading: boolean
}>()

const emit = defineEmits<{
  selectOrg: [id: string]
}>()
</script>

<template>
  <div class="widget-frame" data-widget-id="organizations-list">
    <section class="panel org-list-panel">
      <header class="panel-title-row">
        <h2>{{ t('All Companies') }} ({{ organizations.length }})</h2>
      </header>
      <div v-if="isOrganizationsLoading && organizations.length === 0" class="graph-strip-message">
        <span>{{ t('Loading companies.') }}</span>
      </div>
      <div v-else-if="organizations.length === 0" class="graph-strip-message">
        <span>{{ t('No companies yet.') }}</span>
      </div>
      <template v-else>
        <button
          v-for="org in organizations"
          :key="org.organization_id"
          type="button"
          class="org-row"
          :class="{ active: selectedOrganizationId === org.organization_id }"
          @click="emit('selectOrg', org.organization_id)"
        >
          <span class="round-icon blue"><Icon icon="tabler:building" :size="20" /></span>
          <div>
            <strong>{{ org.display_name }}</strong>
            <p>{{ org.industry || t('Unknown industry') }}{{ org.country ? ` · ${org.country}` : '' }}</p>
          </div>
          <small>{{ org.status }}{{ org.watchlist ? ` · ⚠ ${t('watchlist')}` : '' }}</small>
        </button>
      </template>
    </section>
  </div>
</template>

<style scoped>
.org-list-panel {
  padding: 12px;
}
.org-row {
  display: grid;
  grid-template-columns: 44px 1fr auto;
  gap: 10px;
  align-items: center;
  width: 100%;
  min-height: var(--hh-widget-card-compact);
  border: 1px solid transparent;
  border-radius: var(--hh-radius-md);
  background: transparent;
  color: #e6f7f5;
  padding: 9px 10px;
  text-align: left;
  cursor: pointer;
}
.org-row.active {
  border-color: rgba(45, 240, 206, 0.24);
  background: rgba(25, 109, 100, 0.24);
}
.org-row strong {
  display: block;
  color: var(--hh-color-text-bright);
  font-size: 14px;
  font-weight: 560;
}
.org-row p,
.org-row small {
  display: block;
  margin-top: 5px;
  overflow: hidden;
  font-size: 11px;
  font-style: normal;
  text-overflow: ellipsis;
  white-space: nowrap;
}
</style>
```

### `frontend/src/domains/organizations/views/OrganizationsPage.vue`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/domains/organizations/views/OrganizationsPage.vue`
- Size bytes / Размер в байтах: `2202`
- Included characters / Включено символов: `2202`
- Truncated / Обрезано: `no`

```text
<script setup lang="ts">
import { useI18n } from '../../../platform/i18n'
import Icon from '../../../shared/ui/Icon.vue'
import { useOrganizationsQuery } from '../queries/useOrganizationsQuery'
import OrganizationsList from '../components/OrganizationsList.vue'
import OrganizationsDetail from '../components/OrganizationsDetail.vue'
import type { Organization } from '../types/organization'
import { ref, computed } from 'vue'

const { t } = useI18n()

const { data: organizationsData, isLoading } = useOrganizationsQuery()

const selectedOrganizationId = ref('')

const selectedOrganization = computed(() => {
  const orgs = organizationsData.value ?? []
  return orgs.find((o: Organization) => o.organization_id === selectedOrganizationId.value) ?? orgs[0] ?? null
})

const orgPeople = computed(() => [])
</script>

<template>
  <section class="organizations-page">
    <div class="view-header">
      <div class="view-title-with-icon">
        <span class="hero-mark small"><Icon icon="tabler:building" :size="28" /></span>
        <div>
          <h1>{{ t('Companies') }}</h1>
          <p>{{ t('All companies and organizations from your communications') }}</p>
        </div>
      </div>
    </div>
    <div class="org-layout">
      <OrganizationsList
        :organizations="organizationsData ?? []"
        :selectedOrganizationId="selectedOrganizationId"
        :isOrganizationsLoading="isLoading"
        @selectOrg="(id) => { selectedOrganizationId = id }"
      />
      <OrganizationsDetail
        :selectedOrganization="selectedOrganization as unknown as Record<string, unknown>"
        :orgPeople="orgPeople as unknown as unknown[]"
      />
    </div>
  </section>
</template>

<style scoped>
.organizations-page {
  display: grid;
  grid-template-columns: repeat(var(--hh-layout-columns), minmax(0, 1fr));
  grid-auto-flow: row;
  grid-auto-rows: min-content;
  align-content: start;
  gap: var(--hh-layout-gap);
  height: 100%;
  min-height: 0;
  overflow: hidden;
  padding-right: 0;
}
.organizations-page > * {
  grid-column: 1 / -1;
  min-width: 0;
}
.org-layout {
  display: grid;
  grid-template-columns: 320px 1fr;
  gap: var(--hh-layout-gap);
  min-height: 0;
}
</style>
```

### `frontend/src/domains/personas/components/PersonsDetail.vue`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/domains/personas/components/PersonsDetail.vue`
- Size bytes / Размер в байтах: `5973`
- Included characters / Включено символов: `5972`
- Truncated / Обрезано: `no`

```text
<script setup lang="ts">
import { useI18n } from '../../../platform/i18n'
import Icon from '../../../shared/ui/Icon.vue'
import { dossierSectionPreview } from '../stores/personas'
import type { PersonItem, PersonDossier } from '../types/persona'

const { t } = useI18n()

defineProps<{
  selectedPerson: PersonItem | null
  personDossier: PersonDossier | null
  isPersonDossierLoading: boolean
  personDossierError: string
  whatsNew: any[]
  projects: any[]
}>()
</script>

<template>
  <section class="person-detail">
    <template v-if="selectedPerson">
      <div class="widget-frame" data-widget-id="persons-hero">
        <section class="panel person-hero">
          <span class="round-icon ghost large-avatar"><Icon icon="tabler:user" :size="40" /></span>
          <div>
            <h1>{{ selectedPerson.name }}</h1>
            <p>{{ selectedPerson.role }} at {{ selectedPerson.company }}</p>
            <small>{{ selectedPerson.status ?? selectedPerson.channel ?? 'Contact' }}</small>
          </div>
          <div class="chat-actions">
            <button type="button" disabled><Icon icon="tabler:mail" :size="17" /></button>
            <button type="button" disabled><Icon icon="tabler:phone" :size="17" /></button>
            <button type="button" disabled><Icon icon="tabler:video" :size="17" /></button>
            <button type="button" disabled><Icon icon="tabler:brand-whatsapp" :size="17" /></button>
          </div>
        </section>
      </div>
      <div class="section-tabs">
        <button type="button" class="active">Overview</button>
        <button type="button" disabled>Communications</button>
        <button type="button" disabled>Documents <em>24</em></button>
        <button type="button" disabled>Tasks <em>7</em></button>
        <button type="button" disabled>Projects <em>5</em></button>
        <button type="button" disabled>Notes</button>
      </div>
      <div class="person-cards">
        <div class="widget-frame" data-widget-id="persons-information">
          <section class="panel info-card">
            <h2>Person Information</h2>
            <ul class="detail-list">
              <li><Icon icon="tabler:mail" :size="17" /> {{ selectedPerson.company }} <em>Primary</em></li>
              <li><Icon icon="tabler:phone" :size="17" /> +1 (555) 123-4567 <em>Mobile</em></li>
              <li><Icon icon="tabler:brand-telegram" :size="17" /> @john.smith <em>Telegram</em></li>
              <li><Icon icon="tabler:map-pin" :size="17" /> New York, USA <em>Local Time: 18:42</em></li>
            </ul>
          </section>
        </div>
        <div class="widget-frame" data-widget-id="persons-about">
          <section class="panel info-card">
            <h2>Persona Dossier</h2>
            <p v-if="isPersonDossierLoading">Loading dossier...</p>
            <p v-else-if="personDossierError" class="inline-error">{{ personDossierError }}</p>
            <template v-else-if="personDossier">
              <p>{{ personDossier.summary || 'No dossier summary yet.' }}</p>
              <div v-if="dossierSectionPreview(personDossier).length" class="tag-cloud">
                <span v-for="item in dossierSectionPreview(personDossier)" :key="item">{{ item }}</span>
              </div>
              <small>{{ personDossier.source_refs.length }} source refs · generated {{ new Date(personDossier.generated_at).toLocaleString() }}</small>
            </template>
            <p v-else>No dossier generated yet.</p>
          </section>
        </div>
        <div class="widget-frame" data-widget-id="persons-relationship-strength">
          <section class="panel info-card">
            <h2>Relationship Strength</h2>
            <div class="big-score">85</div>
            <strong>Strong</strong>
            <p>Last interaction 2 hours ago</p>
          </section>
        </div>
        <div class="widget-frame span-2" data-widget-id="persons-recent-interactions">
          <section class="panel info-card span-2">
            <h2>Recent Interactions</h2>
            <div v-for="item in whatsNew.slice(0, 3)" :key="item.title" class="feed-row compact-row">
              <span class="round-icon" :class="item.tone"><Icon :icon="item.icon" :size="18" /></span>
              <div>
                <strong>{{ item.title }}</strong>
                <p>{{ item.meta }}</p>
              </div>
              <time>{{ item.time }}</time>
            </div>
          </section>
        </div>
        <div class="widget-frame" data-widget-id="persons-active-projects">
          <section class="panel info-card">
            <h2>{{ t('Active Projects') }}</h2>
            <div v-for="project in projects.slice(0, 3)" :key="project.name" class="related-row">
              <span class="round-icon" :class="project.tone"><Icon :icon="project.icon" :size="16" /></span>
              <strong>{{ project.name }}</strong>
              <em>{{ project.progress }}%</em>
            </div>
          </section>
        </div>
      </div>
    </template>
    <template v-else>
      <section class="panel empty-domain-state">
        <Icon icon="tabler:user" :size="42" />
        <div>
          <h2>No person selected</h2>
          <p>Макошь will show relationship memory here after persons are imported from local sources.</p>
        </div>
      </section>
    </template>
  </section>
</template>

<style scoped>
.person-detail {
  display: grid;
  gap: 12px;
  align-content: start;
  min-width: 0;
}
.person-hero {
  display: grid;
  grid-template-columns: auto 1fr auto;
  align-items: center;
  gap: 12px;
  min-height: var(--hh-widget-card-compact);
  border-bottom: 1px solid rgba(102, 189, 180, 0.12);
  padding: 12px 16px;
}
.large-avatar {
  width: 92px;
  height: 92px;
  border-radius: var(--hh-radius-round);
  display: flex;
  align-items: center;
  justify-content: center;
}
.person-cards {
  display: grid;
  grid-template-columns: repeat(3, minmax(0, 1fr));
  gap: 12px;
}
</style>
```

### `frontend/src/domains/personas/components/PersonsIdentityReview.vue`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/domains/personas/components/PersonsIdentityReview.vue`
- Size bytes / Размер в байтах: `4403`
- Included characters / Включено символов: `4399`
- Truncated / Обрезано: `no`

```text
<script setup lang="ts">
import { useI18n } from '../../../platform/i18n'
import Icon from '../../../shared/ui/Icon.vue'
import type { PersonIdentityCandidate } from '../types/persona'

const { t } = useI18n()

defineProps<{
  suggestedIdentityCandidates: PersonIdentityCandidate[]
  confirmedMergeIdentityCandidates: PersonIdentityCandidate[]
  isIdentityCandidatesLoading: boolean
  identityCandidatesError: string
  identityConfidence: (candidate: PersonIdentityCandidate) => string
  setIdentityCandidateReview: (candidate: PersonIdentityCandidate, state: string) => Promise<void>
  splitConfirmedIdentityMerge: (candidate: PersonIdentityCandidate) => Promise<void>
  splitCandidateForConfirmedMerge: (candidate: PersonIdentityCandidate) => PersonIdentityCandidate | null
}>()
</script>

<template>
  <div class="widget-frame" data-widget-id="persons-identity-review">
    <section class="panel info-card">
      <h2>{{ t('Person Identity Review') }}</h2>
      <p class="identity-note">{{ t('Person merges are only suggested and are not applied until confirmed.') }}</p>
      <p v-if="isIdentityCandidatesLoading" class="inline-copy">{{ t('Loading identity suggestions…') }}</p>
      <p v-else-if="identityCandidatesError" class="inline-error">{{ identityCandidatesError }}</p>
      <p v-else-if="suggestedIdentityCandidates.length === 0 && confirmedMergeIdentityCandidates.length === 0" class="inline-copy">
        {{ t('No identity suggestions right now.') }}
      </p>
      <template v-else>
        <div v-for="candidate in suggestedIdentityCandidates" :key="candidate.candidate_id" class="identity-candidate-row">
          <div>
            <strong>{{ candidate.candidate_kind }}</strong>
            <p>{{ candidate.evidence_summary }}</p>
            <small>Left: {{ candidate.left_person_id }}</small>
            <small>Right: {{ candidate.right_person_id ?? t('N/A') }}</small>
            <small>{{ t('Confidence') }}: {{ identityConfidence(candidate) }} · {{ candidate.review_state }}</small>
          </div>
          <div class="identity-actions">
            <button type="button" @click="() => setIdentityCandidateReview(candidate, 'user_confirmed')">
              <Icon icon="tabler:check" :size="15" /> {{ t('Confirm') }}
            </button>
            <button type="button" @click="() => setIdentityCandidateReview(candidate, 'user_rejected')">
              <Icon icon="tabler:x" :size="15" /> {{ t('Reject') }}
            </button>
          </div>
        </div>
        <div v-for="candidate in confirmedMergeIdentityCandidates" :key="candidate.candidate_id" class="identity-candidate-row">
          <div>
            <strong>{{ candidate.candidate_kind }}</strong>
            <p>{{ candidate.evidence_summary }}</p>
            <small>Left: {{ candidate.left_person_id }}</small>
            <small>Right: {{ candidate.right_person_id ?? t('N/A') }}</small>
            <small>{{ t('Confidence') }}: {{ identityConfidence(candidate) }} · {{ candidate.review_state }}</small>
          </div>
          <div class="identity-actions">
            <button
              type="button"
              :disabled="splitCandidateForConfirmedMerge(candidate) === null"
              :title="splitCandidateForConfirmedMerge(candidate) === null ? t('Refresh identity candidates to create a split review for this confirmed link') : undefined"
              @click="() => splitConfirmedIdentityMerge(candidate)"
            >
              <Icon icon="tabler:arrows-split" :size="15" /> {{ t('Split') }}
            </button>
          </div>
        </div>
      </template>
    </section>
  </div>
</template>

<style scoped>
.identity-note {
  margin: 0 0 8px;
  color: var(--hh-color-text-muted);
  font-size: 12px;
}
.identity-candidate-row {
  display: grid;
  gap: 8px;
  padding: 10px 0;
  border-top: 1px solid rgba(102, 189, 180, 0.08);
}
.identity-candidate-row:first-child {
  border-top: none;
}
.identity-candidate-row strong {
  display: block;
  margin-bottom: 3px;
}
.identity-candidate-row p {
  margin: 0 0 4px;
  color: #dbe9e8;
}
.identity-candidate-row small {
  display: block;
  color: var(--hh-color-text-muted);
}
.identity-actions {
  display: inline-flex;
  gap: 7px;
  margin-top: 8px;
}
.identity-actions button {
  display: inline-flex;
  align-items: center;
  gap: 4px;
  padding: 4px 8px;
  height: auto;
  font-size: 11px;
}
</style>
```

### `frontend/src/domains/personas/components/PersonsIdentityTraceReview.vue`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/domains/personas/components/PersonsIdentityTraceReview.vue`
- Size bytes / Размер в байтах: `5446`
- Included characters / Включено символов: `5444`
- Truncated / Обрезано: `no`

```text
<script setup lang="ts">
import { useI18n } from '../../../platform/i18n'
import Icon from '../../../shared/ui/Icon.vue'
import { computed } from 'vue'
import { formatIdentityTraceKind, formatIdentityTraceValue, identityTraceConfidence } from '../stores/personas'
import type { PersonIdentity, PersonaOption } from '../types/persona'

const { t } = useI18n()

const props = defineProps<{
  identityTraces: PersonIdentity[]
  persons: PersonaOption[]
  selectedPersonaId: string | null
  isLoading: boolean
  error: string
  assigningIdentityTraceId: string | null
  onReload: () => Promise<void>
  onAssign: (trace: PersonIdentity, personId: string) => Promise<void>
}>()

const pendingIdentityTraces = computed(() =>
  props.identityTraces.filter((trace) => trace.person_id === null)
)

const selectedPersonaByTrace: Record<string, string> = {}

function targetPersonaId(trace: PersonIdentity): string {
  return selectedPersonaByTrace[trace.id] ?? props.selectedPersonaId ?? props.persons[0]?.person_id ?? ''
}

function personaLabel(person: PersonaOption): string {
  return person.company ? `${person.name} · ${person.company}` : person.name
}
</script>

<template>
  <div class="widget-frame" data-widget-id="persons-identity-trace-review">
    <section class="panel info-card relationship-review-panel" :aria-busy="isLoading">
      <header>
        <div>
          <span class="panel-kicker">{{ t('Identity Resolution') }}</span>
          <h2>{{ t('Unattached Traces') }}</h2>
        </div>
        <button type="button" :title="t('Reload identity traces')" @click="() => onReload()" :disabled="isLoading">
          <Icon icon="tabler:refresh" :size="15" />
        </button>
      </header>

      <div v-if="error" class="relationship-review-state error">
        <span>{{ error }}</span>
        <button type="button" @click="() => onReload()" :disabled="isLoading">{{ t('Retry') }}</button>
      </div>
      <div v-else-if="isLoading" class="relationship-review-state">
        <span>{{ t('Loading identity traces') }}</span>
      </div>
      <div v-else-if="pendingIdentityTraces.length === 0" class="relationship-review-state">
        <span>{{ t('No unattached identity traces') }}</span>
      </div>
      <div v-else class="relationship-review-list">
        <article v-for="trace in pendingIdentityTraces" :key="trace.id" class="relationship-review-item">
          <div>
            <strong>{{ formatIdentityTraceKind(trace.identity_type) }}</strong>
            <p>{{ formatIdentityTraceValue(trace) }}</p>
            <small>
              {{ t('Source') }}: {{ trace.source }}
              · {{ t('Confidence') }}: {{ identityTraceConfidence(trace) }}
            </small>
          </div>
          <div class="identity-trace-target">
            <select
              :value="targetPersonaId(trace)"
              :aria-label="t('Target Persona')"
              @change="(e) => { selectedPersonaByTrace[trace.id] = (e.target as HTMLSelectElement).value }"
            >
              <option v-for="person in persons" :key="person.person_id" :value="person.person_id">
                {{ personaLabel(person) }}
              </option>
            </select>
            <button
              type="button"
              :disabled="assigningIdentityTraceId === trace.id || persons.length === 0"
              @click="() => onAssign(trace, targetPersonaId(trace))"
            >
              <Icon icon="tabler:link" :size="14" /> {{ t('Assign') }}
            </button>
          </div>
        </article>
      </div>
    </section>
  </div>
</template>

<style scoped>
.relationship-review-panel {
  display: grid;
  gap: 10px;
}
.relationship-review-panel header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 10px;
}
.relationship-review-panel header button {
  width: 32px;
  padding: 0;
}
.relationship-review-state {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 10px;
  min-height: 42px;
  color: var(--hh-color-text-muted);
  font-size: 12px;
}
.relationship-review-state.error {
  color: var(--hh-color-danger);
}
.relationship-review-list {
  display: grid;
  gap: 9px;
}
.relationship-review-item {
  display: grid;
  gap: 8px;
  border-top: 1px solid rgba(102, 189, 180, 0.08);
  padding-top: 10px;
}
.relationship-review-item:first-child {
  border-top: none;
  padding-top: 0;
}
.relationship-review-item strong {
  display: block;
  margin-bottom: 3px;
  overflow-wrap: anywhere;
}
.relationship-review-item p,
.relationship-review-item small {
  display: block;
  margin: 0 0 4px;
  color: var(--hh-color-text-muted);
  font-size: 11px;
  line-height: 1.35;
  overflow-wrap: anywhere;
}
.identity-trace-target {
  display: grid;
  grid-template-columns: minmax(0, 1fr) auto;
  gap: 8px;
  align-items: center;
}
.identity-trace-target select {
  min-width: 0;
  height: 30px;
  border: 1px solid var(--hh-border-subtle);
  border-radius: var(--hh-radius-control);
  background: rgba(4, 21, 24, 0.72);
  color: var(--hh-color-text-soft);
  padding: 0 8px;
  font-size: 11px;
}
.identity-trace-target button {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  gap: 6px;
  min-height: 30px;
  border: 1px solid var(--hh-border-subtle);
  border-radius: var(--hh-radius-control);
  background: rgba(4, 21, 24, 0.72);
  color: var(--hh-color-text-soft);
  padding: 0 10px;
  font-size: 11px;
}
</style>
```

### `frontend/src/domains/personas/components/PersonsList.vue`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/domains/personas/components/PersonsList.vue`
- Size bytes / Размер в байтах: `3991`
- Included characters / Включено символов: `3991`
- Truncated / Обрезано: `no`

```text
<script setup lang="ts">
import { ref, computed } from 'vue'
import { useVirtualizer } from '@tanstack/vue-virtual'
import { useI18n } from '../../../platform/i18n'
import Icon from '../../../shared/ui/Icon.vue'
import type { PersonItem } from '../types/persona'

const { t } = useI18n()

const props = defineProps<{
  personList: PersonItem[]
  selectedPersonIndex: number
}>()

const emit = defineEmits<{
  selectPerson: [index: number]
}>()

const parentRef = ref<HTMLDivElement | null>(null)

const virtualOptions = computed(() => ({
  count: props.personList.length,
  getScrollElement: () => parentRef.value,
  estimateSize: () => 70,
  overscan: 5
}))

const virtualizer = useVirtualizer(virtualOptions)

const virtualItems = computed(() => virtualizer.value.getVirtualItems())
const totalSize = computed(() => virtualizer.value.getTotalSize())
</script>

<template>
  <div class="widget-frame" data-widget-id="persons-list">
    <section class="panel persons-list-panel">
      <header>
        <div>
          <h1>{{ t('Persons') }}</h1>
          <p>{{ personList.length }} {{ t('persons') }}</p>
        </div>
        <button type="button" class="primary-button" disabled>{{ t('New Person') }}</button>
      </header>
      <div class="filter-tabs compact">
        <button type="button" class="active">{{ t('All') }}</button>
        <button type="button" disabled>{{ t('People') }} <em>532</em></button>
        <button type="button" disabled>{{ t('Companies') }} <em>110</em></button>
      </div>
      <label class="local-search">
        <Icon icon="tabler:search" :size="17" />
        <input :placeholder="t('Search persons...')" />
      </label>
      <div ref="parentRef" class="persons-scroll-container">
        <div v-if="personList.length === 0" class="muted p-4">{{ t('No persons found') }}</div>
        <div v-else :style="{ height: `${totalSize}px` }">
          <button
            v-for="vitem in virtualItems"
            :key="personList[vitem.index].person_id"
            type="button"
            class="person-row"
            :class="{ active: selectedPersonIndex === vitem.index }"
            :style="{ transform: `translateY(${vitem.start}px)`, height: `${vitem.size}px` }"
            @click="emit('selectPerson', vitem.index)"
          >
            <span class="round-icon ghost"><Icon icon="tabler:user" :size="20" /></span>
            <span>
              <strong>{{ personList[vitem.index].name }}</strong>
              <small>{{ personList[vitem.index].role }}</small>
              <em>{{ personList[vitem.index].company }}</em>
            </span>
            <small>{{ personList[vitem.index].status ?? personList[vitem.index].channel ?? t('Email') }}</small>
          </button>
        </div>
      </div>
    </section>
  </div>
</template>

<style scoped>
.persons-scroll-container {
  flex: 1;
  overflow-y: auto;
}
.persons-list-panel {
  display: flex;
  flex-direction: column;
  padding: 12px;
  height: 100%;
}
.persons-list-panel header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: 12px;
}
.persons-list-panel .person-row {
  position: relative;
  display: grid;
  grid-template-columns: 44px 1fr auto;
  gap: 10px;
  align-items: center;
  width: 100%;
  min-height: var(--hh-widget-card-compact);
  border: 1px solid transparent;
  border-radius: var(--hh-radius-md);
  background: transparent;
  color: #e6f7f5;
  padding: 9px 10px;
  text-align: left;
  cursor: pointer;
}
.persons-list-panel .person-row.active {
  border-color: rgba(45, 240, 206, 0.24);
  background: rgba(25, 109, 100, 0.24);
}
.persons-list-panel .person-row strong {
  display: block;
  color: var(--hh-color-text-bright);
  font-size: 14px;
  font-weight: 560;
}
.persons-list-panel .person-row small,
.persons-list-panel .person-row em {
  display: block;
  margin-top: 5px;
  overflow: hidden;
  font-size: 11px;
  font-style: normal;
  text-overflow: ellipsis;
  white-space: nowrap;
}
</style>
```

### `frontend/src/domains/personas/components/PersonsRelationshipReview.vue`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/domains/personas/components/PersonsRelationshipReview.vue`
- Size bytes / Размер в байтах: `5036`
- Included characters / Включено символов: `5032`
- Truncated / Обрезано: `no`

```text
<script setup lang="ts">
import { useI18n } from '../../../platform/i18n'
import Icon from '../../../shared/ui/Icon.vue'
import { computed } from 'vue'
import { formatRelationshipType, formatRelationshipScore } from '../stores/personas'
import type { Relationship } from '../types/persona'

const { t } = useI18n()

const props = defineProps<{
  relationships: Relationship[]
  selectedPersonaId: string | null
  isLoading: boolean
  error: string
  reviewingRelationshipId: string | null
  onReload: () => Promise<void>
  onReview: (relationship: Relationship, reviewState: string) => Promise<void>
}>()

const suggestedRelationships = computed(() =>
  props.relationships.filter((r) => r.review_state === 'suggested')
)

function relationshipPeer(relationship: Relationship): string {
  const selId = props.selectedPersonaId
  if (selId && relationship.source_entity_id === selId) {
    return `${relationship.target_entity_kind}:${relationship.target_entity_id.slice(0, 8)}...`
  }
  if (selId && relationship.target_entity_id === selId) {
    return `${relationship.source_entity_kind}:${relationship.source_entity_id.slice(0, 8)}...`
  }
  return `${relationship.source_entity_kind}:${relationship.source_entity_id.slice(0, 8)}... → ${relationship.target_entity_kind}:${relationship.target_entity_id.slice(0, 8)}...`
}
</script>

<template>
  <div class="widget-frame" data-widget-id="persons-relationship-review">
    <section class="panel info-card relationship-review-panel" :aria-busy="isLoading">
      <header>
        <div>
          <span class="panel-kicker">{{ t('Relationships') }}</span>
          <h2>{{ t('Relationship Review') }}</h2>
        </div>
        <button type="button" :title="t('Reload relationships')" @click="() => onReload()" :disabled="isLoading">
          <Icon icon="tabler:refresh" :size="15" />
        </button>
      </header>

      <div v-if="error" class="relationship-review-state error">
        <span>{{ error }}</span>
        <button type="button" @click="() => onReload()" :disabled="isLoading">{{ t('Retry') }}</button>
      </div>
      <div v-else-if="isLoading" class="relationship-review-state">
        <span>{{ t('Loading relationships') }}</span>
      </div>
      <div v-else-if="suggestedRelationships.length === 0" class="relationship-review-state">
        <span>{{ t('No suggested relationships') }}</span>
      </div>
      <div v-else class="relationship-review-list">
        <article v-for="relationship in suggestedRelationships" :key="relationship.relationship_id" class="relationship-review-item">
          <div>
            <strong>{{ formatRelationshipType(relationship.relationship_type) }}</strong>
            <p>{{ relationshipPeer(relationship) }}</p>
            <small>
              {{ t('Trust') }}: {{ formatRelationshipScore(relationship.trust_score) }}
              · {{ t('Strength') }}: {{ formatRelationshipScore(relationship.strength_score) }}
              · {{ t('Confidence') }}: {{ formatRelationshipScore(relationship.confidence) }}
            </small>
          </div>
          <div class="relationship-review-actions">
            <button
              type="button"
              :disabled="reviewingRelationshipId === relationship.relationship_id"
              @click="() => onReview(relationship, 'user_confirmed')"
            >
              <Icon icon="tabler:check" :size="14" /> {{ t('Confirm') }}
            </button>
            <button
              type="button"
              :disabled="reviewingRelationshipId === relationship.relationship_id"
              @click="() => onReview(relationship, 'user_rejected')"
            >
              <Icon icon="tabler:x" :size="14" /> {{ t('Reject') }}
            </button>
          </div>
        </article>
      </div>
    </section>
  </div>
</template>

<style scoped>
.relationship-review-panel {
  display: grid;
  gap: 10px;
}
.relationship-review-panel header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 10px;
}
.relationship-review-panel header button {
  width: 32px;
  padding: 0;
}
.relationship-review-state {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 10px;
  min-height: 42px;
  color: var(--hh-color-text-muted);
  font-size: 12px;
}
.relationship-review-state.error {
  color: var(--hh-color-danger);
}
.relationship-review-list {
  display: grid;
  gap: 9px;
}
.relationship-review-item {
  display: grid;
  gap: 8px;
  border-top: 1px solid rgba(102, 189, 180, 0.08);
  padding-top: 10px;
}
.relationship-review-item:first-child {
  border-top: none;
  padding-top: 0;
}
.relationship-review-item strong {
  display: block;
  margin-bottom: 3px;
  overflow-wrap: anywhere;
}
.relationship-review-item p,
.relationship-review-item small {
  display: block;
  margin: 0 0 4px;
  color: var(--hh-color-text-muted);
  font-size: 11px;
  line-height: 1.35;
  overflow-wrap: anywhere;
}
.relationship-review-actions {
  display: flex;
  gap: 7px;
  flex-wrap: wrap;
}
</style>
```

### `frontend/src/domains/personas/views/PersonsPage.vue`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/domains/personas/views/PersonsPage.vue`
- Size bytes / Размер в байтах: `6465`
- Included characters / Включено символов: `6465`
- Truncated / Обрезано: `no`

```text
<script setup lang="ts">
import { computed } from 'vue'
import { useI18n } from '../../../platform/i18n'
import { usePersonsQuery, useIdentityCandidatesQuery, useIdentityTracesQuery, useRelationshipsQuery } from '../queries/usePersonasQuery'
import { usePersonasStore } from '../stores/personas'
import PersonsList from '../components/PersonsList.vue'
import PersonsDetail from '../components/PersonsDetail.vue'
import PersonsIdentityReview from '../components/PersonsIdentityReview.vue'
import PersonsIdentityTraceReview from '../components/PersonsIdentityTraceReview.vue'
import PersonsRelationshipReview from '../components/PersonsRelationshipReview.vue'
import type { PersonItem, PersonIdentityCandidate } from '../types/persona'

const { t } = useI18n()
const store = usePersonasStore()

const { data: personsData } = usePersonsQuery()
const { data: identityCandidatesData } = useIdentityCandidatesQuery()
const { data: identityTracesData } = useIdentityTracesQuery()
const { data: relationshipsData } = useRelationshipsQuery()

const personList = computed<PersonItem[]>(() => {
  return (personsData.value ?? []).map((p) => ({
    person_id: p.person_id,
    name: p.display_name,
    role: p.preferred_channel || t('Contact'),
    company: p.email_address,
    status: p.last_interaction_at ? t('Online') : undefined,
    channel: p.preferred_channel ?? undefined
  }))
})

const selectedPerson = computed(() =>
  personList.value[store.selectedPersonIndex] ?? personList.value[0] ?? null
)

const suggestedIdentityCandidates = computed(() =>
  (identityCandidatesData.value ?? []).filter(
    (item: PersonIdentityCandidate) => item.review_state === 'suggested'
  )
)

const confirmedMergeIdentityCandidates = computed(() =>
  (identityCandidatesData.value ?? []).filter(
    (item: PersonIdentityCandidate) =>
      item.candidate_kind === 'merge_persons' &&
      item.review_state === 'user_confirmed'
  )
)

function identityConfidence(item: PersonIdentityCandidate): string {
  return `${Math.round(item.confidence * 100)}%`
}

async function setIdentityCandidateReview(candidate: PersonIdentityCandidate, state: string) {
  await store.reviewCandidate(candidate, state as any)
}

async function splitConfirmedIdentityMerge(candidate: PersonIdentityCandidate) {
  // Simplified: just mark back to suggested to re-evaluate
  await store.reviewCandidate(candidate, 'suggested' as any)
}

function splitCandidateForConfirmedMerge(candidate: PersonIdentityCandidate): PersonIdentityCandidate | null {
  return null
}

async function loadRelationships() {
  // Relationships loaded via TanStack Query
}

async function loadTraces() {
  // Traces loaded via TanStack Query
}
</script>

<template>
  <section class="persons-page">
    <div class="persons-layout">
      <PersonsList
        :personList="personList"
        :selectedPersonIndex="store.selectedPersonIndex"
        @selectPerson="store.selectPerson"
      />
      <PersonsDetail
        :selectedPerson="selectedPerson"
        :personDossier="store.personDossier"
        :isPersonDossierLoading="store.isPersonDossierLoading"
        :personDossierError="store.personDossierError"
        :whatsNew="[]"
        :projects="[]"
      />
      <aside class="stacked-rail">
        <div class="widget-frame" data-widget-id="persons-ai-summary">
          <section class="panel info-card">
            <h2>{{ t('AI Summary') }}</h2>
            <p>{{ t('John is a key strategic partner and decision maker. You have a strong professional relationship with frequent communication across multiple projects.') }}</p>
          </section>
        </div>
        <PersonsIdentityReview
          :suggestedIdentityCandidates="suggestedIdentityCandidates"
          :confirmedMergeIdentityCandidates="confirmedMergeIdentityCandidates"
          :isIdentityCandidatesLoading="false"
          :identityCandidatesError="store.identityCandidatesError"
          :identityConfidence="identityConfidence"
          :setIdentityCandidateReview="setIdentityCandidateReview"
          :splitConfirmedIdentityMerge="splitConfirmedIdentityMerge"
          :splitCandidateForConfirmedMerge="splitCandidateForConfirmedMerge"
        />
        <PersonsIdentityTraceReview
          :identityTraces="identityTracesData ?? []"
          :persons="personList"
          :selectedPersonaId="selectedPerson?.person_id ?? null"
          :isLoading="false"
          :error="store.identityTracesError"
          :assigningIdentityTraceId="store.assigningIdentityTraceId"
          :onReload="loadTraces"
          :onAssign="store.assignTraceToPersona"
        />
        <PersonsRelationshipReview
          :relationships="relationshipsData ?? []"
          :selectedPersonaId="selectedPerson?.person_id ?? null"
          :isLoading="false"
          :error="store.relationshipsError"
          :reviewingRelationshipId="store.reviewingRelationshipId"
          :onReload="loadRelationships"
          :onReview="store.reviewRelation"
        />
        <div class="widget-frame" data-widget-id="persons-related-documents">
          <section class="panel info-card">
            <h2>{{ t('Related Documents') }}</h2>
            <p>{{ t('Documents will appear here when processing is complete.') }}</p>
          </section>
        </div>
        <div class="widget-frame" data-widget-id="persons-recent-notes">
          <section class="panel info-card">
            <h2>{{ t('Recent Notes') }}</h2>
            <p>{{ t('Discussed expansion to EU market') }}</p>
            <p>{{ t('Prefers email for official communication') }}</p>
            <p>{{ t('Interested in AI/ML integration') }}</p>
          </section>
        </div>
      </aside>
    </div>
  </section>
</template>

<style scoped>
.persons-page {
  display: grid;
  grid-template-columns: repeat(var(--hh-layout-columns), minmax(0, 1fr));
  grid-auto-flow: row;
  grid-auto-rows: min-content;
  align-content: start;
  gap: var(--hh-layout-gap);
  height: 100%;
  min-height: 0;
  overflow: hidden;
  padding-right: 0;
}
.persons-page > * {
  grid-column: 1 / -1;
  min-width: 0;
}
.persons-layout {
  --hh-zone-rows: 12;
  display: grid;
  grid-template-columns: repeat(var(--hh-layout-columns), minmax(0, 1fr));
  grid-auto-flow: dense;
  grid-auto-rows: min-content;
  align-content: start;
  align-items: stretch;
  gap: var(--hh-layout-gap);
  width: 100%;
  min-width: 0;
  min-height: 0;
  max-height: 100%;
  overflow: hidden;
}
</style>
```

### `frontend/src/domains/projects/components/ProjectsDashboard.vue`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/domains/projects/components/ProjectsDashboard.vue`
- Size bytes / Размер в байтах: `5454`
- Included characters / Включено символов: `5453`
- Truncated / Обрезано: `no`

```text
<script setup lang="ts">
import { useI18n } from '../../../platform/i18n'
import Icon from '../../../shared/ui/Icon.vue'
import type { ProjectDetail, ProjectRecord, ProjectStats, ProjectTimelineItem, ProjectMessageSummary, ProjectDocumentSummary } from '../types/project'
import { projectTimelineIcon, projectDocumentIcon, formatProjectDateTime } from '../stores/projects'

const { t } = useI18n()

const props = defineProps<{
  selectedProjectDetail: ProjectDetail | null
  selectedProjectRecord: ProjectRecord
  selectedProjectStats: ProjectStats
  formatNumber: (num: number) => string
}>()

function projectMessageSender(message: ProjectMessageSummary): string {
  return message.sender || t('Unknown')
}
</script>

<template>
  <!-- Project Summary -->
  <div class="widget-frame">
    <section class="panel info-card">
      <h2>{{ t('Project Summary') }}</h2>
      <div class="summary-numbers">
        <article><strong>{{ props.formatNumber(props.selectedProjectStats.document_count) }}</strong><span>{{ t('Documents') }}</span></article>
        <article><strong>{{ props.formatNumber(props.selectedProjectStats.message_count) }}</strong><span>{{ t('Messages') }}</span></article>
        <article><strong>{{ props.formatNumber(props.selectedProjectStats.people_count) }}</strong><span>{{ t('People') }}</span></article>
        <article><strong>{{ props.formatNumber(props.selectedProjectStats.graph_connection_count) }}</strong><span>{{ t('Graph links') }}</span></article>
      </div>
    </section>
  </div>

  <!-- Knowledge Graph -->
  <div class="widget-frame">
    <section class="panel graph-card-large">
      <h2>{{ t('Knowledge Graph') }}</h2>
      <div class="radial-graph">
        <div class="graph-center"><Icon icon="tabler:cube" :size="30" /><span>{{ props.selectedProjectRecord.name }}</span></div>
        <span class="graph-chip graph-chip-messages">{{ t('Messages') }} {{ props.formatNumber(props.selectedProjectStats.message_count) }}</span>
        <span class="graph-chip graph-chip-documents">{{ t('Documents') }} {{ props.formatNumber(props.selectedProjectStats.document_count) }}</span>
        <span class="graph-chip graph-chip-people">{{ t('People') }} {{ props.formatNumber(props.selectedProjectStats.people_count) }}</span>
        <span class="graph-chip graph-chip-links">{{ t('Links') }} {{ props.formatNumber(props.selectedProjectStats.graph_connection_count) }}</span>
      </div>
    </section>
  </div>

  <!-- Project Timeline -->
  <div class="widget-frame">
    <section class="panel info-card">
      <h2>{{ t('Project Timeline') }}</h2>
      <template v-if="props.selectedProjectDetail?.timeline.length">
        <div v-for="item in props.selectedProjectDetail.timeline" :key="item.item_id" class="timeline-mini">
          <Icon :icon="projectTimelineIcon(item.item_kind)" :size="16" />
          <time>{{ formatProjectDateTime(item.occurred_at) }}</time>
          <strong>{{ item.title }}</strong>
        </div>
      </template>
      <p v-else class="muted-copy">{{ t('No timeline items from local sources.') }}</p>
    </section>
  </div>

  <!-- Recent Communications -->
  <div class="widget-frame">
    <section class="panel info-card">
      <h2>{{ t('Recent Communications') }}</h2>
      <template v-if="props.selectedProjectDetail?.recent_messages.length">
        <div v-for="message in props.selectedProjectDetail.recent_messages" :key="message.message_id" class="related-row">
          <span class="round-icon cyan"><Icon icon="tabler:mail" :size="16" /></span>
          <strong>{{ projectMessageSender(message) }}</strong>
          <em>{{ formatProjectDateTime(message.occurred_at) }}</em>
        </div>
      </template>
      <p v-else class="muted-copy">{{ t('No linked communications.') }}</p>
    </section>
  </div>

  <!-- Top Documents -->
  <div class="widget-frame">
    <section class="panel info-card">
      <h2>{{ t('Top Documents') }}</h2>
      <template v-if="props.selectedProjectDetail?.documents.length">
        <div v-for="document in props.selectedProjectDetail.documents" :key="document.document_id" class="doc-mini">
          <Icon :icon="projectDocumentIcon(document.document_kind)" :size="20" />
          <span><strong>{{ document.title }}</strong><small>{{ document.document_kind }} · {{ formatProjectDateTime(document.imported_at) }}</small></span>
        </div>
      </template>
      <p v-else class="muted-copy">{{ t('No linked documents.') }}</p>
    </section>
  </div>

  <!-- Source Evidence -->
  <div class="widget-frame">
    <section class="panel info-card">
      <h2>{{ t('Source Evidence') }}</h2>
      <div class="summary-numbers compact">
        <article><strong>{{ props.formatNumber(props.selectedProjectStats.message_count + props.selectedProjectStats.document_count) }}</strong><span>{{ t('Matched records') }}</span></article>
        <article><strong>{{ formatProjectDateTime(props.selectedProjectStats.latest_activity_at) }}</strong><span>{{ t('Last activity') }}</span></article>
      </div>
    </section>
  </div>

  <!-- Open Promises -->
  <div class="widget-frame">
    <section class="panel info-card">
      <h2>{{ t('Open Promises') }}</h2>
      <p class="muted-copy">{{ t('No task candidates connected to this project.') }}</p>
      <button type="button" class="link-row" disabled>{{ t('View all promises') }} <Icon icon="tabler:arrow-right" :size="15" /></button>
    </section>
  </div>
</template>
```

### `frontend/src/domains/projects/components/ProjectsHero.vue`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/domains/projects/components/ProjectsHero.vue`
- Size bytes / Размер в байтах: `4636`
- Included characters / Включено символов: `4632`
- Truncated / Обрезано: `no`

```text
<script setup lang="ts">
import { useI18n } from '../../../platform/i18n'
import Icon from '../../../shared/ui/Icon.vue'
import type { ProjectRecord, ProjectStats, ProjectSummary } from '../types/project'
import { projectStatusLabel, formatProjectDate } from '../stores/projects'

const { t } = useI18n()

const props = defineProps<{
  projectsError: string
  isProjectsLoading: boolean
  selectedProjectRecord: ProjectRecord | null
  selectedProjectStats: ProjectStats
  projectSummaries: ProjectSummary[]
  selectProject: (item: ProjectSummary) => void
  loadProjects: () => void
}>()
</script>

<template>
  <!-- Error state -->
  <div v-if="props.projectsError && !props.selectedProjectRecord" class="widget-frame">
    <section class="panel info-card project-empty-state">
      <span class="icon-placeholder">⚠</span>
      <h2>{{ t('Projects unavailable') }}</h2>
      <p>{{ props.projectsError }}</p>
      <button type="button" class="primary-button" @click="props.loadProjects">{{ t('Retry') }}</button>
    </section>
  </div>

  <!-- Empty state -->
  <div v-else-if="!props.selectedProjectRecord" class="widget-frame">
    <section class="panel info-card project-empty-state">
      <span class="icon-placeholder">◻</span>
      <h2>{{ t('No projects returned') }}</h2>
      <p>{{ props.isProjectsLoading ? t('Loading local projects...') : t('Local project records are empty.') }}</p>
    </section>
  </div>

  <!-- Projects loaded -->
  <template v-else>
    <div class="widget-frame">
      <header class="project-hero panel">
        <div class="project-logo"><Icon icon="tabler:cube" :size="48" /></div>
        <div>
          <h1>{{ props.selectedProjectRecord.name }} <em>{{ projectStatusLabel(props.selectedProjectRecord.status) }}</em></h1>
          <p>{{ props.selectedProjectRecord.kind }}</p>
          <small>{{ props.selectedProjectRecord.description }}</small>
        </div>
        <button type="button" class="primary-button" disabled>
          <Icon icon="tabler:calendar-stats" :size="16" /> {{ t('Prepare brief') }}
        </button>
      </header>
    </div>

    <div class="widget-frame">
      <div class="project-meta-strip panel">
        <article><span>{{ t('Owner') }}</span><strong>{{ props.selectedProjectRecord.owner_display_name }}</strong></article>
        <article><span>{{ t('People') }}</span><strong>{{ props.selectedProjectStats.people_count }}</strong></article>
        <article><span>{{ t('Start Date') }}</span><strong>{{ formatProjectDate(props.selectedProjectRecord.start_date) }}</strong></article>
        <article><span>{{ t('Target Date') }}</span><strong>{{ formatProjectDate(props.selectedProjectRecord.target_date) }}</strong></article>
        <article>
          <span>{{ t('Progress') }}</span>
          <progress class="progress" :max="100" :value="props.selectedProjectRecord.progress_percent" :aria-label="`${props.selectedProjectRecord.name} progress`" />
          <strong>{{ props.selectedProjectRecord.progress_percent }}%</strong>
        </article>
      </div>
    </div>

    <div v-if="props.projectSummaries.length > 1" class="widget-frame">
      <div class="project-switcher panel">
        <button
          v-for="item in props.projectSummaries"
          :key="item.project.project_id"
          type="button"
          :class="{ active: item.project.project_id === props.selectedProjectRecord.project_id }"
          @click="props.selectProject(item)"
        >
          <Icon icon="tabler:cube" :size="16" />
          <span>{{ item.project.name }}</span>
          <em>{{ item.project.progress_percent }}%</em>
        </button>
      </div>
    </div>

    <div class="widget-frame">
      <div class="section-tabs">
        <button type="button" class="active">{{ t('Overview') }}</button>
        <button type="button" disabled>{{ t('Communications') }} <em>{{ props.selectedProjectStats.message_count }}</em></button>
        <button type="button" disabled>{{ t('Tasks') }}</button>
        <button type="button" disabled>{{ t('Documents') }} <em>{{ props.selectedProjectStats.document_count }}</em></button>
        <button type="button" disabled>{{ t('Calendar') }}</button>
        <button type="button" disabled>{{ t('Team') }} <em>{{ props.selectedProjectStats.people_count }}</em></button>
        <button type="button" disabled>{{ t('Notes') }}</button>
        <button type="button" disabled>{{ t('Files') }}</button>
        <button type="button" disabled>{{ t('Settings') }}</button>
      </div>
    </div>

    <p v-if="props.projectsError" class="inline-error">{{ props.projectsError }}</p>
  </template>
</template>
```

### `frontend/src/domains/projects/components/ProjectsRail.vue`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/domains/projects/components/ProjectsRail.vue`
- Size bytes / Размер в байтах: `2667`
- Included characters / Включено символов: `2667`
- Truncated / Обрезано: `no`

```text
<script setup lang="ts">
import { useI18n } from '../../../platform/i18n'
import Icon from '../../../shared/ui/Icon.vue'
import type { ProjectDetail, ProjectRecord, ProjectStats, ProjectSummary } from '../types/project'
import { projectStatusLabel } from '../stores/projects'

const { t } = useI18n()

const props = defineProps<{
  selectedProjectDetail: ProjectDetail | null
  selectedProjectRecord: ProjectRecord
  selectedProjectStats: ProjectStats
  relatedProjectSummaries: ProjectSummary[]
  formatNumber: (num: number) => string
}>()
</script>

<template>
  <aside class="stacked-rail project-side">
    <!-- Project Health -->
    <div class="widget-frame">
      <section class="panel info-card">
        <h2>{{ t('Project Health') }}</h2>
        <div class="health-row"><span>{{ t('Status') }}</span><strong>{{ projectStatusLabel(props.selectedProjectRecord.status) }}</strong></div>
        <div class="health-row"><span>{{ t('Progress') }}</span><strong>{{ props.selectedProjectRecord.progress_percent }}%</strong></div>
        <div class="health-row"><span>{{ t('Graph Links') }}</span><strong>{{ props.formatNumber(props.selectedProjectStats.graph_connection_count) }}</strong></div>
      </section>
    </div>

    <!-- Key People -->
    <div class="widget-frame">
      <section class="panel info-card">
        <h2>{{ t('Key People') }}</h2>
        <template v-if="props.selectedProjectDetail?.key_people.length">
          <div v-for="person in props.selectedProjectDetail.key_people" :key="person.email_address" class="person-compact">
            <span class="round-icon ghost"><Icon icon="tabler:user" :size="16" /></span>
            <span><strong>{{ person.display_name }}</strong><small>{{ person.email_address }}</small></span>
            <em>{{ props.formatNumber(person.interaction_count) }}</em>
          </div>
        </template>
        <p v-else class="muted-copy">{{ t('No linked people.') }}</p>
      </section>
    </div>

    <!-- Related Projects -->
    <div class="widget-frame">
      <section class="panel info-card">
        <h2>{{ t('Related Projects') }}</h2>
        <template v-if="props.relatedProjectSummaries.length">
          <div v-for="item in props.relatedProjectSummaries.slice(0, 4)" :key="item.project.project_id" class="related-row">
            <span class="round-icon cyan"><Icon icon="tabler:cube" :size="16" /></span>
            <strong>{{ item.project.name }}</strong>
            <em>{{ item.project.progress_percent }}%</em>
          </div>
        </template>
        <p v-else class="muted-copy">{{ t('No related project records.') }}</p>
      </section>
    </div>
  </aside>
</template>
```

### `frontend/src/domains/projects/views/ProjectsPage.vue`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/domains/projects/views/ProjectsPage.vue`
- Size bytes / Размер в байтах: `3134`
- Included characters / Включено символов: `3134`
- Truncated / Обрезано: `no`

```text
<script setup lang="ts">
import { computed } from 'vue'
import { useI18n } from '../../../platform/i18n'
import { useProjectsQuery, useProjectQuery } from '../queries/useProjectsQuery'
import { useProjectsStore } from '../stores/projects'
import ProjectsHero from '../components/ProjectsHero.vue'
import ProjectsDashboard from '../components/ProjectsDashboard.vue'
import ProjectsRail from '../components/ProjectsRail.vue'
import type { ProjectSummary, ProjectDetail } from '../types/project'

const { t } = useI18n()
const store = useProjectsStore()

const { data: projectsData, isLoading: isProjectsLoading, error: projectsErrorObj, refetch: refetchProjects } = useProjectsQuery()
const { data: projectDetailData, isLoading: isDetailLoading } = useProjectQuery(store.selectedProjectId || null)

const projectsError = computed<string>(() => {
  if (projectsErrorObj.value) return projectsErrorObj.value?.message ?? t('Unknown projects error')
  return ''
})

const projectSummaries = computed<ProjectSummary[]>(() => {
  return projectsData.value ?? []
})

const selectedProjectDetail = computed<ProjectDetail | null>(() => {
  return projectDetailData.value ?? null
})

const selectedProjectRecord = computed(() => {
  return selectedProjectDetail.value?.project ?? projectSummaries.value[0]?.project ?? null
})

const selectedProjectStats = computed(() => {
  return selectedProjectDetail.value?.stats ?? projectSummaries.value[0]?.stats ?? { message_count: 0, document_count: 0, people_count: 0, graph_connection_count: 0, latest_activity_at: null }
})

const relatedProjectSummaries = computed<ProjectSummary[]>(() => {
  const currentId = selectedProjectRecord.value?.project_id
  return projectSummaries.value.filter((item) => item.project.project_id !== currentId)
})

function selectProject(item: ProjectSummary) {
  if (item.project.project_id === store.selectedProjectId && selectedProjectDetail.value) return
  store.selectProject(item.project.project_id)
}

function loadProjects() {
  refetchProjects()
}

function formatNumber(value: number): string {
  return new Intl.NumberFormat('en-US').format(value)
}
</script>

<template>
  <section class="projects-page">
    <ProjectsHero
      :projectsError="projectsError"
      :isProjectsLoading="isProjectsLoading"
      :selectedProjectRecord="selectedProjectRecord"
      :selectedProjectStats="selectedProjectStats"
      :projectSummaries="projectSummaries"
      :selectProject="selectProject"
      :loadProjects="loadProjects"
    />

    <div v-if="selectedProjectRecord" class="project-dashboard-grid">
      <ProjectsDashboard
        :selectedProjectDetail="selectedProjectDetail"
        :selectedProjectRecord="selectedProjectRecord"
        :selectedProjectStats="selectedProjectStats"
        :formatNumber="formatNumber"
      />
      <ProjectsRail
        :selectedProjectDetail="selectedProjectDetail"
        :selectedProjectRecord="selectedProjectRecord"
        :selectedProjectStats="selectedProjectStats"
        :relatedProjectSummaries="relatedProjectSummaries"
        :formatNumber="formatNumber"
      />
    </div>
  </section>
</template>
```

### `frontend/src/domains/review/views/ReviewPage.vue`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/domains/review/views/ReviewPage.vue`
- Size bytes / Размер в байтах: `20723`
- Included characters / Включено символов: `12000`
- Truncated / Обрезано: `yes`

```text
<script setup lang="ts">
import { onMounted, ref } from 'vue'
import { useReviewStore } from '../stores/review'
import Icon from '../../../shared/ui/Icon.vue'
import type {
	Decision,
	Obligation,
	Relationship,
	ReviewItem,
	ReviewItemPromotionRequest
} from '../types/review'

const store = useReviewStore()
const promoteDrafts = ref<Record<string, ReviewItemPromotionRequest>>({})

onMounted(() => {
	void loadReviewWorkspace()
})

async function loadReviewWorkspace() {
	await store.loadAll()
	syncPromoteDrafts()
}

function relationshipPeer(item: Relationship): string {
	return `${item.source_entity_kind}:${item.source_entity_id} → ${item.target_entity_kind}:${item.target_entity_id}`
}

function decisionEntityLabel(item: Decision): string {
	if (!item.decided_by_entity_kind || !item.decided_by_entity_id) return 'Unknown'
	return `${item.decided_by_entity_kind}:${item.decided_by_entity_id}`
}

function obligationEntityLabel(item: Obligation): string {
	return `${item.obligated_entity_kind}:${item.obligated_entity_id}`
}

function formatItemTime(value: string | null | undefined): string {
	if (!value) return ''
	const date = new Date(value)
	if (Number.isNaN(date.getTime())) return ''
	return new Intl.DateTimeFormat('en', {
		month: 'short',
		day: 'numeric',
		hour: '2-digit',
		minute: '2-digit'
	}).format(date)
}

async function handleReview(
	action: import('../types/review').ReviewWorkspaceItemAction
) {
	await store.reviewItem(action)
}

async function syncPromoteDrafts() {
	store.reviewItems.forEach((item: ReviewItem) => {
		if (promoteDrafts.value[item.review_item_id]) return
		promoteDrafts.value[item.review_item_id] = deriveDefaultPromotion(item)
	})
}

function deriveDefaultPromotion(item: ReviewItem): ReviewItemPromotionRequest {
	const defaults: Record<string, ReviewItemPromotionRequest> = {
		new_person: { target_domain: 'persons', target_entity_kind: 'person', target_entity_id: '' },
		new_organization: {
			target_domain: 'organizations',
			target_entity_kind: 'organization',
			target_entity_id: ''
		},
		potential_task: { target_domain: 'tasks', target_entity_kind: 'task', target_entity_id: '' },
		potential_obligation: {
			target_domain: 'obligations',
			target_entity_kind: 'obligation',
			target_entity_id: ''
		},
		potential_decision: {
			target_domain: 'decisions',
			target_entity_kind: 'decision',
			target_entity_id: ''
		},
		potential_relationship: {
			target_domain: 'relationships',
			target_entity_kind: 'relationship',
			target_entity_id: ''
		},
		potential_project: {
			target_domain: 'projects',
			target_entity_kind: 'project',
			target_entity_id: ''
		},
		knowledge_candidate: {
			target_domain: 'documents',
			target_entity_kind: 'document',
			target_entity_id: `document:review-note:${item.review_item_id}`
		}
	}
	return defaults[item.item_kind] ?? { target_domain: '', target_entity_kind: '', target_entity_id: '' }
}

function canPromote(item: ReviewItem): boolean {
	const draft = promoteDrafts.value[item.review_item_id]
	return !!(
		draft &&
		draft.target_domain.trim() &&
		draft.target_entity_kind.trim() &&
		draft.target_entity_id.trim() &&
		store.reviewingItemKey !== `review_item_promote:${item.review_item_id}`
	)
}

async function handlePromote(item: ReviewItem) {
	const draft = promoteDrafts.value[item.review_item_id]
	if (!draft) return
	await handleReview({
		kind: 'review_item_promote',
		item,
		promotion: { ...draft }
	})
}

function reviewItemButtonPrefix(item: ReviewItem): string {
	return `review_item:${item.review_item_id}`
}

function canArchive(item: ReviewItem): boolean {
	return store.reviewingItemKey !== `review_item_archive:${item.review_item_id}`
}

function reviewItemKindLabel(itemKind: ReviewItem['item_kind']): string {
	return itemKind
}
</script>

<template>
  <div class="review-page">
    <!-- Header -->
    <div class="view-header">
      <div class="header-title-group">
        <h2 class="view-title">Review Workspace</h2>
        <p class="view-subtitle">Review and confirm suggested items</p>
      </div>
        <button type="button" class="ghost-button" @click="loadReviewWorkspace()">
        <Icon icon="tabler:refresh" />
        Refresh
      </button>
    </div>

    <!-- Error banner -->
    <div v-if="store.error" class="error-banner">
      <Icon icon="tabler:alert-circle" />
      <span>{{ store.error }}</span>
    </div>

    <!-- Metrics -->
    <div class="review-metrics">
      <div class="metric-card">
        <span class="metric-value">{{ store.reviewItemsCount }}</span>
        <span class="metric-label">Review Items</span>
      </div>
      <div class="metric-card">
        <span class="metric-value">{{ store.totalSuggestedCount }}</span>
        <span class="metric-label">Suggested</span>
      </div>
      <div class="metric-card">
        <span class="metric-value">{{ store.relationsSuggestedCount }}</span>
        <span class="metric-label">Relationships</span>
      </div>
      <div class="metric-card">
        <span class="metric-value">{{ store.decisionsSuggestedCount }}</span>
        <span class="metric-label">Decisions</span>
      </div>
      <div class="metric-card">
        <span class="metric-value">{{ store.obligationsSuggestedCount }}</span>
        <span class="metric-label">Obligations</span>
      </div>
      <div class="metric-card">
        <span class="metric-value">{{ store.contradictionsSuggestedCount }}</span>
        <span class="metric-label">Polygraph</span>
      </div>
    </div>

    <!-- Review Board -->
    <div class="review-board">
      <!-- Canonical Review Items -->
      <div class="review-panel">
        <h3 class="panel-title">
          <Icon icon="tabler:inbox" />
          Canonical Inbox
          <span v-if="store.reviewItemsCount > 0" class="panel-badge">
            {{ store.reviewItemsCount }}
          </span>
        </h3>
        <div v-if="store.reviewItems.length === 0" class="panel-empty">
          <p>No canonical review items available</p>
        </div>
        <div v-else class="panel-items">
          <div
            v-for="item in store.reviewItems.filter((r) => r.status === 'new' || r.status === 'in_review')"
            :key="item.review_item_id"
            class="review-item review-item--canonical"
          >
            <div class="item-info">
              <p class="item-desc">{{ item.title }}</p>
              <p class="item-meta">
                {{ reviewItemKindLabel(item.item_kind) }} · {{ item.summary }}
              </p>
              <p class="item-meta">Confidence: {{ item.confidence.toFixed(2) }}</p>
            </div>
            <div class="item-actions item-actions--stacked">
              <div class="action-row">
                <button
                  type="button"
                  class="action-btn confirm"
                  :disabled="store.reviewingItemKey === reviewItemButtonPrefix(item)"
                  @click="handleReview({ kind: 'review_item', item, action: 'approve' })"
                >
				<Icon icon="tabler:check" /> Approve
                </button>
                <button
                  v-if="item.status === 'new'"
                  type="button"
                  class="action-btn"
                  :disabled="store.reviewingItemKey === reviewItemButtonPrefix(item)"
                  @click="handleReview({ kind: 'review_item_take', item })"
                >
                  <Icon icon="tabler:player-play" /> Take
                </button>
                <span v-else class="status-pill">
                  status: {{ item.status }}
                </span>
                <button
                  type="button"
                  class="action-btn reject"
                  :disabled="store.reviewingItemKey === reviewItemButtonPrefix(item)"
                  @click="handleReview({ kind: 'review_item', item, action: 'dismiss' })"
                >
                  <Icon icon="tabler:x" /> Dismiss
                </button>
              </div>
              <div class="action-row">
                <input
                  v-model="promoteDrafts[item.review_item_id].target_domain"
                  type="text"
                  class="review-input"
                  placeholder="target_domain"
                />
                <input
                  v-model="promoteDrafts[item.review_item_id].target_entity_kind"
                  type="text"
                  class="review-input"
                  placeholder="entity_kind"
                />
                <input
                  v-model="promoteDrafts[item.review_item_id].target_entity_id"
                  type="text"
                  class="review-input"
                  placeholder="entity_id"
                />
              </div>
              <div class="action-row">
                <button
                  type="button"
                  class="action-btn promote"
                  :disabled="!canPromote(item)"
                  @click="handlePromote(item)"
                >
                  <Icon icon="tabler:arrow-up-right" /> Promote
                </button>
                <button
                  type="button"
                  class="action-btn archive"
                  :disabled="!canArchive(item)"
                  @click="handleReview({ kind: 'review_item_archive', item })"
                >
                  <Icon icon="tabler:archive" /> Archive
                </button>
                <span v-if="item.target_domain" class="status-pill">
                  promoted: {{ item.target_domain }}/{{ item.target_entity_kind }}/{{ item.target_entity_id }}
                </span>
              </div>
            </div>
          </div>
        </div>
      </div>

      <!-- Relationships -->
      <div class="review-panel">
        <h3 class="panel-title">
          <Icon icon="tabler:users" />
          Relationships
          <span v-if="store.relationsSuggestedCount > 0" class="panel-badge">
            {{ store.relationsSuggestedCount }}
          </span>
        </h3>
        <div v-if="store.relationships.length === 0" class="panel-empty">
          <p>No relationships to review</p>
        </div>
        <div v-else class="panel-items">
          <div
            v-for="item in store.relationships.filter((r) => r.review_state === 'suggested')"
            :key="item.relationship_id"
            class="review-item"
          >
            <div class="item-info">
              <p class="item-desc">{{ relationshipPeer(item) }}</p>
              <p class="item-meta">{{ item.relationship_type }} · Score: {{ item.trust_score?.toFixed(1) }}</p>
            </div>
            <div class="item-actions">
              <button
                type="button"
                class="action-btn confirm"
                :disabled="store.reviewingItemKey === 'relationship:' + item.relationship_id"
                @click="handleReview({ kind: 'relationship', item, reviewState: 'user_confirmed' })"
              >
                <Icon icon="tabler:check" /> Confirm
              </button>
              <button
                type="button"
                class="action-btn reject"
                :disabled="store.reviewingItemKey === 'relationship:' + item.relationship_id"
                @click="handleReview({ kind: 'relationship', item, reviewState: 'user_rejected' })"
              >
                <Icon icon="tabler:x" /> Reject
              </button>
            </div>
          </div>
        </div>
      </div>

      <!-- Decisions -->
      <div class="review-panel">
        <h3 class="panel-title">
          <Icon icon="tabler:scale" />
          Decisions
          <span v-if="store.decisionsSuggestedCount > 0" class="panel-badge">
            {{ store.decisionsSuggestedCount }}
          </span>
        </h3>
        <div v-if="store.decisions.length === 0" class="panel-empty">
          <p>No decisions to review</p>
        </div>
        <div v-else class="panel-items">
          <div

```
_Source file truncated after 12000 characters. / Исходный файл обрезан после 12000 символов._

### `frontend/src/domains/settings/components/AISettingsControlCenter.vue`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/domains/settings/components/AISettingsControlCenter.vue`
- Size bytes / Размер в байтах: `5601`
- Included characters / Включено символов: `5601`
- Truncated / Обрезано: `no`

```text
<script setup lang="ts">
import { ref } from 'vue'
import { useI18n } from '../../../platform/i18n'

const { t } = useI18n()

type AiTab = 'overview' | 'api-providers' | 'model-routing' | 'prompt-studio' | 'runs-health'

const activeTab = ref<AiTab>('overview')

const tabs: Array<{ id: AiTab; label: string }> = [
  { id: 'overview', label: 'Overview' },
  { id: 'api-providers', label: 'API Providers' },
  { id: 'model-routing', label: 'Model Routing' },
  { id: 'prompt-studio', label: 'Prompt Studio' },
  { id: 'runs-health', label: 'Runs Health' }
]

const statusCards = [
  { label: 'Ollama', status: 'connected', detail: 'v0.5.0' },
  { label: 'Default Model', status: 'configured', detail: 'llama3.2:3b' },
  { label: 'Embeddings', status: 'active', detail: 'nomic-embed-text' },
  { label: 'Last Run', status: 'idle', detail: '2 mins ago' }
]
</script>

<template>
  <div class="settings-page">
    <section class="panel settings-list-panel settings-primary-pane">
      <header class="panel-title-row">
        <div>
          <h2>{{ t('AI Control Center') }}</h2>
          <p>{{ t('Configure AI providers, model routing, prompts and runs.') }}</p>
        </div>
      </header>

      <!-- Tabs -->
      <div class="ai-tabs">
        <button
          v-for="tab in tabs"
          :key="tab.id"
          type="button"
          class="ai-tab-btn"
          :class="{ active: activeTab === tab.id }"
          @click="activeTab = tab.id"
        >
          {{ t(tab.label) }}
        </button>
      </div>

      <!-- Overview tab -->
      <div v-if="activeTab === 'overview'" class="ai-tab-content">
        <div class="ai-status-grid">
          <div
            v-for="card in statusCards"
            :key="card.label"
            class="ai-status-card panel"
          >
            <header>
              <span class="ai-status-dot" :class="card.status" />
              <strong>{{ t(card.label) }}</strong>
            </header>
            <p>{{ card.detail }}</p>
          </div>
        </div>

        <section class="ai-section">
          <h3>{{ t('Quick Actions') }}</h3>
          <div class="ai-quick-actions">
            <button type="button" class="makosh-btn makosh-btn--outline" disabled>
              {{ t('Test Connection') }}
            </button>
            <button type="button" class="makosh-btn makosh-btn--outline" disabled>
              {{ t('Run Diagnostics') }}
            </button>
            <button type="button" class="makosh-btn makosh-btn--outline" disabled>
              {{ t('View Logs') }}
            </button>
          </div>
        </section>
      </div>

      <!-- API Providers tab -->
      <div v-else-if="activeTab === 'api-providers'" class="ai-tab-content">
        <div class="empty-panel fill">
          <p>{{ t('AI API provider configuration will be available in a future update.') }}</p>
        </div>
      </div>

      <!-- Model Routing tab -->
      <div v-else-if="activeTab === 'model-routing'" class="ai-tab-content">
        <div class="empty-panel fill">
          <p>{{ t('Model routing configuration will be available in a future update.') }}</p>
        </div>
      </div>

      <!-- Prompt Studio tab -->
      <div v-else-if="activeTab === 'prompt-studio'" class="ai-tab-content">
        <div class="empty-panel fill">
          <p>{{ t('Prompt Studio will be available in a future update.') }}</p>
        </div>
      </div>

      <!-- Runs Health tab -->
      <div v-else-if="activeTab === 'runs-health'" class="ai-tab-content">
        <div class="empty-panel fill">
          <p>{{ t('AI runs health dashboard will be available in a future update.') }}</p>
        </div>
      </div>
    </section>
  </div>
</template>

<style scoped>
.ai-tabs {
  display: flex;
  gap: 4px;
  padding: 8px 12px;
  border-top: 1px solid var(--hh-border);
  border-bottom: 1px solid var(--hh-border);
  overflow-x: auto;
}

.ai-tab-btn {
  padding: 6px 14px;
  border: 1px solid transparent;
  border-radius: var(--hh-radius-sm);
  background: transparent;
  color: var(--hh-text-secondary);
  font-size: 12px;
  font-weight: 620;
  cursor: pointer;
  white-space: nowrap;
  transition: all 100ms ease;
}

.ai-tab-btn:hover {
  border-color: var(--hh-border);
  color: var(--hh-text-primary);
}

.ai-tab-btn.active {
  border-color: var(--hh-accent);
  background: var(--hh-accent-tint, color-mix(in srgb, var(--hh-accent) 10%, transparent));
  color: var(--hh-accent);
}

.ai-tab-content {
  padding: 16px;
  flex: 1;
  min-height: 0;
  overflow-y: auto;
}

.ai-status-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(180px, 1fr));
  gap: 10px;
  margin-bottom: 20px;
}

.ai-status-card {
  padding: 12px;
  border-radius: var(--hh-radius-md);
}

.ai-status-card header {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-bottom: 6px;
}

.ai-status-card strong {
  font-size: 13px;
  font-weight: 620;
  color: var(--hh-text-primary);
}

.ai-status-card p {
  font-size: 11px;
  color: var(--hh-text-muted);
  margin: 0;
}

.ai-status-dot {
  width: 8px;
  height: 8px;
  border-radius: 50%;
  flex-shrink: 0;
}

.ai-status-dot.connected,
.ai-status-dot.active {
  background: var(--hh-status-success, #22c55e);
}

.ai-status-dot.configured {
  background: var(--hh-accent);
}

.ai-status-dot.idle {
  background: var(--hh-text-muted);
}

.ai-section {
  margin-top: 16px;
}

.ai-section h3 {
  margin: 0 0 10px;
  font-size: 13px;
  font-weight: 680;
  color: var(--hh-text-primary);
}

.ai-quick-actions {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
}
</style>
```

### `frontend/src/domains/settings/components/AppearanceSettings.vue`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/domains/settings/components/AppearanceSettings.vue`
- Size bytes / Размер в байтах: `4124`
- Included characters / Включено символов: `4124`
- Truncated / Обрезано: `no`

```text
<script setup lang="ts">
import { useI18n } from '../../../platform/i18n'
import {
	backgroundBrightnessValues,
	panelBlurValues,
	panelOpacityValues,
	type BackgroundBrightness,
	type PanelBlur,
	type PanelOpacity,
	type ThemeSettings
} from '../../../platform/theme/settings'
import { useThemeStore } from '../../../shared/stores/theme'
import AccentPicker from './appearance/AccentPicker.vue'
import AppearanceHeader from './appearance/AppearanceHeader.vue'
import BackgroundPicker from './appearance/BackgroundPicker.vue'
import SpacingDensityControl from './appearance/SpacingDensityControl.vue'
import ThemeRangeControl from './appearance/ThemeRangeControl.vue'

const { t } = useI18n()
const theme = useThemeStore()

function saveThemePatch(patch: Partial<ThemeSettings>) {
	theme.updateThemeDraft(patch)
	void theme.saveThemeSettings()
}

function previewThemePatch(patch: Partial<ThemeSettings>) {
	theme.updateThemeDraft(patch)
}

function commitThemeSettings() {
	void theme.saveThemeSettings()
}

function updateBackgroundBrightness(value: number) {
	const backgroundBrightness = pickAllowedNumber(value, backgroundBrightnessValues)
	if (backgroundBrightness !== null) {
		saveThemePatch({ backgroundBrightness })
	}
}

function updatePanelOpacity(value: number) {
	const panelOpacity = pickAllowedNumber(value, panelOpacityValues)
	if (panelOpacity !== null) {
		previewThemePatch({ panelOpacity })
	}
}

function updatePanelBlur(value: number) {
	const panelBlur = pickAllowedNumber(value, panelBlurValues)
	if (panelBlur !== null) {
		previewThemePatch({ panelBlur })
	}
}

function resetTheme() {
	theme.resetThemeSettings()
	void theme.saveThemeSettings()
}

function pickAllowedNumber<T extends BackgroundBrightness | PanelOpacity | PanelBlur>(
	value: number,
	allowed: readonly T[]
): T | null {
	return allowed.includes(value as T) ? (value as T) : null
}
</script>

<template>
	<div class="settings-page">
		<section class="panel settings-list-panel settings-primary-pane">
				<AppearanceHeader
					:title="t('Interface Appearance')"
					:description="t('Choose shell background, brightness and application accent color.')"
					:is-saving="theme.isSavingTheme"
					:save-state-label="t(theme.themePersistenceLabel)"
					:persistence-error="theme.themePersistenceError ? t(theme.themePersistenceError) : ''"
					@reset="resetTheme"
				/>

			<BackgroundPicker
				:value="theme.effectiveThemeSettings.shellBackground"
				:title="t('Shell Background')"
				:description="t('Background image for the desktop shell.')"
				@change="saveThemePatch({ shellBackground: $event })"
			/>

			<ThemeRangeControl
				id="shell-brightness"
				:label="t('Shell Brightness')"
				:description="t('Controls shell brightness level.')"
				:value="theme.effectiveThemeSettings.backgroundBrightness"
				:min="30"
				:max="100"
				:step="10"
				unit="%"
				@preview="updateBackgroundBrightness"
				@commit="commitThemeSettings"
			/>

			<AccentPicker
				:value="theme.effectiveThemeSettings.accentColor"
				:title="t('Accent Color')"
				:description="t('Application accent color used for highlights and active elements.')"
				@change="saveThemePatch({ accentColor: $event })"
			/>

			<ThemeRangeControl
				id="panel-opacity"
				:label="t('Panel Opacity')"
				:description="t('Controls the opacity of panels and cards.')"
				:value="theme.effectiveThemeSettings.panelOpacity"
				:min="40"
				:max="100"
				:step="10"
				unit="%"
				@preview="updatePanelOpacity"
				@commit="commitThemeSettings"
			/>

			<ThemeRangeControl
				id="panel-blur"
				:label="t('Panel Blur')"
				:description="t('Controls background blur behind panels.')"
				:value="theme.effectiveThemeSettings.panelBlur"
				:min="0"
				:max="24"
				:step="4"
				unit="px"
				@preview="updatePanelBlur"
				@commit="commitThemeSettings"
			/>

			<SpacingDensityControl
				:value="theme.effectiveThemeSettings.spacingDensity"
				:title="t('Spacing Density')"
				:description="t('Controls interface padding density.')"
				@change="saveThemePatch({ spacingDensity: $event })"
			/>
		</section>
	</div>
</template>
```

### `frontend/src/domains/settings/components/ApplicationSettings.vue`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/domains/settings/components/ApplicationSettings.vue`
- Size bytes / Размер в байтах: `9660`
- Included characters / Включено символов: `9660`
- Truncated / Обрезано: `no`

```text
<script setup lang="ts">
import { computed } from 'vue'
import { useI18n } from '../../../platform/i18n'
import { useApplicationSettingsQuery, groupSettingsByCategory } from '../queries/useSettingsQuery'
import { useSettingsStore } from '../stores/settings'
import type { ApplicationSetting } from '../types/settings'

const { t } = useI18n()
const store = useSettingsStore()
const { data: appSettingsData, isLoading } = useApplicationSettingsQuery()

const applicationSettings = computed(() => appSettingsData.value?.items ?? [])
const settingsByCategory = computed(() => groupSettingsByCategory(applicationSettings.value))

function settingDraftValue(setting: ApplicationSetting): string {
  const draft = store.settingDrafts[setting.setting_key]
  if (draft !== undefined) return draft
  return String(setting.value ?? '')
}

function settingHasChanged(setting: ApplicationSetting): boolean {
  const draft = store.settingDrafts[setting.setting_key]
  if (draft === undefined) return false
  return draft !== String(setting.value)
}

function settingControlType(setting: ApplicationSetting): string {
  const allowedValues = setting.metadata?.allowed_values
  if (Array.isArray(allowedValues) && allowedValues.length > 0) return 'select'
  if (setting.value_kind === 'boolean') return 'checkbox'
  if (setting.value_kind === 'integer') return 'number'
  return 'text'
}

function settingAllowedValues(setting: ApplicationSetting): string[] {
  const vals = setting.metadata?.allowed_values
  return Array.isArray(vals) ? vals.map(String) : []
}

function settingMetadataFlag(setting: ApplicationSetting, key: string): boolean {
  return Boolean(setting.metadata?.[key])
}

function settingMetadataText(setting: ApplicationSetting, key: string): string {
  const val = setting.metadata?.[key]
  return typeof val === 'string' ? val : ''
}

function categoryLabel(category: string): string {
  const labels: Record<string, string> = {
    general: 'General',
    frontend: 'Interface',
    ai: 'AI',
    privacy: 'Privacy',
    notifications: 'Notifications'
  }
  return labels[category] ?? category
}

async function handleSave(setting: ApplicationSetting) {
  await store.saveSetting(setting)
}

function handleInput(setting: ApplicationSetting, event: Event) {
  const target = event.target as HTMLInputElement | HTMLSelectElement
  store.updateSettingDraft(setting.setting_key, target.value)
}
</script>

<template>
  <div class="settings-page">
    <section class="panel settings-list-panel settings-primary-pane">
      <header class="panel-title-row">
        <div>
          <h2>{{ t('Application Settings') }}</h2>
          <p>{{ t('All non-secret settings except database connectivity.') }}</p>
        </div>
      </header>

      <!-- Action messages -->
      <div v-if="store.actionMessage" class="setup-state success">{{ store.actionMessage }}</div>
      <div v-if="store.errorMessage" class="inline-error">{{ store.errorMessage }}</div>

      <!-- Loading -->
      <div v-if="isLoading && applicationSettings.length === 0" class="empty-panel fill">
        {{ t('Loading settings...') }}
      </div>

      <!-- Empty -->
      <div v-else-if="Object.keys(settingsByCategory).length === 0" class="empty-panel fill">
        {{ t('No application settings are declared yet.') }}
      </div>

      <!-- Settings list -->
      <div v-else class="settings-category-list">
        <section
          v-for="(settings, category) in settingsByCategory"
          :key="category"
          class="settings-category"
        >
          <header>
            <h3>{{ categoryLabel(category) }}</h3>
            <span>{{ settings.length }}</span>
          </header>

          <div
            v-for="setting in settings"
            :key="setting.setting_key"
            class="setting-row"
          >
            <div class="setting-copy">
              <strong>{{ setting.label }}</strong>
              <p>{{ setting.description }}</p>
              <div class="setting-meta-row">
                <code>{{ setting.setting_key }}</code>
                <em v-if="settingMetadataFlag(setting, 'bootstrap')">Bootstrap</em>
                <em v-if="settingMetadataFlag(setting, 'restart_required')">Restart</em>
                <em v-if="settingMetadataText(setting, 'env_var')">
                  {{ settingMetadataText(setting, 'env_var') }}
                </em>
              </div>
            </div>

            <div class="setting-control">
              <!-- Select control -->
              <select
                v-if="settingControlType(setting) === 'select'"
                class="makosh-select-control"
                :value="settingDraftValue(setting)"
                :disabled="!setting.is_editable"
                @change="(e) => handleInput(setting, e)"
              >
                <option
                  v-for="val in settingAllowedValues(setting)"
                  :key="val"
                  :value="val"
                >
                  {{ val }}
                </option>
              </select>

              <!-- Boolean checkbox -->
              <input
                v-else-if="settingControlType(setting) === 'checkbox'"
                type="checkbox"
                :checked="setting.value === true"
                :disabled="!setting.is_editable"
                @change="(e) => {
                  const checked = (e.target as HTMLInputElement).checked
                  store.updateSettingDraft(setting.setting_key, checked ? 'true' : 'false')
                }"
              />

              <!-- Number input -->
              <input
                v-else-if="settingControlType(setting) === 'number'"
                type="number"
                class="makosh-input-control"
                :value="settingDraftValue(setting)"
                :disabled="!setting.is_editable"
                @input="(e) => handleInput(setting, e)"
              />

              <!-- Text input -->
              <input
                v-else
                type="text"
                class="makosh-input-control"
                :value="settingDraftValue(setting)"
                :disabled="!setting.is_editable"
                @input="(e) => handleInput(setting, e)"
              />

              <!-- Save button -->
              <button
                v-if="settingHasChanged(setting)"
                type="button"
                class="makosh-btn makosh-btn--primary"
                :disabled="store.savingSettingKey === setting.setting_key"
                @click="handleSave(setting)"
              >
                {{ store.savingSettingKey === setting.setting_key ? t('Saving...') : t('Save') }}
              </button>
            </div>
          </div>
        </section>
      </div>
    </section>
  </div>
</template>

<style scoped>
.setting-row {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: 16px;
  padding: 12px 0;
  border-bottom: 1px solid var(--hh-border);
}

.setting-row:last-child {
  border-bottom: none;
}

.setting-copy {
  flex: 1;
  min-width: 0;
}

.setting-copy strong {
  font-size: 13px;
  font-weight: 620;
  color: var(--hh-text-primary);
}

.setting-copy p {
  margin: 2px 0 0;
  font-size: 11px;
  color: var(--hh-text-muted);
  line-height: 1.4;
}

.setting-meta-row {
  display: flex;
  flex-wrap: wrap;
  gap: 6px;
  margin-top: 4px;
  align-items: center;
}

.setting-meta-row code {
  font-size: 10px;
  color: var(--hh-text-muted);
  background: var(--hh-hover-bg);
  padding: 1px 4px;
  border-radius: 3px;
}

.setting-meta-row em {
  font-size: 10px;
  font-weight: 620;
  color: var(--hh-accent);
  font-style: normal;
  background: color-mix(in srgb, var(--hh-accent) 12%, transparent);
  padding: 1px 5px;
  border-radius: 3px;
}

.setting-control {
  display: flex;
  align-items: center;
  gap: 8px;
  flex-shrink: 0;
}

.makosh-select-control,
.makosh-input-control {
  min-width: 160px;
  height: 2.125rem;
  padding: 0 0.625rem;
  background: var(--hh-surface-deep);
  border: 1px solid var(--hh-border);
  border-radius: var(--hh-radius-sm);
  color: var(--hh-text-primary);
  font-size: 0.8125rem;
  font-family: inherit;
  outline: none;
}

.makosh-select-control:focus-visible,
.makosh-input-control:focus-visible {
  box-shadow: 0 0 0 2px var(--hh-focus-ring);
  border-color: var(--hh-accent);
}

.makosh-input-control[type="checkbox"] {
  min-width: auto;
  width: 1rem;
  height: 1rem;
}

/* Section headers */
.settings-category {
  padding: 0;
}

.settings-category header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 8px 0 4px;
  border-bottom: 1px solid var(--hh-border);
  margin-bottom: 4px;
}

.settings-category header h3 {
  margin: 0;
  font-size: 12px;
  font-weight: 720;
  color: var(--hh-text-secondary);
  text-transform: uppercase;
  letter-spacing: 0.03em;
}

.settings-category header span {
  font-size: 11px;
  color: var(--hh-text-muted);
}

.setup-state.success {
  padding: 8px 12px;
  background: color-mix(in srgb, var(--hh-status-success, #22c55e) 15%, transparent);
  border: 1px solid color-mix(in srgb, var(--hh-status-success) 30%, transparent);
  border-radius: var(--hh-radius-sm);
  color: var(--hh-status-success, #22c55e);
  font-size: 12px;
  margin-bottom: 8px;
}

.inline-error {
  padding: 8px 12px;
  background: color-mix(in srgb, var(--hh-status-danger, #ef4444) 15%, transparent);
  border: 1px solid color-mix(in srgb, var(--hh-status-danger) 30%, transparent);
  border-radius: var(--hh-radius-sm);
  color: var(--hh-status-danger, #ef4444);
  font-size: 12px;
  margin-bottom: 8px;
}
</style>
```

### `frontend/src/domains/settings/components/IntegrationsSettings.vue`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/domains/settings/components/IntegrationsSettings.vue`
- Size bytes / Размер в байтах: `14669`
- Included characters / Включено символов: `12000`
- Truncated / Обрезано: `yes`

```text
<script setup lang="ts">
import { computed, ref } from 'vue'
import { useI18n } from '../../../platform/i18n'
import { settingsKeys, useProviderAccountsQuery } from '../queries/useSettingsQuery'
import { useSettingsStore } from '../stores/settings'
import { deleteMailAccount, exportMailAccountSettings, importMailAccountSettings, logoutMailAccount } from '../api/settings'
import type { ProviderAccount } from '../types/settings'
import { useQueryClient } from '@tanstack/vue-query'
import ZoomSettingsPanelShell from '../../../shared/zoom/ZoomSettingsPanelShell.vue'
import YandexTelemostSettingsPanelShell from '../../../shared/yandexTelemost/YandexTelemostSettingsPanelShell.vue'
type MailProviderKind = 'gmail' | 'icloud' | 'imap'

const { t } = useI18n()
const store = useSettingsStore()
const queryClient = useQueryClient()
const { data: accountsData } = useProviderAccountsQuery()
const accounts = computed(() => accountsData.value?.items ?? [])

const selectedAccount = computed(() => {
  if (!store.selectedIntegrationId) return null
  return accounts.value.find((a) => a.account_id === store.selectedIntegrationId) ?? null
})

const isImportPanelOpen = ref(false)
const mailImportJson = ref('')
const activeMailAction = ref<string | null>(null)

function isMailProvider(providerKind: string): boolean {
  const mailProviders: MailProviderKind[] = ['gmail', 'icloud', 'imap']
  return mailProviders.includes(providerKind as MailProviderKind)
}

function isZoomProvider(providerKind: string): boolean {
  return providerKind === 'zoom_user' || providerKind === 'zoom_server_to_server'
}

function isYandexTelemostProvider(providerKind: string): boolean {
  return providerKind === 'yandex_telemost_user'
}

const groups = computed(() => {
  const mail = accounts.value.filter((a) => isMailProvider(a.provider_kind))
  const zoom = accounts.value.filter((a) => isZoomProvider(a.provider_kind))
  const yandexTelemost = accounts.value.filter((a) => isYandexTelemostProvider(a.provider_kind))
  const other = accounts.value.filter((a) => !isMailProvider(a.provider_kind) && !isZoomProvider(a.provider_kind) && !isYandexTelemostProvider(a.provider_kind))
  const rows = []
  if (mail.length) rows.push({ label: t('Mail accounts'), items: mail })
  if (zoom.length) rows.push({ label: t('Zoom accounts'), items: zoom })
  if (yandexTelemost.length) rows.push({ label: t('Yandex Telemost accounts'), items: yandexTelemost })
  if (other.length) rows.push({ label: t('Other accounts'), items: other })
  if (!rows.length) rows.push({ label: t('Accounts'), items: accounts.value })
  return rows
})

function providerIcon(providerKind: string): string {
  const icons: Record<string, string> = {
    gmail: 'tabler:mail',
    icloud: 'tabler:cloud',
    imap: 'tabler:server',
    zoom_user: 'tabler:video',
    zoom_server_to_server: 'tabler:video-plus',
    yandex_telemost_user: 'tabler:video-plus',
  }
  return icons[providerKind] || 'tabler:plug-connected'
}

function providerLabel(providerKind: string): string {
  const labels: Record<string, string> = {
    gmail: 'Gmail',
    icloud: 'iCloud',
    imap: 'IMAP',
    zoom_user: 'Zoom (OAuth/Live)',
    zoom_server_to_server: 'Zoom (Server-to-Server)',
    yandex_telemost_user: 'Yandex Telemost',
  }
  return labels[providerKind] || providerKind
}

function providerDisplayName(account: ProviderAccount): string {
  return (
    account.display_name ||
    account.label ||
    account.email ||
    (typeof account.config?.email === 'string' ? account.config.email : null) ||
    account.external_account_id ||
    account.account_id
  )
}

function statusText(account: ProviderAccount): string {
  if (typeof account.is_authenticated === 'boolean' && !account.is_authenticated) return t('Not authenticated')
  if (typeof account.is_active === 'boolean' && !account.is_active) return t('Inactive')
  if (isZoomProvider(account.provider_kind) || isYandexTelemostProvider(account.provider_kind)) return t('Configured')
  return t('Active')
}

function statusClass(account: ProviderAccount) {
  if (typeof account.is_authenticated === 'boolean' && !account.is_authenticated) return 'unauthenticated'
  if (typeof account.is_active === 'boolean' && !account.is_active) return 'inactive'
  if (isZoomProvider(account.provider_kind) || isYandexTelemostProvider(account.provider_kind)) return 'configured'
  return 'active'
}

function selectedAccountEmail(account: ProviderAccount): string {
  return account.email || (typeof account.config?.email === 'string' ? account.config.email : '')
}

async function refreshSettings() {
  await queryClient.invalidateQueries({ queryKey: settingsKeys.workspace() })
}

async function handleExport(accountId: string) {
  activeMailAction.value = accountId
  try {
    const result = await exportMailAccountSettings(accountId)
    if (result.result) {
      const filename = `mail-account-${accountId}-${result.result.exported_at}.json`
      const blob = new Blob([JSON.stringify(result.result, null, 2)], { type: 'application/json' })
      const url = URL.createObjectURL(blob)
      const link = document.createElement('a')
      link.href = url
      link.download = filename
      document.body.appendChild(link)
      link.click()
      link.remove()
      URL.revokeObjectURL(url)
      store.setActionMessage(t('Mail account settings exported'))
    }
  } catch (err) {
    store.setError(err instanceof Error ? err.message : 'Export failed')
  } finally {
    activeMailAction.value = null
  }
}

async function handleLogout(accountId: string) {
  activeMailAction.value = accountId
  try {
    await logoutMailAccount(accountId)
    store.setActionMessage(t('Mail account logged out'))
    await refreshSettings()
  } catch (err) {
    store.setError(err instanceof Error ? err.message : 'Logout failed')
  } finally {
    activeMailAction.value = null
  }
}

async function handleDelete(accountId: string) {
  activeMailAction.value = accountId
  try {
    await deleteMailAccount(accountId)
    store.setActionMessage(t('Mail account deleted'))
    await refreshSettings()
  } catch (err) {
    store.setError(err instanceof Error ? err.message : 'Delete failed')
  } finally {
    activeMailAction.value = null
  }
}

async function handleImport() {
  try {
    const parsed = JSON.parse(mailImportJson.value)
    await importMailAccountSettings(parsed)
    store.setActionMessage(t('Mail account settings imported'))
    isImportPanelOpen.value = false
    mailImportJson.value = ''
    await refreshSettings()
  } catch (err) {
    store.setError(err instanceof Error ? err.message : 'Import failed')
  }
}

</script>

<template>
  <div class="settings-page">
    <section class="panel settings-list-panel settings-primary-pane">
      <header class="panel-title-row">
        <div>
          <h2>{{ t('Integrations') }}</h2>
          <p>{{ t('Connected accounts and provider services.') }}</p>
        </div>
      </header>
      <div v-if="store.actionMessage" class="setup-state success">{{ store.actionMessage }}</div>
      <div v-if="store.errorMessage" class="inline-error">{{ store.errorMessage }}</div>

      <div v-if="accounts.length === 0" class="empty-panel fill">{{ t('No connected accounts yet.') }}</div>
      <div v-else v-for="group in groups" :key="group.label" class="integration-group">
        <h3>{{ group.label }}</h3>
        <div class="integrations-table">
          <button
            v-for="account in group.items"
            :key="account.account_id"
            type="button"
            class="integration-row"
            :class="{ selected: store.selectedIntegrationId === account.account_id }"
            @click="store.selectIntegration(account.account_id)"
          >
            <div class="integration-info">
              <span class="integration-icon" v-text="providerIcon(account.provider_kind)" />
              <div>
                <strong>{{ providerDisplayName(account) }}</strong>
                <span class="integration-provider">{{ providerLabel(account.provider_kind) }}</span>
              </div>
            </div>
            <div class="integration-status">
              <span class="status-dot" :class="statusClass(account)" />
              <span>{{ statusText(account) }}</span>
            </div>
          </button>
        </div>
      </div>

      <div v-if="selectedAccount" class="integration-inspector">
        <header>
          <h3>{{ providerDisplayName(selectedAccount) }}</h3>
          <button type="button" class="makosh-btn makosh-btn--ghost" @click="store.selectIntegration(null)">
            {{ t('Close') }}
          </button>
        </header>

        <div class="inspector-details">
          <div class="detail-row"><span>{{ t('Provider') }}</span><strong>{{ providerLabel(selectedAccount.provider_kind) }}</strong></div>
          <div class="detail-row"><span>{{ t('External account id') }}</span><strong>{{ selectedAccount.external_account_id }}</strong></div>
          <div v-if="selectedAccountEmail(selectedAccount)" class="detail-row">
            <span>{{ t('Email') }}</span><strong>{{ selectedAccountEmail(selectedAccount) }}</strong>
          </div>
          <div class="detail-row"><span>{{ t('Status') }}</span><strong>{{ statusText(selectedAccount) }}</strong></div>
        </div>

        <div v-if="isMailProvider(selectedAccount.provider_kind)" class="inspector-actions">
          <button type="button" class="makosh-btn makosh-btn--outline" :disabled="activeMailAction===selectedAccount.account_id" @click="handleExport(selectedAccount.account_id)">
            {{ t('Export') }}
          </button>
          <button type="button" class="makosh-btn makosh-btn--outline" :disabled="activeMailAction===selectedAccount.account_id" @click="handleLogout(selectedAccount.account_id)">
            {{ t('Logout') }}
          </button>
          <button type="button" class="makosh-btn makosh-btn--destructive" :disabled="activeMailAction===selectedAccount.account_id" @click="handleDelete(selectedAccount.account_id)">
            {{ t('Delete') }}
          </button>
        </div>

      </div>

      <ZoomSettingsPanelShell :selected-account="selectedAccount" @removed="store.selectIntegration(null)" />
      <YandexTelemostSettingsPanelShell :selected-account="selectedAccount" />

      <div class="integration-import-section">
        <button type="button" class="makosh-btn makosh-btn--secondary" @click="isImportPanelOpen = !isImportPanelOpen">
          {{ isImportPanelOpen ? t('Cancel') : t('Import Mail Settings') }}
        </button>
        <div v-if="isImportPanelOpen" class="import-panel">
          <textarea v-model="mailImportJson" class="import-textarea" rows="6" :placeholder="t('Paste exported mail account JSON here...')" />
          <button type="button" class="makosh-btn makosh-btn--primary" :disabled="!mailImportJson.trim()" @click="handleImport">
            {{ t('Import') }}
          </button>
        </div>
      </div>
    </section>
  </div>
</template>

<style scoped>
.integration-group { margin-top: 10px; border: 1px solid var(--hh-border); border-radius: var(--hh-radius-md); background: var(--hh-surface-deep); padding: 10px; }
.integration-group h3,.integration-section h3 { margin: 0 0 6px; font-size: 13px; color: var(--hh-text-secondary); }
.integrations-table { display: grid; gap: 4px; }
.integration-row { display: flex; align-items: center; justify-content: space-between; padding: 10px 12px; border-radius: var(--hh-radius-sm); cursor: pointer; transition: background 100ms ease; }
.integration-row:hover, .integration-row.selected { background: var(--hh-hover-bg); }
.integration-row.selected { border: 1px solid var(--hh-border); }
.integration-info { display: flex; align-items: center; gap: 10px; }
.integration-icon { font-size: 1.25rem; color: var(--hh-text-secondary); }
.integration-provider { font-size: 11px; color: var(--hh-text-muted); }
.integration-status { display: flex; align-items: center; gap: 6px; fo
```
_Source file truncated after 12000 characters. / Исходный файл обрезан после 12000 символов._

### `frontend/src/domains/settings/components/LanguageSettings.vue`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/domains/settings/components/LanguageSettings.vue`
- Size bytes / Размер в байтах: `2197`
- Included characters / Включено символов: `2190`
- Truncated / Обрезано: `no`

```text
<script setup lang="ts">
import { useI18n } from '../../../platform/i18n'
import { saveApplicationSetting } from '../api/settings'
import { FRONTEND_LOCALE_SETTING_KEY } from '../types/settings'
import type { Locale } from '../../../platform/i18n/types'

const { t, locale, setLocale } = useI18n()

const localeOptions = [
  { value: 'en', label: 'English' },
  { value: 'ru', label: 'Русский' }
]

async function updateLocale(value: string) {
  if (value !== 'en' && value !== 'ru') return
  setLocale(value as Locale)
  try {
    await saveApplicationSetting(FRONTEND_LOCALE_SETTING_KEY, value)
  } catch (err) {
    // Revert on failure
    const revert = value === 'en' ? 'ru' : 'en'
    setLocale(revert as Locale)
    console.error('Failed to save locale setting:', err)
  }
}
</script>

<template>
  <div class="settings-page">
    <section class="panel settings-list-panel settings-primary-pane">
      <header class="panel-title-row">
        <div>
          <h2>{{ t('Interface Language') }}</h2>
          <p>{{ t('Choose the display language for the Макошь interface.') }}</p>
        </div>
      </header>
      <div class="settings-category-list">
        <div class="setting-row">
          <span>{{ t('Language') }}</span>
          <div class="setting-control">
            <select
              class="makosh-select-control"
              :value="locale"
              @change="(e) => updateLocale((e.target as HTMLSelectElement).value)"
            >
              <option v-for="opt in localeOptions" :key="opt.value" :value="opt.value">
                {{ opt.label }}
              </option>
            </select>
          </div>
        </div>
      </div>
    </section>
  </div>
</template>

<style scoped>
.makosh-select-control {
  min-width: 180px;
  height: 2.125rem;
  padding: 0 0.625rem;
  background: var(--hh-surface-deep);
  border: 1px solid var(--hh-border);
  border-radius: var(--hh-radius-sm);
  color: var(--hh-text-primary);
  font-size: 0.8125rem;
  font-family: inherit;
  cursor: pointer;
  outline: none;
}

.makosh-select-control:focus-visible {
  box-shadow: 0 0 0 2px var(--hh-focus-ring);
  border-color: var(--hh-accent);
}
</style>
```

### `frontend/src/domains/settings/components/SidebarSettings.vue`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/domains/settings/components/SidebarSettings.vue`
- Size bytes / Размер в байтах: `9969`
- Included characters / Включено символов: `9969`
- Truncated / Обрезано: `no`

```text
<script setup lang="ts">
import { computed } from 'vue'
import { useQueryClient } from '@tanstack/vue-query'
import { useI18n } from '../../../platform/i18n'
import {
  useSidebarStore,
  type SidebarItemId,
  type SidebarNavGroup,
  type SidebarSettings as SidebarSettingsValue
} from '../../../shared/stores/sidebar'
import { saveApplicationSetting } from '../api/settings'
import { settingsKeys } from '../queries/useSettingsQuery'
import { useSettingsStore } from '../stores/settings'
import {
  FRONTEND_SIDEBAR_SETTING_KEY,
  type ApplicationSettingValue
} from '../types/settings'
import SidebarGroupEditor from './sidebar/SidebarGroupEditor.vue'
import SidebarNavigationList from './sidebar/SidebarNavigationList.vue'
import SidebarSettingsSummary from './sidebar/SidebarSettingsSummary.vue'

const { t } = useI18n()
const sidebar = useSidebarStore()
const store = useSettingsStore()
const queryClient = useQueryClient()

const sidebarItemLabels = computed<Record<SidebarItemId, { label: string; icon: string }>>(() => {
  const labels = {} as Record<SidebarItemId, { label: string; icon: string }>
  const itemIds = new Set<SidebarItemId>([
    ...sidebar.effectiveSidebarSettings.hiddenItemIds,
    ...sidebar.effectiveSidebarSettings.groups.flatMap((group) => group.itemIds),
    ...sidebar.sidebarRootEntries.flatMap((entry) => entry.kind === 'item' ? [entry.item.itemId] : [])
  ])

  for (const itemId of itemIds) {
    const item = sidebar.sidebarConfigItem(itemId)
    if (item) labels[itemId] = { label: item.label, icon: item.icon }
  }

  return labels
})

const sidebarGroupOptions = computed(() =>
  sidebar.effectiveSidebarSettings.groups.map((group, index) => ({
    value: group.id,
    label: sidebarGroupLabel(group, index)
  }))
)

const sidebarRuleSummaries = computed(() => [
  { text: t('Default keeps the current sidebar order'), badge: t('Preset') },
  { text: t('Communications sources stay nested'), badge: t('Context') },
  { text: t('Hidden domains stay recoverable here'), badge: t('Safe') },
  { text: t('Settings store no message content'), badge: t('Privacy') }
])

function sidebarGroupLabel(group: SidebarNavGroup, index: number): string {
  return group.label || (group.id === 'communications' ? 'Communications' : `Group ${index + 1}`)
}

function sidebarRootIndexForGroup(groupId: string): number {
  const normalized = sidebar.sidebarGroupIdFromLabel(groupId)
  return sidebar.effectiveSidebarSettings.rootItemIds.indexOf(`group:${normalized}`)
}

function toApplicationSettingValue(settings: SidebarSettingsValue): ApplicationSettingValue {
  return {
    schemaVersion: settings.schemaVersion,
    rootItemIds: [...settings.rootItemIds],
    groups: settings.groups.map((group) => ({
      id: group.id,
      label: group.label,
      icon: group.icon,
      itemIds: [...group.itemIds],
      separatorBeforeItemIds: [...group.separatorBeforeItemIds]
    })),
    hiddenItemIds: [...settings.hiddenItemIds]
  }
}

async function handleSaveSidebar(): Promise<void> {
  store.isSidebarSettingsSaving = true
  store.sidebarError = ''
  try {
    await saveApplicationSetting(
      FRONTEND_SIDEBAR_SETTING_KEY,
      toApplicationSettingValue(sidebar.effectiveSidebarSettings)
    )
    sidebar.setSidebarSettings(sidebar.effectiveSidebarSettings)
    queryClient.invalidateQueries({ queryKey: settingsKeys.application() })
    store.setActionMessage(t('Sidebar saved'))
  } catch (err) {
    store.sidebarError = err instanceof Error ? err.message : t('Failed to save sidebar')
  } finally {
    store.isSidebarSettingsSaving = false
  }
}

</script>

<template>
  <div class="settings-layout sidebar-settings-layout">
    <section class="panel settings-list-panel settings-primary-pane sidebar-settings-panel">
      <header class="panel-title-row">
        <div>
          <h2>{{ t('Sidebar Navigation') }}</h2>
          <p>{{ t('Configure workspace groups, order and hidden domains.') }}</p>
        </div>
        <div class="sidebar-settings-actions">
          <button
            type="button"
            class="makosh-btn makosh-btn--outline"
            :disabled="!sidebar.sidebarDraft || store.isSidebarSettingsSaving"
            @click="sidebar.cancelSidebarSettingsEditing()"
          >
            {{ t('Cancel') }}
          </button>
          <button
            type="button"
            class="makosh-btn makosh-btn--outline"
            :disabled="store.isSidebarSettingsSaving"
            @click="sidebar.resetSidebarSettingsToDefault()"
          >
            {{ t('Default') }}
          </button>
          <button
            type="button"
            class="makosh-btn makosh-btn--primary"
            :disabled="!sidebar.sidebarDraft || store.isSidebarSettingsSaving"
            @click="handleSaveSidebar"
          >
            {{ store.isSidebarSettingsSaving ? t('Saving...') : t('Save') }}
          </button>
        </div>
      </header>

      <div v-if="store.sidebarError" class="inline-error">{{ store.sidebarError }}</div>

      <form class="sidebar-group-create" @submit.prevent="sidebar.addSidebarGroup()">
        <label>
          <span>{{ t('New group') }}</span>
          <input
            v-model="store.newSidebarGroupLabel"
            :placeholder="t('Focus, Library, Planning')"
            autocomplete="off"
          />
        </label>
        <button type="submit" class="makosh-btn makosh-btn--secondary">
          {{ t('Create Group') }}
        </button>
      </form>

      <div class="sidebar-config-list">
        <SidebarNavigationList
          :entries="sidebar.sidebarRootEntries"
          :hidden-item-ids="sidebar.sidebarHiddenNavItems"
          :root-item-count="sidebar.effectiveSidebarSettings.rootItemIds.length"
          :group-options="sidebarGroupOptions"
          :root-label="t('Root level')"
          :sidebar-root-label="t('Sidebar root')"
          :expandable-group-label="t('Expandable group')"
          :items-label="t('items')"
          :hidden-label="t('Hidden from sidebar')"
          :root-domain-label="t('Root domain')"
          :move-to-group-label="t('Move to group')"
          :show-label="t('Show')"
          :hide-label="t('Hide')"
          @move-group="sidebar.moveSidebarGroup"
          @remove-group="sidebar.removeSidebarGroup"
          @move-root-item="sidebar.moveSidebarRootItem"
          @move-item-to-group="sidebar.moveSidebarItemToGroup"
          @toggle-hidden="sidebar.toggleSidebarItemHidden"
        />

        <SidebarGroupEditor
          v-for="(group, groupIndex) in sidebar.effectiveSidebarSettings.groups"
          :key="group.id"
          :group="group"
          :group-index="groupIndex"
          :root-index="sidebarRootIndexForGroup(group.id)"
          :root-item-count="sidebar.effectiveSidebarSettings.rootItemIds.length"
          :item-labels="sidebarItemLabels"
          :hidden-item-ids="sidebar.sidebarHiddenNavItems"
          :group-options="sidebarGroupOptions"
          :group-label-text="t('Group label')"
          :default-placeholder="t('Primary')"
          :group-placeholder="t('Group {n}').replace('{n}', String(groupIndex + 1))"
          :visible-domain-label="t('Visible domain')"
          :hidden-label="t('Hidden from sidebar')"
          :no-items-label="t('No items in this group.')"
          :move-to-group-label="t('Move to group')"
          :divider-label="t('Divider')"
          :show-label="t('Show')"
          :hide-label="t('Hide')"
          @rename="sidebar.updateSidebarGroupLabel"
          @move-group="sidebar.moveSidebarGroup"
          @remove-group="sidebar.removeSidebarGroup"
          @move-item-to-group="sidebar.moveSidebarItemToGroup"
          @move-item="sidebar.moveSidebarItem"
          @toggle-divider="sidebar.toggleSidebarGroupSeparator"
          @toggle-hidden="sidebar.toggleSidebarItemHidden"
        />
      </div>
    </section>

    <SidebarSettingsSummary
      :entries="sidebar.sidebarRootEntries"
      :hidden-item-ids="sidebar.sidebarHiddenNavItems"
      :item-labels="sidebarItemLabels"
      :preview-label="t('Preview')"
      :hidden-label="t('Hidden')"
      :rules-label="t('Rules')"
      :root-domain-label="t('Root domain')"
      :empty-group-label="t('Empty group')"
      :no-hidden-label="t('No domains are hidden.')"
      :show-label="t('Show')"
      :rules="sidebarRuleSummaries"
      @toggle-hidden="sidebar.toggleSidebarItemHidden"
    />
  </div>
</template>

<style scoped>
.sidebar-settings-layout {
  display: grid;
  grid-template-columns: minmax(0, 1fr) minmax(260px, 310px);
  gap: var(--hh-layout-gap);
  height: 100%;
  min-height: 0;
  overflow: hidden;
}

.sidebar-settings-panel {
  display: flex;
  flex-direction: column;
  min-width: 0;
  min-height: 0;
}

.sidebar-settings-actions,
.sidebar-group-create {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
}

.sidebar-group-create {
  align-items: center;
  padding: 12px;
  border-top: 1px solid var(--hh-border);
}

.sidebar-group-create label {
  display: flex;
  align-items: center;
  gap: 8px;
  flex: 1;
  font-size: 12px;
  color: var(--hh-text-secondary);
}

.sidebar-group-create input {
  flex: 1;
  min-width: 0;
  height: 34px;
  padding: 0 10px;
  background: var(--hh-surface-deep);
  border: 1px solid var(--hh-border);
  border-radius: var(--hh-radius-sm);
  color: var(--hh-text-primary);
  font-size: 12px;
  outline: none;
}

.sidebar-group-create input:focus-visible {
  box-shadow: 0 0 0 2px var(--hh-focus-ring);
  border-color: var(--hh-accent);
}

.sidebar-config-list {
  flex: 1 1 auto;
  min-height: 0;
  overflow-x: hidden;
  overflow-y: auto;
  padding: 8px;
}

.inline-error {
  padding: 8px 12px;
  background: color-mix(in srgb, var(--hh-status-danger) 15%, transparent);
  border: 1px solid color-mix(in srgb, var(--hh-status-danger) 30%, transparent);
  border-radius: var(--hh-radius-sm);
  color: var(--hh-status-danger);
  font-size: 12px;
  margin: 8px;
}
</style>
```
