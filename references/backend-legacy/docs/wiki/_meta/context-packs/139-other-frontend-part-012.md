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

- Chunk ID / ID чанка: `139-other-frontend-part-012`
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

### `frontend/src/integrations/zoom/components/ZoomBridgeLab.vue`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/integrations/zoom/components/ZoomBridgeLab.vue`
- Size bytes / Размер в байтах: `17904`
- Included characters / Включено символов: `12000`
- Truncated / Обрезано: `yes`

```text
<script setup lang="ts">
import { computed, ref } from 'vue'
import { useI18n } from '../../../platform/i18n'
import { useSettingsStore } from '../../../shared/zoom/settingsBridge'
import type { ProviderAccount } from '../../../shared/zoom/settingsBridge'
import type { ZoomParticipantSnapshot, ZoomRecordingRef } from '../types/zoom'
import {
  useBridgeZoomMeetingMutation,
  useBridgeZoomRecordingMutation,
  useBridgeZoomTranscriptMutation,
  useImportZoomTranscriptFileMutation,
} from '../queries/useZoomRuntimeQuery'

const props = defineProps<{
  selectedAccount?: ProviderAccount | null
}>()

const { t } = useI18n()
const store = useSettingsStore()

const activeBridgeAction = ref<string | null>(null)
const bridgeResult = ref('')
const exampleMeetingParticipants = '[{"display_name":"Alice"}]'
const exampleMeetingRecordingRefs = '[{"recording_id":"rec-1"}]'
const exampleFixtureMetadata = '{"source":"fixture"}'
const exampleRecordingMetadata = '{"channel":"cloud"}'
const exampleTranscriptSegments = '[{"text":"Hello","start_ms":0,"end_ms":1000}]'

const meetingForm = ref({
  meeting_id: '',
  meeting_uuid: '',
  topic: '',
  host_email: '',
  join_url: '',
  started_at: '',
  ended_at: '',
  duration_seconds: '',
  transcript_ref: '',
  participants_json: '[]',
  recording_refs_json: '[]',
  metadata_json: '{}',
})

const recordingForm = ref({
  meeting_id: '',
  recording_id: '',
  recording_type: '',
  download_ref: '',
  file_extension: '',
  file_size_bytes: '',
  recorded_at: '',
  metadata_json: '{}',
  request_metadata_json: '{}',
})

const transcriptForm = ref({
  transcript_id: '',
  meeting_id: '',
  meeting_uuid: '',
  source_recording_ref: '',
  language_code: '',
  transcript_text: '',
  segments_json: '[]',
  metadata_json: '{}',
})

const transcriptFileForm = ref({
  transcript_id: '',
  meeting_id: '',
  meeting_uuid: '',
  source_recording_ref: '',
  language_code: '',
  file_name: '',
  content_type: 'text/vtt',
  file_text: '',
  metadata_json: '{}',
})

const bridgeMeeting = useBridgeZoomMeetingMutation()
const bridgeRecording = useBridgeZoomRecordingMutation()
const bridgeTranscript = useBridgeZoomTranscriptMutation()
const importTranscriptFile = useImportZoomTranscriptFileMutation()

const selectedZoomAccountId = computed(() => props.selectedAccount?.account_id ?? '')

function selectedAccountIdOrError(): string | null {
  const accountId = selectedZoomAccountId.value.trim()
  if (!accountId) {
    store.setError(t('Select a Zoom account before using the bridge lab'))
    return null
  }
  return accountId
}

function parseJsonObject(raw: string, label: string): Record<string, unknown> {
  const trimmed = raw.trim()
  if (!trimmed) return {}
  const parsed = JSON.parse(trimmed)
  if (!parsed || typeof parsed !== 'object' || Array.isArray(parsed)) {
    throw new Error(`${label} must be a JSON object`)
  }
  return parsed as Record<string, unknown>
}

function parseJsonArray<T>(raw: string, label: string): T[] {
  const trimmed = raw.trim()
  if (!trimmed) return []
  const parsed = JSON.parse(trimmed)
  if (!Array.isArray(parsed)) {
    throw new Error(`${label} must be a JSON array`)
  }
  return parsed as T[]
}

function optionalNumber(raw: string): number | undefined {
  const trimmed = raw.trim()
  if (!trimmed) return undefined
  const parsed = Number.parseInt(trimmed, 10)
  if (!Number.isFinite(parsed)) throw new Error('Numeric field must be a valid integer')
  return parsed
}

function optionalString(raw: string): string | undefined {
  const trimmed = raw.trim()
  return trimmed ? trimmed : undefined
}

async function handleBridgeMeeting() {
  const accountId = selectedAccountIdOrError()
  if (!accountId) return
  activeBridgeAction.value = 'meeting'
  try {
    const result = await bridgeMeeting.mutateAsync({
      account_id: accountId,
      meeting_id: meetingForm.value.meeting_id.trim(),
      meeting_uuid: optionalString(meetingForm.value.meeting_uuid),
      topic: optionalString(meetingForm.value.topic),
      host_email: optionalString(meetingForm.value.host_email),
      join_url: optionalString(meetingForm.value.join_url),
      started_at: optionalString(meetingForm.value.started_at),
      ended_at: optionalString(meetingForm.value.ended_at),
      duration_seconds: optionalNumber(meetingForm.value.duration_seconds),
      transcript_ref: optionalString(meetingForm.value.transcript_ref),
      participants: parseJsonArray<ZoomParticipantSnapshot>(meetingForm.value.participants_json, 'Participants'),
      recording_refs: parseJsonArray<ZoomRecordingRef>(meetingForm.value.recording_refs_json, 'Recording refs'),
      metadata: parseJsonObject(meetingForm.value.metadata_json, 'Meeting metadata'),
    })
    bridgeResult.value = `meeting:${result.call_id}:${result.event_id}`
    store.setActionMessage(t('Zoom meeting observation ingested'))
  } catch (err) {
    store.setError(err instanceof Error ? err.message : 'Zoom meeting bridge ingest failed')
  } finally {
    activeBridgeAction.value = null
  }
}

async function handleBridgeRecording() {
  const accountId = selectedAccountIdOrError()
  if (!accountId) return
  activeBridgeAction.value = 'recording'
  try {
    const result = await bridgeRecording.mutateAsync({
      account_id: accountId,
      meeting_id: recordingForm.value.meeting_id.trim(),
      recording: {
        recording_id: recordingForm.value.recording_id.trim(),
        recording_type: optionalString(recordingForm.value.recording_type),
        download_ref: optionalString(recordingForm.value.download_ref),
        file_extension: optionalString(recordingForm.value.file_extension),
        file_size_bytes: optionalNumber(recordingForm.value.file_size_bytes),
        recorded_at: optionalString(recordingForm.value.recorded_at),
        metadata: parseJsonObject(recordingForm.value.metadata_json, 'Recording metadata'),
      },
      metadata: parseJsonObject(recordingForm.value.request_metadata_json, 'Recording request metadata'),
    })
    bridgeResult.value = `recording:${result.recording_id}:${result.event_id}`
    store.setActionMessage(t('Zoom recording observation ingested'))
  } catch (err) {
    store.setError(err instanceof Error ? err.message : 'Zoom recording bridge ingest failed')
  } finally {
    activeBridgeAction.value = null
  }
}

async function handleBridgeTranscript() {
  const accountId = selectedAccountIdOrError()
  if (!accountId) return
  activeBridgeAction.value = 'transcript'
  try {
    const result = await bridgeTranscript.mutateAsync({
      account_id: accountId,
      transcript_id: transcriptForm.value.transcript_id.trim(),
      meeting_id: transcriptForm.value.meeting_id.trim(),
      meeting_uuid: optionalString(transcriptForm.value.meeting_uuid),
      source_recording_ref: optionalString(transcriptForm.value.source_recording_ref),
      language_code: optionalString(transcriptForm.value.language_code),
      transcript_text: transcriptForm.value.transcript_text.trim(),
      segments: parseJsonArray(transcriptForm.value.segments_json, 'Transcript segments'),
      metadata: parseJsonObject(transcriptForm.value.metadata_json, 'Transcript metadata'),
    })
    bridgeResult.value = `transcript:${result.transcript_id}:${result.event_id}`
    store.setActionMessage(t('Zoom transcript observation ingested'))
  } catch (err) {
    store.setError(err instanceof Error ? err.message : 'Zoom transcript bridge ingest failed')
  } finally {
    activeBridgeAction.value = null
  }
}

async function handleImportTranscriptFile() {
  const accountId = selectedAccountIdOrError()
  if (!accountId) return
  activeBridgeAction.value = 'transcript-file'
  try {
    const result = await importTranscriptFile.mutateAsync({
      account_id: accountId,
      transcript_id: transcriptFileForm.value.transcript_id.trim(),
      meeting_id: transcriptFileForm.value.meeting_id.trim(),
      meeting_uuid: optionalString(transcriptFileForm.value.meeting_uuid),
      source_recording_ref: optionalString(transcriptFileForm.value.source_recording_ref),
      language_code: optionalString(transcriptFileForm.value.language_code),
      file_name: optionalString(transcriptFileForm.value.file_name),
      content_type: optionalString(transcriptFileForm.value.content_type),
      file_text: transcriptFileForm.value.file_text,
      metadata: parseJsonObject(transcriptFileForm.value.metadata_json, 'Transcript file metadata'),
    })
    bridgeResult.value = `transcript-file:${result.transcript_id}:${result.event_id}:${result.import_format}`
    store.setActionMessage(t('Zoom transcript file imported'))
  } catch (err) {
    store.setError(err instanceof Error ? err.message : 'Zoom transcript file import failed')
  } finally {
    activeBridgeAction.value = null
  }
}
</script>

<template>
  <section class="integration-section bridge-lab">
    <header class="bridge-lab-header">
      <div>
        <h4>{{ t('Runtime bridge lab') }}</h4>
        <p>{{ t('Use the selected Zoom account to validate meetings, recordings and transcript ingestion.') }}</p>
      </div>
      <div class="bridge-account">
        <span>{{ t('Account') }}</span>
        <strong>{{ selectedZoomAccountId || '-' }}</strong>
      </div>
    </header>

    <div v-if="bridgeResult" class="bridge-result">
      <span>{{ t('Last result') }}</span>
      <code>{{ bridgeResult }}</code>
    </div>

    <div class="bridge-grid">
      <form class="bridge-card" @submit.prevent="handleBridgeMeeting">
        <h5>{{ t('Meeting observation') }}</h5>
        <input v-model="meetingForm.meeting_id" class="makosh-input-control" type="text" :placeholder="t('Meeting id')" required />
        <input v-model="meetingForm.meeting_uuid" class="makosh-input-control" type="text" :placeholder="t('Meeting UUID')" />
        <input v-model="meetingForm.topic" class="makosh-input-control" type="text" :placeholder="t('Topic')" />
        <input v-model="meetingForm.host_email" class="makosh-input-control" type="email" :placeholder="t('host@example.test')" />
        <input v-model="meetingForm.join_url" class="makosh-input-control" type="text" :placeholder="t('https://example.zoom.us/j/...')" />
        <input v-model="meetingForm.started_at" class="makosh-input-control" type="text" :placeholder="t('2026-06-28T12:00:00Z')" />
        <input v-model="meetingForm.ended_at" class="makosh-input-control" type="text" :placeholder="t('2026-06-28T12:30:00Z')" />
        <input v-model="meetingForm.duration_seconds" class="makosh-input-control" type="number" min="0" :placeholder="t('1800')" />
        <input v-model="meetingForm.transcript_ref" class="makosh-input-control" type="text" :placeholder="t('provider transcript ref')" />
        <textarea v-model="meetingForm.participants_json" class="bridge-textarea" rows="4" :placeholder="exampleMeetingParticipants" />
        <textarea v-model="meetingForm.recording_refs_json" class="bridge-textarea" rows="4" :placeholder="exampleMeetingRecordingRefs" />
        <textarea v-model="meetingForm.metadata_json" class="bridge-textarea" rows="4" :placeholder="exampleFixtureMetadata" />
        <button type="submit" class="makosh-btn makosh-btn--outline"
          :disabled="activeBridgeAction === 'meeting' || bridgeMeeting.isPending.value">
          {{ bridgeMeeting.isPending.value ? t('Ingesting...') : t('Ingest meeting') }}
        </button>
      </form>

      <form class="bridge-card" @submit.prevent="handleBridgeRecording">
        <h5>{{ t('Recording observation') }}</h5>
        <input v-model="recordingForm.meeting_id" class="makosh-input-control" type="text" :placeholder="t('Meeting id')" required />
        <input v-model="recordingForm.recording_id" class="makosh-input-control" type="text" :placeholder="t('Recording id')" required />
        <input v-model="recordingForm.recording_type" class="makosh-input-control" type="text" :placeholder="t('shared_screen_with_speaker_view')" />
        <input v-model="recordingForm.download_re
```
_Source file truncated after 12000 characters. / Исходный файл обрезан после 12000 символов._

### `frontend/src/integrations/zoom/components/ZoomObservedCallsPanel.vue`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/integrations/zoom/components/ZoomObservedCallsPanel.vue`
- Size bytes / Размер в байтах: `10629`
- Included characters / Включено символов: `10608`
- Truncated / Обрезано: `no`

```text
<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import { useI18n } from '../../../platform/i18n'
import type { ProviderAccount } from '../../../shared/zoom/settingsBridge'
import { useZoomCallTranscriptQuery, useZoomProviderCallsQuery } from '../queries/useZoomRuntimeQuery'
import { extractZoomRecordingRefs, formatZoomTranscriptProvenance } from './zoomEvidence'

const { t } = useI18n()

const props = defineProps<{
  selectedAccount?: ProviderAccount | null
}>()

const selectedCallId = ref<string | null>(null)
const providerCallsQuery = useZoomProviderCallsQuery(
  computed(() => props.selectedAccount?.account_id ?? null),
  12
)
const calls = computed(() => providerCallsQuery.data.value ?? [])
const transcriptQuery = useZoomCallTranscriptQuery(selectedCallId)
const selectedCall = computed(
  () => calls.value.find((call) => call.call_id === selectedCallId.value) ?? calls.value[0] ?? null
)
const selectedTranscript = computed(() => transcriptQuery.data.value)
const selectedRecordingRefs = computed(() => extractZoomRecordingRefs(selectedCall.value?.metadata))
const selectedTranscriptProvenance = computed(() =>
  formatZoomTranscriptProvenance(selectedTranscript.value?.provenance)
)

watch(
  calls,
  (nextCalls) => {
    if (nextCalls.length === 0) {
      selectedCallId.value = null
      return
    }
    const hasSelectedCall = nextCalls.some((call) => call.call_id === selectedCallId.value)
    if (!hasSelectedCall) selectedCallId.value = nextCalls[0]?.call_id ?? null
  },
  { immediate: true }
)

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

function describeCall(call: { provider_call_id: string; call_state: string; direction: string }): string {
  return [call.provider_call_id, call.call_state, call.direction].filter(Boolean).join(' · ')
}

function metadataString(key: string): string {
  const value = selectedCall.value?.metadata?.[key]
  return typeof value === 'string' && value.trim().length > 0 ? value : '—'
}

function formatBytes(value: number | null | undefined): string {
  if (typeof value !== 'number' || !Number.isFinite(value) || value < 0) return '—'
  if (value < 1024) return `${value} B`
  if (value < 1024 * 1024) return `${(value / 1024).toFixed(1)} KB`
  return `${(value / (1024 * 1024)).toFixed(1)} MB`
}
</script>

<template>
  <section class="integration-section zoom-observed-calls">
    <header class="zoom-observed-calls__header">
      <div>
        <h4>{{ t('Observed calls') }}</h4>
        <p>{{ t('Projected provider call evidence for the selected Zoom account.') }}</p>
      </div>
      <span class="zoom-observed-calls__count">{{ calls.length }}</span>
    </header>

    <div v-if="!selectedAccount?.account_id" class="zoom-observed-calls__placeholder">
      {{ t('Select a Zoom account to inspect observed calls.') }}
    </div>
    <div v-else-if="providerCallsQuery.isLoading.value" class="zoom-observed-calls__placeholder">
      {{ t('Loading Zoom call evidence...') }}
    </div>
    <div v-else-if="calls.length === 0" class="zoom-observed-calls__placeholder">
      {{ t('No Zoom calls projected for this account yet.') }}
    </div>
    <div v-else class="zoom-observed-calls__content">
      <div class="zoom-observed-calls__list">
        <button
          v-for="call in calls"
          :key="call.call_id"
          type="button"
          class="zoom-observed-calls__row"
          :class="{ 'zoom-observed-calls__row--active': call.call_id === selectedCallId }"
          @click="selectedCallId = call.call_id"
        >
          <div>
            <strong>{{ describeCall(call) }}</strong>
            <p>{{ call.provider_chat_id || '—' }}</p>
          </div>
          <small>{{ formatDate(call.started_at ?? call.created_at) }}</small>
        </button>
      </div>

      <div v-if="selectedCall" class="zoom-observed-calls__detail">
        <header>
          <strong>{{ t('Transcript evidence') }}</strong>
          <small>{{ selectedCall.call_id }}</small>
        </header>
        <dl class="zoom-observed-calls__meta">
          <div><dt>{{ t('Meeting id') }}</dt><dd>{{ selectedCall.provider_call_id }}</dd></div>
          <div><dt>{{ t('Direction') }}</dt><dd>{{ selectedCall.direction }}</dd></div>
          <div><dt>{{ t('State') }}</dt><dd>{{ selectedCall.call_state }}</dd></div>
          <div><dt>{{ t('Started') }}</dt><dd>{{ formatDate(selectedCall.started_at) }}</dd></div>
          <div><dt>{{ t('Topic') }}</dt><dd>{{ metadataString('topic') }}</dd></div>
          <div><dt>{{ t('Host email') }}</dt><dd>{{ metadataString('host_email') }}</dd></div>
        </dl>

        <div class="zoom-observed-calls__section">
          <header>
            <strong>{{ t('Recording references') }}</strong>
            <small>{{ selectedRecordingRefs.length }}</small>
          </header>
          <div v-if="selectedRecordingRefs.length === 0" class="zoom-observed-calls__placeholder">
            {{ t('No recording references projected for this call yet.') }}
          </div>
          <div v-else class="zoom-observed-calls__recordings">
            <article v-for="recording in selectedRecordingRefs" :key="recording.recording_id" class="zoom-observed-calls__recording">
              <dl class="zoom-observed-calls__meta">
                <div><dt>{{ t('Recording id') }}</dt><dd>{{ recording.recording_id }}</dd></div>
                <div><dt>{{ t('Type') }}</dt><dd>{{ recording.recording_type ?? '—' }}</dd></div>
                <div><dt>{{ t('Format') }}</dt><dd>{{ recording.file_extension ?? '—' }}</dd></div>
                <div><dt>{{ t('Size') }}</dt><dd>{{ formatBytes(recording.file_size_bytes) }}</dd></div>
                <div><dt>{{ t('Recorded') }}</dt><dd>{{ formatDate(recording.recorded_at) }}</dd></div>
                <div><dt>{{ t('Download ref') }}</dt><dd>{{ recording.download_ref ?? '—' }}</dd></div>
              </dl>
            </article>
          </div>
        </div>

        <div v-if="transcriptQuery.isLoading.value" class="zoom-observed-calls__placeholder">
          {{ t('Loading Zoom transcript evidence...') }}
        </div>
        <div v-else-if="!selectedTranscript" class="zoom-observed-calls__placeholder">
          {{ t('No transcript evidence projected for this call yet.') }}
        </div>
        <div v-else class="zoom-observed-calls__transcript">
          <dl class="zoom-observed-calls__meta">
            <div><dt>{{ t('Status') }}</dt><dd>{{ selectedTranscript.transcript_status }}</dd></div>
            <div><dt>{{ t('Provider') }}</dt><dd>{{ selectedTranscript.stt_provider }}</dd></div>
            <div><dt>{{ t('Language') }}</dt><dd>{{ selectedTranscript.language_code ?? '—' }}</dd></div>
            <div><dt>{{ t('Audio ref') }}</dt><dd>{{ selectedTranscript.source_audio_ref ?? '—' }}</dd></div>
          </dl>
          <p>{{ selectedTranscript.transcript_text }}</p>
          <div class="zoom-observed-calls__section">
            <header>
              <strong>{{ t('Transcript provenance') }}</strong>
              <small>{{ selectedTranscript.transcript_id }}</small>
            </header>
            <pre class="zoom-observed-calls__provenance">{{ selectedTranscriptProvenance }}</pre>
          </div>
        </div>
      </div>
    </div>
  </section>
</template>

<style scoped>
.zoom-observed-calls { display: grid; gap: 12px; }
.zoom-observed-calls__header {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: 12px;
}
.zoom-observed-calls__header h4,
.zoom-observed-calls__header p {
  margin: 0;
}
.zoom-observed-calls__header p,
.zoom-observed-calls__meta dt,
.zoom-observed-calls__row small,
.zoom-observed-calls__detail small {
  color: var(--hh-text-muted);
  font-size: 11px;
}
.zoom-observed-calls__count {
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
.zoom-observed-calls__content {
  display: grid;
  gap: 12px;
  grid-template-columns: minmax(0, 0.9fr) minmax(0, 1.1fr);
}
.zoom-observed-calls__list,
.zoom-observed-calls__detail,
.zoom-observed-calls__transcript,
.zoom-observed-calls__recordings,
.zoom-observed-calls__section {
  display: grid;
  gap: 8px;
}
.zoom-observed-calls__row,
.zoom-observed-calls__placeholder {
  padding: 10px 12px;
  border-radius: var(--hh-radius-sm);
  border: 1px solid var(--hh-border);
  background: color-mix(in srgb, var(--hh-surface-deep) 88%, white 12%);
  font-size: 12px;
}
.zoom-observed-calls__row {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: 8px;
  text-align: left;
  cursor: pointer;
  color: var(--hh-text-primary);
}
.zoom-observed-calls__row--active {
  border-color: var(--hh-accent);
  box-shadow: inset 0 0 0 1px color-mix(in srgb, var(--hh-accent) 32%, transparent);
}
.zoom-observed-calls__row strong,
.zoom-observed-calls__detail strong {
  display: block;
  font-size: 12px;
}
.zoom-observed-calls__row p,
.zoom-observed-calls__transcript p {
  margin: 2px 0 0;
  font-size: 12px;
  word-break: break-word;
}
.zoom-observed-calls__detail {
  align-content: start;
}
.zoom-observed-calls__detail header {
  display: flex;
  align-items: baseline;
  justify-content: space-between;
  gap: 8px;
}
.zoom-observed-calls__section header {
  display: flex;
  align-items: baseline;
  justify-content: space-between;
  gap: 8px;
}
.zoom-observed-calls__meta {
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: 8px 12px;
  margin: 0;
}
.zoom-observed-calls__meta dd {
  margin: 2px 0 0;
  font-size: 12px;
  word-break: break-word;
}
.zoom-observed-calls__recording,
.zoom-observed-calls__provenance {
  padding: 10px 12px;
  border-radius: var(--hh-radius-sm);
  border: 1px solid var(--hh-border);
  background: color-mix(in srgb, var(--hh-surface-deep) 88%, white 12%);
}
.zoom-observed-calls__provenance {
  margin: 0;
  overflow-x: auto;
  white-space: pre-wrap;
  word-break: break-word;
  font-size: 11px;
  line-height: 1.45;
  color: var(--hh-text-muted);
}
@media (max-width: 1100px) {
  .zoom-observed-calls__content { grid-template-columns: minmax(0, 1fr); }
}
</style>
```

### `frontend/src/integrations/zoom/components/ZoomRecordingImportsPanel.vue`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/integrations/zoom/components/ZoomRecordingImportsPanel.vue`
- Size bytes / Размер в байтах: `7789`
- Included characters / Включено символов: `7775`
- Truncated / Обрезано: `no`

```text
<script setup lang="ts">
import { computed } from 'vue'
import { useI18n } from '../../../platform/i18n'
import { useSettingsStore } from '../../../shared/zoom/settingsBridge'
import type { ProviderAccount } from '../../../shared/zoom/settingsBridge'
import {
  useRemoveZoomRecordingImportMutation,
  useZoomRecordingImportsQuery,
} from '../queries/useZoomRuntimeQuery'

const { t } = useI18n()
const store = useSettingsStore()

const props = defineProps<{
  selectedAccount?: ProviderAccount | null
}>()

const recordingImportsQuery = useZoomRecordingImportsQuery(
  computed(() => props.selectedAccount?.account_id ?? null),
  12
)
const removeRecordingImport = useRemoveZoomRecordingImportMutation(
  computed(() => props.selectedAccount?.account_id ?? null)
)
const recordingImports = computed(() => recordingImportsQuery.data.value ?? [])

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

function formatBytes(value: number | null | undefined): string {
  if (typeof value !== 'number' || !Number.isFinite(value) || value < 0) return '—'
  if (value < 1024) return `${value} B`
  if (value < 1024 * 1024) return `${(value / 1024).toFixed(1)} KB`
  return `${(value / (1024 * 1024)).toFixed(1)} MB`
}

function shortHash(value: string): string {
  return value.length > 18 ? `${value.slice(0, 10)}…${value.slice(-6)}` : value
}

function formatRetention(item: { retention_mode: string; retention_days: number }): string {
  if (item.retention_mode === 'delete_after_n_days' && item.retention_days > 0) {
    return `${item.retention_days}d`
  }
  return t('Manual only')
}

async function handleRemoveImport(attachmentId: string, recordingId?: string | null) {
  const accountId = props.selectedAccount?.account_id
  if (!accountId) return
  const confirmed = window.confirm(
    t('Remove this imported Zoom recording blob from local storage and audit views?')
  )
  if (!confirmed) return

  try {
    const result = await removeRecordingImport.mutateAsync({
      attachmentId,
      request: {
        reason: recordingId
          ? `operator_removed_recording:${recordingId}`
          : 'operator_removed_recording_import',
      },
    })
    store.setActionMessage(
      result.blob_file_removed
        ? t('Zoom recording import removed from local storage')
        : t('Zoom recording import metadata removed')
    )
  } catch (err) {
    store.setError(err instanceof Error ? err.message : 'Zoom recording import removal failed')
  }
}
</script>

<template>
  <section class="integration-section zoom-recording-imports">
    <header class="zoom-recording-imports__header">
      <div>
        <h4>{{ t('Recording import audit') }}</h4>
        <p>{{ t('Imported Zoom recording blobs for the selected account.') }}</p>
      </div>
      <span class="zoom-recording-imports__count">{{ recordingImports.length }}</span>
    </header>

    <div v-if="!selectedAccount?.account_id" class="zoom-recording-imports__placeholder">
      {{ t('Select a Zoom account to inspect imported recording files.') }}
    </div>
    <div
      v-else-if="recordingImportsQuery.isLoading.value"
      class="zoom-recording-imports__placeholder"
    >
      {{ t('Loading imported Zoom recording files...') }}
    </div>
    <div v-else-if="recordingImports.length === 0" class="zoom-recording-imports__placeholder">
      {{ t('No imported Zoom recording files for this account yet.') }}
    </div>
    <div v-else class="zoom-recording-imports__list">
      <article
        v-for="item in recordingImports"
        :key="item.attachment_id"
        class="zoom-recording-imports__item"
      >
        <header>
          <strong>{{ item.recording_id ?? item.attachment_id }}</strong>
          <small>{{ formatDate(item.created_at) }}</small>
        </header>
        <dl class="zoom-recording-imports__meta">
          <div><dt>{{ t('Meeting id') }}</dt><dd>{{ item.meeting_id ?? '—' }}</dd></div>
          <div><dt>{{ t('Source') }}</dt><dd>{{ item.source ?? '—' }}</dd></div>
          <div><dt>{{ t('Filename') }}</dt><dd>{{ item.filename ?? '—' }}</dd></div>
          <div><dt>{{ t('Content type') }}</dt><dd>{{ item.content_type }}</dd></div>
          <div><dt>{{ t('Size') }}</dt><dd>{{ formatBytes(item.size_bytes) }}</dd></div>
          <div><dt>{{ t('Scan') }}</dt><dd>{{ item.scan_status }}</dd></div>
          <div><dt>{{ t('Storage') }}</dt><dd>{{ item.storage_kind }}</dd></div>
          <div><dt>{{ t('Retention') }}</dt><dd>{{ formatRetention(item) }}</dd></div>
          <div><dt>{{ t('Expires') }}</dt><dd>{{ formatDate(item.expires_at) }}</dd></div>
          <div><dt>{{ t('SHA-256') }}</dt><dd>{{ shortHash(item.sha256) }}</dd></div>
        </dl>
        <p v-if="item.scan_summary" class="zoom-recording-imports__summary">{{ item.scan_summary }}</p>
        <div class="zoom-recording-imports__actions">
          <button
            type="button"
            class="zoom-recording-imports__remove"
            :disabled="removeRecordingImport.isPending.value"
            @click="handleRemoveImport(item.attachment_id, item.recording_id)"
          >
            {{ t('Remove local import') }}
          </button>
        </div>
      </article>
    </div>
  </section>
</template>

<style scoped>
.zoom-recording-imports { display: grid; gap: 12px; }
.zoom-recording-imports__header {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: 12px;
}
.zoom-recording-imports__header h4,
.zoom-recording-imports__header p,
.zoom-recording-imports__item header {
  margin: 0;
}
.zoom-recording-imports__header p,
.zoom-recording-imports__meta dt,
.zoom-recording-imports__item small {
  color: var(--hh-text-muted);
  font-size: 11px;
}
.zoom-recording-imports__actions {
  display: flex;
  justify-content: flex-end;
}
.zoom-recording-imports__count {
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
.zoom-recording-imports__list,
.zoom-recording-imports__item {
  display: grid;
  gap: 8px;
}
.zoom-recording-imports__placeholder,
.zoom-recording-imports__item {
  padding: 10px 12px;
  border-radius: var(--hh-radius-sm);
  border: 1px solid var(--hh-border);
  background: color-mix(in srgb, var(--hh-surface-deep) 88%, white 12%);
  font-size: 12px;
}
.zoom-recording-imports__item header {
  display: flex;
  align-items: baseline;
  justify-content: space-between;
  gap: 8px;
}
.zoom-recording-imports__item strong {
  display: block;
  font-size: 12px;
}
.zoom-recording-imports__meta {
  display: grid;
  gap: 8px;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  margin: 0;
}
.zoom-recording-imports__meta div {
  display: grid;
  gap: 2px;
}
.zoom-recording-imports__meta dt,
.zoom-recording-imports__meta dd,
.zoom-recording-imports__summary {
  margin: 0;
  word-break: break-word;
}
.zoom-recording-imports__remove {
  min-height: 28px;
  padding: 0 10px;
  border-radius: 6px;
  border: 1px solid color-mix(in srgb, var(--hh-danger, #c84b4b) 55%, var(--hh-border) 45%);
  background: color-mix(in srgb, var(--hh-surface) 85%, white 15%);
  color: var(--hh-text);
  font-size: 12px;
}
.zoom-recording-imports__remove:disabled {
  opacity: 0.6;
}
@media (max-width: 900px) {
  .zoom-recording-imports__meta {
    grid-template-columns: minmax(0, 1fr);
  }
}
</style>
```

### `frontend/src/integrations/zoom/components/ZoomRecordingMaintenancePanel.vue`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/integrations/zoom/components/ZoomRecordingMaintenancePanel.vue`
- Size bytes / Размер в байтах: `7669`
- Included characters / Включено символов: `7669`
- Truncated / Обрезано: `no`

```text
<script setup lang="ts">
import { computed, ref } from 'vue'
import { useQueryClient } from '@tanstack/vue-query'
import { useI18n } from '../../../platform/i18n'
import { settingsKeys, useApplicationSettingsQuery, useSettingsStore } from '../../../shared/zoom/settingsBridge'
import type { ProviderAccount } from '../../../shared/zoom/settingsBridge'
import {
  useCleanupZoomRetentionMutation,
  useSyncZoomRecordingsMutation,
} from '../queries/useZoomRuntimeQuery'

const props = defineProps<{
  selectedAccount?: ProviderAccount | null
}>()

const { t } = useI18n()
const store = useSettingsStore()
const queryClient = useQueryClient()
const { data: applicationSettingsData } = useApplicationSettingsQuery()

const recordingSyncForm = ref({
  user_id: '',
  from: '2026-06-01',
  to: '2026-06-30',
  page_size: '30',
  max_meetings: '100',
  api_base_url: '',
})
const activeAction = ref<string | null>(null)

const selectedZoomAccountId = computed(() => props.selectedAccount?.account_id ?? null)
const syncZoomRecordings = useSyncZoomRecordingsMutation()
const cleanupZoomRetention = useCleanupZoomRetentionMutation(selectedZoomAccountId)

const zoomRecordingRetentionDays = computed(() =>
  applicationIntegerSetting('privacy.zoom_recording_import_retention_days')
)
const zoomTranscriptRetentionDays = computed(() =>
  applicationIntegerSetting('privacy.zoom_transcript_retention_days')
)

function isZoomProvider(providerKind: string): boolean {
  return providerKind === 'zoom_user' || providerKind === 'zoom_server_to_server'
}

function valueOrUndefined(input: string): string | undefined {
  const trimmed = input.trim()
  return trimmed.length ? trimmed : undefined
}

function positiveIntegerOrUndefined(input: string): number | undefined {
  const trimmed = input.trim()
  if (!trimmed) return undefined
  const parsed = Number.parseInt(trimmed, 10)
  return Number.isFinite(parsed) && parsed > 0 ? parsed : undefined
}

function applicationIntegerSetting(settingKey: string): number {
  const setting = applicationSettingsData.value?.items?.find((item) => item.setting_key === settingKey)
  return typeof setting?.value === 'number' ? setting.value : 0
}

async function refreshSettings() {
  await queryClient.invalidateQueries({ queryKey: settingsKeys.workspace() })
}

async function handleSyncZoomRecordings() {
  if (!props.selectedAccount || !isZoomProvider(props.selectedAccount.provider_kind)) return

  activeAction.value = `recording-sync:${props.selectedAccount.account_id}`
  try {
    const result = await syncZoomRecordings.mutateAsync({
      account_id: props.selectedAccount.account_id,
      user_id: valueOrUndefined(recordingSyncForm.value.user_id),
      from: recordingSyncForm.value.from.trim(),
      to: recordingSyncForm.value.to.trim(),
      page_size: positiveIntegerOrUndefined(recordingSyncForm.value.page_size),
      max_meetings: positiveIntegerOrUndefined(recordingSyncForm.value.max_meetings),
      api_base_url: valueOrUndefined(recordingSyncForm.value.api_base_url),
    })
    store.setActionMessage(
      `Zoom recording sync completed: ${result.meetings_recorded} meetings, ${result.recordings_recorded} recordings, ${result.media_downloads_recorded} media downloads, ${result.transcripts_recorded} transcripts`
    )
    await refreshSettings()
  } catch (err) {
    store.setError(err instanceof Error ? err.message : 'Zoom recording sync failed')
  } finally {
    activeAction.value = null
  }
}

async function handleCleanupZoomRetention() {
  if (!selectedZoomAccountId.value) return

  activeAction.value = `retention-cleanup:${selectedZoomAccountId.value}`
  try {
    const result = await cleanupZoomRetention.mutateAsync({
      remove_recordings: true,
      remove_transcripts: true,
      limit: 100,
    })
    store.setActionMessage(
      t('Zoom retention cleanup removed') +
        ` ${result.recordings_removed} ${t('recordings')} / ${result.transcripts_removed} ${t('transcripts')}`
    )
    await refreshSettings()
  } catch (err) {
    store.setError(err instanceof Error ? err.message : 'Zoom retention cleanup failed')
  } finally {
    activeAction.value = null
  }
}
</script>

<template>
  <section
    v-if="selectedAccount && isZoomProvider(selectedAccount.provider_kind)"
    class="integration-section compact"
  >
    <h4>{{ t('Manual recording sync') }}</h4>
    <p class="integration-section-description">
      {{ t('Provider-side recording media downloads require privacy.zoom_remote_recording_download_enabled. Provider-side transcript downloads require privacy.zoom_remote_transcript_download_enabled.') }}
    </p>
    <p class="integration-section-description">
      {{ t('Imported recording blobs follow privacy.zoom_recording_import_retention_days') }} =
      {{ zoomRecordingRetentionDays }}.
      {{ t('Transcript evidence follows privacy.zoom_transcript_retention_days') }} =
      {{ zoomTranscriptRetentionDays }}.
      {{ t('0 means explicit removal only.') }}
    </p>
    <form class="integration-form" @submit.prevent="handleSyncZoomRecordings">
      <input
        v-model="recordingSyncForm.user_id"
        class="makosh-input-control"
        type="text"
        :placeholder="t('Zoom user id override (optional)')"
      />
      <div class="maintenance-grid">
        <input v-model="recordingSyncForm.from" class="makosh-input-control" type="date" required />
        <input v-model="recordingSyncForm.to" class="makosh-input-control" type="date" required />
      </div>
      <div class="maintenance-grid">
        <input
          v-model="recordingSyncForm.page_size"
          class="makosh-input-control"
          type="number"
          min="1"
          max="100"
          :placeholder="t('30')"
        />
        <input
          v-model="recordingSyncForm.max_meetings"
          class="makosh-input-control"
          type="number"
          min="1"
          max="500"
          :placeholder="t('100')"
        />
      </div>
      <input
        v-model="recordingSyncForm.api_base_url"
        class="makosh-input-control"
        type="text"
        :placeholder="t('https://api.zoom.us/v2')"
      />
      <button
        type="submit"
        class="makosh-btn makosh-btn--outline"
        :disabled="activeAction === `recording-sync:${selectedAccount.account_id}` || syncZoomRecordings.isPending.value"
      >
        {{ syncZoomRecordings.isPending.value ? t('Syncing...') : t('Sync cloud recordings') }}
      </button>
    </form>
    <div class="inspector-actions">
      <button
        type="button"
        class="makosh-btn makosh-btn--outline"
        :disabled="activeAction === `retention-cleanup:${selectedAccount.account_id}` || cleanupZoomRetention.isPending.value"
        @click="handleCleanupZoomRetention"
      >
        {{ cleanupZoomRetention.isPending.value ? t('Cleaning...') : t('Run retention cleanup') }}
      </button>
    </div>
  </section>
</template>

<style scoped>
.integration-section { border: 1px solid var(--hh-border); border-radius: var(--hh-radius-md); background: var(--hh-surface-deep); padding: 12px; }
.integration-section.compact { margin-top: 12px; }
.integration-section h4 { margin: 0 0 6px; }
.integration-section-description { margin: 0 0 8px; font-size: 12px; color: var(--hh-text-muted); }
.integration-form { display: grid; gap: 8px; }
.integration-form button { margin-top: 6px; }
.maintenance-grid { display: grid; gap: 12px; grid-template-columns: repeat(2, minmax(0, 1fr)); }
.inspector-actions { display: flex; gap: 8px; margin-top: 12px; flex-wrap: wrap; }
@media (max-width: 960px) {
  .maintenance-grid { grid-template-columns: minmax(0, 1fr); }
}
</style>
```

### `frontend/src/integrations/zoom/components/ZoomSettingsPanel.vue`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/integrations/zoom/components/ZoomSettingsPanel.vue`
- Size bytes / Размер в байтах: `30693`
- Included characters / Включено символов: `12000`
- Truncated / Обрезано: `yes`

```text
<script setup lang="ts">
import { computed, ref } from 'vue'
import { useQueryClient } from '@tanstack/vue-query'
import { useI18n } from '../../../platform/i18n'
import { settingsKeys, useSettingsStore } from '../../../shared/zoom/settingsBridge'
import type { ProviderAccount } from '../../../shared/zoom/settingsBridge'
import ZoomAuditEventsPanel from './ZoomAuditEventsPanel.vue'
import ZoomBridgeLab from './ZoomBridgeLab.vue'
import ZoomRecordingMaintenancePanel from './ZoomRecordingMaintenancePanel.vue'
import ZoomObservedCallsPanel from './ZoomObservedCallsPanel.vue'
import ZoomRecordingImportsPanel from './ZoomRecordingImportsPanel.vue'
import {
  useAuthorizeZoomServerToServerMutation,
  useCompleteZoomOAuthMutation,
  useMaintainZoomTokensMutation,
  useRefreshZoomTokenMutation,
  useRemoveZoomRuntimeMutation,
  useSetupZoomFixtureAccountMutation,
  useSetupZoomLiveAccountMutation,
  useStartZoomOAuthMutation,
  useStartZoomRuntimeMutation,
  useStopZoomRuntimeMutation,
  useZoomCapabilitiesQuery,
  useZoomRuntimeStatusQuery,
} from '../queries/useZoomRuntimeQuery'

type ZoomAuthShape = 'oauth_user' | 'server_to_server'

const props = defineProps<{
  selectedAccount?: ProviderAccount | null
}>()

const emit = defineEmits<{
  removed: []
}>()

const { t } = useI18n()
const store = useSettingsStore()
const queryClient = useQueryClient()

const fixtureForm = ref({
  account_id: '',
  display_name: '',
  external_account_id: '',
  account_email: '',
})

const liveForm = ref({
  account_id: '',
  display_name: '',
  external_account_id: '',
  auth_shape: 'oauth_user' as ZoomAuthShape,
  client_id: '',
  token_secret_ref: '',
  client_secret_ref: '',
  webhook_secret_ref: '',
})

const oauthStartForm = ref({
  redirect_uri: 'http://127.0.0.1:8080/api/v1/integrations/zoom/oauth/callback',
  client_secret: '',
  client_secret_ref: '',
  webhook_secret_ref: '',
  scopes: 'meeting:read recording:read',
  authorization_endpoint: '',
  token_endpoint: '',
})

const oauthCompleteForm = ref({
  setup_id: '',
  state: '',
  authorization_code: '',
  external_account_id: '',
})

const s2sAuthorizeForm = ref({
  client_secret: '',
  client_secret_ref: '',
  zoom_account_id: '',
  token_endpoint: '',
})

const tokenRefreshForm = ref({
  refresh_expiring_within_seconds: '60',
})

const tokenMaintenanceForm = ref({
  refresh_expiring_within_seconds: '300',
})

const activeAction = ref<string | null>(null)
const pendingOAuthAuthorizationUrl = ref<string>('')

const setupZoomFixtureAccount = useSetupZoomFixtureAccountMutation()
const setupZoomLiveAccount = useSetupZoomLiveAccountMutation()
const startZoomOAuth = useStartZoomOAuthMutation()
const completeZoomOAuth = useCompleteZoomOAuthMutation()
const authorizeZoomServerToServer = useAuthorizeZoomServerToServerMutation()
const refreshZoomToken = useRefreshZoomTokenMutation()
const maintainZoomTokens = useMaintainZoomTokensMutation()
const startZoomRuntime = useStartZoomRuntimeMutation()
const stopZoomRuntime = useStopZoomRuntimeMutation()
const removeZoomRuntime = useRemoveZoomRuntimeMutation()

const selectedZoomAccountId = computed(() => props.selectedAccount?.account_id ?? null)
const { data: selectedZoomRuntime } = useZoomRuntimeStatusQuery(selectedZoomAccountId)
const { data: zoomCapabilities } = useZoomCapabilitiesQuery()

const selectedZoomConfig = computed<Record<string, unknown>>(() =>
  asRecord(props.selectedAccount?.config) ?? {}
)

const selectedZoomAuthShape = computed(() => {
  if (typeof selectedZoomRuntime.value?.auth_shape === 'string') return selectedZoomRuntime.value.auth_shape
  if (props.selectedAccount?.provider_kind === 'zoom_server_to_server') return 'server_to_server'
  if (props.selectedAccount?.provider_kind === 'zoom_user') return 'oauth_user'
  return null
})

const selectedTokenRotationPolicy = computed<Record<string, unknown>>(() => {
  return asRecord(selectedZoomRuntime.value?.metadata?.token_rotation_policy) ?? {}
})
const selectedTokenRotationPolicyConfig = computed<Record<string, unknown>>(
  () => asRecord(selectedTokenRotationPolicy.value.policy) ?? {}
)

const selectedZoomRuntimeBlockers = computed(() => selectedZoomRuntime.value?.runtime_blockers?.join(', ') || t('None'))
const plannedZoomFeatures = computed(() => zoomCapabilities.value?.planned_features?.join(', ') || t('None'))
const unsupportedZoomFeatures = computed(() => zoomCapabilities.value?.unsupported_features?.join(', ') || t('None'))

function isZoomProvider(providerKind: string): boolean {
  return providerKind === 'zoom_user' || providerKind === 'zoom_server_to_server'
}

function selectedAccountEmail(): string {
  if (!props.selectedAccount) return ''
  return props.selectedAccount.email || selectedString(selectedZoomConfig.value, 'email') || ''
}

function selectedDisplayName(): string {
  if (!props.selectedAccount) return ''
  return (
    props.selectedAccount.display_name ||
    props.selectedAccount.label ||
    selectedAccountEmail() ||
    props.selectedAccount.external_account_id ||
    props.selectedAccount.account_id
  )
}

function selectedClientId(): string {
  return selectedString(selectedZoomConfig.value, 'client_id') || ''
}

function selectedRuntimePolicyLabel(key: string): string {
  const value = selectedTokenRotationPolicy.value[key]
  if (typeof value === 'boolean') return value ? t('Yes') : t('No')
  if (typeof value === 'number') return String(value)
  if (typeof value === 'string') return value
  return '-'
}

function selectedRuntimePolicyDate(): string {
  const value = selectedTokenRotationPolicy.value.expires_at
  return typeof value === 'string' ? value : '-'
}

function selectedRuntimePolicyThreshold(key: string): string {
  const value = selectedTokenRotationPolicyConfig.value[key]
  return typeof value === 'number' ? `${value}s` : '-'
}

function valueOrUndefined(input: string): string | undefined {
  const trimmed = input.trim()
  return trimmed.length ? trimmed : undefined
}

function positiveIntegerOrUndefined(input: string): number | undefined {
  const trimmed = input.trim()
  if (!trimmed) return undefined
  const parsed = Number.parseInt(trimmed, 10)
  return Number.isFinite(parsed) && parsed > 0 ? parsed : undefined
}

function splitScopes(input: string): string[] {
  return input
    .split(/[\s,]+/)
    .map((value) => value.trim())
    .filter(Boolean)
}

function asRecord(value: unknown): Record<string, unknown> | null {
  if (!value || typeof value !== 'object' || Array.isArray(value)) return null
  return value as Record<string, unknown>
}

function selectedString(record: Record<string, unknown>, key: string): string | null {
  const value = record[key]
  return typeof value === 'string' && value.trim() ? value.trim() : null
}

async function refreshSettings() {
  await queryClient.invalidateQueries({ queryKey: settingsKeys.workspace() })
}

async function handleCreateZoomFixture() {
  const account_id = fixtureForm.value.account_id.trim()
  const display_name = fixtureForm.value.display_name.trim()
  const external_account_id = fixtureForm.value.external_account_id.trim()
  const account_email = fixtureForm.value.account_email.trim()
  if (!account_id || !display_name || !external_account_id) {
    store.setError(t('Account id, display name and external account id are required'))
    return
  }

  activeAction.value = 'fixture'
  try {
    await setupZoomFixtureAccount.mutateAsync({
      account_id,
      display_name,
      external_account_id,
      account_email: account_email || undefined,
    })
    fixtureForm.value = { account_id: '', display_name: '', external_account_id: '', account_email: '' }
    store.setActionMessage(t('Zoom fixture account created'))
    await refreshSettings()
  } catch (err) {
    store.setError(err instanceof Error ? err.message : 'Zoom fixture setup failed')
  } finally {
    activeAction.value = null
  }
}

async function handleCreateZoomLive() {
  const account_id = liveForm.value.account_id.trim()
  const display_name = liveForm.value.display_name.trim()
  const external_account_id = liveForm.value.external_account_id.trim()
  const client_id = liveForm.value.client_id.trim()
  if (!account_id || !display_name || !external_account_id || !client_id) {
    store.setError(t('Account id, display name, external account id and client id are required'))
    return
  }

  activeAction.value = 'live'
  try {
    await setupZoomLiveAccount.mutateAsync({
      account_id,
      display_name,
      external_account_id,
      auth_shape: liveForm.value.auth_shape,
      client_id,
      token_secret_ref: valueOrUndefined(liveForm.value.token_secret_ref),
      client_secret_ref: valueOrUndefined(liveForm.value.client_secret_ref),
      webhook_secret_ref: valueOrUndefined(liveForm.value.webhook_secret_ref),
    })
    liveForm.value = {
      account_id: '',
      display_name: '',
      external_account_id: '',
      auth_shape: liveForm.value.auth_shape,
      client_id: '',
      token_secret_ref: '',
      client_secret_ref: '',
      webhook_secret_ref: '',
    }
    store.setActionMessage(t('Zoom live account metadata registered'))
    await refreshSettings()
  } catch (err) {
    store.setError(err instanceof Error ? err.message : 'Zoom live setup failed')
  } finally {
    activeAction.value = null
  }
}

async function handleStartZoomOAuth() {
  if (!props.selectedAccount || !isZoomProvider(props.selectedAccount.provider_kind)) return
  if (selectedZoomAuthShape.value !== 'oauth_user') {
    store.setError(t('Selected Zoom account is not an OAuth user account'))
    return
  }
  const client_id = selectedClientId()
  if (!client_id) {
    store.setError(t('Selected Zoom account has no client id in metadata'))
    return
  }

  activeAction.value = `oauth-start:${props.selectedAccount.account_id}`
  try {
    const response = await startZoomOAuth.mutateAsync({
      account_id: props.selectedAccount.account_id,
      display_name: selectedDisplayName(),
      external_account_id: props.selectedAccount.external_account_id,
      account_email: selectedAccountEmail() || undefined,
      client_id,
      client_secret: valueOrUndefined(oauthStartForm.value.client_secret),
      client_secret_ref: valueOrUndefined(oauthStartForm.value.client_secret_ref),
      webhook_secret_ref: valueOrUndefined(oauthStartForm.value.webhook_secret_ref),
      redirect_uri: oauthStartForm.value.redirect_uri.trim(),
      scopes: splitScopes(oauthStartForm.value.scopes),
      authorization_endpoint: valueOrUndefined(oauthStartForm.value.authorization_endpoint),
      token_endpoint: valueOrUndefined(oauthStartForm.value.token_endpoint),
    })
    oauthCompleteForm.value.setup_id = response.setup_id
    oauthCompleteForm.value.state = response.state
    oauthCompleteForm.value.external_account_id = props.selectedAccount.external_account_id
    pendingOAuthAuthorizationUrl.value = response.authorization_url
    window.open(response.authorization_url, '_blank', 'noopener,noreferrer')
    store.setActionMessage(t('Zoom OAuth authorization started'))
    await refreshSettings()
  } catch (err) {
    store.setError(err instanceof Error ? err.message : 'Zoom OAuth start failed')
  } finally {
    activeAction.value = null
  }
}

async function handleCompleteZoomOAuth() {
  if (!props.selectedAccount || !isZoomProvider(props.selectedAccount.provider_kind)) return
  activeAction.value = `oauth-complete:${props.selectedAccount.account_id}`
  try {
    await completeZoomOAuth.mutateAsync({
      setup_id: oauthCompleteForm.value.setup_id.trim(),
      state: oauthCompleteForm.value.state.trim(),
      authorization_code: oauthCompleteForm.value.authorization_code.trim(),
      external_account_id: valueOrUndefined(oauthCompleteForm.value.external_account_id),
    })
    oauthCompleteForm.value.authorization_code = ''
    pendingOAuthAuthorizationUrl.value = ''
    store.setActionMessage(t('Zoom OAuth authorization completed'))
    await refreshSettings()
  } catch (err) {
    st
```
_Source file truncated after 12000 characters. / Исходный файл обрезан после 12000 символов._

### `frontend/src/platform/event-tracing/EventTracePanel.vue`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/platform/event-tracing/EventTracePanel.vue`
- Size bytes / Размер в байтах: `9335`
- Included characters / Включено символов: `9335`
- Truncated / Обрезано: `no`

```text
<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import Icon from '../../shared/ui/Icon.vue'
import type { EventTrace, StoredEventEnvelope } from './types'

const props = defineProps<{
  trace: EventTrace | null
  isLoading?: boolean
  errorMessage?: string
}>()

const selectedEventId = ref<string | null>(null)

const orderedEvents = computed(() => props.trace?.events ?? [])
const selectedEvent = computed(() => {
  return orderedEvents.value.find((item) => item.event.event_id === selectedEventId.value) ?? orderedEvents.value[0] ?? null
})
const edgeByChild = computed(() => new Map((props.trace?.edges ?? []).map((edge) => [edge.child_event_id, edge.parent_event_id])))
const annotationsByEvent = computed(() => {
  const map = new Map<string, string[]>()
  for (const annotation of props.trace?.consumer_annotations ?? []) {
    const entries = map.get(annotation.event_id) ?? []
    entries.push(`${annotation.consumer_name}: ${annotation.status}`)
    map.set(annotation.event_id, entries)
  }
  for (const deadLetter of props.trace?.dead_letters ?? []) {
    const entries = map.get(deadLetter.event_id) ?? []
    entries.push(`DLQ: ${deadLetter.consumer_name ?? 'unknown'}`)
    map.set(deadLetter.event_id, entries)
  }
  return map
})

watch(
  orderedEvents,
  (events) => {
    if (!events.some((item) => item.event.event_id === selectedEventId.value)) {
      selectedEventId.value = events[0]?.event.event_id ?? null
    }
  },
  { immediate: true }
)

function shortId(value: string | null | undefined): string {
  if (!value) return 'none'
  if (value.length <= 24) return value
  return `${value.slice(0, 10)}...${value.slice(-8)}`
}

function sourceLabel(event: StoredEventEnvelope): string {
  const source = event.event.source
  const kind = typeof source.kind === 'string' ? source.kind : 'source'
  const account = typeof source.account_id === 'string' ? source.account_id : null
  return account ? `${kind} / ${shortId(account)}` : kind
}

function subjectLabel(event: StoredEventEnvelope): string {
  const subject = event.event.subject
  const kind = typeof subject.kind === 'string' ? subject.kind : 'subject'
  const id = firstString(subject.entity_id, subject.message_id, subject.observation_id, subject.id)
  return id ? `${kind} / ${shortId(id)}` : kind
}

function firstString(...values: unknown[]): string | null {
  for (const value of values) {
    if (typeof value === 'string' && value.trim().length > 0) return value
  }
  return null
}

function jsonPreview(value: unknown): string {
  return JSON.stringify(value, null, 2)
}
</script>

<template>
  <section class="event-trace-panel">
    <div class="trace-header">
      <div class="trace-identity">
        <Icon icon="tabler:route" class="trace-icon" />
        <div>
          <span class="trace-label">Trace</span>
          <strong>{{ shortId(trace?.correlation_id) }}</strong>
        </div>
      </div>
      <div class="trace-metrics" aria-label="Trace metrics">
        <span><strong>{{ trace?.events.length ?? 0 }}</strong> events</span>
        <span><strong>{{ trace?.edges.length ?? 0 }}</strong> edges</span>
        <span><strong>{{ trace?.missing_parent_ids.length ?? 0 }}</strong> missing</span>
        <span><strong>{{ trace?.dead_letters.length ?? 0 }}</strong> DLQ</span>
      </div>
    </div>

    <div v-if="isLoading" class="trace-state">Loading trace</div>
    <div v-else-if="errorMessage" class="trace-state trace-error">{{ errorMessage }}</div>
    <div v-else-if="!trace" class="trace-state">No trace selected</div>
    <div v-else-if="orderedEvents.length === 0" class="trace-state">Trace is empty</div>
    <div v-else class="trace-body">
      <ol class="trace-events">
        <li v-for="item in orderedEvents" :key="item.event.event_id">
          <button
            class="trace-event"
            :class="{ active: selectedEvent?.event.event_id === item.event.event_id }"
            @click="selectedEventId = item.event.event_id"
          >
            <span class="event-position">#{{ item.position }}</span>
            <span class="event-copy">
              <span class="event-type">{{ item.event.event_type }}</span>
              <span class="event-meta">
                {{ shortId(item.event.event_id) }}
                <template v-if="edgeByChild.has(item.event.event_id)">
                  parent {{ shortId(edgeByChild.get(item.event.event_id)) }}
                </template>
                <template v-else-if="trace.root_event_ids.includes(item.event.event_id)">
                  root
                </template>
              </span>
            </span>
            <span v-if="annotationsByEvent.has(item.event.event_id)" class="event-badge">
              {{ annotationsByEvent.get(item.event.event_id)?.length }}
            </span>
          </button>
        </li>
      </ol>

      <aside v-if="selectedEvent" class="trace-detail">
        <div class="detail-title">
          <span>{{ selectedEvent.event.event_type }}</span>
          <code>{{ shortId(selectedEvent.event.event_id) }}</code>
        </div>
        <dl class="detail-grid">
          <div>
            <dt>Source</dt>
            <dd>{{ sourceLabel(selectedEvent) }}</dd>
          </div>
          <div>
            <dt>Subject</dt>
            <dd>{{ subjectLabel(selectedEvent) }}</dd>
          </div>
          <div>
            <dt>Causation</dt>
            <dd>{{ shortId(selectedEvent.event.causation_id) }}</dd>
          </div>
          <div>
            <dt>Recorded</dt>
            <dd>{{ selectedEvent.event.recorded_at }}</dd>
          </div>
        </dl>
        <div v-if="annotationsByEvent.has(selectedEvent.event.event_id)" class="detail-annotations">
          <span v-for="entry in annotationsByEvent.get(selectedEvent.event.event_id)" :key="entry">
            {{ entry }}
          </span>
        </div>
        <pre>{{ jsonPreview(selectedEvent.event.subject) }}</pre>
      </aside>
    </div>
  </section>
</template>

<style scoped>
.event-trace-panel {
  display: flex;
  min-height: 0;
  height: 100%;
  flex-direction: column;
  background: var(--hh-bg-primary, #fff);
  color: var(--hh-text-primary, #1f2937);
}

.trace-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 1rem;
  padding: 0.875rem 1rem;
  border-bottom: 1px solid var(--hh-border, #e5e7eb);
}

.trace-identity,
.trace-metrics,
.trace-event,
.detail-title {
  display: flex;
  align-items: center;
}

.trace-identity {
  min-width: 0;
  gap: 0.625rem;
}

.trace-icon {
  width: 1.25rem;
  height: 1.25rem;
  color: var(--hh-accent, #2563eb);
}

.trace-label {
  display: block;
  font-size: 0.6875rem;
  color: var(--hh-text-muted, #6b7280);
  text-transform: uppercase;
}

.trace-metrics {
  flex-wrap: wrap;
  justify-content: flex-end;
  gap: 0.5rem;
  font-size: 0.75rem;
  color: var(--hh-text-secondary, #4b5563);
}

.trace-metrics span,
.event-badge,
.detail-annotations span {
  border: 1px solid var(--hh-border, #e5e7eb);
  border-radius: 6px;
  padding: 0.1875rem 0.4375rem;
  background: var(--hh-bg-secondary, #f9fafb);
}

.trace-state {
  padding: 2rem;
  color: var(--hh-text-secondary, #6b7280);
  font-size: 0.875rem;
}

.trace-error {
  color: var(--hh-danger, #b91c1c);
}

.trace-body {
  display: grid;
  grid-template-columns: minmax(20rem, 0.95fr) minmax(22rem, 1.05fr);
  min-height: 0;
  flex: 1;
}

.trace-events {
  min-height: 0;
  overflow: auto;
  margin: 0;
  padding: 0.75rem;
  list-style: none;
  border-right: 1px solid var(--hh-border, #e5e7eb);
}

.trace-event {
  width: 100%;
  min-height: 3.125rem;
  gap: 0.75rem;
  border: 1px solid transparent;
  border-radius: 6px;
  background: transparent;
  padding: 0.5rem 0.625rem;
  text-align: left;
  color: inherit;
  cursor: pointer;
}

.trace-event:hover,
.trace-event.active {
  border-color: var(--hh-border-accent-soft, #bfdbfe);
  background: var(--hh-bg-selected, #eff6ff);
}

.event-position,
.event-badge {
  flex: 0 0 auto;
  font-size: 0.75rem;
  color: var(--hh-text-muted, #6b7280);
}

.event-copy {
  display: flex;
  min-width: 0;
  flex: 1;
  flex-direction: column;
  gap: 0.125rem;
}

.event-type,
.detail-title span {
  overflow-wrap: anywhere;
  font-size: 0.8125rem;
  font-weight: 600;
}

.event-meta,
.detail-grid {
  font-size: 0.75rem;
  color: var(--hh-text-secondary, #4b5563);
}

.trace-detail {
  min-width: 0;
  overflow: auto;
  padding: 1rem;
}

.detail-title {
  justify-content: space-between;
  gap: 1rem;
  padding-bottom: 0.75rem;
  border-bottom: 1px solid var(--hh-border, #e5e7eb);
}

.detail-title code,
.trace-detail pre {
  font-size: 0.75rem;
}

.detail-grid {
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: 0.75rem;
  margin: 0.875rem 0;
}

.detail-grid dt {
  color: var(--hh-text-muted, #6b7280);
}

.detail-grid dd {
  margin: 0.125rem 0 0;
  overflow-wrap: anywhere;
  color: var(--hh-text-primary, #1f2937);
}

.detail-annotations {
  display: flex;
  flex-wrap: wrap;
  gap: 0.375rem;
  margin-bottom: 0.875rem;
  font-size: 0.75rem;
}

.trace-detail pre {
  overflow: auto;
  max-height: 16rem;
  margin: 0;
  border: 1px solid var(--hh-border, #e5e7eb);
  border-radius: 6px;
  padding: 0.75rem;
  background: var(--hh-bg-secondary, #f9fafb);
  white-space: pre-wrap;
}
</style>
```

### `frontend/src/platform/event-tracing/EventTraceWorkspace.vue`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/platform/event-tracing/EventTraceWorkspace.vue`
- Size bytes / Размер в байтах: `4352`
- Included characters / Включено символов: `4352`
- Truncated / Обрезано: `no`

```text
<script setup lang="ts">
import { computed, ref } from 'vue'
import Icon from '../../shared/ui/Icon.vue'
import EventTracePanel from './EventTracePanel.vue'
import {
  useEventTraceByCorrelationIdQuery,
  useEventTraceByEventIdQuery
} from './queries'

type TraceLookupMode = 'event' | 'correlation'

const lookupMode = ref<TraceLookupMode>('event')
const inputValue = ref('')
const submittedValue = ref('')

const eventLookupId = computed(() => lookupMode.value === 'event' ? submittedValue.value : null)
const correlationLookupId = computed(() => lookupMode.value === 'correlation' ? submittedValue.value : null)
const eventTraceQuery = useEventTraceByEventIdQuery(eventLookupId)
const correlationTraceQuery = useEventTraceByCorrelationIdQuery(correlationLookupId)

const activeQuery = computed(() => lookupMode.value === 'event' ? eventTraceQuery : correlationTraceQuery)
const trace = computed(() => activeQuery.value.data.value ?? null)
const isLoading = computed(() => activeQuery.value.isFetching.value)
const errorMessage = computed(() => {
  const error = activeQuery.value.error.value
  if (!error) return ''
  return error instanceof Error ? error.message : 'Trace request failed'
})

function submitLookup(): void {
  submittedValue.value = inputValue.value.trim()
}
</script>

<template>
  <section class="event-trace-workspace">
    <header class="workspace-toolbar">
      <div class="toolbar-title">
        <Icon icon="tabler:route" />
        <div>
          <h1>Event Traces</h1>
          <span>event_log</span>
        </div>
      </div>
      <form class="trace-search" @submit.prevent="submitLookup">
        <div class="mode-toggle" aria-label="Trace lookup mode">
          <button
            type="button"
            :class="{ active: lookupMode === 'event' }"
            @click="lookupMode = 'event'"
          >
            Event
          </button>
          <button
            type="button"
            :class="{ active: lookupMode === 'correlation' }"
            @click="lookupMode = 'correlation'"
          >
            Trace
          </button>
        </div>
        <input
          v-model="inputValue"
          spellcheck="false"
          :placeholder="lookupMode === 'event' ? 'event_id' : 'correlation_id'"
        >
        <button type="submit" class="search-button" aria-label="Fetch trace">
          <Icon icon="tabler:search" />
        </button>
      </form>
    </header>

    <EventTracePanel
      :trace="trace"
      :is-loading="isLoading"
      :error-message="errorMessage"
    />
  </section>
</template>

<style scoped>
.event-trace-workspace {
  display: flex;
  height: 100%;
  min-height: 0;
  flex-direction: column;
  background: var(--hh-bg-primary, #fff);
}

.workspace-toolbar,
.toolbar-title,
.trace-search,
.mode-toggle {
  display: flex;
  align-items: center;
}

.workspace-toolbar {
  justify-content: space-between;
  gap: 1rem;
  padding: 0.875rem 1rem;
  border-bottom: 1px solid var(--hh-border, #e5e7eb);
}

.toolbar-title {
  min-width: 0;
  gap: 0.625rem;
}

.toolbar-title svg {
  width: 1.25rem;
  height: 1.25rem;
  color: var(--hh-accent, #2563eb);
}

.toolbar-title h1 {
  margin: 0;
  font-size: 1rem;
  line-height: 1.2;
}

.toolbar-title span {
  font-size: 0.75rem;
  color: var(--hh-text-muted, #6b7280);
}

.trace-search {
  min-width: min(34rem, 58vw);
  gap: 0.5rem;
}

.mode-toggle {
  flex: 0 0 auto;
  overflow: hidden;
  border: 1px solid var(--hh-border, #e5e7eb);
  border-radius: 6px;
}

.mode-toggle button,
.search-button {
  min-height: 2rem;
  border: 0;
  background: transparent;
  color: var(--hh-text-secondary, #4b5563);
  cursor: pointer;
}

.mode-toggle button {
  padding: 0 0.625rem;
  font-size: 0.8125rem;
}

.mode-toggle button.active {
  background: var(--hh-bg-selected, #eff6ff);
  color: var(--hh-accent, #2563eb);
}

.trace-search input {
  min-width: 0;
  flex: 1;
  height: 2rem;
  border: 1px solid var(--hh-border, #e5e7eb);
  border-radius: 6px;
  padding: 0 0.625rem;
  background: var(--hh-bg-secondary, #f9fafb);
  color: var(--hh-text-primary, #1f2937);
  font: inherit;
}

.search-button {
  display: inline-flex;
  width: 2rem;
  align-items: center;
  justify-content: center;
  border: 1px solid var(--hh-border, #e5e7eb);
  border-radius: 6px;
}

.search-button svg {
  width: 1rem;
  height: 1rem;
}
</style>
```

### `frontend/src/shared/mailSetup/AccountSetupModal.vue`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/shared/mailSetup/AccountSetupModal.vue`
- Size bytes / Размер в байтах: `13384`
- Included characters / Включено символов: `12000`
- Truncated / Обрезано: `yes`

```text
<script setup lang="ts">
import { computed, ref } from 'vue'
import { useForm } from 'vee-validate'
import { loadFrontendConfig } from '../../platform/config/env'
import Icon from '../ui/Icon.vue'
import Button from '../ui/Button.vue'
import Dialog from '../ui/Dialog.vue'
import {
  useSetupImapEmailAccountMutation,
  useStartGmailOAuthSetupMutation
} from '../../integrations/mail/queries/accountSetupQueries'
import {
  accountSetupFormDefaults,
  accountSetupFormToGmailOAuthStart,
  accountSetupFormToImapRequest,
  accountSetupVeeValidationSchema,
  type AccountSetupFormValues,
  type MailAccountSetupProvider
} from '../../integrations/mail/forms/accountSetupForm'

const emit = defineEmits<{
  close: []
}>()

const step = ref(1)
const setupError = ref('')
const setupStatusMessage = ref('')
const frontendConfig = loadFrontendConfig()
const gmailOAuthSetupMutation = useStartGmailOAuthSetupMutation()
const imapEmailAccountSetupMutation = useSetupImapEmailAccountMutation()

const providerOptions: { kind: MailAccountSetupProvider; label: string; icon: string; description: string }[] = [
  { kind: 'gmail', label: 'Gmail', icon: 'tabler:brand-google', description: 'Google OAuth account setup' },
  { kind: 'icloud', label: 'iCloud', icon: 'tabler:brand-apple', description: 'iCloud Mail with app password' },
  { kind: 'imap', label: 'IMAP', icon: 'tabler:mail', description: 'Generic IMAP and SMTP account' }
]

const {
  errors,
  handleSubmit,
  isSubmitting,
  resetForm,
  setFieldValue,
  values: formValues
} = useForm<AccountSetupFormValues>({
  validationSchema: accountSetupVeeValidationSchema,
  initialValues: accountSetupFormDefaults('icloud')
})

const selectedProvider = computed(() => formValues.provider_kind)
const selectedProviderInfo = computed(() =>
  providerOptions.find(p => p.kind === selectedProvider.value)
)
const submitLabel = computed(() => {
  if (isSubmitting.value) return selectedProvider.value === 'gmail' ? 'Starting...' : 'Connecting...'
  return selectedProvider.value === 'gmail' ? 'Continue with Google' : 'Connect Account'
})

function selectProvider(kind: MailAccountSetupProvider) {
  resetForm({ values: accountSetupFormDefaults(kind) })
  setupError.value = ''
  setupStatusMessage.value = ''
  step.value = 2
}

function goBack() {
  if (step.value > 1) {
    step.value--
    setupError.value = ''
    setupStatusMessage.value = ''
  }
}

const submitAccountSetup = handleSubmit(async (values) => {
  setupError.value = ''
  setupStatusMessage.value = ''

  try {
    if (values.provider_kind === 'gmail') {
      const response = await gmailOAuthSetupMutation.mutateAsync(
        accountSetupFormToGmailOAuthStart(values, frontendConfig.apiBaseUrl)
      )
      window.open(response.authorization_url, '_blank', 'noopener,noreferrer')
      setupStatusMessage.value = 'Google authorization opened'
      return
    }

    await imapEmailAccountSetupMutation.mutateAsync(accountSetupFormToImapRequest(values))
    emit('close')
  } catch (e) {
    setupError.value = e instanceof Error ? e.message : 'Setup failed'
  }
})

function updateStringField(
  key: keyof AccountSetupFormValues,
  event: Event
) {
  setFieldValue(key, (event.target as HTMLInputElement).value)
}

function updateNumberField(
  key: keyof AccountSetupFormValues,
  event: Event
) {
  setFieldValue(key, Number((event.target as HTMLInputElement).value))
}

function updateBooleanField(
  key: keyof AccountSetupFormValues,
  event: Event
) {
  setFieldValue(key, (event.target as HTMLInputElement).checked)
}

function handleClose() {
  emit('close')
}
</script>

<template>
  <Dialog :open="true" content-class="account-setup-dialog" @update:open="(open) => { if (!open) handleClose() }">
    <template #header>
      <div class="modal-header">
        <div class="modal-header-left">
          <Button v-if="step > 1" variant="ghost" size="sm" @click="goBack">
            <Icon icon="tabler:arrow-left" />
          </Button>
          <h2 v-if="step === 1">Add Mail Account</h2>
          <h2 v-else-if="step === 2">Configure {{ selectedProviderInfo?.label }}</h2>
        </div>
      </div>
    </template>

    <div class="setup-modal">
      <!-- Step 1: Provider selection -->
      <div v-if="step === 1" class="provider-selection">
        <p class="step-desc">Select a mail provider to connect</p>
        <div class="provider-grid">
          <button
            v-for="provider in providerOptions"
            :key="provider.kind"
            class="provider-card"
            type="button"
            @click="selectProvider(provider.kind)"
          >
            <Icon :icon="provider.icon" class="provider-icon" />
            <span class="provider-label">{{ provider.label }}</span>
            <span class="provider-desc">{{ provider.description }}</span>
          </button>
        </div>
      </div>

      <!-- Step 2: Account details -->
      <div v-else-if="step === 2" class="account-details">
        <p class="step-desc">Enter your {{ selectedProviderInfo?.label }} account details</p>

        <div class="form-fields">
          <div class="field">
            <label>Account Name</label>
            <input
              type="text"
              :value="formValues.display_name"
              placeholder="e.g., Personal Gmail"
              @input="updateStringField('display_name', $event)"
            />
          </div>
          <div class="field">
            <label>Email Address</label>
            <input
              type="email"
              :value="formValues.email"
              placeholder="you@example.com"
              @input="updateStringField('email', $event)"
            />
            <span v-if="errors.email" class="field-error">{{ errors.email }}</span>
          </div>

          <template v-if="selectedProvider === 'imap'">
            <div class="field">
              <label>IMAP Host</label>
              <input
                type="text"
                :value="formValues.imap_host"
                placeholder="imap.example.com"
                @input="updateStringField('imap_host', $event)"
              />
              <span v-if="errors.imap_host" class="field-error">{{ errors.imap_host }}</span>
            </div>
            <div class="field-row">
              <div class="field">
                <label>IMAP Port</label>
                <input
                  type="number"
                  :value="formValues.imap_port"
                  @input="updateNumberField('imap_port', $event)"
                />
              </div>
              <div class="field">
                <label>Username</label>
                <input
                  type="text"
                  :value="formValues.username"
                  placeholder="user@example.com"
                  @input="updateStringField('username', $event)"
                />
              </div>
            </div>
            <div class="field">
              <label>Password</label>
              <input
                type="password"
                :value="formValues.password"
                placeholder="Mailbox password"
                @input="updateStringField('password', $event)"
              />
              <span v-if="errors.password" class="field-error">{{ errors.password }}</span>
            </div>
            <div class="field">
              <label>SMTP Host</label>
              <input
                type="text"
                :value="formValues.smtp_host"
                placeholder="smtp.example.com"
                @input="updateStringField('smtp_host', $event)"
              />
            </div>
            <div class="field-row">
              <div class="field">
                <label>SMTP Port</label>
                <input
                  type="number"
                  :value="formValues.smtp_port"
                  @input="updateNumberField('smtp_port', $event)"
                />
              </div>
              <div class="field checkbox-field">
                <label class="checkbox-label">
                  <input
                    type="checkbox"
                    :checked="formValues.imap_tls"
                    @change="updateBooleanField('imap_tls', $event)"
                  />
                  IMAP TLS
                </label>
              </div>
            </div>
            <div class="field-row">
              <div class="field checkbox-field">
                <label class="checkbox-label">
                  <input
                    type="checkbox"
                    :checked="formValues.smtp_tls"
                    @change="updateBooleanField('smtp_tls', $event)"
                  />
                  SMTP TLS
                </label>
              </div>
              <div class="field checkbox-field">
                <label class="checkbox-label">
                  <input
                    type="checkbox"
                    :checked="formValues.smtp_starttls"
                    @change="updateBooleanField('smtp_starttls', $event)"
                  />
                  SMTP STARTTLS
                </label>
              </div>
            </div>
          </template>

          <div v-if="selectedProvider === 'icloud'" class="field">
            <label>App Password</label>
            <input
              type="password"
              :value="formValues.password"
              placeholder="Your app-specific password"
              @input="updateStringField('password', $event)"
            />
            <span v-if="errors.password" class="field-error">{{ errors.password }}</span>
          </div>
        </div>

        <div v-if="setupError" class="setup-error">{{ setupError }}</div>
        <div v-if="setupStatusMessage" class="setup-status">{{ setupStatusMessage }}</div>

        <div class="form-actions">
          <Button variant="default" @click="submitAccountSetup" :loading="isSubmitting">
            {{ submitLabel }}
          </Button>
          <Button variant="ghost" @click="goBack">Back</Button>
        </div>
      </div>
    </div>
  </Dialog>
</template>

<style scoped>
:deep(.account-setup-dialog) {
  max-width: 520px;
}

.setup-modal {
  width: 100%;
  display: flex;
  flex-direction: column;
}

.modal-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding-right: 2.5rem;
  border-bottom: 1px solid var(--hh-border, #e5e7eb);
}

.modal-header-left {
  display: flex;
  align-items: center;
  gap: 0.5rem;
}

.modal-header-left h2 {
  margin: 0;
  font-size: 1rem;
  font-weight: 600;
}

.step-desc {
  margin: 0 0 1rem;
  font-size: 0.875rem;
  color: var(--hh-text-secondary, #6b7280);
}

.provider-selection,
.account-details {
  padding: 1rem;
  overflow-y: auto;
}

.provider-grid {
  display: flex;
  flex-direction: column;
  gap: 0.5rem;
}

.provider-card {
  display: flex;
  align-items: center;
  gap: 0.75rem;
  width: 100%;
  padding: 0.75rem 1rem;
  border: 1px solid var(--hh-border, #e5e7eb);
  border-radius: 0.5rem;
  background: transparent;
  color: inherit;
  cursor: pointer;
  font: inherit;
  text-align: left;
  transition: background 0.1s, border-color 0.1s;
}

.provider-card:hover {
  background: var(--hh-bg-hover, #f3f4f6);
  border-color: var(--hh-accent, #3b82f6);
}

.provider-icon {
  width: 28px;
  height: 28px;
  color: var(--hh-accent, #3b82f6);
  flex-shrink: 0;
}

.provider-label {
  font-weight: 500;
  font-size: 0.875rem;
  color: var(--hh-text-primary, #1f2937);
}

.provider-desc {
  font-size: 0.75rem;
  color: var(--hh-text-secondary, #6b7280);
  margin-left: auto;
}

.form-fields {
  display: flex;
  flex-direction: column;
  gap: 0.75rem;
}

.field {
  display: flex;
  flex-direction: column;
  gap: 0.25rem;
}

.field label {
  font-size: 0.75rem;
  font-weight: 500;
  color: var(--hh-text-secondary, #6b7280);
}

.field input[type="text"],
.field input[type="email"],
.field input[type="password"],
.field input[type="number"] {
  padding: 0.5rem 0.625rem;
  border: 1p
```
_Source file truncated after 12000 characters. / Исходный файл обрезан после 12000 символов._

### `frontend/src/shared/mailSync/MailSyncSettingsStrip.vue`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/shared/mailSync/MailSyncSettingsStrip.vue`
- Size bytes / Размер в байтах: `5257`
- Included characters / Включено символов: `5257`
- Truncated / Обрезано: `no`

```text
<script setup lang="ts">
import { computed, watch } from 'vue'
import { useForm } from 'vee-validate'
import Button from '../ui/Button.vue'
import Icon from '../ui/Icon.vue'
import {
  syncSettingsFormDefaults,
  syncSettingsFormToUpdate,
  syncSettingsVeeValidationSchema,
  type SyncSettingsFormValues
} from './syncSettingsForm'
import type {
  MailSyncSettings,
  MailSyncSettingsUpdate
} from './types'

const props = defineProps<{
  settings: MailSyncSettings | null
  isLoading: boolean
  isSaving: boolean
}>()

const emit = defineEmits<{
  update: [settings: MailSyncSettingsUpdate]
}>()

const {
  errors,
  handleSubmit,
  setFieldValue,
  setValues,
  values: formValues
} = useForm<SyncSettingsFormValues>({
  validationSchema: syncSettingsVeeValidationSchema,
  initialValues: syncSettingsFormDefaults(props.settings)
})

const isDisabled = computed(() => props.isLoading || props.isSaving || !props.settings)
const syncStateLabel = computed(() => (formValues.sync_enabled ? 'Enabled' : 'Paused'))

watch(
  () => props.settings,
  (settings) => setValues(syncSettingsFormDefaults(settings)),
  { immediate: true }
)

const submitSettings = handleSubmit((values) => {
  emit('update', syncSettingsFormToUpdate(values))
})

function updateBooleanField(event: Event): void {
  const input = event.target as HTMLInputElement
  setFieldValue('sync_enabled', input.checked)
}

function updateNumberField(field: 'batch_size' | 'poll_interval_seconds', event: Event): void {
  const input = event.target as HTMLInputElement
  setFieldValue(field, Number(input.value))
}
</script>

<template>
  <section v-if="settings || isLoading" class="mail-sync-settings-strip" aria-label="Provider sync settings">
    <div class="sync-settings-heading">
      <Icon icon="tabler:refresh-dot" class="sync-settings-icon" />
      <div>
        <div class="sync-settings-title">Provider sync</div>
        <div class="sync-settings-meta">
          <span v-if="isLoading">Loading settings...</span>
          <span v-else>{{ syncStateLabel }}</span>
        </div>
      </div>
    </div>

    <form class="sync-settings-form" @submit.prevent="submitSettings">
      <label class="sync-toggle">
        <input
          :checked="formValues.sync_enabled"
          type="checkbox"
          :disabled="isDisabled"
          @change="updateBooleanField"
        />
        <span>Sync</span>
      </label>

      <label class="sync-field">
        <span>Batch</span>
        <input
          :value="formValues.batch_size"
          type="number"
          min="1"
          max="500"
          step="1"
          :disabled="isDisabled"
          @input="updateNumberField('batch_size', $event)"
        />
        <small v-if="errors.batch_size">{{ errors.batch_size }}</small>
      </label>

      <label class="sync-field">
        <span>Poll, sec</span>
        <input
          :value="formValues.poll_interval_seconds"
          type="number"
          min="60"
          max="86400"
          step="60"
          :disabled="isDisabled"
          @input="updateNumberField('poll_interval_seconds', $event)"
        />
        <small v-if="errors.poll_interval_seconds">{{ errors.poll_interval_seconds }}</small>
      </label>

      <Button variant="outline" size="sm" :disabled="isDisabled" :loading="isSaving" type="submit">
        Save
      </Button>
    </form>
  </section>
</template>

<style scoped>
.mail-sync-settings-strip {
  display: grid;
  grid-template-columns: minmax(10rem, 0.4fr) minmax(0, 1fr);
  gap: 0.75rem;
  align-items: center;
  padding: 0.5rem 0.75rem;
  border-bottom: 1px solid var(--hh-border, #e5e7eb);
  background: color-mix(in srgb, var(--hh-bg-primary, #ffffff) 82%, transparent);
  backdrop-filter: blur(var(--hh-panel-blur));
}

.sync-settings-heading {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  min-width: 0;
}

.sync-settings-icon {
  width: 16px;
  height: 16px;
  color: var(--hh-accent, #2563eb);
}

.sync-settings-title {
  color: var(--hh-text-primary, #1f2937);
  font-size: 0.75rem;
  font-weight: 700;
}

.sync-settings-meta {
  color: var(--hh-text-secondary, #6b7280);
  font-size: 0.6875rem;
}

.sync-settings-form {
  display: flex;
  align-items: flex-start;
  justify-content: flex-end;
  gap: 0.5rem;
  min-width: 0;
}

.sync-toggle,
.sync-field {
  display: grid;
  gap: 0.1875rem;
  color: var(--hh-text-secondary, #6b7280);
  font-size: 0.6875rem;
  font-weight: 600;
}

.sync-toggle {
  grid-template-columns: auto auto;
  align-items: center;
  padding-top: 1.2rem;
}

.sync-toggle input {
  accent-color: var(--hh-accent, #2563eb);
}

.sync-field input {
  width: 6.5rem;
  min-height: 1.75rem;
  border: 1px solid var(--hh-border, #d1d5db);
  border-radius: var(--hh-radius-sm, 0.375rem);
  padding: 0.25rem 0.375rem;
  background: color-mix(in srgb, var(--hh-bg-primary, #ffffff) 74%, transparent);
  color: var(--hh-text-primary, #111827);
  font-size: 0.75rem;
}

.sync-field small {
  max-width: 8rem;
  color: var(--hh-text-error, #ef4444);
  font-size: 0.625rem;
  line-height: 1.2;
}

@media (max-width: 900px) {
  .mail-sync-settings-strip {
    grid-template-columns: 1fr;
  }

  .sync-settings-form {
    justify-content: flex-start;
    flex-wrap: wrap;
  }
}
</style>
```

### `frontend/src/shared/transitions/FadeTransition.vue`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/shared/transitions/FadeTransition.vue`
- Size bytes / Размер в байтах: `628`
- Included characters / Включено символов: `628`
- Truncated / Обрезано: `no`

```text
<script setup lang="ts">
import { computed } from 'vue'

const props = withDefaults(defineProps<{
  duration?: number
  mode?: 'in-out' | 'out-in'
  appear?: boolean
}>(), {
  duration: 200,
  mode: 'out-in',
  appear: false
})

const cssDuration = computed(() => `${props.duration}ms`)
</script>

<template>
  <Transition
    :name="'makosh-fade'"
    :mode="mode"
    :appear="appear"
  >
    <slot />
  </Transition>
</template>

<style scoped>
.makosh-fade-enter-active,
.makosh-fade-leave-active {
  transition: opacity v-bind(cssDuration) ease;
}

.makosh-fade-enter-from,
.makosh-fade-leave-to {
  opacity: 0;
}
</style>
```

### `frontend/src/shared/transitions/SlideTransition.vue`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/shared/transitions/SlideTransition.vue`
- Size bytes / Размер в байтах: `2061`
- Included characters / Включено символов: `2061`
- Truncated / Обрезано: `no`

```text
<script setup lang="ts">
import { computed } from 'vue'

const props = withDefaults(defineProps<{
  direction?: 'up' | 'down' | 'left' | 'right'
  duration?: number
  distance?: string
  mode?: 'in-out' | 'out-in'
  appear?: boolean
}>(), {
  direction: 'up',
  duration: 200,
  distance: '1rem',
  mode: 'out-in',
  appear: false
})

const cssDuration = computed(() => `${props.duration}ms`)
const cssDistance = computed(() => props.distance)

const nameClass = computed(() => `makosh-slide-${props.direction}`)
</script>

<template>
  <Transition
    :name="nameClass"
    :mode="mode"
    :appear="appear"
  >
    <slot />
  </Transition>
</template>

<style scoped>
/* Up */
.makosh-slide-up-enter-active,
.makosh-slide-up-leave-active {
  transition: all v-bind(cssDuration) cubic-bezier(0.16, 1, 0.3, 1);
}
.makosh-slide-up-enter-from {
  opacity: 0;
  transform: translateY(v-bind(cssDistance));
}
.makosh-slide-up-leave-to {
  opacity: 0;
  transform: translateY(calc(-1 * v-bind(cssDistance)));
}

/* Down */
.makosh-slide-down-enter-active,
.makosh-slide-down-leave-active {
  transition: all v-bind(cssDuration) cubic-bezier(0.16, 1, 0.3, 1);
}
.makosh-slide-down-enter-from {
  opacity: 0;
  transform: translateY(calc(-1 * v-bind(cssDistance)));
}
.makosh-slide-down-leave-to {
  opacity: 0;
  transform: translateY(v-bind(cssDistance));
}

/* Left */
.makosh-slide-left-enter-active,
.makosh-slide-left-leave-active {
  transition: all v-bind(cssDuration) cubic-bezier(0.16, 1, 0.3, 1);
}
.makosh-slide-left-enter-from {
  opacity: 0;
  transform: translateX(v-bind(cssDistance));
}
.makosh-slide-left-leave-to {
  opacity: 0;
  transform: translateX(calc(-1 * v-bind(cssDistance)));
}

/* Right */
.makosh-slide-right-enter-active,
.makosh-slide-right-leave-active {
  transition: all v-bind(cssDuration) cubic-bezier(0.16, 1, 0.3, 1);
}
.makosh-slide-right-enter-from {
  opacity: 0;
  transform: translateX(calc(-1 * v-bind(cssDistance)));
}
.makosh-slide-right-leave-to {
  opacity: 0;
  transform: translateX(v-bind(cssDistance));
}
</style>
```

### `frontend/src/shared/ui/Avatar.vue`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/shared/ui/Avatar.vue`
- Size bytes / Размер в байтах: `2039`
- Included characters / Включено символов: `2039`
- Truncated / Обрезано: `no`

```text
<script setup lang="ts">
import { AvatarRoot, AvatarImage, AvatarFallback } from 'reka-ui'
import { computed } from 'vue'

const props = withDefaults(defineProps<{
  src?: string
  alt?: string
  fallback?: string
  size?: 'sm' | 'md' | 'lg' | 'xl'
  class?: string
}>(), {
  alt: 'avatar',
  size: 'md'
})

const rootClasses = computed(() => [
  'makosh-avatar-root',
  `makosh-avatar--${props.size}`,
  props.class
])

const fallbackText = computed(() => {
  if (props.fallback) return props.fallback.slice(0, 2).toUpperCase()
  if (props.alt && props.alt !== 'avatar') return props.alt.slice(0, 2).toUpperCase()
  return '?'
})
</script>

<template>
  <AvatarRoot :class="rootClasses">
    <AvatarImage v-if="src" :src="src" :alt="alt" class="makosh-avatar-image" />
    <AvatarFallback class="makosh-avatar-fallback" :delay-ms="src ? 300 : 0">
      {{ fallbackText }}
    </AvatarFallback>
  </AvatarRoot>
</template>

<style scoped>
.makosh-avatar-root {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  vertical-align: middle;
  border-radius: 9999px;
  background: var(--hh-hover-bg);
  border: 1px solid var(--hh-border);
  overflow: hidden;
  flex-shrink: 0;
  user-select: none;
}

.makosh-avatar-image {
  width: 100%;
  height: 100%;
  object-fit: cover;
  border-radius: inherit;
}

.makosh-avatar-fallback {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 100%;
  height: 100%;
  font-weight: 600;
  color: var(--hh-text-secondary);
  line-height: 1;
}

.makosh-avatar--sm .makosh-avatar-fallback {
  font-size: 0.625rem;
}

.makosh-avatar--sm {
  width: 1.5rem;
  height: 1.5rem;
}

.makosh-avatar--md .makosh-avatar-fallback {
  font-size: 0.75rem;
}

.makosh-avatar--md {
  width: 2rem;
  height: 2rem;
}

.makosh-avatar--lg .makosh-avatar-fallback {
  font-size: 0.875rem;
}

.makosh-avatar--lg {
  width: 2.5rem;
  height: 2.5rem;
}

.makosh-avatar--xl .makosh-avatar-fallback {
  font-size: 1rem;
}

.makosh-avatar--xl {
  width: 3rem;
  height: 3rem;
}
</style>
```

### `frontend/src/shared/ui/Badge.vue`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/shared/ui/Badge.vue`
- Size bytes / Размер в байтах: `1663`
- Included characters / Включено символов: `1663`
- Truncated / Обрезано: `no`

```text
<script setup lang="ts">
import { computed } from 'vue'

const props = withDefaults(defineProps<{
  variant?: 'default' | 'accent' | 'success' | 'warning' | 'danger' | 'info' | 'neutral'
  size?: 'sm' | 'md'
  class?: string
}>(), {
  variant: 'default',
  size: 'sm'
})

const classes = computed(() => [
  'makosh-badge',
  `makosh-badge--${props.variant}`,
  `makosh-badge--${props.size}`,
  props.class
])
</script>

<template>
  <span :class="classes">
    <slot />
  </span>
</template>

<style scoped>
.makosh-badge {
  display: inline-flex;
  align-items: center;
  font-family: var(--hh-font-sans);
  font-weight: 500;
  white-space: nowrap;
  border-radius: var(--hh-radius-pill);
  line-height: 1;
}

.makosh-badge--sm {
  height: 1.25rem;
  padding: 0 0.5rem;
  font-size: 0.6875rem;
}

.makosh-badge--md {
  height: 1.5rem;
  padding: 0 0.625rem;
  font-size: 0.75rem;
}

/* Variants */
.makosh-badge--default {
  background: var(--hh-hover-bg);
  color: var(--hh-text-secondary);
}

.makosh-badge--accent {
  background: var(--hh-accent-tint);
  color: var(--hh-accent);
}

.makosh-badge--success {
  background: var(--hh-status-success-surface);
  color: var(--hh-status-success-text);
}

.makosh-badge--warning {
  background: var(--hh-status-warning-surface);
  color: var(--hh-status-warning-text);
}

.makosh-badge--danger {
  background: var(--hh-status-danger-surface);
  color: var(--hh-status-danger-text);
}

.makosh-badge--info {
  background: var(--hh-status-info-surface);
  color: var(--hh-status-info-text);
}

.makosh-badge--neutral {
  background: var(--hh-status-neutral-surface);
  color: var(--hh-status-neutral-text);
}
</style>
```

### `frontend/src/shared/ui/Button.vue`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/shared/ui/Button.vue`
- Size bytes / Размер в байтах: `3848`
- Included characters / Включено символов: `3848`
- Truncated / Обрезано: `no`

```text
<script setup lang="ts">
import { computed } from 'vue'
import Icon from './Icon.vue'

const props = withDefaults(defineProps<{
  variant?: 'default' | 'secondary' | 'outline' | 'ghost' | 'destructive'
  size?: 'sm' | 'md' | 'lg'
  disabled?: boolean
  loading?: boolean
  icon?: string
  class?: string
  type?: 'button' | 'submit' | 'reset'
}>(), {
  variant: 'default',
  size: 'md',
  disabled: false,
  loading: false,
  type: 'button'
})

const emit = defineEmits<{
  click: [event: MouseEvent]
}>()

const classes = computed(() => {
  return [
    'makosh-btn',
    `makosh-btn--${props.variant}`,
    `makosh-btn--${props.size}`,
    { 'makosh-btn--disabled': props.disabled || props.loading },
    props.class
  ]
})

function handleClick(event: MouseEvent): void {
  if (!props.disabled && !props.loading) {
    emit('click', event)
  }
}
</script>

<template>
  <button
    :class="classes"
    :disabled="disabled || loading"
    :type="type"
    @click="handleClick"
  >
    <Icon v-if="loading" icon="tabler:loader-2" size="1em" class="makosh-btn-spinner" />
    <Icon v-else-if="icon" :icon="icon" size="1em" />
    <span v-if="$slots.default" class="makosh-btn-text">
      <slot />
    </span>
  </button>
</template>

<style scoped>
.makosh-btn {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  gap: 0.375rem;
  font-family: var(--hh-font-sans);
  font-weight: 500;
  border: 1px solid transparent;
  border-radius: var(--hh-radius-sm);
  cursor: pointer;
  transition: all 150ms ease;
  white-space: nowrap;
  user-select: none;
  line-height: 1;
}

.makosh-btn:focus-visible {
  outline: 2px solid var(--hh-focus-ring);
  outline-offset: 2px;
}

.makosh-btn--disabled {
  opacity: 0.5;
  cursor: not-allowed;
  pointer-events: none;
}

/* Variants */
.makosh-btn--default {
  background: var(--hh-accent);
  color: var(--hh-accent-contrast);
  border-color: var(--hh-accent);
}
.makosh-btn--default:hover:not(:disabled) {
  filter: brightness(1.1);
}
.makosh-btn--default:active:not(:disabled) {
  filter: brightness(0.9);
}

.makosh-btn--secondary {
  background: var(--hh-hover-bg);
  color: var(--hh-text-primary);
  border-color: var(--hh-border);
}
.makosh-btn--secondary:hover:not(:disabled) {
  background: var(--hh-active-bg);
  border-color: var(--hh-border-accent);
}
.makosh-btn--secondary:active:not(:disabled) {
  background: var(--hh-accent-tint);
}

.makosh-btn--outline {
  background: transparent;
  color: var(--hh-text-primary);
  border-color: var(--hh-border);
}
.makosh-btn--outline:hover:not(:disabled) {
  background: var(--hh-hover-bg);
  border-color: var(--hh-border-accent);
}
.makosh-btn--outline:active:not(:disabled) {
  background: var(--hh-accent-tint);
}

.makosh-btn--ghost {
  background: transparent;
  color: var(--hh-text-secondary);
  border-color: transparent;
}
.makosh-btn--ghost:hover:not(:disabled) {
  background: var(--hh-hover-bg);
  color: var(--hh-text-primary);
}
.makosh-btn--ghost:active:not(:disabled) {
  background: var(--hh-active-bg);
}

.makosh-btn--destructive {
  background: var(--hh-danger-tint);
  color: var(--hh-color-danger);
  border-color: transparent;
}
.makosh-btn--destructive:hover:not(:disabled) {
  background: var(--hh-color-danger-strong);
  color: white;
}
.makosh-btn--destructive:active:not(:disabled) {
  filter: brightness(0.9);
}

/* Sizes */
.makosh-btn--sm {
  height: 1.75rem;
  padding: 0 0.625rem;
  font-size: 0.75rem;
  border-radius: var(--hh-radius-xs);
}

.makosh-btn--md {
  height: 2.125rem;
  padding: 0 0.875rem;
  font-size: 0.8125rem;
}

.makosh-btn--lg {
  height: 2.5rem;
  padding: 0 1.125rem;
  font-size: 0.875rem;
}

.makosh-btn-spinner {
  animation: makosh-spin 1s linear infinite;
}

@keyframes makosh-spin {
  from { transform: rotate(0deg); }
  to { transform: rotate(360deg); }
}
</style>
```

### `frontend/src/shared/ui/Card.vue`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/shared/ui/Card.vue`
- Size bytes / Размер в байтах: `445`
- Included characters / Включено символов: `445`
- Truncated / Обрезано: `no`

```text
<script setup lang="ts">
import { computed } from 'vue'

const props = withDefaults(defineProps<{
  class?: string
}>(), {})

const classes = computed(() => ['makosh-card', props.class])
</script>

<template>
  <div :class="classes">
    <slot />
  </div>
</template>

<style scoped>
.makosh-card {
  background: var(--hh-surface-panel);
  border: 1px solid var(--hh-border);
  border-radius: var(--hh-radius-md);
  overflow: hidden;
}
</style>
```

### `frontend/src/shared/ui/CardContent.vue`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/shared/ui/CardContent.vue`
- Size bytes / Размер в байтах: `346`
- Included characters / Включено символов: `346`
- Truncated / Обрезано: `no`

```text
<script setup lang="ts">
import { computed } from 'vue'

const props = withDefaults(defineProps<{
  class?: string
}>(), {})

const classes = computed(() => ['makosh-card-content', props.class])
</script>

<template>
  <div :class="classes">
    <slot />
  </div>
</template>

<style scoped>
.makosh-card-content {
  padding: 1.25rem;
}
</style>
```

### `frontend/src/shared/ui/CardDescription.vue`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/shared/ui/CardDescription.vue`
- Size bytes / Размер в байтах: `418`
- Included characters / Включено символов: `418`
- Truncated / Обрезано: `no`

```text
<script setup lang="ts">
import { computed } from 'vue'

const props = withDefaults(defineProps<{
  class?: string
}>(), {})

const classes = computed(() => ['makosh-card-description', props.class])
</script>

<template>
  <p :class="classes">
    <slot />
  </p>
</template>

<style scoped>
.makosh-card-description {
  font-size: 0.8125rem;
  color: var(--hh-text-muted);
  margin: 0;
  line-height: 1.4;
}
</style>
```

### `frontend/src/shared/ui/CardFooter.vue`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/shared/ui/CardFooter.vue`
- Size bytes / Размер в байтах: `449`
- Included characters / Включено символов: `449`
- Truncated / Обрезано: `no`

```text
<script setup lang="ts">
import { computed } from 'vue'

const props = withDefaults(defineProps<{
  class?: string
}>(), {})

const classes = computed(() => ['makosh-card-footer', props.class])
</script>

<template>
  <div :class="classes">
    <slot />
  </div>
</template>

<style scoped>
.makosh-card-footer {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  padding: 0.75rem 1.25rem;
  border-top: 1px solid var(--hh-border);
}
</style>
```

### `frontend/src/shared/ui/CardHeader.vue`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/shared/ui/CardHeader.vue`
- Size bytes / Размер в байтах: `453`
- Included characters / Включено символов: `453`
- Truncated / Обрезано: `no`

```text
<script setup lang="ts">
import { computed } from 'vue'

const props = withDefaults(defineProps<{
  class?: string
}>(), {})

const classes = computed(() => ['makosh-card-header', props.class])
</script>

<template>
  <div :class="classes">
    <slot />
  </div>
</template>

<style scoped>
.makosh-card-header {
  display: flex;
  flex-direction: column;
  gap: 0.25rem;
  padding: 1rem 1.25rem;
  border-bottom: 1px solid var(--hh-border);
}
</style>
```

### `frontend/src/shared/ui/CardTitle.vue`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/shared/ui/CardTitle.vue`
- Size bytes / Размер в байтах: `430`
- Included characters / Включено символов: `430`
- Truncated / Обрезано: `no`

```text
<script setup lang="ts">
import { computed } from 'vue'

const props = withDefaults(defineProps<{
  class?: string
}>(), {})

const classes = computed(() => ['makosh-card-title', props.class])
</script>

<template>
  <h3 :class="classes">
    <slot />
  </h3>
</template>

<style scoped>
.makosh-card-title {
  font-size: 0.9375rem;
  font-weight: 600;
  color: var(--hh-text-primary);
  margin: 0;
  line-height: 1.3;
}
</style>
```

### `frontend/src/shared/ui/Command.vue`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/shared/ui/Command.vue`
- Size bytes / Размер в байтах: `8123`
- Included characters / Включено символов: `8103`
- Truncated / Обрезано: `no`

```text
<script setup lang="ts">
import { DialogRoot, DialogPortal, DialogOverlay, DialogContent } from 'reka-ui'
import { ref, computed, watch, nextTick } from 'vue'
import Icon from './Icon.vue'

export interface CommandGroup {
  label: string
  items: CommandItem[]
}

export interface CommandItem {
  id: string
  label: string
  description?: string
  icon?: string
  keywords?: string[]
  onSelect?: () => void
}

const props = withDefaults(defineProps<{
  open?: boolean
  groups?: CommandGroup[]
  placeholder?: string
  emptyMessage?: string
  class?: string
  contentClass?: string
}>(), {
  placeholder: 'Поиск...',
  emptyMessage: 'Ничего не найдено'
})

const emit = defineEmits<{
  'update:open': [value: boolean]
  'select': [item: CommandItem]
}>()

const query = ref('')
const inputRef = ref<HTMLInputElement | null>(null)
const selectedIndex = ref(0)

const flatItems = computed(() => {
  return (props.groups || []).flatMap((g) => g.items)
})

const filteredGroups = computed(() => {
  const q = query.value.toLowerCase().trim()
  if (!q) return props.groups || []

  return (props.groups || [])
    .map((group) => ({
      ...group,
      items: group.items.filter((item) => {
        const labelMatch = item.label.toLowerCase().includes(q)
        const descMatch = item.description?.toLowerCase().includes(q)
        const keywordMatch = item.keywords?.some((k) => k.toLowerCase().includes(q))
        return labelMatch || descMatch || keywordMatch
      })
    }))
    .filter((g) => g.items.length > 0)
})

const filteredFlatItems = computed(() => {
  return filteredGroups.value.flatMap((g) => g.items)
})

watch(() => props.open, (isOpen) => {
  if (isOpen) {
    query.value = ''
    selectedIndex.value = 0
    nextTick(() => inputRef.value?.focus())
  }
})

function handleKeyDown(event: KeyboardEvent): void {
  const items = filteredFlatItems.value
  if (items.length === 0) return

  switch (event.key) {
    case 'ArrowDown':
      event.preventDefault()
      selectedIndex.value = Math.min(selectedIndex.value + 1, items.length - 1)
      break
    case 'ArrowUp':
      event.preventDefault()
      selectedIndex.value = Math.max(selectedIndex.value - 1, 0)
      break
    case 'Enter':
      event.preventDefault()
      const selected = items[selectedIndex.value]
      if (selected) {
        selected.onSelect?.()
        emit('select', selected)
        emit('update:open', false)
      }
      break
  }
}

function selectItem(item: CommandItem): void {
  item.onSelect?.()
  emit('select', item)
  emit('update:open', false)
}

const contentClasses = computed(() => [
  'makosh-command-content',
  props.contentClass
])
</script>

<template>
  <DialogRoot :open="open" @update:open="(val) => emit('update:open', val)">
    <DialogPortal>
      <DialogOverlay class="makosh-command-overlay" @pointerdown="emit('update:open', false)">
        <DialogContent :class="contentClasses" @keydown="handleKeyDown" @open-auto-focus="(e: Event) => e.preventDefault()">
          <div class="makosh-command-input-wrapper">
            <Icon icon="tabler:search" size="1.125rem" class="makosh-command-search-icon" />
            <input
              ref="inputRef"
              v-model="query"
              class="makosh-command-input"
              :placeholder="placeholder"
              @keydown.stop="handleKeyDown"
            />
            <kbd class="makosh-command-kbd">ESC</kbd>
          </div>

          <div class="makosh-command-list">
            <template v-if="filteredGroups.length > 0">
              <div v-for="(group, gi) in filteredGroups" :key="gi" class="makosh-command-group">
                <div class="makosh-command-group-label">{{ group.label }}</div>
                <button
                  v-for="(item, ii) in group.items"
                  :key="item.id"
                  class="makosh-command-item"
                  :class="{ 'makosh-command-item--selected': flatItems.indexOf(item) === selectedIndex }"
                  @click="selectItem(item)"
                  @mouseenter="selectedIndex = flatItems.indexOf(item)"
                >
                  <Icon v-if="item.icon" :icon="item.icon" size="1.125rem" class="makosh-command-item-icon" />
                  <div class="makosh-command-item-text">
                    <span class="makosh-command-item-label">{{ item.label }}</span>
                    <span v-if="item.description" class="makosh-command-item-desc">{{ item.description }}</span>
                  </div>
                </button>
              </div>
            </template>
            <div v-else-if="query" class="makosh-command-empty">
              <Icon icon="tabler:search-off" size="1.5rem" />
              <span>{{ emptyMessage }}</span>
            </div>
          </div>
        </DialogContent>
      </DialogOverlay>
    </DialogPortal>
  </DialogRoot>
</template>

<style scoped>
.makosh-command-overlay {
  position: fixed;
  inset: 0;
  background: rgba(0, 0, 0, 0.6);
  display: flex;
  align-items: flex-start;
  justify-content: center;
  z-index: 100;
  padding-top: 12vh;
  animation: command-overlay-in 150ms ease;
}

.makosh-command-content {
  width: 90vw;
  max-width: 560px;
  max-height: 60vh;
  background: var(--hh-surface-panel);
  border: 1px solid var(--hh-border);
  border-radius: var(--hh-radius-lg);
  box-shadow: var(--hh-shadow-modal);
  overflow: hidden;
  display: flex;
  flex-direction: column;
  animation: command-content-in 150ms ease;
}

.makosh-command-input-wrapper {
  display: flex;
  align-items: center;
  gap: 0.625rem;
  padding: 0.875rem 1rem;
  border-bottom: 1px solid var(--hh-border);
  flex-shrink: 0;
}

.makosh-command-search-icon {
  flex-shrink: 0;
  color: var(--hh-text-muted);
}

.makosh-command-input {
  flex: 1;
  background: transparent;
  border: none;
  outline: none;
  font-size: 0.875rem;
  color: var(--hh-text-primary);
  font-family: inherit;
  line-height: 1.5;
}

.makosh-command-input::placeholder {
  color: var(--hh-text-muted);
}

.makosh-command-kbd {
  flex-shrink: 0;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  padding: 0.125rem 0.375rem;
  font-size: 0.625rem;
  font-weight: 500;
  color: var(--hh-text-muted);
  background: var(--hh-hover-bg);
  border: 1px solid var(--hh-border);
  border-radius: var(--hh-radius-xs);
  font-family: inherit;
  line-height: 1.4;
}

.makosh-command-list {
  flex: 1;
  overflow-y: auto;
  padding: 0.5rem;
}

.makosh-command-group {
  margin-bottom: 0.25rem;
}

.makosh-command-group-label {
  padding: 0.375rem 0.5rem;
  font-size: 0.6875rem;
  font-weight: 600;
  color: var(--hh-text-muted);
  text-transform: uppercase;
  letter-spacing: 0.05em;
}

.makosh-command-item {
  display: flex;
  align-items: center;
  gap: 0.625rem;
  width: 100%;
  padding: 0.5rem 0.625rem;
  border-radius: var(--hh-radius-sm);
  background: transparent;
  border: none;
  cursor: pointer;
  text-align: left;
  font-family: inherit;
  transition: background 100ms ease;
}

.makosh-command-item:hover,
.makosh-command-item--selected {
  background: var(--hh-hover-bg);
}

.makosh-command-item:focus-visible {
  outline: 2px solid var(--hh-focus-ring);
  outline-offset: -2px;
}

.makosh-command-item-icon {
  flex-shrink: 0;
  color: var(--hh-text-secondary);
}

.makosh-command-item-text {
  flex: 1;
  min-width: 0;
}

.makosh-command-item-label {
  display: block;
  font-size: 0.8125rem;
  font-weight: 500;
  color: var(--hh-text-primary);
  line-height: 1.4;
}

.makosh-command-item-desc {
  display: block;
  font-size: 0.6875rem;
  color: var(--hh-text-muted);
  line-height: 1.3;
  margin-top: 0.0625rem;
}

.makosh-command-empty {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 0.5rem;
  padding: 2rem 1rem;
  color: var(--hh-text-muted);
  font-size: 0.8125rem;
}

@keyframes command-overlay-in {
  from { opacity: 0; }
  to { opacity: 1; }
}

@keyframes command-content-in {
  from {
    opacity: 0;
    transform: translateY(-1rem) scale(0.97);
  }
  to {
    opacity: 1;
    transform: translateY(0) scale(1);
  }
}
</style>
```

### `frontend/src/shared/ui/Dialog.vue`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/shared/ui/Dialog.vue`
- Size bytes / Размер в байтах: `3432`
- Included characters / Включено символов: `3432`
- Truncated / Обрезано: `no`

```text
<script setup lang="ts">
import { DialogRoot, DialogTrigger, DialogPortal, DialogOverlay, DialogContent, DialogTitle, DialogDescription, DialogClose } from 'reka-ui'
import { computed } from 'vue'
import Icon from './Icon.vue'

const props = withDefaults(defineProps<{
  open?: boolean
  title?: string
  description?: string
  class?: string
  contentClass?: string
}>(), {
  open: false
})

const emit = defineEmits<{
  'update:open': [value: boolean]
}>()

const contentClasses = computed(() => ['makosh-dialog-content', props.contentClass])
</script>

<template>
  <DialogRoot :open="open" @update:open="(val) => emit('update:open', val)">
    <DialogTrigger v-if="$slots.trigger" as-child>
      <slot name="trigger" />
    </DialogTrigger>
    <DialogPortal>
      <DialogOverlay class="makosh-dialog-overlay">
        <DialogContent :class="contentClasses">
          <div class="makosh-dialog-header">
            <DialogTitle v-if="title" class="makosh-dialog-title">{{ title }}</DialogTitle>
            <DialogDescription v-if="description" class="makosh-dialog-description">{{ description }}</DialogDescription>
            <slot name="header" />
          </div>
          <div class="makosh-dialog-body">
            <slot />
          </div>
          <div v-if="$slots.footer" class="makosh-dialog-footer">
            <slot name="footer" />
          </div>
          <DialogClose class="makosh-dialog-close" as-child>
            <button class="makosh-dialog-close-btn">
              <Icon icon="tabler:x" size="1.125rem" />
            </button>
          </DialogClose>
        </DialogContent>
      </DialogOverlay>
    </DialogPortal>
  </DialogRoot>
</template>

<style scoped>
.makosh-dialog-overlay {
  position: fixed;
  inset: 0;
  background: rgba(0, 0, 0, 0.6);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 100;
  animation: dialog-overlay-in 200ms ease;
}

.makosh-dialog-content {
  position: relative;
  width: 90vw;
  max-width: 500px;
  max-height: 85vh;
  background: var(--hh-surface-panel);
  border: 1px solid var(--hh-border);
  border-radius: var(--hh-radius-lg);
  box-shadow: var(--hh-shadow-modal);
  overflow-y: auto;
  animation: dialog-content-in 200ms ease;
}

.makosh-dialog-header {
  padding: 1.5rem 1.5rem 0;
}

.makosh-dialog-title {
  font-size: 1.125rem;
  font-weight: 600;
  color: var(--hh-text-primary);
  margin: 0;
  line-height: 1.3;
}

.makosh-dialog-description {
  font-size: 0.8125rem;
  color: var(--hh-text-muted);
  margin: 0.25rem 0 0;
  line-height: 1.4;
}

.makosh-dialog-body {
  padding: 1.25rem 1.5rem;
}

.makosh-dialog-footer {
  display: flex;
  align-items: center;
  justify-content: flex-end;
  gap: 0.5rem;
  padding: 0 1.5rem 1.25rem;
}

.makosh-dialog-close {
  position: absolute;
  top: 1rem;
  right: 1rem;
}

.makosh-dialog-close-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 2rem;
  height: 2rem;
  border-radius: var(--hh-radius-sm);
  border: none;
  background: transparent;
  color: var(--hh-text-muted);
  cursor: pointer;
  transition: all 150ms ease;
}

.makosh-dialog-close-btn:hover {
  background: var(--hh-hover-bg);
  color: var(--hh-text-primary);
}

@keyframes dialog-overlay-in {
  from { opacity: 0; }
  to { opacity: 1; }
}

@keyframes dialog-content-in {
  from { opacity: 0; transform: scale(0.95); }
  to { opacity: 1; transform: scale(1); }
}
</style>
```

### `frontend/src/shared/ui/DropdownMenu.vue`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/shared/ui/DropdownMenu.vue`
- Size bytes / Размер в байтах: `1354`
- Included characters / Включено символов: `1354`
- Truncated / Обрезано: `no`

```text
<script setup lang="ts">
import { DropdownMenuRoot, DropdownMenuTrigger, DropdownMenuPortal, DropdownMenuContent, DropdownMenuItem, DropdownMenuSeparator, DropdownMenuLabel } from 'reka-ui'
import { computed } from 'vue'

const props = withDefaults(defineProps<{
  class?: string
  align?: 'start' | 'center' | 'end'
  side?: 'top' | 'bottom'
  sideOffset?: number
}>(), {
  align: 'start',
  side: 'bottom',
  sideOffset: 4
})

const contentClasses = computed(() => ['makosh-dropdown-content', props.class])
</script>

<template>
  <DropdownMenuRoot>
    <DropdownMenuTrigger as-child>
      <slot name="trigger" />
    </DropdownMenuTrigger>
    <DropdownMenuPortal>
      <DropdownMenuContent
        :class="contentClasses"
        :align="align"
        :side="side"
        :side-offset="sideOffset"
      >
        <slot />
      </DropdownMenuContent>
    </DropdownMenuPortal>
  </DropdownMenuRoot>
</template>

<style scoped>
.makosh-dropdown-content {
  min-width: 180px;
  background: var(--hh-surface-panel);
  border: 1px solid var(--hh-border);
  border-radius: var(--hh-radius-md);
  padding: 0.25rem;
  box-shadow: 0 4px 16px rgba(0, 0, 0, 0.3);
  z-index: 100;
  animation: dropdown-in 150ms ease;
}

@keyframes dropdown-in {
  from { opacity: 0; transform: translateY(-4px); }
  to { opacity: 1; transform: translateY(0); }
}
</style>
```

### `frontend/src/shared/ui/DropdownMenuItem.vue`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/shared/ui/DropdownMenuItem.vue`
- Size bytes / Размер в байтах: `1472`
- Included characters / Включено символов: `1472`
- Truncated / Обрезано: `no`

```text
<script setup lang="ts">
import { DropdownMenuItem } from 'reka-ui'
import { computed } from 'vue'
import Icon from './Icon.vue'

const props = withDefaults(defineProps<{
  icon?: string
  disabled?: boolean
  class?: string
  inset?: boolean
}>(), {
  disabled: false,
  inset: false
})

const classes = computed(() => [
  'makosh-dropdown-item',
  { 'makosh-dropdown-item--inset': props.inset, 'makosh-dropdown-item--disabled': props.disabled },
  props.class
])
</script>

<template>
  <DropdownMenuItem :class="classes" :disabled="disabled">
    <Icon v-if="icon" :icon="icon" size="1rem" class="makosh-dropdown-item-icon" />
    <slot />
  </DropdownMenuItem>
</template>

<style scoped>
.makosh-dropdown-item {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  width: 100%;
  padding: 0.5rem 0.75rem;
  font-size: 0.8125rem;
  color: var(--hh-text-secondary);
  border-radius: var(--hh-radius-xs);
  cursor: pointer;
  outline: none;
  user-select: none;
  transition: background 100ms ease;
  border: none;
  background: transparent;
  text-align: left;
  font-family: var(--hh-font-sans);
}

.makosh-dropdown-item:hover,
.makosh-dropdown-item[data-highlighted] {
  background: var(--hh-hover-bg);
  color: var(--hh-text-primary);
}

.makosh-dropdown-item--inset {
  padding-left: 2.25rem;
}

.makosh-dropdown-item--disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.makosh-dropdown-item-icon {
  flex-shrink: 0;
  color: var(--hh-text-muted);
}
</style>
```

### `frontend/src/shared/ui/DropdownMenuLabel.vue`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/shared/ui/DropdownMenuLabel.vue`
- Size bytes / Размер в байтах: `567`
- Included characters / Включено символов: `567`
- Truncated / Обрезано: `no`

```text
<script setup lang="ts">
import { DropdownMenuLabel } from 'reka-ui'
import { computed } from 'vue'

const props = withDefaults(defineProps<{
  class?: string
}>(), {})

const classes = computed(() => ['makosh-dropdown-label', props.class])
</script>

<template>
  <DropdownMenuLabel :class="classes">
    <slot />
  </DropdownMenuLabel>
</template>

<style scoped>
.makosh-dropdown-label {
  padding: 0.5rem 0.75rem 0.25rem;
  font-size: 0.6875rem;
  font-weight: 600;
  color: var(--hh-text-muted);
  text-transform: uppercase;
  letter-spacing: 0.05em;
}
</style>
```
