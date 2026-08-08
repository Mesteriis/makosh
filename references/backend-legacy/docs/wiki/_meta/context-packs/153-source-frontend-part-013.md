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

- Chunk ID / ID чанка: `153-source-frontend-part-013`
- Group / Группа: `frontend`
- Role / Роль: `source`
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

### `frontend/src/integrations/telegram/stores/telegramCommandAudit.ts`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/integrations/telegram/stores/telegramCommandAudit.ts`
- Size bytes / Размер в байтах: `15701`
- Included characters / Включено символов: `12000`
- Truncated / Обрезано: `yes`

```typescript
import type { TelegramProviderWriteCommand } from '../types/telegram'

export type TelegramCommandAuditTone = 'neutral' | 'progress' | 'success' | 'warning' | 'danger'

export type TelegramCommandAuditState = {
  label: string
  detail: string
  tone: TelegramCommandAuditTone
  is_dead_letter: boolean
}

function providerStateString(
  command: TelegramProviderWriteCommand,
  key: string
): string | null {
  const value = command.provider_state[key]
  return typeof value === 'string' && value.trim().length > 0 ? value : null
}

function providerStateBoolean(
  command: TelegramProviderWriteCommand,
  key: string
): boolean | null {
  const value = command.provider_state[key]
  return typeof value === 'boolean' ? value : null
}

function payloadString(
  command: TelegramProviderWriteCommand,
  key: string
): string | null {
  const value = command.payload[key]
  return typeof value === 'string' && value.trim().length > 0 ? value : null
}

function payloadNumber(
  command: TelegramProviderWriteCommand,
  key: string
): number | null {
  const value = command.payload[key]
  return typeof value === 'number' ? value : null
}

function providerStateNumber(
  command: TelegramProviderWriteCommand,
  key: string
): number | null {
  const value = command.provider_state[key]
  return typeof value === 'number' ? value : null
}

function folderId(command: TelegramProviderWriteCommand): number | null {
  return providerStateNumber(command, 'provider_folder_id') ?? payloadNumber(command, 'provider_folder_id')
}

function textLengthLabel(value: string): string {
  return `${value.length} chars`
}

function editMismatchDetail(command: TelegramProviderWriteCommand): string | null {
  if (command.command_kind !== 'edit' || command.reconciliation_status !== 'mismatch') return null

  const expectedBody = providerStateString(command, 'expected_body_text')
  const observedBody = providerStateString(command, 'observed_body_text')
  if (expectedBody && observedBody) {
    return `Provider mismatch · expected ${textLengthLabel(expectedBody)}, observed ${textLengthLabel(observedBody)}`
  }
  return 'Provider mismatch observed'
}

function reactionMismatchDetail(command: TelegramProviderWriteCommand): string | null {
  if (
    (command.command_kind !== 'react' && command.command_kind !== 'unreact') ||
    command.reconciliation_status !== 'mismatch'
  ) {
    return null
  }

  const reactionEmoji =
    providerStateString(command, 'reaction_emoji') ??
    payloadString(command, 'reaction_emoji')
  const observedIsChosen = providerStateBoolean(command, 'observed_is_chosen')
  if (!reactionEmoji) return 'Provider mismatch observed'
  if (observedIsChosen === true) {
    return `Provider mismatch · reaction ${reactionEmoji} is still present`
  }
  if (observedIsChosen === false) {
    return `Provider mismatch · reaction ${reactionEmoji} is still absent`
  }
  return `Provider mismatch · reaction ${reactionEmoji}`
}

function pinMismatchDetail(command: TelegramProviderWriteCommand): string | null {
  if (
    (command.command_kind !== 'pin' && command.command_kind !== 'unpin') ||
    command.reconciliation_status !== 'mismatch'
  ) {
    return null
  }

  const observedIsPinned = providerStateBoolean(command, 'observed_is_pinned')
  if (observedIsPinned === true) {
    return command.provider_message_id
      ? 'Provider mismatch · message is still pinned'
      : 'Provider mismatch · chat is still pinned'
  }
  if (observedIsPinned === false) {
    return command.provider_message_id
      ? 'Provider mismatch · message is still unpinned'
      : 'Provider mismatch · chat is still unpinned'
  }
  return 'Provider mismatch observed'
}

function chatLifecycleMismatchDetail(command: TelegramProviderWriteCommand): string | null {
  const pinMismatch = pinMismatchDetail(command)
  if (pinMismatch) return pinMismatch

  if (command.command_kind === 'mark_unread' && command.reconciliation_status === 'mismatch') {
    const observedIsMarkedUnread = providerStateBoolean(command, 'observed_is_marked_as_unread')
    if (observedIsMarkedUnread === true) return 'Provider mismatch · chat is still unread'
    if (observedIsMarkedUnread === false) return 'Provider mismatch · chat is still read'
    return 'Provider mismatch observed'
  }

  if (
    (command.command_kind === 'archive' || command.command_kind === 'unarchive') &&
    command.reconciliation_status === 'mismatch'
  ) {
    const observedIsArchived = providerStateBoolean(command, 'observed_is_archived')
    if (observedIsArchived === true) return 'Provider mismatch · chat is still archived'
    if (observedIsArchived === false) return 'Provider mismatch · chat is still unarchived'
    return 'Provider mismatch observed'
  }

  if (
    (command.command_kind === 'mute' || command.command_kind === 'unmute') &&
    command.reconciliation_status === 'mismatch'
  ) {
    const observedIsMuted = providerStateBoolean(command, 'observed_is_muted')
    if (observedIsMuted === true) return 'Provider mismatch · chat is still muted'
    if (observedIsMuted === false) return 'Provider mismatch · chat is still unmuted'
    return 'Provider mismatch observed'
  }

  return null
}

function messageLifecycleDetail(command: TelegramProviderWriteCommand): string | null {
  const mismatch = editMismatchDetail(command)
  if (mismatch) return mismatch
  const reactionMismatch = reactionMismatchDetail(command)
  if (reactionMismatch) return reactionMismatch
  const chatMismatch = chatLifecycleMismatchDetail(command)
  if (chatMismatch) return chatMismatch

  switch (command.command_kind) {
    case 'edit': {
      const observedBody = providerStateString(command, 'body_text')
      if (observedBody) return `Provider text observed · ${textLengthLabel(observedBody)}`
      const targetBody = payloadString(command, 'new_text')
      if (targetBody) return `Target text · ${textLengthLabel(targetBody)}`
      return null
    }
    case 'delete': {
      if (providerStateBoolean(command, 'is_deleted') === true) {
        return 'Provider delete observed'
      }
      const reasonClass = payloadString(command, 'reason_class')
      return reasonClass ? `Delete requested · ${reasonClass}` : 'Delete requested'
    }
    case 'restore_visibility': {
      if (command.status === 'completed') return 'Visibility restored locally'
      const reason = payloadString(command, 'reason')
      return reason ? `Visibility restore requested · ${reason}` : 'Visibility restore requested'
    }
    case 'mark_unread': {
      const isMarkedUnread = providerStateBoolean(command, 'is_marked_as_unread')
      if (isMarkedUnread === true) return 'Marked unread on provider'
      if (isMarkedUnread === false) return 'Marked read on provider'
      return 'Mark chat unread'
    }
    case 'pin':
    case 'unpin': {
      const isPinned = providerStateBoolean(command, 'is_pinned')
      if (isPinned === true) return 'Pinned on provider'
      if (isPinned === false) return 'Unpinned on provider'
      return command.command_kind === 'pin' ? 'Pin requested' : 'Unpin requested'
    }
    case 'archive':
    case 'unarchive': {
      const isArchived = providerStateBoolean(command, 'is_archived')
      if (isArchived === true) return 'Archived on provider'
      if (isArchived === false) return 'Restored from archive on provider'
      return command.command_kind === 'archive' ? 'Archive requested' : 'Unarchive requested'
    }
    case 'mute':
    case 'unmute': {
      const isMuted = providerStateBoolean(command, 'is_muted')
      if (isMuted === true) return 'Muted on provider'
      if (isMuted === false) return 'Unmuted on provider'
      return command.command_kind === 'mute' ? 'Mute requested' : 'Unmute requested'
    }
    case 'folder_add':
    case 'folder_remove': {
      const providerFolderId = folderId(command)
      if (providerFolderId !== null && command.status === 'completed') {
        return command.command_kind === 'folder_add'
          ? `Folder ${providerFolderId} observed on provider`
          : `Folder ${providerFolderId} removal observed on provider`
      }
      if (providerFolderId !== null) {
        return command.command_kind === 'folder_add'
          ? `Add to folder ${providerFolderId}`
          : `Remove from folder ${providerFolderId}`
      }
      return command.command_kind === 'folder_add' ? 'Add to folder' : 'Remove from folder'
    }
    case 'react':
    case 'unreact': {
      const reactionEmoji =
        providerStateString(command, 'reaction_emoji') ??
        payloadString(command, 'reaction_emoji')
      const isChosen = providerStateBoolean(command, 'is_chosen')
      if (reactionEmoji && isChosen === true) {
        return `Reaction ${reactionEmoji} present on provider`
      }
      if (reactionEmoji && isChosen === false) {
        return `Reaction ${reactionEmoji} absent on provider`
      }
      if (reactionEmoji) {
        return command.command_kind === 'react'
          ? `Add reaction ${reactionEmoji}`
          : `Remove reaction ${reactionEmoji}`
      }
      return command.command_kind === 'react' ? 'Add reaction' : 'Remove reaction'
    }
    default:
      return null
  }
}

function executingCommandDetail(command: TelegramProviderWriteCommand): string {
  const participantLifecycle = participantLifecycleDetail(command)
  if (participantLifecycle) return participantLifecycle

  const readProgress = markReadProgress(command)
  if (readProgress) return readProgress

  const lifecycleDetail = messageLifecycleDetail(command)
  if (lifecycleDetail) return lifecycleDetail

  const progressDetail = providerStateString(command, 'progress_detail')
  if (progressDetail) return progressDetail

  const uploadPhase = providerStateString(command, 'upload_phase')
  if (uploadPhase === 'dispatching_to_provider') return 'Uploading local media to Telegram'

  if (command.reconciliation_status === 'awaiting_provider') {
    return 'Awaiting provider-observed state'
  }

  return telegramCommandRetrySummary(command)
}

function retryBudget(command: TelegramProviderWriteCommand): { used: number; max: number } {
  return {
    used: Math.max(0, command.retry_count),
    max: Math.max(0, command.max_retries),
  }
}

export function telegramCommandRetrySummary(command: TelegramProviderWriteCommand): string {
  const { used, max } = retryBudget(command)
  if (max === 0) return 'No retry budget'
  return `${Math.min(used, max)}/${max} retries used`
}

export function telegramCommandSubject(command: TelegramProviderWriteCommand): string {
  if (command.command_kind === 'mark_read') {
    return command.provider_message_id
      ? `Read through ${command.provider_message_id}`
      : 'Mark chat read'
  }
  if (command.command_kind === 'mark_unread') {
    return 'Mark chat unread'
  }
  if (command.command_kind === 'edit') {
    return 'Edit message'
  }
  if (command.command_kind === 'delete') {
    return 'Delete message'
  }
  if (command.command_kind === 'restore_visibility') {
    return 'Restore message visibility'
  }
  if (command.command_kind === 'pin') {
    return command.provider_message_id ? 'Pin message' : 'Pin chat'
  }
  if (command.command_kind === 'unpin') {
    return command.provider_message_id ? 'Unpin message' : 'Unpin chat'
  }
  if (command.command_kind === 'archive') {
    return 'Archive chat'
  }
  if (command.command_kind === 'unarchive') {
    return 'Unarchive chat'
  }
  if (command.command_kind === 'mute') {
    return 'Mute chat'
  }
  if (command.command_kind === 'unmute') {
    return 'Unmute chat'
  }
  if (command.command_kind === 'folder_add') {
    const providerFolderId = folderId(command)
    return providerFolderId !== null
      ? `Add chat to folder ${providerFolderId}`
      : 'Add chat to folder'
  }
  if (command.command_kind === 'folder_remove') {
    const providerFolderId = folderId(command)
    return providerFolderId !== null
      ? `Remove chat from folder ${providerFolderId}`
      : 'Remove chat from fo
```
_Source file truncated after 12000 characters. / Исходный файл обрезан после 12000 символов._

### `frontend/src/integrations/telegram/stores/telegramRuntimeStatus.test.ts`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/integrations/telegram/stores/telegramRuntimeStatus.test.ts`
- Size bytes / Размер в байтах: `1848`
- Included characters / Включено символов: `1848`
- Truncated / Обрезано: `no`

```typescript
import { describe, expect, it } from 'vitest'

import { telegramRuntimeCommandTarget } from './telegramRuntimeStatus'

describe('telegramRuntimeCommandTarget', () => {
  it('formats mark-read command targets as read progress', () => {
    expect(
      telegramRuntimeCommandTarget({
        account_id: 'account-1',
        provider_kind: 'telegram_user',
        runtime_kind: 'fixture',
        status: 'running',
        fixture_runtime: true,
        tdjson_path: null,
        tdjson_runtime_available: false,
        tdjson_probe_error: null,
        telegram_api_id_configured: false,
        telegram_api_hash_configured: false,
        telegram_app_credentials_configured: false,
        live_send_available: false,
        runtime_blockers: [],
        last_error: null,
        last_command_status: 'completed',
        last_command_kind: 'mark_read',
        last_command_message_id: 'chat-1:777',
        updated_at: '2026-06-17T12:00:00Z',
      })
    ).toBe('Read through chat-1:777')
  })

  it('falls back to generic runtime command targets for other command kinds', () => {
    expect(
      telegramRuntimeCommandTarget({
        account_id: 'account-1',
        provider_kind: 'telegram_user',
        runtime_kind: 'fixture',
        status: 'running',
        fixture_runtime: true,
        tdjson_path: null,
        tdjson_runtime_available: false,
        tdjson_probe_error: null,
        telegram_api_id_configured: false,
        telegram_api_hash_configured: false,
        telegram_app_credentials_configured: false,
        live_send_available: false,
        runtime_blockers: [],
        last_error: null,
        last_command_status: 'completed',
        last_command_kind: 'pin',
        last_command_provider_chat_id: 'chat-1',
        updated_at: '2026-06-17T12:00:00Z',
      })
    ).toBe('chat-1')
  })
})
```

### `frontend/src/integrations/telegram/stores/telegramRuntimeStatus.ts`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/integrations/telegram/stores/telegramRuntimeStatus.ts`
- Size bytes / Размер в байтах: `535`
- Included characters / Включено символов: `535`
- Truncated / Обрезано: `no`

```typescript
import type { TelegramRuntimeStatus } from '../types/telegram'

export function telegramRuntimeCommandTarget(status: TelegramRuntimeStatus | null): string | null {
  if (!status?.last_command_status) return null

  if (status.last_command_kind === 'mark_read' && status.last_command_message_id) {
    return `Read through ${status.last_command_message_id}`
  }

  return status.last_command_message_id
    ?? status.last_command_telegram_chat_id
    ?? status.last_command_provider_chat_id
    ?? status.last_command_id
    ?? null
}
```

### `frontend/src/integrations/telegram/types/automation.ts`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/integrations/telegram/types/automation.ts`
- Size bytes / Размер в байтах: `1153`
- Included characters / Включено символов: `1153`
- Truncated / Обрезано: `no`

```typescript
export type TelegramAutomationTemplate = {
  template_id: string
  name: string
  body_template: string
  required_variables: string[]
  created_at: string
  updated_at: string
}

export type TelegramAutomationPolicy = {
  policy_id: string
  template_id: string
  name: string
  enabled: boolean
  account_id: string
  allowed_chat_ids: string[]
  trigger_kind: string
  max_sends_per_hour: number
  quiet_hours: unknown
  expires_at: string | null
  conditions: unknown
  created_at: string
  updated_at: string
}

export type TelegramAutomationTemplateListResponse = {
  items: TelegramAutomationTemplate[]
}

export type TelegramAutomationPolicyListResponse = {
  items: TelegramAutomationPolicy[]
}

export type TelegramSendDryRunRequest = {
  command_id: string
  policy_id: string
  provider_chat_id: string
  variables: Record<string, string>
  source_context?: Record<string, string>
}

export type TelegramSendDryRunResponse = {
  outbound_message_id: string
  policy_id: string
  template_id: string
  account_id: string
  provider_chat_id: string
  rendered_text: string
  rendered_preview_hash: string
  status: string
  event_id: string
}
```

### `frontend/src/integrations/telegram/types/telegram.ts`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/integrations/telegram/types/telegram.ts`
- Size bytes / Размер в байтах: `68`
- Included characters / Включено символов: `68`
- Truncated / Обрезано: `no`

```typescript
export type * from '../../../shared/communications/types/telegram'
```

### `frontend/src/integrations/telegram/types/telegramRealtime.ts`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/integrations/telegram/types/telegramRealtime.ts`
- Size bytes / Размер в байтах: `1483`
- Included characters / Включено символов: `1483`
- Truncated / Обрезано: `no`

```typescript
// ADR-0091 Telegram realtime event surface.
// Keep this separate from telegram.ts so the core domain type file stays below
// the repository line-count limit while realtime coverage grows.

export type TelegramRealtimeEventType =
  | 'telegram.sync.started'
  | 'telegram.sync.progress'
  | 'telegram.sync.completed'
  | 'telegram.sync.failed'
  | 'telegram.message.created'
  | 'telegram.message.updated'
  | 'telegram.message.edited'
  | 'telegram.message.deleted'
  | 'telegram.message.tombstoned'
  | 'telegram.message.visibility_restored'
  | 'telegram.reaction.changed'
  | 'telegram.chat.updated'
  | 'telegram.chat.pinned'
  | 'telegram.chat.archived'
  | 'telegram.chat.muted'
  | 'telegram.typing.changed'
  | 'telegram.topic.updated'
  | 'telegram.participant.updated'
  | 'telegram.media.download.started'
  | 'telegram.media.download.progress'
  | 'telegram.media.download.failed'
  | 'telegram.media.downloaded'
  | 'telegram.media.upload.started'
  | 'telegram.media.upload.progress'
  | 'telegram.media.upload.failed'
  | 'telegram.media.upload.completed'
  | 'telegram.command.status_changed'
  | 'telegram.command.reconciled'

export type TelegramRealtimeEvent = {
  event_type: TelegramRealtimeEventType
  event_id: string
  occurred_at: string
  subject: { id: string; kind: string }
  payload: Record<string, unknown>
}

export type TelegramRealtimeMessage =
  | { type: 'event'; data: TelegramRealtimeEvent }
  | { type: 'lagged'; data: { skipped: number } }
```

### `frontend/src/integrations/telegram/views/TelegramRuntimePanel.boundary.test.ts`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/integrations/telegram/views/TelegramRuntimePanel.boundary.test.ts`
- Size bytes / Размер в байтах: `1854`
- Included characters / Включено символов: `1854`
- Truncated / Обрезано: `no`

```typescript
import { describe, expect, it } from 'vitest'
import { readFileSync } from 'node:fs'

describe('TelegramRuntimePanel realtime boundary', () => {
	it('relies on the global realtime bootstrap instead of opening a panel-level telegram socket', () => {
		const source = readFileSync(new URL('./TelegramRuntimePanel.vue', import.meta.url), 'utf8')

		expect(source).not.toContain('createTelegramRealtimeConnection')
		expect(source).not.toContain('realtimeCleanup')
		expect(source).not.toContain('onMounted(() =>')
		expect(source).not.toContain('onUnmounted(() =>')
		expect(source).toContain('useRealtimeStatusStore()')
		expect(source).toContain('useTelegramAccountsQuery()')
		expect(source).toContain('useTelegramCapabilitiesQuery()')
		expect(source).toContain('useTelegramRuntimeStatusQuery(')
		expect(source).toContain('useStopTelegramRuntimeMutation()')
		expect(source).toContain('useStartTelegramRuntimeMutation()')
		expect(source).toContain('useRestartTelegramRuntimeMutation()')
		expect(source).toContain(':title="realtimeStatus.realtimeStatusDetail"')
		expect(source).toContain(':class="realtimeStatus.realtimeStatusTone"')
		expect(source).toContain("setTelegramRuntime('start')")
		expect(source).toContain("setTelegramRuntime('stop')")
		expect(source).toContain("setTelegramRuntime('restart')")
		expect(source).not.toContain('useTelegramMessagesQuery(')
		expect(source).not.toContain('useTelegramMessageSearchQuery(')
		expect(source).not.toContain('useTelegramMediaSearchQuery(')
		expect(source).not.toContain('useTelegramSendActions(')
		expect(source).not.toContain('Telegram' + 'Message' + 'Thread')
		expect(source).not.toContain('Telegram' + 'Chat' + 'List')
		expect(source).not.toContain('telegramChatGroupFilters(')
		expect(source).toContain('telegram-runtime-panel')
		expect(source).not.toContain('telegram-page')
	})
})
```

### `frontend/src/integrations/whatsapp/api/whatsapp.ts`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/integrations/whatsapp/api/whatsapp.ts`
- Size bytes / Размер в байтах: `15190`
- Included characters / Включено символов: `12000`
- Truncated / Обрезано: `yes`

```typescript
import { ApiClient } from '../../../platform/api/ApiClient'
import type {
  WhatsappAccountListResponse,
  WhatsAppChatSyncResponse,
  WhatsappCapabilitiesResponse,
  WhatsAppCallsSyncResponse,
  WhatsAppContactsSyncResponse,
  WhatsAppMediaSyncResponse,
  WhatsAppMembersSyncResponse,
  WhatsAppPresenceSyncResponse,
  WhatsAppStatusSyncResponse,
  WhatsappLiveAccountSetupRequest,
  WhatsAppProviderCommand,
  WhatsAppProviderCommandListResponse,
  WhatsAppPairCodeSession,
  WhatsAppQrLinkSession,
  WhatsAppRuntimeHealth,
  WhatsAppRuntimeRemoveResponse,
  WhatsAppRuntimeStatus,
  WhatsappWebSessionListResponse,
  WhatsappWebAccountSetupRequest,
  WhatsappWebAccountSetupResponse,
  WhatsappWebFixtureMessageRequest,
  WhatsappWebMessageIngestResponse,
  WhatsappWebSession,
} from '../types/whatsapp'

// --- Capabilities ---
export async function fetchWhatsappCapabilities(): Promise<WhatsappCapabilitiesResponse> {
  return ApiClient.instance.get<WhatsappCapabilitiesResponse>(
    '/api/v1/integrations/whatsapp/capabilities',
    'WhatsApp capabilities request failed'
  )
}

export async function fetchWhatsappAccountCapabilities(
  accountId: string
): Promise<WhatsappCapabilitiesResponse> {
  return ApiClient.instance.get<WhatsappCapabilitiesResponse>(
    `/api/v1/integrations/whatsapp/accounts/${encodeURIComponent(accountId)}/capabilities`,
    'WhatsApp account capabilities request failed'
  )
}

export async function fetchWhatsappAccounts(
  includeRemoved = false
): Promise<WhatsappAccountListResponse> {
  const params = new URLSearchParams()
  if (includeRemoved) {
    params.set('include_removed', 'true')
  }
  const suffix = params.toString() ? `?${params.toString()}` : ''
  return ApiClient.instance.get<WhatsappAccountListResponse>(
    `/api/v1/integrations/whatsapp/accounts${suffix}`,
    'WhatsApp accounts request failed'
  )
}

// --- Sessions ---
export async function fetchWhatsappWebSessions(
  accountId?: string,
  limit = 50
): Promise<WhatsappWebSessionListResponse> {
  const params = new URLSearchParams({ limit: String(Math.trunc(limit)) })
  if (accountId?.trim()) {
    params.set('account_id', accountId.trim())
  }
  return ApiClient.instance.get<WhatsappWebSessionListResponse>(
    `/api/v1/integrations/whatsapp/sessions?${params.toString()}`,
    'WhatsApp Web sessions request failed'
  )
}

export async function fetchWhatsappRuntimeStatus(accountId: string): Promise<WhatsAppRuntimeStatus> {
  const params = new URLSearchParams({ account_id: accountId.trim() })
  return ApiClient.instance.get<WhatsAppRuntimeStatus>(
    `/api/v1/integrations/whatsapp/runtime/status?${params.toString()}`,
    'WhatsApp runtime status request failed'
  )
}

export async function fetchWhatsappRuntimeHealth(accountId: string): Promise<WhatsAppRuntimeHealth> {
  const params = new URLSearchParams({ account_id: accountId.trim() })
  return ApiClient.instance.get<WhatsAppRuntimeHealth>(
    `/api/v1/integrations/whatsapp/runtime/health?${params.toString()}`,
    'WhatsApp runtime health request failed'
  )
}

export async function startWhatsappRuntime(request: {
  account_id: string
}): Promise<WhatsAppRuntimeStatus> {
  return ApiClient.instance.post<WhatsAppRuntimeStatus>(
    '/api/v1/integrations/whatsapp/runtime/start',
    request,
    'WhatsApp runtime start failed'
  )
}

export async function stopWhatsappRuntime(request: {
  account_id: string
}): Promise<WhatsAppRuntimeStatus> {
  return ApiClient.instance.post<WhatsAppRuntimeStatus>(
    '/api/v1/integrations/whatsapp/runtime/stop',
    request,
    'WhatsApp runtime stop failed'
  )
}

export async function revokeWhatsappRuntime(request: {
  account_id: string
}): Promise<WhatsAppRuntimeStatus> {
  return ApiClient.instance.post<WhatsAppRuntimeStatus>(
    '/api/v1/integrations/whatsapp/runtime/revoke',
    request,
    'WhatsApp runtime revoke failed'
  )
}

export async function relinkWhatsappRuntime(request: {
  account_id: string
}): Promise<WhatsAppRuntimeStatus> {
  return ApiClient.instance.post<WhatsAppRuntimeStatus>(
    '/api/v1/integrations/whatsapp/runtime/relink',
    request,
    'WhatsApp runtime relink failed'
  )
}

export async function rotateWhatsappRuntime(request: {
  account_id: string
}): Promise<WhatsAppRuntimeStatus> {
  return ApiClient.instance.post<WhatsAppRuntimeStatus>(
    '/api/v1/integrations/whatsapp/runtime/rotate',
    request,
    'WhatsApp runtime rotate failed'
  )
}

export async function removeWhatsappRuntime(request: {
  account_id: string
}): Promise<WhatsAppRuntimeRemoveResponse> {
  return ApiClient.instance.post<WhatsAppRuntimeRemoveResponse>(
    '/api/v1/integrations/whatsapp/runtime/remove',
    request,
    'WhatsApp runtime remove failed'
  )
}

export async function startWhatsappQrLink(request: {
  account_id: string
}): Promise<WhatsAppQrLinkSession> {
  return ApiClient.instance.post<WhatsAppQrLinkSession>(
    '/api/v1/integrations/whatsapp/login/qr/start',
    request,
    'WhatsApp QR link start failed'
  )
}

export async function startWhatsappPairCodeLink(request: {
  account_id: string
  phone_number: string
}): Promise<WhatsAppPairCodeSession> {
  return ApiClient.instance.post<WhatsAppPairCodeSession>(
    '/api/v1/integrations/whatsapp/login/pair-code/start',
    request,
    'WhatsApp pair-code link start failed'
  )
}

export async function fetchWhatsappProviderCommands(params: {
  account_id: string
  provider_chat_id?: string
  provider_message_id?: string
  command_kinds?: string[]
  limit?: number
}): Promise<WhatsAppProviderCommandListResponse> {
  const query = new URLSearchParams({ account_id: params.account_id.trim() })
  if (params.provider_chat_id?.trim()) {
    query.set('provider_chat_id', params.provider_chat_id.trim())
  }
  if (params.provider_message_id?.trim()) {
    query.set('provider_message_id', params.provider_message_id.trim())
  }
  if (params.command_kinds?.length) {
    query.set('command_kinds', params.command_kinds.join(','))
  }
  if (typeof params.limit === 'number') {
    query.set('limit', String(Math.trunc(params.limit)))
  }
  return ApiClient.instance.get<WhatsAppProviderCommandListResponse>(
    `/api/v1/integrations/whatsapp/commands?${query.toString()}`,
    'WhatsApp provider commands request failed'
  )
}

export async function fetchWhatsappSyncPresence(params: {
  account_id: string
  provider_chat_id?: string
  limit?: number
}): Promise<WhatsAppPresenceSyncResponse> {
  return ApiClient.instance.post<WhatsAppPresenceSyncResponse>(
    '/api/v1/integrations/whatsapp/provider-sync/presence',
    params,
    'WhatsApp presence sync request failed'
  )
}

export async function fetchWhatsappSyncChats(params: {
  account_id: string
  limit?: number
}): Promise<WhatsAppChatSyncResponse> {
  return ApiClient.instance.post<WhatsAppChatSyncResponse>(
    '/api/v1/integrations/whatsapp/provider-sync/chats',
    params,
    'WhatsApp chats sync request failed'
  )
}

export async function fetchWhatsappSyncHistory(params: {
  account_id: string
  provider_chat_id: string
  limit?: number
}): Promise<WhatsAppStatusSyncResponse> {
  return ApiClient.instance.post<WhatsAppStatusSyncResponse>(
    '/api/v1/integrations/whatsapp/provider-sync/history',
    params,
    'WhatsApp history sync request failed'
  )
}

export async function fetchWhatsappSyncMembers(params: {
  account_id: string
  provider_chat_id: string
  limit?: number
}): Promise<WhatsAppMembersSyncResponse> {
  return ApiClient.instance.post<WhatsAppMembersSyncResponse>(
    `/api/v1/integrations/whatsapp/provider-sync/conversations/${encodeURIComponent(params.provider_chat_id)}/members`,
    { account_id: params.account_id, limit: params.limit },
    'WhatsApp members sync request failed'
  )
}

export async function fetchWhatsappSyncCalls(params: {
  account_id: string
  provider_chat_id?: string
  limit?: number
}): Promise<WhatsAppCallsSyncResponse> {
  return ApiClient.instance.post<WhatsAppCallsSyncResponse>(
    '/api/v1/integrations/whatsapp/provider-sync/calls',
    params,
    'WhatsApp calls sync request failed'
  )
}

export async function fetchWhatsappSyncContacts(params: {
  account_id: string
  limit?: number
}): Promise<WhatsAppContactsSyncResponse> {
  return ApiClient.instance.post<WhatsAppContactsSyncResponse>(
    '/api/v1/integrations/whatsapp/provider-sync/contacts',
    params,
    'WhatsApp contacts sync request failed'
  )
}

export async function fetchWhatsappSyncStatuses(params: {
  account_id: string
  limit?: number
}): Promise<WhatsAppStatusSyncResponse> {
  return ApiClient.instance.post<WhatsAppStatusSyncResponse>(
    '/api/v1/integrations/whatsapp/provider-sync/statuses',
    params,
    'WhatsApp status sync request failed'
  )
}

export async function fetchWhatsappSyncMedia(params: {
  account_id: string
  provider_chat_id?: string
  content_type?: string
  limit?: number
}): Promise<WhatsAppMediaSyncResponse> {
  return ApiClient.instance.post<WhatsAppMediaSyncResponse>(
    '/api/v1/integrations/whatsapp/provider-sync/media',
    params,
    'WhatsApp media sync request failed'
  )
}

export async function publishWhatsappStatus(request: {
  account_id: string
  idempotency_key: string
  text: string
  command_id?: string
}): Promise<WhatsAppProviderCommand> {
  return ApiClient.instance.post<WhatsAppProviderCommand>(
    '/api/v1/integrations/whatsapp/provider-commands/statuses/publish',
    request,
    'WhatsApp status publish failed'
  )
}

export async function retryWhatsappProviderCommand(
  commandId: string
): Promise<WhatsAppProviderCommand> {
  return ApiClient.instance.post<WhatsAppProviderCommand>(
    `/api/v1/integrations/whatsapp/commands/${encodeURIComponent(commandId)}/retry`,
    {},
    'WhatsApp provider command retry failed'
  )
}

export async function deadLetterWhatsappProviderCommand(params: {
  command_id: string
  reason: string
}): Promise<WhatsAppProviderCommand> {
  return ApiClient.instance.post<WhatsAppProviderCommand>(
    `/api/v1/integrations/whatsapp/commands/${encodeURIComponent(params.command_id)}/dead-letter`,
    { reason: params.reason },
    'WhatsApp provider command dead-letter failed'
  )
}

// --- Account setup ---
export async function setupWhatsappWebFixtureAccount(
  request: WhatsappWebAccountSetupRequest
): Promise<WhatsappWebAccountSetupResponse> {
  return ApiClient.instance.post<WhatsappWebAccountSetupResponse>(
    '/api/v1/integrations/whatsapp/fixtures/accounts',
    request,
    'WhatsApp Web account setup request failed'
  )
}

export async function setupWhatsappLiveAccount(
  request: WhatsappLiveAccountSetupRequest
): Promise<WhatsappWebAccountSetupResponse> {
  return ApiClient.instance.post<WhatsappWebAccountSetupResponse>(
    '/api/v1/integrations/whatsapp/accounts',
    request,
    'WhatsApp live account setup request failed'
  )
}

// --- Fixture message ingest ---
export async function ingestWhatsappWebFixtureMessage(
  request: WhatsappWebFixtureMessageRequest
): Promise<WhatsappWebMessageIngestResponse> {
  return ApiClient.instance.post<WhatsappWebMessageIngestResponse>(
    '/api/v1/integrations/whatsapp/fixtures/messages',
    request,
    'WhatsApp Web fixture message request failed'
  )
}

export async function loadWhatsappWebWorkspace(
  selectedSessionId: string
): Promise<{
  capabilities: WhatsappCapabilitiesResponse | null
  sessions: WhatsappWebSession[]
  selectedSessionId: string
  error: string
}> {
  try {
    const [capabilityResponse, sessionResponse] = await Promise.all([
      fetchWhatsappCapabilities(),
      fetchWhatsappWebSessions(),
    ])

    const sessions = sessionResponse.items
    let nextSessionId = selectedSessionId
    if (!sessions.some((s) => s.session_id === nextSessionId)) {
      nextSessionId = sessions[0]?.session_id ?? ''
    }

    return {
      capabilities: capabilityResponse,
      sessions,
      selectedSessionId: nextSessionId,
      error: ''
    }
  } catch (error) {
    return {
      capabilities: nul
```
_Source file truncated after 12000 characters. / Исходный файл обрезан после 12000 символов._

### `frontend/src/integrations/whatsapp/api/whatsappCompanion.test.ts`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/integrations/whatsapp/api/whatsappCompanion.test.ts`
- Size bytes / Размер в байтах: `8499`
- Included characters / Включено символов: `8499`
- Truncated / Обрезано: `no`

```typescript
import { afterEach, describe, expect, it, vi } from 'vitest'

const invokeMock = vi.hoisted(() => vi.fn())

vi.mock('@tauri-apps/api/core', () => ({
  invoke: invokeMock,
}))

import {
  getWhatsappWebCompanionManifest,
  openWhatsappWebCompanion,
  relayWhatsappWebCompanionObservation,
} from './whatsappCompanion'

describe('whatsapp WebView companion Tauri bridge', () => {
  afterEach(() => {
    invokeMock.mockReset()
    vi.unstubAllGlobals()
  })

  it('opens the visible companion through Tauri invoke, not backend HTTP', async () => {
    const fetchMock = vi.fn()
    vi.stubGlobal('fetch', fetchMock)
    invokeMock.mockResolvedValueOnce(companionManifest({ opened_window: true }))

    const result = await openWhatsappWebCompanion(' wa-live-1 ')

    expect(result.opened_window).toBe(true)
    expect(result.provider_shape).toBe('whatsapp_web_companion')
    expect(invokeMock).toHaveBeenCalledWith('open_whatsapp_web_companion', {
      request: { account_id: 'wa-live-1' },
    })
    expect(fetchMock).not.toHaveBeenCalled()
  })

  it('loads the sanitized manifest without exposing secret-bearing fields', async () => {
    invokeMock.mockResolvedValueOnce(companionManifest({ opened_window: false }))

    const result = await getWhatsappWebCompanionManifest('wa-live-1')

    expect(invokeMock).toHaveBeenCalledWith('whatsapp_web_companion_manifest', {
      request: { account_id: 'wa-live-1' },
    })
    expect(result.target_url).toBe('https://web.whatsapp.com/')
    expect(result.bridge_routes.authorized_session_path).toBe(
      '/api/v1/integrations/whatsapp/runtime-bridge/sessions/authorized'
    )
    expect(result.command_channel.completion_rule).toBe(
      'provider_observed_event_reconciliation_required'
    )
    expect(result.event_extractor.state).toBe(
      'contract_injected_relay_dispatch_available'
    )
    expect(result.event_extractor.origin_guard).toBe('https://web.whatsapp.com')
    expect(result.event_extractor.relay_channel).toBe(
      'tauri_allowlisted_companion_runtime_bridge_dispatch'
    )
    expect(result.event_extractor.runtime_bridge_dispatch).toBe(
      'runtime_events_bridge_wired_smoke_pending'
    )
    expect(result.event_extractor.forbidden_reads).toContain('message_bodies')
    expect(result.event_extractor.forbidden_reads).toContain('media_bytes')
    expect(result.secret_policy.cookies).toBe('not_read_or_returned_by_tauri_command')
    expect(JSON.stringify(result)).not.toContain('cookie_value')
    expect(JSON.stringify(result)).not.toContain('session_blob')
  })

  it('relays sanitized companion observations only through the Tauri relay command', async () => {
    const fetchMock = vi.fn()
    vi.stubGlobal('fetch', fetchMock)
    invokeMock.mockResolvedValueOnce({
      account_id: 'wa-live-1',
      provider_shape: 'whatsapp_web_companion',
      runtime_kind: 'webview_companion',
      window_label: 'whatsapp-companion-wa-live-1',
      event_family: 'message',
      provider_event_id: 'provider-event-1',
      observed_at: '2026-06-26T20:00:00Z',
      target_runtime_bridge_path:
        '/api/v1/integrations/whatsapp/runtime-bridge/runtime-events',
      typed_runtime_bridge_path:
        '/api/v1/integrations/whatsapp/runtime-bridge/messages',
      relay_state: 'dispatched_to_backend_runtime_bridge_runtime_event',
      relay_channel: 'tauri_allowlisted_companion_runtime_bridge_dispatch',
      sanitized_metadata: { provider_chat_id: 'chat-1' },
      runtime_event_kind: 'webview_companion.message.observed',
      import_batch_id: 'whatsapp-webview-companion:wa-live-1:provider-event-1',
      runtime_bridge_http_status: 200,
      event_flow:
        'visible_webview_companion -> tauri_allowlisted_relay_preflight -> protected_runtime_bridge -> raw_evidence -> signal_hub_accepted -> projection_reconciliation',
      completion_rule: 'provider_observed_event_reconciliation_required',
    })

    const result = await relayWhatsappWebCompanionObservation(' wa-live-1 ', {
      event_family: 'message',
      provider_event_id: 'provider-event-1',
      observed_at: '2026-06-26T20:00:00Z',
      metadata: {
        provider_chat_id: 'chat-1',
      },
    })

    expect(result.relay_state).toBe(
      'dispatched_to_backend_runtime_bridge_runtime_event'
    )
    expect(result.target_runtime_bridge_path).toBe(
      '/api/v1/integrations/whatsapp/runtime-bridge/runtime-events'
    )
    expect(result.typed_runtime_bridge_path).toBe(
      '/api/v1/integrations/whatsapp/runtime-bridge/messages'
    )
    expect(result.runtime_event_kind).toBe('webview_companion.message.observed')
    expect(result.runtime_bridge_http_status).toBe(200)
    expect(invokeMock).toHaveBeenCalledWith('whatsapp_web_companion_relay_observation', {
      request: {
        account_id: 'wa-live-1',
        event_family: 'message',
        provider_event_id: 'provider-event-1',
        observed_at: '2026-06-26T20:00:00Z',
        metadata: {
          provider_chat_id: 'chat-1',
        },
      },
    })
    expect(fetchMock).not.toHaveBeenCalled()
  })

  it('rejects empty account ids before invoking Tauri', async () => {
    await expect(openWhatsappWebCompanion(' ')).rejects.toThrow(
      'account_id is required for WhatsApp Web companion'
    )

    expect(invokeMock).not.toHaveBeenCalled()
  })
})

function companionManifest(overrides: Partial<{ opened_window: boolean }>) {
  return {
    account_id: 'wa-live-1',
    provider_shape: 'whatsapp_web_companion',
    runtime_kind: 'webview_companion',
    driver_id: 'tauri_visible_webview_companion',
    window_label: 'whatsapp-companion-wa-live-1',
    target_url: 'https://web.whatsapp.com/',
    opened_window: overrides.opened_window ?? false,
    focused_existing_window: false,
    owner_visible: true,
    hidden_headless_mode: 'forbidden',
    tauri_ipc_available_to_companion_window: false,
    event_flow:
      'visible_webview_companion -> protected_runtime_bridge -> raw_evidence -> signal_hub_accepted -> projection_reconciliation',
    event_extractor: {
      state: 'contract_injected_relay_dispatch_available',
      relay_command: 'whatsapp_web_companion_relay_observation',
      initialization_script: 'installed_on_visible_companion_window',
      script_scope: 'main_frame_only',
      origin_guard: 'https://web.whatsapp.com',
      navigation_guard: 'https://web.whatsapp.com_only',
      relay_channel: 'tauri_allowlisted_companion_runtime_bridge_dispatch',
      runtime_bridge_dispatch: 'runtime_events_bridge_wired_smoke_pending',
      allowed_observations: [
        'runtime_lifecycle_metadata',
        'message_identity_metadata',
        'media_metadata_without_bytes',
      ],
      forbidden_reads: [
        'cookies',
        'web_storage',
        'indexed_db',
        'browser_profile_secrets',
        'session_material',
        'message_bodies',
        'media_bytes',
      ],
      next_gate: 'manual_live_smoke_before_public_availability',
    },
    bridge_routes: {
      authorized_session_path:
        '/api/v1/integrations/whatsapp/runtime-bridge/sessions/authorized',
      runtime_event_path: '/api/v1/integrations/whatsapp/runtime-bridge/runtime-events',
      sync_lifecycle_path: '/api/v1/integrations/whatsapp/runtime-bridge/sync-lifecycle',
      message_paths: ['/api/v1/integrations/whatsapp/runtime-bridge/messages'],
      conversation_paths: ['/api/v1/integrations/whatsapp/runtime-bridge/dialogs'],
      media_paths: ['/api/v1/integrations/whatsapp/runtime-bridge/media'],
    },
    command_channel: {
      kind: 'durable_outbox',
      claim_path: '/api/v1/integrations/whatsapp/runtime-bridge/commands/claim',
      failure_path:
        '/api/v1/integrations/whatsapp/runtime-bridge/commands/{command_id}/failed',
      completion_rule: 'provider_observed_event_reconciliation_required',
    },
    secret_policy: {
      session_material: 'host_vault_only_via_authorized_session_bridge',
      cookies: 'not_read_or_returned_by_tauri_command',
      browser_profile_secrets: 'not_read_or_returned_by_tauri_command',
      qr_pair_code_artifacts: 'owner_visible_runtime_only',
      message_bodies: 'excluded_from_manifest_and_health',
      media_bytes: 'local_blob_storage_only_not_manifest_or_postgres',
      postgres_storage: 'metadata_bindings_only_no_session_cookie_or_profile_secret',
    },
    remaining_blockers: [
      'whatsapp_webview_runtime_panel_action_not_implemented',
      'whatsapp_webview_live_smoke_required',
    ],
  }
}
```

### `frontend/src/integrations/whatsapp/api/whatsappCompanion.ts`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/integrations/whatsapp/api/whatsappCompanion.ts`
- Size bytes / Размер в байтах: `1610`
- Included characters / Включено символов: `1610`
- Truncated / Обрезано: `no`

```typescript
import { invoke } from '@tauri-apps/api/core'
import type {
  WhatsAppWebCompanionManifest,
  WhatsAppWebCompanionRelayObservationReceipt,
  WhatsAppWebCompanionRelayObservationRequest,
} from '../../../shared/communications/types/whatsapp'

type WhatsAppWebCompanionRequest = {
  account_id: string
}

export async function getWhatsappWebCompanionManifest(
  accountId: string
): Promise<WhatsAppWebCompanionManifest> {
  return invoke<WhatsAppWebCompanionManifest>(
    'whatsapp_web_companion_manifest',
    companionRequest(accountId)
  )
}

export async function openWhatsappWebCompanion(
  accountId: string
): Promise<WhatsAppWebCompanionManifest> {
  return invoke<WhatsAppWebCompanionManifest>(
    'open_whatsapp_web_companion',
    companionRequest(accountId)
  )
}

export async function relayWhatsappWebCompanionObservation(
  accountId: string,
  observation: Omit<WhatsAppWebCompanionRelayObservationRequest, 'account_id'>
): Promise<WhatsAppWebCompanionRelayObservationReceipt> {
  return invoke<WhatsAppWebCompanionRelayObservationReceipt>(
    'whatsapp_web_companion_relay_observation',
    {
      request: {
        ...observation,
        account_id: companionAccountId(accountId),
      },
    }
  )
}

function companionRequest(accountId: string): { request: WhatsAppWebCompanionRequest } {
  return {
    request: {
      account_id: companionAccountId(accountId),
    },
  }
}

function companionAccountId(accountId: string): string {
  const trimmed = accountId.trim()
  if (!trimmed) {
    throw new Error('account_id is required for WhatsApp Web companion')
  }
  return trimmed
}
```

### `frontend/src/integrations/whatsapp/api/whatsappRuntime.test.ts`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/integrations/whatsapp/api/whatsappRuntime.test.ts`
- Size bytes / Размер в байтах: `21903`
- Included characters / Включено символов: `12000`
- Truncated / Обрезано: `yes`

```typescript
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { ApiClient } from '../../../platform/api/ApiClient'
import {
  deadLetterWhatsappProviderCommand,
  fetchWhatsappAccounts,
  fetchWhatsappAccountCapabilities,
  fetchWhatsappProviderCommands,
  fetchWhatsappSyncChats,
  fetchWhatsappSyncCalls,
  fetchWhatsappSyncContacts,
  fetchWhatsappSyncHistory,
  fetchWhatsappSyncMedia,
  fetchWhatsappSyncMembers,
  fetchWhatsappSyncPresence,
  fetchWhatsappSyncStatuses,
  fetchWhatsappRuntimeHealth,
  fetchWhatsappRuntimeStatus,
  publishWhatsappStatus,
  relinkWhatsappRuntime,
  retryWhatsappProviderCommand,
  rotateWhatsappRuntime,
  removeWhatsappRuntime,
  revokeWhatsappRuntime,
  setupWhatsappLiveAccount,
  startWhatsappPairCodeLink,
  startWhatsappQrLink,
  startWhatsappRuntime,
  stopWhatsappRuntime,
} from './whatsapp'

describe('whatsapp runtime API', () => {
  beforeEach(() => {
    ApiClient.resetForTests()
    ApiClient.init('http://127.0.0.1:8080', 'test-secret')
  })

  afterEach(() => {
    vi.unstubAllGlobals()
    ApiClient.resetForTests()
  })

  it('loads whatsapp account list with and without removed accounts', async () => {
    const ok = (body: unknown) =>
      new Response(JSON.stringify(body), {
        status: 200,
        headers: { 'Content-Type': 'application/json' },
      })
    const fetchMock = vi
      .fn()
      .mockResolvedValueOnce(ok({ items: [{ account_id: 'wa-1', provider_kind: 'whatsapp_web', provider_shape: 'whatsapp_web_companion', display_name: 'Account One', external_account_id: 'wa:1', runtime: 'live_blocked', lifecycle_state: 'created', created_at: '2026-06-25T10:00:00Z', updated_at: '2026-06-25T10:00:00Z' }] }))
      .mockResolvedValueOnce(ok({ items: [{ account_id: 'wa-removed', provider_kind: 'whatsapp_web', provider_shape: 'whatsapp_web_companion', display_name: 'Removed Account', external_account_id: 'wa:removed', runtime: 'live_blocked', lifecycle_state: 'removed', created_at: '2026-06-25T10:00:00Z', updated_at: '2026-06-25T10:00:00Z' }] }))
    vi.stubGlobal('fetch', fetchMock)

    await fetchWhatsappAccounts()
    await fetchWhatsappAccounts(true)

    expect(fetchMock).toHaveBeenCalledTimes(2)
    expect(fetchMock.mock.calls[0][0]).toContain('/api/v1/integrations/whatsapp/accounts')
    expect(fetchMock.mock.calls[0][0]).not.toContain('include_removed=true')
    expect(fetchMock.mock.calls[1][0]).toContain('/api/v1/integrations/whatsapp/accounts?include_removed=true')
    expect(fetchMock.mock.calls[0][1].method).toBe('GET')
    expect(fetchMock.mock.calls[1][1].method).toBe('GET')
  })

  it('posts live whatsapp account setup by provider shape', async () => {
    const fetchMock = vi.fn().mockResolvedValueOnce(
      new Response(JSON.stringify({
        account_id: 'wa-live-1',
        provider_kind: 'whatsapp_business_cloud',
        runtime: 'live_blocked',
        session: {
          session_id: 'session-1',
          account_id: 'wa-live-1',
          device_name: 'WhatsApp Business Cloud API',
          companion_runtime: 'api_credentials',
          link_state: 'blocked',
          local_state_path: 'docker/data/whatsapp/business-cloud/wa-live-1',
          last_sync_at: null,
          metadata: {},
          created_at: '2026-06-25T10:00:00Z',
          updated_at: '2026-06-25T10:00:00Z',
        },
      }), {
        status: 200,
        headers: { 'Content-Type': 'application/json' },
      })
    )
    vi.stubGlobal('fetch', fetchMock)

    await setupWhatsappLiveAccount({
      account_id: 'wa-live-1',
      provider_kind: 'whatsapp_business_cloud',
      provider_shape: 'whatsapp_business_cloud',
      display_name: 'Business Cloud',
      external_account_id: 'wa-business-1',
      local_state_path: 'docker/data/whatsapp/business-cloud/wa-live-1',
    })

    expect(fetchMock).toHaveBeenCalledTimes(1)
    expect(fetchMock.mock.calls[0][0]).toContain('/api/v1/integrations/whatsapp/accounts')
    expect(fetchMock.mock.calls[0][1].method).toBe('POST')
    expect(JSON.parse(fetchMock.mock.calls[0][1].body as string)).toEqual({
      account_id: 'wa-live-1',
      provider_kind: 'whatsapp_business_cloud',
      provider_shape: 'whatsapp_business_cloud',
      display_name: 'Business Cloud',
      external_account_id: 'wa-business-1',
      local_state_path: 'docker/data/whatsapp/business-cloud/wa-live-1',
    })
  })

  it('calls account capabilities and runtime lifecycle routes', async () => {
    const ok = (body: unknown) =>
      new Response(JSON.stringify(body), {
        status: 200,
        headers: { 'Content-Type': 'application/json' },
      })
    const fetchMock = vi
      .fn()
      .mockResolvedValueOnce(ok({ version: '2.0', runtime_mode: 'fixture', provider_shapes: [], account_scope: null, capabilities: [], planned_features: [], unsupported_features: [] }))
      .mockResolvedValueOnce(ok({ account_id: 'wa-1', provider_kind: 'whatsapp_web', provider_shape: 'whatsapp_web_companion', runtime_kind: 'fixture', status: 'linked', fixture_runtime: true, live_runtime_available: false, live_send_available: false, qr_pairing_available: true, pair_code_available: true, media_download_available: false, media_upload_available: false, session_restore_available: true, session_secret_ref: 'secret:wa-1', runtime_blockers: [], last_error: null, updated_at: '2026-06-25T10:00:00Z' }))
      .mockResolvedValueOnce(ok({ account_id: 'wa-1', provider_shape: 'whatsapp_web_companion', runtime_kind: 'fixture', status: 'available', healthy: true, checks: {}, checked_at: '2026-06-25T10:00:00Z' }))
      .mockResolvedValueOnce(ok({ account_id: 'wa-1', provider_kind: 'whatsapp_web', provider_shape: 'whatsapp_web_companion', runtime_kind: 'fixture', status: 'available', fixture_runtime: true, live_runtime_available: false, live_send_available: false, qr_pairing_available: true, pair_code_available: true, media_download_available: false, media_upload_available: false, session_restore_available: true, session_secret_ref: 'secret:wa-1', runtime_blockers: [], last_error: null, updated_at: '2026-06-25T10:00:00Z' }))
      .mockResolvedValueOnce(ok({ account_id: 'wa-1', provider_kind: 'whatsapp_web', provider_shape: 'whatsapp_web_companion', runtime_kind: 'fixture', status: 'linked', fixture_runtime: true, live_runtime_available: false, live_send_available: false, qr_pairing_available: true, pair_code_available: true, media_download_available: false, media_upload_available: false, session_restore_available: true, session_secret_ref: 'secret:wa-1', runtime_blockers: [], last_error: null, updated_at: '2026-06-25T10:00:00Z' }))
      .mockResolvedValueOnce(ok({ account_id: 'wa-1', provider_kind: 'whatsapp_web', provider_shape: 'whatsapp_web_companion', runtime_kind: 'fixture', status: 'revoked', fixture_runtime: true, live_runtime_available: false, live_send_available: false, qr_pairing_available: false, pair_code_available: false, media_download_available: false, media_upload_available: false, session_restore_available: false, session_secret_ref: null, runtime_blockers: ['whatsapp_session_revoked'], last_error: null, updated_at: '2026-06-25T10:00:00Z' }))
      .mockResolvedValueOnce(ok({ account_id: 'wa-1', provider_kind: 'whatsapp_web', provider_shape: 'whatsapp_web_companion', runtime_kind: 'fixture', status: 'link_required', fixture_runtime: true, live_runtime_available: false, live_send_available: false, qr_pairing_available: true, pair_code_available: true, media_download_available: false, media_upload_available: false, session_restore_available: false, session_secret_ref: null, runtime_blockers: ['whatsapp_session_link_required'], last_error: null, updated_at: '2026-06-25T10:00:00Z' }))
      .mockResolvedValueOnce(ok({ account_id: 'wa-1', provider_kind: 'whatsapp_web', provider_shape: 'whatsapp_web_companion', runtime_kind: 'fixture', status: 'link_required', fixture_runtime: true, live_runtime_available: false, live_send_available: false, qr_pairing_available: true, pair_code_available: true, media_download_available: false, media_upload_available: false, session_restore_available: false, session_secret_ref: null, runtime_blockers: ['whatsapp_session_link_required'], last_error: null, updated_at: '2026-06-25T10:00:00Z' }))
      .mockResolvedValueOnce(ok({ account_id: 'wa-1', provider_kind: 'whatsapp_web', removed: true, unbound_secret_refs: ['secret:wa-1'], removed_at: '2026-06-25T10:00:00Z' }))
      .mockResolvedValueOnce(ok({ account_id: 'wa-1', provider_shape: 'whatsapp_web_companion', runtime_kind: 'fixture', status: 'qr_pending', setup_id: 'qr-1', qr_svg: '<svg />', expires_at: null, runtime_blockers: [] }))
      .mockResolvedValueOnce(ok({ account_id: 'wa-1', provider_shape: 'whatsapp_web_companion', runtime_kind: 'fixture', status: 'pair_code_pending', setup_id: 'pair-1', phone_number: '+34123456789', pair_code: '123-456', expires_at: null, runtime_blockers: [] }))
    vi.stubGlobal('fetch', fetchMock)

    await fetchWhatsappAccountCapabilities('wa-1')
    await fetchWhatsappRuntimeStatus('wa-1')
    await fetchWhatsappRuntimeHealth('wa-1')
    await startWhatsappRuntime({ account_id: 'wa-1' })
    await stopWhatsappRuntime({ account_id: 'wa-1' })
    await revokeWhatsappRuntime({ account_id: 'wa-1' })
    await relinkWhatsappRuntime({ account_id: 'wa-1' })
    await rotateWhatsappRuntime({ account_id: 'wa-1' })
    await removeWhatsappRuntime({ account_id: 'wa-1' })
    await startWhatsappQrLink({ account_id: 'wa-1' })
    await startWhatsappPairCodeLink({ account_id: 'wa-1', phone_number: '+34123456789' })

    expect(fetchMock).toHaveBeenCalledTimes(11)
    expect(fetchMock.mock.calls[0][0]).toContain('/api/v1/integrations/whatsapp/accounts/wa-1/capabilities')
    expect(fetchMock.mock.calls[1][0]).toContain('/api/v1/integrations/whatsapp/runtime/status?account_id=wa-1')
    expect(fetchMock.mock.calls[2][0]).toContain('/api/v1/integrations/whatsapp/runtime/health?account_id=wa-1')
    expect(fetchMock.mock.calls[3][0]).toContain('/api/v1/integrations/whatsapp/runtime/start')
    expect(fetchMock.mock.calls[4][0]).toContain('/api/v1/integrations/whatsapp/runtime/stop')
    expect(fetchMock.mock.calls[5][0]).toContain('/api/v1/integrations/whatsapp/runtime/revoke')
    expect(fetchMock.mock.calls[6][0]).toContain('/api/v1/integrations/whatsapp/runtime/relink')
    expect(fetchMock.mock.calls[7][0]).toContain('/api/v1/integrations/whatsapp/runtime/rotate')
    expect(fetchMock.mock.calls[8][0]).toContain('/api/v1/integrations/whatsapp/runtime/remove')
    expect(fetchMock.mock.calls[9][0]).toContain('/api/v1/integrations/whatsapp/login/qr/start')
    expect(fetchMock.mock.calls[10][0]).toContain('/api/v1/integrations/whatsapp/login/pair-code/start')
  })

  it('loads provider commands and posts retry/dead-letter actions', async () => {
    const ok = (body: unknown) =>
      new Response(JSON.stringify(body), {
        status: 200,
        headers: { 'Content-Type': 'application/json' },
      })
    const fetchMock = vi
      .fn()
      .mockResolvedValueOnce(ok({
        items: [
          {
            command_id: 'wa-cmd-1',
            account_id: 'wa-1',
            command_kind: 'send_text',
            idempotency_key: 'send:1',
            provider_chat_id: 'chat-1',
            provider_message_id: null,
            capability_state: 'available',
            action_class: 'provider_write',
            confirmation_decision: 'not_required',
            status: 'failed',
            retry_count: 1,
            max_retries: 3,
            last_error: 'temporary failure',
            result_payload: {},
            audit_metadata: {},
            provider_state: {},
            reconciliation_status: 'not_observed',
            next_attempt_at: null,
            last_attempt_at: '2026-06-26T09:00:00Z',
            provider_observed_at: null,
            reconciled_at: null,
            dead_lettered_at: null,
            completed_at: null,
            created_at: '2026-06-2
```
_Source file truncated after 12000 characters. / Исходный файл обрезан после 12000 символов._

### `frontend/src/integrations/whatsapp/queries/realtimeWhatsAppRuntimePatchValues.ts`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/integrations/whatsapp/queries/realtimeWhatsAppRuntimePatchValues.ts`
- Size bytes / Размер в байтах: `752`
- Included characters / Включено символов: `752`
- Truncated / Обрезано: `no`

```typescript
import { stringValue } from '../../../shared/communications/queries/realtimePatchShared'

export type WhatsAppRuntimeEventPayload = Record<string, unknown>

export function integerValue(value: unknown): number | null {
	return typeof value === 'number' && Number.isInteger(value) ? value : null
}

export function booleanValue(value: unknown): boolean | null {
	return typeof value === 'boolean' ? value : null
}

export function stringArray(value: unknown): string[] | null {
	return Array.isArray(value) && value.every((item) => typeof item === 'string')
		? [...value]
		: null
}

export function nullableStringValue(value: unknown, fallback: string | null): string | null {
	if (value === null) return null
	return stringValue(value) ?? fallback
}
```

### `frontend/src/integrations/whatsapp/queries/realtimeWhatsAppRuntimePatches.ts`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/integrations/whatsapp/queries/realtimeWhatsAppRuntimePatches.ts`
- Size bytes / Размер в байтах: `12626`
- Included characters / Включено символов: `12000`
- Truncated / Обрезано: `yes`

```typescript
import type {
	WhatsAppCallSyncItem,
	WhatsAppChatSyncItem,
	WhatsAppContactSyncItem,
	WhatsAppMembersSyncItem,
	WhatsAppPresenceSyncItem,
	WhatsAppProviderCommand,
	WhatsAppRuntimeStatus,
	WhatsappWebMessage,
	WhatsappWebSession,
} from '../types/whatsapp'
import {
	isRecord,
	storedEventEnvelope,
	stringValue,
} from '../../../shared/communications/queries/realtimePatchShared'
import {
	booleanValue,
	integerValue,
	nullableStringValue,
	stringArray,
	type WhatsAppRuntimeEventPayload,
} from './realtimeWhatsAppRuntimePatchValues'
import {
	patchCallList,
	patchChatsList,
	patchContactsList,
	patchMembersList,
	patchPresenceList,
	patchStatusesList,
} from './realtimeWhatsAppRuntimeSyncPatches'

export type WhatsAppRuntimePatchQueryClient = {
	getQueriesData?: <TData>(filters: { queryKey: readonly unknown[] }) => Array<
		[readonly unknown[], TData | undefined]
	>
	setQueryData?: <TData>(
		queryKey: readonly unknown[],
		updater: TData | ((data: TData | undefined) => TData | undefined)
	) => unknown
}

export function applyWhatsAppRuntimeRealtimePatch(
	eventData: string,
	queryClient: WhatsAppRuntimePatchQueryClient
): boolean {
	const { getQueriesData, setQueryData } = queryClient
	if (!getQueriesData || !setQueryData) return false

	const envelope = storedEventEnvelope(eventData)
	const eventType = stringValue(envelope?.event?.event_type)
	if (!eventType || !eventType.startsWith('whatsapp.')) return false

	const payload = isRecord(envelope?.event?.payload)
		? (envelope.event?.payload as WhatsAppRuntimeEventPayload)
		: undefined
	if (!payload) return false

	let patched = false
	for (const [queryKey, data] of getQueriesData<WhatsappWebSession[]>({
		queryKey: ['integrations', 'whatsapp', 'sessions'],
	})) {
		const updated = patchSessionList(queryKey, data, eventType, payload)
		if (updated !== data) {
			setQueryData(queryKey, updated)
			patched = true
		}
	}

	for (const [queryKey, data] of getQueriesData<WhatsAppRuntimeStatus | null>({
		queryKey: ['integrations', 'whatsapp', 'runtime', 'status'],
	})) {
		const updated = patchRuntimeStatus(queryKey, data, eventType, payload)
		if (updated !== data) {
			setQueryData(queryKey, updated)
			patched = true
		}
	}

	for (const [queryKey, data] of getQueriesData<WhatsAppProviderCommand[]>({
		queryKey: ['integrations', 'whatsapp', 'commands'],
	})) {
		const updated = patchCommandList(queryKey, data, eventType, payload)
		if (updated !== data) {
			setQueryData(queryKey, updated)
			patched = true
		}
	}

	for (const [queryKey, data] of getQueriesData<WhatsAppPresenceSyncItem[]>({
		queryKey: ['integrations', 'whatsapp', 'runtime', 'sync-presence'],
	})) {
		const updated = patchPresenceList(queryKey, data, eventType, payload)
		if (updated !== data) {
			setQueryData(queryKey, updated)
			patched = true
		}
	}

	for (const [queryKey, data] of getQueriesData<WhatsAppCallSyncItem[]>({
		queryKey: ['integrations', 'whatsapp', 'runtime', 'sync-calls'],
	})) {
		const updated = patchCallList(queryKey, data, eventType, payload)
		if (updated !== data) {
			setQueryData(queryKey, updated)
			patched = true
		}
	}

	for (const [queryKey, data] of getQueriesData<WhatsappWebMessage[]>({
		queryKey: ['integrations', 'whatsapp', 'runtime', 'sync-statuses'],
	})) {
		const updated = patchStatusesList(queryKey, data, eventType, payload)
		if (updated !== data) {
			setQueryData(queryKey, updated)
			patched = true
		}
	}

	for (const [queryKey, data] of getQueriesData<WhatsAppChatSyncItem[]>({
		queryKey: ['integrations', 'whatsapp', 'runtime', 'sync-chats'],
	})) {
		const updated = patchChatsList(queryKey, data, eventType, payload)
		if (updated !== data) {
			setQueryData(queryKey, updated)
			patched = true
		}
	}

	for (const [queryKey, data] of getQueriesData<WhatsAppMembersSyncItem[]>({
		queryKey: ['integrations', 'whatsapp', 'runtime', 'sync-members'],
	})) {
		const updated = patchMembersList(queryKey, data, eventType, payload)
		if (updated !== data) {
			setQueryData(queryKey, updated)
			patched = true
		}
	}

	for (const [queryKey, data] of getQueriesData<WhatsAppContactSyncItem[]>({
		queryKey: ['integrations', 'whatsapp', 'runtime', 'sync-contacts'],
	})) {
		const updated = patchContactsList(queryKey, data, eventType, payload)
		if (updated !== data) {
			setQueryData(queryKey, updated)
			patched = true
		}
	}

	return patched
}

function patchSessionList(
	queryKey: readonly unknown[],
	sessions: WhatsappWebSession[] | undefined,
	eventType: string,
	payload: WhatsAppRuntimeEventPayload
): WhatsappWebSession[] | undefined {
	if (!sessions) return sessions
	if (
		eventType !== 'whatsapp.runtime.status_changed' &&
		eventType !== 'whatsapp.session.link_state_changed' &&
		eventType !== 'whatsapp.runtime.event'
	) {
		return sessions
	}

	const payloadAccountId = stringValue(payload.account_id)
	if (!payloadAccountId) return sessions
	const queryAccountId = typeof queryKey[3] === 'string' && queryKey[3] !== 'all' ? queryKey[3] : null
	if (queryAccountId && queryAccountId !== payloadAccountId) return sessions

	let changed = false
	const updatedSessions = sessions.map((session) => {
		if (session.account_id !== payloadAccountId) return session
		const updated = patchSession(session, eventType, payload)
		if (updated !== session) changed = true
		return updated
	})

	return changed ? updatedSessions : sessions
}

function patchSession(
	session: WhatsappWebSession,
	eventType: string,
	payload: WhatsAppRuntimeEventPayload
): WhatsappWebSession {
	if (eventType === 'whatsapp.session.link_state_changed') {
		const linkState = linkStateValue(payload.link_state)
		if (!linkState) return session
		return {
			...session,
			link_state: linkState,
			updated_at: stringValue(payload.occurred_at) ?? session.updated_at,
		}
	}

	if (eventType === 'whatsapp.runtime.status_changed') {
		const status = stringValue(payload.status)
		if (!status) return session
		return {
			...session,
			metadata: {
				...session.metadata,
				runtime_status: status,
				runtime_status_source: stringValue(payload.source),
			},
			updated_at: stringValue(payload.occurred_at) ?? session.updated_at,
		}
	}

	const runtimeStatus = stringValue(payload.runtime_status)
	const lifecycleState = stringValue(payload.lifecycle_state)
	if (!runtimeStatus && !lifecycleState) return session

	return {
		...session,
		link_state: linkStateValue(lifecycleState) ?? session.link_state,
		metadata: {
			...session.metadata,
			...(runtimeStatus ? { runtime_status: runtimeStatus } : {}),
			...(lifecycleState ? { lifecycle_state: lifecycleState } : {}),
			...(stringValue(payload.provider_shape)
				? { provider_shape: stringValue(payload.provider_shape) }
				: {}),
			...(stringValue(payload.runtime_kind)
				? { runtime_kind: stringValue(payload.runtime_kind) }
				: {}),
			...(stringValue(payload.provider_event_id)
				? { provider_event_id: stringValue(payload.provider_event_id) }
				: {}),
		},
		updated_at:
			stringValue(payload.occurred_at) ??
			stringValue(payload.observed_at) ??
			session.updated_at,
	}
}

function patchRuntimeStatus(
	queryKey: readonly unknown[],
	status: WhatsAppRuntimeStatus | null | undefined,
	eventType: string,
	payload: WhatsAppRuntimeEventPayload
): WhatsAppRuntimeStatus | null | undefined {
	if (!status) return status

	const payloadAccountId = stringValue(payload.account_id)
	if (!payloadAccountId) return status
	const queryAccountId =
		typeof queryKey[4] === 'string' && queryKey[4] !== 'none' ? queryKey[4] : null
	if (queryAccountId && queryAccountId !== payloadAccountId) return status
	if (status.account_id !== payloadAccountId) return status

	if (eventType === 'whatsapp.runtime.status_changed') {
		const nextStatus = stringValue(payload.status)
		if (!nextStatus) return status
		return {
			...status,
			provider_kind: stringValue(payload.provider_kind) ?? status.provider_kind,
			provider_shape: stringValue(payload.provider_shape) ?? status.provider_shape,
			runtime_kind: stringValue(payload.runtime_kind) ?? status.runtime_kind,
			status: nextStatus,
			fixture_runtime: booleanValue(payload.fixture_runtime) ?? status.fixture_runtime,
			live_runtime_available:
				booleanValue(payload.live_runtime_available) ?? status.live_runtime_available,
			live_send_available:
				booleanValue(payload.live_send_available) ?? status.live_send_available,
			qr_pairing_available:
				booleanValue(payload.qr_pairing_available) ?? status.qr_pairing_available,
			pair_code_available:
				booleanValue(payload.pair_code_available) ?? status.pair_code_available,
			media_download_available:
				booleanValue(payload.media_download_available) ?? status.media_download_available,
			media_upload_available:
				booleanValue(payload.media_upload_available) ?? status.media_upload_available,
			session_restore_available:
				booleanValue(payload.session_restore_available) ?? status.session_restore_available,
			runtime_blockers: stringArray(payload.runtime_blockers) ?? status.runtime_blockers,
			last_error: nullableStringValue(payload.last_error, status.last_error),
			updated_at: status.updated_at,
		}
	}

	if (eventType === 'whatsapp.session.link_state_changed') {
		const linkState = stringValue(payload.link_state)
		if (!linkState) return status
		return {
			...status,
			provider_shape: stringValue(payload.provider_shape) ?? status.provider_shape,
			runtime_kind: stringValue(payload.runtime_kind) ?? status.runtime_kind,
			status: linkState,
		}
	}

	const runtimeStatus = stringValue(payload.runtime_status)
	const lifecycleState = stringValue(payload.lifecycle_state)
	if (!runtimeStatus && !lifecycleState) return status

	return {
		...status,
		status: lifecycleState ?? runtimeStatus ?? status.status,
	}
}

function patchCommandList(
	queryKey: readonly unknown[],
	commands: WhatsAppProviderCommand[] | undefined,
	eventType: string,
	payload: WhatsAppRuntimeEventPayload
): WhatsAppProviderCommand[] | undefined {
	if (!commands || eventType !== 'whatsapp.command.status_changed') return commands

	const payloadAccountId = stringValue(payload.account_id)
	if (!payloadAccountId) return commands
	const queryAccountId =
		typeof queryKey[3] === 'string' && queryKey[3] !== 'none' ? queryKey[3] : null
	if (queryAccountId && queryAccountId !== payloadAccountId) return commands

	const commandId = stringValue(payload.command_id)
	if (!commandId) return commands

	let changed = false
	const updatedCommands = commands.map((command) => {
		if (command.command_id !== commandId) return command
		changed = true
		return {
			...command,
			account_id: payloadAccountId,
			command_kind: stringValue(payload.command_kind) ?? command.command_kind,
			provider_chat_id: stringValue(payload.provider_chat_id) ?? command.provider_chat_id,
			provider_message_id: nullableStringValue(
				payload.provider_message_id,
				command.provider_message_id
			),
			status: stringValue(payload.status) ?? command.status,
			last_error: nullableStringValue(payload.last_error, command.last_error),
			retry_count: integerValue(payload.retry_count) ?? command.retry_count,
			max_retries: integerValue(payload.max_retries) ?? command.max_retries,
			reconciliation_status:
				stringValue(payload.reconciliation_status) ?? command.reconciliation_status,
			next_attempt_at: nullableStringValue(payload.next_attempt_at, command.next_attempt_at),
			last_attempt_at: nullableStringValue(payload.last_attempt_at, command.last_attempt_at),
			provider_observed_at: nullableStringValue(
				payload.provider_observed_at,
				command.provider_observed_at
			),
			reconciled_at: nullableStringValue(payload.reconciled_at, command.reconciled_at),
			dead_lettered_at: nullableStringValue(
				payload.dead_lettered_at,
				command.dead_lettered_at
			),
			completed_at: nullableStringValue(payload.completed_at, command.completed_at),
			updated_at:
				nullableStringValue(payload.completed_at, null) ??
				nullableStringValue(payload.reconciled_at, null) ??
				nullableStringValue(payload.p
```
_Source file truncated after 12000 characters. / Исходный файл обрезан после 12000 символов._

### `frontend/src/integrations/whatsapp/queries/realtimeWhatsAppRuntimeSyncPatches.ts`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/integrations/whatsapp/queries/realtimeWhatsAppRuntimeSyncPatches.ts`
- Size bytes / Размер в байтах: `15221`
- Included characters / Включено символов: `12000`
- Truncated / Обрезано: `yes`

```typescript
import type {
	WhatsAppCallSyncItem,
	WhatsAppChatSyncItem,
	WhatsAppContactSyncItem,
	WhatsAppMembersSyncItem,
	WhatsAppPresenceSyncItem,
	WhatsappWebMessage,
} from '../types/whatsapp'
import { isRecord, stringValue } from '../../../shared/communications/queries/realtimePatchShared'
import {
	booleanValue,
	integerValue,
	nullableStringValue,
	stringArray,
	type WhatsAppRuntimeEventPayload,
} from './realtimeWhatsAppRuntimePatchValues'

export function patchPresenceList(
	queryKey: readonly unknown[],
	items: WhatsAppPresenceSyncItem[] | undefined,
	eventType: string,
	payload: WhatsAppRuntimeEventPayload
): WhatsAppPresenceSyncItem[] | undefined {
	if (!items || eventType !== 'whatsapp.presence.changed') return items

	const payloadAccountId = stringValue(payload.account_id)
	if (!payloadAccountId) return items
	const queryAccountId =
		typeof queryKey[4] === 'string' && queryKey[4] !== 'none' ? queryKey[4] : null
	if (queryAccountId && queryAccountId !== payloadAccountId) return items

	const queryProviderChatId =
		typeof queryKey[5] === 'string' && queryKey[5] !== 'all' ? queryKey[5] : null
	const payloadProviderChatId = nullableStringValue(payload.provider_chat_id, null)
	if (queryProviderChatId && queryProviderChatId !== payloadProviderChatId) return items

	const identityId =
		nullableStringValue(payload.identity_id, null) ??
		stringValue(payload.provider_identity_id)
	if (!identityId) return items

	const nextItem: WhatsAppPresenceSyncItem = {
		identity_id: identityId,
		account_id: payloadAccountId,
		channel_kind: 'whatsapp_web',
		provider_chat_id: payloadProviderChatId,
		provider_identity_id: stringValue(payload.provider_identity_id) ?? identityId,
		identity_kind: stringValue(payload.identity_kind) ?? 'whatsapp',
		display_name: nullableStringValue(payload.display_name, null),
		address: nullableStringValue(payload.address, null),
		presence_state: stringValue(payload.presence_state) ?? 'unknown',
		last_seen_at: nullableStringValue(payload.last_seen_at, null),
		observed_at: nullableStringValue(payload.observed_at, null),
		identity_metadata: {},
	}

	let changed = false
	const updatedItems = items.map((item) => {
		if (
			item.identity_id !== identityId &&
			item.provider_identity_id !== nextItem.provider_identity_id
		) {
			return item
		}
		changed = true
		return {
			...item,
			...nextItem,
			channel_kind: item.channel_kind,
			identity_metadata: item.identity_metadata ?? {},
		}
	})

	if (changed) return updatedItems
	return [nextItem, ...items]
}

export function patchCallList(
	queryKey: readonly unknown[],
	items: WhatsAppCallSyncItem[] | undefined,
	eventType: string,
	payload: WhatsAppRuntimeEventPayload
): WhatsAppCallSyncItem[] | undefined {
	if (!items || eventType !== 'whatsapp.call.updated') return items

	const payloadAccountId = stringValue(payload.account_id)
	const callId = stringValue(payload.call_id)
	const providerCallId = stringValue(payload.provider_call_id)
	const providerChatId = stringValue(payload.provider_chat_id)
	if (!payloadAccountId || !callId || !providerCallId || !providerChatId) return items

	const queryAccountId =
		typeof queryKey[4] === 'string' && queryKey[4] !== 'none' ? queryKey[4] : null
	if (queryAccountId && queryAccountId !== payloadAccountId) return items

	const queryProviderChatId =
		typeof queryKey[5] === 'string' && queryKey[5] !== 'all' ? queryKey[5] : null
	if (queryProviderChatId && queryProviderChatId !== providerChatId) return items

	const nextItem: WhatsAppCallSyncItem = {
		call_id: callId,
		account_id: payloadAccountId,
		provider_call_id: providerCallId,
		provider_chat_id: providerChatId,
		direction: stringValue(payload.direction) ?? 'unknown',
		call_state: stringValue(payload.call_state) ?? 'unknown',
		started_at: nullableStringValue(payload.started_at, null),
		ended_at: nullableStringValue(payload.ended_at, null),
		observed_at: nullableStringValue(payload.observed_at, null),
		metadata: {},
	}

	let changed = false
	const updatedItems = items.map((item) => {
		if (item.call_id !== callId && item.provider_call_id !== providerCallId) return item
		changed = true
		return {
			...item,
			...nextItem,
			metadata: item.metadata ?? {},
		}
	})

	if (changed) return updatedItems
	return [nextItem, ...items]
}

export function patchStatusesList(
	queryKey: readonly unknown[],
	items: WhatsappWebMessage[] | undefined,
	eventType: string,
	payload: WhatsAppRuntimeEventPayload
): WhatsappWebMessage[] | undefined {
	if (!items) return items
	if (eventType !== 'whatsapp.status.updated' && eventType !== 'whatsapp.status.deleted') {
		return items
	}

	const payloadAccountId = stringValue(payload.account_id)
	const messageId = stringValue(payload.message_id)
	if (!payloadAccountId || !messageId) return items

	const queryAccountId =
		typeof queryKey[4] === 'string' && queryKey[4] !== 'none' ? queryKey[4] : null
	if (queryAccountId && queryAccountId !== payloadAccountId) return items

	const statusState =
		stringValue(payload.status_state) ??
		(eventType === 'whatsapp.status.deleted' ? 'deleted' : 'posted')
	const providerStatusId = stringValue(payload.provider_status_id) ?? messageId
	const nextOccurredAt =
		nullableStringValue(payload.occurred_at, null) ??
		nullableStringValue(payload.observed_at, null)

	const nextItem: WhatsappWebMessage = {
		message_id: messageId,
		raw_record_id: stringValue(payload.raw_record_id) ?? `status:${providerStatusId}`,
		account_id: payloadAccountId,
		provider_message_id: providerStatusId,
		provider_chat_id: `whatsapp_status_feed:${payloadAccountId}`,
		chat_title: 'status-feed',
		sender:
			stringValue(payload.sender_id) ??
			stringValue(payload.sender_address) ??
			providerStatusId,
		sender_display_name:
			nullableStringValue(payload.sender_display_name, null) ??
			nullableStringValue(payload.viewer_display_name, null),
		text: '',
		occurred_at: nextOccurredAt,
		projected_at: nextOccurredAt ?? new Date().toISOString(),
		channel_kind: 'whatsapp_web',
		delivery_state: statusState === 'deleted' ? 'deleted' : 'published',
		metadata: {
			provider_status_id: providerStatusId,
			status_state: statusState,
			...(stringValue(payload.sender_identity_kind)
				? { sender_identity_kind: stringValue(payload.sender_identity_kind) }
				: {}),
			...(stringValue(payload.sender_address)
				? { sender_address: stringValue(payload.sender_address) }
				: {}),
			...(stringValue(payload.sender_push_name)
				? { sender_push_name: stringValue(payload.sender_push_name) }
				: {}),
			...(stringValue(payload.viewer_id) ? { viewer_id: stringValue(payload.viewer_id) } : {}),
			...(stringValue(payload.actor_class)
				? { actor_class: stringValue(payload.actor_class) }
				: {}),
			...(stringValue(payload.reason_class)
				? { reason_class: stringValue(payload.reason_class) }
				: {}),
			...(stringValue(payload.tombstone_id)
				? { tombstone_id: stringValue(payload.tombstone_id) }
				: {}),
		},
	}

	let changed = false
	const updatedItems = items.map((item) => {
		const itemProviderStatusId =
			typeof item.metadata?.provider_status_id === 'string'
				? item.metadata.provider_status_id
				: null
		if (item.message_id !== messageId && itemProviderStatusId !== providerStatusId) return item
		changed = true
		return {
			...item,
			...nextItem,
			text: item.text,
			metadata: {
				...item.metadata,
				...nextItem.metadata,
			},
		}
	})

	if (changed) return updatedItems
	return [nextItem, ...items]
}

export function patchChatsList(
	queryKey: readonly unknown[],
	items: WhatsAppChatSyncItem[] | undefined,
	eventType: string,
	payload: WhatsAppRuntimeEventPayload
): WhatsAppChatSyncItem[] | undefined {
	if (!items || eventType !== 'whatsapp.dialog.updated') return items

	const payloadAccountId = stringValue(payload.account_id)
	const providerChatId = stringValue(payload.provider_chat_id)
	if (!payloadAccountId || !providerChatId) return items

	const queryAccountId =
		typeof queryKey[4] === 'string' && queryKey[4] !== 'none' ? queryKey[4] : null
	if (queryAccountId && queryAccountId !== payloadAccountId) return items

	const nextItem: WhatsAppChatSyncItem = {
		conversation_id: stringValue(payload.conversation_id) ?? providerChatId,
		account_id: payloadAccountId,
		channel_kind: 'whatsapp_web',
		provider_chat_id: providerChatId,
		title: stringValue(payload.chat_title) ?? providerChatId,
		chat_kind: nullableStringValue(payload.chat_kind, null),
		is_archived: booleanValue(payload.is_archived) ?? false,
		is_pinned: booleanValue(payload.is_pinned) ?? false,
		is_muted: booleanValue(payload.is_muted) ?? false,
		is_unread: booleanValue(payload.is_unread) ?? false,
		unread_count: integerValue(payload.unread_count),
		participant_count: integerValue(payload.participant_count),
		community_parent_chat_id: nullableStringValue(payload.community_parent_chat_id, null),
		community_parent_title: nullableStringValue(payload.community_parent_title, null),
		invite_link: nullableStringValue(payload.invite_link, null),
		is_community_root: booleanValue(payload.is_community_root) ?? false,
		is_broadcast: booleanValue(payload.is_broadcast) ?? false,
		is_newsletter: booleanValue(payload.is_newsletter) ?? false,
		avatar_metadata: isRecord(payload.avatar_metadata)
			? { ...(payload.avatar_metadata as Record<string, unknown>) }
			: {},
		provider_labels: stringArray(payload.provider_labels) ?? [],
	}

	let changed = false
	const updatedItems = items.map((item) => {
		if (item.provider_chat_id !== providerChatId && item.conversation_id !== nextItem.conversation_id) {
			return item
		}
		changed = true
		return {
			...item,
			...nextItem,
			channel_kind: item.channel_kind,
			avatar_metadata: nextItem.avatar_metadata,
			provider_labels: nextItem.provider_labels,
		}
	})

	if (changed) return updatedItems
	return [nextItem, ...items]
}

export function patchMembersList(
	queryKey: readonly unknown[],
	items: WhatsAppMembersSyncItem[] | undefined,
	eventType: string,
	payload: WhatsAppRuntimeEventPayload
): WhatsAppMembersSyncItem[] | undefined {
	if (!items || eventType !== 'whatsapp.participant.changed') return items

	const payloadAccountId = stringValue(payload.account_id)
	const providerChatId = stringValue(payload.provider_chat_id)
	const participantId = stringValue(payload.participant_id)
	const providerMemberId = stringValue(payload.provider_member_id)
	if (!payloadAccountId || !providerChatId || !participantId || !providerMemberId) return items

	const queryAccountId =
		typeof queryKey[4] === 'string' && queryKey[4] !== 'none' ? queryKey[4] : null
	if (queryAccountId && queryAccountId !== payloadAccountId) return items

	const queryProviderChatId =
		typeof queryKey[5] === 'string' && queryKey[5] !== 'none' ? queryKey[5] : null
	if (queryProviderChatId && queryProviderChatId !== providerChatId) return items

	const nextItem: WhatsAppMembersSyncItem = {
		participant_id: participantId,
		conversation_id: stringValue(payload.conversation_id) ?? providerChatId,
		account_id: payloadAccountId,
		provider_chat_id: providerChatId,
		provider_member_id: providerMemberId,
		provider_identity_id: nullableStringValue(payload.provider_identity_id, null),
		sender_display_name: nullableStringValue(payload.display_name, null),
		role: stringValue(payload.role) ?? 'member',
		status: nullableStringValue(payload.status, null),
		identity_kind: null,
		address: null,
		is_admin: false,
		is_owner: false,
		participant_metadata: {
			...(stringValue(payload.previous_role)
				? { previous_role: stringValue(payload.previous_role) }
				: {}),
			...(stringValue(payload.previous_status)
				? { previous_status: stringValue(payload.previous_status) }
				: {}),
			...(booleanValue(payload.role_changed) !== null
				? { role_changed: booleanValue(payload.role_changed) }
				: {}),
			...(booleanValue(payload.membership_changed) !== null
				? { membership_changed: booleanValue(payload.membership_changed) }
				: {}),
		},
		identity_metadata: {},
	}

	let chang
```
_Source file truncated after 12000 characters. / Исходный файл обрезан после 12000 символов._

### `frontend/src/integrations/whatsapp/queries/useWhatsappQuery.ts`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/integrations/whatsapp/queries/useWhatsappQuery.ts`
- Size bytes / Размер в байтах: `2115`
- Included characters / Включено символов: `2115`
- Truncated / Обрезано: `no`

```typescript
import { useQuery } from '@tanstack/vue-query'
import { computed, toValue, type MaybeRefOrGetter } from 'vue'
import {
  fetchWhatsappAccounts,
  fetchWhatsappAccountCapabilities,
  fetchWhatsappCapabilities,
  fetchWhatsappWebSessions,
} from '../api/whatsapp'
import type {
  WhatsappAccountSummary,
  WhatsappCapabilitiesResponse,
  WhatsappWebSession,
} from '../types/whatsapp'
import { whatsappQueryKeys } from './whatsappQueryKeys'

export { whatsappQueryKeys } from './whatsappQueryKeys'
export * from './useWhatsappRuntimeQuery'

export function useWhatsappCapabilitiesQuery() {
  return useQuery<WhatsappCapabilitiesResponse>({
    queryKey: whatsappQueryKeys.capabilities,
    queryFn: () => fetchWhatsappCapabilities()
  })
}

export function useWhatsappAccountsQuery(
  includeRemoved: MaybeRefOrGetter<boolean> = false
) {
  return useQuery<WhatsappAccountSummary[]>({
    queryKey: computed(() => [
      ...whatsappQueryKeys.accounts,
      toValue(includeRemoved) ? 'with-removed' : 'active',
    ]),
    queryFn: async () => {
      const response = await fetchWhatsappAccounts(toValue(includeRemoved))
      return response.items
    }
  })
}

export function useWhatsappAccountCapabilitiesQuery(
  accountId: MaybeRefOrGetter<string | null | undefined>
) {
  return useQuery<WhatsappCapabilitiesResponse | null>({
    queryKey: computed(() => [
      ...whatsappQueryKeys.accountCapabilities,
      toValue(accountId) ?? 'none',
    ]),
    queryFn: async () => {
      const value = toValue(accountId)
      if (!value) return null
      return fetchWhatsappAccountCapabilities(value)
    },
    enabled: computed(() => Boolean(toValue(accountId))),
  })
}

export function useWhatsappSessionsQuery(
  accountId: MaybeRefOrGetter<string | null | undefined>,
  limit = 50
) {
  return useQuery<WhatsappWebSession[]>({
    queryKey: computed(() => [
      ...whatsappQueryKeys.sessions,
      toValue(accountId) ?? 'all',
      limit,
    ]),
    queryFn: async () => {
      const res = await fetchWhatsappWebSessions(toValue(accountId) ?? undefined, limit)
      return res.items
    },
  })
}
```

### `frontend/src/integrations/whatsapp/queries/useWhatsappRuntimeQuery.ts`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/integrations/whatsapp/queries/useWhatsappRuntimeQuery.ts`
- Size bytes / Размер в байтах: `13761`
- Included characters / Включено символов: `12000`
- Truncated / Обрезано: `yes`

```typescript
import { useMutation, useQuery, useQueryClient } from '@tanstack/vue-query'
import { computed, toValue, type MaybeRefOrGetter } from 'vue'
import {
  deadLetterWhatsappProviderCommand,
  fetchWhatsappProviderCommands,
  fetchWhatsappSyncChats,
  fetchWhatsappSyncCalls,
  fetchWhatsappSyncContacts,
  fetchWhatsappSyncHistory,
  fetchWhatsappSyncMedia,
  fetchWhatsappSyncMembers,
  fetchWhatsappSyncPresence,
  fetchWhatsappSyncStatuses,
  fetchWhatsappRuntimeHealth,
  fetchWhatsappRuntimeStatus,
  publishWhatsappStatus,
  relinkWhatsappRuntime,
  retryWhatsappProviderCommand,
  rotateWhatsappRuntime,
  removeWhatsappRuntime,
  revokeWhatsappRuntime,
  setupWhatsappLiveAccount,
  startWhatsappPairCodeLink,
  startWhatsappQrLink,
  startWhatsappRuntime,
  stopWhatsappRuntime,
} from '../api/whatsapp'
import type {
  WhatsAppCallSyncItem,
  WhatsAppChatSyncItem,
  WhatsAppContactSyncItem,
  WhatsAppMediaSyncItem,
  WhatsAppMembersSyncItem,
  WhatsAppProviderCommand,
  WhatsAppPairCodeSession,
  WhatsAppPresenceSyncItem,
  WhatsappWebMessage,
  WhatsAppProviderCommandListResponse,
  WhatsAppQrLinkSession,
  WhatsAppRuntimeHealth,
  WhatsAppRuntimeRemoveResponse,
  WhatsAppRuntimeStatus,
  WhatsappLiveAccountSetupRequest,
  WhatsappWebAccountSetupResponse,
} from '../types/whatsapp'
import { whatsappQueryKeys } from './whatsappQueryKeys'

export function useWhatsappRuntimeStatusQuery(
  accountId: MaybeRefOrGetter<string | null | undefined>
) {
  return useQuery<WhatsAppRuntimeStatus | null>({
    queryKey: computed(() => [
      ...whatsappQueryKeys.runtimeStatus,
      toValue(accountId) ?? 'none',
    ]),
    queryFn: async () => {
      const value = toValue(accountId)
      if (!value) return null
      return fetchWhatsappRuntimeStatus(value)
    },
    enabled: computed(() => Boolean(toValue(accountId))),
  })
}

export function useWhatsappRuntimeHealthQuery(
  accountId: MaybeRefOrGetter<string | null | undefined>
) {
  return useQuery<WhatsAppRuntimeHealth | null>({
    queryKey: computed(() => [
      ...whatsappQueryKeys.runtimeHealth,
      toValue(accountId) ?? 'none',
    ]),
    queryFn: async () => {
      const value = toValue(accountId)
      if (!value) return null
      return fetchWhatsappRuntimeHealth(value)
    },
    enabled: computed(() => Boolean(toValue(accountId))),
  })
}

function invalidateWhatsappRuntime(queryClient: ReturnType<typeof useQueryClient>) {
  queryClient.invalidateQueries({ queryKey: whatsappQueryKeys.accounts })
  queryClient.invalidateQueries({ queryKey: whatsappQueryKeys.sessions })
  queryClient.invalidateQueries({ queryKey: whatsappQueryKeys.capabilities })
  queryClient.invalidateQueries({ queryKey: whatsappQueryKeys.accountCapabilities })
  queryClient.invalidateQueries({ queryKey: whatsappQueryKeys.runtimeStatus })
  queryClient.invalidateQueries({ queryKey: whatsappQueryKeys.runtimeHealth })
  queryClient.invalidateQueries({ queryKey: whatsappQueryKeys.commands })
  queryClient.invalidateQueries({ queryKey: whatsappQueryKeys.syncChats })
  queryClient.invalidateQueries({ queryKey: whatsappQueryKeys.syncHistory })
  queryClient.invalidateQueries({ queryKey: whatsappQueryKeys.syncMembers })
  queryClient.invalidateQueries({ queryKey: whatsappQueryKeys.syncStatuses })
  queryClient.invalidateQueries({ queryKey: whatsappQueryKeys.syncPresence })
  queryClient.invalidateQueries({ queryKey: whatsappQueryKeys.syncCalls })
  queryClient.invalidateQueries({ queryKey: whatsappQueryKeys.syncContacts })
  queryClient.invalidateQueries({ queryKey: whatsappQueryKeys.syncMedia })
}

export function useStartWhatsappRuntimeMutation() {
  const queryClient = useQueryClient()
  return useMutation({
    mutationFn: (request: { account_id: string }) => startWhatsappRuntime(request),
    onSuccess: () => invalidateWhatsappRuntime(queryClient),
  })
}

export function useStopWhatsappRuntimeMutation() {
  const queryClient = useQueryClient()
  return useMutation({
    mutationFn: (request: { account_id: string }) => stopWhatsappRuntime(request),
    onSuccess: () => invalidateWhatsappRuntime(queryClient),
  })
}

export function useRevokeWhatsappRuntimeMutation() {
  const queryClient = useQueryClient()
  return useMutation({
    mutationFn: (request: { account_id: string }) => revokeWhatsappRuntime(request),
    onSuccess: () => invalidateWhatsappRuntime(queryClient),
  })
}

export function useRelinkWhatsappRuntimeMutation() {
  const queryClient = useQueryClient()
  return useMutation({
    mutationFn: (request: { account_id: string }) => relinkWhatsappRuntime(request),
    onSuccess: () => invalidateWhatsappRuntime(queryClient),
  })
}

export function useRotateWhatsappRuntimeMutation() {
  const queryClient = useQueryClient()
  return useMutation({
    mutationFn: (request: { account_id: string }) => rotateWhatsappRuntime(request),
    onSuccess: () => invalidateWhatsappRuntime(queryClient),
  })
}

export function useRemoveWhatsappRuntimeMutation() {
  const queryClient = useQueryClient()
  return useMutation<WhatsAppRuntimeRemoveResponse, Error, { account_id: string }>({
    mutationFn: (request) => removeWhatsappRuntime(request),
    onSuccess: () => invalidateWhatsappRuntime(queryClient),
  })
}

export function useStartWhatsappQrLinkMutation() {
  const queryClient = useQueryClient()
  return useMutation<WhatsAppQrLinkSession, Error, { account_id: string }>({
    mutationFn: (request) => startWhatsappQrLink(request),
    onSuccess: () => invalidateWhatsappRuntime(queryClient),
  })
}

export function useStartWhatsappPairCodeLinkMutation() {
  const queryClient = useQueryClient()
  return useMutation<WhatsAppPairCodeSession, Error, { account_id: string; phone_number: string }>({
    mutationFn: (request) => startWhatsappPairCodeLink(request),
    onSuccess: () => invalidateWhatsappRuntime(queryClient),
  })
}

export function useSetupWhatsappLiveAccountMutation() {
  const queryClient = useQueryClient()
  return useMutation<WhatsappWebAccountSetupResponse, Error, WhatsappLiveAccountSetupRequest>({
    mutationFn: (request) => setupWhatsappLiveAccount(request),
    onSuccess: () => invalidateWhatsappRuntime(queryClient),
  })
}

export function useWhatsappProviderCommandsQuery(
  accountId: MaybeRefOrGetter<string | null | undefined>,
  limit = 25
) {
  return useQuery<WhatsAppProviderCommand[]>({
    queryKey: computed(() => [
      ...whatsappQueryKeys.commands,
      toValue(accountId) ?? 'none',
      limit,
    ]),
    queryFn: async () => {
      const value = toValue(accountId)
      if (!value) return []
      const response: WhatsAppProviderCommandListResponse =
        await fetchWhatsappProviderCommands({
          account_id: value,
          limit,
        })
      return response.items
    },
    enabled: computed(() => Boolean(toValue(accountId))),
  })
}

export function useWhatsappSyncChatsQuery(
  accountId: MaybeRefOrGetter<string | null | undefined>,
  limit = 12
) {
  return useQuery<WhatsAppChatSyncItem[]>({
    queryKey: computed(() => [
      ...whatsappQueryKeys.syncChats,
      toValue(accountId) ?? 'none',
      limit,
    ]),
    queryFn: async () => {
      const value = toValue(accountId)
      if (!value) return []
      const response = await fetchWhatsappSyncChats({
        account_id: value,
        limit,
      })
      return response.items
    },
    enabled: computed(() => Boolean(toValue(accountId))),
  })
}

export function useWhatsappSyncHistoryQuery(
  accountId: MaybeRefOrGetter<string | null | undefined>,
  providerChatId: MaybeRefOrGetter<string | null | undefined>,
  limit = 12
) {
  return useQuery<WhatsappWebMessage[]>({
    queryKey: computed(() => [
      ...whatsappQueryKeys.syncHistory,
      toValue(accountId) ?? 'none',
      toValue(providerChatId) ?? 'none',
      limit,
    ]),
    queryFn: async () => {
      const account = toValue(accountId)
      const providerChatIdValue = toValue(providerChatId)
      if (!account || !providerChatIdValue) return []
      const response = await fetchWhatsappSyncHistory({
        account_id: account,
        provider_chat_id: providerChatIdValue,
        limit,
      })
      return response.items
    },
    enabled: computed(() => Boolean(toValue(accountId) && toValue(providerChatId))),
  })
}

export function useWhatsappSyncMembersQuery(
  accountId: MaybeRefOrGetter<string | null | undefined>,
  providerChatId: MaybeRefOrGetter<string | null | undefined>,
  limit = 12
) {
  return useQuery<WhatsAppMembersSyncItem[]>({
    queryKey: computed(() => [
      ...whatsappQueryKeys.syncMembers,
      toValue(accountId) ?? 'none',
      toValue(providerChatId) ?? 'none',
      limit,
    ]),
    queryFn: async () => {
      const account = toValue(accountId)
      const providerChatIdValue = toValue(providerChatId)
      if (!account || !providerChatIdValue) return []
      const response = await fetchWhatsappSyncMembers({
        account_id: account,
        provider_chat_id: providerChatIdValue,
        limit,
      })
      return response.items
    },
    enabled: computed(() => Boolean(toValue(accountId) && toValue(providerChatId))),
  })
}

export function useWhatsappSyncPresenceQuery(
  accountId: MaybeRefOrGetter<string | null | undefined>,
  providerChatId: MaybeRefOrGetter<string | null | undefined> = null,
  limit = 12
) {
  return useQuery<WhatsAppPresenceSyncItem[]>({
    queryKey: computed(() => [
      ...whatsappQueryKeys.syncPresence,
      toValue(accountId) ?? 'none',
      toValue(providerChatId) ?? 'all',
      limit,
    ]),
    queryFn: async () => {
      const value = toValue(accountId)
      if (!value) return []
      const response = await fetchWhatsappSyncPresence({
        account_id: value,
        provider_chat_id: toValue(providerChatId) ?? undefined,
        limit,
      })
      return response.items
    },
    enabled: computed(() => Boolean(toValue(accountId))),
  })
}

export function useWhatsappSyncStatusesQuery(
  accountId: MaybeRefOrGetter<string | null | undefined>,
  limit = 12
) {
  return useQuery<WhatsappWebMessage[]>({
    queryKey: computed(() => [
      ...whatsappQueryKeys.syncStatuses,
      toValue(accountId) ?? 'none',
      limit,
    ]),
    queryFn: async () => {
      const value = toValue(accountId)
      if (!value) return []
      const response = await fetchWhatsappSyncStatuses({
        account_id: value,
        limit,
      })
      return response.items
    },
    enabled: computed(() => Boolean(toValue(accountId))),
  })
}

export function useWhatsappSyncCallsQuery(
  accountId: MaybeRefOrGetter<string | null | undefined>,
  providerChatId: MaybeRefOrGetter<string | null | undefined> = null,
  limit = 12
) {
  return useQuery<WhatsAppCallSyncItem[]>({
    queryKey: computed(() => [
      ...whatsappQueryKeys.syncCalls,
      toValue(accountId) ?? 'none',
      toValue(providerChatId) ?? 'all',
      limit,
    ]),
    queryFn: async () => {
      const value = toValue(accountId)
      if (!value) return []
      const response = await fetchWhatsappSyncCalls({
        account_id: value,
        provider_chat_id: toValue(providerChatId) ?? undefined,
        limit,
      })
      return response.items
    },
    enabled: computed(() => Boolean(toValue(accountId))),
  })
}

export function useWhatsappSyncContactsQuery(
  accountId: MaybeRefOrGetter<string | null | undefined>,
  limit = 12
) {
  return useQuery<WhatsAppContactSyncItem[]>({
    queryKey: computed(() => [
      ...whatsappQueryKeys.syncContacts,
      toValue(accountId) ?? 'none',
      limit,
    ]),
    queryFn: async () => {
      const value = toValue(accountId)
      if (!value) return []
      const response = await fetchWhatsappSyncContacts({
        account_id: value,
        limit,
      })
      return response.items
    },
    enabled: computed(() => Boolean(toValue(accountId))),
  })
}

export function useWhatsappSyncMediaQuery(
  accountId: MaybeRefOrGetter<string | null | undefined>,
  providerChatId: MaybeRefO
```
_Source file truncated after 12000 characters. / Исходный файл обрезан после 12000 символов._

### `frontend/src/integrations/whatsapp/queries/whatsappQueryKeys.ts`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/integrations/whatsapp/queries/whatsappQueryKeys.ts`
- Size bytes / Размер в байтах: `1179`
- Included characters / Включено символов: `1179`
- Truncated / Обрезано: `no`

```typescript
export const whatsappQueryKeys = {
  accounts: ['integrations', 'whatsapp', 'accounts'] as const,
  capabilities: ['integrations', 'whatsapp', 'capabilities'] as const,
  accountCapabilities: ['integrations', 'whatsapp', 'account-capabilities'] as const,
  sessions: ['integrations', 'whatsapp', 'sessions'] as const,
  runtimeStatus: ['integrations', 'whatsapp', 'runtime', 'status'] as const,
  runtimeHealth: ['integrations', 'whatsapp', 'runtime', 'health'] as const,
  commands: ['integrations', 'whatsapp', 'commands'] as const,
  syncChats: ['integrations', 'whatsapp', 'runtime', 'sync-chats'] as const,
  syncHistory: ['integrations', 'whatsapp', 'runtime', 'sync-history'] as const,
  syncMembers: ['integrations', 'whatsapp', 'runtime', 'sync-members'] as const,
  syncStatuses: ['integrations', 'whatsapp', 'runtime', 'sync-statuses'] as const,
  syncPresence: ['integrations', 'whatsapp', 'runtime', 'sync-presence'] as const,
  syncCalls: ['integrations', 'whatsapp', 'runtime', 'sync-calls'] as const,
  syncContacts: ['integrations', 'whatsapp', 'runtime', 'sync-contacts'] as const,
  syncMedia: ['integrations', 'whatsapp', 'runtime', 'sync-media'] as const,
}
```

### `frontend/src/integrations/whatsapp/stores/whatsapp.ts`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/integrations/whatsapp/stores/whatsapp.ts`
- Size bytes / Размер в байтах: `3941`
- Included characters / Включено символов: `3941`
- Truncated / Обрезано: `no`

```typescript
import { ref, computed } from 'vue'
import { defineStore } from 'pinia'
import type {
  WhatsappWebSession,
  WhatsappCapabilitiesResponse
} from '../types/whatsapp'

export interface WhatsappMessageForm {
  account_id: string
  provider_chat_id: string
  provider_message_id: string
  chat_title: string
  sender_id: string
  sender_display_name: string
  text: string
  import_batch_id: string
  occurred_at: string
  delivery_state: string
}

export const useWhatsappStore = defineStore('whatsapp-ui', () => {
  // Data state
  const whatsappSessions = ref<WhatsappWebSession[]>([])
  const whatsappCapabilities = ref<WhatsappCapabilitiesResponse | null>(null)

  // Selection state
  const selectedWhatsappSessionId = ref('')

  // UI state
  const whatsappError = ref('')
  const whatsappActionMessage = ref('')
  const isWhatsappLoading = ref(false)
  const isWhatsappActionSubmitting = ref(false)

  // Fixture message form
  const whatsappMessageForm = ref<WhatsappMessageForm>({
    account_id: 'whatsapp-primary',
    provider_chat_id: 'wa-fixture-chat-1',
    provider_message_id: '',
    chat_title: 'WhatsApp Fixture Chat',
    sender_id: 'wa-fixture-sender-1',
    sender_display_name: 'Alice',
    text: 'WhatsApp fixture WhatsApp Web message for local memory and graph recall.',
    import_batch_id: 'whatsapp-web-fixture-ui',
    occurred_at: new Date().toISOString(),
    delivery_state: 'received'
  })

  // Derived
  const selectedWhatsappSession = computed(() =>
    whatsappSessions.value.find((s) => s.session_id === selectedWhatsappSessionId.value) ??
    whatsappSessions.value[0] ??
    null
  )

  const whatsappClosureCapabilities = computed(() =>
    whatsappCapabilities.value?.capabilities.filter((c) => c.closure_gate) ?? []
  )

  const whatsappBlockedCapabilities = computed(() =>
    whatsappCapabilities.value?.capabilities.filter((c) => c.status === 'blocked') ?? []
  )

  // Actions
  function setWhatsappData(data: {
    sessions: WhatsappWebSession[]
    capabilities: WhatsappCapabilitiesResponse | null
    selectedSessionId: string
    error: string
  }) {
    whatsappSessions.value = data.sessions
    whatsappCapabilities.value = data.capabilities
    selectedWhatsappSessionId.value = data.selectedSessionId
    whatsappError.value = data.error
  }

  function selectWhatsappSession(session: WhatsappWebSession) {
    selectedWhatsappSessionId.value = session.session_id
    whatsappMessageForm.value = {
      ...whatsappMessageForm.value,
      account_id: session.account_id
    }
  }

  function setWhatsappLoading(loading: boolean) {
    isWhatsappLoading.value = loading
  }

  function setWhatsappActionSubmitting(submitting: boolean) {
    isWhatsappActionSubmitting.value = submitting
  }

  function setWhatsappError(error: string) {
    whatsappError.value = error
  }

  function setWhatsappActionMessage(message: string) {
    whatsappActionMessage.value = message
  }

  function resetWhatsappMessageForm() {
    whatsappMessageForm.value = {
      account_id: 'whatsapp-primary',
      provider_chat_id: 'wa-fixture-chat-1',
      provider_message_id: '',
      chat_title: 'WhatsApp Fixture Chat',
      sender_id: 'wa-fixture-sender-1',
      sender_display_name: 'Alice',
      text: '',
      import_batch_id: 'whatsapp-web-fixture-ui',
      occurred_at: new Date().toISOString(),
      delivery_state: 'received'
    }
  }

  return {
    // State
    whatsappSessions,
    whatsappCapabilities,
    selectedWhatsappSessionId,
    whatsappError,
    whatsappActionMessage,
    isWhatsappLoading,
    isWhatsappActionSubmitting,
    whatsappMessageForm,
    // Derived
    selectedWhatsappSession,
    whatsappClosureCapabilities,
    whatsappBlockedCapabilities,
    // Actions
    setWhatsappData,
    selectWhatsappSession,
    setWhatsappLoading,
    setWhatsappActionSubmitting,
    setWhatsappError,
    setWhatsappActionMessage,
    resetWhatsappMessageForm
  }
})
```

### `frontend/src/integrations/whatsapp/types/whatsapp.ts`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/integrations/whatsapp/types/whatsapp.ts`
- Size bytes / Размер в байтах: `68`
- Included characters / Включено символов: `68`
- Truncated / Обрезано: `no`

```typescript
export type * from '../../../shared/communications/types/whatsapp'
```

### `frontend/src/integrations/whatsapp/views/WhatsAppRuntimePanel.boundary.test.ts`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/integrations/whatsapp/views/WhatsAppRuntimePanel.boundary.test.ts`
- Size bytes / Размер в байтах: `3664`
- Included characters / Включено символов: `3664`
- Truncated / Обрезано: `no`

```typescript
import { readFileSync } from 'node:fs'
import { describe, expect, it } from 'vitest'

function readSource(relativePath: string): string {
  return readFileSync(new URL(relativePath, import.meta.url), 'utf8')
}

describe('WhatsAppRuntimePanel boundary', () => {
  it('surfaces projected sync snapshots for chats, history, members, presence, calls and media through query wiring', () => {
    const source = readSource('./WhatsAppRuntimePanel.vue')
    const snapshotsSource = readSource('../components/WhatsAppRuntimeSnapshots.vue')

    expect(source).toContain('useWhatsappSyncChatsQuery')
    expect(source).toContain('useWhatsappSyncHistoryQuery')
    expect(source).toContain('useWhatsappSyncMembersQuery')
    expect(source).toContain('useWhatsappSyncPresenceQuery(selectedAccountId, selectedSyncChatIdResolved, 8)')
    expect(source).toContain('useWhatsappSyncCallsQuery(selectedAccountId, selectedSyncChatIdResolved, 8)')
    expect(source).toContain('useWhatsappSyncMediaQuery(selectedAccountId, selectedSyncChatIdResolved, 8)')
    expect(snapshotsSource).toContain("Chats")
    expect(snapshotsSource).toContain("History")
    expect(snapshotsSource).toContain("Members")
    expect(snapshotsSource).toContain("Select a synced chat to inspect recent history.")
    expect(snapshotsSource).toContain("Select a synced chat to inspect roster members.")
    expect(snapshotsSource).toContain("No projected presence for the selected synced chat yet.")
    expect(snapshotsSource).toContain("No projected calls for the selected synced chat yet.")
    expect(snapshotsSource).toContain("No projected media for the selected synced chat yet.")
    expect(source).toContain('selectedSyncChatId')
    expect(snapshotsSource).toContain('snapshot-select')
  })

  it('exposes rotate as an owner-visible runtime lifecycle control', () => {
    const source = readSource('./WhatsAppRuntimePanel.vue')
    const controlSource = readSource('../components/WhatsAppRuntimeControl.vue')

    expect(source).toContain('useRotateWhatsappRuntimeMutation')
    expect(controlSource).toContain("emit('set-runtime-state', 'rotate')")
    expect(controlSource).toContain("Rotate")
  })

  it('exposes the owner-visible WebView companion action through the typed Tauri bridge only', () => {
    const source = readSource('./WhatsAppRuntimePanel.vue')
    const controlSource = readSource('../components/WhatsAppRuntimeControl.vue')

    expect(source).toContain("import { openWhatsappWebCompanion } from '../api/whatsappCompanion'")
    expect(source).toContain('async function openVisibleWebCompanion()')
    expect(source).toContain("selectedRuntimeProviderShape.value === 'whatsapp_web_companion'")
    expect(source).toContain("openWhatsappWebCompanion(accountId)")
    expect(controlSource).toContain("Open Companion")
    expect(controlSource).toContain('companionOpenManifest.event_extractor.relay_channel')
    expect(source).not.toContain('window.fetch')
    expect(source).not.toContain('globalThis.fetch')
    expect(source).not.toContain('/api/v1/integrations/whatsapp/runtime-bridge')
    expect(source).not.toContain('ApiClient')
  })

  it('renders nested runtime health diagnostics from backend checks', () => {
    const source = readSource('./WhatsAppRuntimePanel.vue')
    const controlSource = readSource('../components/WhatsAppRuntimeControl.vue')

    expect(source).toContain('runtimeHealthChecks')
    expect(controlSource).toContain('Health diagnostics')
    expect(controlSource).toContain('runtimeHealthCheckStatus')
    expect(controlSource).toContain('runtimeHealthCheckDetail')
    expect(controlSource).toContain('runtimeHealth?.checked_at')
  })
})
```

### `frontend/src/integrations/whatsapp/views/WhatsAppRuntimePanel.helpers.ts`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/integrations/whatsapp/views/WhatsAppRuntimePanel.helpers.ts`
- Size bytes / Размер в байтах: `4729`
- Included characters / Включено символов: `4726`
- Truncated / Обрезано: `no`

```typescript
import type {
	WhatsAppCallSyncItem,
	WhatsAppChatSyncItem,
	WhatsAppContactSyncItem,
	WhatsAppMediaSyncItem,
	WhatsAppMembersSyncItem,
	WhatsAppPresenceSyncItem,
	WhatsAppProviderCommand,
	WhatsappWebMessage,
} from '../../../shared/communications/types/whatsapp'

export function commandStatusTone(command: WhatsAppProviderCommand): string {
	if (command.status === 'completed') return 'available'
	if (command.status === 'executing' || command.status === 'queued' || command.status === 'retrying') return 'degraded'
	return 'blocked'
}

export function canRetryCommand(command: WhatsAppProviderCommand): boolean {
	return ['failed', 'dead_letter', 'retrying', 'cancelled'].includes(command.status)
}

export function canDeadLetterCommand(command: WhatsAppProviderCommand): boolean {
	return !['completed', 'dead_letter'].includes(command.status)
}

export function commandTimestamp(command: WhatsAppProviderCommand): string {
	const value =
		command.completed_at
		?? command.provider_observed_at
		?? command.last_attempt_at
		?? command.updated_at
	if (!value) return '-'
	const date = new Date(value)
	return Number.isNaN(date.getTime())
		? value
		: new Intl.DateTimeFormat('en', {
			month: 'short',
			day: 'numeric',
			hour: '2-digit',
			minute: '2-digit',
		}).format(date)
}

export function providerTargetLabel(command: WhatsAppProviderCommand): string {
	return command.provider_message_id
		? `${command.provider_chat_id} · ${command.provider_message_id}`
		: command.provider_chat_id
}

export function snapshotTimestamp(value: string | null | undefined): string {
	if (!value) return '-'
	const date = new Date(value)
	return Number.isNaN(date.getTime())
		? value
		: new Intl.DateTimeFormat('en', {
			month: 'short',
			day: 'numeric',
			hour: '2-digit',
			minute: '2-digit',
		}).format(date)
}

export function runtimeHealthCheckStatus(value: unknown): string {
	if (isRecordValue(value)) {
		if (typeof value.status === 'string' && value.status.trim()) return value.status
		if (typeof value.healthy === 'boolean') return value.healthy ? 'healthy' : 'degraded'
	}
	if (typeof value === 'string' && value.trim()) return value
	return '-'
}

export function runtimeHealthCheckDetail(value: unknown): string {
	if (isRecordValue(value)) {
		const reason = value.reason
		if (typeof reason === 'string' && reason.trim()) return reason
		const error = value.error
		if (typeof error === 'string' && error.trim()) return error
		const details = value.details
		if (typeof details === 'string' && details.trim()) return details
	}
	if (typeof value === 'boolean') return value ? 'ok' : 'blocked'
	if (typeof value === 'number') return String(value)
	return ''
}

function isRecordValue(value: unknown): value is Record<string, unknown> {
	return typeof value === 'object' && value !== null && !Array.isArray(value)
}

export function presenceLabel(item: WhatsAppPresenceSyncItem): string {
	return item.display_name ?? item.address ?? item.provider_identity_id
}

export function chatLabel(item: WhatsAppChatSyncItem): string {
	return item.title || item.provider_chat_id
}

export function chatMeta(item: WhatsAppChatSyncItem): string {
	const parts: string[] = []
	if (item.chat_kind) parts.push(item.chat_kind)
	if (typeof item.unread_count === 'number' && item.unread_count > 0) parts.push(`unread ${item.unread_count}`)
	if (typeof item.participant_count === 'number') parts.push(`${item.participant_count} members`)
	if (item.is_archived) parts.push('archived')
	if (item.is_pinned) parts.push('pinned')
	if (item.is_muted) parts.push('muted')
	return parts.join(' · ') || item.provider_chat_id
}

export function historyLabel(item: WhatsappWebMessage): string {
	return item.sender_display_name ?? item.sender ?? item.provider_message_id
}

export function statusLabel(item: WhatsappWebMessage): string {
	return item.sender_display_name ?? item.sender ?? item.provider_message_id
}

export function statusPreview(item: WhatsappWebMessage): string {
	const text = item.text?.trim()
	if (text) return text
	const mediaType = typeof item.metadata?.media_type === 'string' ? item.metadata.media_type : null
	return mediaType ? `[${mediaType}]` : '-'
}

export function callLabel(item: WhatsAppCallSyncItem): string {
	return `${item.direction} · ${item.call_state}`
}

export function contactLabel(item: WhatsAppContactSyncItem): string {
	return item.display_name ?? item.push_name ?? item.address ?? item.provider_identity_id
}

export function mediaLabel(item: WhatsAppMediaSyncItem): string {
	return item.filename ?? item.provider_attachment_id
}

export function memberLabel(item: WhatsAppMembersSyncItem): string {
	return item.sender_display_name ?? item.address ?? item.provider_member_id
}
```

### `frontend/src/integrations/yandexTelemost/api/yandexTelemost.ts`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/integrations/yandexTelemost/api/yandexTelemost.ts`
- Size bytes / Размер в байтах: `6552`
- Included characters / Включено символов: `6552`
- Truncated / Обрезано: `no`

```typescript
import { invoke } from '@tauri-apps/api/core'
import { ApiClient } from '../../../platform/api/ApiClient'
import type {
  YandexTelemostAccountListResponse,
  YandexTelemostAccountSetupRequest,
  YandexTelemostAccountSetupResponse,
  YandexTelemostCapabilitiesResponse,
  YandexTelemostCohostPage,
  YandexTelemostCompanionManifest,
  YandexTelemostCompanionOpenRequest,
  YandexTelemostConferenceCreateRequest,
  YandexTelemostConferenceOperationResponse,
  YandexTelemostConferenceUpdateRequest,
  YandexTelemostConferenceWebviewManifest,
  YandexTelemostRecordingIntentResponse,
  YandexTelemostRecordingBridgeRequest,
  YandexTelemostRecordingBridgeResponse,
  YandexTelemostRecordingSession,
  YandexTelemostRecordingStopReceipt,
  YandexTelemostRuntimeStatus,
  YandexTelemostWebviewManifestRequest,
} from '../types/yandexTelemost'

export async function fetchYandexTelemostCapabilities(): Promise<YandexTelemostCapabilitiesResponse> {
  return ApiClient.instance.get<YandexTelemostCapabilitiesResponse>(
    '/api/v1/integrations/yandex-telemost/capabilities',
    'Yandex Telemost capabilities request failed'
  )
}

export async function fetchYandexTelemostAccounts(
  includeRemoved = false
): Promise<YandexTelemostAccountListResponse> {
  const params = new URLSearchParams()
  if (includeRemoved) params.set('include_removed', 'true')
  const suffix = params.toString() ? `?${params.toString()}` : ''
  return ApiClient.instance.get<YandexTelemostAccountListResponse>(
    `/api/v1/integrations/yandex-telemost/accounts${suffix}`,
    'Yandex Telemost accounts request failed'
  )
}

export async function setupYandexTelemostAccount(
  request: YandexTelemostAccountSetupRequest
): Promise<YandexTelemostAccountSetupResponse> {
  return ApiClient.instance.post<YandexTelemostAccountSetupResponse>(
    '/api/v1/integrations/yandex-telemost/accounts',
    request,
    'Yandex Telemost account setup failed'
  )
}

export async function fetchYandexTelemostRuntimeStatus(
  accountId: string
): Promise<YandexTelemostRuntimeStatus> {
  const params = new URLSearchParams({ account_id: accountId.trim() })
  return ApiClient.instance.get<YandexTelemostRuntimeStatus>(
    `/api/v1/integrations/yandex-telemost/runtime/status?${params.toString()}`,
    'Yandex Telemost runtime status request failed'
  )
}

export async function createYandexTelemostConference(
  request: YandexTelemostConferenceCreateRequest
): Promise<YandexTelemostConferenceOperationResponse> {
  const { account_id, ...body } = request
  return ApiClient.instance.post<YandexTelemostConferenceOperationResponse>(
    '/api/v1/integrations/yandex-telemost/conferences',
    { account_id, body },
    'Yandex Telemost conference creation failed'
  )
}

export async function readYandexTelemostConference(
  accountId: string,
  conferenceId: string
): Promise<YandexTelemostConferenceOperationResponse> {
  return ApiClient.instance.get<YandexTelemostConferenceOperationResponse>(
    `/api/v1/integrations/yandex-telemost/conferences/${encodeURIComponent(accountId.trim())}/${encodeURIComponent(conferenceId.trim())}`,
    'Yandex Telemost conference read failed'
  )
}

export async function updateYandexTelemostConference(
  accountId: string,
  conferenceId: string,
  request: YandexTelemostConferenceUpdateRequest
): Promise<YandexTelemostConferenceOperationResponse> {
  return ApiClient.instance.patch<YandexTelemostConferenceOperationResponse>(
    `/api/v1/integrations/yandex-telemost/conferences/${encodeURIComponent(accountId.trim())}/${encodeURIComponent(conferenceId.trim())}`,
    request,
    'Yandex Telemost conference update failed'
  )
}

export async function fetchYandexTelemostCohosts(
  accountId: string,
  conferenceId: string,
  offset?: number | null,
  limit?: number | null
): Promise<YandexTelemostCohostPage> {
  const params = new URLSearchParams()
  if (typeof offset === 'number') params.set('offset', String(offset))
  if (limit) params.set('limit', String(limit))
  const suffix = params.toString() ? `?${params.toString()}` : ''
  return ApiClient.instance.get<YandexTelemostCohostPage>(
    `/api/v1/integrations/yandex-telemost/conferences/${encodeURIComponent(accountId.trim())}/${encodeURIComponent(conferenceId.trim())}/cohosts${suffix}`,
    'Yandex Telemost cohosts request failed'
  )
}

export async function fetchYandexTelemostWebviewManifest(
  request: YandexTelemostWebviewManifestRequest
): Promise<YandexTelemostConferenceWebviewManifest> {
  return ApiClient.instance.post<YandexTelemostConferenceWebviewManifest>(
    '/api/v1/integrations/yandex-telemost/webview/manifest',
    request,
    'Yandex Telemost webview manifest request failed'
  )
}

export async function fetchYandexTelemostRecordingIntent(
  request: YandexTelemostWebviewManifestRequest
): Promise<YandexTelemostRecordingIntentResponse> {
  return ApiClient.instance.post<YandexTelemostRecordingIntentResponse>(
    '/api/v1/integrations/yandex-telemost/recording/intent',
    request,
    'Yandex Telemost recording intent request failed'
  )
}

export async function openYandexTelemostCompanion(
  request: YandexTelemostCompanionOpenRequest
): Promise<YandexTelemostCompanionManifest> {
  return invoke<YandexTelemostCompanionManifest>('open_yandex_telemost_companion', { request })
}

export async function prepareYandexTelemostAudioDevice(request: {
  device_name?: string | null
}): Promise<Record<string, unknown>> {
  return invoke<Record<string, unknown>>('yandex_telemost_prepare_audio_device', { request })
}

export async function startYandexTelemostRecording(request: {
  account_id: string
  join_url: string
  conference_id?: string | null
  window_label?: string | null
  audio_input?: string | null
  consent_attested: boolean
}): Promise<YandexTelemostRecordingSession> {
  return invoke<YandexTelemostRecordingSession>('yandex_telemost_recording_start', { request })
}

export async function stopYandexTelemostRecording(
  recordingSessionId: string
): Promise<YandexTelemostRecordingStopReceipt> {
  return invoke<YandexTelemostRecordingStopReceipt>('yandex_telemost_recording_stop', {
    request: { recording_session_id: recordingSessionId },
  })
}

export async function completeYandexTelemostRecording(
  request: YandexTelemostRecordingBridgeRequest
): Promise<YandexTelemostRecordingBridgeResponse> {
  return ApiClient.instance.post<YandexTelemostRecordingBridgeResponse>(
    '/api/v1/integrations/yandex-telemost/runtime-bridge/recordings',
    request,
    'Yandex Telemost recording completion bridge failed'
  )
}
```

### `frontend/src/integrations/yandexTelemost/queries/useYandexTelemostRuntimeQuery.ts`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/integrations/yandexTelemost/queries/useYandexTelemostRuntimeQuery.ts`
- Size bytes / Размер в байтах: `2024`
- Included characters / Включено символов: `2024`
- Truncated / Обрезано: `no`

```typescript
import { useMutation, useQuery, useQueryClient } from '@tanstack/vue-query'
import { computed, toValue, type MaybeRefOrGetter } from 'vue'
import {
  fetchYandexTelemostAccounts,
  fetchYandexTelemostCapabilities,
  fetchYandexTelemostRuntimeStatus,
  setupYandexTelemostAccount,
} from '../api/yandexTelemost'
import type {
  YandexTelemostAccount,
  YandexTelemostAccountSetupRequest,
  YandexTelemostAccountSetupResponse,
  YandexTelemostCapabilitiesResponse,
  YandexTelemostRuntimeStatus,
} from '../types/yandexTelemost'
import { yandexTelemostQueryKeys } from './yandexTelemostQueryKeys'

export function useYandexTelemostCapabilitiesQuery() {
  return useQuery<YandexTelemostCapabilitiesResponse>({
    queryKey: yandexTelemostQueryKeys.capabilities,
    queryFn: fetchYandexTelemostCapabilities,
  })
}

export function useYandexTelemostAccountsQuery(includeRemoved: MaybeRefOrGetter<boolean> = false) {
  return useQuery<YandexTelemostAccount[]>({
    queryKey: computed(() => [...yandexTelemostQueryKeys.accounts, toValue(includeRemoved)]),
    queryFn: async () => (await fetchYandexTelemostAccounts(toValue(includeRemoved))).items,
  })
}

export function useYandexTelemostRuntimeStatusQuery(
  accountId: MaybeRefOrGetter<string | null | undefined>
) {
  return useQuery<YandexTelemostRuntimeStatus | null>({
    queryKey: computed(() => [...yandexTelemostQueryKeys.runtimeStatus, toValue(accountId) ?? 'none']),
    queryFn: async () => {
      const value = toValue(accountId)
      if (!value) return null
      return fetchYandexTelemostRuntimeStatus(value)
    },
    enabled: computed(() => Boolean(toValue(accountId))),
  })
}

export function useSetupYandexTelemostAccountMutation() {
  const queryClient = useQueryClient()
  return useMutation<YandexTelemostAccountSetupResponse, Error, YandexTelemostAccountSetupRequest>({
    mutationFn: setupYandexTelemostAccount,
    onSuccess: async () => {
      await queryClient.invalidateQueries({ queryKey: yandexTelemostQueryKeys.accounts })
    },
  })
}
```

### `frontend/src/integrations/yandexTelemost/queries/yandexTelemostQueryKeys.ts`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/integrations/yandexTelemost/queries/yandexTelemostQueryKeys.ts`
- Size bytes / Размер в байтах: `272`
- Included characters / Включено символов: `272`
- Truncated / Обрезано: `no`

```typescript
export const yandexTelemostQueryKeys = {
  capabilities: ['integrations', 'yandex-telemost', 'capabilities'] as const,
  accounts: ['integrations', 'yandex-telemost', 'accounts'] as const,
  runtimeStatus: ['integrations', 'yandex-telemost', 'runtime-status'] as const,
}
```
