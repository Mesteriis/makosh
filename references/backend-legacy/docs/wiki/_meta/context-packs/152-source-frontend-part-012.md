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

- Chunk ID / ID чанка: `152-source-frontend-part-012`
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

### `frontend/src/integrations/telegram/api/telegramMediaUpload.test.ts`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/integrations/telegram/api/telegramMediaUpload.test.ts`
- Size bytes / Размер в байтах: `1464`
- Included characters / Включено символов: `1464`
- Truncated / Обрезано: `no`

```typescript
import { afterEach, describe, expect, it, vi } from 'vitest'
import { ApiClient } from '../../../platform/api/ApiClient'
import { uploadTelegramMedia } from './telegramMediaUpload'

describe('telegramMediaUpload api', () => {
  afterEach(() => {
    vi.unstubAllGlobals()
    ApiClient.resetForTests()
  })

  it('queues Telegram media upload through the provider command endpoint', async () => {
    ApiClient.init('http://127.0.0.1:8080', 'test-secret')
    const fetchMock = vi.fn().mockResolvedValue(
      new Response(
        JSON.stringify({
          command_id: 'tcmd-media-1',
          account_id: 'telegram-1',
          provider_chat_id: '123',
          attachment_id: 'att-import:1',
          blob_id: 'blob:1',
          media_type: 'document',
          status: 'queued',
          reconciliation_status: 'not_observed'
        }),
        { status: 200 }
      )
    )
    vi.stubGlobal('fetch', fetchMock)

    const response = await uploadTelegramMedia({
      account_id: 'telegram-1',
      provider_chat_id: '123',
      attachment_id: 'att-import:1',
      media_type: 'document'
    })

    expect(response.status).toBe('queued')
    const [url, init] = fetchMock.mock.calls[0]
    expect(url).toContain('/api/v1/integrations/telegram/provider-media/upload')
    expect(init.method).toBe('POST')
    expect(JSON.parse(init.body as string)).toMatchObject({
      attachment_id: 'att-import:1',
      media_type: 'document'
    })
  })
})
```

### `frontend/src/integrations/telegram/api/telegramMediaUpload.ts`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/integrations/telegram/api/telegramMediaUpload.ts`
- Size bytes / Размер в байтах: `982`
- Included characters / Включено символов: `982`
- Truncated / Обрезано: `no`

```typescript
import { ApiClient } from '../../../platform/api/ApiClient'

export type TelegramMediaUploadRequest = {
  command_id?: string
  account_id: string
  provider_chat_id: string
  attachment_id?: string
  blob_id?: string
  media_type: TelegramMediaUploadKind
  caption?: string
  filename?: string
}

export type TelegramMediaUploadResponse = {
  command_id: string
  account_id: string
  provider_chat_id: string
  attachment_id?: string | null
  blob_id: string
  media_type: TelegramMediaUploadKind
  status: string
  reconciliation_status: string
}

export type TelegramMediaUploadKind =
  | 'photo'
  | 'video'
  | 'document'
  | 'audio'
  | 'voice'
  | 'sticker'
  | 'animation'

export async function uploadTelegramMedia(
  request: TelegramMediaUploadRequest
): Promise<TelegramMediaUploadResponse> {
  return ApiClient.instance.post<TelegramMediaUploadResponse>(
    '/api/v1/integrations/telegram/provider-media/upload',
    request,
    'Telegram media upload failed'
  )
}
```

### `frontend/src/integrations/telegram/api/telegramQr.test.ts`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/integrations/telegram/api/telegramQr.test.ts`
- Size bytes / Размер в байтах: `3343`
- Included characters / Включено символов: `3343`
- Truncated / Обрезано: `no`

```typescript
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { ApiClient } from '../../../platform/api/ApiClient'
import {
  cancelTelegramQrLogin,
  getTelegramQrLoginStatus,
  startTelegramQrLogin,
  submitTelegramQrPassword,
} from './telegram'

describe('telegram QR login API', () => {
  beforeEach(() => {
    ApiClient.resetForTests()
    ApiClient.init('http://127.0.0.1:8080', 'test-secret')
  })

  afterEach(() => {
    vi.unstubAllGlobals()
    ApiClient.resetForTests()
  })

  it('calls start, status, password and cancel QR routes', async () => {
    const fetchMock = vi
      .fn()
      .mockResolvedValueOnce(
        new Response(JSON.stringify({ setup_id: 'qr-1', account_id: 'acc-1', status: 'waiting_qr_scan', qr_link: null, qr_svg: '<svg />', telegram_user_id: null, telegram_username: null, suggested_account_id: null, suggested_display_name: null, suggested_external_account_id: null, expires_at: null, poll_after_ms: 1500, message: null }), {
          status: 200,
          headers: { 'Content-Type': 'application/json' },
        })
      )
      .mockResolvedValueOnce(
        new Response(JSON.stringify({ setup_id: 'qr-1', account_id: 'acc-1', status: 'waiting_password', qr_link: null, qr_svg: null, telegram_user_id: null, telegram_username: null, suggested_account_id: null, suggested_display_name: null, suggested_external_account_id: null, expires_at: null, poll_after_ms: 1500, message: null }), {
          status: 200,
          headers: { 'Content-Type': 'application/json' },
        })
      )
      .mockResolvedValueOnce(
        new Response(JSON.stringify({ setup_id: 'qr-1', account_id: 'acc-1', status: 'ready', qr_link: null, qr_svg: null, telegram_user_id: '123', telegram_username: 'demo', suggested_account_id: 'acc-1-ready', suggested_display_name: 'Demo', suggested_external_account_id: 'telegram:123', expires_at: null, poll_after_ms: 1500, message: null }), {
          status: 200,
          headers: { 'Content-Type': 'application/json' },
        })
      )
      .mockResolvedValueOnce(
        new Response(JSON.stringify({ setup_id: 'qr-1', cancelled: true }), {
          status: 200,
          headers: { 'Content-Type': 'application/json' },
        })
      )
    vi.stubGlobal('fetch', fetchMock)

    await startTelegramQrLogin({
      account_id: 'acc-1',
      display_name: 'Demo',
      external_account_id: 'telegram:123',
      tdlib_data_path: '/tmp/telegram-demo',
      transcription_enabled: false,
    })
    await getTelegramQrLoginStatus('qr-1')
    await submitTelegramQrPassword('qr-1', { password: 'secret' })
    await cancelTelegramQrLogin('qr-1')

    expect(fetchMock).toHaveBeenCalledTimes(4)
    expect(fetchMock.mock.calls[0][0]).toContain('/api/v1/integrations/telegram/login/qr/start')
    expect(fetchMock.mock.calls[1][0]).toContain('/api/v1/integrations/telegram/login/qr/qr-1')
    expect(fetchMock.mock.calls[2][0]).toContain('/api/v1/integrations/telegram/login/qr/qr-1/password')
    expect(fetchMock.mock.calls[3][0]).toContain('/api/v1/integrations/telegram/login/qr/qr-1')
    expect(fetchMock.mock.calls[0][1].method).toBe('POST')
    expect(fetchMock.mock.calls[1][1].method).toBe('GET')
    expect(fetchMock.mock.calls[2][1].method).toBe('POST')
    expect(fetchMock.mock.calls[3][1].method).toBe('DELETE')
  })
})
```

### `frontend/src/integrations/telegram/api/telegramSearch.test.ts`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/integrations/telegram/api/telegramSearch.test.ts`
- Size bytes / Размер в байтах: `1529`
- Included characters / Включено символов: `1529`
- Truncated / Обрезано: `no`

```typescript
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { ApiClient } from '../../../platform/api/ApiClient'
import {
  searchTelegramProviderMessages,
} from './telegramSearch'

describe('telegram search API', () => {
  beforeEach(() => {
    ApiClient.resetForTests()
    ApiClient.init('http://127.0.0.1:8080', 'test-secret')
  })

  afterEach(() => {
    vi.unstubAllGlobals()
    ApiClient.resetForTests()
  })

  it('builds Telegram provider search trigger requests with required account scope', async () => {
    const fetchMock = vi.fn().mockResolvedValue(
      new Response(JSON.stringify({
        account_id: 'telegram-account-1',
        provider_chat_id: 'chat-42',
        query: 'project alpha',
        limit: 25,
        status: 'queued',
        error: null,
      }), {
        status: 200,
        headers: { 'Content-Type': 'application/json' },
      })
    )
    vi.stubGlobal('fetch', fetchMock)

    await searchTelegramProviderMessages({
      q: 'project alpha',
      account_id: 'telegram-account-1',
      provider_chat_id: 'chat-42',
      limit: 25,
    })

    expect(fetchMock).toHaveBeenCalledOnce()
    const [url, init] = fetchMock.mock.calls[0]
    expect(url).toContain('/api/v1/integrations/telegram/provider-search')
    expect(init.method).toBe('POST')
    const body = JSON.parse(init.body as string)
    expect(body).toEqual({
      q: 'project alpha',
      account_id: 'telegram-account-1',
      provider_chat_id: 'chat-42',
      limit: 25,
    })
  })
})
```

### `frontend/src/integrations/telegram/api/telegramSearch.ts`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/integrations/telegram/api/telegramSearch.ts`
- Size bytes / Размер в байтах: `796`
- Included characters / Включено символов: `796`
- Truncated / Обрезано: `no`

```typescript
import { ApiClient } from '../../../platform/api/ApiClient'

export type TelegramProviderSearchCommandResponse = {
  account_id: string
  provider_chat_id?: string | null
  query: string
  limit: number
  status: string
  error?: string | null
}

export async function searchTelegramProviderMessages(params: {
  q: string
  account_id: string
  provider_chat_id?: string
  limit?: number
}): Promise<TelegramProviderSearchCommandResponse> {
  const body = {
    q: params.q.trim(),
    account_id: params.account_id.trim(),
    provider_chat_id: params.provider_chat_id?.trim(),
    limit: params.limit,
  }
  return ApiClient.instance.post<TelegramProviderSearchCommandResponse>(
    '/api/v1/integrations/telegram/provider-search',
    body,
    'Telegram provider search trigger failed'
  )
}
```

### `frontend/src/integrations/telegram/components/TelegramAccountManager.boundary.test.ts`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/integrations/telegram/components/TelegramAccountManager.boundary.test.ts`
- Size bytes / Размер в байтах: `912`
- Included characters / Включено символов: `912`
- Truncated / Обрезано: `no`

```typescript
import { describe, expect, it } from 'vitest'
import { readFileSync } from 'node:fs'

describe('TelegramAccountManager boundary', () => {
  it('uses vee-validate form state together with account lifecycle query mutations', () => {
    const source = readFileSync(new URL('./TelegramAccountManager.vue', import.meta.url), 'utf8')

    expect(source).toContain("from 'vee-validate'")
    expect(source).toContain('useTelegramAccountsQuery')
    expect(source).toContain('useSetupTelegramAccountMutation')
    expect(source).toContain('useLogoutTelegramAccountMutation')
    expect(source).toContain('useRemoveTelegramAccountMutation')
    expect(source).toContain('telegramAccountSetupSchema')
    expect(source).toContain('TelegramCapabilityMatrix')
    expect(source).toContain('TelegramQrLoginPanel')
    expect(source).toContain('setFieldValue')
    expect(source).toContain('props.selectedAccountId')
  })
})
```

### `frontend/src/integrations/telegram/components/TelegramCallTranscriptPanel.boundary.test.ts`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/integrations/telegram/components/TelegramCallTranscriptPanel.boundary.test.ts`
- Size bytes / Размер в байтах: `651`
- Included characters / Включено символов: `651`
- Truncated / Обрезано: `no`

```typescript
import { describe, expect, it } from 'vitest'
import { readFileSync } from 'node:fs'
import { resolve } from 'node:path'

describe('TelegramCallTranscriptPanel boundary', () => {
  it('loads transcript evidence through the query layer instead of inline fetch', () => {
    const source = readFileSync(
      resolve('src/integrations/telegram/components/TelegramCallTranscriptPanel.vue'),
      'utf8'
    )

    expect(source).toContain('useTelegramCallTranscriptQuery')
    expect(source).toContain("t('Transcript')")
    expect(source).toContain("t('No transcript projected for this call yet.')")
    expect(source).not.toContain('fetch(')
  })
})
```

### `frontend/src/integrations/telegram/components/TelegramCallsPanel.boundary.test.ts`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/integrations/telegram/components/TelegramCallsPanel.boundary.test.ts`
- Size bytes / Размер в байтах: `740`
- Included characters / Включено символов: `740`
- Truncated / Обрезано: `no`

```typescript
import { describe, expect, it } from 'vitest'
import { readFileSync } from 'node:fs'
import { resolve } from 'node:path'

describe('TelegramCallsPanel boundary', () => {
  it('loads projected call metadata through the query layer and filters locally before transcript rendering', () => {
    const source = readFileSync(
      resolve('src/integrations/telegram/components/TelegramCallsPanel.vue'),
      'utf8'
    )

    expect(source).toContain('useTelegramCallsQuery')
    expect(source).toContain('filteredCalls')
    expect(source).toContain('TelegramCallTranscriptPanel')
    expect(source).toContain("t('Search projected calls')")
    expect(source).toContain("t('Recent Calls')")
    expect(source).not.toContain('fetch(')
  })
})
```

### `frontend/src/integrations/telegram/components/TelegramCapabilityMatrix.boundary.test.ts`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/integrations/telegram/components/TelegramCapabilityMatrix.boundary.test.ts`
- Size bytes / Размер в байтах: `788`
- Included characters / Включено символов: `788`
- Truncated / Обрезано: `no`

```typescript
import { describe, expect, it } from 'vitest'
import { readFileSync } from 'node:fs'

describe('TelegramCapabilityMatrix boundary', () => {
  it('loads account-scoped capabilities through the query layer', () => {
    const source = readFileSync(new URL('./TelegramCapabilityMatrix.vue', import.meta.url), 'utf8')

    expect(source).toContain('useTelegramAccountCapabilitiesQuery')
    expect(source).toContain('planned_features')
    expect(source).toContain('unsupported_features')
    expect(source).toContain('capability.operation')
    expect(source).toContain('capability.status')
    expect(source).toContain("t('Capabilities')")
    expect(source).toContain('confirmation_required')
    expect(source).toContain('closure_gate')
    expect(source).not.toContain('fetch(')
  })
})
```

### `frontend/src/integrations/telegram/components/TelegramCommandAuditPanel.boundary.test.ts`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/integrations/telegram/components/TelegramCommandAuditPanel.boundary.test.ts`
- Size bytes / Размер в байтах: `1039`
- Included characters / Включено символов: `1039`
- Truncated / Обрезано: `no`

```typescript
import { describe, expect, it } from 'vitest'
import { readFileSync } from 'node:fs'
import { resolve } from 'node:path'

describe('TelegramCommandAuditPanel boundary', () => {
  it('loads durable command rows through the query layer and filters them locally', () => {
    const source = readFileSync(
      resolve('src/integrations/telegram/components/TelegramCommandAuditPanel.vue'),
      'utf8'
    )

    expect(source).toContain('useTelegramCommandsQuery')
    expect(source).toContain('providerChatId: computed(() =>')
    expect(source).toContain('telegramCommandAuditState')
    expect(source).toContain('telegramCommandSubject')
    expect(source).toContain('telegramCommandRetrySummary')
    expect(source).toContain('filteredCommands')
    expect(source).toContain("t('Current chat only')")
    expect(source).toContain("t('Search command rows')")
    expect(source).toContain("t('Recent Commands')")
    expect(source).toContain('telegram-command-audit__item--dead-letter')
    expect(source).not.toContain('fetch(')
  })
})
```

### `frontend/src/integrations/telegram/components/TelegramQrLoginPanel.boundary.test.ts`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/integrations/telegram/components/TelegramQrLoginPanel.boundary.test.ts`
- Size bytes / Размер в байтах: `812`
- Included characters / Включено символов: `812`
- Truncated / Обрезано: `no`

```typescript
import { describe, expect, it } from 'vitest'
import { readFileSync } from 'node:fs'
import { resolve } from 'node:path'

describe('TelegramQrLoginPanel boundary', () => {
  it('drives QR login through query/mutation hooks rather than inline fetch', () => {
    const source = readFileSync(
      resolve('src/integrations/telegram/components/TelegramQrLoginPanel.vue'),
      'utf8'
    )

    expect(source).toContain('useStartTelegramQrLoginMutation')
    expect(source).toContain('useTelegramQrLoginStatusQuery')
    expect(source).toContain('useCancelTelegramQrLoginMutation')
    expect(source).toContain('useSubmitTelegramQrPasswordMutation')
    expect(source).toContain("t('Start QR')")
    expect(source).toContain("t('Apply Suggested Account')")
    expect(source).not.toMatch(/\bfetch\s*\(/)
  })
})
```

### `frontend/src/integrations/telegram/components/TelegramStatusMessages.boundary.test.ts`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/integrations/telegram/components/TelegramStatusMessages.boundary.test.ts`
- Size bytes / Размер в байтах: `1176`
- Included characters / Включено символов: `1176`
- Truncated / Обрезано: `no`

```typescript
import { describe, expect, it } from 'vitest'
import { readFileSync } from 'node:fs'

describe('TelegramStatusMessages realtime recovery status', () => {
  it('renders shared realtime state without owning transport or fetching data', () => {
    const source = readFileSync(new URL('./TelegramStatusMessages.vue', import.meta.url), 'utf8')

    expect(source).toContain('realtimeStatusLabel: string')
    expect(source).toContain('realtimeStatusDetail: string')
    expect(source).toContain('realtimeRecoveryDetail: string')
    expect(source).toContain("realtimeStatusTone: 'neutral' | 'success' | 'warning' | 'danger'")
    expect(source).toContain('useRealtimeStatusStore')
    expect(source).toContain('realtimeStatus.canTriggerReconnect')
    expect(source).toContain("t('Reconnect realtime')")
    expect(source).toContain('realtimeStatus.requestReconnect()')
    expect(source).toContain("{{ t('Realtime') }}: {{ realtimeStatusLabel }}")
    expect(source).toContain("{{ t('Recovery') }}: {{ realtimeRecoveryDetail }}")
    expect(source).not.toContain('fetch(')
    expect(source).not.toContain('new WebSocket')
    expect(source).not.toContain('EventSource')
  })
})
```

### `frontend/src/integrations/telegram/forms/telegramAccountSetupForm.ts`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/integrations/telegram/forms/telegramAccountSetupForm.ts`
- Size bytes / Размер в байтах: `2680`
- Included characters / Включено символов: `2680`
- Truncated / Обрезано: `no`

```typescript
import { toTypedSchema } from '@vee-validate/zod'
import { z } from 'zod'

export const telegramAccountSetupSchema = toTypedSchema(
  z
    .object({
      account_id: z.string().trim().min(1, 'Account ID is required'),
      provider_kind: z.enum(['telegram_user', 'telegram_bot']),
      display_name: z.string().trim().min(1, 'Display name is required'),
      external_account_id: z.string().trim().min(1, 'External account ID is required'),
      api_id: z.union([z.number().int().positive(), z.nan()]).optional(),
      api_hash: z.string().trim().optional(),
      bot_token: z.string().trim().optional(),
      session_encryption_key: z.string().trim().optional(),
      tdlib_data_path: z.string().trim().optional(),
      qr_authorized: z.boolean(),
      transcription_enabled: z.boolean(),
    })
    .superRefine((value, ctx) => {
      if (value.provider_kind === 'telegram_user') {
        if (value.qr_authorized) {
          if (!value.tdlib_data_path) {
            ctx.addIssue({
              code: z.ZodIssueCode.custom,
              path: ['tdlib_data_path'],
              message: 'TDLib data path is required for QR-authorized user accounts',
            })
          }
          return
        }
        if (!value.api_id || Number.isNaN(value.api_id)) {
          ctx.addIssue({
            code: z.ZodIssueCode.custom,
            path: ['api_id'],
            message: 'API ID is required for Telegram user accounts',
          })
        }
        if (!value.api_hash) {
          ctx.addIssue({
            code: z.ZodIssueCode.custom,
            path: ['api_hash'],
            message: 'API hash is required for Telegram user accounts',
          })
        }
        return
      }

      if (!value.bot_token) {
        ctx.addIssue({
          code: z.ZodIssueCode.custom,
          path: ['bot_token'],
          message: 'Bot token is required for Telegram bot accounts',
        })
      }
    })
)

export type TelegramAccountSetupFormValues = {
  account_id: string
  provider_kind: 'telegram_user' | 'telegram_bot'
  display_name: string
  external_account_id: string
  api_id?: number
  api_hash?: string
  bot_token?: string
  session_encryption_key?: string
  tdlib_data_path?: string
  qr_authorized: boolean
  transcription_enabled: boolean
}

export function defaultTelegramAccountSetupValues(): TelegramAccountSetupFormValues {
  return {
    account_id: '',
    provider_kind: 'telegram_user',
    display_name: '',
    external_account_id: '',
    api_id: undefined,
    api_hash: '',
    bot_token: '',
    session_encryption_key: '',
    tdlib_data_path: '',
    qr_authorized: false,
    transcription_enabled: false,
  }
}
```

### `frontend/src/integrations/telegram/queries/realtimeTelegramCommandPatches.ts`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/integrations/telegram/queries/realtimeTelegramCommandPatches.ts`
- Size bytes / Размер в байтах: `10709`
- Included characters / Включено символов: `10709`
- Truncated / Обрезано: `no`

```typescript
import {
  isRecord,
  numberValue,
  storedEventEnvelope,
  stringValue,
} from '../../../shared/communications/queries/realtimePatchShared'
import type { TelegramProviderWriteCommand } from '../types/telegram'

type TelegramEventPayload = Record<string, unknown>
type TelegramStoredEventEnvelope = {
  event?: {
    event_type?: unknown
    payload?: unknown
  }
}

export type TelegramCommandRealtimePatchQueryClient = {
  getQueriesData?: <TData>(filters: { queryKey: readonly unknown[] }) => Array<
    [readonly unknown[], TData | undefined]
  >
  setQueryData?: <TData>(
    queryKey: readonly unknown[],
    updater: TData | ((data: TData | undefined) => TData | undefined)
  ) => unknown
}

export function applyTelegramCommandRealtimePatch(
  eventData: string,
  queryClient: TelegramCommandRealtimePatchQueryClient
): boolean {
  const { getQueriesData, setQueryData } = queryClient
  if (!getQueriesData || !setQueryData) return false

  const envelope = storedEventEnvelope(eventData) as TelegramStoredEventEnvelope | null
  const eventType = stringValue(envelope?.event?.event_type)
  if (!eventType || !eventType.startsWith('telegram.')) return false

  const payload = isRecord(envelope?.event?.payload)
    ? (envelope?.event?.payload as TelegramEventPayload)
    : undefined

  let patched = false
  for (const [queryKey, data] of getQueriesData<TelegramProviderWriteCommand[]>({
    queryKey: ['integrations', 'telegram', 'commands']
  })) {
    const updated = patchTelegramCommandList(queryKey, data, eventType, payload)
    if (updated !== data) {
      setQueryData(queryKey, updated)
      patched = true
    }
  }

  return patched
}

export function patchTelegramCommandList(
  queryKey: readonly unknown[],
  commands: TelegramProviderWriteCommand[] | undefined,
  eventType: string,
  payload: TelegramEventPayload | undefined
): TelegramProviderWriteCommand[] | undefined {
  if (
    !commands ||
    (
      eventType !== 'telegram.command.status_changed' &&
      eventType !== 'telegram.command.reconciled' &&
      eventType !== 'telegram.media.upload.started' &&
      eventType !== 'telegram.media.upload.progress'
    ) ||
    !payload
  ) return commands
  if (queryKey[0] !== 'integrations' || queryKey[1] !== 'telegram' || queryKey[2] !== 'commands') return commands

  const queryAccountId = typeof queryKey[3] === 'string' && queryKey[3] !== 'none' ? queryKey[3] : null
  const queryProviderChatId =
    typeof queryKey[5] === 'string' && queryKey[5] !== 'all' ? queryKey[5] : null
  const queryProviderMessageId =
    typeof queryKey[6] === 'string' && queryKey[6] !== 'all' ? queryKey[6] : null
  const queryCommandKinds =
    typeof queryKey[7] === 'string' && queryKey[7] !== 'all'
      ? new Set(queryKey[7].split('|').filter((value) => value.length > 0))
      : null
  const payloadAccountId = stringValue(payload.account_id)
  if (queryAccountId && payloadAccountId && payloadAccountId !== queryAccountId) return commands
  const commandId = stringValue(payload.command_id)
  const newStatus = stringValue(payload.status)
  if (!commandId || !newStatus) return commands

  const payloadProviderChatId = stringValue(payload.provider_chat_id)
  if (queryProviderChatId && payloadProviderChatId && payloadProviderChatId !== queryProviderChatId) {
    return commands
  }
  const payloadProviderMessageId =
    stringValue(payload.provider_message_id) ?? stringValue(payload.message_id)
  if (
    queryProviderMessageId &&
    payloadProviderMessageId &&
    payloadProviderMessageId !== queryProviderMessageId
  ) {
    return commands
  }
  const payloadCommandKind = insertedCommandKind(eventType, payload)
  if (queryCommandKinds && payloadCommandKind && !queryCommandKinds.has(payloadCommandKind)) {
    return commands
  }

  const matchIndex = commands.findIndex((command) => command.command_id === commandId)
  if (matchIndex < 0) {
    if (!payloadAccountId) return commands
    return insertCommand(
      queryKey,
      commands,
      payloadAccountId,
      commandId,
      newStatus,
      eventType,
      payload
    )
  }

  const matchedCommand = commands[matchIndex]
  if (queryAccountId && matchedCommand.account_id !== queryAccountId) return commands

  const nextCommand = {
    ...matchedCommand,
    status: newStatus as TelegramProviderWriteCommand['status'],
    retry_count: numberValue(payload.retry_count) ?? matchedCommand.retry_count,
    max_retries: numberValue(payload.max_retries) ?? matchedCommand.max_retries,
    last_error: normalizeNullableString(payload.last_error, matchedCommand.last_error),
    result_payload: recordValue(payload.result_payload) ?? matchedCommand.result_payload,
    next_attempt_at: normalizeNullableString(payload.next_attempt_at, matchedCommand.next_attempt_at),
    last_attempt_at: normalizeNullableString(payload.last_attempt_at, matchedCommand.last_attempt_at),
    provider_observed_at: normalizeNullableString(payload.provider_observed_at, matchedCommand.provider_observed_at),
    provider_state: recordValue(payload.provider_state) ?? matchedCommand.provider_state,
    reconciliation_status:
      stringValue(payload.reconciliation_status) ?? matchedCommand.reconciliation_status,
    reconciled_at: normalizeNullableString(payload.reconciled_at, matchedCommand.reconciled_at),
    dead_lettered_at: normalizeNullableString(payload.dead_lettered_at, matchedCommand.dead_lettered_at),
    completed_at: normalizeNullableString(payload.completed_at, matchedCommand.completed_at),
    updated_at: new Date().toISOString(),
  } satisfies TelegramProviderWriteCommand

  return commands.map((command, index) => index === matchIndex ? nextCommand : command)
}

function normalizeNullableString(value: unknown, fallback: string | null): string | null {
  if (value === null) return null
  return stringValue(value) ?? fallback
}

function recordValue(value: unknown): Record<string, unknown> | null {
  return isRecord(value) ? value : null
}

function insertCommand(
  queryKey: readonly unknown[],
  commands: TelegramProviderWriteCommand[],
  accountId: string,
  commandId: string,
  status: string,
  eventType: string,
  payload: TelegramEventPayload
): TelegramProviderWriteCommand[] {
  const commandKind = insertedCommandKind(eventType, payload)
  if (!commandKind) return commands

  const now = new Date().toISOString()
  const eventPayload = insertedPayload(payload)
  const limit = typeof queryKey[4] === 'number' ? queryKey[4] : null
  const nextCommand = {
    command_id: commandId,
    account_id: accountId,
    command_kind: commandKind,
    idempotency_key: stringValue(payload.idempotency_key) ?? commandId,
    provider_chat_id: stringValue(payload.provider_chat_id) ?? '',
    provider_message_id:
      stringValue(payload.provider_message_id) ?? stringValue(payload.message_id),
    target_ref: recordValue(payload.target_ref) ?? {},
    payload: eventPayload,
    capability_state: capabilityStateValue(payload.capability_state),
    action_class: actionClassValue(payload.action_class),
    confirmation_decision: confirmationDecisionValue(payload.confirmation_decision),
    status: status as TelegramProviderWriteCommand['status'],
    retry_count: numberValue(payload.retry_count) ?? 0,
    max_retries: numberValue(payload.max_retries) ?? 0,
    last_error: normalizeNullableString(payload.last_error, null),
    result_payload: recordValue(payload.result_payload) ?? {},
    audit_metadata: recordValue(payload.audit_metadata) ?? {},
    actor_id: stringValue(payload.actor_id) ?? 'makosh-frontend',
    happened_at: stringValue(payload.happened_at) ?? now,
    next_attempt_at: normalizeNullableString(payload.next_attempt_at, null),
    last_attempt_at: normalizeNullableString(payload.last_attempt_at, null),
    locked_at: null,
    locked_by: null,
    provider_observed_at: normalizeNullableString(payload.provider_observed_at, null),
    provider_state: recordValue(payload.provider_state) ?? {},
    reconciliation_status: stringValue(payload.reconciliation_status) ?? 'not_observed',
    reconciled_at: normalizeNullableString(payload.reconciled_at, null),
    dead_lettered_at: normalizeNullableString(payload.dead_lettered_at, null),
    completed_at: normalizeNullableString(payload.completed_at, null),
    created_at: stringValue(payload.created_at) ?? now,
    updated_at: stringValue(payload.updated_at) ?? now,
  } satisfies TelegramProviderWriteCommand

  const nextCommands = [nextCommand, ...commands]
  return typeof limit === 'number' ? nextCommands.slice(0, limit) : nextCommands
}

function insertedPayload(payload: TelegramEventPayload): Record<string, unknown> {
  const explicitPayload = recordValue(payload.payload)
  if (explicitPayload) return explicitPayload

  const fallbackPayload: Record<string, unknown> = {}
  const action = stringValue(payload.action)
  const providerChatId = stringValue(payload.provider_chat_id)
  const telegramChatId = stringValue(payload.telegram_chat_id)

  if (action) fallbackPayload.action = action
  if (providerChatId) fallbackPayload.provider_chat_id = providerChatId
  if (telegramChatId) fallbackPayload.telegram_chat_id = telegramChatId

  return fallbackPayload
}

function insertedCommandKind(
  eventType: string,
  payload: TelegramEventPayload
): TelegramProviderWriteCommand['command_kind'] | null {
  if (
    eventType === 'telegram.media.upload.started' ||
    eventType === 'telegram.media.upload.progress'
  ) {
    return 'send_media'
  }

  const explicitKind = stringValue(payload.command_kind)
  if (explicitKind) return explicitKind as TelegramProviderWriteCommand['command_kind']

  const action = stringValue(payload.action)
  if (action === 'join' || action === 'leave') {
    return action as TelegramProviderWriteCommand['command_kind']
  }

  return null
}

function capabilityStateValue(value: unknown): TelegramProviderWriteCommand['capability_state'] {
  const normalized = stringValue(value)
  return normalized === 'blocked' || normalized === 'degraded' || normalized === 'unsupported'
    ? normalized
    : 'available'
}

function actionClassValue(value: unknown): TelegramProviderWriteCommand['action_class'] {
  const normalized = stringValue(value)
  return normalized === 'read' ||
    normalized === 'local_write' ||
    normalized === 'destructive' ||
    normalized === 'export' ||
    normalized === 'secret_access' ||
    normalized === 'automation'
    ? normalized
    : 'provider_write'
}

function confirmationDecisionValue(
  value: unknown
): TelegramProviderWriteCommand['confirmation_decision'] {
  const normalized = stringValue(value)
  return normalized === 'not_required' || normalized === 'rejected'
    ? normalized
    : 'confirmed'
}
```

### `frontend/src/integrations/telegram/queries/telegramQueryKeys.ts`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/integrations/telegram/queries/telegramQueryKeys.ts`
- Size bytes / Размер в байтах: `785`
- Included characters / Включено символов: `785`
- Truncated / Обрезано: `no`

```typescript
export const telegramQueryKeys = {
  capabilities: ['integrations', 'telegram', 'capabilities'] as const,
  accountCapabilities: ['integrations', 'telegram', 'account-capabilities'] as const,
  accounts: ['integrations', 'telegram', 'accounts'] as const,
  chats: ['integrations', 'telegram', 'provider-conversations'] as const,
  folders: ['integrations', 'telegram', 'provider-folders'] as const,
  chatDetail: ['integrations', 'telegram', 'provider-conversation-detail'] as const,
  chatMembers: ['integrations', 'telegram', 'provider-conversation-members'] as const,
  runtime: ['integrations', 'telegram', 'runtime'] as const,
  calls: ['integrations', 'telegram', 'provider-calls'] as const,
  callTranscript: ['integrations', 'telegram', 'provider-call-transcript'] as const,
}
```

### `frontend/src/integrations/telegram/queries/useTelegramAutomationQuery.ts`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/integrations/telegram/queries/useTelegramAutomationQuery.ts`
- Size bytes / Размер в байтах: `1713`
- Included characters / Включено символов: `1713`
- Truncated / Обрезано: `no`

```typescript
import { useMutation, useQuery } from '@tanstack/vue-query'
import { computed, toValue, type MaybeRefOrGetter } from 'vue'
import {
  fetchTelegramAutomationPolicies,
  fetchTelegramAutomationTemplates,
  runTelegramSendDryRun,
} from '../api/telegramAutomation'
import type {
  TelegramAutomationPolicy,
  TelegramAutomationTemplate,
  TelegramSendDryRunRequest,
  TelegramSendDryRunResponse,
} from '../types/automation'

export const telegramAutomationQueryKeys = {
  policies: ['integrations', 'telegram', 'automation', 'policies'] as const,
  templates: ['integrations', 'telegram', 'automation', 'templates'] as const,
}

export function useTelegramAutomationPoliciesQuery(
  accountId: MaybeRefOrGetter<string | null | undefined>
) {
  return useQuery<TelegramAutomationPolicy[]>({
    queryKey: computed(() => [
      ...telegramAutomationQueryKeys.policies,
      toValue(accountId) ?? 'none',
    ]),
    queryFn: async () => {
      const response = await fetchTelegramAutomationPolicies()
      const value = toValue(accountId)
      if (!value) return []
      return response.items.filter((policy) => policy.account_id === value)
    },
    enabled: computed(() => Boolean(toValue(accountId))),
  })
}

export function useTelegramAutomationTemplatesQuery() {
  return useQuery<TelegramAutomationTemplate[]>({
    queryKey: telegramAutomationQueryKeys.templates,
    queryFn: async () => {
      const response = await fetchTelegramAutomationTemplates()
      return response.items
    },
  })
}

export function useTelegramSendDryRunMutation() {
  return useMutation<TelegramSendDryRunResponse, Error, TelegramSendDryRunRequest>({
    mutationFn: (request) => runTelegramSendDryRun(request),
  })
}
```

### `frontend/src/integrations/telegram/queries/useTelegramLifecycleQuery.ts`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/integrations/telegram/queries/useTelegramLifecycleQuery.ts`
- Size bytes / Размер в байтах: `2114`
- Included characters / Включено символов: `2114`
- Truncated / Обрезано: `no`

```typescript
import { useMutation, useQuery, useQueryClient } from '@tanstack/vue-query'
import { computed, toValue, type MaybeRefOrGetter } from 'vue'
import {
  fetchTelegramCommands,
  retryTelegramCommand,
} from '../api/telegramLifecycle'
import type {
  TelegramProviderWriteCommand,
} from '../types/telegram'

export function useTelegramCommandsQuery(
  accountId: MaybeRefOrGetter<string | null | undefined>,
  limit: MaybeRefOrGetter<number> = 25,
  enabled: MaybeRefOrGetter<boolean> = true,
  filters?: {
    providerChatId?: MaybeRefOrGetter<string | null | undefined>
    providerMessageId?: MaybeRefOrGetter<string | null | undefined>
    commandKinds?: MaybeRefOrGetter<string[] | null | undefined>
  }
) {
  return useQuery<TelegramProviderWriteCommand[]>({
    queryKey: computed(() => {
      const commandKinds = [...(toValue(filters?.commandKinds) ?? [])]
        .filter((value) => value.trim().length > 0)
        .sort()
      return [
        'integrations',
        'telegram',
        'commands',
        toValue(accountId) ?? 'none',
        toValue(limit),
        toValue(filters?.providerChatId) ?? 'all',
        toValue(filters?.providerMessageId) ?? 'all',
        commandKinds.length > 0 ? commandKinds.join('|') : 'all',
      ]
    }),
    queryFn: async () => {
      const response = await fetchTelegramCommands(toValue(accountId) as string, toValue(limit), {
        providerChatId: toValue(filters?.providerChatId),
        providerMessageId: toValue(filters?.providerMessageId),
        commandKinds: toValue(filters?.commandKinds) ?? undefined,
      })
      return response.items
    },
    enabled: computed(() => Boolean(toValue(accountId)) && Boolean(toValue(enabled))),
  })
}

export function useTelegramCommandRetryMutation() {
  const queryClient = useQueryClient()
  return useMutation({
    mutationFn: retryTelegramCommand,
    onSuccess: (command) => {
      queryClient.invalidateQueries({ queryKey: ['integrations', 'telegram', 'commands', command.account_id] })
      queryClient.invalidateQueries({ queryKey: ['integrations', 'telegram', 'commands'] })
    },
  })
}
```

### `frontend/src/integrations/telegram/queries/useTelegramMembersQuery.ts`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/integrations/telegram/queries/useTelegramMembersQuery.ts`
- Size bytes / Размер в байтах: `784`
- Included characters / Включено символов: `784`
- Truncated / Обрезано: `no`

```typescript
import { useMutation, useQueryClient } from '@tanstack/vue-query'
import { syncTelegramChatMembers } from '../api/telegram'
import { telegramQueryKeys } from './useTelegramQuery'

export function useSyncTelegramChatMembersMutation() {
  const queryClient = useQueryClient()
  return useMutation({
    mutationFn: async (telegramChatId: string) => syncTelegramChatMembers(telegramChatId),
    onSuccess: (_response, telegramChatId) => {
      queryClient.invalidateQueries({ queryKey: [...telegramQueryKeys.chatMembers, telegramChatId] })
      queryClient.invalidateQueries({ queryKey: telegramQueryKeys.chats })
      queryClient.invalidateQueries({ queryKey: telegramQueryKeys.chatDetail })
      queryClient.invalidateQueries({ queryKey: telegramQueryKeys.runtime })
    },
  })
}
```

### `frontend/src/integrations/telegram/queries/useTelegramMutations.ts`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/integrations/telegram/queries/useTelegramMutations.ts`
- Size bytes / Размер в байтах: `9492`
- Included characters / Включено символов: `9492`
- Truncated / Обрезано: `no`

```typescript
import { useMutation, useQueryClient } from '@tanstack/vue-query'
import {
  addTelegramChatToFolder,
  archiveTelegramChat,
  downloadTelegramMedia,
  ingestTelegramFixtureMessage,
  logoutTelegramAccount,
  markTelegramChatRead,
  markTelegramChatUnread,
  muteTelegramChat,
  pinTelegramChat,
  reassignTelegramChatFolders,
  removeTelegramAccount,
  removeTelegramChatFromFolder,
  setupTelegramAccount,
  syncTelegramChats,
  syncTelegramHistory,
  unarchiveTelegramChat,
  unmuteTelegramChat,
  unpinTelegramChat,
} from '../api/telegram'
import type {
  TelegramChatSyncRequest,
  TelegramHistorySyncRequest,
  TelegramMediaDownloadRequest,
} from '../types/telegram'
import { telegramQueryKeys } from './telegramQueryKeys'

export function useSetupTelegramAccountMutation() {
  const queryClient = useQueryClient()
  return useMutation({
    mutationFn: setupTelegramAccount,
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: telegramQueryKeys.accounts })
      queryClient.invalidateQueries({ queryKey: telegramQueryKeys.capabilities })
      queryClient.invalidateQueries({ queryKey: telegramQueryKeys.runtime })
    },
  })
}

export function useLogoutTelegramAccountMutation() {
  const queryClient = useQueryClient()
  return useMutation({
    mutationFn: (accountId: string) => logoutTelegramAccount(accountId),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: telegramQueryKeys.accounts })
      queryClient.invalidateQueries({ queryKey: telegramQueryKeys.runtime })
      queryClient.invalidateQueries({ queryKey: telegramQueryKeys.capabilities })
    },
  })
}

export function useRemoveTelegramAccountMutation() {
  const queryClient = useQueryClient()
  return useMutation({
    mutationFn: (accountId: string) => removeTelegramAccount(accountId),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: telegramQueryKeys.accounts })
      queryClient.invalidateQueries({ queryKey: telegramQueryKeys.runtime })
      queryClient.invalidateQueries({ queryKey: telegramQueryKeys.capabilities })
      queryClient.invalidateQueries({ queryKey: telegramQueryKeys.chats })
      queryClient.invalidateQueries({ queryKey: telegramQueryKeys.folders })
    },
  })
}

export function useSyncTelegramChatsMutation() {
  const queryClient = useQueryClient()
  return useMutation({
    mutationFn: (request: TelegramChatSyncRequest) => syncTelegramChats(request),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: telegramQueryKeys.chats })
      queryClient.invalidateQueries({ queryKey: telegramQueryKeys.folders })
      queryClient.invalidateQueries({ queryKey: telegramQueryKeys.runtime })
    },
  })
}

export function useSyncTelegramHistoryMutation() {
  const queryClient = useQueryClient()
  return useMutation({
    mutationFn: (request: TelegramHistorySyncRequest) => syncTelegramHistory(request),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: telegramQueryKeys.chats })
    },
  })
}

export function useIngestTelegramFixtureMutation() {
  const queryClient = useQueryClient()
  return useMutation({
    mutationFn: (request: Parameters<typeof ingestTelegramFixtureMessage>[0]) =>
      ingestTelegramFixtureMessage(request),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: telegramQueryKeys.chats })
    },
  })
}

export function useDownloadTelegramMediaMutation() {
  const queryClient = useQueryClient()
  return useMutation({
    mutationFn: (request: TelegramMediaDownloadRequest) => downloadTelegramMedia(request),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: telegramQueryKeys.runtime })
    },
  })
}

function useTelegramChatLifecycleMutation(
  mutationFn: (args: {
    telegramChatId: string
    accountId: string
    providerChatId: string
  }) => Promise<unknown>,
  invalidateFolders = false,
  invalidateDetail = false
) {
  const queryClient = useQueryClient()
  return useMutation({
    mutationFn,
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: telegramQueryKeys.chats })
      if (invalidateFolders) {
        queryClient.invalidateQueries({ queryKey: telegramQueryKeys.folders })
      }
      if (invalidateDetail) {
        queryClient.invalidateQueries({ queryKey: telegramQueryKeys.chatDetail })
      }
    },
  })
}

export function usePinTelegramChatMutation() {
  return useTelegramChatLifecycleMutation(({ telegramChatId, accountId, providerChatId }) =>
    pinTelegramChat(telegramChatId, { account_id: accountId, provider_chat_id: providerChatId }))
}

export function useUnpinTelegramChatMutation() {
  return useTelegramChatLifecycleMutation(({ telegramChatId, accountId, providerChatId }) =>
    unpinTelegramChat(telegramChatId, { account_id: accountId, provider_chat_id: providerChatId }))
}

export function useArchiveTelegramChatMutation() {
  return useTelegramChatLifecycleMutation(({ telegramChatId, accountId, providerChatId }) =>
    archiveTelegramChat(telegramChatId, { account_id: accountId, provider_chat_id: providerChatId }))
}

export function useUnarchiveTelegramChatMutation() {
  return useTelegramChatLifecycleMutation(({ telegramChatId, accountId, providerChatId }) =>
    unarchiveTelegramChat(telegramChatId, { account_id: accountId, provider_chat_id: providerChatId }))
}

export function useMuteTelegramChatMutation() {
  return useTelegramChatLifecycleMutation(({ telegramChatId, accountId, providerChatId }) =>
    muteTelegramChat(telegramChatId, { account_id: accountId, provider_chat_id: providerChatId }))
}

export function useUnmuteTelegramChatMutation() {
  return useTelegramChatLifecycleMutation(({ telegramChatId, accountId, providerChatId }) =>
    unmuteTelegramChat(telegramChatId, { account_id: accountId, provider_chat_id: providerChatId }))
}

export function useAddTelegramChatToFolderMutation() {
  const queryClient = useQueryClient()
  return useMutation({
    mutationFn: ({ telegramChatId, accountId, providerChatId, providerFolderId }: {
      telegramChatId: string
      accountId: string
      providerChatId: string
      providerFolderId: number
    }) => addTelegramChatToFolder(telegramChatId, providerFolderId, {
      account_id: accountId,
      provider_chat_id: providerChatId,
    }),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: telegramQueryKeys.chats })
      queryClient.invalidateQueries({ queryKey: telegramQueryKeys.folders })
      queryClient.invalidateQueries({ queryKey: telegramQueryKeys.chatDetail })
    },
  })
}

export function useRemoveTelegramChatFromFolderMutation() {
  const queryClient = useQueryClient()
  return useMutation({
    mutationFn: ({ telegramChatId, accountId, providerChatId, providerFolderId }: {
      telegramChatId: string
      accountId: string
      providerChatId: string
      providerFolderId: number
    }) => removeTelegramChatFromFolder(telegramChatId, providerFolderId, {
      account_id: accountId,
      provider_chat_id: providerChatId,
    }),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: telegramQueryKeys.chats })
      queryClient.invalidateQueries({ queryKey: telegramQueryKeys.folders })
      queryClient.invalidateQueries({ queryKey: telegramQueryKeys.chatDetail })
    },
  })
}

export function useReassignTelegramChatFoldersMutation() {
  const queryClient = useQueryClient()
  return useMutation({
    mutationFn: ({ telegramChatId, accountId, providerChatId, targetProviderFolderIds }: {
      telegramChatId: string
      accountId: string
      providerChatId: string
      targetProviderFolderIds: number[]
    }) => reassignTelegramChatFolders(telegramChatId, {
      account_id: accountId,
      provider_chat_id: providerChatId,
      target_provider_folder_ids: targetProviderFolderIds,
    }),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: telegramQueryKeys.chats })
      queryClient.invalidateQueries({ queryKey: telegramQueryKeys.folders })
      queryClient.invalidateQueries({ queryKey: telegramQueryKeys.chatDetail })
    },
  })
}

export function useMarkReadTelegramChatMutation() {
  const queryClient = useQueryClient()
  return useMutation({
    mutationFn: ({ telegramChatId, accountId, providerChatId, lastReadInboxProviderMessageId }: {
      telegramChatId: string
      accountId: string
      providerChatId: string
      lastReadInboxProviderMessageId?: string
    }) => markTelegramChatRead(telegramChatId, {
      account_id: accountId,
      provider_chat_id: providerChatId,
      ...(lastReadInboxProviderMessageId
        ? {
            last_read_inbox_provider_message_id: lastReadInboxProviderMessageId,
          }
        : {}),
    }),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: telegramQueryKeys.chats })
      queryClient.invalidateQueries({ queryKey: telegramQueryKeys.chatDetail })
    },
  })
}

export function useMarkUnreadTelegramChatMutation() {
  const queryClient = useQueryClient()
  return useMutation({
    mutationFn: ({ telegramChatId, accountId, providerChatId }: {
      telegramChatId: string
      accountId: string
      providerChatId: string
    }) => markTelegramChatUnread(telegramChatId, {
      account_id: accountId,
      provider_chat_id: providerChatId,
    }),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: telegramQueryKeys.chats })
      queryClient.invalidateQueries({ queryKey: telegramQueryKeys.chatDetail })
    },
  })
}
```

### `frontend/src/integrations/telegram/queries/useTelegramParticipantLifecycleQuery.test.ts`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/integrations/telegram/queries/useTelegramParticipantLifecycleQuery.test.ts`
- Size bytes / Размер в байтах: `2529`
- Included characters / Включено символов: `2529`
- Truncated / Обрезано: `no`

```typescript
import { QueryClient } from '@tanstack/vue-query'
import { describe, expect, it } from 'vitest'
import type { TelegramProviderWriteCommand } from '../types/telegram'
import { primeTelegramParticipantLifecycleCommandCache } from './useTelegramParticipantLifecycleQuery'

function queryClient() {
  return new QueryClient({
    defaultOptions: {
      queries: {
        retry: false,
      },
    },
  })
}

describe('telegram participant lifecycle command cache priming', () => {
  it('inserts join command into matching account command caches before realtime reconciliation', () => {
    const client = queryClient()
    const accountCommandsKey = ['integrations', 'telegram', 'commands', 'account-1', 10] as const
    const otherAccountCommandsKey = ['integrations', 'telegram', 'commands', 'account-2', 10] as const

    client.setQueryData<TelegramProviderWriteCommand[]>(accountCommandsKey, [])
    client.setQueryData<TelegramProviderWriteCommand[]>(otherAccountCommandsKey, [])

    primeTelegramParticipantLifecycleCommandCache(client, 'account-1', {
      telegram_chat_id: 'tgchat-1',
      provider_chat_id: 'chat-1',
      action: 'join',
      status: 'queued',
      command_id: 'cmd-join-1',
    })

    expect(client.getQueryData<TelegramProviderWriteCommand[]>(accountCommandsKey)).toMatchObject([
      {
        command_id: 'cmd-join-1',
        account_id: 'account-1',
        command_kind: 'join',
        provider_chat_id: 'chat-1',
        status: 'queued',
      },
    ])
    expect(client.getQueryData<TelegramProviderWriteCommand[]>(otherAccountCommandsKey)).toEqual([])
  })

  it('inserts leave command with current chat target metadata before reconciliation arrives', () => {
    const client = queryClient()
    const accountCommandsKey = ['integrations', 'telegram', 'commands', 'account-1', 10] as const

    client.setQueryData<TelegramProviderWriteCommand[]>(accountCommandsKey, [])

    primeTelegramParticipantLifecycleCommandCache(client, 'account-1', {
      telegram_chat_id: 'tgchat-9',
      provider_chat_id: 'chat-9',
      action: 'leave',
      status: 'queued',
      command_id: 'cmd-leave-9',
    })

    expect(client.getQueryData<TelegramProviderWriteCommand[]>(accountCommandsKey)).toMatchObject([
      {
        command_id: 'cmd-leave-9',
        account_id: 'account-1',
        command_kind: 'leave',
        provider_chat_id: 'chat-9',
        target_ref: {
          provider_chat_id: 'chat-9',
          telegram_chat_id: 'tgchat-9',
        },
      },
    ])
  })
})
```

### `frontend/src/integrations/telegram/queries/useTelegramParticipantLifecycleQuery.ts`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/integrations/telegram/queries/useTelegramParticipantLifecycleQuery.ts`
- Size bytes / Размер в байтах: `3587`
- Included characters / Включено символов: `3587`
- Truncated / Обрезано: `no`

```typescript
import { useMutation, useQueryClient } from '@tanstack/vue-query'
import { joinTelegramChat, leaveTelegramChat } from '../api/telegram'
import { patchTelegramCommandList } from './realtimeTelegramCommandPatches'
import { telegramQueryKeys } from './useTelegramQuery'
import type {
  TelegramChatLifecycleCommandResponse,
  TelegramProviderWriteCommand,
} from '../types/telegram'

type TelegramParticipantLifecycleInput = {
  telegramChatId?: string | null
  accountId: string
  providerChatId: string
}

type TelegramParticipantLifecycleQueryClient = Pick<
  ReturnType<typeof useQueryClient>,
  'getQueriesData' | 'setQueryData'
>

export function primeTelegramParticipantLifecycleCommandCache(
  queryClient: TelegramParticipantLifecycleQueryClient,
  accountId: string,
  command: TelegramChatLifecycleCommandResponse
) {
  if (!queryClient.getQueriesData || !queryClient.setQueryData) return

  const payload = {
    account_id: accountId,
    provider_chat_id: command.provider_chat_id,
    telegram_chat_id: command.telegram_chat_id,
    action: command.action,
    status: command.status,
    command_id: command.command_id,
    capability_state: 'available',
    action_class: 'provider_write',
    confirmation_decision: 'confirmed',
    target_ref: {
      provider_chat_id: command.provider_chat_id,
      telegram_chat_id: command.telegram_chat_id,
    },
    payload: {
      provider_chat_id: command.provider_chat_id,
      telegram_chat_id: command.telegram_chat_id,
      action: command.action,
    },
  }

  for (const [queryKey] of queryClient.getQueriesData<TelegramProviderWriteCommand[]>({
    queryKey: ['integrations', 'telegram', 'commands'],
  })) {
    queryClient.setQueryData<TelegramProviderWriteCommand[] | undefined>(
      queryKey,
      (cachedCommands) =>
        patchTelegramCommandList(
          queryKey,
          cachedCommands,
          'telegram.command.status_changed',
          payload
        )
    )
  }
}

function invalidateParticipantLifecycleCaches(queryClient: ReturnType<typeof useQueryClient>) {
  queryClient.invalidateQueries({ queryKey: telegramQueryKeys.chats })
  queryClient.invalidateQueries({ queryKey: telegramQueryKeys.chatDetail })
  queryClient.invalidateQueries({ queryKey: telegramQueryKeys.chatMembers })
  queryClient.invalidateQueries({ queryKey: telegramQueryKeys.runtime })
  queryClient.invalidateQueries({ queryKey: ['integrations', 'telegram', 'commands'] })
}

export function useJoinTelegramChatMutation() {
  const queryClient = useQueryClient()
  return useMutation({
    mutationFn: ({ accountId, providerChatId }: TelegramParticipantLifecycleInput) =>
      joinTelegramChat({ account_id: accountId, provider_chat_id: providerChatId }),
    onSuccess: (command, variables) => {
      primeTelegramParticipantLifecycleCommandCache(queryClient, variables.accountId, command)
      invalidateParticipantLifecycleCaches(queryClient)
    },
  })
}

export function useLeaveTelegramChatMutation() {
  const queryClient = useQueryClient()
  return useMutation({
    mutationFn: ({ telegramChatId, accountId, providerChatId }: TelegramParticipantLifecycleInput) => {
      if (!telegramChatId?.trim()) {
        throw new Error('Telegram chat id is required for leave command')
      }
      return leaveTelegramChat(telegramChatId, { account_id: accountId, provider_chat_id: providerChatId })
    },
    onSuccess: (command, variables) => {
      primeTelegramParticipantLifecycleCommandCache(queryClient, variables.accountId, command)
      invalidateParticipantLifecycleCaches(queryClient)
    },
  })
}
```

### `frontend/src/integrations/telegram/queries/useTelegramQrLoginQuery.ts`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/integrations/telegram/queries/useTelegramQrLoginQuery.ts`
- Size bytes / Размер в байтах: `2170`
- Included characters / Включено символов: `2170`
- Truncated / Обрезано: `no`

```typescript
import { useMutation, useQuery, useQueryClient } from '@tanstack/vue-query'
import { computed, toValue, type MaybeRefOrGetter } from 'vue'
import {
  cancelTelegramQrLogin,
  getTelegramQrLoginStatus,
  startTelegramQrLogin,
  submitTelegramQrPassword,
} from '../api/telegram'
import type {
  TelegramQrLoginPasswordRequest,
  TelegramQrLoginStartRequest,
  TelegramQrLoginStatusResponse,
} from '../types/telegram'

export const telegramQrLoginQueryKeys = {
  status: ['integrations', 'telegram', 'qr-login-status'] as const,
}

export function useStartTelegramQrLoginMutation() {
  return useMutation({
    mutationFn: (request: TelegramQrLoginStartRequest) => startTelegramQrLogin(request),
  })
}

export function useTelegramQrLoginStatusQuery(
  setupId: MaybeRefOrGetter<string | null | undefined>
) {
  return useQuery<TelegramQrLoginStatusResponse | null>({
    queryKey: computed(() => [
      ...telegramQrLoginQueryKeys.status,
      toValue(setupId) ?? 'none',
    ]),
    queryFn: async () => {
      const value = toValue(setupId)
      if (!value) return null
      return getTelegramQrLoginStatus(value)
    },
    enabled: computed(() => Boolean(toValue(setupId))),
  })
}

export function useCancelTelegramQrLoginMutation(
  setupId: MaybeRefOrGetter<string | null | undefined>
) {
  const queryClient = useQueryClient()
  return useMutation({
    mutationFn: async () => {
      const value = toValue(setupId)
      if (!value) {
        throw new Error('Telegram QR setup ID is required')
      }
      return cancelTelegramQrLogin(value)
    },
    onSuccess: () => {
      const value = toValue(setupId)
      if (!value) return
      queryClient.removeQueries({
        queryKey: [...telegramQrLoginQueryKeys.status, value],
      })
    },
  })
}

export function useSubmitTelegramQrPasswordMutation(
  setupId: MaybeRefOrGetter<string | null | undefined>
) {
  return useMutation({
    mutationFn: async (request: TelegramQrLoginPasswordRequest) => {
      const value = toValue(setupId)
      if (!value) {
        throw new Error('Telegram QR setup ID is required')
      }
      return submitTelegramQrPassword(value, request)
    },
  })
}
```

### `frontend/src/integrations/telegram/queries/useTelegramQuery.ts`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/integrations/telegram/queries/useTelegramQuery.ts`
- Size bytes / Размер в байтах: `3336`
- Included characters / Включено символов: `3336`
- Truncated / Обрезано: `no`

```typescript
import { useQuery } from '@tanstack/vue-query'
import { computed, toValue, type MaybeRefOrGetter } from 'vue'
import {
  fetchTelegramAccountCapabilities,
  fetchTelegramCapabilities,
  fetchTelegramFolders,
  fetchTelegramAccounts,
  fetchTelegramCalls,
  fetchTelegramCallTranscript,
} from '../api/telegram'
import type {
  TelegramCapabilitiesResponse,
  TelegramCall,
  TelegramCallTranscript,
  TelegramAccount,
  TelegramChatGroupFilter,
} from '../types/telegram'
import { telegramQueryKeys } from './telegramQueryKeys'

export { telegramQueryKeys } from './telegramQueryKeys'
export * from './useTelegramMutations'

// --- Fetch capabilities ---
export function useTelegramCapabilitiesQuery() {
  return useQuery<TelegramCapabilitiesResponse>({
    queryKey: telegramQueryKeys.capabilities,
    queryFn: () => fetchTelegramCapabilities()
  })
}

export function useTelegramAccountCapabilitiesQuery(
  accountId: MaybeRefOrGetter<string | null | undefined>
) {
  return useQuery<TelegramCapabilitiesResponse | null>({
    queryKey: computed(() => [
      ...telegramQueryKeys.accountCapabilities,
      toValue(accountId) ?? 'none',
    ]),
    queryFn: async () => {
      const value = toValue(accountId)
      if (!value) return null
      return fetchTelegramAccountCapabilities(value)
    },
    enabled: computed(() => Boolean(toValue(accountId))),
  })
}

// --- Fetch accounts ---
export function useTelegramAccountsQuery() {
  return useQuery<TelegramAccount[]>({
    queryKey: telegramQueryKeys.accounts,
    queryFn: async () => {
      const res = await fetchTelegramAccounts()
      return res.items
    }
  })
}

export function useTelegramFoldersQuery(
  accountId?: MaybeRefOrGetter<string | undefined>
) {
  return useQuery<TelegramChatGroupFilter[]>({
    queryKey: computedTelegramFoldersQueryKey(accountId),
    queryFn: async () => {
      const res = await fetchTelegramFolders(toValue(accountId))
      return res.items
    }
  })
}

// --- Fetch calls ---
export function useTelegramCallsQuery(
  accountId?: MaybeRefOrGetter<string | undefined>,
  limit: MaybeRefOrGetter<number> = 50
) {
  return useQuery<TelegramCall[]>({
    queryKey: computedTelegramCallsQueryKey(accountId, limit),
    queryFn: async () => {
      const res = await fetchTelegramCalls(toValue(accountId), toValue(limit))
      return res.items
    }
  })
}

export function useTelegramCallTranscriptQuery(
  callId: MaybeRefOrGetter<string | null | undefined>
) {
  return useQuery<TelegramCallTranscript | null>({
    queryKey: computed(() => [
      ...telegramQueryKeys.callTranscript,
      toValue(callId) ?? 'none',
    ]),
    queryFn: async () => {
      const value = toValue(callId)
      if (!value) return null
      const res = await fetchTelegramCallTranscript(value)
      return res.transcript
    },
    enabled: computed(() => Boolean(toValue(callId))),
  })
}

function computedTelegramFoldersQueryKey(
  accountId?: MaybeRefOrGetter<string | undefined>
) {
  return computed(() => [
    ...telegramQueryKeys.folders,
    toValue(accountId) ?? 'all',
  ])
}

function computedTelegramCallsQueryKey(
  accountId?: MaybeRefOrGetter<string | undefined>,
  limit: MaybeRefOrGetter<number> = 50
) {
  return computed(() => [
    ...telegramQueryKeys.calls,
    toValue(accountId) ?? 'all',
    toValue(limit)
  ])
}
```

### `frontend/src/integrations/telegram/queries/useTelegramRuntimeQuery.ts`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/integrations/telegram/queries/useTelegramRuntimeQuery.ts`
- Size bytes / Размер в байтах: `1798`
- Included characters / Включено символов: `1798`
- Truncated / Обрезано: `no`

```typescript
import { useMutation, useQuery, useQueryClient } from '@tanstack/vue-query'
import { computed, toValue, type MaybeRefOrGetter } from 'vue'
import {
  fetchTelegramRuntimeStatus,
  restartTelegramRuntime,
  startTelegramRuntime,
  stopTelegramRuntime,
} from '../api/telegram'
import type { TelegramRuntimeStatus } from '../types/telegram'
import { telegramQueryKeys } from './useTelegramQuery'

export function useTelegramRuntimeStatusQuery(accountId: MaybeRefOrGetter<string | null>) {
  return useQuery<TelegramRuntimeStatus>({
    queryKey: computed(() => [...telegramQueryKeys.runtime, toValue(accountId) ?? 'none']),
    queryFn: () => fetchTelegramRuntimeStatus(toValue(accountId) ?? ''),
    enabled: computed(() => Boolean(toValue(accountId))),
  })
}

function invalidateTelegramRuntime(queryClient: ReturnType<typeof useQueryClient>) {
  queryClient.invalidateQueries({ queryKey: telegramQueryKeys.runtime })
  queryClient.invalidateQueries({ queryKey: telegramQueryKeys.accounts })
}

export function useStartTelegramRuntimeMutation() {
  const queryClient = useQueryClient()
  return useMutation({
    mutationFn: (request: { account_id: string }) => startTelegramRuntime(request),
    onSuccess: () => invalidateTelegramRuntime(queryClient),
  })
}

export function useStopTelegramRuntimeMutation() {
  const queryClient = useQueryClient()
  return useMutation({
    mutationFn: (request: { account_id: string }) => stopTelegramRuntime(request),
    onSuccess: () => invalidateTelegramRuntime(queryClient),
  })
}

export function useRestartTelegramRuntimeMutation() {
  const queryClient = useQueryClient()
  return useMutation({
    mutationFn: (request: { account_id: string }) => restartTelegramRuntime(request),
    onSuccess: () => invalidateTelegramRuntime(queryClient),
  })
}
```

### `frontend/src/integrations/telegram/stores/telegramCommandAudit.test.ts`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/integrations/telegram/stores/telegramCommandAudit.test.ts`
- Size bytes / Размер в байтах: `13449`
- Included characters / Включено символов: `12000`
- Truncated / Обрезано: `yes`

```typescript
import { describe, expect, it } from 'vitest'
import {
  isTelegramCommandDeadLetter,
  telegramCommandAuditState,
  telegramCommandSubject,
  telegramCommandRetrySummary,
} from './telegramCommandAudit'
import type { TelegramProviderWriteCommand } from '../types/telegram'

function command(overrides: Partial<TelegramProviderWriteCommand>): TelegramProviderWriteCommand {
  return {
    command_id: 'cmd-1',
    account_id: 'acct-1',
    command_kind: 'edit',
    idempotency_key: 'idem-1',
    provider_chat_id: 'chat-1',
    provider_message_id: 'msg-1',
    target_ref: {},
    payload: {},
    capability_state: 'available',
    action_class: 'provider_write',
    confirmation_decision: 'not_required',
    status: 'queued',
    retry_count: 0,
    max_retries: 3,
    last_error: null,
    result_payload: {},
    audit_metadata: {},
    actor_id: 'makosh-frontend',
    happened_at: '2026-06-17T10:00:00Z',
    next_attempt_at: null,
    last_attempt_at: null,
    locked_at: null,
    locked_by: null,
    provider_observed_at: null,
    provider_state: {},
    reconciliation_status: 'not_observed',
    reconciled_at: null,
    dead_lettered_at: null,
    completed_at: null,
    created_at: '2026-06-17T10:00:00Z',
    updated_at: '2026-06-17T10:00:00Z',
    ...overrides,
  }
}

describe('telegram command audit projection', () => {
  it('summarizes retry budget without exposing provider internals', () => {
    expect(telegramCommandRetrySummary(command({ retry_count: 1, max_retries: 3 }))).toBe(
      '1/3 retries used'
    )
    expect(telegramCommandRetrySummary(command({ retry_count: 9, max_retries: 3 }))).toBe(
      '3/3 retries used'
    )
    expect(telegramCommandRetrySummary(command({ max_retries: 0 }))).toBe('No retry budget')
  })

  it('marks failed commands with exhausted retry budget as dead-lettered', () => {
    const failed = command({
      status: 'failed',
      retry_count: 3,
      max_retries: 3,
      last_error: 'TDLib request failed',
    })

    expect(isTelegramCommandDeadLetter(failed)).toBe(true)
    expect(telegramCommandAuditState(failed)).toEqual({
      label: 'Dead-lettered',
      detail: 'TDLib request failed',
      tone: 'danger',
      is_dead_letter: true,
    })
  })

  it('keeps retryable failures separate from dead-lettered failures', () => {
    const failed = command({
      status: 'failed',
      retry_count: 1,
      max_retries: 3,
      last_error: 'Transient provider failure',
    })

    expect(isTelegramCommandDeadLetter(failed)).toBe(false)
    expect(telegramCommandAuditState(failed)).toMatchObject({
      label: 'Failed',
      detail: 'Transient provider failure',
      tone: 'warning',
      is_dead_letter: false,
    })
  })

  it('treats explicit durable dead-letter status as final until manual retry', () => {
    const failed = command({
      status: 'dead_letter',
      retry_count: 1,
      max_retries: 3,
      dead_lettered_at: '2026-06-17T10:01:00Z',
      last_error: 'Unsupported command kind',
    })

    expect(isTelegramCommandDeadLetter(failed)).toBe(true)
    expect(telegramCommandAuditState(failed)).toMatchObject({
      label: 'Dead-lettered',
      detail: 'Unsupported command kind',
      tone: 'danger',
      is_dead_letter: true,
    })
  })

  it('shows upload progress detail for executing media commands when provider state supplies it', () => {
    const executing = command({
      command_kind: 'send_media',
      status: 'executing',
      provider_state: {
        upload_phase: 'dispatching_to_provider',
        progress_detail: 'Uploading local media to Telegram',
      },
    })

    expect(telegramCommandAuditState(executing)).toMatchObject({
      label: 'Executing',
      detail: 'Uploading local media to Telegram',
      tone: 'progress',
      is_dead_letter: false,
    })
  })

  it('formats targeted mark-read commands as readable progress instead of raw message ids', () => {
    const executing = command({
      command_kind: 'mark_read',
      status: 'executing',
      provider_message_id: 'chat-1:777',
    })
    const completed = command({
      command_kind: 'mark_read',
      status: 'completed',
      provider_message_id: 'chat-1:777',
      provider_state: {
        last_read_inbox_message_id: 'chat-1:778',
      },
    })

    expect(telegramCommandSubject(executing)).toBe('Read through chat-1:777')
    expect(telegramCommandAuditState(executing).detail).toBe('Read through chat-1:777')
    expect(telegramCommandAuditState(completed).detail).toBe('Read through chat-1:778')
  })

  it('formats mark-unread commands without leaking provider-specific placeholders', () => {
    const commandRow = command({
      command_kind: 'mark_unread',
      provider_message_id: null,
    })

    expect(telegramCommandSubject(commandRow)).toBe('Mark chat unread')
  })

  it('formats folder add/remove commands as readable chat-folder actions', () => {
    const addQueued = command({
      command_kind: 'folder_add',
      provider_message_id: null,
      payload: {
        provider_folder_id: 7,
      },
    })
    const removeCompleted = command({
      command_kind: 'folder_remove',
      provider_message_id: null,
      status: 'completed',
      payload: {
        provider_folder_id: 9,
      },
      provider_state: {
        provider_folder_id: 9,
      },
    })

    expect(telegramCommandSubject(addQueued)).toBe('Add chat to folder 7')
    expect(telegramCommandAuditState(addQueued).detail).toBe('Add to folder 7')
    expect(telegramCommandSubject(removeCompleted)).toBe('Remove chat from folder 9')
    expect(telegramCommandAuditState(removeCompleted).detail).toBe(
      'Folder 9 removal observed on provider'
    )
  })

  it('describes provider-observed mark-unread mismatch as a reconciliation outcome', () => {
    const mismatch = command({
      command_kind: 'mark_unread',
      provider_message_id: null,
      status: 'failed',
      last_error: 'Provider observed a different unread state than requested',
      reconciliation_status: 'mismatch',
      provider_state: {
        observed_is_marked_as_unread: false,
      },
    })

    expect(telegramCommandSubject(mismatch)).toBe('Mark chat unread')
    expect(telegramCommandAuditState(mismatch)).toMatchObject({
      label: 'Failed',
      detail: 'Provider mismatch · chat is still read',
      tone: 'warning',
      is_dead_letter: false,
    })
  })

  it('describes edit reconciliation from provider-observed text state', () => {
    const queued = command({
      command_kind: 'edit',
      payload: {
        new_text: 'Queued provider edit body',
      },
    })
    const completed = command({
      command_kind: 'edit',
      status: 'completed',
      provider_state: {
        body_text: 'Provider observed edited body',
      },
    })

    expect(telegramCommandSubject(queued)).toBe('Edit message')
    expect(telegramCommandAuditState(queued).detail).toBe('Target text · 25 chars')
    expect(telegramCommandAuditState(completed).detail).toBe(
      'Provider text observed · 29 chars'
    )
  })

  it('describes provider-observed edit mismatch as a reconciliation outcome', () => {
    const mismatch = command({
      command_kind: 'edit',
      status: 'failed',
      last_error: 'Provider observed a different message body than requested',
      reconciliation_status: 'mismatch',
      provider_state: {
        expected_body_text: 'Expected provider edit body',
        observed_body_text: 'Observed provider body',
      },
    })

    expect(telegramCommandSubject(mismatch)).toBe('Edit message')
    expect(telegramCommandAuditState(mismatch)).toMatchObject({
      label: 'Failed',
      detail: 'Provider mismatch · expected 27 chars, observed 22 chars',
      tone: 'warning',
      is_dead_letter: false,
    })
  })

  it('describes delete reconciliation from provider-observed tombstone state', () => {
    const queued = command({
      command_kind: 'delete',
      payload: {
        reason_class: 'deleted_by_owner',
      },
    })
    const completed = command({
      command_kind: 'delete',
      status: 'completed',
      provider_state: {
        is_deleted: true,
      },
    })

    expect(telegramCommandSubject(queued)).toBe('Delete message')
    expect(telegramCommandAuditState(queued).detail).toBe(
      'Delete requested · deleted_by_owner'
    )
    expect(telegramCommandAuditState(completed).detail).toBe('Provider delete observed')
  })

  it('describes reaction reconciliation from provider-observed chosen state', () => {
    const queued = command({
      command_kind: 'react',
      payload: {
        reaction_emoji: '👍',
      },
    })
    const completed = command({
      command_kind: 'unreact',
      status: 'completed',
      provider_state: {
        reaction_emoji: '👍',
        is_chosen: false,
      },
    })

    expect(telegramCommandSubject(queued)).toBe('Add reaction 👍')
    expect(telegramCommandAuditState(queued).detail).toBe('Add reaction 👍')
    expect(telegramCommandSubject(completed)).toBe('Remove reaction 👍')
    expect(telegramCommandAuditState(completed).detail).toBe(
      'Reaction 👍 absent on provider'
    )
  })

  it('describes provider-observed reaction mismatch as a reconciliation outcome', () => {
    const mismatch = command({
      command_kind: 'react',
      status: 'failed',
      last_error: 'Provider observed a different reaction state than requested',
      reconciliation_status: 'mismatch',
      provider_state: {
        reaction_emoji: '👍',
        observed_is_chosen: false,
      },
      payload: {
        reaction_emoji: '👍',
      },
    })

    expect(telegramCommandSubject(mismatch)).toBe('Add reaction 👍')
    expect(telegramCommandAuditState(mismatch)).toMatchObject({
      label: 'Failed',
      detail: 'Provider mismatch · reaction 👍 is still absent',
      tone: 'warning',
      is_dead_letter: false,
    })
  })

  it('describes provider-observed pin mismatch as a reconciliation outcome', () => {
    const mismatch = command({
      command_kind: 'unpin',
      status: 'failed',
      last_error: 'Provider observed a different pin state than requested',
      reconciliation_status: 'mismatch',
      provider_state: {
        observed_is_pinned: true,
      },
      payload: {
        is_pinned: false,
      },
    })

    expect(telegramCommandSubject(mismatch)).toBe('Unpin message')
    expect(telegramCommandAuditState(mismatch)).toMatchObject({
      label: 'Failed',
      detail: 'Provider mismatch · message is still pinned',
      tone: 'warning',
      is_dead_letter: false,
    })
  })

  it('distinguishes dialog pin commands from message pin commands in user-facing subjects', () => {
    const chatPin = command({
      command_kind: 'pin',
      provider_message_id: null,
    })
    const chatUnpinMismatch = command({
      command_kind: 'unpin',
      provider_message_id: null,
      status: 'failed',
      last_error: 'Provider observed a different dialog pin state than requested',
      reconciliation_status: 'mismatch',
      provider_state: {
        observed_is_pinned: true,
      },
    })

    expect(telegramCommandSubject(chatPin)).toBe('Pin chat')
    expect(telegramCommandSubject(chatUnpinMismatch)).toBe('Unpin chat')
    expect(telegramCommandAuditState(chatUnpinMismatch).detail).toBe(
      'Provider mismatch · chat is still pinned'
    )
  })

  it('describes provider-observed archive mismatch as a reconciliation outcome', () => {
    const mismatch = command({
      command_kind: 'unarchive',
      provider_message_id: null,
      status: 'failed',
      last_error: 'Provider observed a different archive state than requested',
      reconciliation_status: 'mismatch',
      provider_state: {
        observed_is_archived: true,
      },
    })

    expect(telegramCommandSubject(mismatch)).toBe('Unarchive chat')
    expect(telegramCommandAuditState(mismatch)).toMatchObject({
      label: 'Failed',
      detail: 'Provider mismatch · chat is
```
_Source file truncated after 12000 characters. / Исходный файл обрезан после 12000 символов._
