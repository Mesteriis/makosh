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

- Chunk ID / ID чанка: `132-other-frontend-part-005`
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

### `frontend/src/domains/agents/components/AgentsRail.vue`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/domains/agents/components/AgentsRail.vue`
- Size bytes / Размер в байтах: `2150`
- Included characters / Включено символов: `2148`
- Truncated / Обрезано: `no`

```text
<script setup lang="ts">
import Icon from '../../../shared/ui/Icon.vue'
import type { AiRun, AiStatus, AiCitation, OwnerPersona } from '../types/agents'
import { aiRuntimeSummary, runStatusLabel, safeCitations } from '../stores/agents'
import { formatDuration, formatDateTime } from '../stores/agents'

interface Props {
	aiRuns: AiRun[]
	aiStatus: AiStatus | null
	ownerPersona: OwnerPersona | null
	isAiLoading: boolean
}

const props = defineProps<Props>()

function formatAgentPersonaName(agentId: string): string {
	return `${agentId.trim().toLowerCase()}@sh-inc.ru`
}
</script>

<template>
	<aside class="stacked-rail">
		<section class="panel info-card">
			<h2>Runtime</h2>
			<div class="health-row">
				<span>Status</span>
				<strong>{{ aiRuntimeSummary(aiStatus, isAiLoading) }}</strong>
			</div>
			<div class="health-row">
				<span>Owner Persona</span>
				<strong>{{ ownerPersona?.display_name ?? 'not set' }}</strong>
			</div>
			<div class="health-row">
				<span>Chat</span>
				<strong>{{ aiStatus?.chat_model ?? 'unknown' }}</strong>
			</div>
			<div class="health-row">
				<span>Embedding</span>
				<strong>{{ aiStatus?.embedding_model ?? 'unknown' }}</strong>
			</div>
		</section>

		<section class="panel info-card">
			<h2>Run History</h2>
			<template v-if="aiRuns.length">
				<div v-for="run in aiRuns.slice(0, 6)" :key="run.run_id" class="deadline">
					<span>{{ formatAgentPersonaName(run.agent_id) }} · {{ runStatusLabel(run) }}</span>
					<time>{{ formatDateTime(run.started_at) }} · {{ formatDuration(run.duration_ms) }}</time>
				</div>
			</template>
			<p v-else>No AI runs persisted yet.</p>
		</section>

		<section class="panel info-card">
			<h2>Latest Citations</h2>
			<template v-if="aiRuns[0] && safeCitations(aiRuns[0].citations).length">
				<div v-for="citation in safeCitations(aiRuns[0].citations).slice(0, 3)" :key="citation.source_id + citation.source_kind" class="evidence-row">
					<strong>{{ citation.title }}</strong>
					<p>{{ citation.excerpt }}</p>
				</div>
			</template>
			<p v-else>Citations appear after an answer or briefing run.</p>
		</section>
	</aside>
</template>
```

### `frontend/src/domains/agents/components/AgentsRuntimeMetrics.vue`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/domains/agents/components/AgentsRuntimeMetrics.vue`
- Size bytes / Размер в байтах: `1755`
- Included characters / Включено символов: `1755`
- Truncated / Обрезано: `no`

```text
<script setup lang="ts">
import Icon from '../../../shared/ui/Icon.vue'
import type { AiStatus, AiAgent, AiRun } from '../types/agents'
import { aiRuntimeSummary, formatDuration } from '../stores/agents'

interface Props {
	aiStatus: AiStatus | null
	aiAgents: AiAgent[]
	aiRuns: AiRun[]
	isAiLoading: boolean
}

const props = defineProps<Props>()

function formatAgentPersonaName(agentId: string): string {
	return `${agentId.trim().toLowerCase()}@sh-inc.ru`
}
</script>

<template>
	<div class="metric-grid agent-metrics">
		<article class="metric-card">
			<span>Runtime</span>
			<strong>{{ aiRuntimeSummary(aiStatus, isAiLoading) }}</strong>
			<small>{{ aiStatus?.version ? `Ollama ${aiStatus.version}` : 'Ollama' }}</small>
		</article>
		<article class="metric-card">
			<span>Agents</span>
			<strong>{{ aiAgents.length }}</strong>
			<small>{{ aiAgents.length ? 'Registered' : 'Not loaded' }}</small>
		</article>
		<article class="metric-card">
			<span>Run History</span>
			<strong>{{ aiRuns.length }}</strong>
			<small>Persisted runs</small>
		</article>
		<article class="metric-card">
			<span>Embedding</span>
			<strong>{{ aiStatus?.embedding_dimension ?? 0 }}</strong>
			<small>{{ aiStatus?.embedding_model ?? 'No model' }}</small>
		</article>
		<article class="metric-card">
			<span>Suggested Tasks</span>
			<strong>0</strong>
			<small>Review queue</small>
		</article>
		<article class="metric-card">
			<span>Latest Duration</span>
			<strong>{{ formatDuration(aiRuns[0]?.duration_ms) }}</strong>
			<small>{{ aiRuns[0] ? formatAgentPersonaName(aiRuns[0].agent_id) : 'No runs' }}</small>
		</article>
	</div>
</template>

<style scoped>
.agent-metrics {
	grid-template-columns: repeat(6, 1fr);
	margin-bottom: 12px;
}
</style>
```

### `frontend/src/domains/agents/components/AgentsWorkflows.vue`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/domains/agents/components/AgentsWorkflows.vue`
- Size bytes / Размер в байтах: `5602`
- Included characters / Включено символов: `5602`
- Truncated / Обрезано: `no`

```text
<script setup lang="ts">
import Icon from '../../../shared/ui/Icon.vue'
import type { AiAnswerResponse, AiMeetingPrepResponse, AiTaskCandidateRefreshResponse, AiCitation } from '../types/agents'
import { safeCitations } from '../stores/agents'

interface Props {
	aiQuestion: string
	aiMeetingTopic: string
	aiTaskQuery: string
	aiAnswerResult: AiAnswerResponse | null
	aiMeetingPrepResult: AiMeetingPrepResponse | null
	aiTaskRefreshResult: AiTaskCandidateRefreshResponse | null
	isAiAnswerSubmitting: boolean
	isAiMeetingPrepSubmitting: boolean
	isAiTaskRefreshSubmitting: boolean
}

interface Emits {
	(e: 'update:aiQuestion', value: string): void
	(e: 'update:aiMeetingTopic', value: string): void
	(e: 'update:aiTaskQuery', value: string): void
	(e: 'submitAnswer'): void
	(e: 'submitMeetingPrep'): void
	(e: 'refreshTasks'): void
}

const props = defineProps<Props>()
const emit = defineEmits<Emits>()
</script>

<template>
	<div class="ai-workflow-grid">
		<form class="ai-workflow-block" @submit.prevent="emit('submitAnswer')">
			<label>
				<span>Ask AI</span>
				<textarea
					:value="aiQuestion"
					@input="emit('update:aiQuestion', ($event.target as HTMLTextAreaElement).value)"
					rows="4"
				></textarea>
			</label>
			<button type="submit" :disabled="isAiAnswerSubmitting || !aiQuestion.trim()">
				<Icon icon="tabler:sparkles" width="16" height="16" />Ask
			</button>
		</form>
		<form class="ai-workflow-block" @submit.prevent="emit('submitMeetingPrep')">
			<label>
				<span>Prepare brief</span>
				<textarea
					:value="aiMeetingTopic"
					@input="emit('update:aiMeetingTopic', ($event.target as HTMLTextAreaElement).value)"
					rows="4"
				></textarea>
			</label>
			<button type="submit" :disabled="isAiMeetingPrepSubmitting || !aiMeetingTopic.trim()">
				<Icon icon="tabler:calendar-stats" width="16" height="16" />Prepare
			</button>
		</form>
		<form class="ai-workflow-block" @submit.prevent="emit('refreshTasks')">
			<label>
				<span>Task extraction</span>
				<textarea
					:value="aiTaskQuery"
					@input="emit('update:aiTaskQuery', ($event.target as HTMLTextAreaElement).value)"
					rows="4"
				></textarea>
			</label>
			<button type="submit" :disabled="isAiTaskRefreshSubmitting || !aiTaskQuery.trim()">
				<Icon icon="tabler:checkbox" width="16" height="16" />Refresh candidates
			</button>
		</form>
	</div>

	<div v-if="aiAnswerResult" class="ai-result-block">
		<h3>Answer</h3>
		<p>{{ aiAnswerResult.answer }}</p>
		<div class="citation-list">
			<div v-for="citation in aiAnswerResult.citations" :key="citation.source_id + citation.source_kind" class="citation-row">
				<strong>{{ citation.title }}</strong>
				<span>{{ citation.source_kind }}:{{ citation.source_id }}</span>
				<p>{{ citation.excerpt }}</p>
			</div>
		</div>
	</div>

	<div v-if="aiMeetingPrepResult" class="ai-result-block">
		<h3>Meeting Brief</h3>
		<p>{{ aiMeetingPrepResult.briefing }}</p>
		<div class="citation-list">
			<div v-for="citation in aiMeetingPrepResult.citations" :key="citation.source_id + citation.source_kind" class="citation-row">
				<strong>{{ citation.title }}</strong>
				<span>{{ citation.source_kind }}:{{ citation.source_id }}</span>
				<p>{{ citation.excerpt }}</p>
			</div>
		</div>
	</div>

	<div v-if="aiTaskRefreshResult" class="ai-result-block">
		<h3>Task Candidates</h3>
		<p>{{ aiTaskRefreshResult.created_count }} suggested candidates refreshed. Review them in Tasks.</p>
	</div>
</template>

<style scoped>
.ai-workflow-grid {
	display: grid;
	grid-template-columns: repeat(3, minmax(0, 1fr));
	gap: 10px;
	margin-top: 16px;
}

.ai-workflow-block {
	display: grid;
	gap: 10px;
	min-height: var(--hh-widget-panel-large);
	border: 1px solid rgba(111, 205, 195, 0.1);
	border-radius: var(--hh-radius-md);
	background: rgba(5, 22, 25, 0.54);
	padding: 12px;
}

.ai-workflow-block label {
	display: grid;
	gap: 8px;
}

.ai-workflow-block span {
	color: var(--hh-color-text-soft);
	font-size: 12px;
	font-weight: 650;
}

.ai-workflow-block textarea {
	width: 100%;
	min-height: 92px;
	max-height: 130px;
	resize: vertical;
	border: 1px solid rgba(111, 205, 195, 0.16);
	border-radius: var(--hh-radius-md);
	background: rgba(2, 9, 11, 0.7);
	color: var(--hh-color-text);
	font-size: 12px;
	line-height: 1.45;
	padding: 9px 10px;
}

.ai-workflow-block button {
	display: inline-flex;
	align-items: center;
	justify-content: center;
	gap: 7px;
	min-height: 34px;
	border-radius: var(--hh-radius-md);
	background: var(--hh-color-accent);
	color: var(--hh-color-accent-contrast);
	font-size: 12px;
	font-weight: 760;
	border: none;
	cursor: pointer;
}

.ai-workflow-block button:disabled {
	background: rgba(111, 205, 195, 0.16);
	color: #789b98;
	cursor: not-allowed;
}

.ai-result-block {
	display: grid;
	gap: 10px;
	margin-top: 14px;
	border-top: 1px solid var(--hh-border-muted);
	padding-top: 14px;
}

.ai-result-block h3 {
	margin: 0;
	color: var(--hh-color-text-bright);
	font-size: 15px;
}

.ai-result-block > p {
	margin: 0;
	color: var(--hh-color-text-soft);
	font-size: 13px;
	line-height: 1.55;
}

.citation-list {
	display: grid;
	gap: 8px;
}

.citation-row {
	display: grid;
	gap: 4px;
	border-left: 2px solid var(--hh-border-accent);
	background: rgba(45, 240, 206, 0.045);
	padding: 8px 10px;
}

.citation-row strong,
.citation-row span,
.citation-row p {
	overflow-wrap: anywhere;
}

.citation-row strong {
	color: var(--hh-color-text);
	font-size: 12px;
}

.citation-row span {
	color: #7ea4a0;
	font-size: 10px;
}

.citation-row p {
	margin: 0;
	color: #bcd3d1;
	font-size: 12px;
	line-height: 1.45;
}
</style>
```

### `frontend/src/domains/agents/views/AgentsPage.vue`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/domains/agents/views/AgentsPage.vue`
- Size bytes / Размер в байтах: `3584`
- Included characters / Включено символов: `3584`
- Truncated / Обрезано: `no`

```text
<script setup lang="ts">
import { watch } from 'vue'
import Icon from '../../../shared/ui/Icon.vue'
import { useAiWorkspaceQuery } from '../queries/useAgentsQuery'
import { useAgentsStore, aiModelSummary } from '../stores/agents'
import AgentsRuntimeMetrics from '../components/AgentsRuntimeMetrics.vue'
import AgentsGrid from '../components/AgentsGrid.vue'
import AgentsDetail from '../components/AgentsDetail.vue'
import AgentsWorkflows from '../components/AgentsWorkflows.vue'
import AgentsRail from '../components/AgentsRail.vue'

const store = useAgentsStore()

const { data: workspaceData, isLoading, refetch } = useAiWorkspaceQuery()

watch(workspaceData, (val) => {
	if (val) {
		store.setWorkspace(val)
		store.setLoading(false)
	}
})

watch(isLoading, (val) => {
	store.setLoading(val)
})
</script>

<template>
	<section class="agents-page">
		<div class="view-header">
			<div class="view-title-with-icon">
				<span class="hero-mark small"><Icon icon="tabler:robot" width="28" height="28" /></span>
				<div>
					<h1>AI Agents</h1>
					<p>Local AI agents, runtime and run history</p>
				</div>
			</div>
			<button type="button" class="primary-button" :disabled="store.isAiLoading" @click="refetch()">
				<Icon icon="tabler:refresh" width="16" height="16" />Refresh
			</button>
		</div>

		<AgentsRuntimeMetrics
			:ai-status="store.aiStatus"
			:ai-agents="store.aiAgents"
			:ai-runs="store.aiRuns"
			:is-ai-loading="store.isAiLoading"
		/>

		<p v-if="store.aiError" class="inline-error">{{ store.aiError }}</p>

		<div class="filter-bar">
			<button type="button" class="active">Local Agents</button>
			<button type="button" disabled>{{ aiModelSummary(store.aiStatus) }}</button>
			<button type="button" disabled>{{ store.aiStatus?.chat_model_available ? 'Chat model ready' : 'Chat model missing' }}</button>
			<button type="button" disabled>{{ store.aiStatus?.embedding_model_available ? 'Embedding ready' : 'Embedding missing' }}</button>
		</div>

		<div class="agents-layout">
			<section class="agent-main">
				<AgentsGrid
					:agent-cards="store.agentCards"
					:selected-agent-index="store.selectedAgentIndex"
					:is-ai-loading="store.isAiLoading"
					@select-agent="store.selectAgent($event)"
				/>
				<AgentsDetail :selected-agent="store.selectedAgent" />
				<AgentsWorkflows
					:ai-question="store.aiQuestion"
					:ai-meeting-topic="store.aiMeetingTopic"
					:ai-task-query="store.aiTaskQuery"
					:ai-answer-result="store.aiAnswerResult"
					:ai-meeting-prep-result="store.aiMeetingPrepResult"
					:ai-task-refresh-result="store.aiTaskRefreshResult"
					:is-ai-answer-submitting="store.isAiAnswerSubmitting"
					:is-ai-meeting-prep-submitting="store.isAiMeetingPrepSubmitting"
					:is-ai-task-refresh-submitting="store.isAiTaskRefreshSubmitting"
					@update:ai-question="store.aiQuestion = $event"
					@update:ai-meeting-topic="store.aiMeetingTopic = $event"
					@update:ai-task-query="store.aiTaskQuery = $event"
					@submit-answer="store.submitAiAnswer()"
					@submit-meeting-prep="store.prepareAiBrief()"
					@refresh-tasks="store.refreshTasksFromAi()"
				/>
			</section>

			<AgentsRail
				:ai-runs="store.aiRuns"
				:ai-status="store.aiStatus"
				:owner-persona="store.ownerPersona"
				:is-ai-loading="store.isAiLoading"
			/>
		</div>
	</section>
</template>

<style scoped>
.agent-main {
	display: grid;
	gap: 12px;
	align-content: start;
	min-width: 0;
}

.agents-layout {
	display: grid;
	grid-template-columns: minmax(760px, 1fr) 310px;
	gap: 12px;
	min-height: var(--hh-widget-workbench-large);
}
</style>
```

### `frontend/src/domains/calendar/components/CalendarSourceStatus.vue`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/domains/calendar/components/CalendarSourceStatus.vue`
- Size bytes / Размер в байтах: `892`
- Included characters / Включено символов: `892`
- Truncated / Обрезано: `no`

```text
<script setup lang="ts">
import { useI18n } from '../../../platform/i18n'
import type { CalendarAccount, CalendarSource } from '../types/calendar'

const { t } = useI18n()

defineProps<{
  calendarSources: CalendarSource[]
  calendarAccounts: CalendarAccount[]
}>()
</script>

<template>
  <section class="panel info-card">
    <h2>{{ t('Calendars') }}</h2>
    <template v-if="calendarSources.length === 0">
      <label v-for="acct in calendarAccounts" :key="acct.account_id" class="mini-check">
        <input type="checkbox" checked disabled />{{ acct.account_name }}<em>{{ acct.provider }}</em>
      </label>
    </template>
    <template v-else>
      <label v-for="src in calendarSources" :key="src.source_id" class="mini-check">
        <input type="checkbox" checked disabled />{{ src.name }}<em>{{ src.timezone || '' }}</em>
      </label>
    </template>
  </section>
</template>
```

### `frontend/src/domains/calendar/components/CalendarToolbar.vue`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/domains/calendar/components/CalendarToolbar.vue`
- Size bytes / Размер в байтах: `1908`
- Included characters / Включено символов: `1908`
- Truncated / Обрезано: `no`

```text
<script setup lang="ts">
import { useI18n } from '../../../platform/i18n'
import Icon from '../../../shared/ui/Icon.vue'
import { useCalendarStore } from '../stores/calendar'
import type { CalendarViewMode } from '../types/calendar'

const { t } = useI18n()
const store = useCalendarStore()

const emit = defineEmits<{
  (e: 'search-calendar'): void
  (e: 'load-calendar'): void
  (e: 'load-weekly-brief'): void
  (e: 'refresh-all'): void
}>()

function setMode(mode: CalendarViewMode) {
  store.setViewMode(mode)
}
</script>

<template>
  <div class="widget-frame">
    <div class="view-header">
      <div class="view-title-with-icon">
        <span class="hero-mark small"><Icon icon="tabler:calendar" :size="28" /></span>
        <div>
          <h1>{{ t('Calendar') }}</h1>
          <p>{{ t('All your events from connected calendars') }}</p>
        </div>
      </div>
      <div class="search-bar">
        <input
          type="text"
          :placeholder="t('Search events...')"
          :value="store.searchQuery"
          @input="store.setSearchQuery(($event.target as HTMLInputElement).value); emit('search-calendar')"
        />
      </div>
      <div class="section-tabs pill-tabs">
        <button
          v-for="mode in (['day', 'week', 'month', 'agenda'] as CalendarViewMode[])"
          :key="mode"
          type="button"
          :class="['pill-tab', { active: store.viewMode === mode }]"
          @click="setMode(mode)"
        >{{ t(mode.charAt(0).toUpperCase() + mode.slice(1)) }}</button>
      </div>
      <button type="button" class="primary-button" @click="store.toggleNewEventForm()">
        <Icon icon="tabler:plus" :size="16" /> {{ t('New Event') }}
      </button>
      <button type="button" class="ghost-button" @click="emit('refresh-all')" :title="t('Refresh')">
        <Icon icon="tabler:refresh" :size="16" />
      </button>
    </div>
  </div>
</template>
```

### `frontend/src/domains/calendar/components/CalendarUpcoming.vue`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/domains/calendar/components/CalendarUpcoming.vue`
- Size bytes / Размер в байтах: `1134`
- Included characters / Включено символов: `1134`
- Truncated / Обрезано: `no`

```text
<script setup lang="ts">
import { useI18n } from '../../../platform/i18n'
import type { CalendarEvent } from '../types/calendar'
import { formatEventDate, formatEventTime } from '../stores/calendar'

const { t } = useI18n()

const props = defineProps<{
  calendarEvents: CalendarEvent[]
}>()

const emit = defineEmits<{
  (e: 'prepare-event', evt: CalendarEvent): void
}>()

function handleClick(evt: CalendarEvent) {
  emit('prepare-event', evt)
}
</script>

<template>
  <section class="panel info-card">
    <h2>{{ t('Upcoming') }}</h2>
    <p v-if="calendarEvents.length === 0" class="muted">{{ t('No upcoming events') }}</p>
    <template v-else>
      <div
        v-for="evt in calendarEvents.filter(e => new Date(e.start_at) >= new Date()).slice(0, 8)"
        :key="evt.event_id"
        class="deadline"
        role="button"
        tabindex="0"
        @click="handleClick(evt)"
        @keydown.enter="handleClick(evt)"
      >
        <span>{{ formatEventDate(evt.start_at) }} &middot; {{ evt.title }}</span>
        <time>{{ formatEventTime(evt.start_at) }}</time>
      </div>
    </template>
  </section>
</template>
```

### `frontend/src/domains/calendar/components/CalendarWeekGrid.vue`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/domains/calendar/components/CalendarWeekGrid.vue`
- Size bytes / Размер в байтах: `2281`
- Included characters / Включено символов: `2281`
- Truncated / Обрезано: `no`

```text
<script setup lang="ts">
import { useI18n } from '../../../platform/i18n'
import type { CalendarEvent, CalendarAccount } from '../types/calendar'
import { eventTypeTone, formatEventTime, formatEventDayShort } from '../stores/calendar'

const { t } = useI18n()

const props = defineProps<{
  weekColumns: string[]
  calendarSearchResults: CalendarEvent[]
  filteredEvents: CalendarEvent[]
  isCalendarLoading: boolean
  calendarAccounts: CalendarAccount[]
  selectedEvent: CalendarEvent | null
  onPrepareEvent: (evt: CalendarEvent) => void
}>()

function handleEventClick(evt: CalendarEvent) {
  props.onPrepareEvent(evt)
}
</script>

<template>
  <section class="panel week-board">
    <div class="week-header">
      <strong v-for="(day, i) in weekColumns" :key="i">{{ day }}</strong>
    </div>
    <div class="event-list">
      <div v-if="isCalendarLoading" class="loading-state">{{ t('Loading events...') }}</div>
      <div
        v-else-if="(calendarSearchResults.length > 0 ? calendarSearchResults : filteredEvents).length === 0"
        class="empty-state"
      >{{ t('No events') }}</div>
      <template v-else>
        <div
          v-for="evt in (calendarSearchResults.length > 0 ? calendarSearchResults : filteredEvents)"
          :key="evt.event_id"
          :class="['event-row', eventTypeTone(evt.event_type)]"
          role="button"
          tabindex="0"
          @click="handleEventClick(evt)"
          @keydown.enter="handleEventClick(evt)"
        >
          <span class="event-day">{{ formatEventDayShort(evt.start_at) }}</span>
          <span class="event-time">
            {{ formatEventTime(evt.start_at) }} - {{ formatEventTime(evt.end_at) }}
          </span>
          <strong>{{ evt.title }}</strong>
          <span class="event-type-chip">{{ evt.event_type || t('event') }}</span>
          <em v-if="evt.importance_score && evt.importance_score > 0.5" class="importance-dot high"></em>
          <em v-if="evt.readiness_score != null && evt.readiness_score < 0.5" class="importance-dot warn"></em>
        </div>
      </template>
    </div>
    <footer class="source-footer">
      <span v-for="acct in calendarAccounts" :key="acct.account_id" class="source-badge">{{ acct.account_name }}</span>
    </footer>
  </section>
</template>
```

### `frontend/src/domains/calendar/views/CalendarPage.vue`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/domains/calendar/views/CalendarPage.vue`
- Size bytes / Размер в байтах: `9903`
- Included characters / Включено символов: `9903`
- Truncated / Обрезано: `no`

```text
<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { useI18n } from '../../../platform/i18n'
import CalendarToolbar from '../components/CalendarToolbar.vue'
import CalendarWeekGrid from '../components/CalendarWeekGrid.vue'
import CalendarUpcoming from '../components/CalendarUpcoming.vue'
import CalendarSourceStatus from '../components/CalendarSourceStatus.vue'
import {
  useCalendarEventsQuery,
  useCalendarAccountsQuery
} from '../queries/useCalendarEventsQuery'
import { useCalendarStore, getWeekStart, getWeekColumns, filterWeekEvents, formatEventDateTime, eventTypeLabel } from '../stores/calendar'
import {
  fetchCalendarSources,
  fetchWeeklyBrief,
  fetchEventBrief,
  fetchEventAgenda,
  searchCalendarEvents,
  createCalendarEvent,
  fetchEventContextPack
} from '../api/calendar'
import Icon from '../../../shared/ui/Icon.vue'
import Button from '../../../shared/ui/Button.vue'
import type { CalendarEvent, CalendarSource, WeeklyBrief } from '../types/calendar'

const { t } = useI18n()
const store = useCalendarStore()

// TanStack Query data
const { data: accountsData, isLoading: isAccountsLoading } = useCalendarAccountsQuery()
const { data: eventsData, isLoading: isEventsLoading, refetch: refetchEvents } = useCalendarEventsQuery(200)

const calendarAccounts = computed(() => accountsData.value ?? [])
const calendarEvents = computed(() => eventsData.value ?? [])

// Calendar sources loaded manually (depends on accounts)
const calendarSources = ref<CalendarSource[]>([])

// Search results
const searchResults = ref<CalendarEvent[]>([])

// Week calculation
const weekStart = ref(getWeekStart())
const weekColumns = computed(() => getWeekColumns(weekStart.value))
const filteredEvents = computed(() => filterWeekEvents(calendarEvents.value, weekStart.value))

// Event detail state
const eventContext = ref<Record<string, unknown> | null>(null)

// Derived
const isLoading = computed(() => isAccountsLoading || isEventsLoading)
const displayEvents = computed(() =>
  searchResults.value.length > 0 ? searchResults.value : filteredEvents.value
)

async function loadSources() {
  const results: CalendarSource[] = []
  for (const acct of calendarAccounts.value) {
    try {
      const res = await fetchCalendarSources(acct.account_id)
      results.push(...res.items)
    } catch (_) { /* sources optional */ }
  }
  calendarSources.value = results
}

async function loadWeeklyBrief() {
  try {
    const brief = await fetchWeeklyBrief()
    store.setWeeklyBrief(brief as unknown as WeeklyBrief)
  } catch (_) {
    store.setWeeklyBrief(null)
  }
}

async function handleSearch() {
  if (!store.searchQuery.trim()) {
    searchResults.value = []
    return
  }
  try {
    const res = await searchCalendarEvents(store.searchQuery)
    searchResults.value = (res.results as CalendarEvent[]) || []
  } catch (_) {
    searchResults.value = []
  }
}

async function handlePrepareEvent(evt: CalendarEvent) {
  store.selectEvent(evt)
  try {
    const [ctx, brief, agenda] = await Promise.all([
      fetchEventContextPack(evt.event_id),
      fetchEventBrief(evt.event_id),
      fetchEventAgenda(evt.event_id)
    ])
    eventContext.value = ctx as unknown as Record<string, unknown>
    store.setEventBrief(brief)
    store.setEventAgenda(agenda as unknown as Record<string, unknown> | null)
  } catch (_) {
    eventContext.value = null
    store.setEventBrief(null)
    store.setEventAgenda(null)
  }
}

async function handleCreateEvent() {
  if (!store.newEventTitle || !store.newEventStart || !store.newEventEnd) return
  try {
    await createCalendarEvent({
      title: store.newEventTitle,
      start_at: new Date(store.newEventStart).toISOString(),
      end_at: new Date(store.newEventEnd).toISOString(),
      event_type: store.newEventType
    })
    store.resetNewEventForm()
    refetchEvents()
  } catch (e) {
    store.setCalendarError(e instanceof Error ? e.message : 'Create failed')
  }
}

async function handleRefreshAll() {
  refetchEvents()
  loadSources()
  loadWeeklyBrief()
}

onMounted(() => {
  loadSources()
  loadWeeklyBrief()
})
</script>

<template>
  <section class="calendar-page">
    <CalendarToolbar
      @search-calendar="handleSearch"
      @load-calendar="refetchEvents"
      @load-weekly-brief="loadWeeklyBrief"
      @refresh-all="handleRefreshAll"
    />

    <!-- New Event Form -->
    <div v-if="store.showNewEventForm" class="panel new-event-form">
      <h3>{{ t('New Event') }}</h3>
      <div class="form-row">
        <input
          type="text"
          :placeholder="t('Event title')"
          v-model="store.newEventTitle"
        />
        <select v-model="store.newEventType">
          <option
            v-for="opt in ['meeting', 'focus', 'deadline', 'personal', 'travel', 'tax', 'review', 'planning']"
            :key="opt"
            :value="opt"
          >{{ eventTypeLabel(opt) }}</option>
        </select>
      </div>
      <div class="form-row">
        <input type="datetime-local" v-model="store.newEventStart" />
        <span>&rarr;</span>
        <input type="datetime-local" v-model="store.newEventEnd" />
      </div>
      <div class="form-actions">
        <Button variant="default" @click="handleCreateEvent">{{ t('Create') }}</Button>
        <Button variant="ghost" @click="store.resetNewEventForm()">{{ t('Cancel') }}</Button>
      </div>
    </div>

    <!-- Filter bar -->
    <div class="filter-bar">
      <span>{{ calendarAccounts.length }} {{ t('accounts') }} &middot; {{ calendarEvents.length }} {{ t('events') }}</span>
      <span v-if="store.calendarError" class="error-text">{{ store.calendarError }}</span>
      <span v-if="searchResults.length > 0" class="search-hint">
        {{ t('Search') }}: {{ searchResults.length }} {{ t('results for') }} "{{ store.searchQuery }}"
      </span>
    </div>

    <!-- Main layout -->
    <div class="calendar-layout">
      <CalendarWeekGrid
        :week-columns="weekColumns"
        :calendar-search-results="searchResults"
        :filtered-events="filteredEvents"
        :is-calendar-loading="isAccountsLoading || isEventsLoading"
        :calendar-accounts="calendarAccounts"
        :selected-event="store.selectedEvent"
        :on-prepare-event="handlePrepareEvent"
      />
      <aside class="stacked-rail">
        <!-- Weekly Brief -->
        <div class="panel info-card">
          <h2>
            {{ t('Weekly Brief') }}
            <Button variant="ghost" size="sm" @click="loadWeeklyBrief">
              <Icon icon="tabler:refresh" :size="12" />
            </Button>
          </h2>
          <template v-if="store.weeklyBrief">
            <div class="metric-grid tiny">
              <article class="metric-card">
                <span>{{ t('Events') }}</span>
                <strong>{{ store.weeklyBrief.upcoming_events_this_week || 0 }}</strong>
              </article>
              <article class="metric-card">
                <span>{{ t('Overdue') }}</span>
                <strong>{{ store.weeklyBrief.overdue_deadlines || 0 }}</strong>
              </article>
              <article class="metric-card">
                <span>{{ t('No Notes') }}</span>
                <strong>{{ store.weeklyBrief.past_events_without_notes || 0 }}</strong>
              </article>
            </div>
          </template>
          <p v-else class="muted">{{ t('Click refresh to load') }}</p>
        </div>

        <CalendarUpcoming
          :calendar-events="calendarEvents"
          @prepare-event="handlePrepareEvent"
        />

        <!-- Event Detail -->
        <div v-if="store.selectedEvent" class="panel info-card event-detail">
          <h2>
            {{ store.selectedEvent.title }}
            <Button variant="ghost" size="sm" @click="store.selectEvent(null)">
              <Icon icon="tabler:x" :size="14" />
            </Button>
          </h2>
          <div class="event-meta">
            <span><Icon icon="tabler:clock" :size="14" /> {{ formatEventDateTime(store.selectedEvent.start_at) }}</span>
            <span v-if="store.selectedEvent.location">
              <Icon icon="tabler:map-pin" :size="14" /> {{ store.selectedEvent.location }}
            </span>
            <span :class="['chip', store.selectedEvent.status]">{{ store.selectedEvent.status }}</span>
          </div>
          <div v-if="store.eventBrief" class="brief-section">
            <h4>{{ t('Brief') }}</h4>
            <div v-if="(store.eventBrief.participants as any[])?.length" class="brief-participants">
              <span
                v-for="(p, idx) in (store.eventBrief.participants as any[])"
                :key="idx"
                class="participant-chip"
              >{{ p.name || p.email }}</span>
            </div>
            <p v-if="(store.eventBrief.context as any)?.summary" class="muted">
              {{ (store.eventBrief.context as any).summary }}
            </p>
          </div>
          <div v-if="store.eventAgenda" class="brief-section">
            <h4>{{ t('Agenda') }}</h4>
            <ul v-if="store.eventAgenda.suggested_agenda" class="agenda-list">
              <li v-for="(item, idx) in store.eventAgenda.suggested_agenda" :key="idx">{{ item }}</li>
            </ul>
          </div>
          <div class="event-actions">
            <Button variant="default" size="sm" @click="store.selectedEvent && handlePrepareEvent(store.selectedEvent)">
              <Icon icon="tabler:brain" :size="14" /> {{ t('Prepare') }}
            </Button>
            <Button variant="ghost" size="sm" @click="store.selectEvent(null)">
              <Icon icon="tabler:check" :size="14" /> {{ t('Complete') }}
            </Button>
          </div>
        </div>

        <CalendarSourceStatus
          :calendar-sources="calendarSources"
          :calendar-accounts="calendarAccounts"
        />
      </aside>
    </div>
  </section>
</template>
```

### `frontend/src/domains/communications/components/AttachmentSearchPanel.css`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/domains/communications/components/AttachmentSearchPanel.css`
- Size bytes / Размер в байтах: `4114`
- Included characters / Включено символов: `4114`
- Truncated / Обрезано: `no`

```text
.attachment-search-panel {
  border-top: 1px solid var(--hh-border);
  border-bottom: 1px solid var(--hh-border);
  background: color-mix(in srgb, var(--hh-surface-panel) 82%, transparent);
}

.attachment-search-toggle {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 0.5rem;
  width: 100%;
  min-height: 2rem;
  padding: 0.375rem 0.5rem;
  border: none;
  background: transparent;
  color: var(--hh-text-primary);
  cursor: pointer;
}

.attachment-search-title {
  display: inline-flex;
  align-items: center;
  gap: 0.375rem;
  font-size: 0.75rem;
  font-weight: 600;
}

.attachment-search-count {
  font-size: 0.6875rem;
  color: var(--hh-text-muted);
}

.attachment-search-body {
  display: flex;
  flex-direction: column;
  gap: 0.5rem;
  padding: 0 0.5rem 0.625rem;
}

.attachment-search-form {
  display: grid;
  grid-template-columns: minmax(0, 1.4fr) minmax(0, 1fr) minmax(0, 0.9fr) auto;
  gap: 0.375rem;
  align-items: end;
}

.attachment-search-field {
  min-width: 0;
  display: flex;
  flex-direction: column;
  gap: 0.25rem;
  color: var(--hh-text-secondary);
  font-size: 0.6875rem;
}

.attachment-search-field input,
.attachment-search-field select {
  height: 2rem;
  min-width: 0;
  border: 1px solid var(--hh-border);
  border-radius: var(--hh-radius-sm);
  background: var(--hh-surface-panel);
  color: var(--hh-text-primary);
  padding: 0 0.5rem;
  font: inherit;
}

.attachment-search-submit,
.attachment-search-more {
  height: 2rem;
  border: 1px solid var(--hh-border);
  border-radius: var(--hh-radius-sm);
  background: var(--hh-accent);
  color: var(--hh-accent-contrast);
  padding: 0 0.625rem;
  cursor: pointer;
}

.attachment-search-submit:disabled,
.attachment-search-more:disabled {
  opacity: 0.6;
  cursor: not-allowed;
}

.attachment-search-table-shell {
  max-height: 16rem;
  overflow: auto;
  border: 1px solid var(--hh-border);
  border-radius: var(--hh-radius-sm);
  background: var(--hh-surface-panel);
}

.attachment-search-grid {
  min-width: 680px;
  font-size: 0.75rem;
}

.attachment-search-head {
  position: sticky;
  top: 0;
  z-index: 1;
  background: color-mix(in srgb, var(--hh-bg-primary) 72%, transparent);
}

.attachment-search-virtual {
  position: relative;
}

.attachment-search-row {
  display: grid;
  grid-template-columns:
    minmax(180px, 1.35fr)
    minmax(180px, 1.3fr)
    minmax(140px, 0.9fr)
    minmax(80px, 0.5fr)
    minmax(100px, 0.6fr);
  align-items: center;
}

.attachment-search-result-row {
  position: absolute;
  left: 0;
  right: 0;
}

.attachment-search-cell {
  min-width: 0;
  padding: 0.5rem;
  border-bottom: 1px solid var(--hh-border);
  text-align: left;
  vertical-align: middle;
}

.attachment-search-cell--head {
  color: var(--hh-text-secondary);
  font-weight: 600;
}

.attachment-search-file,
.attachment-search-message {
  min-width: 0;
  display: flex;
  align-items: center;
  gap: 0.375rem;
}

.attachment-search-name,
.attachment-search-subject {
  min-width: 0;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.attachment-search-muted {
  color: var(--hh-text-muted);
}

.attachment-search-scan {
  display: inline-flex;
  align-items: center;
  width: max-content;
  max-width: 100%;
  padding: 0.125rem 0.375rem;
  border-radius: 999px;
  background: color-mix(in srgb, var(--hh-text-muted) 12%, transparent);
  color: var(--hh-text-secondary);
  font-size: 0.6875rem;
}

.attachment-search-scan.att-scan--clean {
  background: color-mix(in srgb, var(--hh-color-success) 18%, transparent);
  color: var(--hh-color-success);
}

.attachment-search-scan.att-scan--suspicious {
  background: color-mix(in srgb, var(--hh-color-warning) 18%, transparent);
  color: var(--hh-color-warning);
}

.attachment-search-scan.att-scan--danger {
  background: color-mix(in srgb, var(--hh-color-danger) 18%, transparent);
  color: var(--hh-color-danger);
}

.attachment-search-error {
  color: var(--hh-color-danger);
  font-size: 0.75rem;
}

.attachment-search-empty {
  color: var(--hh-text-muted);
  font-size: 0.75rem;
  padding: 0.75rem;
  text-align: center;
}
```

### `frontend/src/domains/communications/components/AttachmentSearchPanel.vue`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/domains/communications/components/AttachmentSearchPanel.vue`
- Size bytes / Размер в байтах: `9110`
- Included characters / Включено символов: `9110`
- Truncated / Обрезано: `no`

```text
<script setup lang="ts">
import { computed, ref } from 'vue'
import { useForm } from 'vee-validate'
import { FlexRender, getCoreRowModel, useVueTable } from '@tanstack/vue-table'
import { useVirtualizer } from '@tanstack/vue-virtual'
import Icon from '../../../shared/ui/Icon.vue'
import { attachmentIcon } from '../stores/communications'
import { useAttachmentSearchQuery } from '../queries/useCommunicationsQuery'
import { useAttachmentSearchResultPrefetch } from '../queries/communicationPrefetch'
import {
  attachmentScanStatusOptions,
  attachmentSearchFormDefaults,
  attachmentSearchFormToRequest,
  attachmentSearchVeeValidationSchema,
  type AttachmentSearchFormValues
} from '../forms/attachmentSearchForm'
import type { AttachmentSearchRequest, AttachmentSearchResult } from '../types/attachments'
import { formatAttachmentSize, scanStatusClass } from './attachmentTable'
import {
  attachmentSearchTableColumns,
  attachmentSearchTableRowId
} from './attachmentSearchTable'
import './AttachmentSearchPanel.css'

const props = defineProps<{
  accountId: string | null
}>()

const isOpen = ref(false)
const hasSubmitted = ref(false)
const submittedRequest = ref<AttachmentSearchRequest>({ limit: 50 })
const parentRef = ref<HTMLDivElement | null>(null)
const prefetchAttachmentMessage = useAttachmentSearchResultPrefetch()

const {
  errors,
  handleSubmit,
  setFieldValue,
  values: formValues
} = useForm<AttachmentSearchFormValues>({
  validationSchema: attachmentSearchVeeValidationSchema,
  initialValues: attachmentSearchFormDefaults()
})

const {
  data: resultsData,
  error,
  fetchNextPage,
  hasNextPage,
  isFetchingNextPage,
  isLoading
} = useAttachmentSearchQuery(
  () => submittedRequest.value,
  () => isOpen.value && hasSubmitted.value
)

const results = computed(() => resultsData.value ?? [])
const errorMessage = computed(() => {
  if (!error.value) return ''
  return error.value instanceof Error ? error.value.message : 'Attachment search failed'
})

const table = useVueTable({
  get data() {
    return results.value
  },
  columns: attachmentSearchTableColumns,
  getCoreRowModel: getCoreRowModel(),
  getRowId: attachmentSearchTableRowId
})

const tableRows = computed(() => table.getRowModel().rows)
const virtualOptions = computed(() => ({
  count: tableRows.value.length,
  getScrollElement: () => parentRef.value,
  estimateSize: () => 44,
  overscan: 8
}))
const virtualizer = useVirtualizer(virtualOptions)
const virtualRows = computed(() => virtualizer.value.getVirtualItems())
const totalSize = computed(() => virtualizer.value.getTotalSize())

const submitSearch = handleSubmit((values) => {
  submittedRequest.value = attachmentSearchFormToRequest(values, props.accountId)
  hasSubmitted.value = true
})

function toggleOpen() {
  isOpen.value = !isOpen.value
}

function updateSearchField(key: keyof AttachmentSearchFormValues, event: Event) {
  const input = event.target as HTMLInputElement | HTMLSelectElement
  setFieldValue(key, input.value)
}

function requestNextPage() {
  if (!hasNextPage.value || isFetchingNextPage.value) return
  void fetchNextPage()
}

function loadMore() {
  requestNextPage()
}

function handleResultPrefetch(result: AttachmentSearchResult) {
  void prefetchAttachmentMessage(result)
}

function handleResultsScroll() {
  const el = parentRef.value
  if (!el) return
  if (el.scrollTop + el.clientHeight >= el.scrollHeight - 180) {
    requestNextPage()
  }
}
</script>

<template>
  <section class="attachment-search-panel">
    <button class="attachment-search-toggle" type="button" :aria-expanded="isOpen" @click="toggleOpen">
      <span class="attachment-search-title">
        <Icon icon="tabler:paperclip" />
        Attachment search
      </span>
      <span class="attachment-search-count">{{ hasSubmitted ? `${results.length} results` : 'Metadata' }}</span>
    </button>

    <div v-if="isOpen" class="attachment-search-body">
      <form class="attachment-search-form" @submit.prevent="submitSearch">
        <label class="attachment-search-field">
          <span>Query</span>
          <input
            :value="formValues.query"
            type="text"
            autocomplete="off"
            placeholder="invoice, contract, pdf"
            @input="updateSearchField('query', $event)"
          />
          <small v-if="errors.query">{{ errors.query }}</small>
        </label>
        <label class="attachment-search-field">
          <span>Content type</span>
          <input
            :value="formValues.content_type"
            type="text"
            autocomplete="off"
            placeholder="application/pdf"
            @input="updateSearchField('content_type', $event)"
          />
          <small v-if="errors.content_type">{{ errors.content_type }}</small>
        </label>
        <label class="attachment-search-field">
          <span>Scan</span>
          <select :value="formValues.scan_status" @change="updateSearchField('scan_status', $event)">
            <option value="">Any</option>
            <option v-for="status in attachmentScanStatusOptions" :key="status" :value="status">{{ status }}</option>
          </select>
        </label>
        <button class="attachment-search-submit" type="submit" :disabled="isLoading">Search</button>
      </form>

      <p v-if="errorMessage" class="attachment-search-error">{{ errorMessage }}</p>
      <div v-else-if="hasSubmitted && results.length === 0 && !isLoading" class="attachment-search-empty">
        No attachment metadata found
      </div>
      <div
        v-else-if="results.length"
        ref="parentRef"
        class="attachment-search-table-shell"
        @scroll="handleResultsScroll"
      >
        <div class="attachment-search-grid" role="table">
          <div class="attachment-search-head" role="rowgroup">
            <div
              v-for="headerGroup in table.getHeaderGroups()"
              :key="headerGroup.id"
              class="attachment-search-row attachment-search-header-row"
              role="row"
            >
              <div
                v-for="header in headerGroup.headers"
                :key="header.id"
                class="attachment-search-cell attachment-search-cell--head"
                role="columnheader"
              >
                <FlexRender
                  v-if="!header.isPlaceholder"
                  :render="header.column.columnDef.header"
                  :props="header.getContext()"
                />
              </div>
            </div>
          </div>
          <div class="attachment-search-virtual" role="rowgroup" :style="{ height: `${totalSize}px` }">
            <div
              v-for="virtualRow in virtualRows"
              :key="String(virtualRow.key)"
              class="attachment-search-row attachment-search-result-row"
              role="row"
              :style="{
                transform: `translateY(${virtualRow.start}px)`,
                height: `${virtualRow.size}px`
              }"
              tabindex="0"
              @mouseenter="handleResultPrefetch(tableRows[virtualRow.index].original)"
              @focus="handleResultPrefetch(tableRows[virtualRow.index].original)"
            >
              <div
                v-for="cell in tableRows[virtualRow.index].getVisibleCells()"
                :key="cell.id"
                class="attachment-search-cell"
                role="cell"
              >
                <div v-if="cell.column.id === 'filename'" class="attachment-search-file">
                  <Icon :icon="attachmentIcon(tableRows[virtualRow.index].original.content_type)" />
                  <span class="attachment-search-name">
                    {{ tableRows[virtualRow.index].original.filename || 'Unnamed' }}
                  </span>
                </div>
                <div v-else-if="cell.column.id === 'message_subject'" class="attachment-search-message">
                  <Icon icon="tabler:mail" />
                  <span class="attachment-search-subject">
                    {{ tableRows[virtualRow.index].original.message_subject }}
                  </span>
                </div>
                <span v-else-if="cell.column.id === 'size'">
                  {{ formatAttachmentSize(tableRows[virtualRow.index].original.size_bytes) }}
                </span>
                <span
                  v-else-if="cell.column.id === 'scan_status'"
                  class="attachment-search-scan"
                  :class="scanStatusClass(tableRows[virtualRow.index].original.scan_status)"
                >
                  {{ tableRows[virtualRow.index].original.scan_status }}
                </span>
                <span v-else>{{ cell.getValue() }}</span>
              </div>
            </div>
          </div>
        </div>
      </div>

      <button
        v-if="hasSubmitted && hasNextPage"
        class="attachment-search-more"
        type="button"
        :disabled="isFetchingNextPage"
        @click="loadMore"
      >
        {{ isFetchingNextPage ? 'Loading...' : 'Load more' }}
      </button>
    </div>
  </section>
</template>
```

### `frontend/src/domains/communications/components/BilingualReplyPanel.vue`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/domains/communications/components/BilingualReplyPanel.vue`
- Size bytes / Размер в байтах: `8104`
- Included characters / Включено символов: `8104`
- Truncated / Обрезано: `no`

```text
<script setup lang="ts">
import { computed, ref } from 'vue'
import { useForm } from 'vee-validate'
import Button from '../../../shared/ui/Button.vue'
import Icon from '../../../shared/ui/Icon.vue'
import { usePrepareBilingualReplyFlowMutation } from '../queries/useCommunicationsQuery'
import {
  bilingualReplyFlowFormDefaults,
  bilingualReplyFlowFormToRequest,
  bilingualReplyFlowVeeValidationSchema,
  bilingualReplyToneOptions,
  type BilingualReplyFlowFormValues
} from '../forms/bilingualReplyFlowForm'
import type {
  BilingualReplyFlowResponse,
  BilingualReplyTone,
  BilingualTranslationStep
} from '../types/bilingualReplyFlow'

const props = defineProps<{
  messageId: string
}>()

const emit = defineEmits<{
  sendBilingualReply: [payload: BilingualReplyFlowResponse]
}>()

const result = ref<BilingualReplyFlowResponse | null>(null)
const prepareMutation = usePrepareBilingualReplyFlowMutation()

const {
  errors,
  handleSubmit,
  setFieldValue,
  values: formValues
} = useForm<BilingualReplyFlowFormValues>({
  validationSchema: bilingualReplyFlowVeeValidationSchema,
  initialValues: bilingualReplyFlowFormDefaults()
})

const isPreparing = computed(() => prepareMutation.isPending.value)
const errorMessage = computed(() => {
  const error = prepareMutation.error.value
  if (!error) return ''
  return error instanceof Error ? error.message : 'Bilingual reply preparation failed'
})

const submitBilingualReply = handleSubmit(async (values) => {
  result.value = await prepareMutation.mutateAsync({
    messageId: props.messageId,
    request: bilingualReplyFlowFormToRequest(values)
  })
})

function updateReplyText(event: Event): void {
  const input = event.target as HTMLTextAreaElement
  setFieldValue('replyTextRu', input.value)
}

function updateTone(event: Event): void {
  const input = event.target as HTMLSelectElement
  setFieldValue('tone', input.value as BilingualReplyTone)
}

function stepText(step: BilingualTranslationStep): string {
  return step.text ?? step.reason ?? 'Pending'
}

function stepStatus(step: BilingualTranslationStep): string {
  return step.translated ? 'translated' : 'review required'
}

function sendPreparedReply(): void {
  if (!result.value?.send_ready) return
  emit('sendBilingualReply', result.value)
}
</script>

<template>
  <section class="bilingual-reply-panel">
    <form class="bilingual-reply-form" @submit.prevent="submitBilingualReply">
      <div class="bilingual-reply-header">
        <span class="bilingual-reply-title">
          <Icon icon="tabler:language-hiragana" />
          Bilingual reply
        </span>
        <span class="bilingual-reply-state">{{ result?.send_ready ? 'Ready' : 'Review' }}</span>
      </div>

      <label class="bilingual-reply-field">
        <span>Reply in Russian</span>
        <textarea
          :value="formValues.replyTextRu"
          rows="5"
          autocomplete="off"
          @input="updateReplyText"
        />
        <small v-if="errors.replyTextRu">{{ errors.replyTextRu }}</small>
      </label>

      <label class="bilingual-reply-field bilingual-reply-tone">
        <span>Tone</span>
        <select :value="formValues.tone" @change="updateTone">
          <option v-for="tone in bilingualReplyToneOptions" :key="tone" :value="tone">
            {{ tone }}
          </option>
        </select>
        <small v-if="errors.tone">{{ errors.tone }}</small>
      </label>

      <div class="bilingual-reply-actions">
        <Button type="submit" size="sm" :loading="isPreparing" :disabled="isPreparing">
          <Icon icon="tabler:arrows-exchange" />
          Prepare
        </Button>
        <span v-if="errorMessage" class="bilingual-reply-error">{{ errorMessage }}</span>
      </div>
    </form>

    <div v-if="result" class="bilingual-reply-review">
      <section class="bilingual-reply-step">
        <div class="bilingual-reply-step-header">
          <span>Original</span>
          <span>{{ result.original.language }} {{ Math.round(result.original.confidence * 100) }}%</span>
        </div>
        <p>{{ result.original.text }}</p>
      </section>

      <section class="bilingual-reply-step">
        <div class="bilingual-reply-step-header">
          <span>Translation</span>
          <span>{{ stepStatus(result.translation) }}</span>
        </div>
        <p>{{ stepText(result.translation) }}</p>
      </section>

      <section class="bilingual-reply-step">
        <div class="bilingual-reply-step-header">
          <span>Reply in Russian</span>
          <span>{{ result.reply.tone }}</span>
        </div>
        <p>{{ result.reply.text }}</p>
      </section>

      <section class="bilingual-reply-step">
        <div class="bilingual-reply-step-header">
          <span>Back Translation</span>
          <span>{{ stepStatus(result.back_translation) }}</span>
        </div>
        <p>{{ stepText(result.back_translation) }}</p>
      </section>

      <div class="bilingual-reply-send">
        <Button
          type="button"
          size="sm"
          variant="outline"
          :disabled="!result.send_ready"
          @click="sendPreparedReply"
        >
          <Icon icon="tabler:send" />
          Open Compose
        </Button>
      </div>
    </div>
  </section>
</template>

<style scoped>
.bilingual-reply-panel {
  display: flex;
  flex-direction: column;
  gap: 0.75rem;
  padding: 0.75rem;
  border: 1px solid var(--hh-border, rgba(148, 163, 184, 0.28));
  border-radius: var(--hh-radius-sm, 0.375rem);
  background: var(--hh-surface-glass, rgba(255, 255, 255, 0.72));
  backdrop-filter: blur(18px);
  box-shadow: var(--hh-shadow-sm, 0 10px 28px rgba(15, 23, 42, 0.08));
}

.bilingual-reply-form {
  display: grid;
  grid-template-columns: minmax(0, 1fr) minmax(8rem, 12rem);
  gap: 0.75rem;
}

.bilingual-reply-header,
.bilingual-reply-actions {
  grid-column: 1 / -1;
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 0.75rem;
}

.bilingual-reply-title {
  display: inline-flex;
  align-items: center;
  gap: 0.375rem;
  color: var(--hh-text-primary, #1f2937);
  font-size: 0.8125rem;
  font-weight: 600;
}

.bilingual-reply-state {
  color: var(--hh-text-secondary, #6b7280);
  font-size: 0.75rem;
  text-transform: uppercase;
}

.bilingual-reply-field {
  display: flex;
  flex-direction: column;
  gap: 0.375rem;
  min-width: 0;
}

.bilingual-reply-field span {
  color: var(--hh-text-secondary, #6b7280);
  font-size: 0.75rem;
  font-weight: 500;
}

.bilingual-reply-field textarea,
.bilingual-reply-field select {
  width: 100%;
  border: 1px solid var(--hh-border, #d1d5db);
  border-radius: var(--hh-radius-xs, 0.25rem);
  background: var(--hh-bg-surface, #fff);
  color: var(--hh-text-primary, #1f2937);
  font: inherit;
  font-size: 0.8125rem;
}

.bilingual-reply-field textarea {
  min-height: 7rem;
  padding: 0.625rem;
  resize: vertical;
}

.bilingual-reply-field select {
  height: 2.125rem;
  padding: 0 0.5rem;
  text-transform: capitalize;
}

.bilingual-reply-field small,
.bilingual-reply-error {
  color: var(--hh-color-danger, #dc2626);
  font-size: 0.75rem;
}

.bilingual-reply-send {
  grid-column: 1 / -1;
}

.bilingual-reply-review {
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: 0.625rem;
}

.bilingual-reply-step {
  display: flex;
  flex-direction: column;
  gap: 0.375rem;
  min-width: 0;
  padding: 0.625rem;
  border: 1px solid var(--hh-border, rgba(148, 163, 184, 0.24));
  border-radius: var(--hh-radius-xs, 0.25rem);
  background: var(--hh-bg-elevated, rgba(255, 255, 255, 0.62));
}

.bilingual-reply-step-header {
  display: flex;
  justify-content: space-between;
  gap: 0.75rem;
  color: var(--hh-text-secondary, #6b7280);
  font-size: 0.75rem;
  font-weight: 600;
  text-transform: uppercase;
}

.bilingual-reply-step p {
  margin: 0;
  color: var(--hh-text-primary, #1f2937);
  font-size: 0.8125rem;
  line-height: 1.45;
  overflow-wrap: anywhere;
  white-space: pre-wrap;
}

@media (max-width: 760px) {
  .bilingual-reply-form,
  .bilingual-reply-review {
    grid-template-columns: 1fr;
  }
}
</style>
```

### `frontend/src/domains/communications/components/BulkActionsBar.vue`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/domains/communications/components/BulkActionsBar.vue`
- Size bytes / Размер в байтах: `4843`
- Included characters / Включено символов: `4843`
- Truncated / Обрезано: `no`

```text
<script setup lang="ts">
import Icon from '../../../shared/ui/Icon.vue'
import type { BulkMessageAction, BulkMessageActionRequest } from '../types/communications'
import {
  MAIL_MESSAGE_DRAG_TYPE,
  hasCommunicationMessageDragType,
  parseCommunicationMessageDragPayload
} from './mailDragDrop'

type BulkActionCommand = Omit<BulkMessageActionRequest, 'message_ids'>

const props = defineProps<{
  selectedCount: number
  isRunning: boolean
}>()

const emit = defineEmits<{
  action: [command: BulkActionCommand]
  clear: []
}>()

const actions: { action: BulkMessageAction; label: string; icon: string }[] = [
  { action: 'mark_read', label: 'Read', icon: 'tabler:mail-opened' },
  { action: 'mark_unread', label: 'Unread', icon: 'tabler:mail' },
  { action: 'archive', label: 'Archive', icon: 'tabler:archive' },
  { action: 'trash', label: 'Trash', icon: 'tabler:trash' },
  { action: 'pin', label: 'Pin', icon: 'tabler:pin' },
  { action: 'unpin', label: 'Unpin', icon: 'tabler:pinned-off' },
  { action: 'important', label: 'Important', icon: 'tabler:flag' },
  { action: 'not_important', label: 'Normal', icon: 'tabler:flag-off' }
]

const metadataActions: { command: BulkActionCommand; label: string; icon: string }[] = [
  { command: { action: 'add_label', label: 'Follow up' }, label: 'Label', icon: 'tabler:tag' },
  { command: { action: 'remove_label', label: 'Follow up' }, label: 'Unlabel', icon: 'tabler:tag-off' }
]

function handleActionDrop(event: DragEvent, action: BulkMessageAction) {
  if (props.isRunning || !event.dataTransfer) return
  const payload = parseCommunicationMessageDragPayload(event.dataTransfer.getData(MAIL_MESSAGE_DRAG_TYPE))
  if (!payload) return
  emit('action', { action })
}

function nextBusinessMorningIso() {
  const nextMorning = new Date()
  nextMorning.setDate(nextMorning.getDate() + 1)
  nextMorning.setHours(9, 0, 0, 0)
  return nextMorning.toISOString()
}

function handleActionDragOver(event: DragEvent) {
  if (!event.dataTransfer || !hasCommunicationMessageDragType(event.dataTransfer.types)) return
  event.preventDefault()
  event.dataTransfer.dropEffect = 'move'
}
</script>

<template>
  <div class="bulk-actions-bar">
    <div class="bulk-count">{{ selectedCount }} selected</div>
    <div class="bulk-buttons">
      <button
        v-for="item in actions"
        :key="item.action"
        class="bulk-button"
        type="button"
        :disabled="isRunning"
        @dragover="handleActionDragOver"
        @drop.prevent="handleActionDrop($event, item.action)"
        @click="emit('action', { action: item.action })"
      >
        <Icon :icon="item.icon" class="bulk-icon" />
        <span>{{ item.label }}</span>
      </button>
      <button
        v-for="item in metadataActions"
        :key="`${item.command.action}-${item.command.label}`"
        class="bulk-button"
        type="button"
        :disabled="isRunning"
        @click="emit('action', item.command)"
      >
        <Icon :icon="item.icon" class="bulk-icon" />
        <span>{{ item.label }}</span>
      </button>
      <button
        class="bulk-button"
        type="button"
        :disabled="isRunning"
        @click="emit('action', { action: 'snooze', snooze_until: nextBusinessMorningIso() })"
      >
        <Icon icon="tabler:clock-pause" class="bulk-icon" />
        <span>Snooze</span>
      </button>
      <button class="bulk-button icon-only" type="button" :disabled="isRunning" title="Clear selection" @click="emit('clear')">
        <Icon icon="tabler:x" class="bulk-icon" />
      </button>
    </div>
  </div>
</template>

<style scoped>
.bulk-actions-bar {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 0.5rem;
  padding: 0.5rem;
  border-right: 1px solid var(--hh-border, #e5e7eb);
  border-bottom: 1px solid var(--hh-border, #e5e7eb);
  background: color-mix(in srgb, var(--hh-bg-primary, #ffffff) 84%, transparent);
  backdrop-filter: blur(var(--hh-panel-blur));
}

.bulk-count {
  font-size: 0.75rem;
  color: var(--hh-text-secondary, #6b7280);
  white-space: nowrap;
}

.bulk-buttons {
  display: flex;
  align-items: center;
  justify-content: flex-end;
  flex-wrap: wrap;
  gap: 0.25rem;
}

.bulk-button {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  gap: 0.25rem;
  min-height: 1.75rem;
  padding: 0 0.5rem;
  border: 1px solid var(--hh-border, #e5e7eb);
  border-radius: 6px;
  background: color-mix(in srgb, var(--hh-bg-primary, #ffffff) 90%, transparent);
  color: var(--hh-text-primary, #1f2937);
  font-size: 0.75rem;
  cursor: pointer;
}

.bulk-button:hover:not(:disabled) {
  background: var(--hh-bg-hover, #f3f4f6);
}

.bulk-button:disabled {
  cursor: wait;
  opacity: 0.65;
}

.bulk-button.icon-only {
  width: 1.75rem;
  padding: 0;
}

.bulk-icon {
  width: 14px;
  height: 14px;
}
</style>
```

### `frontend/src/domains/communications/components/CommunicationFolderStrip.css`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/domains/communications/components/CommunicationFolderStrip.css`
- Size bytes / Размер в байтах: `5397`
- Included characters / Включено символов: `5397`
- Truncated / Обрезано: `no`

```text
.mail-folder-strip {
  display: flex;
  align-items: center;
  gap: 0.375rem;
  min-height: 2rem;
  overflow: hidden;
}

.mail-folder-virtual-scroll {
  flex: 1 1 auto;
  min-width: 8rem;
  height: 1.875rem;
  overflow-x: auto;
  overflow-y: hidden;
}

.mail-folder-virtual-track {
  position: relative;
  height: 1.875rem;
}

.mail-folder-virtual-row {
  position: absolute;
  top: 0;
  left: 0;
  height: 1.875rem;
  overflow: hidden;
}

.mail-folder-skeleton {
  width: 11rem;
  height: 1.75rem;
  border-radius: var(--hh-radius-sm);
  background: linear-gradient(90deg, var(--hh-surface-muted), var(--hh-surface-panel), var(--hh-surface-muted));
  animation: mail-folder-pulse 1.6s ease-in-out infinite;
}

.mail-folder-label,
.mail-folder-count {
  font-size: 0.6875rem;
  color: var(--hh-text-muted);
}

.mail-folder-item {
  display: inline-flex;
  align-items: center;
  gap: 0.25rem;
  width: 14.75rem;
  max-width: 14.75rem;
  height: 1.75rem;
  padding: 0 0.375rem;
  border: 1px solid var(--hh-border);
  border-radius: var(--hh-radius-sm);
  background: var(--hh-surface-panel);
}

.mail-folder-indent {
  display: inline-block;
  height: 1px;
  flex: 0 0 auto;
}

.mail-folder-item.active {
  border-color: var(--hh-accent);
  background: color-mix(in srgb, var(--hh-accent) 10%, var(--hh-surface-panel));
}

.mail-folder-item.dropping {
  border-color: var(--hh-accent);
  box-shadow: 0 0 0 2px color-mix(in srgb, var(--hh-accent) 24%, transparent);
}

.mail-folder-item.reordering {
  opacity: 0.72;
}

.mail-folder-reorder {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 1rem;
  height: 1.5rem;
  border: none;
  background: transparent;
  color: var(--hh-text-muted);
  cursor: grab;
  padding: 0;
}

.mail-folder-reorder:active {
  cursor: grabbing;
}

.mail-folder-select {
  display: inline-flex;
  align-items: center;
  gap: 0.375rem;
  min-width: 0;
  border: none;
  background: transparent;
  color: inherit;
  padding: 0;
  cursor: pointer;
}

.mail-folder-path {
  font-size: 0.625rem;
  color: var(--hh-text-muted);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.mail-folder-color {
  width: 0.625rem;
  height: 0.625rem;
  border-radius: 999px;
  flex: 0 0 auto;
}

.mail-folder-color--blue {
  background: #3b82f6;
}

.mail-folder-color--green {
  background: #10b981;
}

.mail-folder-color--amber {
  background: #f59e0b;
}

.mail-folder-color--red {
  background: #ef4444;
}

.mail-folder-color--violet {
  background: #8b5cf6;
}

.mail-folder-name {
  min-width: 0;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  font-size: 0.75rem;
  color: var(--hh-text-primary);
}

.mail-folder-tool {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 1.75rem;
  height: 1.75rem;
  border: 1px solid var(--hh-border);
  border-radius: var(--hh-radius-sm);
  background: var(--hh-surface-panel);
  color: var(--hh-text-secondary);
  cursor: pointer;
}

.mail-folder-tool:hover {
  color: var(--hh-text-primary);
  border-color: var(--hh-border-accent);
}

.mail-folder-tool.danger:hover {
  color: var(--hh-color-danger);
}

.mail-folder-icon {
  width: 0.875rem;
  height: 0.875rem;
}

.mail-folder-form {
  display: flex;
  flex-direction: column;
  gap: 0.75rem;
}

.mail-folder-form-row {
  display: grid;
  grid-template-columns: minmax(0, 1fr) 7rem;
  gap: 0.75rem;
}

.mail-folder-field {
  display: flex;
  flex-direction: column;
  gap: 0.25rem;
  font-size: 0.75rem;
  color: var(--hh-text-secondary);
}

.mail-folder-field input,
.mail-folder-field textarea {
  width: 100%;
  padding: 0.5rem 0.625rem;
  border: 1px solid var(--hh-border);
  border-radius: var(--hh-radius-sm);
  background: var(--hh-surface-panel);
  color: var(--hh-text-primary);
  font: inherit;
}

.mail-folder-path-preview {
  display: flex;
  align-items: center;
  min-height: 2.25rem;
  padding: 0.5rem 0.625rem;
  border: 1px solid var(--hh-border);
  border-radius: var(--hh-radius-sm);
  background: color-mix(in srgb, var(--hh-surface-panel) 72%, transparent);
  color: var(--hh-text-primary);
  font: inherit;
}

.mail-folder-error {
  font-size: 0.6875rem;
  color: var(--hh-color-danger);
}

.mail-folder-delete-impact {
  display: flex;
  flex-direction: column;
  gap: 0.25rem;
  padding: 0.625rem 0.75rem;
  border: 1px solid var(--hh-border);
  border-radius: var(--hh-radius-sm);
  background: color-mix(in srgb, var(--hh-surface-panel) 78%, transparent);
  font-size: 0.75rem;
  color: var(--hh-text-secondary);
}

.mail-folder-delete-impact p {
  margin: 0;
}

.mail-folder-delete-impact-note {
  color: var(--hh-text-muted);
}

.mail-folder-drop-status {
  font-size: 0.6875rem;
  color: var(--hh-text-muted);
  white-space: nowrap;
}

.mail-folder-dialog-button {
  height: 2rem;
  padding: 0 0.75rem;
  border: 1px solid var(--hh-border);
  border-radius: var(--hh-radius-sm);
  background: var(--hh-surface-panel);
  color: var(--hh-text-primary);
  cursor: pointer;
}

.mail-folder-dialog-button.primary {
  background: var(--hh-accent);
  color: var(--hh-accent-contrast);
  border-color: var(--hh-accent);
}

.mail-folder-dialog-button.danger {
  color: var(--hh-color-danger);
}

.mail-folder-dialog-button:disabled {
  opacity: 0.6;
  cursor: not-allowed;
}

@keyframes mail-folder-pulse {
  0%, 100% { opacity: 0.65; }
  50% { opacity: 1; }
}
```

### `frontend/src/domains/communications/components/CommunicationFolderStrip.vue`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/domains/communications/components/CommunicationFolderStrip.vue`
- Size bytes / Размер в байтах: `18447`
- Included characters / Включено символов: `12000`
- Truncated / Обрезано: `yes`

```text
<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import { useForm } from 'vee-validate'
import { useVirtualizer } from '@tanstack/vue-virtual'
import Dialog from '../../../shared/ui/Dialog.vue'
import Icon from '../../../shared/ui/Icon.vue'
import {
  useCopyMessageToFolderMutation,
  useCreateCommunicationFolderMutation,
  useDeleteCommunicationFolderMutation,
  useCommunicationFoldersQuery,
  useMoveMessageToFolderMutation,
  useUpdateCommunicationFolderMutation
} from '../queries/useCommunicationsQuery'
import {
  composeCommunicationFolderName,
  mailFolderDeleteDialogCopy,
  mailFolderFormDefaults,
  mailFolderFormToInput,
  mailFolderParentPathOptions,
  mailFolderMessageCountLabel,
  mailFolderVeeValidationSchema,
  splitCommunicationFolderName,
  validateCommunicationFolderParentPath,
  type CommunicationFolderFormValues
} from '../forms/mailFolderForm'
import {
  MAIL_MESSAGE_DRAG_TYPE,
  hasCommunicationMessageDragType,
  parseCommunicationMessageDragPayload
} from './mailDragDrop'
import './CommunicationFolderStrip.css'
import {
  createChildFolderDraft,
  mailFolderHierarchyDeleteImpact,
  mailFolderColorClass,
  orderCommunicationFolderDisplayRows,
  type CommunicationFolderDisplayRow
} from './mailFolderPresentation'
import { useCommunicationFolderReorder } from './useCommunicationFolderReorder'
import type { CommunicationFolder } from '../types/folders'

const props = defineProps<{
  accountId: string | null
  activeId: string
}>()
const emit = defineEmits<{
  select: [folderId: string]
  deleted: [folder: CommunicationFolder]
}>()

const {
  data: folderData,
  fetchNextPage,
  hasNextPage,
  isFetchingNextPage,
  isLoading
} = useCommunicationFoldersQuery(() => props.accountId || undefined)
const folders = computed(() => folderData.value ?? [])
const createMutation = useCreateCommunicationFolderMutation()
const updateMutation = useUpdateCommunicationFolderMutation()
const deleteMutation = useDeleteCommunicationFolderMutation()
const copyMessageMutation = useCopyMessageToFolderMutation()
const moveMessageMutation = useMoveMessageToFolderMutation()
const dialogOpen = ref(false)
const editingFolder = ref<CommunicationFolder | null>(null)
const deleteDialogOpen = ref(false)
const deletingFolder = ref<CommunicationFolder | null>(null)
const deleteError = ref('')
const folderPathError = ref('')
const dropTargetId = ref('')
const dropStatus = ref('')
const dropError = ref('')
const folderVirtualScrollRef = ref<HTMLDivElement | null>(null)
const parentPath = ref('')
const leafName = ref('')
const isSaving = computed(() => createMutation.isPending.value || updateMutation.isPending.value)
const isDeleting = computed(() => deleteMutation.isPending.value)
const isDropping = computed(() =>
  copyMessageMutation.isPending.value || moveMessageMutation.isPending.value || folderReorder.isReordering.value
)
const dialogTitle = computed(() => editingFolder.value ? 'Edit folder' : 'New folder')
const folderDialogDescription = computed(() =>
  editingFolder.value
    ? 'Update the local folder name, color and ordering.'
    : 'Create a local folder for the selected mail account.'
)
const deleteCopy = computed(() => {
  return deletingFolder.value ? mailFolderDeleteDialogCopy(deletingFolder.value) : null
})
const deleteImpact = computed(() => {
  return deletingFolder.value
    ? mailFolderHierarchyDeleteImpact(orderedFolders.value, deletingFolder.value.folder_id)
    : { descendantCount: 0, descendantLeafNames: [] }
})
const deleteDialogDescription = computed(() =>
  deleteCopy.value?.message ?? 'Confirm local folder deletion.'
)
const folderVirtualOptions = computed(() => ({
  count: folders.value.length,
  getScrollElement: () => folderVirtualScrollRef.value,
  estimateSize: () => 216,
  horizontal: true,
  overscan: 6
}))
const folderVirtualizer = useVirtualizer(folderVirtualOptions)
const virtualFolders = computed(() => folderVirtualizer.value.getVirtualItems())
const folderVirtualTotalSize = computed(() => folderVirtualizer.value.getTotalSize())

const {
  errors,
  handleSubmit,
  resetForm,
  setFieldValue,
  values: formValues
} = useForm<CommunicationFolderFormValues>({
  validationSchema: mailFolderVeeValidationSchema,
  initialValues: mailFolderFormDefaults()
})

const folderRows = computed<CommunicationFolderDisplayRow[]>(() => orderCommunicationFolderDisplayRows(folders.value))
const orderedFolders = computed(() => folderRows.value.map((row) => row.folder))
const folderReorder = useCommunicationFolderReorder(orderedFolders, updateMutation.mutateAsync)
const folderPathPreview = computed(() => composeCommunicationFolderName(parentPath.value, leafName.value))
const parentPathOptions = computed(() => mailFolderParentPathOptions(orderedFolders.value, editingFolder.value))
const parentPathValidationMessage = computed(() => validateCommunicationFolderParentPath(parentPath.value, editingFolder.value))

const folderIndentUnit = 0.75

function folderIndent(folder: CommunicationFolderDisplayRow): string {
  return `${Math.min(folder.depth, 8) * folderIndentUnit}rem`
}

watch(dialogOpen, (open) => {
  if (!open) {
    editingFolder.value = null
    folderPathError.value = ''
  }
})
watch(deleteDialogOpen, (open) => {
  if (!open) deletingFolder.value = null
})
watch([parentPath, leafName], () => {
  setFieldValue('name', folderPathPreview.value)
  if (folderPathError.value && !parentPathValidationMessage.value) {
    folderPathError.value = ''
  }
})

function openCreateDialog() {
  editingFolder.value = null
  resetForm({ values: mailFolderFormDefaults() })
  syncFolderNameParts('')
  setFieldValue('sort_order', 0)
  dialogOpen.value = true
}

function openCreateChildDialog(folder: CommunicationFolder) {
  editingFolder.value = null
  const draft = createChildFolderDraft(folder)
  resetForm({
    values: mailFolderFormDefaults({
      ...folder,
      folder_id: '',
      name: '',
      description: null,
      message_count: 0,
      sort_order: draft.sortOrder
    })
  })
  parentPath.value = draft.parentPath
  leafName.value = ''
  setFieldValue('name', '')
  setFieldValue('sort_order', draft.sortOrder)
  folderPathError.value = ''
  dialogOpen.value = true
}

function openEditDialog(folder: CommunicationFolder) {
  editingFolder.value = folder
  resetForm({ values: mailFolderFormDefaults(folder) })
  syncFolderNameParts(folder.name)
  dialogOpen.value = true
}

function openDeleteDialog(folder: CommunicationFolder) {
  deletingFolder.value = folder
  deleteError.value = ''
  deleteDialogOpen.value = true
}

const submitFolder = handleSubmit(async (values) => {
  if (parentPathValidationMessage.value) {
    folderPathError.value = parentPathValidationMessage.value
    return
  }
  const request = mailFolderFormToInput(values, props.accountId)
  if (editingFolder.value) {
    await updateMutation.mutateAsync({
      folderId: editingFolder.value.folder_id,
      request
    })
  } else {
    await createMutation.mutateAsync(request)
  }
  dialogOpen.value = false
})

async function confirmDeleteFolder() {
  const folder = deletingFolder.value
  if (!folder) return
  deleteError.value = ''
  try {
    await deleteMutation.mutateAsync(folder.folder_id)
    if (folder.folder_id === props.activeId) emit('deleted', folder)
    deleteDialogOpen.value = false
  } catch (error) {
    deleteError.value = error instanceof Error ? error.message : 'Folder deletion failed'
  }
}

function updateField(key: keyof CommunicationFolderFormValues, event: Event) {
  const input = event.target as HTMLInputElement
  setFieldValue(key, key === 'sort_order' ? Number(input.value) : input.value)
}

function updateParentPath(event: Event) {
  parentPath.value = (event.target as HTMLInputElement).value
}

function updateLeafName(event: Event) {
  leafName.value = (event.target as HTMLInputElement).value
}

function syncFolderNameParts(name: string) {
  const parts = splitCommunicationFolderName(name)
  parentPath.value = parts.parentPath
  leafName.value = parts.leafName
  setFieldValue('name', composeCommunicationFolderName(parts.parentPath, parts.leafName))
  folderPathError.value = ''
}

function handleFolderDragOver(event: DragEvent) {
  if (!event.dataTransfer || isDropping.value) return
  if (folderReorder.canHandleDragOver(event)) {
    event.preventDefault()
    event.dataTransfer.dropEffect = 'move'
    return
  }
  if (!hasCommunicationMessageDragType(event.dataTransfer.types)) return
  event.preventDefault()
  event.dataTransfer.dropEffect = event.altKey ? 'copy' : 'move'
}

function handleFolderVirtualScroll() {
  const el = folderVirtualScrollRef.value
  if (!el || !hasNextPage.value || isFetchingNextPage.value) return
  if (el.scrollLeft + el.clientWidth >= el.scrollWidth - 320) {
    void fetchNextPage()
  }
}

async function handleFolderDrop(event: DragEvent, folder: CommunicationFolder) {
  if (!event.dataTransfer || isDropping.value) return
  if (await folderReorder.handleDrop(event, folder)) {
    dropStatus.value = ''
    dropError.value = ''
    return
  }
  const payload = parseCommunicationMessageDragPayload(event.dataTransfer.getData(MAIL_MESSAGE_DRAG_TYPE))
  if (!payload) return

  const operation = event.altKey ? 'copy' : 'move'
  const mutation = operation === 'copy' ? copyMessageMutation : moveMessageMutation
  dropTargetId.value = folder.folder_id
  dropStatus.value = ''
  dropError.value = ''

  try {
    await Promise.all(payload.message_ids.map((messageId) =>
      mutation.mutateAsync({ folderId: folder.folder_id, messageId })
    ))
    const verb = operation === 'copy' ? 'Copied' : 'Moved'
    dropStatus.value = `${verb} ${payload.message_ids.length} message${payload.message_ids.length === 1 ? '' : 's'} to ${folder.name}`
  } catch (error) {
    dropError.value = error instanceof Error ? error.message : 'Folder drop failed'
  } finally {
    dropTargetId.value = ''
  }
}
</script>

<template>
  <div class="mail-folder-strip">
    <div v-if="isLoading" class="mail-folder-skeleton" />
    <template v-else>
      <span v-if="folders.length" class="mail-folder-label">Folders</span>
      <div
        v-if="folders.length"
        ref="folderVirtualScrollRef"
        class="mail-folder-virtual-scroll"
        @scroll="handleFolderVirtualScroll"
      >
        <div
          class="mail-folder-virtual-track"
          :style="{ width: `${folderVirtualTotalSize}px` }"
        >
          <div
            v-for="virtualFolder in virtualFolders"
            :key="String(virtualFolder.key)"
            class="mail-folder-virtual-row"
            :style="{
              width: `${virtualFolder.size}px`,
              transform: `translateX(${virtualFolder.start}px)`
            }"
          >
            <div
              class="mail-folder-item"
              :class="{ active: folderRows[virtualFolder.index].folder.folder_id === activeId, dropping: dropTargetId === folderRows[virtualFolder.index].folder.folder_id || folderReorder.targetId.value === folderRows[virtualFolder.index].folder.folder_id, reordering: folderReorder.sourceId.value === folderRows[virtualFolder.index].folder.folder_id }"
              @dragover="handleFolderDragOver"
              @drop.prevent="handleFolderDrop($event, folderRows[virtualFolder.index].folder)"
            >
              <span class="mail-folder-indent" :style="{ width: folderIndent(folderRows[virtualFolder.index]) }" />
              <button
                class="mail-folder-reorder"
                type="button"
                draggable="true"
                :title="`Reorder ${folderRows[virtualFolder.index].folder.name}`"
                @dragstart="folderReorder.handleDragStart($event, folderRows[virtualFolder.index].folder)"
                @dragend="folderReorder.handleDragEnd"
              >
                <Icon icon="tabler:grip-vertical" class="mail-folder-icon" />
              </button>
              <button
                class="mail-folder-select"
                type="button"

```
_Source file truncated after 12000 characters. / Исходный файл обрезан после 12000 символов._

### `frontend/src/domains/communications/components/CommunicationList.vue`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/domains/communications/components/CommunicationList.vue`
- Size bytes / Размер в байтах: `4956`
- Included characters / Включено символов: `4956`
- Truncated / Обрезано: `no`

```text
<script setup lang="ts">
import { ref, computed, watch } from 'vue'
import { useVirtualizer } from '@tanstack/vue-virtual'
import CommunicationListItem from './CommunicationListItem.vue'
import { useCommunicationMessagePrefetch } from '../queries/communicationPrefetch'
import type { CommunicationMessageSummary } from '../types/communications'

const props = defineProps<{
  messages: CommunicationMessageSummary[]
  selectedIndex: number
  selectedMessageIds: string[]
  isLoading: boolean
  hasNextPage: boolean
  isFetchingNextPage: boolean
}>()

const emit = defineEmits<{
  select: [index: number]
  toggleSelection: [messageId: string, extendRange: boolean]
  selectVisible: [messageIds: string[]]
  clearSelection: []
  loadMore: []
}>()

const parentRef = ref<HTMLDivElement | null>(null)
const prefetchCommunicationMessage = useCommunicationMessagePrefetch()

const virtualOptions = computed(() => ({
  count: props.messages.length,
  getScrollElement: () => parentRef.value,
  estimateSize: () => 72,
  overscan: 5
}))

const virtualizer = useVirtualizer(virtualOptions)

const virtualItems = computed(() => virtualizer.value.getVirtualItems())
const totalSize = computed(() => virtualizer.value.getTotalSize())
const selectedMessageIdSet = computed(() => new Set(props.selectedMessageIds))
const visibleMessageIds = computed(() => props.messages.map((message) => message.message_id))

function handleSelect(index: number) {
  emit('select', index)
}

function handlePrefetch(messageId: string) {
  void prefetchCommunicationMessage(messageId)
}

function handleScroll() {
  const el = parentRef.value
  if (!el || !props.hasNextPage || props.isFetchingNextPage) return
  if (el.scrollTop + el.clientHeight >= el.scrollHeight - 360) {
    emit('loadMore')
  }
}

function handleKeydown(event: KeyboardEvent) {
  if ((event.metaKey || event.ctrlKey) && event.key.toLowerCase() === 'a') {
    if (visibleMessageIds.value.length === 0) return
    event.preventDefault()
    emit('selectVisible', visibleMessageIds.value)
    return
  }

  if (event.key === 'Escape') {
    if (props.selectedMessageIds.length === 0) return
    event.preventDefault()
    emit('clearSelection')
    return
  }

  const current = props.messages[props.selectedIndex]
  if (!current) return

  if (event.code === 'Space') {
    event.preventDefault()
    emit('toggleSelection', current.message_id, event.shiftKey)
    return
  }

  if (!event.shiftKey) return
  const offset = event.key === 'ArrowDown' ? 1 : event.key === 'ArrowUp' ? -1 : 0
  if (offset === 0) return

  const nextIndex = props.selectedIndex + offset
  const next = props.messages[nextIndex]
  if (!next) return
  event.preventDefault()
  emit('select', nextIndex)
  emit('toggleSelection', next.message_id, true)
}

// Scroll selected item into view
watch(() => props.selectedIndex, (idx) => {
  if (idx >= 0) {
    virtualizer.value.scrollToIndex(idx, { align: 'center' })
  }
})
</script>

<template>
  <div
    ref="parentRef"
    class="mail-list-container"
    tabindex="0"
    role="listbox"
    aria-multiselectable="true"
    @keydown="handleKeydown"
    @scroll="handleScroll"
  >
    <div v-if="isLoading" class="mail-list-loading">
      <span>Loading messages...</span>
    </div>
    <div v-else-if="messages.length === 0" class="mail-list-empty">
      <span>No messages found</span>
    </div>
    <div
      v-else
      class="mail-list-virtual"
      :style="{ height: `${totalSize}px` }"
    >
      <div
        v-for="vitem in virtualItems"
        :key="String(vitem.key)"
        class="mail-list-row"
        :style="{
          transform: `translateY(${vitem.start}px)`,
          height: `${vitem.size}px`
        }"
      >
        <CommunicationListItem
          :message="messages[vitem.index]"
          :is-selected="vitem.index === selectedIndex"
          :is-checked="selectedMessageIdSet.has(messages[vitem.index].message_id)"
          :selected-message-ids="selectedMessageIds"
          @select="handleSelect(vitem.index)"
          @toggle-selection="emit('toggleSelection', messages[vitem.index].message_id, $event)"
          @prefetch="handlePrefetch(messages[vitem.index].message_id)"
        />
      </div>
    </div>
    <div v-if="isFetchingNextPage" class="mail-list-page-loading">
      <span>Loading more...</span>
    </div>
  </div>
</template>

<style scoped>
.mail-list-container {
  height: 100%;
  overflow-y: auto;
  overflow-x: hidden;
  position: relative;
}

.mail-list-loading,
.mail-list-empty,
.mail-list-page-loading {
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 2rem;
  font-size: 0.875rem;
  color: var(--hh-text-secondary, #6b7280);
}

.mail-list-loading,
.mail-list-empty {
  height: 100%;
}

.mail-list-page-loading {
  min-height: 3rem;
}

.mail-list-virtual {
  position: relative;
  width: 100%;
}

.mail-list-row {
  position: absolute;
  left: 0;
  right: 0;
  overflow: hidden;
}
</style>
```

### `frontend/src/domains/communications/components/CommunicationListItem.vue`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/domains/communications/components/CommunicationListItem.vue`
- Size bytes / Размер в байтах: `5625`
- Included characters / Включено символов: `5625`
- Truncated / Обрезано: `no`

```text
<script setup lang="ts">
import { computed } from 'vue'
import Icon from '../../../shared/ui/Icon.vue'
import type { CommunicationMessageSummary } from '../types/communications'
import { messageTime, communicationChannelIcon, senderLabel, conversationPreview } from '../stores/communications'
import { MAIL_MESSAGE_DRAG_TYPE, createCommunicationMessageDragPayload } from './mailDragDrop'

const props = defineProps<{
  message: CommunicationMessageSummary
  isSelected: boolean
  isChecked: boolean
  selectedMessageIds: string[]
}>()

const emit = defineEmits<{
  select: []
  toggleSelection: [extendRange: boolean]
  prefetch: []
}>()

const timeLabel = computed(() => messageTime(props.message.projected_at ?? props.message.occurred_at))
const channelIcon = computed(() => communicationChannelIcon(props.message.channel_kind))
const sender = computed(() => senderLabel(props.message.sender))
const preview = computed(() => conversationPreview(props.message))
const isUnread = computed(() => props.message.workflow_state === 'new')
const isImportant = computed(() => (props.message.importance_score ?? 0) >= 7)

function handleDragStart(event: DragEvent) {
  if (!props.isChecked || !event.dataTransfer) {
    event.preventDefault()
    return
  }
  event.dataTransfer.effectAllowed = 'move'
  event.dataTransfer.setData(MAIL_MESSAGE_DRAG_TYPE, createCommunicationMessageDragPayload(props.message.message_id, props.selectedMessageIds))
  event.dataTransfer.setData('text/plain', props.message.subject)
}
</script>

<template>
  <div
    class="mail-list-item-shell"
    :class="{ selected: isSelected, checked: isChecked, unread: isUnread }"
    role="option"
    :aria-selected="isChecked || isSelected"
    :draggable="isChecked"
    :title="isChecked ? 'Drag selected message to an action' : undefined"
    @mouseenter="emit('prefetch')"
    @dragstart="handleDragStart"
  >
    <button
      class="selection-toggle"
      type="button"
      :aria-pressed="isChecked"
      :title="isChecked ? 'Deselect message' : 'Select message'"
      @click.stop="emit('toggleSelection', $event.shiftKey)"
    >
      <Icon :icon="isChecked ? 'tabler:checkbox' : 'tabler:square'" class="selection-icon" />
    </button>
    <button class="mail-list-item" type="button" @focus="emit('prefetch')" @click="emit('select')">
      <div class="item-header">
        <div class="sender-row">
          <Icon :icon="channelIcon" class="channel-icon" />
          <span class="sender-name" :class="{ 'font-semibold': isUnread }">{{ sender }}</span>
          <span class="time-label">{{ timeLabel }}</span>
        </div>
        <div class="subject-row">
          <span v-if="isImportant" class="important-badge" title="Important">!</span>
          <span class="subject" :class="{ 'font-semibold': isUnread }">{{ message.subject }}</span>
        </div>
      </div>
      <div class="item-preview">{{ preview }}</div>
      <div v-if="message.attachment_count > 0" class="attachment-indicator">
        <Icon icon="tabler:paperclip" class="clip-icon" />
        <span>{{ message.attachment_count }}</span>
      </div>
    </button>
  </div>
</template>

<style scoped>
.mail-list-item-shell {
  display: flex;
  width: 100%;
  border-bottom: 1px solid var(--hh-border, #e5e7eb);
  background: transparent;
  transition: background-color 0.1s;
}

.mail-list-item-shell:hover {
  background-color: var(--hh-bg-hover, #f3f4f6);
}

.mail-list-item-shell.selected {
  background-color: var(--hh-bg-selected, #eff6ff);
  border-left: 3px solid var(--hh-accent, #3b82f6);
}

.mail-list-item-shell.checked {
  background-color: color-mix(in srgb, var(--hh-accent, #3b82f6) 8%, transparent);
}

.mail-list-item-shell.unread {
  background-color: var(--hh-bg-unread, #fafafa);
}

.selection-toggle {
  display: flex;
  align-items: flex-start;
  justify-content: center;
  width: 2rem;
  padding: 0.75rem 0 0 0;
  border: none;
  background: transparent;
  cursor: pointer;
  color: var(--hh-text-tertiary, #9ca3af);
  flex-shrink: 0;
}

.selection-toggle:hover {
  color: var(--hh-accent, #3b82f6);
}

.selection-icon {
  width: 16px;
  height: 16px;
}

.mail-list-item {
  display: flex;
  flex: 1;
  min-width: 0;
  flex-direction: column;
  padding: 0.625rem 0.75rem 0.625rem 0;
  border: none;
  background: transparent;
  text-align: left;
  gap: 0.25rem;
  cursor: pointer;
}

.item-header {
  display: flex;
  flex-direction: column;
  gap: 0.125rem;
}

.sender-row {
  display: flex;
  align-items: center;
  gap: 0.375rem;
}

.channel-icon {
  width: 14px;
  height: 14px;
  color: var(--hh-text-tertiary, #9ca3af);
  flex-shrink: 0;
}

.sender-name {
  flex: 1;
  font-size: 0.8125rem;
  color: var(--hh-text-primary, #1f2937);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.time-label {
  font-size: 0.6875rem;
  color: var(--hh-text-tertiary, #9ca3af);
  white-space: nowrap;
  flex-shrink: 0;
}

.subject-row {
  display: flex;
  align-items: center;
  gap: 0.25rem;
}

.important-badge {
  color: #ef4444;
  font-weight: 700;
  font-size: 0.8125rem;
  line-height: 1;
}

.subject {
  font-size: 0.8125rem;
  color: var(--hh-text-primary, #1f2937);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.item-preview {
  font-size: 0.75rem;
  color: var(--hh-text-secondary, #6b7280);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  line-height: 1.3;
}

.attachment-indicator {
  display: flex;
  align-items: center;
  gap: 0.25rem;
  font-size: 0.6875rem;
  color: var(--hh-text-tertiary, #9ca3af);
}

.clip-icon {
  width: 12px;
  height: 12px;
}
</style>
```

### `frontend/src/domains/communications/components/CommunicationViewer.vue`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/domains/communications/components/CommunicationViewer.vue`
- Size bytes / Размер в байтах: `11600`
- Included characters / Включено символов: `11600`
- Truncated / Обрезано: `no`

```text
<script setup lang="ts">
import { computed } from 'vue'
import Icon from '../../../shared/ui/Icon.vue'
import Button from '../../../shared/ui/Button.vue'
import Tabs from '../../../shared/ui/Tabs.vue'
import MessageBodyTab from './MessageBodyTab.vue'
import MessageHeadersTab from './MessageHeadersTab.vue'
import MessageAttachmentsTab from './MessageAttachmentsTab.vue'
import MessageRelatedTab from './MessageRelatedTab.vue'
import MessageTimelineTab from './MessageTimelineTab.vue'
import type {
  AiReplyResponse,
  CommunicationMessageDetailResponse,
  CommunicationMessageInsight,
  MessageContextTab,
  MessageExportFormat
} from '../types/communications'
import type {
  CommunicationAiState,
  CommunicationAiStateTransitionRequest
} from '../types/aiState'
import {
  useMessageAiStateQuery,
  useUpdateMessageAiStateMutation
} from '../queries/useCommunicationsQuery'
import { senderLabel, senderEmail, messageTime } from '../stores/communications'
import type { BilingualReplyFlowResponse } from '../types/bilingualReplyFlow'

const props = defineProps<{
  detail: CommunicationMessageDetailResponse | null
  insight: CommunicationMessageInsight | null
  activeTab: MessageContextTab
}>()

const emit = defineEmits<{
  'update:activeTab': [tab: MessageContextTab]
  reply: []
  replyAll: []
  forwardMessage: []
  redirectMessage: [recipientsText: string]
  createTask: []
  createNote: []
  translate: []
  generateAiReply: [payload: { tone: string; language: string }]
  applyAiReply: [payload: AiReplyResponse]
  reviewSecurity: []
  reviewRecipients: []
  analyze: []
  markMessageRead: []
  markMessageUnread: []
  deleteFromProvider: []
  togglePin: []
  toggleImportant: []
  mute: []
  exportMessage: [format: MessageExportFormat]
  addLabel: [label: string]
  removeLabel: [label: string]
  snoozeMessage: [until: string]
  openCompose: []
  sendBilingualReply: [payload: BilingualReplyFlowResponse]
}>()

const message = computed(() => props.detail?.message ?? null)
const sender = computed(() => message.value ? senderLabel(message.value.sender) : '')
const email = computed(() => message.value ? senderEmail(message.value.sender) : '')
const time = computed(() => message.value ? messageTime(message.value.projected_at ?? message.value.occurred_at) : '')
const messageId = computed(() => message.value?.message_id ?? null)
const {
  data: aiStateRecord,
  isFetching: isAiStateFetching
} = useMessageAiStateQuery(() => messageId.value)
const updateAiStateMutation = useUpdateMessageAiStateMutation()
const currentAiState = computed(() => aiStateRecord.value?.ai_state ?? 'NEW')
const aiStateDetail = computed(() => {
  if (aiStateRecord.value?.review_reason) return aiStateRecord.value.review_reason
  if (aiStateRecord.value?.last_error) return aiStateRecord.value.last_error
  if (isAiStateFetching.value) return 'Loading AI state...'
  return 'No review note'
})
const isAiStateUpdating = computed(() => updateAiStateMutation.isPending.value)

const tabs = [
  { id: 'message' as MessageContextTab, label: 'Message' },
  { id: 'attachments' as MessageContextTab, label: 'Attachments' },
  { id: 'headers' as MessageContextTab, label: 'Headers' },
  { id: 'related' as MessageContextTab, label: 'Related' },
  { id: 'timeline' as MessageContextTab, label: 'Timeline' }
]

function setTab(tabId: string) {
  emit('update:activeTab', tabId as MessageContextTab)
}

function transitionAiState(aiState: CommunicationAiState): void {
  const id = messageId.value
  if (!id || isAiStateUpdating.value) return

  let request: CommunicationAiStateTransitionRequest = { ai_state: aiState }
  if (aiState === 'REVIEW_REQUIRED') {
    request = { ai_state: aiState, review_reason: 'Manual review requested from Mail UI' }
  }
  if (aiState === 'FAILED') {
    request = { ai_state: aiState, last_error: 'Manual failure recorded from Mail UI' }
  }
  void updateAiStateMutation.mutateAsync({ messageId: id, request })
}
</script>

<template>
  <div class="mail-viewer">
    <!-- Empty state -->
    <div v-if="!detail" class="viewer-empty">
      <Icon icon="tabler:mail" class="empty-icon" />
      <p>Select a message to view</p>
    </div>

    <!-- Message detail -->
    <div v-else class="viewer-content">
      <!-- Header -->
      <div class="viewer-header">
        <div class="header-actions-top">
          <Button variant="ghost" size="sm" @click="emit('togglePin')">
            <Icon icon="tabler:pin" />
          </Button>
          <Button variant="ghost" size="sm" @click="emit('toggleImportant')">
            <Icon icon="tabler:star" />
          </Button>
        <Button variant="ghost" size="sm" @click="emit('mute')">
            <Icon icon="tabler:bell-off" />
          </Button>
          <Button variant="ghost" size="sm" @click="emit('markMessageRead')">
            <Icon icon="tabler:mail-opened" />
          </Button>
          <Button variant="ghost" size="sm" @click="emit('deleteFromProvider')">
            <Icon icon="tabler:trash" />
          </Button>
          <Button variant="ghost" size="sm" @click="emit('forwardMessage')">
            <Icon icon="tabler:mail-forward" />
          </Button>
        </div>
        <h2 class="viewer-subject">{{ message?.subject }}</h2>
        <div class="viewer-sender-row">
          <div class="sender-info">
            <span class="sender-name">{{ sender }}</span>
            <span class="sender-email">{{ email }}</span>
          </div>
          <span class="viewer-time">{{ time }}</span>
        </div>
        <section class="ai-state-panel" aria-label="AI state">
          <div>
            <span class="ai-state-kicker">AI state</span>
            <strong>{{ currentAiState }}</strong>
            <p>{{ aiStateDetail }}</p>
          </div>
          <div class="ai-state-actions">
            <button
              class="ai-state-action"
              type="button"
              :disabled="isAiStateUpdating"
              @click="transitionAiState('PROCESSING')"
            >
              Process
            </button>
            <button
              class="ai-state-action"
              type="button"
              :disabled="isAiStateUpdating"
              @click="transitionAiState('REVIEW_REQUIRED')"
            >
              Review
            </button>
            <button
              class="ai-state-action"
              type="button"
              :disabled="isAiStateUpdating"
              @click="transitionAiState('PROCESSED')"
            >
              Done
            </button>
            <button
              class="ai-state-action"
              type="button"
              :disabled="isAiStateUpdating"
              @click="transitionAiState('FAILED')"
            >
              Failed
            </button>
            <button
              class="ai-state-action"
              type="button"
              :disabled="isAiStateUpdating"
              @click="transitionAiState('ARCHIVED')"
            >
              Archive
            </button>
          </div>
        </section>
      </div>

      <!-- Tabs -->
      <Tabs :tabs="tabs.map(t => ({ id: t.id, label: t.label }))" :active="activeTab" @select="setTab" />

      <!-- Tab content -->
      <div class="viewer-body">
        <MessageBodyTab
          v-if="activeTab === 'message'"
          :detail="detail"
          :insight="insight"
          @reply="emit('reply')"
          @create-task="emit('createTask')"
          @create-note="emit('createNote')"
          @translate="emit('translate')"
          @generate-ai-reply="emit('generateAiReply', $event)"
          @apply-ai-reply="emit('applyAiReply', $event)"
          @review-security="emit('reviewSecurity')"
          @review-recipients="emit('reviewRecipients')"
          @analyze="emit('analyze')"
          @send-bilingual-reply="emit('sendBilingualReply', $event)"
        />
        <MessageAttachmentsTab v-else-if="activeTab === 'attachments'" :detail="detail" />
        <MessageHeadersTab v-else-if="activeTab === 'headers'" :detail="detail" />
        <MessageRelatedTab
          v-else-if="activeTab === 'related'"
          :detail="detail"
          @mark-message-read="emit('markMessageRead')"
          @mark-message-unread="emit('markMessageUnread')"
          @delete-from-provider="emit('deleteFromProvider')"
          @toggle-pin="emit('togglePin')"
          @toggle-important="emit('toggleImportant')"
          @mute="emit('mute')"
          @reply-all="emit('replyAll')"
          @forward-message="emit('forwardMessage')"
          @redirect-message="emit('redirectMessage', $event)"
          @export-message="emit('exportMessage', $event)"
          @add-label="emit('addLabel', $event)"
          @remove-label="emit('removeLabel', $event)"
          @snooze-message="emit('snoozeMessage', $event)"
        />
        <MessageTimelineTab v-else-if="activeTab === 'timeline'" :detail="detail" />
      </div>
    </div>
  </div>
</template>

<style scoped>
.mail-viewer {
  height: 100%;
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

.viewer-empty {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  height: 100%;
  color: var(--hh-text-secondary, #6b7280);
  gap: 0.75rem;
}

.empty-icon {
  width: 48px;
  height: 48px;
  opacity: 0.3;
}

.viewer-content {
  display: flex;
  flex-direction: column;
  height: 100%;
  overflow: hidden;
}

.viewer-header {
  padding: 1rem 1rem 0.5rem;
  border-bottom: 1px solid var(--hh-border, #e5e7eb);
  display: flex;
  flex-direction: column;
  gap: 0.375rem;
}

.header-actions-top {
  display: flex;
  gap: 0.25rem;
  justify-content: flex-end;
}

.viewer-subject {
  font-size: 1.125rem;
  font-weight: 600;
  color: var(--hh-text-primary, #1f2937);
  margin: 0;
  line-height: 1.3;
}

.viewer-sender-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 0.5rem;
}

.ai-state-panel {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 0.75rem;
  margin-top: 0.375rem;
  padding: 0.625rem;
  border: 1px solid var(--hh-border, #e5e7eb);
  border-radius: 8px;
  background: color-mix(in srgb, var(--hh-bg-primary, #ffffff) 86%, transparent);
  backdrop-filter: blur(var(--hh-panel-blur));
}

.ai-state-kicker {
  display: block;
  color: var(--hh-text-tertiary, #9ca3af);
  font-size: 0.6875rem;
  text-transform: uppercase;
  letter-spacing: 0;
}

.ai-state-panel strong {
  color: var(--hh-text-primary, #1f2937);
  font-size: 0.8125rem;
}

.ai-state-panel p {
  margin: 0.125rem 0 0;
  color: var(--hh-text-secondary, #6b7280);
  font-size: 0.75rem;
}

.ai-state-actions {
  display: flex;
  flex-wrap: wrap;
  justify-content: flex-end;
  gap: 0.375rem;
}

.ai-state-action {
  min-height: 1.75rem;
  border: 1px solid var(--hh-border, #e5e7eb);
  border-radius: 6px;
  background: color-mix(in srgb, var(--hh-accent, #3b82f6) 10%, transparent);
  color: var(--hh-accent, #3b82f6);
  cursor: pointer;
  font: inherit;
  font-size: 0.75rem;
  padding: 0 0.5rem;
  white-space: nowrap;
}

.ai-state-action:disabled {
  cursor: progress;
  opacity: 0.6;
}

.sender-info {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  min-width: 0;
}

.sender-name {
  font-size: 0.875rem;
  font-weight: 500;
  color: var(--hh-text-primary, #1f2937);
}

.sender-email {
  font-size: 0.75rem;
  color: var(--hh-text-tertiary, #9ca3af);
}

.viewer-time {
  font-size: 0.75rem;
  color: var(--hh-text-tertiary, #9ca3af);
  white-space: nowrap;
}

.viewer-body {
  flex: 1;
  overflow-y: auto;
  padding: 0.75rem;
}
</style>
```

### `frontend/src/domains/communications/components/CommunicationsActionBar.vue`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/domains/communications/components/CommunicationsActionBar.vue`
- Size bytes / Размер в байтах: `6459`
- Included characters / Включено символов: `6459`
- Truncated / Обрезано: `no`

```text
<script setup lang="ts">
import { computed } from 'vue'
import Icon from '../../../shared/ui/Icon.vue'
import Button from '../../../shared/ui/Button.vue'
import CommunicationsTopbarSlot from './CommunicationsTopbarSlot.vue'
import DraftStrip from './DraftStrip.vue'
import HealthStrip from './HealthStrip.vue'
import MailCertificateStrip from './MailCertificateStrip.vue'
import MailResourceOverviewStrip from './MailResourceOverviewStrip.vue'
import MailSyncSettingsStrip from '../../../shared/mailSync/MailSyncSettingsStrip.vue'
import type {
  CommunicationSectionId,
  CommunicationDraft,
  CommunicationArchitectureBlocker,
  MailSyncSettings,
  MailSyncSettingsUpdate,
  MailboxHealth,
  MessageExportResponse,
  SenderStats,
  SubscriptionSource,
  WorkflowStateCountItem
} from '../types/communications'

type SectionTab = {
  id: CommunicationSectionId
  label: string
  icon: string
}

const props = defineProps<{
  searchQuery: string
  sectionTabs: SectionTab[]
  activeSectionId: CommunicationSectionId
  stateCounts: WorkflowStateCountItem[]
  isSyncBusy: boolean
  syncStatusMessage: string
  syncError: string
  syncSettings: MailSyncSettings | null
  isSyncSettingsLoading: boolean
  isSyncSettingsSaving: boolean
  health: MailboxHealth | null
  subscriptions: SubscriptionSource[]
  topSenders: SenderStats[]
  blockers: CommunicationArchitectureBlocker[]
  areResourcesLoading: boolean
  hasMoreSubscriptions: boolean
  isLoadingMoreSubscriptions: boolean
  hasMoreTopSenders: boolean
  isLoadingMoreTopSenders: boolean
  drafts: CommunicationDraft[]
  hasMoreDrafts: boolean
  isLoadingMoreDrafts: boolean
  actionStatus: string
  actionError: string
  lastMessageExport: MessageExportResponse | null
  pageError: string
}>()

const emit = defineEmits<{
  'update:searchQuery': [query: string]
  search: []
  openAccountSetup: []
  compose: []
  syncNow: []
  updateSyncSettings: [settings: MailSyncSettingsUpdate]
  clearSyncStatus: []
  loadMoreSubscriptions: []
  loadMoreTopSenders: []
  selectSection: [sectionId: CommunicationSectionId]
  openDraft: [draft: CommunicationDraft]
  deleteDraft: [draftId: string]
  loadMoreDrafts: []
  clearPageError: []
}>()

const messageExportDownloadHref = computed(() => {
  if (!props.lastMessageExport) return ''
  const encoded = encodeURIComponent(props.lastMessageExport.content)
  return `data:${props.lastMessageExport.content_type};charset=utf-8,${encoded}`
})
</script>

<template>
  <Teleport to="#makosh-topbar-slot">
    <CommunicationsTopbarSlot
      :search-query="searchQuery"
      :is-sync-busy="isSyncBusy"
      @update:search-query="emit('update:searchQuery', $event)"
      @search="emit('search')"
      @open-account-setup="emit('openAccountSetup')"
      @compose="emit('compose')"
      @sync-now="emit('syncNow')"
    />
  </Teleport>

  <div class="communications-actionbar">
    <div v-if="syncStatusMessage || syncError" class="sync-status-bar">
      <span v-if="syncStatusMessage" class="sync-status-msg">{{ syncStatusMessage }}</span>
      <span v-if="syncError" class="sync-status-error">{{ syncError }}</span>
      <Button variant="ghost" size="sm" @click="emit('clearSyncStatus')">
        <Icon icon="tabler:x" />
      </Button>
    </div>

    <MailSyncSettingsStrip
      :settings="syncSettings"
      :is-loading="isSyncSettingsLoading"
      :is-saving="isSyncSettingsSaving"
      @update="emit('updateSyncSettings', $event)"
    />
    <HealthStrip :health="health" />
    <MailCertificateStrip />
    <MailResourceOverviewStrip
      :subscriptions="subscriptions"
      :top-senders="topSenders"
      :blockers="blockers"
      :is-loading="areResourcesLoading"
      :has-more-subscriptions="hasMoreSubscriptions"
      :is-loading-more-subscriptions="isLoadingMoreSubscriptions"
      :has-more-top-senders="hasMoreTopSenders"
      :is-loading-more-top-senders="isLoadingMoreTopSenders"
      @load-more-subscriptions="emit('loadMoreSubscriptions')"
      @load-more-top-senders="emit('loadMoreTopSenders')"
    />
    <DraftStrip
      :drafts="drafts"
      :has-more="hasMoreDrafts"
      :is-loading-more="isLoadingMoreDrafts"
      @open-draft="emit('openDraft', $event)"
      @delete-draft="emit('deleteDraft', $event)"
      @load-more="emit('loadMoreDrafts')"
    />
  </div>

  <div v-if="actionStatus" class="action-toast">
    <Icon icon="tabler:check-circle" />
    <span>{{ actionStatus }}</span>
  </div>
  <div v-if="lastMessageExport" class="action-toast export-ready">
    <Icon icon="tabler:download" />
    <span>Export ready</span>
    <a :href="messageExportDownloadHref" :download="lastMessageExport.filename">
      {{ lastMessageExport.filename }}
    </a>
  </div>
  <div v-if="actionError" class="action-toast error">
    <Icon icon="tabler:alert-circle" />
    <span>{{ actionError }}</span>
  </div>

  <div v-if="pageError" class="page-error">
    <Icon icon="tabler:alert-triangle" />
    <span>{{ pageError }}</span>
    <Button variant="ghost" size="sm" @click="emit('clearPageError')">
      <Icon icon="tabler:x" />
    </Button>
  </div>
</template>

<style scoped>
.communications-actionbar {
  flex-shrink: 0;
}

.sync-status-bar {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  padding: 0.25rem 0.75rem;
  font-size: 0.75rem;
  background: var(--hh-bg-info-light, #eff6ff);
  border-bottom: 1px solid var(--hh-border, #e5e7eb);
}

.sync-status-msg {
  flex: 1;
  color: var(--hh-accent, #3b82f6);
}

.sync-status-error {
  flex: 1;
  color: var(--hh-text-error, #ef4444);
}

.action-toast,
.page-error {
  position: fixed;
  bottom: 1rem;
  left: 50%;
  transform: translateX(-50%);
  display: flex;
  align-items: center;
  gap: 0.5rem;
  padding: 0.5rem 1rem;
  border-radius: 0.5rem;
  font-size: 0.8125rem;
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.1);
  z-index: 50;
}

.action-toast {
  background: var(--hh-bg-success-light, #f0fdf4);
  color: var(--hh-text-success, #16a34a);
  animation: toast-in 0.2s ease-out;
}

.export-ready {
  bottom: 3.75rem;
}

.export-ready a {
  color: inherit;
  font-weight: 600;
  text-decoration: underline;
  text-underline-offset: 2px;
}

.action-toast.error,
.page-error {
  background: var(--hh-bg-error-light, #fef2f2);
  color: var(--hh-text-error, #ef4444);
}

@keyframes toast-in {
  from {
    opacity: 0;
    transform: translateX(-50%) translateY(8px);
  }
  to {
    opacity: 1;
    transform: translateX(-50%) translateY(0);
  }
}

</style>
```

### `frontend/src/domains/communications/components/CommunicationsCallsPanel.vue`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/domains/communications/components/CommunicationsCallsPanel.vue`
- Size bytes / Размер в байтах: `14224`
- Included characters / Включено символов: `12000`
- Truncated / Обрезано: `yes`

```text
<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import { useI18n } from '../../../platform/i18n'
import {
  useProviderCallsQuery,
  useProviderCallTranscriptQuery,
} from '../queries/useCommunicationsQuery'
import type { ProviderCall } from '../types/communications'

type MeetingParticipant = {
  participant_id?: string | null
  display_name?: string | null
  email?: string | null
}

type MeetingRecordingRef = {
  recording_id?: string | null
  recording_type?: string | null
  download_ref?: string | null
  file_extension?: string | null
  file_size_bytes?: number | null
  recorded_at?: string | null
}

const { t } = useI18n()

const props = defineProps<{
  mode: 'calls' | 'meetings'
}>()

const selectedCallId = ref<string | null>(null)
const providerCallsQuery = useProviderCallsQuery(
  undefined,
  50,
  computed(() => (props.mode === 'meetings' ? 'zoom' : undefined))
)
const providerCalls = computed(() => providerCallsQuery.data.value ?? [])
const visibleCalls = computed(() =>
  providerCalls.value.filter((call) => matchesMode(call, props.mode))
)
const selectedCall = computed(
  () => visibleCalls.value.find((call) => call.call_id === selectedCallId.value) ?? visibleCalls.value[0] ?? null
)
const selectedTranscript = useProviderCallTranscriptQuery(
  computed(() => selectedCall.value?.call_id ?? null)
)

watch(
  visibleCalls,
  (nextCalls) => {
    if (nextCalls.length === 0) {
      selectedCallId.value = null
      return
    }
    if (!nextCalls.some((call) => call.call_id === selectedCallId.value)) {
      selectedCallId.value = nextCalls[0]?.call_id ?? null
    }
  },
  { immediate: true }
)

function matchesMode(call: ProviderCall, mode: 'calls' | 'meetings'): boolean {
  if (mode === 'calls') return true
  return meetingProvider(call) === 'zoom'
}

function metadataString(call: ProviderCall | null, key: string): string {
  const value = call?.metadata?.[key]
  return typeof value === 'string' && value.trim() ? value : '—'
}

function metadataOptionalString(call: ProviderCall | null, key: string): string | null {
  const value = call?.metadata?.[key]
  return typeof value === 'string' && value.trim() ? value : null
}

function meetingProvider(call: ProviderCall): string | null {
  const provider = call.metadata?.provider
  return typeof provider === 'string' && provider.trim() ? provider.trim() : null
}

function meetingParticipants(call: ProviderCall | null): MeetingParticipant[] {
  const value = call?.metadata?.participants
  if (!Array.isArray(value)) return []
  return value.filter((item): item is MeetingParticipant => typeof item === 'object' && item !== null)
}

function meetingRecordingRefs(call: ProviderCall | null): MeetingRecordingRef[] {
  const value = call?.metadata?.recording_refs
  if (!Array.isArray(value)) return []
  return value.filter((item): item is MeetingRecordingRef => typeof item === 'object' && item !== null)
}

function participantLabel(participant: MeetingParticipant): string {
  return participant.display_name?.trim() || participant.email?.trim() || participant.participant_id?.trim() || '—'
}

function participantSecondary(participant: MeetingParticipant): string {
  return participant.email?.trim() || participant.participant_id?.trim() || '—'
}

function recordingLabel(recording: MeetingRecordingRef): string {
  const parts = [
    recording.recording_id?.trim(),
    recording.recording_type?.trim(),
    recording.file_extension?.trim(),
  ].filter(Boolean)
  return parts.length ? parts.join(' · ') : '—'
}

function formatFileSize(value: number | null | undefined): string {
  if (typeof value !== 'number' || value <= 0) return '—'
  if (value >= 1024 * 1024 * 1024) return `${(value / (1024 * 1024 * 1024)).toFixed(1)} GB`
  if (value >= 1024 * 1024) return `${(value / (1024 * 1024)).toFixed(1)} MB`
  if (value >= 1024) return `${Math.round(value / 1024)} KB`
  return `${value} B`
}

function formatDate(value: string | null | undefined): string {
  if (!value) return '—'
  const parsed = new Date(value)
  if (Number.isNaN(parsed.getTime())) return '—'
  return new Intl.DateTimeFormat('en', {
    month: 'short',
    day: 'numeric',
    hour: '2-digit',
    minute: '2-digit',
  }).format(parsed)
}

function describeCall(call: ProviderCall): string {
  const primary = call.provider_call_id || call.call_id
  return [primary, call.call_state, call.direction].filter(Boolean).join(' · ')
}
</script>

<template>
  <section class="communications-calls-panel">
    <header class="communications-calls-panel__header">
      <div>
        <h3>{{ mode === 'meetings' ? t('Meetings') : t('Calls') }}</h3>
        <p>
          {{
            mode === 'meetings'
              ? t('Provider-neutral meeting evidence projected from calls, including Zoom meetings.')
              : t('Provider-neutral call evidence across integrations.')
          }}
        </p>
      </div>
      <span class="communications-calls-panel__count">{{ visibleCalls.length }}</span>
    </header>

    <div v-if="providerCallsQuery.isLoading.value" class="communications-calls-panel__placeholder">
      {{ t('Loading call evidence...') }}
    </div>
    <div
      v-else-if="visibleCalls.length === 0"
      class="communications-calls-panel__placeholder"
    >
      {{
        mode === 'meetings'
          ? t('No projected meetings yet.')
          : t('No projected calls yet.')
      }}
    </div>
    <div v-else class="communications-calls-panel__content">
      <div class="communications-calls-panel__list">
        <button
          v-for="call in visibleCalls"
          :key="call.call_id"
          type="button"
          class="communications-calls-panel__row"
          :class="{ 'communications-calls-panel__row--active': call.call_id === selectedCallId }"
          @click="selectedCallId = call.call_id"
        >
          <div>
            <strong>{{ describeCall(call) }}</strong>
            <p>{{ metadataString(call, 'topic') }}</p>
          </div>
          <small>{{ formatDate(call.started_at ?? call.created_at) }}</small>
        </button>
      </div>

      <div v-if="selectedCall" class="communications-calls-panel__detail">
        <header>
          <strong>{{ metadataString(selectedCall, 'topic') }}</strong>
          <small>{{ selectedCall.call_id }}</small>
        </header>
        <dl class="communications-calls-panel__meta">
          <div><dt>{{ t('Provider') }}</dt><dd>{{ meetingProvider(selectedCall) ?? '—' }}</dd></div>
          <div><dt>{{ t('Provider id') }}</dt><dd>{{ selectedCall.provider_call_id }}</dd></div>
          <div><dt>{{ t('Meeting id') }}</dt><dd>{{ metadataString(selectedCall, 'meeting_id') }}</dd></div>
          <div><dt>{{ t('Direction') }}</dt><dd>{{ selectedCall.direction }}</dd></div>
          <div><dt>{{ t('State') }}</dt><dd>{{ selectedCall.call_state }}</dd></div>
          <div><dt>{{ t('Started') }}</dt><dd>{{ formatDate(selectedCall.started_at) }}</dd></div>
          <div><dt>{{ t('Ended') }}</dt><dd>{{ formatDate(selectedCall.ended_at) }}</dd></div>
          <div><dt>{{ t('Host email') }}</dt><dd>{{ metadataString(selectedCall, 'host_email') }}</dd></div>
          <div><dt>{{ t('Transcript ref') }}</dt><dd>{{ metadataString(selectedCall, 'transcript_ref') }}</dd></div>
          <div><dt>{{ t('Join url') }}</dt><dd>{{ metadataString(selectedCall, 'join_url') }}</dd></div>
          <div><dt>{{ t('Participants') }}</dt><dd>{{ meetingParticipants(selectedCall).length || '—' }}</dd></div>
          <div><dt>{{ t('Recording refs') }}</dt><dd>{{ meetingRecordingRefs(selectedCall).length || '—' }}</dd></div>
        </dl>

        <div
          v-if="meetingParticipants(selectedCall).length > 0"
          class="communications-calls-panel__evidence"
        >
          <strong>{{ t('Participants') }}</strong>
          <div class="communications-calls-panel__chips">
            <div
              v-for="participant in meetingParticipants(selectedCall)"
              :key="participant.participant_id || participant.email || participant.display_name || 'participant'"
              class="communications-calls-panel__chip"
            >
              <span>{{ participantLabel(participant) }}</span>
              <small>{{ participantSecondary(participant) }}</small>
            </div>
          </div>
        </div>

        <div
          v-if="meetingRecordingRefs(selectedCall).length > 0"
          class="communications-calls-panel__evidence"
        >
          <strong>{{ t('Recording references') }}</strong>
          <div class="communications-calls-panel__chips">
            <div
              v-for="recording in meetingRecordingRefs(selectedCall)"
              :key="recording.recording_id || recording.download_ref || 'recording'"
              class="communications-calls-panel__chip"
            >
              <span>{{ recordingLabel(recording) }}</span>
              <small>{{ formatDate(recording.recorded_at) }} · {{ formatFileSize(recording.file_size_bytes) }}</small>
            </div>
          </div>
          <a
            v-if="metadataOptionalString(selectedCall, 'join_url')"
            class="communications-calls-panel__link"
            :href="metadataOptionalString(selectedCall, 'join_url') ?? '#'"
            target="_blank"
            rel="noreferrer"
          >
            {{ t('Open join URL') }}
          </a>
        </div>

        <div
          v-if="selectedTranscript.isLoading.value"
          class="communications-calls-panel__placeholder"
        >
          {{ t('Loading transcript evidence...') }}
        </div>
        <div
          v-else-if="!selectedTranscript.data.value"
          class="communications-calls-panel__placeholder"
        >
          {{ t('No transcript evidence projected for this call yet.') }}
        </div>
        <div v-else class="communications-calls-panel__transcript">
          <dl class="communications-calls-panel__meta">
            <div><dt>{{ t('Transcript status') }}</dt><dd>{{ selectedTranscript.data.value.transcript_status }}</dd></div>
            <div><dt>{{ t('Provider') }}</dt><dd>{{ selectedTranscript.data.value.stt_provider }}</dd></div>
            <div><dt>{{ t('Language') }}</dt><dd>{{ selectedTranscript.data.value.language_code ?? '—' }}</dd></div>
            <div><dt>{{ t('Audio ref') }}</dt><dd>{{ selectedTranscript.data.value.source_audio_ref ?? '—' }}</dd></div>
          </dl>
          <p>{{ selectedTranscript.data.value.transcript_text }}</p>
        </div>
      </div>
    </div>
  </section>
</template>

<style scoped>
.communications-calls-panel {
  display: grid;
  gap: 12px;
  padding: 16px;
}
.communications-calls-panel__header {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: 12px;
}
.communications-calls-panel__header h3,
.communications-calls-panel__header p,
.communications-calls-panel__detail header,
.communications-calls-panel__row p {
  margin: 0;
}
.communications-calls-panel__header p,
.communications-calls-panel__meta dt,
.communications-calls-panel__row small,
.communications-calls-panel__detail small {
  color: var(--hh-text-muted);
  font-size: 11px;
}
.communications-calls-panel__count {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  min-width: 24px;
  min-height: 24px;
  padding: 0 8px;
  border-radius: 999px;
  border: 1px solid var(--hh-border);
  background: color-mix(in srgb, var(--hh-surface-deep) 88%, white 12%);
  font-size: 11px;
  font-weight: 600;
}
.communications-calls-panel__content {
  display: grid;
  gap: 12px;
  grid-template-columns: minmax(280px, 0.8fr) minmax(0, 1.2fr);
}
.communications-calls-panel__list,
.communications-calls-panel__detail,
.communications-calls-panel__transcript {
  display: grid;
  gap: 8px;
}
.communications-calls-panel__row,
.communications-calls-panel__placeholder {
  padding: 10px 12px;
  border-radius: var(--hh-radius-sm);
  border: 1px solid var(--hh-border);
  background: color
```
_Source file truncated after 12000 characters. / Исходный файл обрезан после 12000 символов._

### `frontend/src/domains/communications/components/CommunicationsContextInspector.vue`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/domains/communications/components/CommunicationsContextInspector.vue`
- Size bytes / Размер в байтах: `5514`
- Included characters / Включено символов: `5514`
- Truncated / Обрезано: `no`

```text
<script setup lang="ts">
import { computed } from 'vue'
import Icon from '../../../shared/ui/Icon.vue'
import Button from '../../../shared/ui/Button.vue'
import type { CommunicationMessageDetailResponse, InspectorMode } from '../types/communications'
import { senderLabel, senderEmail } from '../stores/communications'

const props = defineProps<{
  detail: CommunicationMessageDetailResponse | null
  inspectorMode: InspectorMode
}>()

const emit = defineEmits<{
  'update:inspectorMode': [mode: InspectorMode]
}>()

const modes = [
  { id: 'context' as InspectorMode, icon: 'tabler:bulb', label: 'Context' },
  { id: 'contact' as InspectorMode, icon: 'tabler:user', label: 'Contact' },
  { id: 'organization' as InspectorMode, icon: 'tabler:building-community', label: 'Organization' }
]

const message = computed(() => props.detail?.message ?? null)
const sender = computed(() => message.value ? senderLabel(message.value.sender) : '')
const email = computed(() => message.value ? senderEmail(message.value.sender) : '')
</script>

<template>
  <div class="context-inspector">
    <div class="inspector-header">
      <Icon icon="tabler:bulb" class="inspector-icon" />
      <span class="inspector-title">Inspector</span>
    </div>

    <!-- Mode selector -->
    <div class="mode-selector">
      <Button
        v-for="(m, idx) in modes"
        :key="idx"
        variant="ghost"
        size="sm"
        :class="inspectorMode === m.id ? 'active' : ''"
        @click="emit('update:inspectorMode', m.id)"
      >
        <Icon :icon="m.icon" /> {{ m.label }}
      </Button>
    </div>

    <!-- Context panel -->
    <div v-if="!detail" class="inspector-empty">
      Select a message to inspect
    </div>
    <div v-else class="inspector-content">
      <div class="sender-profile">
        <div class="profile-avatar">{{ sender.charAt(0).toUpperCase() }}</div>
        <div class="profile-info">
          <span class="profile-name">{{ sender }}</span>
          <span class="profile-email">{{ email }}</span>
        </div>
      </div>

      <div class="inspector-section">
        <h4 class="section-title">Summary</h4>
        <p class="section-text">
          {{ message?.ai_summary || 'No AI summary available' }}
        </p>
      </div>

      <div class="inspector-section">
        <h4 class="section-title">Metadata</h4>
        <div class="meta-grid">
          <div class="meta-item">
            <span class="meta-label">Importance</span>
            <span class="meta-value">{{ message?.importance_score ?? 'N/A' }}</span>
          </div>
          <div class="meta-item">
            <span class="meta-label">Category</span>
            <span class="meta-value">{{ message?.ai_category || 'N/A' }}</span>
          </div>
          <div class="meta-item">
            <span class="meta-label">State</span>
            <span class="meta-value">{{ message?.workflow_state }}</span>
          </div>
        </div>
      </div>

      <div class="inspector-section">
        <h4 class="section-title">Attachments</h4>
        <p class="section-text">{{ detail?.attachments?.length ?? 0 }} files</p>
      </div>
    </div>
  </div>
</template>

<style scoped>
.context-inspector {
  display: flex;
  flex-direction: column;
  height: 100%;
  overflow: hidden;
}

.inspector-header {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  padding: 0.75rem;
  border-bottom: 1px solid var(--hh-border, #e5e7eb);
}

.inspector-icon {
  width: 18px;
  height: 18px;
  color: var(--hh-accent, #3b82f6);
}

.inspector-title {
  font-size: 0.875rem;
  font-weight: 600;
  color: var(--hh-text-primary, #1f2937);
}

.mode-selector {
  display: flex;
  gap: 0.25rem;
  padding: 0.375rem;
  border-bottom: 1px solid var(--hh-border, #e5e7eb);
}
.mode-selector :deep(.active) {
  background: var(--hh-bg-selected, #eff6ff);
  color: var(--hh-accent, #3b82f6);
}

.inspector-empty {
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 2rem;
  font-size: 0.8125rem;
  color: var(--hh-text-secondary, #6b7280);
}

.inspector-content {
  flex: 1;
  overflow-y: auto;
  padding: 0.75rem;
  display: flex;
  flex-direction: column;
  gap: 1rem;
}

.sender-profile {
  display: flex;
  align-items: center;
  gap: 0.75rem;
}

.profile-avatar {
  width: 40px;
  height: 40px;
  border-radius: 50%;
  background: var(--hh-accent, #3b82f6);
  color: #fff;
  display: flex;
  align-items: center;
  justify-content: center;
  font-weight: 600;
  font-size: 1rem;
  flex-shrink: 0;
}

.profile-info {
  display: flex;
  flex-direction: column;
  gap: 0.125rem;
  min-width: 0;
}

.profile-name {
  font-size: 0.875rem;
  font-weight: 600;
  color: var(--hh-text-primary, #1f2937);
}

.profile-email {
  font-size: 0.75rem;
  color: var(--hh-text-tertiary, #9ca3af);
}

.inspector-section {
  display: flex;
  flex-direction: column;
  gap: 0.375rem;
}

.section-title {
  font-size: 0.75rem;
  font-weight: 600;
  text-transform: uppercase;
  letter-spacing: 0.05em;
  color: var(--hh-text-secondary, #6b7280);
  margin: 0;
}

.section-text {
  font-size: 0.8125rem;
  color: var(--hh-text-primary, #1f2937);
  margin: 0;
  line-height: 1.4;
}

.meta-grid {
  display: flex;
  flex-direction: column;
  gap: 0.25rem;
}

.meta-item {
  display: flex;
  justify-content: space-between;
  align-items: center;
  font-size: 0.75rem;
}

.meta-label {
  color: var(--hh-text-secondary, #6b7280);
}

.meta-value {
  color: var(--hh-text-primary, #1f2937);
  font-weight: 500;
}
</style>
```
