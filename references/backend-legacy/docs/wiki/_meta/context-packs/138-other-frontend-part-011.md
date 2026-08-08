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

- Chunk ID / ID чанка: `138-other-frontend-part-011`
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

### `frontend/src/integrations/telegram/components/TelegramAccountManager.vue`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/integrations/telegram/components/TelegramAccountManager.vue`
- Size bytes / Размер в байтах: `10732`
- Included characters / Включено символов: `10730`
- Truncated / Обрезано: `no`

```text
<script setup lang="ts">
import { computed, ref } from 'vue'
import { useForm } from 'vee-validate'
import { useI18n } from '../../../platform/i18n'
import Icon from '../../../shared/ui/Icon.vue'
import TelegramCapabilityMatrix from './TelegramCapabilityMatrix.vue'
import TelegramQrLoginPanel from './TelegramQrLoginPanel.vue'
import {
  defaultTelegramAccountSetupValues,
  telegramAccountSetupSchema,
  type TelegramAccountSetupFormValues,
} from '../forms/telegramAccountSetupForm'
import {
  useLogoutTelegramAccountMutation,
  useRemoveTelegramAccountMutation,
  useSetupTelegramAccountMutation,
  useTelegramAccountsQuery,
} from '../queries/useTelegramQuery'

const { t } = useI18n()

const props = defineProps<{
  selectedAccountId: string | null
}>()

const isSetupOpen = ref(false)
const setupMutation = useSetupTelegramAccountMutation()
const logoutMutation = useLogoutTelegramAccountMutation()
const removeMutation = useRemoveTelegramAccountMutation()
const accountsQuery = useTelegramAccountsQuery()

const {
  defineField,
  errors,
  handleSubmit,
  resetForm,
  setFieldValue,
  values,
} = useForm<TelegramAccountSetupFormValues>({
  validationSchema: telegramAccountSetupSchema,
  initialValues: defaultTelegramAccountSetupValues(),
})

const [accountId] = defineField('account_id')
const [providerKind] = defineField('provider_kind')
const [displayName] = defineField('display_name')
const [externalAccountId] = defineField('external_account_id')
const [apiId] = defineField('api_id')
const [apiHash] = defineField('api_hash')
const [botToken] = defineField('bot_token')
const [sessionEncryptionKey] = defineField('session_encryption_key')
const [tdlibDataPath] = defineField('tdlib_data_path')
const [qrAuthorized] = defineField('qr_authorized')
const [transcriptionEnabled] = defineField('transcription_enabled')

const isBusy = computed(
  () =>
    setupMutation.isPending.value ||
    logoutMutation.isPending.value ||
    removeMutation.isPending.value
)

const accounts = computed(() => accountsQuery.data.value ?? [])

const submit = handleSubmit(async (formValues) => {
  await setupMutation.mutateAsync({
    account_id: formValues.account_id.trim(),
    provider_kind: formValues.provider_kind,
    display_name: formValues.display_name.trim(),
    external_account_id: formValues.external_account_id.trim(),
    api_id: formValues.provider_kind === 'telegram_user' && !formValues.qr_authorized ? formValues.api_id : undefined,
    api_hash: formValues.provider_kind === 'telegram_user' && !formValues.qr_authorized ? formValues.api_hash?.trim() : undefined,
    bot_token: formValues.provider_kind === 'telegram_bot' ? formValues.bot_token?.trim() : undefined,
    session_encryption_key: formValues.session_encryption_key?.trim() || undefined,
    tdlib_data_path: formValues.tdlib_data_path?.trim() || undefined,
    qr_authorized: formValues.provider_kind === 'telegram_user' ? formValues.qr_authorized : false,
    transcription_enabled: formValues.transcription_enabled,
  })
  resetForm({ values: defaultTelegramAccountSetupValues() })
  isSetupOpen.value = false
})

async function logoutAccount(accountId: string) {
  await logoutMutation.mutateAsync(accountId)
}

async function removeAccount(accountId: string) {
  await removeMutation.mutateAsync(accountId)
}

function lifecycleTone(state: string): 'active' | 'muted' | 'danger' {
  if (state === 'removed') return 'danger'
  if (state === 'logged_out') return 'muted'
  return 'active'
}

function applyQrSuggestedAccount(payload: {
  account_id: string
  display_name: string
  external_account_id: string
  qr_authorized: boolean
}) {
  setFieldValue('account_id', payload.account_id)
  setFieldValue('display_name', payload.display_name)
  setFieldValue('external_account_id', payload.external_account_id)
  setFieldValue('qr_authorized', payload.qr_authorized)
}
</script>

<template>
  <article class="telegram-account-manager telegram-rail-card">
    <div class="telegram-account-manager__header">
      <div>
        <h3>{{ t('Accounts') }}</h3>
        <p>{{ t('Manage Telegram account metadata, setup and lifecycle locally.') }}</p>
      </div>
      <button type="button" @click="isSetupOpen = !isSetupOpen">
        <Icon :icon="isSetupOpen ? 'tabler:x' : 'tabler:user-plus'" width="16" height="16" />
        {{ isSetupOpen ? t('Close') : t('Add Account') }}
      </button>
    </div>

    <form v-if="isSetupOpen" class="telegram-account-manager__form" @submit.prevent="submit">
      <label>
        <span>{{ t('Account ID') }}</span>
        <input v-model="accountId" type="text" autocomplete="off" />
        <small>{{ errors.account_id }}</small>
      </label>
      <label>
        <span>{{ t('Provider Kind') }}</span>
        <select v-model="providerKind">
          <option value="telegram_user">telegram_user</option>
          <option value="telegram_bot">telegram_bot</option>
        </select>
        <small>{{ errors.provider_kind }}</small>
      </label>
      <label>
        <span>{{ t('Display Name') }}</span>
        <input v-model="displayName" type="text" autocomplete="off" />
        <small>{{ errors.display_name }}</small>
      </label>
      <label>
        <span>{{ t('External Account ID') }}</span>
        <input v-model="externalAccountId" type="text" autocomplete="off" />
        <small>{{ errors.external_account_id }}</small>
      </label>

      <template v-if="providerKind === 'telegram_user'">
        <label class="telegram-account-manager__checkbox">
          <input v-model="qrAuthorized" type="checkbox" />
          <span>{{ t('QR authorized user account') }}</span>
        </label>
        <label v-if="!qrAuthorized">
          <span>{{ t('API ID') }}</span>
          <input v-model.number="apiId" type="number" min="1" step="1" />
          <small>{{ errors.api_id }}</small>
        </label>
        <label v-if="!qrAuthorized">
          <span>{{ t('API hash') }}</span>
          <input v-model="apiHash" type="password" autocomplete="off" />
          <small>{{ errors.api_hash }}</small>
        </label>
      </template>

      <label v-else>
        <span>{{ t('Bot token') }}</span>
        <input v-model="botToken" type="password" autocomplete="off" />
        <small>{{ errors.bot_token }}</small>
      </label>

      <label>
        <span>{{ t('Session encryption key') }}</span>
        <input v-model="sessionEncryptionKey" type="password" autocomplete="off" />
        <small>{{ errors.session_encryption_key }}</small>
      </label>
      <label>
        <span>TDLib data path</span>
        <input v-model="tdlibDataPath" type="text" autocomplete="off" />
        <small>{{ errors.tdlib_data_path }}</small>
      </label>
      <label class="telegram-account-manager__checkbox">
        <input v-model="transcriptionEnabled" type="checkbox" />
        <span>{{ t('Enable transcription') }}</span>
      </label>
      <TelegramQrLoginPanel
        v-if="providerKind === 'telegram_user'"
        :formValues="values"
        @applySuggested="applyQrSuggestedAccount"
      />
      <div class="telegram-account-manager__actions">
        <button type="submit" :disabled="isBusy">
          <Icon icon="tabler:device-floppy" width="16" height="16" />
          {{ t('Save Account') }}
        </button>
      </div>
    </form>

    <div v-if="accounts.length === 0" class="telegram-account-manager__empty">
      {{ t('No Telegram accounts configured yet.') }}
    </div>
    <div v-else class="telegram-account-manager__list">
      <article
        v-for="account in accounts"
        :key="account.account_id"
        class="telegram-account-manager__item"
        :data-state="lifecycleTone(account.lifecycle_state)"
      >
        <div>
          <strong>{{ account.display_name }}</strong>
          <p>{{ account.account_id }} · {{ account.provider_kind }}</p>
          <small>{{ account.runtime }} · {{ account.lifecycle_state }}</small>
        </div>
        <div class="telegram-account-manager__item-actions">
          <button
            type="button"
            :disabled="isBusy || account.lifecycle_state !== 'active'"
            @click="void logoutAccount(account.account_id)"
          >
            {{ t('Logout') }}
          </button>
          <button
            type="button"
            class="danger"
            :disabled="isBusy || account.lifecycle_state === 'removed'"
            @click="void removeAccount(account.account_id)"
          >
            {{ t('Remove') }}
          </button>
        </div>
        <em v-if="props.selectedAccountId === account.account_id">{{ t('Selected') }}</em>
      </article>
    </div>

    <TelegramCapabilityMatrix :accountId="props.selectedAccountId" />
  </article>
</template>

<style scoped>
.telegram-account-manager__header,
.telegram-account-manager__item,
.telegram-account-manager__item-actions,
.telegram-account-manager__actions {
  display: flex;
  gap: 10px;
}
.telegram-account-manager__header,
.telegram-account-manager__item {
  justify-content: space-between;
  align-items: flex-start;
}
.telegram-account-manager__header p,
.telegram-account-manager__item p,
.telegram-account-manager__item small,
.telegram-account-manager__empty,
.telegram-account-manager__form small {
  margin: 2px 0 0;
  font-size: 11px;
  color: var(--color-text-secondary, #777);
}
.telegram-account-manager__form,
.telegram-account-manager__list {
  display: flex;
  flex-direction: column;
  gap: 10px;
  margin-top: 12px;
}
.telegram-account-manager__form label {
  display: flex;
  flex-direction: column;
  gap: 4px;
  font-size: 12px;
}
.telegram-account-manager__form input,
.telegram-account-manager__form select {
  border: 1px solid var(--color-border, #e0e0e0);
  border-radius: 6px;
  padding: 6px 8px;
  font-size: 12px;
  background: var(--color-surface, #fff);
}
.telegram-account-manager__checkbox {
  flex-direction: row !important;
  align-items: center;
}
.telegram-account-manager__header button,
.telegram-account-manager__actions button,
.telegram-account-manager__item-actions button {
  display: inline-flex;
  align-items: center;
  gap: 4px;
  border: 1px solid var(--color-border, #e0e0e0);
  border-radius: 6px;
  background: var(--color-surface, #fff);
  padding: 5px 10px;
  font-size: 12px;
  cursor: pointer;
}
.telegram-account-manager__item {
  border: 1px solid var(--color-border, #e0e0e0);
  border-radius: 8px;
  padding: 10px;
  background: var(--color-surface, #fff);
}
.telegram-account-manager__item[data-state='active'] {
  border-color: var(--color-primary-light, #bbdefb);
}
.telegram-account-manager__item[data-state='danger'] {
  opacity: 0.7;
}
.telegram-account-manager__item-actions .danger {
  color: #b42318;
}
</style>
```

### `frontend/src/integrations/telegram/components/TelegramCallTranscriptPanel.vue`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/integrations/telegram/components/TelegramCallTranscriptPanel.vue`
- Size bytes / Размер в байтах: `4940`
- Included characters / Включено символов: `4932`
- Truncated / Обрезано: `no`

```text
<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import { useI18n } from '../../../platform/i18n'
import type { TelegramCall } from '../types/telegram'
import { useTelegramCallTranscriptQuery } from '../queries/useTelegramQuery'

const { t } = useI18n()

const props = defineProps<{
  calls: TelegramCall[]
}>()

const selectedCallId = ref<string | null>(null)
const transcriptQuery = useTelegramCallTranscriptQuery(selectedCallId)

const selectedCall = computed(() =>
  props.calls.find((call) => call.call_id === selectedCallId.value) ?? props.calls[0] ?? null
)

const selectedTranscript = computed(() => transcriptQuery.data.value)

watch(
  () => props.calls,
  (calls) => {
    if (calls.length === 0) {
      selectedCallId.value = null
      return
    }
    const hasCurrent = calls.some((call) => call.call_id === selectedCallId.value)
    if (!hasCurrent) {
      selectedCallId.value = calls[0]?.call_id ?? null
    }
  },
  { immediate: true }
)

function formatDate(value: string | null | undefined): string {
  if (!value) return '—'
  const date = new Date(value)
  if (Number.isNaN(date.getTime())) return '—'
  return new Intl.DateTimeFormat('en', {
    month: 'short',
    day: 'numeric',
    hour: '2-digit',
    minute: '2-digit',
  }).format(date)
}
</script>

<template>
  <div class="telegram-call-transcripts">
    <div class="telegram-call-list">
      <button
        v-for="call in calls"
        :key="call.call_id"
        type="button"
        class="telegram-call-row"
        :class="{ 'telegram-call-row--active': call.call_id === selectedCallId }"
        @click="selectedCallId = call.call_id"
      >
        <div>
          <strong>{{ call.status }}</strong>
          <p>{{ call.provider_chat_id }}</p>
        </div>
        <small>{{ formatDate(call.occurred_at) }}</small>
      </button>
    </div>

    <div v-if="selectedCall" class="telegram-call-transcript-card">
      <header>
        <strong>{{ t('Transcript') }}</strong>
        <small>{{ selectedCall.call_id }}</small>
      </header>
      <div v-if="transcriptQuery.isLoading.value" class="telegram-call-placeholder">
        {{ t('Loading Telegram transcript...') }}
      </div>
      <div v-else-if="!selectedTranscript" class="telegram-call-placeholder">
        {{ t('No transcript projected for this call yet.') }}
      </div>
      <div v-else class="telegram-call-transcript-copy">
        <dl>
          <div><dt>{{ t('Status') }}</dt><dd>{{ selectedTranscript.transcript_status }}</dd></div>
          <div><dt>{{ t('Provider') }}</dt><dd>{{ selectedTranscript.stt_provider }}</dd></div>
          <div><dt>{{ t('Language') }}</dt><dd>{{ selectedTranscript.language_code ?? '—' }}</dd></div>
          <div><dt>{{ t('Audio Ref') }}</dt><dd>{{ selectedTranscript.source_audio_ref ?? '—' }}</dd></div>
        </dl>
        <p>{{ selectedTranscript.transcript_text }}</p>
      </div>
    </div>
  </div>
</template>

<style scoped>
.telegram-call-transcripts {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.telegram-call-list {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.telegram-call-row {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: 10px;
  padding: 10px 12px;
  border-radius: 10px;
  border: 1px solid var(--color-border, #e0e0e0);
  background: var(--color-bg, #fafafa);
  text-align: left;
  cursor: pointer;
}

.telegram-call-row--active {
  border-color: var(--color-primary, #0066cc);
  background: var(--color-primary-subtle, #e3f2fd);
}

.telegram-call-row strong,
.telegram-call-transcript-card strong {
  display: block;
  font-size: 12px;
  color: var(--color-text, #333);
}

.telegram-call-row p,
.telegram-call-transcript-copy p {
  margin: 2px 0 0;
  font-size: 12px;
  color: var(--color-text, #333);
  word-break: break-word;
}

.telegram-call-row small,
.telegram-call-transcript-card small,
.telegram-call-transcript-copy dt {
  color: var(--color-text-secondary, #777);
  font-size: 11px;
}

.telegram-call-transcript-card {
  display: flex;
  flex-direction: column;
  gap: 10px;
  border-top: 1px solid var(--color-border, #e0e0e0);
  padding-top: 12px;
}

.telegram-call-transcript-card header {
  display: flex;
  align-items: baseline;
  justify-content: space-between;
  gap: 8px;
}

.telegram-call-transcript-copy {
  display: flex;
  flex-direction: column;
  gap: 10px;
}

.telegram-call-transcript-copy dl {
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: 8px 12px;
  margin: 0;
}

.telegram-call-transcript-copy dl div {
  min-width: 0;
}

.telegram-call-transcript-copy dd {
  margin: 2px 0 0;
  font-size: 12px;
  color: var(--color-text, #333);
  word-break: break-word;
}

.telegram-call-placeholder {
  padding: 10px 12px;
  border-radius: 10px;
  background: var(--color-bg, #fafafa);
  color: var(--color-text-secondary, #777);
  font-size: 12px;
}
</style>
```

### `frontend/src/integrations/telegram/components/TelegramCallsPanel.vue`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/integrations/telegram/components/TelegramCallsPanel.vue`
- Size bytes / Размер в байтах: `3433`
- Included characters / Включено символов: `3433`
- Truncated / Обрезано: `no`

```text
<script setup lang="ts">
import { computed, ref } from 'vue'
import { useI18n } from '../../../platform/i18n'
import Icon from '../../../shared/ui/Icon.vue'
import TelegramCallTranscriptPanel from './TelegramCallTranscriptPanel.vue'
import { useTelegramCallsQuery } from '../queries/useTelegramQuery'

const { t } = useI18n()

const props = defineProps<{
  selectedAccountId: string | null
}>()

const searchQuery = ref('')
const recentCallsQuery = useTelegramCallsQuery(computed(() => props.selectedAccountId ?? undefined), 10)
const recentCalls = computed(() => recentCallsQuery.data.value ?? [])
const filteredCalls = computed(() => {
  const query = searchQuery.value.trim().toLowerCase()
  if (!query) return recentCalls.value
  return recentCalls.value.filter((call) =>
    [call.call_id, call.provider_chat_id, call.status]
      .join(' ')
      .toLowerCase()
      .includes(query)
  )
})
</script>

<template>
  <article class="telegram-rail-card telegram-calls-panel">
    <header class="telegram-calls-panel__header">
      <div>
        <h3>{{ t('Recent Calls') }}</h3>
        <p>{{ t('Projected Telegram calls for the selected account.') }}</p>
      </div>
      <span class="telegram-calls-panel__count">{{ filteredCalls.length }}</span>
    </header>

    <label v-if="recentCalls.length > 0" class="telegram-calls-panel__search">
      <Icon icon="tabler:search" width="15" height="15" />
      <input
        v-model="searchQuery"
        type="search"
        :placeholder="t('Search projected calls')"
      />
    </label>

    <div v-if="!selectedAccountId" class="telegram-call-placeholder">
      {{ t('Select a Telegram account to inspect projected calls.') }}
    </div>
    <div v-else-if="recentCallsQuery.isLoading.value" class="telegram-call-placeholder">
      {{ t('Loading Telegram calls...') }}
    </div>
    <div v-else-if="recentCalls.length === 0" class="telegram-call-placeholder">
      {{ t('No Telegram calls projected for this account yet.') }}
    </div>
    <div v-else-if="filteredCalls.length === 0" class="telegram-call-placeholder">
      {{ t('No projected calls match this search.') }}
    </div>
    <TelegramCallTranscriptPanel v-else :calls="filteredCalls" />
  </article>
</template>

<style scoped>
.telegram-calls-panel {
  display: flex;
  flex-direction: column;
  gap: 10px;
}

.telegram-calls-panel__header,
.telegram-calls-panel__search {
  display: flex;
  align-items: center;
  gap: 8px;
}

.telegram-calls-panel__header {
  justify-content: space-between;
  align-items: flex-start;
}

.telegram-calls-panel__header h3,
.telegram-calls-panel__header p {
  margin: 0;
}

.telegram-calls-panel__header p,
.telegram-calls-panel__search {
  font-size: 11px;
  color: var(--color-text-secondary, #777);
}

.telegram-calls-panel__count {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  min-width: 24px;
  min-height: 24px;
  padding: 0 8px;
  border-radius: 999px;
  background: var(--color-primary-subtle, #e3f2fd);
  color: var(--color-primary, #0066cc);
  font-size: 11px;
  font-weight: 600;
}

.telegram-calls-panel__search {
  padding: 8px 10px;
  border: 1px solid var(--color-border, #e0e0e0);
  border-radius: 999px;
  background: var(--color-surface, #fff);
}

.telegram-calls-panel__search input {
  flex: 1;
  border: none;
  background: transparent;
  font: inherit;
  color: var(--color-text, #333);
  outline: none;
}
</style>
```

### `frontend/src/integrations/telegram/components/TelegramCapabilityMatrix.vue`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/integrations/telegram/components/TelegramCapabilityMatrix.vue`
- Size bytes / Размер в байтах: `5359`
- Included characters / Включено символов: `5355`
- Truncated / Обрезано: `no`

```text
<script setup lang="ts">
import { computed } from 'vue'
import { useI18n } from '../../../platform/i18n'
import { useTelegramAccountCapabilitiesQuery } from '../queries/useTelegramQuery'

const { t } = useI18n()

const props = defineProps<{
  accountId: string | null
}>()

const capabilitiesQuery = useTelegramAccountCapabilitiesQuery(computed(() => props.accountId))
const accountScope = computed(() => capabilitiesQuery.data.value?.account_scope ?? null)
const capabilityRows = computed(() => capabilitiesQuery.data.value?.capabilities ?? [])
const plannedFeatures = computed(() => capabilitiesQuery.data.value?.planned_features ?? [])
const unsupportedFeatures = computed(() => capabilitiesQuery.data.value?.unsupported_features ?? [])

function tone(status: string): string {
  if (status === 'available') return 'success'
  if (status === 'degraded') return 'warning'
  if (status === 'planned') return 'planned'
  if (status === 'blocked') return 'danger'
  return 'muted'
}
</script>

<template>
  <section class="telegram-capability-matrix telegram-rail-card">
    <header class="telegram-capability-matrix__header">
      <div>
        <h3>{{ t('Capabilities') }}</h3>
        <p>{{ t('Account-scoped capability contract for the current Telegram account.') }}</p>
      </div>
    </header>

    <div v-if="!accountId" class="telegram-capability-matrix__state">
      {{ t('Select a Telegram chat/account to inspect capability state.') }}
    </div>
    <div v-else-if="capabilitiesQuery.isLoading.value" class="telegram-capability-matrix__state">
      {{ t('Loading Telegram capabilities...') }}
    </div>
    <div v-else-if="!accountScope" class="telegram-capability-matrix__state">
      {{ t('No account-scoped capability payload is available.') }}
    </div>
    <div v-else class="telegram-capability-matrix__body">
      <dl class="telegram-capability-matrix__scope">
        <div><dt>{{ t('Account') }}</dt><dd>{{ accountScope.account_id }}</dd></div>
        <div><dt>{{ t('Provider') }}</dt><dd>{{ accountScope.provider_kind }}</dd></div>
        <div><dt>{{ t('Runtime') }}</dt><dd>{{ accountScope.runtime_kind }}</dd></div>
        <div><dt>{{ t('Lifecycle') }}</dt><dd>{{ accountScope.lifecycle_state }}</dd></div>
      </dl>

      <div v-if="plannedFeatures.length > 0" class="telegram-capability-matrix__features">
        <strong>{{ t('Planned Initiatives') }}</strong>
        <p>{{ plannedFeatures.join(' · ') }}</p>
      </div>

      <div v-if="unsupportedFeatures.length > 0" class="telegram-capability-matrix__features">
        <strong>{{ t('Unsupported Features') }}</strong>
        <p>{{ unsupportedFeatures.join(' · ') }}</p>
      </div>

      <div class="telegram-capability-matrix__list">
        <article v-for="capability in capabilityRows" :key="capability.operation" class="telegram-capability-matrix__item">
          <div class="telegram-capability-matrix__item-head">
            <strong>{{ capability.operation }}</strong>
            <span :data-tone="tone(capability.status)">{{ capability.status }}</span>
          </div>
          <small>{{ capability.category }} · {{ capability.action_class }}</small>
          <p>{{ capability.reason }}</p>
          <small>
            {{ t('Confirm') }}: {{ capability.confirmation_required ? 'yes' : 'no' }}
            · {{ t('Closure gate') }}: {{ capability.closure_gate ? 'yes' : 'no' }}
          </small>
        </article>
      </div>
    </div>
  </section>
</template>

<style scoped>
.telegram-capability-matrix,
.telegram-capability-matrix__body,
.telegram-capability-matrix__list {
  display: flex;
  flex-direction: column;
  gap: 10px;
}

.telegram-capability-matrix__header p,
.telegram-capability-matrix__state,
.telegram-capability-matrix__item small,
.telegram-capability-matrix__scope dt {
  margin: 2px 0 0;
  font-size: 11px;
  color: var(--color-text-secondary, #777);
}

.telegram-capability-matrix__scope {
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: 8px 12px;
  margin: 0;
}

.telegram-capability-matrix__scope dd {
  margin: 2px 0 0;
  font-size: 12px;
  color: var(--color-text, #333);
  word-break: break-word;
}

.telegram-capability-matrix__features p,
.telegram-capability-matrix__item p {
  margin: 2px 0 0;
  font-size: 12px;
  color: var(--color-text, #333);
  word-break: break-word;
}

.telegram-capability-matrix__item {
  padding: 8px;
  border-radius: 8px;
  background: var(--color-surface, #fff);
  border: 1px solid var(--color-border, #e0e0e0);
}

.telegram-capability-matrix__item-head {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 8px;
}

.telegram-capability-matrix__item-head span {
  border-radius: 999px;
  padding: 2px 8px;
  font-size: 11px;
  text-transform: lowercase;
}

.telegram-capability-matrix__item-head span[data-tone='success'] {
  background: #e7f6ec;
  color: #206a3a;
}

.telegram-capability-matrix__item-head span[data-tone='warning'] {
  background: #fff4e5;
  color: #9a5b00;
}

.telegram-capability-matrix__item-head span[data-tone='danger'] {
  background: #fdecea;
  color: #b42318;
}

.telegram-capability-matrix__item-head span[data-tone='planned'] {
  background: #eef4ff;
  color: #3538cd;
}

.telegram-capability-matrix__item-head span[data-tone='muted'] {
  background: #f2f4f7;
  color: #667085;
}
</style>
```

### `frontend/src/integrations/telegram/components/TelegramCommandAuditPanel.vue`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/integrations/telegram/components/TelegramCommandAuditPanel.vue`
- Size bytes / Размер в байтах: `8821`
- Included characters / Включено символов: `8814`
- Truncated / Обрезано: `no`

```text
<script setup lang="ts">
import { computed, ref } from 'vue'
import { useI18n } from '../../../platform/i18n'
import Icon from '../../../shared/ui/Icon.vue'
import type { TelegramProviderWriteCommand } from '../types/telegram'
import { useTelegramCommandRetryMutation, useTelegramCommandsQuery } from '../queries/useTelegramLifecycleQuery'
import {
  telegramCommandAuditState,
  telegramCommandRetrySummary,
  telegramCommandSubject
} from '../stores/telegramCommandAudit'

const { t } = useI18n()

const props = defineProps<{
  selectedAccountId: string | null
  selectedProviderChatId: string | null
}>()

const searchQuery = ref('')
const currentChatOnly = ref(true)
const commandsQuery = useTelegramCommandsQuery(
  computed(() => props.selectedAccountId),
  20,
  true,
  {
    providerChatId: computed(() =>
      currentChatOnly.value ? props.selectedProviderChatId : null
    ),
  }
)
const retryMutation = useTelegramCommandRetryMutation()
const commands = computed(() => commandsQuery.data.value ?? [])
const filteredCommands = computed(() => {
  const query = searchQuery.value.trim().toLowerCase()
  return commands.value.filter((command) => {
    if (
      currentChatOnly.value &&
      props.selectedProviderChatId &&
      command.provider_chat_id !== props.selectedProviderChatId
    ) {
      return false
    }
    if (!query) return true
    return [
      command.command_kind,
      command.status,
      command.provider_chat_id,
      command.provider_message_id ?? '',
      command.capability_state,
      command.action_class,
    ]
      .join(' ')
      .toLowerCase()
      .includes(query)
  })
})

function formatDate(value: string | null | undefined): string {
  if (!value) return '—'
  const date = new Date(value)
  if (Number.isNaN(date.getTime())) return '—'
  return new Intl.DateTimeFormat('en', {
    month: 'short',
    day: 'numeric',
    hour: '2-digit',
    minute: '2-digit',
  }).format(date)
}

function commandTitle(command: TelegramProviderWriteCommand): string {
  return [command.command_kind, command.status].join(' · ')
}

function commandStateClass(command: TelegramProviderWriteCommand): string {
  return `telegram-command-audit__state--${telegramCommandAuditState(command).tone}`
}

function canRetry(command: TelegramProviderWriteCommand): boolean {
  return command.status === 'dead_letter' || command.status === 'failed'
}
</script>

<template>
  <article class="telegram-rail-card telegram-command-audit">
    <header class="telegram-command-audit__header">
      <div>
        <h3>{{ t('Recent Commands') }}</h3>
        <p>{{ t('Durable Telegram command rows for the selected account.') }}</p>
      </div>
      <label class="telegram-command-audit__toggle">
        <input v-model="currentChatOnly" type="checkbox" />
        <span>{{ t('Current chat only') }}</span>
      </label>
    </header>

    <label v-if="commands.length > 0" class="telegram-command-audit__search">
      <Icon icon="tabler:search" width="15" height="15" />
      <input
        v-model="searchQuery"
        type="search"
        :placeholder="t('Search command rows')"
      />
    </label>

    <div v-if="!selectedAccountId" class="telegram-call-placeholder">
      {{ t('Select a Telegram account to inspect command audit rows.') }}
    </div>
    <div v-else-if="commandsQuery.isLoading.value" class="telegram-call-placeholder">
      {{ t('Loading Telegram command audit...') }}
    </div>
    <div v-else-if="commands.length === 0" class="telegram-call-placeholder">
      {{ t('No Telegram command rows projected for this account yet.') }}
    </div>
    <div v-else-if="filteredCommands.length === 0" class="telegram-call-placeholder">
      {{ t('No Telegram command rows match this filter.') }}
    </div>
    <div v-else class="telegram-command-audit__list">
      <article
        v-for="command in filteredCommands"
        :key="command.command_id"
        class="telegram-command-audit__item"
        :class="{ 'telegram-command-audit__item--dead-letter': telegramCommandAuditState(command).is_dead_letter }"
      >
        <div class="telegram-command-audit__row">
          <strong>{{ commandTitle(command) }}</strong>
          <small>{{ formatDate(command.happened_at) }}</small>
        </div>
        <p>{{ telegramCommandSubject(command) }}</p>
        <div class="telegram-command-audit__badges">
          <span class="telegram-command-audit__state" :class="commandStateClass(command)">
            {{ t(telegramCommandAuditState(command).label) }}
          </span>
          <span>{{ telegramCommandRetrySummary(command) }}</span>
        </div>
        <small>
          {{ command.capability_state }} · {{ command.action_class }} · {{ command.confirmation_decision }}
        </small>
        <small>{{ t('Reconciliation') }}: {{ command.reconciliation_status }}</small>
        <small>{{ telegramCommandAuditState(command).detail }}</small>
        <button
          v-if="canRetry(command)"
          type="button"
          class="telegram-command-audit__retry"
          :disabled="retryMutation.isPending.value"
          @click="retryMutation.mutate(command.command_id)"
        >
          <Icon icon="tabler:refresh" width="14" height="14" />
          {{ t('Retry command') }}
        </button>
      </article>
    </div>
  </article>
</template>

<style scoped>
.telegram-command-audit,
.telegram-command-audit__list {
  display: flex;
  flex-direction: column;
  gap: 10px;
}

.telegram-command-audit__header,
.telegram-command-audit__row,
.telegram-command-audit__toggle,
.telegram-command-audit__search,
.telegram-command-audit__badges {
  display: flex;
  align-items: center;
  gap: 8px;
}

.telegram-command-audit__header {
  justify-content: space-between;
  align-items: flex-start;
}

.telegram-command-audit__header h3,
.telegram-command-audit__header p,
.telegram-command-audit__item p,
.telegram-command-audit__item small {
  margin: 0;
}

.telegram-command-audit__header p,
.telegram-command-audit__item small,
.telegram-command-audit__toggle,
.telegram-command-audit__search {
  font-size: 11px;
  color: var(--color-text-secondary, #777);
}

.telegram-command-audit__search {
  padding: 8px 10px;
  border: 1px solid var(--color-border, #e0e0e0);
  border-radius: 999px;
  background: var(--color-surface, #fff);
}

.telegram-command-audit__search input {
  flex: 1;
  border: none;
  background: transparent;
  font: inherit;
  color: var(--color-text, #333);
  outline: none;
}

.telegram-command-audit__item {
  display: flex;
  flex-direction: column;
  gap: 4px;
  padding: 10px 12px;
  border-radius: 10px;
  border: 1px solid var(--color-border, #e0e0e0);
  background: var(--color-surface, #fff);
}

.telegram-command-audit__item--dead-letter {
  border-color: color-mix(in srgb, var(--color-danger, #b42318) 55%, var(--color-border, #e0e0e0));
}

.telegram-command-audit__row {
  justify-content: space-between;
}

.telegram-command-audit__item strong,
.telegram-command-audit__item p {
  font-size: 12px;
  color: var(--color-text, #333);
  word-break: break-word;
}

.telegram-command-audit__badges {
  flex-wrap: wrap;
  font-size: 11px;
  color: var(--color-text-secondary, #777);
}

.telegram-command-audit__state {
  padding: 2px 7px;
  border-radius: 999px;
  border: 1px solid var(--color-border, #e0e0e0);
  background: var(--color-bg, #fafafa);
  color: var(--color-text-secondary, #777);
}

.telegram-command-audit__state--danger {
  border-color: color-mix(in srgb, var(--color-danger, #b42318) 55%, transparent);
  background: color-mix(in srgb, var(--color-danger, #b42318) 12%, transparent);
  color: var(--color-danger, #b42318);
}

.telegram-command-audit__state--warning {
  border-color: color-mix(in srgb, var(--color-warning, #b54708) 55%, transparent);
  background: color-mix(in srgb, var(--color-warning, #b54708) 12%, transparent);
  color: var(--color-warning, #b54708);
}

.telegram-command-audit__state--progress {
  border-color: color-mix(in srgb, var(--color-accent, #2563eb) 45%, transparent);
  background: color-mix(in srgb, var(--color-accent, #2563eb) 10%, transparent);
  color: var(--color-accent, #2563eb);
}

.telegram-command-audit__state--success {
  border-color: color-mix(in srgb, var(--color-success, #027a48) 45%, transparent);
  background: color-mix(in srgb, var(--color-success, #027a48) 10%, transparent);
  color: var(--color-success, #027a48);
}

.telegram-command-audit__retry {
  display: inline-flex;
  align-items: center;
  align-self: flex-start;
  gap: 5px;
  border: 1px solid var(--color-border, #d6dce3);
  border-radius: 999px;
  background: var(--color-surface, #fff);
  color: var(--color-text, #333);
  padding: 5px 9px;
  cursor: pointer;
}

.telegram-command-audit__retry:disabled {
  cursor: not-allowed;
  opacity: 0.55;
}
</style>
```

### `frontend/src/integrations/telegram/components/TelegramQrLoginPanel.vue`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/integrations/telegram/components/TelegramQrLoginPanel.vue`
- Size bytes / Размер в байтах: `8190`
- Included characters / Включено символов: `8184`
- Truncated / Обрезано: `no`

```text
<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import { useI18n } from '../../../platform/i18n'
import Icon from '../../../shared/ui/Icon.vue'
import type { TelegramQrLoginStatusResponse } from '../types/telegram'
import type { TelegramAccountSetupFormValues } from '../forms/telegramAccountSetupForm'
import {
  useCancelTelegramQrLoginMutation,
  useStartTelegramQrLoginMutation,
  useSubmitTelegramQrPasswordMutation,
  useTelegramQrLoginStatusQuery,
} from '../queries/useTelegramQrLoginQuery'

const { t } = useI18n()

const props = defineProps<{
  formValues: TelegramAccountSetupFormValues
}>()

const emit = defineEmits<{
  applySuggested: [payload: {
    account_id: string
    display_name: string
    external_account_id: string
    qr_authorized: boolean
  }]
}>()

const setupId = ref<string | null>(null)
const localStatus = ref<TelegramQrLoginStatusResponse | null>(null)
const password = ref('')
const startMutation = useStartTelegramQrLoginMutation()
const statusQuery = useTelegramQrLoginStatusQuery(setupId)
const cancelMutation = useCancelTelegramQrLoginMutation(setupId)
const passwordMutation = useSubmitTelegramQrPasswordMutation(setupId)

const activeStatus = computed(() => statusQuery.data.value ?? localStatus.value)
const isBusy = computed(
  () =>
    startMutation.isPending.value ||
    statusQuery.isFetching.value ||
    cancelMutation.isPending.value ||
    passwordMutation.isPending.value
)
const canStart = computed(
  () =>
    props.formValues.provider_kind === 'telegram_user' &&
    props.formValues.account_id.trim().length > 0 &&
    props.formValues.display_name.trim().length > 0 &&
    props.formValues.external_account_id.trim().length > 0 &&
    (props.formValues.tdlib_data_path?.trim().length ?? 0) > 0
)

watch(
  () => statusQuery.data.value,
  (value) => {
    if (!value) return
    localStatus.value = value
  }
)

function clearSessionState() {
  setupId.value = null
  localStatus.value = null
  password.value = ''
}

async function startQrLogin() {
  const response = await startMutation.mutateAsync({
    account_id: props.formValues.account_id.trim(),
    display_name: props.formValues.display_name.trim(),
    external_account_id: props.formValues.external_account_id.trim(),
    api_id: props.formValues.api_id,
    api_hash: props.formValues.api_hash?.trim() || undefined,
    session_encryption_key: props.formValues.session_encryption_key?.trim() || undefined,
    tdlib_data_path: props.formValues.tdlib_data_path?.trim() || undefined,
    transcription_enabled: props.formValues.transcription_enabled,
  })
  setupId.value = response.setup_id
  localStatus.value = response
  password.value = ''
}

async function cancelQrLogin() {
  await cancelMutation.mutateAsync()
  clearSessionState()
}

async function submitPassword() {
  const response = await passwordMutation.mutateAsync({ password: password.value })
  localStatus.value = response
  password.value = ''
}

async function refreshStatus() {
  await statusQuery.refetch()
}

function applySuggested() {
  const status = activeStatus.value
  if (!status) return
  emit('applySuggested', {
    account_id: status.suggested_account_id ?? props.formValues.account_id.trim(),
    display_name: status.suggested_display_name ?? props.formValues.display_name.trim(),
    external_account_id: status.suggested_external_account_id ?? props.formValues.external_account_id.trim(),
    qr_authorized: status.status === 'ready',
  })
}
</script>

<template>
  <section class="telegram-qr-panel">
    <header class="telegram-qr-panel__header">
      <div>
        <strong>{{ t('QR Login') }}</strong>
        <p>{{ t('Use TDLib QR authorization for a Telegram user account before saving it locally.') }}</p>
      </div>
      <button type="button" :disabled="isBusy || !canStart" @click="void startQrLogin()">
        <Icon icon="tabler:qrcode" width="16" height="16" />
        {{ t('Start QR') }}
      </button>
    </header>

    <p v-if="!canStart" class="telegram-qr-panel__hint">
      {{ t('Fill account ID, display name, external account ID and TDLib data path before starting QR login.') }}
    </p>

    <div v-if="activeStatus" class="telegram-qr-panel__status">
      <div class="telegram-qr-panel__meta">
        <strong>{{ activeStatus.status }}</strong>
        <small>{{ activeStatus.message ?? activeStatus.setup_id }}</small>
      </div>

      <div v-if="activeStatus.qr_svg" class="telegram-qr-panel__qr" v-html="activeStatus.qr_svg" />

      <label v-if="activeStatus.status === 'waiting_password'" class="telegram-qr-panel__password">
        <span>{{ t('2FA Password') }}</span>
        <input v-model="password" type="password" autocomplete="off" />
      </label>

      <div class="telegram-qr-panel__actions">
        <button
          v-if="activeStatus.status === 'waiting_qr_scan' || activeStatus.status === 'waiting_password'"
          type="button"
          :disabled="isBusy"
          @click="void refreshStatus()"
        >
          <Icon icon="tabler:refresh" width="16" height="16" />
          {{ t('Refresh Status') }}
        </button>
        <button
          v-if="activeStatus.status === 'waiting_password'"
          type="button"
          :disabled="isBusy || password.trim().length === 0"
          @click="void submitPassword()"
        >
          {{ t('Submit Password') }}
        </button>
        <button
          v-if="activeStatus.status === 'ready'"
          type="button"
          :disabled="isBusy"
          @click="applySuggested"
        >
          {{ t('Apply Suggested Account') }}
        </button>
        <button
          v-if="activeStatus.status !== 'ready' && activeStatus.status !== 'expired' && activeStatus.status !== 'failed'"
          type="button"
          :disabled="isBusy"
          @click="void cancelQrLogin()"
        >
          {{ t('Cancel QR') }}
        </button>
      </div>

      <dl v-if="activeStatus.status === 'ready'" class="telegram-qr-panel__identity">
        <div><dt>{{ t('Suggested Account ID') }}</dt><dd>{{ activeStatus.suggested_account_id ?? '—' }}</dd></div>
        <div><dt>{{ t('Suggested Display Name') }}</dt><dd>{{ activeStatus.suggested_display_name ?? '—' }}</dd></div>
        <div><dt>{{ t('Suggested External ID') }}</dt><dd>{{ activeStatus.suggested_external_account_id ?? '—' }}</dd></div>
      </dl>
    </div>
  </section>
</template>

<style scoped>
.telegram-qr-panel {
  display: flex;
  flex-direction: column;
  gap: 10px;
  border: 1px solid var(--color-border, #e0e0e0);
  border-radius: 8px;
  padding: 10px;
  background: var(--color-surface, #fff);
}

.telegram-qr-panel__header,
.telegram-qr-panel__actions {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: 10px;
}

.telegram-qr-panel__header p,
.telegram-qr-panel__hint,
.telegram-qr-panel__meta small,
.telegram-qr-panel__identity dt {
  margin: 2px 0 0;
  font-size: 11px;
  color: var(--color-text-secondary, #777);
}

.telegram-qr-panel__header button,
.telegram-qr-panel__actions button {
  display: inline-flex;
  align-items: center;
  gap: 4px;
  border: 1px solid var(--color-border, #e0e0e0);
  border-radius: 6px;
  background: var(--color-surface, #fff);
  padding: 5px 10px;
  font-size: 12px;
  cursor: pointer;
}

.telegram-qr-panel__status {
  display: flex;
  flex-direction: column;
  gap: 10px;
}

.telegram-qr-panel__meta {
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.telegram-qr-panel__qr {
  display: flex;
  justify-content: center;
  padding: 8px;
  border-radius: 8px;
  background: var(--color-bg, #fafafa);
}

.telegram-qr-panel__password {
  display: flex;
  flex-direction: column;
  gap: 4px;
  font-size: 12px;
}

.telegram-qr-panel__password input {
  border: 1px solid var(--color-border, #e0e0e0);
  border-radius: 6px;
  padding: 6px 8px;
  font-size: 12px;
  background: var(--color-surface, #fff);
}

.telegram-qr-panel__identity {
  display: grid;
  grid-template-columns: minmax(0, 1fr);
  gap: 8px;
  margin: 0;
}

.telegram-qr-panel__identity dd {
  margin: 2px 0 0;
  font-size: 12px;
  color: var(--color-text, #333);
  word-break: break-word;
}
</style>
```

### `frontend/src/integrations/telegram/components/TelegramStatusMessages.vue`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/integrations/telegram/components/TelegramStatusMessages.vue`
- Size bytes / Размер в байтах: `2567`
- Included characters / Включено символов: `2567`
- Truncated / Обрезано: `no`

```text
<script setup lang="ts">
import { useRealtimeStatusStore } from '../../../shared/stores/realtimeStatus'
import { useI18n } from '../../../platform/i18n'

const { t } = useI18n()
const realtimeStatus = useRealtimeStatusStore()

defineProps<{
  actionMessage: string
  error: string
  realtimeStatusLabel: string
  realtimeStatusDetail: string
  realtimeRecoveryDetail: string
  realtimeStatusTone: 'neutral' | 'success' | 'warning' | 'danger'
}>()
</script>

<template>
  <p
    class="telegram-realtime-state"
    :class="realtimeStatusTone"
    :title="realtimeStatusDetail"
    :aria-label="realtimeStatusDetail"
  >
    {{ t('Realtime') }}: {{ realtimeStatusLabel }}
  </p>
  <p class="telegram-recovery-state" :title="realtimeRecoveryDetail" :aria-label="realtimeRecoveryDetail">
    {{ t('Recovery') }}: {{ realtimeRecoveryDetail }}
    <button
      v-if="realtimeStatus.canTriggerReconnect"
      type="button"
      class="telegram-recovery-state__action"
      :title="t('Reconnect realtime')"
      @click="realtimeStatus.requestReconnect()"
    >
      {{ t('Reconnect realtime') }}
    </button>
  </p>
  <p v-if="actionMessage" class="setup-state success">{{ actionMessage }}</p>
  <p v-if="error" class="inline-error">{{ error }}</p>
</template>

<style scoped>
.setup-state {
  padding: 8px 16px;
  margin: 0;
  font-size: 13px;
  border-radius: 6px;
}
.success {
  background: var(--color-success-bg, #e6f7e6);
  color: var(--color-success-text, #2e7d32);
}
.inline-error {
  padding: 8px 16px;
  margin: 0;
  font-size: 13px;
  background: var(--color-error-bg, #fdecea);
  color: var(--color-error-text, #c62828);
  border-radius: 6px;
}
.telegram-realtime-state {
  padding: 6px 16px;
  margin: 0;
  font-size: 12px;
  color: var(--color-text-muted, #666);
  background: var(--color-surface-subtle, #f7f7f7);
  border-bottom: 1px solid var(--color-border, #e0e0e0);
}
.telegram-recovery-state {
  padding: 4px 16px 6px;
  margin: 0;
  font-size: 11px;
  color: var(--color-text-secondary, #777);
  background: var(--color-surface-subtle, #f7f7f7);
  border-bottom: 1px solid var(--color-border, #e0e0e0);
}
.telegram-recovery-state__action {
  margin-left: 8px;
  padding: 0;
  border: none;
  background: transparent;
  color: var(--color-accent, #2563eb);
  font: inherit;
  cursor: pointer;
}
.telegram-realtime-state.success {
  color: var(--color-success-text, #2e7d32);
}
.telegram-realtime-state.warning {
  color: var(--color-warning-text, #946200);
}
.telegram-realtime-state.danger {
  color: var(--color-error-text, #c62828);
}
</style>
```

### `frontend/src/integrations/telegram/views/TelegramRuntimePanel.vue`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/integrations/telegram/views/TelegramRuntimePanel.vue`
- Size bytes / Размер в байтах: `7077`
- Included characters / Включено символов: `7070`
- Truncated / Обрезано: `no`

```text
<script setup lang="ts">
import { computed, ref } from 'vue'
import { useI18n } from '../../../platform/i18n'
import Icon from '../../../shared/ui/Icon.vue'
import { useRealtimeStatusStore } from '../../../shared/stores/realtimeStatus'
import TelegramAccountManager from '../components/TelegramAccountManager.vue'
import TelegramCapabilityMatrix from '../components/TelegramCapabilityMatrix.vue'
import {
  useTelegramAccountsQuery,
  useTelegramCapabilitiesQuery,
} from '../queries/useTelegramQuery'
import {
  useRestartTelegramRuntimeMutation,
  useStartTelegramRuntimeMutation,
  useStopTelegramRuntimeMutation,
  useTelegramRuntimeStatusQuery,
} from '../queries/useTelegramRuntimeQuery'

const { t } = useI18n()
const realtimeStatus = useRealtimeStatusStore()
const selectedAccountId = ref<string | null>(null)
const actionMessage = ref('')
const actionError = ref('')

const accountsQuery = useTelegramAccountsQuery()
const capabilitiesQuery = useTelegramCapabilitiesQuery()
const accounts = computed(() => accountsQuery.data.value ?? [])
const selectedAccount = computed(() =>
  accounts.value.find((account) => account.account_id === selectedAccountId.value) ?? accounts.value[0] ?? null
)
const runtimeStatusQuery = useTelegramRuntimeStatusQuery(computed(() => selectedAccount.value?.account_id ?? null))
const startRuntimeMutation = useStartTelegramRuntimeMutation()
const stopRuntimeMutation = useStopTelegramRuntimeMutation()
const restartRuntimeMutation = useRestartTelegramRuntimeMutation()
const isRuntimeBusy = computed(() =>
  startRuntimeMutation.isPending.value ||
  stopRuntimeMutation.isPending.value ||
  restartRuntimeMutation.isPending.value
)

async function setTelegramRuntime(action: 'start' | 'stop' | 'restart') {
  const accountId = selectedAccount.value?.account_id
  if (!accountId || isRuntimeBusy.value) return
  actionMessage.value = ''
  actionError.value = ''
  try {
    const mutation = action === 'start'
      ? startRuntimeMutation
      : action === 'stop'
        ? stopRuntimeMutation
        : restartRuntimeMutation
    const status = await mutation.mutateAsync({ account_id: accountId })
    actionMessage.value = `Telegram runtime ${status.status}`
  } catch (error) {
    actionError.value = error instanceof Error ? error.message : String(error)
  }
}
</script>

<template>
  <section class="telegram-runtime-panel communications-page">
    <header class="view-header">
      <div class="view-title-with-icon">
        <span class="hero-mark small">
          <Icon icon="tabler:brand-telegram" width="28" height="28" />
        </span>
        <div>
          <h1>{{ t('Telegram Runtime') }}</h1>
          <p>{{ t('Provider setup, runtime status and control') }}</p>
        </div>
      </div>
      <button type="button" class="primary-button" :disabled="accountsQuery.isFetching.value" @click="accountsQuery.refetch()">
        <Icon icon="tabler:refresh" width="16" height="16" />{{ t('Refresh') }}
      </button>
    </header>

    <p
      class="telegram-realtime-state"
      :class="realtimeStatus.realtimeStatusTone"
      :title="realtimeStatus.realtimeStatusDetail"
    >
      {{ t('Realtime') }}: {{ realtimeStatus.realtimeStatusLabel }}
    </p>
    <p v-if="actionMessage" class="setup-state success">{{ actionMessage }}</p>
    <p v-if="actionError" class="inline-error">{{ actionError }}</p>

    <div class="telegram-runtime-grid">
      <section class="panel telegram-runtime-card">
        <header>
          <h2>{{ t('Accounts') }}</h2>
          <span>{{ accounts.length }}</span>
        </header>
        <label class="runtime-field">
          <span>{{ t('Selected account') }}</span>
          <select v-model="selectedAccountId">
            <option :value="null">{{ t('Auto') }}</option>
            <option v-for="account in accounts" :key="account.account_id" :value="account.account_id">
              {{ account.display_name }} · {{ account.account_id }}
            </option>
          </select>
        </label>
        <TelegramAccountManager :selectedAccountId="selectedAccount?.account_id ?? null" />
      </section>

      <section class="panel telegram-runtime-card">
        <header>
          <h2>{{ t('Runtime') }}</h2>
          <span>{{ runtimeStatusQuery.data.value?.status ?? t('unknown') }}</span>
        </header>
        <div class="runtime-actions">
          <button type="button" :disabled="isRuntimeBusy || !selectedAccount" @click="setTelegramRuntime('start')">
            <Icon icon="tabler:player-play" width="16" height="16" />{{ t('Start') }}
          </button>
          <button type="button" :disabled="isRuntimeBusy || !selectedAccount" @click="setTelegramRuntime('stop')">
            <Icon icon="tabler:player-stop" width="16" height="16" />{{ t('Stop') }}
          </button>
          <button type="button" :disabled="isRuntimeBusy || !selectedAccount" @click="setTelegramRuntime('restart')">
            <Icon icon="tabler:reload" width="16" height="16" />{{ t('Restart') }}
          </button>
        </div>
        <dl class="runtime-details">
          <div><dt>{{ t('Account') }}</dt><dd>{{ selectedAccount?.account_id ?? '—' }}</dd></div>
          <div><dt>{{ t('Mode') }}</dt><dd>{{ capabilitiesQuery.data.value?.runtime_mode ?? '—' }}</dd></div>
          <div><dt>TDLib</dt><dd>{{ runtimeStatusQuery.data.value?.tdjson_runtime_available ? t('available') : t('unavailable') }}</dd></div>
          <div><dt>{{ t('Last sync') }}</dt><dd>{{ runtimeStatusQuery.data.value?.last_sync_status ?? '—' }}</dd></div>
        </dl>
      </section>

      <TelegramCapabilityMatrix :accountId="selectedAccount?.account_id ?? null" />
    </div>
  </section>
</template>

<style scoped>
.telegram-runtime-panel {
  display: flex;
  flex-direction: column;
  height: 100%;
  overflow: auto;
}
.view-header,
.view-title-with-icon,
.telegram-runtime-card header,
.runtime-actions {
  display: flex;
  align-items: center;
  gap: 0.75rem;
}
.view-header,
.telegram-runtime-card header {
  justify-content: space-between;
}
.view-header {
  padding: 0.75rem 1rem;
  border-bottom: 1px solid var(--hh-border, #d9e2ec);
}
.telegram-runtime-grid {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(320px, 1fr));
  gap: 1rem;
  padding: 1rem;
}
.telegram-runtime-card {
  padding: 1rem;
}
.runtime-field,
.runtime-details {
  display: grid;
  gap: 0.5rem;
}
.runtime-field {
  margin-bottom: 1rem;
}
.runtime-field select {
  min-height: 2rem;
}
.runtime-actions button,
.primary-button {
  display: inline-flex;
  align-items: center;
  gap: 0.35rem;
  border: 1px solid var(--hh-border, #d9e2ec);
  border-radius: 6px;
  background: var(--hh-bg-primary, #fff);
  color: inherit;
  padding: 0.4rem 0.65rem;
  cursor: pointer;
}
.runtime-details {
  grid-template-columns: repeat(2, minmax(0, 1fr));
  margin: 1rem 0;
}
.runtime-details dd {
  margin: 0.15rem 0 0;
}
.telegram-realtime-state,
.setup-state,
.inline-error {
  padding: 0.6rem 1rem;
  margin: 0;
}
.success {
  color: #206a3a;
}
.inline-error {
  color: #b42318;
}
</style>
```

### `frontend/src/integrations/whatsapp/components/WhatsAppRail.vue`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/integrations/whatsapp/components/WhatsAppRail.vue`
- Size bytes / Размер в байтах: `5260`
- Included characters / Включено символов: `5260`
- Truncated / Обрезано: `no`

```text
<script setup lang="ts">
import { useI18n } from '../../../platform/i18n'
import Icon from '../../../shared/ui/Icon.vue'
import type { WhatsappCapabilitiesResponse, WhatsappCapabilityStatus } from '../types/whatsapp'
import type { WhatsappMessageForm } from '../stores/whatsapp'

const props = defineProps<{
  whatsappCapabilities: WhatsappCapabilitiesResponse | null
  whatsappClosureCapabilities: WhatsappCapabilityStatus[]
  whatsappBlockedCapabilities: WhatsappCapabilityStatus[]
  whatsappProviderAccounts: number
  isWhatsappActionSubmitting: boolean
  openAccountDrawer: () => void
  ingestWhatsappWebMessageFixture: () => void
  whatsappMessageForm: WhatsappMessageForm
}>()

const { t } = useI18n()

function capabilityLabel(capability: string): string {
  const labels: Record<string, string> = {
    'runtime.fixture': 'Fixture Runtime',
    'sessions.manual_state': 'Session Metadata',
    'sessions.restore': 'Session Restore',
    'auth.qr_link_start': 'QR Link',
    'auth.pair_code_link_start': 'Pair-code Link',
    'sync.chats': 'Chat Sync',
    'sync.history': 'History Sync',
    'messages.read_projection': 'Message Projection Reads',
    'search.messages': 'Message Search',
    'search.media': 'Media Search',
    'media.upload_send': 'Media Upload',
    'media.download': 'Media Download',
    'messages.send_text': 'Send Text',
    'messages.reply': 'Reply',
    'messages.forward': 'Forward',
    'messages.edit': 'Edit',
    'messages.delete': 'Delete',
    'messages.react': 'React',
    'messages.unreact': 'Remove Reaction',
    'status.publish': 'Publish Status',
    'conversations.mark_read': 'Mark Read',
    'conversations.mark_unread': 'Mark Unread',
    'conversations.archive': 'Archive',
    'conversations.unarchive': 'Unarchive',
    'conversations.mute': 'Mute',
    'conversations.unmute': 'Unmute',
    'conversations.pin': 'Pin',
    'conversations.unpin': 'Unpin',
    'conversations.join_group': 'Join Group',
    'conversations.leave_group': 'Leave Group'
  }
  return labels[capability] ?? capability
}
</script>

<template>
  <aside class="stacked-rail whatsapp-rail">
    <section class="panel info-card">
      <h2>{{ t('Accounts') }}</h2>
      <div class="setup-summary-card">
        <span class="round-icon green">
          <Icon icon="tabler:brand-whatsapp" width="22" height="22" />
        </span>
        <div>
          <strong>{{ whatsappProviderAccounts }} {{ t('WhatsApp accounts') }}</strong>
          <p>
            {{ whatsappProviderAccounts
              ? t('Companion session records are available for fixture ingestion.')
              : t('No WhatsApp Web account record is configured yet.')
            }}
          </p>
        </div>
      </div>
      <div class="form-actions wide">
        <button type="button" :disabled="isWhatsappActionSubmitting" @click="openAccountDrawer">
          {{ t('Open Wizard') }}
        </button>
      </div>
    </section>

    <section class="panel info-card">
      <h2>{{ t('Runtime Guardrails') }}</h2>
      <div class="health-row">
        <span>{{ t('Mode') }}</span>
        <strong>{{ whatsappCapabilities?.runtime_mode ?? t('unknown') }}</strong>
      </div>
      <ul v-if="whatsappClosureCapabilities.length" class="detail-list">
        <li v-for="cap in whatsappClosureCapabilities" :key="cap.capability">
          {{ capabilityLabel(cap.capability) }}
          <em>{{ cap.status }}</em>
        </li>
      </ul>
      <template v-if="whatsappBlockedCapabilities.length">
        <div class="evidence-row">
          <strong>{{ t('Live Scope') }}</strong>
          <p>{{ whatsappBlockedCapabilities.map((c) => capabilityLabel(c.capability)).join(', ') }}</p>
        </div>
      </template>
      <template v-if="whatsappCapabilities?.unsupported_features?.length">
        <div class="evidence-row">
          <strong>{{ t('Unsupported') }}</strong>
          <p>{{ whatsappCapabilities.unsupported_features.map(capabilityLabel).join(', ') }}</p>
        </div>
      </template>
    </section>

    <section class="panel info-card">
      <h2>{{ t('Ingest Fixture Message') }}</h2>
      <form class="setup-form compact-form" @submit.prevent="ingestWhatsappWebMessageFixture">
        <label><span>{{ t('Account ID') }}</span><input v-model="whatsappMessageForm.account_id" autocomplete="off" /></label>
        <label><span>{{ t('Chat ID') }}</span><input v-model="whatsappMessageForm.provider_chat_id" autocomplete="off" /></label>
        <label><span>{{ t('Chat title') }}</span><input v-model="whatsappMessageForm.chat_title" autocomplete="off" /></label>
        <label><span>{{ t('Sender ID') }}</span><input v-model="whatsappMessageForm.sender_id" autocomplete="off" /></label>
        <label><span>{{ t('Sender') }}</span><input v-model="whatsappMessageForm.sender_display_name" autocomplete="off" /></label>
        <label class="wide"><span>{{ t('Text') }}</span><textarea v-model="whatsappMessageForm.text" rows="3"></textarea></label>
        <div class="form-actions wide">
          <button type="submit" :disabled="isWhatsappActionSubmitting || !whatsappMessageForm.text.trim()">
            {{ t('Ingest Fixture') }}
          </button>
        </div>
      </form>
    </section>
  </aside>
</template>
```

### `frontend/src/integrations/whatsapp/components/WhatsAppRuntimeAccountList.vue`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/integrations/whatsapp/components/WhatsAppRuntimeAccountList.vue`
- Size bytes / Размер в байтах: `1647`
- Included characters / Включено символов: `1646`
- Truncated / Обрезано: `no`

```text
<script setup lang="ts">
import { computed } from 'vue'
import { useI18n } from '../../../platform/i18n'
import type { WhatsappAccountSummary } from '../../../shared/communications/types/whatsapp'

const props = defineProps<{
	accounts: WhatsappAccountSummary[]
	selectedAccountId: string | null
	includeRemovedAccounts: boolean
}>()

const emit = defineEmits<{
	(event: 'update:includeRemovedAccounts', value: boolean): void
	(event: 'select-account', accountId: string): void
}>()

const { t } = useI18n()

const includeRemovedAccountsModel = computed({
	get: () => props.includeRemovedAccounts,
	set: (value: boolean) => emit('update:includeRemovedAccounts', value),
})
</script>

<template>
	<section class="panel runtime-card">
		<header>
			<h2>{{ t('Accounts') }}</h2>
			<span>{{ accounts.length }}</span>
		</header>
		<label class="runtime-field compact checkbox-field">
			<span>{{ t('Include removed') }}</span>
			<input v-model="includeRemovedAccountsModel" type="checkbox" />
		</label>
		<div v-if="accounts.length" class="account-list">
			<button
				v-for="account in accounts"
				:key="account.account_id"
				type="button"
				class="account-row"
				:data-selected="account.account_id === selectedAccountId"
				@click="emit('select-account', account.account_id)"
			>
				<strong>{{ account.display_name }}</strong>
				<span>{{ account.account_id }}</span>
				<small>
					{{ account.provider_shape ?? account.provider_kind }}
					· {{ account.lifecycle_state ?? 'unknown' }}
				</small>
			</button>
		</div>
		<p v-else class="empty-state">{{ t('No WhatsApp accounts configured yet.') }}</p>
	</section>
</template>
```

### `frontend/src/integrations/whatsapp/components/WhatsAppRuntimeAccountProvisioning.vue`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/integrations/whatsapp/components/WhatsAppRuntimeAccountProvisioning.vue`
- Size bytes / Размер в байтах: `4354`
- Included characters / Включено символов: `4354`
- Truncated / Обрезано: `no`

```text
<script setup lang="ts">
import { computed } from 'vue'
import { useI18n } from '../../../platform/i18n'
import Icon from '../../../shared/ui/Icon.vue'
import type {
	WhatsappCapabilitiesResponse,
	WhatsappProviderShape,
	WhatsappProviderShapeStatus,
	WhatsappWebProviderKind,
} from '../../../shared/communications/types/whatsapp'

const props = defineProps<{
	capabilities: WhatsappCapabilitiesResponse | null
	liveAccountProviderKind: WhatsappWebProviderKind
	liveAccountShape: WhatsappProviderShape
	liveAccountId: string
	liveAccountDisplayName: string
	liveAccountExternalId: string
	liveAccountDeviceName: string
	liveAccountLocalStatePath: string
	liveAccountSupportsDeviceFields: boolean
	selectedProviderShapeMeta: WhatsappProviderShapeStatus | null
	liveAccountSessionMode: string
	isSubmitting: boolean
}>()

const emit = defineEmits<{
	(event: 'update:liveAccountShape', value: WhatsappProviderShape): void
	(event: 'update:liveAccountId', value: string): void
	(event: 'update:liveAccountDisplayName', value: string): void
	(event: 'update:liveAccountExternalId', value: string): void
	(event: 'update:liveAccountDeviceName', value: string): void
	(event: 'update:liveAccountLocalStatePath', value: string): void
	(event: 'create-live-account'): void
}>()

const { t } = useI18n()

const liveAccountShapeModel = computed({
	get: () => props.liveAccountShape,
	set: (value: WhatsappProviderShape) => emit('update:liveAccountShape', value),
})
const liveAccountIdModel = computed({
	get: () => props.liveAccountId,
	set: (value: string) => emit('update:liveAccountId', value),
})
const liveAccountDisplayNameModel = computed({
	get: () => props.liveAccountDisplayName,
	set: (value: string) => emit('update:liveAccountDisplayName', value),
})
const liveAccountExternalIdModel = computed({
	get: () => props.liveAccountExternalId,
	set: (value: string) => emit('update:liveAccountExternalId', value),
})
const liveAccountDeviceNameModel = computed({
	get: () => props.liveAccountDeviceName,
	set: (value: string) => emit('update:liveAccountDeviceName', value),
})
const liveAccountLocalStatePathModel = computed({
	get: () => props.liveAccountLocalStatePath,
	set: (value: string) => emit('update:liveAccountLocalStatePath', value),
})
</script>

<template>
	<section class="panel runtime-card">
		<header>
			<h2>{{ t('Account Provisioning') }}</h2>
			<span>{{ liveAccountProviderKind }}</span>
		</header>
		<div class="provisioning-grid">
			<label class="runtime-field compact">
				<span>{{ t('Provider shape') }}</span>
				<select v-model="liveAccountShapeModel">
					<option
						v-for="shape in capabilities?.provider_shapes ?? []"
						:key="shape.provider_shape"
						:value="shape.provider_shape"
					>
						{{ shape.provider_shape }}
					</option>
				</select>
			</label>
			<label class="runtime-field compact">
				<span>{{ t('Account id') }}</span>
				<input v-model="liveAccountIdModel" autocomplete="off" />
			</label>
			<label class="runtime-field compact">
				<span>{{ t('Display name') }}</span>
				<input v-model="liveAccountDisplayNameModel" autocomplete="off" />
			</label>
			<label class="runtime-field compact">
				<span>{{ t('External account id') }}</span>
				<input v-model="liveAccountExternalIdModel" autocomplete="off" />
			</label>
			<label v-if="liveAccountSupportsDeviceFields" class="runtime-field compact">
				<span>{{ t('Device name') }}</span>
				<input v-model="liveAccountDeviceNameModel" autocomplete="off" />
			</label>
			<label class="runtime-field compact">
				<span>{{ t('Local state path') }}</span>
				<input v-model="liveAccountLocalStatePathModel" autocomplete="off" />
			</label>
		</div>
		<div class="evidence-row">
			<strong>{{ t('Provider posture') }}</strong>
			<p>{{ selectedProviderShapeMeta?.reason ?? t('No provider-shape metadata available') }}</p>
		</div>
		<dl class="runtime-details compact">
			<div><dt>{{ t('Runtime mode') }}</dt><dd>{{ liveAccountSessionMode }}</dd></div>
			<div><dt>{{ t('Capability status') }}</dt><dd>{{ selectedProviderShapeMeta?.status ?? '-' }}</dd></div>
		</dl>
		<div class="runtime-actions">
			<button type="button" :disabled="isSubmitting" @click="emit('create-live-account')">
				<Icon icon="tabler:user-plus" width="16" height="16" />{{ t('Create Live Account') }}
			</button>
		</div>
	</section>
</template>
```

### `frontend/src/integrations/whatsapp/components/WhatsAppRuntimeCapabilities.vue`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/integrations/whatsapp/components/WhatsAppRuntimeCapabilities.vue`
- Size bytes / Размер в байтах: `1080`
- Included characters / Включено символов: `1080`
- Truncated / Обрезано: `no`

```text
<script setup lang="ts">
import { useI18n } from '../../../platform/i18n'
import type { WhatsappCapabilitiesResponse } from '../../../shared/communications/types/whatsapp'

defineProps<{
	runtimeCapabilities: WhatsappCapabilitiesResponse | null
}>()

const { t } = useI18n()
</script>

<template>
	<section class="panel runtime-card">
		<header>
			<h2>{{ t('Capabilities') }}</h2>
			<span>{{ runtimeCapabilities?.version ?? '-' }}</span>
		</header>
		<div v-if="runtimeCapabilities?.provider_shapes?.length" class="shape-grid">
			<article v-for="shape in runtimeCapabilities.provider_shapes" :key="shape.provider_shape" class="shape-card">
				<strong>{{ shape.provider_shape }}</strong>
				<span>{{ shape.status }}</span>
				<small>{{ shape.reason }}</small>
			</article>
		</div>
		<ul v-if="runtimeCapabilities?.capabilities?.length" class="detail-list">
			<li v-for="capability in runtimeCapabilities.capabilities" :key="capability.capability">
				<span>{{ capability.capability }}</span>
				<em>{{ capability.status }}</em>
			</li>
		</ul>
	</section>
</template>
```

### `frontend/src/integrations/whatsapp/components/WhatsAppRuntimeCommandAudit.vue`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/integrations/whatsapp/components/WhatsAppRuntimeCommandAudit.vue`
- Size bytes / Размер в байтах: `2404`
- Included characters / Включено символов: `2404`
- Truncated / Обрезано: `no`

```text
<script setup lang="ts">
import { useI18n } from '../../../platform/i18n'
import Icon from '../../../shared/ui/Icon.vue'
import type { WhatsAppProviderCommand } from '../../../shared/communications/types/whatsapp'
import {
	canDeadLetterCommand,
	canRetryCommand,
	commandStatusTone,
	commandTimestamp,
	providerTargetLabel,
} from '../views/WhatsAppRuntimePanel.helpers'

defineProps<{
	providerCommands: WhatsAppProviderCommand[]
	isRuntimeBusy: boolean
}>()

const emit = defineEmits<{
	(event: 'retry', commandId: string): void
	(event: 'dead-letter', commandId: string): void
}>()

const { t } = useI18n()
</script>

<template>
	<section class="panel runtime-card">
		<header>
			<h2>{{ t('Command Audit') }}</h2>
			<span>{{ providerCommands.length }}</span>
		</header>
		<div v-if="providerCommands.length" class="command-list">
			<article
				v-for="command in providerCommands"
				:key="command.command_id"
				class="command-row"
				:data-tone="commandStatusTone(command)"
			>
				<div class="command-head">
					<strong>{{ command.command_kind }}</strong>
					<em>{{ command.status }}</em>
				</div>
				<p class="command-target">{{ providerTargetLabel(command) }}</p>
				<dl class="runtime-details compact">
					<div><dt>{{ t('Capability') }}</dt><dd>{{ command.capability_state }}</dd></div>
					<div><dt>{{ t('Reconciliation') }}</dt><dd>{{ command.reconciliation_status }}</dd></div>
					<div><dt>{{ t('Attempts') }}</dt><dd>{{ command.retry_count }} / {{ command.max_retries }}</dd></div>
					<div><dt>{{ t('Updated') }}</dt><dd>{{ commandTimestamp(command) }}</dd></div>
				</dl>
				<p v-if="command.last_error" class="command-error">{{ command.last_error }}</p>
				<div class="runtime-actions compact">
					<button
						type="button"
						:disabled="isRuntimeBusy || !canRetryCommand(command)"
						@click="emit('retry', command.command_id)"
					>
						<Icon icon="tabler:reload" width="16" height="16" />{{ t('Retry') }}
					</button>
					<button
						type="button"
						:disabled="isRuntimeBusy || !canDeadLetterCommand(command)"
						@click="emit('dead-letter', command.command_id)"
					>
						<Icon icon="tabler:archive-off" width="16" height="16" />{{ t('Dead-letter') }}
					</button>
				</div>
			</article>
		</div>
		<p v-else class="empty-state">{{ t('No WhatsApp provider commands recorded for this account yet.') }}</p>
	</section>
</template>
```

### `frontend/src/integrations/whatsapp/components/WhatsAppRuntimeControl.vue`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/integrations/whatsapp/components/WhatsAppRuntimeControl.vue`
- Size bytes / Размер в байтах: `4956`
- Included characters / Включено символов: `4952`
- Truncated / Обрезано: `no`

```text
<script setup lang="ts">
import { useI18n } from '../../../platform/i18n'
import Icon from '../../../shared/ui/Icon.vue'
import type {
	WhatsAppRuntimeHealth,
	WhatsAppRuntimeStatus,
	WhatsAppWebCompanionManifest,
	WhatsappAccountSummary,
	WhatsappCapabilitiesResponse,
} from '../../../shared/communications/types/whatsapp'
import {
	runtimeHealthCheckDetail,
	runtimeHealthCheckStatus,
	snapshotTimestamp,
} from '../views/WhatsAppRuntimePanel.helpers'

type RuntimeAction = 'start' | 'stop' | 'revoke' | 'relink' | 'rotate' | 'remove'

defineProps<{
	selectedAccountId: string | null
	selectedAccountSummary: WhatsappAccountSummary | null
	runtimeStatus: WhatsAppRuntimeStatus | null
	runtimeCapabilities: WhatsappCapabilitiesResponse | null
	runtimeHealth: WhatsAppRuntimeHealth | null
	runtimeHealthChecks: Array<[string, unknown]>
	companionOpenManifest: WhatsAppWebCompanionManifest | null
	canOpenWebCompanion: boolean
	isRuntimeBusy: boolean
}>()

const emit = defineEmits<{
	(event: 'open-companion'): void
	(event: 'set-runtime-state', action: RuntimeAction): void
}>()

const { t } = useI18n()
</script>

<template>
	<section class="panel runtime-card">
		<header>
			<h2>{{ t('Runtime Control') }}</h2>
			<span>{{ selectedAccountId ?? '-' }}</span>
		</header>
		<div v-if="selectedAccountSummary" class="evidence-row">
			<strong>{{ selectedAccountSummary.display_name }}</strong>
			<p>
				{{ selectedAccountSummary.provider_shape ?? selectedAccountSummary.provider_kind }}
				· {{ selectedAccountSummary.runtime ?? 'unknown' }}
				· {{ selectedAccountSummary.lifecycle_state ?? 'unknown' }}
			</p>
		</div>
		<div class="runtime-actions">
			<button type="button" :disabled="isRuntimeBusy || !selectedAccountId || !canOpenWebCompanion" @click="emit('open-companion')">
				<Icon icon="tabler:brand-whatsapp" width="16" height="16" />{{ t('Open Companion') }}
			</button>
			<button type="button" :disabled="isRuntimeBusy || !selectedAccountId" @click="emit('set-runtime-state', 'start')">
				<Icon icon="tabler:player-play" width="16" height="16" />{{ t('Start') }}
			</button>
			<button type="button" :disabled="isRuntimeBusy || !selectedAccountId" @click="emit('set-runtime-state', 'stop')">
				<Icon icon="tabler:player-stop" width="16" height="16" />{{ t('Stop') }}
			</button>
			<button type="button" :disabled="isRuntimeBusy || !selectedAccountId" @click="emit('set-runtime-state', 'revoke')">
				<Icon icon="tabler:shield-x" width="16" height="16" />{{ t('Revoke') }}
			</button>
			<button type="button" :disabled="isRuntimeBusy || !selectedAccountId" @click="emit('set-runtime-state', 'relink')">
				<Icon icon="tabler:link-plus" width="16" height="16" />{{ t('Relink') }}
			</button>
			<button type="button" :disabled="isRuntimeBusy || !selectedAccountId" @click="emit('set-runtime-state', 'rotate')">
				<Icon icon="tabler:rotate-2" width="16" height="16" />{{ t('Rotate') }}
			</button>
			<button type="button" :disabled="isRuntimeBusy || !selectedAccountId" @click="emit('set-runtime-state', 'remove')">
				<Icon icon="tabler:trash" width="16" height="16" />{{ t('Remove') }}
			</button>
		</div>
		<dl class="runtime-details">
			<div><dt>{{ t('Lifecycle') }}</dt><dd>{{ runtimeStatus?.status ?? '-' }}</dd></div>
			<div><dt>{{ t('Provider shape') }}</dt><dd>{{ runtimeStatus?.provider_shape ?? runtimeCapabilities?.account_scope?.provider_shape ?? '-' }}</dd></div>
			<div><dt>{{ t('Runtime kind') }}</dt><dd>{{ runtimeStatus?.runtime_kind ?? runtimeCapabilities?.runtime_mode ?? '-' }}</dd></div>
			<div><dt>{{ t('Restore') }}</dt><dd>{{ runtimeStatus?.session_restore_available ? t('available') : t('blocked') }}</dd></div>
			<div><dt>{{ t('Health') }}</dt><dd>{{ runtimeHealth?.healthy ? t('healthy') : runtimeHealth?.status ?? '-' }}</dd></div>
			<div><dt>{{ t('Last error') }}</dt><dd>{{ runtimeStatus?.last_error ?? '-' }}</dd></div>
		</dl>
		<div v-if="runtimeStatus?.runtime_blockers?.length" class="evidence-row">
			<strong>{{ t('Runtime blockers') }}</strong>
			<p>{{ runtimeStatus.runtime_blockers.join(', ') }}</p>
		</div>
		<div v-if="companionOpenManifest" class="evidence-row">
			<strong>{{ t('WebView companion') }}</strong>
			<p>
				{{ companionOpenManifest.window_label }}
				· {{ companionOpenManifest.event_extractor.relay_channel }}
				· {{ companionOpenManifest.event_extractor.runtime_bridge_dispatch }}
			</p>
		</div>
		<div v-if="runtimeHealthChecks.length" class="evidence-row">
			<strong>{{ t('Health diagnostics') }}</strong>
			<small>{{ snapshotTimestamp(runtimeHealth?.checked_at) }}</small>
			<ul class="detail-list">
				<li v-for="[checkName, checkValue] in runtimeHealthChecks" :key="checkName">
					<span>{{ checkName }}</span>
					<em>{{ runtimeHealthCheckStatus(checkValue) }}</em>
					<small v-if="runtimeHealthCheckDetail(checkValue)">{{ runtimeHealthCheckDetail(checkValue) }}</small>
				</li>
			</ul>
		</div>
	</section>
</template>
```

### `frontend/src/integrations/whatsapp/components/WhatsAppRuntimeLinking.vue`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/integrations/whatsapp/components/WhatsAppRuntimeLinking.vue`
- Size bytes / Размер в байтах: `2267`
- Included characters / Включено символов: `2265`
- Truncated / Обрезано: `no`

```text
<script setup lang="ts">
import { computed } from 'vue'
import { useI18n } from '../../../platform/i18n'
import Icon from '../../../shared/ui/Icon.vue'
import type {
	WhatsAppPairCodeSession,
	WhatsAppQrLinkSession,
	WhatsAppRuntimeStatus,
} from '../../../shared/communications/types/whatsapp'

const props = defineProps<{
	runtimeStatus: WhatsAppRuntimeStatus | null
	selectedAccountId: string | null
	isRuntimeBusy: boolean
	pairCodePhoneNumber: string
	activeQrSession: WhatsAppQrLinkSession | null
	activePairCodeSession: WhatsAppPairCodeSession | null
}>()

const emit = defineEmits<{
	(event: 'update:pairCodePhoneNumber', value: string): void
	(event: 'set-runtime-state', action: 'qr' | 'pair_code'): void
}>()

const { t } = useI18n()

const pairCodePhoneNumberModel = computed({
	get: () => props.pairCodePhoneNumber,
	set: (value: string) => emit('update:pairCodePhoneNumber', value),
})
</script>

<template>
	<section class="panel runtime-card">
		<header>
			<h2>{{ t('Linking') }}</h2>
			<span>{{ runtimeStatus?.status ?? '-' }}</span>
		</header>
		<div class="runtime-actions">
			<button type="button" :disabled="isRuntimeBusy || !selectedAccountId" @click="emit('set-runtime-state', 'qr')">
				<Icon icon="tabler:qrcode" width="16" height="16" />{{ t('Start QR Link') }}
			</button>
			<button type="button" :disabled="isRuntimeBusy || !selectedAccountId || !pairCodePhoneNumberModel.trim()" @click="emit('set-runtime-state', 'pair_code')">
				<Icon icon="tabler:device-mobile-message" width="16" height="16" />{{ t('Start Pair Code') }}
			</button>
		</div>
		<label class="runtime-field">
			<span>{{ t('Phone number') }}</span>
			<input v-model="pairCodePhoneNumberModel" autocomplete="off" placeholder="+34..." />
		</label>
		<div v-if="activeQrSession" class="evidence-row">
			<strong>{{ t('QR session') }}</strong>
			<p>{{ activeQrSession.status }} · {{ activeQrSession.setup_id }}</p>
			<div v-if="activeQrSession.qr_svg" class="qr-preview" v-html="activeQrSession.qr_svg"></div>
		</div>
		<div v-if="activePairCodeSession" class="evidence-row">
			<strong>{{ t('Pair code') }}</strong>
			<p>{{ activePairCodeSession.pair_code ?? t('blocked') }} · {{ activePairCodeSession.phone_number }}</p>
		</div>
	</section>
</template>
```

### `frontend/src/integrations/whatsapp/components/WhatsAppRuntimeSnapshots.vue`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/integrations/whatsapp/components/WhatsAppRuntimeSnapshots.vue`
- Size bytes / Размер в байтах: `6978`
- Included characters / Включено символов: `6971`
- Truncated / Обрезано: `no`

```text
<script setup lang="ts">
import { computed } from 'vue'
import { useI18n } from '../../../platform/i18n'
import Icon from '../../../shared/ui/Icon.vue'
import type {
	WhatsAppCallSyncItem,
	WhatsAppChatSyncItem,
	WhatsAppContactSyncItem,
	WhatsAppMediaSyncItem,
	WhatsAppMembersSyncItem,
	WhatsAppPresenceSyncItem,
	WhatsappWebMessage,
} from '../../../shared/communications/types/whatsapp'
import {
	callLabel,
	chatLabel,
	chatMeta,
	contactLabel,
	historyLabel,
	mediaLabel,
	memberLabel,
	presenceLabel,
	snapshotTimestamp,
	statusLabel,
	statusPreview,
} from '../views/WhatsAppRuntimePanel.helpers'

const props = defineProps<{
	selectedAccountId: string | null
	selectedSyncChatIdResolved: string | null
	chatItems: WhatsAppChatSyncItem[]
	historyItems: WhatsappWebMessage[]
	memberItems: WhatsAppMembersSyncItem[]
	statusItems: WhatsappWebMessage[]
	presenceItems: WhatsAppPresenceSyncItem[]
	callItems: WhatsAppCallSyncItem[]
	contactItems: WhatsAppContactSyncItem[]
	mediaItems: WhatsAppMediaSyncItem[]
	statusPublishText: string
	isRuntimeBusy: boolean
}>()

const emit = defineEmits<{
	(event: 'select-chat', providerChatId: string): void
	(event: 'update:statusPublishText', value: string): void
	(event: 'publish-status'): void
}>()

const { t } = useI18n()

const statusText = computed({
	get: () => props.statusPublishText,
	set: (value: string) => emit('update:statusPublishText', value),
})
</script>

<template>
	<section class="panel runtime-card">
		<header>
			<h2>{{ t('Projected Snapshots') }}</h2>
			<span>{{ selectedAccountId ?? '-' }}</span>
		</header>
		<div class="snapshot-grid">
			<section class="snapshot-card">
				<div class="snapshot-head">
					<strong>{{ t('Chats') }}</strong>
					<span>{{ chatItems.length }}</span>
				</div>
				<ul v-if="chatItems.length" class="detail-stack compact">
					<li
						v-for="item in chatItems"
						:key="item.provider_chat_id"
						:class="{ selected: item.provider_chat_id === selectedSyncChatIdResolved }"
					>
						<button
							type="button"
							class="snapshot-select"
							:disabled="isRuntimeBusy"
							@click="emit('select-chat', item.provider_chat_id)"
						>
							<strong>{{ chatLabel(item) }}</strong>
							<span>{{ chatMeta(item) }}</span>
						</button>
					</li>
				</ul>
				<p v-else class="empty-state compact">{{ t('No projected chats yet.') }}</p>
			</section>

			<section class="snapshot-card">
				<div class="snapshot-head">
					<strong>{{ t('History') }}</strong>
					<span>{{ historyItems.length }}</span>
				</div>
				<ul v-if="historyItems.length" class="detail-stack compact">
					<li v-for="item in historyItems" :key="item.message_id">
						<strong>{{ historyLabel(item) }}</strong>
						<span>{{ statusPreview(item) }} · {{ snapshotTimestamp(item.occurred_at ?? item.projected_at) }}</span>
					</li>
				</ul>
				<p v-else class="empty-state compact">{{ t('Select a synced chat to inspect recent history.') }}</p>
			</section>

			<section class="snapshot-card">
				<div class="snapshot-head">
					<strong>{{ t('Members') }}</strong>
					<span>{{ memberItems.length }}</span>
				</div>
				<ul v-if="memberItems.length" class="detail-stack compact">
					<li v-for="item in memberItems" :key="item.participant_id">
						<strong>{{ memberLabel(item) }}</strong>
						<span>{{ item.role }}<template v-if="item.status"> · {{ item.status }}</template></span>
					</li>
				</ul>
				<p v-else class="empty-state compact">{{ t('Select a synced chat to inspect roster members.') }}</p>
			</section>

			<section class="snapshot-card">
				<div class="snapshot-head">
					<strong>{{ t('Statuses') }}</strong>
					<span>{{ statusItems.length }}</span>
				</div>
				<label class="runtime-field compact">
					<span>{{ t('Publish text status') }}</span>
					<textarea
						v-model="statusText"
						rows="3"
						maxlength="700"
						:placeholder="t('Share a short status update')"
					/>
				</label>
				<div class="runtime-actions compact">
					<button
						type="button"
						:disabled="isRuntimeBusy || !selectedAccountId || !statusText.trim()"
						@click="emit('publish-status')"
					>
						<Icon icon="tabler:send" width="16" height="16" />{{ t('Publish Status') }}
					</button>
				</div>
				<ul v-if="statusItems.length" class="detail-stack compact">
					<li v-for="item in statusItems" :key="item.message_id">
						<strong>{{ statusLabel(item) }}</strong>
						<span>{{ statusPreview(item) }} · {{ snapshotTimestamp(item.occurred_at ?? item.projected_at) }}</span>
					</li>
				</ul>
				<p v-else class="empty-state compact">{{ t('No projected statuses yet.') }}</p>
			</section>

			<section class="snapshot-card">
				<div class="snapshot-head">
					<strong>{{ t('Presence') }}</strong>
					<span>{{ presenceItems.length }}</span>
				</div>
				<ul v-if="presenceItems.length" class="detail-stack compact">
					<li v-for="item in presenceItems" :key="item.identity_id">
						<strong>{{ presenceLabel(item) }}</strong>
						<span>{{ item.presence_state }} · {{ snapshotTimestamp(item.observed_at) }}</span>
					</li>
				</ul>
				<p v-else class="empty-state compact">{{ t('No projected presence for the selected synced chat yet.') }}</p>
			</section>

			<section class="snapshot-card">
				<div class="snapshot-head">
					<strong>{{ t('Calls') }}</strong>
					<span>{{ callItems.length }}</span>
				</div>
				<ul v-if="callItems.length" class="detail-stack compact">
					<li v-for="item in callItems" :key="item.call_id">
						<strong>{{ item.provider_chat_id }}</strong>
						<span>{{ callLabel(item) }} · {{ snapshotTimestamp(item.started_at ?? item.observed_at) }}</span>
					</li>
				</ul>
				<p v-else class="empty-state compact">{{ t('No projected calls for the selected synced chat yet.') }}</p>
			</section>

			<section class="snapshot-card">
				<div class="snapshot-head">
					<strong>{{ t('Contacts') }}</strong>
					<span>{{ contactItems.length }}</span>
				</div>
				<ul v-if="contactItems.length" class="detail-stack compact">
					<li v-for="item in contactItems" :key="item.identity_id">
						<strong>{{ contactLabel(item) }}</strong>
						<span>{{ item.identity_kind }} · {{ item.address ?? item.provider_identity_id }}</span>
					</li>
				</ul>
				<p v-else class="empty-state compact">{{ t('No projected contacts yet.') }}</p>
			</section>

			<section class="snapshot-card">
				<div class="snapshot-head">
					<strong>{{ t('Media') }}</strong>
					<span>{{ mediaItems.length }}</span>
				</div>
				<ul v-if="mediaItems.length" class="detail-stack compact">
					<li v-for="item in mediaItems" :key="item.attachment_id">
						<strong>{{ mediaLabel(item) }}</strong>
						<span>{{ item.content_type }} · {{ item.provider_chat_id ?? t('unknown') }}</span>
					</li>
				</ul>
				<p v-else class="empty-state compact">{{ t('No projected media for the selected synced chat yet.') }}</p>
			</section>
		</div>
	</section>
</template>
```

### `frontend/src/integrations/whatsapp/components/WhatsAppSessionList.vue`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/integrations/whatsapp/components/WhatsAppSessionList.vue`
- Size bytes / Размер в байтах: `4329`
- Included characters / Включено символов: `4329`
- Truncated / Обрезано: `no`

```text
<script setup lang="ts">
import { ref, computed } from 'vue'
import { useVirtualizer } from '@tanstack/vue-virtual'
import { useI18n } from '../../../platform/i18n'
import Icon from '../../../shared/ui/Icon.vue'
import type { WhatsappWebSession } from '../types/whatsapp'

const props = defineProps<{
  whatsappSessions: WhatsappWebSession[]
  selectedWhatsappSessionId: string
  isWhatsappLoading: boolean
}>()

const emit = defineEmits<{
  selectSession: [session: WhatsappWebSession]
}>()

const { t } = useI18n()

const searchQuery = ref('')

const filteredSessions = computed(() => {
  const q = searchQuery.value.trim().toLowerCase()
  if (!q) return props.whatsappSessions
  return props.whatsappSessions.filter((s) => {
    const searchable = [s.device_name, s.account_id, s.session_id, s.link_state]
      .join(' ')
      .toLowerCase()
    return searchable.includes(q)
  })
})

const parentRef = ref<HTMLDivElement | null>(null)

const virtualOptions = computed(() => ({
  count: filteredSessions.value.length,
  getScrollElement: () => parentRef.value,
  estimateSize: () => 60,
  overscan: 5
}))

const virtualizer = useVirtualizer(virtualOptions)

const virtualItems = computed(() => virtualizer.value.getVirtualItems())
const totalSize = computed(() => virtualizer.value.getTotalSize())

function sessionLinkStateClass(state: string): string {
  if (state === 'linked' || state === 'fixture') return 'link-state-ok'
  if (state === 'degraded') return 'link-state-warn'
  return 'link-state-err'
}
</script>

<template>
  <section class="panel conversation-list">
    <label class="local-search">
      <Icon icon="tabler:search" width="17" height="17" />
      <input
        v-model="searchQuery"
        :placeholder="t('Search WhatsApp sessions...')"
        autocomplete="off"
      />
    </label>

    <div ref="parentRef" class="session-list-scroll">
      <div v-if="isWhatsappLoading && whatsappSessions.length === 0" class="empty-panel">
        {{ t('Loading WhatsApp Web state...') }}
      </div>
      <div v-else-if="whatsappSessions.length === 0" class="empty-panel">
        {{ t('No WhatsApp Web sessions saved yet.') }}
      </div>
      <template v-else>
        <div :style="{ height: `${totalSize}px` }">
          <button
            v-for="vitem in virtualItems"
            :key="filteredSessions[vitem.index].session_id"
            type="button"
            :class="['session-row', { active: selectedWhatsappSessionId === filteredSessions[vitem.index].session_id }]"
            :style="{ transform: `translateY(${vitem.start}px)`, height: `${vitem.size}px` }"
            @click="emit('selectSession', filteredSessions[vitem.index])"
          >
            <span class="round-icon cyan">
              <Icon icon="tabler:brand-whatsapp" width="22" height="22" />
            </span>
            <div class="session-info">
              <div class="session-title">{{ filteredSessions[vitem.index].device_name }}</div>
              <div class="session-meta">
                <span :class="sessionLinkStateClass(filteredSessions[vitem.index].link_state)">{{ filteredSessions[vitem.index].link_state }}</span>
                <span>{{ filteredSessions[vitem.index].companion_runtime }}</span>
              </div>
            </div>
          </button>
        </div>
      </template>
    </div>
  </section>
</template>

<style scoped>
.session-list-scroll {
  flex: 1;
  overflow-y: auto;
}

.session-row {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  width: 100%;
  padding: 0.6rem 0.75rem;
  border: none;
  border-radius: var(--radius-sm);
  background: transparent;
  color: var(--text-primary);
  cursor: pointer;
  text-align: left;
  transition: background 0.15s ease;
}
.session-row:hover {
  background: var(--bg-hover);
}
.session-row.active {
  background: var(--bg-active, var(--accent-bg));
  font-weight: 500;
}
.session-info {
  flex: 1;
  min-width: 0;
}
.session-title {
  font-size: 0.875rem;
  font-weight: 500;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}
.session-meta {
  display: flex;
  gap: 0.4rem;
  font-size: 0.75rem;
  color: var(--text-secondary);
}
.link-state-ok { color: var(--success, #22c55e); }
.link-state-warn { color: var(--warning, #eab308); }
.link-state-err { color: var(--danger, #ef4444); }
</style>
```

### `frontend/src/integrations/whatsapp/components/WhatsAppStatusMessages.vue`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/integrations/whatsapp/components/WhatsAppStatusMessages.vue`
- Size bytes / Размер в байтах: `251`
- Included characters / Включено символов: `251`
- Truncated / Обрезано: `no`

```text
<script setup lang="ts">
defineProps<{
  actionMessage: string
  error: string
}>()
</script>

<template>
  <p v-if="actionMessage" class="setup-state success">{{ actionMessage }}</p>
  <p v-if="error" class="inline-error">{{ error }}</p>
</template>
```

### `frontend/src/integrations/whatsapp/views/WhatsAppRuntimePanel.css`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/integrations/whatsapp/views/WhatsAppRuntimePanel.css`
- Size bytes / Размер в байтах: `4539`
- Included characters / Включено символов: `4539`
- Truncated / Обрезано: `no`

```text
.whatsapp-runtime-panel {
  display: flex;
  flex-direction: column;
  height: 100%;
  overflow: auto;
}
.view-header,
.view-title-with-icon,
.runtime-card header,
.runtime-actions {
  display: flex;
  align-items: center;
  gap: 0.75rem;
}
.view-header {
  justify-content: space-between;
  padding: 0.75rem 1rem;
  border-bottom: 1px solid var(--hh-border, #d9e2ec);
}
.whatsapp-realtime-state {
  padding: 0.6rem 1rem;
  margin: 0;
}
.whatsapp-runtime-grid {
  display: grid;
  grid-template-columns: minmax(280px, 420px) minmax(320px, 1fr);
  gap: 1rem;
  padding: 1rem;
  min-height: 0;
}
.sidebar-stack {
  display: grid;
  gap: 1rem;
  align-content: start;
}
.runtime-stack {
  display: grid;
  gap: 1rem;
}
.runtime-card {
  padding: 1rem;
}
.runtime-card header {
  justify-content: space-between;
}
.runtime-actions {
  flex-wrap: wrap;
}
.runtime-actions button,
.primary-button {
  display: inline-flex;
  align-items: center;
  gap: 0.35rem;
}
.runtime-details {
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: 0.6rem 1rem;
  margin: 1rem 0;
}
.runtime-details.compact {
  margin-top: 0.75rem;
}
.runtime-details dd {
  margin: 0.15rem 0 0;
}
.provisioning-grid {
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: 0.75rem 1rem;
}
.runtime-field {
  display: grid;
  gap: 0.5rem;
  margin-top: 1rem;
}
.checkbox-field {
  grid-template-columns: 1fr auto;
  align-items: center;
}
.runtime-field.compact {
  margin-top: 0;
}
.runtime-field input {
  min-height: 2rem;
}
.runtime-field select {
  min-height: 2rem;
}
.evidence-row {
  display: grid;
  gap: 0.35rem;
  margin-top: 1rem;
}
.evidence-row p {
  margin: 0;
}
.account-list {
  display: grid;
  gap: 0.5rem;
  margin-top: 1rem;
}
.account-row {
  display: grid;
  gap: 0.15rem;
  text-align: left;
  padding: 0.75rem;
  border: 1px solid var(--hh-border, #d9e2ec);
  border-radius: 6px;
  background: var(--hh-bg-primary, #fff);
}
.account-row[data-selected='true'] {
  border-color: var(--hh-accent, #22c55e);
  box-shadow: inset 0 0 0 1px var(--hh-accent, #22c55e);
}
.empty-state {
  margin: 1rem 0 0;
}
.shape-grid {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(180px, 1fr));
  gap: 0.75rem;
  margin-bottom: 1rem;
}
.shape-card {
  display: grid;
  gap: 0.25rem;
  padding: 0.75rem;
  border: 1px solid var(--hh-border, #d9e2ec);
  border-radius: 6px;
}
.detail-list {
  display: grid;
  gap: 0.5rem;
  margin: 0.5rem 0 0;
  padding: 0;
  list-style: none;
}
.detail-list li {
  display: grid;
  gap: 0.15rem;
  padding: 0.5rem 0.625rem;
  border: 1px solid var(--hh-border, #d9e2ec);
  border-radius: 6px;
  background: var(--hh-bg-primary, #fff);
}
.detail-list em {
  font-style: normal;
}
.detail-list small {
  color: var(--hh-text-secondary, #52606d);
}
.command-list {
  display: grid;
  gap: 0.75rem;
}
.snapshot-grid {
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: 0.75rem;
}
.snapshot-card {
  display: grid;
  gap: 0.5rem;
  padding: 0.75rem;
  border: 1px solid var(--hh-border, #d9e2ec);
  border-radius: 6px;
  background: var(--hh-bg-primary, #fff);
}
.snapshot-head {
  display: flex;
  justify-content: space-between;
  gap: 0.75rem;
  align-items: baseline;
}
.snapshot-select {
  display: grid;
  gap: 0.2rem;
  width: 100%;
  padding: 0;
  border: 0;
  background: transparent;
  color: inherit;
  text-align: left;
  font: inherit;
  cursor: pointer;
}
.detail-stack.compact li.selected {
  border-color: var(--hh-accent, #22c55e);
  box-shadow: inset 0 0 0 1px var(--hh-accent, #22c55e);
}
.command-row {
  display: grid;
  gap: 0.5rem;
  padding: 0.75rem;
  border: 1px solid var(--hh-border, #d9e2ec);
  border-radius: 6px;
  background: var(--hh-bg-primary, #fff);
}
.command-row[data-tone='available'] {
  border-color: color-mix(in srgb, var(--hh-accent, #22c55e) 35%, var(--hh-border, #d9e2ec));
}
.command-row[data-tone='blocked'] {
  border-color: color-mix(in srgb, #ef4444 35%, var(--hh-border, #d9e2ec));
}
.command-head {
  display: flex;
  justify-content: space-between;
  gap: 0.75rem;
  align-items: baseline;
}
.command-head em {
  font-style: normal;
}
.command-target,
.command-error {
  margin: 0;
}
.command-target {
  color: var(--hh-text-secondary, #52606d);
}
.command-error {
  color: #b42318;
}
.qr-preview {
  max-width: 14rem;
  padding-top: 0.5rem;
}

@media (max-width: 1024px) {
  .whatsapp-runtime-grid,
  .provisioning-grid,
  .runtime-details,
  .snapshot-grid {
    grid-template-columns: 1fr;
  }
}
```

### `frontend/src/integrations/whatsapp/views/WhatsAppRuntimePanel.vue`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/integrations/whatsapp/views/WhatsAppRuntimePanel.vue`
- Size bytes / Размер в байтах: `27713`
- Included characters / Включено символов: `12000`
- Truncated / Обрезано: `yes`

```text
<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import { useI18n } from '../../../platform/i18n'
import Icon from '../../../shared/ui/Icon.vue'
import { useRealtimeStatusStore } from '../../../shared/stores/realtimeStatus'
import type {
  WhatsAppWebCompanionManifest,
  WhatsappAccountSummary,
  WhatsappProviderShape,
  WhatsappWebProviderKind,
  WhatsappWebSession,
} from '../../../shared/communications/types/whatsapp'
import WhatsAppSessionList from '../components/WhatsAppSessionList.vue'
import WhatsAppRail from '../components/WhatsAppRail.vue'
import WhatsAppStatusMessages from '../components/WhatsAppStatusMessages.vue'
import WhatsAppRuntimeAccountList from '../components/WhatsAppRuntimeAccountList.vue'
import WhatsAppRuntimeAccountProvisioning from '../components/WhatsAppRuntimeAccountProvisioning.vue'
import WhatsAppRuntimeCapabilities from '../components/WhatsAppRuntimeCapabilities.vue'
import WhatsAppRuntimeCommandAudit from '../components/WhatsAppRuntimeCommandAudit.vue'
import WhatsAppRuntimeControl from '../components/WhatsAppRuntimeControl.vue'
import WhatsAppRuntimeLinking from '../components/WhatsAppRuntimeLinking.vue'
import WhatsAppRuntimeSnapshots from '../components/WhatsAppRuntimeSnapshots.vue'
import { useWhatsappStore } from '../stores/whatsapp'
import {
  ingestWhatsappWebMessageFixture,
  setupWhatsappWebFixture,
} from '../api/whatsapp'
import { openWhatsappWebCompanion } from '../api/whatsappCompanion'
import {
  useRelinkWhatsappRuntimeMutation,
  useRemoveWhatsappRuntimeMutation,
  useRotateWhatsappRuntimeMutation,
  useRevokeWhatsappRuntimeMutation,
  useDeadLetterWhatsappProviderCommandMutation,
  usePublishWhatsappStatusMutation,
  useRetryWhatsappProviderCommandMutation,
  useSetupWhatsappLiveAccountMutation,
  useStartWhatsappPairCodeLinkMutation,
  useStartWhatsappQrLinkMutation,
  useStartWhatsappRuntimeMutation,
  useStopWhatsappRuntimeMutation,
  useWhatsappAccountsQuery,
  useWhatsappAccountCapabilitiesQuery,
  useWhatsappCapabilitiesQuery,
  useWhatsappProviderCommandsQuery,
  useWhatsappRuntimeHealthQuery,
  useWhatsappRuntimeStatusQuery,
  useWhatsappSessionsQuery,
  useWhatsappSyncChatsQuery,
  useWhatsappSyncCallsQuery,
  useWhatsappSyncContactsQuery,
  useWhatsappSyncHistoryQuery,
  useWhatsappSyncMediaQuery,
  useWhatsappSyncMembersQuery,
  useWhatsappSyncPresenceQuery,
  useWhatsappSyncStatusesQuery,
} from '../queries/useWhatsappQuery'
const { t } = useI18n()
const realtimeStatus = useRealtimeStatusStore()
const store = useWhatsappStore()
const includeRemovedAccounts = ref(false)
const selectedAccountIdState = ref<string | null>(null)
const accountsQuery = useWhatsappAccountsQuery(includeRemovedAccounts)
const capabilitiesQuery = useWhatsappCapabilitiesQuery()
const accounts = computed(() => accountsQuery.data.value ?? [])
const capabilities = computed(() => capabilitiesQuery.data.value ?? null)
const selectedAccountId = computed(() =>
  selectedAccountIdState.value
  ?? store.selectedWhatsappSession?.account_id
  ?? accounts.value.find((account) => account.lifecycle_state !== 'removed')?.account_id
  ?? accounts.value[0]?.account_id
  ?? null
)
const sessionsQuery = useWhatsappSessionsQuery(selectedAccountId, 100)
const sessions = computed(() => sessionsQuery.data.value ?? [])
const selectedAccountSummary = computed<WhatsappAccountSummary | null>(() =>
  accounts.value.find((account) => account.account_id === selectedAccountId.value) ?? null
)
const accountCapabilitiesQuery = useWhatsappAccountCapabilitiesQuery(selectedAccountId)
const runtimeStatusQuery = useWhatsappRuntimeStatusQuery(selectedAccountId)
const runtimeHealthQuery = useWhatsappRuntimeHealthQuery(selectedAccountId)
const providerCommandsQuery = useWhatsappProviderCommandsQuery(selectedAccountId, 25)
const selectedSyncChatId = ref<string | null>(null)
const chatsSyncQuery = useWhatsappSyncChatsQuery(selectedAccountId, 8)
const chatItems = computed(() => chatsSyncQuery.data.value ?? [])
const selectedSyncChatIdResolved = computed(() =>
  selectedSyncChatId.value
  ?? chatItems.value[0]?.provider_chat_id
  ?? null
)
const historySyncQuery = useWhatsappSyncHistoryQuery(
  selectedAccountId,
  selectedSyncChatIdResolved,
  8
)
const membersSyncQuery = useWhatsappSyncMembersQuery(
  selectedAccountId,
  selectedSyncChatIdResolved,
  8
)
const statusesSyncQuery = useWhatsappSyncStatusesQuery(selectedAccountId, 8)
const presenceSyncQuery = useWhatsappSyncPresenceQuery(selectedAccountId, selectedSyncChatIdResolved, 8)
const callsSyncQuery = useWhatsappSyncCallsQuery(selectedAccountId, selectedSyncChatIdResolved, 8)
const contactsSyncQuery = useWhatsappSyncContactsQuery(selectedAccountId, 8)
const mediaSyncQuery = useWhatsappSyncMediaQuery(selectedAccountId, selectedSyncChatIdResolved, 8)
const startRuntimeMutation = useStartWhatsappRuntimeMutation()
const stopRuntimeMutation = useStopWhatsappRuntimeMutation()
const revokeRuntimeMutation = useRevokeWhatsappRuntimeMutation()
const relinkRuntimeMutation = useRelinkWhatsappRuntimeMutation()
const rotateRuntimeMutation = useRotateWhatsappRuntimeMutation()
const removeRuntimeMutation = useRemoveWhatsappRuntimeMutation()
const retryCommandMutation = useRetryWhatsappProviderCommandMutation()
const deadLetterCommandMutation = useDeadLetterWhatsappProviderCommandMutation()
const publishStatusMutation = usePublishWhatsappStatusMutation()
const setupLiveAccountMutation = useSetupWhatsappLiveAccountMutation()
const qrLinkMutation = useStartWhatsappQrLinkMutation()
const pairCodeMutation = useStartWhatsappPairCodeLinkMutation()
const pairCodePhoneNumber = ref('')
const isCompanionOpening = ref(false)
const companionOpenManifest = ref<WhatsAppWebCompanionManifest | null>(null)
const liveAccountShape = ref<WhatsappProviderShape>('whatsapp_web_companion')
const liveAccountId = ref('whatsapp-live-primary')
const liveAccountDisplayName = ref('WhatsApp Live')
const liveAccountExternalId = ref('whatsapp-live-primary')
const liveAccountDeviceName = ref('Макошь WhatsApp companion')
const liveAccountLocalStatePath = ref('docker/data/whatsapp/blocked/whatsapp-live-primary')

const runtimeCapabilities = computed(
  () => accountCapabilitiesQuery.data.value ?? capabilities.value
)
const runtimeStatus = computed(() => runtimeStatusQuery.data.value ?? null)
const runtimeHealth = computed(() => runtimeHealthQuery.data.value ?? null)
const runtimeHealthChecks = computed(() =>
  Object.entries(runtimeHealth.value?.checks ?? {})
)
const providerCommands = computed(() => providerCommandsQuery.data.value ?? [])
const historyItems = computed(() => historySyncQuery.data.value ?? [])
const memberItems = computed(() => membersSyncQuery.data.value ?? [])
const statusItems = computed(() => statusesSyncQuery.data.value ?? [])
const presenceItems = computed(() => presenceSyncQuery.data.value ?? [])
const callItems = computed(() => callsSyncQuery.data.value ?? [])
const contactItems = computed(() => contactsSyncQuery.data.value ?? [])
const mediaItems = computed(() => mediaSyncQuery.data.value ?? [])
const activeQrSession = computed(() => qrLinkMutation.data.value ?? null)
const activePairCodeSession = computed(() => pairCodeMutation.data.value ?? null)
const selectedProviderShapeMeta = computed(() =>
  capabilities.value?.provider_shapes.find((shape) => shape.provider_shape === liveAccountShape.value) ?? null
)
const liveAccountProviderKind = computed<WhatsappWebProviderKind>(() =>
  liveAccountShape.value === 'whatsapp_business_cloud'
    ? 'whatsapp_business_cloud'
    : 'whatsapp_web'
)
const liveAccountSessionMode = computed(() =>
  liveAccountShape.value === 'whatsapp_business_cloud' ? 'api_credentials' : 'device_session'
)
const liveAccountSupportsDeviceFields = computed(
  () => liveAccountShape.value !== 'whatsapp_business_cloud'
)
const selectedRuntimeProviderShape = computed(
  () =>
    runtimeStatus.value?.provider_shape
    ?? runtimeCapabilities.value?.account_scope?.provider_shape
    ?? selectedAccountSummary.value?.provider_shape
    ?? null
)
const canOpenWebCompanion = computed(
  () => selectedRuntimeProviderShape.value === 'whatsapp_web_companion'
)
const isRuntimeBusy = computed(() =>
  isCompanionOpening.value ||
  setupLiveAccountMutation.isPending.value ||
  startRuntimeMutation.isPending.value ||
  stopRuntimeMutation.isPending.value ||
  revokeRuntimeMutation.isPending.value ||
  relinkRuntimeMutation.isPending.value ||
  rotateRuntimeMutation.isPending.value ||
  removeRuntimeMutation.isPending.value ||
  retryCommandMutation.isPending.value ||
  deadLetterCommandMutation.isPending.value ||
  publishStatusMutation.isPending.value ||
  qrLinkMutation.isPending.value ||
  pairCodeMutation.isPending.value
)
const statusPublishText = ref('')

watch(liveAccountShape, (shape) => {
  if (shape === 'whatsapp_business_cloud') {
    liveAccountDeviceName.value = ''
    liveAccountLocalStatePath.value = `docker/data/whatsapp/business-cloud/${liveAccountId.value.trim() || 'whatsapp-business-cloud'}`
    return
  }
  if (!liveAccountDeviceName.value.trim()) {
    liveAccountDeviceName.value =
      shape === 'whatsapp_native_md'
        ? 'Макошь WhatsApp native runtime'
        : 'Макошь WhatsApp companion'
  }
  liveAccountLocalStatePath.value = `docker/data/whatsapp/blocked/${liveAccountId.value.trim() || 'whatsapp-live-primary'}`
}, { immediate: true })

watch(liveAccountId, (accountId) => {
  const trimmed = accountId.trim()
  if (!trimmed) return
  liveAccountExternalId.value = trimmed
  liveAccountLocalStatePath.value = liveAccountShape.value === 'whatsapp_business_cloud'
    ? `docker/data/whatsapp/business-cloud/${trimmed}`
    : `docker/data/whatsapp/blocked/${trimmed}`
})

watch(accounts, (nextAccounts) => {
  if (!nextAccounts.length) {
    selectedAccountIdState.value = null
    return
  }
  const current = selectedAccountIdState.value
  if (current && nextAccounts.some((account) => account.account_id === current)) {
    return
  }
  selectedAccountIdState.value =
    nextAccounts.find((account) => account.lifecycle_state !== 'removed')?.account_id
    ?? nextAccounts[0]?.account_id
    ?? null
}, { immediate: true })

watch(chatItems, (items) => {
  if (!items.length) {
    selectedSyncChatId.value = null
    return
  }
  if (selectedSyncChatId.value && items.some((item) => item.provider_chat_id === selectedSyncChatId.value)) {
    return
  }
  selectedSyncChatId.value = items[0]?.provider_chat_id ?? null
}, { immediate: true })

watch(
  [sessions, capabilities],
  ([nextSessions, nextCapabilities]) => {
    const selectedSessionId = nextSessions.some((session) => session.session_id === store.selectedWhatsappSessionId)
      ? store.selectedWhatsappSessionId
      : nextSessions[0]?.session_id ?? ''
    store.setWhatsappData({
      sessions: nextSessions,
      capabilities: nextCapabilities,
      selectedSessionId,
      error: '',
    })
  },
  { immediate: true }
)

watch(selectedAccountId, (accountId) => {
  if (!accountId) return
  store.whatsappMessageForm = {
    ...store.whatsappMessageForm,
    account_id: accountId,
  }
})

async function refreshRuntime() {
  await Promise.all([
    accountsQuery.refetch(),
    capabilitiesQuery.refetch(),
    accountCapabilitiesQuery.refetch(),
    sessionsQuery.refetch(),
    runtimeStatusQuery.refetch(),
    runtimeHealthQuery.refetch(),
    providerCommandsQuery.refetch(),
    chatsSyncQuery.refetch(),
    historySyncQuery.refetch(),
    membersSyncQuery.refetch(),
    statusesSyncQuery.refetch(),
    presenceSyncQuery.refetch(),
    callsSyncQuery.refetch(),
    contactsSyncQuery.refetch(),
    mediaSyncQuery.refetch(),
  ])
}

async function createLiveAccount() {
  if (store.isWhatsappActionSubmitting) return
  if (
    !liveAccountId.value.trim() ||
    !liveAccountDisplayName.value.trim() ||
    !liveAccountExternalId.value.trim()
  ) {
    store.setWhatsappError('Account id, display name and external account id are required')
    return
  }
  store.setWhats
```
_Source file truncated after 12000 characters. / Исходный файл обрезан после 12000 символов._

### `frontend/src/integrations/yandexTelemost/components/YandexTelemostSettingsPanel.vue`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/integrations/yandexTelemost/components/YandexTelemostSettingsPanel.vue`
- Size bytes / Размер в байтах: `11620`
- Included characters / Включено символов: `11618`
- Truncated / Обрезано: `no`

```text
<script setup lang="ts">
import { computed, ref } from 'vue'
import { useI18n } from '../../../platform/i18n'
import type { ProviderAccount } from '../../../shared/yandexTelemost/settingsBridge'
import {
  completeYandexTelemostRecording,
  createYandexTelemostConference,
  openYandexTelemostCompanion,
  startYandexTelemostRecording,
  stopYandexTelemostRecording,
} from '../api/yandexTelemost'
import {
  useSetupYandexTelemostAccountMutation,
  useYandexTelemostCapabilitiesQuery,
  useYandexTelemostRuntimeStatusQuery,
} from '../queries/useYandexTelemostRuntimeQuery'
import type { YandexTelemostConference, YandexTelemostRecordingSession } from '../types/yandexTelemost'

const props = defineProps<{
  selectedAccount?: ProviderAccount | null
}>()

const { t } = useI18n()

const setupForm = ref({
  account_id: '',
  display_name: '',
  external_account_id: '',
  oauth_token: '',
  oauth_token_ref: '',
  api_base_url: '',
})

const conferenceForm = ref({
  waiting_room_level: '',
  auto_summary: true,
})

const manualOpenForm = ref({
  join_url: '',
  conference_id: '',
})

const activeAction = ref<string | null>(null)
const actionMessage = ref('')
const errorMessage = ref('')
const lastConference = ref<YandexTelemostConference | null>(null)
const activeRecording = ref<YandexTelemostRecordingSession | null>(null)

const setupAccount = useSetupYandexTelemostAccountMutation()
const { data: capabilities } = useYandexTelemostCapabilitiesQuery()
const selectedTelemostAccountId = computed(() =>
  props.selectedAccount?.provider_kind === 'yandex_telemost_user' ? props.selectedAccount.account_id : null
)
const { data: runtimeStatus } = useYandexTelemostRuntimeStatusQuery(selectedTelemostAccountId)

const isSelected = computed(() => props.selectedAccount?.provider_kind === 'yandex_telemost_user')
const selectedLabel = computed(() => {
  const account = props.selectedAccount
  if (!account) return ''
  return account.display_name || account.external_account_id || account.account_id
})
const canUseSelected = computed(() => Boolean(selectedTelemostAccountId.value))
const safetySummary = computed(() => capabilities.value?.capabilities.find((item) => item.capability === 'telemost.speaker_timeline.webview_hints'))

async function handleSetup() {
  activeAction.value = 'setup'
  errorMessage.value = ''
  actionMessage.value = ''
  try {
    await setupAccount.mutateAsync({
      account_id: setupForm.value.account_id.trim(),
      display_name: setupForm.value.display_name.trim(),
      external_account_id: setupForm.value.external_account_id.trim(),
      oauth_token: valueOrUndefined(setupForm.value.oauth_token),
      oauth_token_ref: valueOrUndefined(setupForm.value.oauth_token_ref),
      api_base_url: valueOrUndefined(setupForm.value.api_base_url),
      metadata: { source: 'settings_panel' },
    })
    setupForm.value.oauth_token = ''
    actionMessage.value = t('Yandex Telemost account connected')
  } catch (error) {
    errorMessage.value = error instanceof Error ? error.message : 'Yandex Telemost setup failed'
  } finally {
    activeAction.value = null
  }
}

async function handleCreateConference() {
  if (!selectedTelemostAccountId.value) return
  activeAction.value = 'create'
  errorMessage.value = ''
  actionMessage.value = ''
  try {
    const response = await createYandexTelemostConference({
      account_id: selectedTelemostAccountId.value,
      waiting_room_level: valueOrUndefined(conferenceForm.value.waiting_room_level),
      is_auto_summarization_enabled: conferenceForm.value.auto_summary,
      metadata: { source: 'settings_panel' },
    })
    lastConference.value = response.conference
    manualOpenForm.value.join_url = response.conference.join_url
    manualOpenForm.value.conference_id = response.conference.id
    actionMessage.value = t('Yandex Telemost conference created')
  } catch (error) {
    errorMessage.value = error instanceof Error ? error.message : 'Yandex Telemost conference creation failed'
  } finally {
    activeAction.value = null
  }
}

async function handleOpenWebview() {
  if (!selectedTelemostAccountId.value) return
  activeAction.value = 'open'
  errorMessage.value = ''
  try {
    const manifest = await openYandexTelemostCompanion({
      account_id: selectedTelemostAccountId.value,
      conference_id: valueOrUndefined(manualOpenForm.value.conference_id),
      join_url: manualOpenForm.value.join_url.trim(),
      display_name: selectedLabel.value,
    })
    actionMessage.value = `${t('Yandex Telemost WebView opened')}: ${manifest.window_label}`
  } catch (error) {
    errorMessage.value = error instanceof Error ? error.message : 'Yandex Telemost WebView open failed'
  } finally {
    activeAction.value = null
  }
}

async function handleStartRecording() {
  if (!selectedTelemostAccountId.value) return
  activeAction.value = 'record'
  errorMessage.value = ''
  try {
    const session = await startYandexTelemostRecording({
      account_id: selectedTelemostAccountId.value,
      conference_id: valueOrUndefined(manualOpenForm.value.conference_id),
      join_url: manualOpenForm.value.join_url.trim(),
      consent_attested: true,
    })
    activeRecording.value = session
    actionMessage.value = `${t('Recording started')}: ${session.audio_path}`
  } catch (error) {
    errorMessage.value = error instanceof Error ? error.message : 'Yandex Telemost recording start failed'
  } finally {
    activeAction.value = null
  }
}

async function handleStopRecording() {
  const session = activeRecording.value
  if (!session) return
  activeAction.value = 'stop-recording'
  errorMessage.value = ''
  try {
    const receipt = await stopYandexTelemostRecording(session.recording_session_id)
    const bridge = await completeYandexTelemostRecording({
      account_id: session.account_id,
      conference_id: session.conference_id,
      join_url: session.join_url,
      recording_session_id: session.recording_session_id,
      output_dir: session.output_dir,
      audio_path: receipt.audio_path,
      speaker_jsonl_path: receipt.speaker_jsonl_path,
      speaker_txt_path: receipt.speaker_txt_path,
      started_at_epoch_ms: session.started_at_epoch_ms,
      stopped_at_epoch_ms: receipt.stopped_at_epoch_ms,
      consent_attested: session.consent_attested,
    })
    actionMessage.value = `${t('Recording stopped')}: ${bridge.bundle_id}`
    activeRecording.value = null
  } catch (error) {
    errorMessage.value = error instanceof Error ? error.message : 'Yandex Telemost recording stop failed'
  } finally {
    activeAction.value = null
  }
}

function valueOrUndefined(input: string): string | undefined {
  const trimmed = input.trim()
  return trimmed.length ? trimmed : undefined
}
</script>

<template>
  <section class="integration-section telemost-panel">
    <header class="panel-title-row">
      <div>
        <h3>{{ t('Yandex Telemost') }}</h3>
        <p class="integration-section-description">
          {{ t('Visible WebView, provider API, local MP3 recorder and speaker timeline hints.') }}
        </p>
      </div>
    </header>

    <div v-if="actionMessage" class="setup-state success">{{ actionMessage }}</div>
    <div v-if="errorMessage" class="inline-error">{{ errorMessage }}</div>

    <form class="integration-form" @submit.prevent="handleSetup">
      <h4>{{ t('Connect account') }}</h4>
      <label>{{ t('Account id') }}<input v-model="setupForm.account_id" /></label>
      <label>{{ t('Display name') }}<input v-model="setupForm.display_name" /></label>
      <label>{{ t('External account id') }}<input v-model="setupForm.external_account_id" /></label>
      <label>{{ t('OAuth token') }}<input v-model="setupForm.oauth_token" type="password" autocomplete="off" /></label>
      <label>{{ t('Existing token secret ref') }}<input v-model="setupForm.oauth_token_ref" /></label>
      <label>{{ t('API base URL') }}<input v-model="setupForm.api_base_url" placeholder="https://cloud-api.yandex.net/v1/telemost-api" /></label>
      <button type="submit" class="makosh-btn makosh-btn--primary" :disabled="activeAction==='setup'">
        {{ t('Connect Yandex Telemost') }}
      </button>
    </form>

    <div v-if="isSelected" class="integration-section nested">
      <h4>{{ t('Selected Telemost account') }}: {{ selectedLabel }}</h4>
      <p class="integration-section-description">
        {{ t('Runtime') }}: {{ runtimeStatus?.lifecycle_state || '-' }} · {{ t('Blockers') }}: {{ runtimeStatus?.blockers?.length ?? 0 }}
      </p>

      <div class="integration-form split">
        <label>{{ t('Waiting room level') }}<input v-model="conferenceForm.waiting_room_level" placeholder="PUBLIC" /></label>
        <label class="inline-check"><input v-model="conferenceForm.auto_summary" type="checkbox" /> {{ t('Request provider auto-summary') }}</label>
        <button type="button" class="makosh-btn makosh-btn--secondary" :disabled="!canUseSelected || activeAction==='create'" @click="handleCreateConference">
          {{ t('Create conference') }}
        </button>
      </div>

      <div class="integration-form split">
        <label>{{ t('Join URL') }}<input v-model="manualOpenForm.join_url" placeholder="https://telemost.yandex.ru/j/..." /></label>
        <label>{{ t('Conference id') }}<input v-model="manualOpenForm.conference_id" /></label>
        <button type="button" class="makosh-btn makosh-btn--secondary" :disabled="!manualOpenForm.join_url.trim() || activeAction==='open'" @click="handleOpenWebview">
          {{ t('Open in Макошь WebView') }}
        </button>
        <button type="button" class="makosh-btn makosh-btn--outline" :disabled="!manualOpenForm.join_url.trim() || Boolean(activeRecording) || activeAction==='record'" @click="handleStartRecording">
          {{ t('Start local MP3 recording') }}
        </button>
        <button type="button" class="makosh-btn makosh-btn--outline" :disabled="!activeRecording || activeAction==='stop-recording'" @click="handleStopRecording">
          {{ t('Stop recording') }}
        </button>
      </div>

      <div v-if="lastConference" class="telemost-result">
        <strong>{{ t('Last conference') }}</strong>
        <code>{{ lastConference.id }}</code>
        <span>{{ lastConference.join_url }}</span>
      </div>
      <div v-if="activeRecording" class="telemost-result">
        <strong>{{ t('Active recording') }}</strong>
        <code>{{ activeRecording.recording_session_id }}</code>
        <span>{{ activeRecording.audio_path }}</span>
        <span>{{ activeRecording.speaker_txt_path }}</span>
      </div>
    </div>

    <div v-if="safetySummary" class="telemost-safety">
      <strong>{{ t('Safety boundary') }}</strong>
      <span>{{ safetySummary.status }} · {{ safetySummary.source }}</span>
    </div>
  </section>
</template>

<style scoped>
.telemost-panel { display: grid; gap: 12px; }
.integration-form.split { margin-top: 12px; }
.integration-form input { width: 100%; padding: 8px; border: 1px solid var(--hh-border); border-radius: var(--hh-radius-sm); background: var(--hh-surface-deep); color: var(--hh-text-primary); }
.inline-check { display: flex !important; grid-template-columns: auto 1fr; align-items: center; gap: 8px; }
.inline-check input { width: auto; }
.nested { background: color-mix(in srgb, var(--hh-surface-deep) 88%, transparent); }
.telemost-result, .telemost-safety { display: grid; gap: 4px; margin-top: 10px; padding: 10px; border: 1px solid var(--hh-border); border-radius: var(--hh-radius-sm); font-size: 12px; color: var(--hh-text-secondary); }
.telemost-result code { color: var(--hh-text-primary); }
</style>
```

### `frontend/src/integrations/zoom/components/ZoomAuditEventsPanel.vue`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/integrations/zoom/components/ZoomAuditEventsPanel.vue`
- Size bytes / Размер в байтах: `4191`
- Included characters / Включено символов: `4181`
- Truncated / Обрезано: `no`

```text
<script setup lang="ts">
import { computed } from 'vue'
import { useI18n } from '../../../platform/i18n'
import type { ProviderAccount } from '../../../shared/zoom/settingsBridge'
import { useZoomAuditEventsQuery } from '../queries/useZoomRuntimeQuery'

const { t } = useI18n()

const props = defineProps<{
  selectedAccount?: ProviderAccount | null
}>()

const auditEventsQuery = useZoomAuditEventsQuery(
  computed(() => props.selectedAccount?.account_id ?? null),
  12
)
const auditEvents = computed(() => auditEventsQuery.data.value ?? [])

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
</script>

<template>
  <section class="integration-section zoom-audit-events">
    <header class="zoom-audit-events__header">
      <div>
        <h4>{{ t('Zoom audit events') }}</h4>
        <p>{{ t('Recent runtime and bridge events for the selected Zoom account.') }}</p>
      </div>
      <span class="zoom-audit-events__count">{{ auditEvents.length }}</span>
    </header>

    <div v-if="!selectedAccount?.account_id" class="zoom-audit-events__placeholder">
      {{ t('Select a Zoom account to inspect recent audit events.') }}
    </div>
    <div v-else-if="auditEventsQuery.isLoading.value" class="zoom-audit-events__placeholder">
      {{ t('Loading Zoom audit events...') }}
    </div>
    <div v-else-if="auditEvents.length === 0" class="zoom-audit-events__placeholder">
      {{ t('No Zoom audit events for this account yet.') }}
    </div>
    <div v-else class="zoom-audit-events__list">
      <article v-for="item in auditEvents" :key="item.event_id" class="zoom-audit-events__item">
        <header>
          <strong>{{ item.event_type }}</strong>
          <small>{{ formatDate(item.occurred_at) }}</small>
        </header>
        <dl class="zoom-audit-events__meta">
          <div><dt>{{ t('Subject') }}</dt><dd>{{ item.subject_kind ?? '—' }}</dd></div>
          <div><dt>{{ t('Entity') }}</dt><dd>{{ item.subject_entity_id ?? '—' }}</dd></div>
          <div><dt>{{ t('Position') }}</dt><dd>{{ item.position }}</dd></div>
          <div><dt>{{ t('Correlation') }}</dt><dd>{{ item.correlation_id ?? '—' }}</dd></div>
        </dl>
      </article>
    </div>
  </section>
</template>

<style scoped>
.zoom-audit-events { display: grid; gap: 12px; }
.zoom-audit-events__header {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: 12px;
}
.zoom-audit-events__header h4,
.zoom-audit-events__header p,
.zoom-audit-events__item header {
  margin: 0;
}
.zoom-audit-events__header p,
.zoom-audit-events__meta dt,
.zoom-audit-events__item small {
  color: var(--hh-text-muted);
  font-size: 11px;
}
.zoom-audit-events__count {
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
.zoom-audit-events__list,
.zoom-audit-events__item {
  display: grid;
  gap: 8px;
}
.zoom-audit-events__placeholder,
.zoom-audit-events__item {
  padding: 10px 12px;
  border-radius: var(--hh-radius-sm);
  border: 1px solid var(--hh-border);
  background: color-mix(in srgb, var(--hh-surface-deep) 88%, white 12%);
  font-size: 12px;
}
.zoom-audit-events__item header {
  display: flex;
  align-items: baseline;
  justify-content: space-between;
  gap: 8px;
}
.zoom-audit-events__item strong {
  display: block;
  font-size: 12px;
}
.zoom-audit-events__meta {
  display: grid;
  gap: 8px;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  margin: 0;
}
.zoom-audit-events__meta div {
  display: grid;
  gap: 2px;
}
.zoom-audit-events__meta dt,
.zoom-audit-events__meta dd {
  margin: 0;
  word-break: break-word;
}
@media (max-width: 900px) {
  .zoom-audit-events__meta {
    grid-template-columns: minmax(0, 1fr);
  }
}
</style>
```
