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

- Chunk ID / ID чанка: `143-source-frontend-part-003`
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

### `frontend/src/domains/communications/api/readReceipts.ts`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/domains/communications/api/readReceipts.ts`
- Size bytes / Размер в байтах: `442`
- Included characters / Включено символов: `442`
- Truncated / Обрезано: `no`

```typescript
import { ApiClient } from '../../../platform/api/ApiClient'
import type {
  CommunicationReadReceipt,
  NewCommunicationReadReceipt
} from '../types/readReceipts'

export async function recordReadReceipt(
  request: NewCommunicationReadReceipt
): Promise<CommunicationReadReceipt> {
  return ApiClient.instance.post<CommunicationReadReceipt>(
    '/api/v1/communications/read-receipts',
    request,
    'Read receipt recording failed'
  )
}
```

### `frontend/src/domains/communications/api/savedSearchApi.ts`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/domains/communications/api/savedSearchApi.ts`
- Size bytes / Размер в байтах: `1183`
- Included characters / Включено символов: `1183`
- Truncated / Обрезано: `no`

```typescript
import {
  createCommunicationSavedSearchConnect,
  deleteCommunicationSavedSearchConnect,
  fetchCommunicationSavedSearchesConnect,
  updateCommunicationSavedSearchConnect
} from './connectCommunications'
import type {
  SavedSearchDeleteResponse,
  SavedSearchInput,
  SavedSearchListResponse,
  SavedSearchUpdate,
  CommunicationSavedSearch
} from '../types/savedSearches'

export async function fetchSavedSearches(
  smartFolder?: boolean,
  accountId?: string,
  limit = 500,
  cursor?: string | null
): Promise<SavedSearchListResponse> {
  return fetchCommunicationSavedSearchesConnect(smartFolder, accountId, limit, cursor ?? undefined)
}

export async function createSavedSearch(request: SavedSearchInput): Promise<CommunicationSavedSearch> {
  return createCommunicationSavedSearchConnect(request)
}

export async function updateSavedSearch(
  savedSearchId: string,
  request: SavedSearchUpdate
): Promise<CommunicationSavedSearch> {
  return updateCommunicationSavedSearchConnect(savedSearchId, request)
}

export async function deleteSavedSearch(savedSearchId: string): Promise<SavedSearchDeleteResponse> {
  return deleteCommunicationSavedSearchConnect(savedSearchId)
}
```

### `frontend/src/domains/communications/api/sendApi.ts`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/domains/communications/api/sendApi.ts`
- Size bytes / Размер в байтах: `554`
- Included characters / Включено символов: `554`
- Truncated / Обрезано: `no`

```typescript
import { redirectMessageConnect, sendCommunicationConnect } from './connectCommunications'
import type { SendCommunicationRequest, SendCommunicationResponse, RedirectMessageRequest } from '../types/communications'

export async function sendEmail(request: SendCommunicationRequest): Promise<SendCommunicationResponse> {
  return sendCommunicationConnect(request)
}

export async function redirectMessage(
  messageId: string,
  request: RedirectMessageRequest
): Promise<SendCommunicationResponse> {
  return redirectMessageConnect(messageId, request)
}
```

### `frontend/src/domains/communications/api/telegramBusinessApi.test.ts`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/domains/communications/api/telegramBusinessApi.test.ts`
- Size bytes / Размер в байтах: `6656`
- Included characters / Включено символов: `6656`
- Truncated / Обрезано: `no`

```typescript
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { ApiClient } from '../../../platform/api'
import {
  fetchTelegramBusinessMessages,
  forwardTelegramBusinessMessage,
  pinTelegramBusinessMessage,
  replyToTelegramBusinessMessage,
  searchTelegramBusinessTopics,
  sendTelegramBusinessMessage,
} from './telegramBusinessApi'

describe('telegram business API', () => {
  beforeEach(() => {
    ApiClient.resetForTests()
    ApiClient.init('http://127.0.0.1:8080', 'test-secret')
  })

  afterEach(() => {
    vi.unstubAllGlobals()
    ApiClient.resetForTests()
  })

  it('searches projected topics through Communications routes', async () => {
    const fetchMock = vi.fn().mockResolvedValueOnce(
      new Response(JSON.stringify({ items: [] }), {
        status: 200,
        headers: { 'Content-Type': 'application/json' },
      })
    )
    vi.stubGlobal('fetch', fetchMock)

    await searchTelegramBusinessTopics('chat-42 ', '  architecture docs ', 25)

    expect(fetchMock).toHaveBeenCalledOnce()
    const [url] = fetchMock.mock.calls[0]
    expect(url).toContain('/api/v1/communications/topics/search?')
    expect(url).toContain('q=architecture+docs')
    expect(url).toContain('telegram_chat_id=chat-42')
    expect(url).toContain('limit=25')
  })

  it('adapts canonical Communication messages to Telegram message DTOs', async () => {
    const fetchMock = vi.fn().mockResolvedValueOnce(
      new Response(
        JSON.stringify({
          items: [
            {
              message_id: 'msg-1',
              raw_record_id: 'raw-1',
              account_id: 'telegram-account-1',
              provider_record_id: 'provider-message-1',
              subject: 'General',
              sender: 'telegram:user:42',
              recipients: [],
              body_text_preview: 'hello from projection',
              occurred_at: '2026-06-20T10:00:00Z',
              projected_at: '2026-06-20T10:00:01Z',
              channel_kind: 'telegram_user',
              conversation_id: 'chat-1',
              sender_display_name: 'Ada',
              delivery_state: 'received',
              workflow_state: 'new',
              importance_score: null,
              ai_category: null,
              ai_summary: null,
              ai_summary_generated_at: null,
              message_metadata: { is_pinned: true },
              attachment_count: 0,
              local_state: 'active',
              local_state_changed_at: null,
            },
          ],
          next_cursor: null,
          has_more: false,
        }),
        { status: 200, headers: { 'Content-Type': 'application/json' } }
      )
    )
    vi.stubGlobal('fetch', fetchMock)

    const response = await fetchTelegramBusinessMessages('telegram-account-1', 'chat-1', 25)

    expect(response.items).toEqual([
      {
        message_id: 'msg-1',
        raw_record_id: 'raw-1',
        account_id: 'telegram-account-1',
        provider_message_id: 'provider-message-1',
        provider_chat_id: 'chat-1',
        chat_title: 'General',
        sender: 'telegram:user:42',
        sender_display_name: 'Ada',
        text: 'hello from projection',
        occurred_at: '2026-06-20T10:00:00Z',
        projected_at: '2026-06-20T10:00:01Z',
        channel_kind: 'telegram_user',
        delivery_state: 'received',
        metadata: { is_pinned: true },
      },
    ])
    const [url] = fetchMock.mock.calls[0]
    expect(url).toContain('/api/v1/communications/messages?')
    expect(url).toContain('channel_kind=telegram')
    expect(url).toContain('conversation_id=chat-1')
  })

  it('uses the provider-neutral pin response shape', async () => {
    const fetchMock = vi.fn().mockResolvedValueOnce(
      new Response(JSON.stringify({ message_id: 'msg-1', pinned: true }), {
        status: 200,
        headers: { 'Content-Type': 'application/json' },
      })
    )
    vi.stubGlobal('fetch', fetchMock)

    const response = await pinTelegramBusinessMessage({ message_id: 'msg-1' })

    expect(response).toEqual({ message_id: 'msg-1', pinned: true })
    const [url, init] = fetchMock.mock.calls[0]
    expect(url).toContain('/api/v1/communications/messages/msg-1/pin')
    expect(init.method).toBe('POST')
    expect(JSON.parse(init.body as string)).toEqual({})
  })

  it('uses the provider-neutral command response shape for message writes', async () => {
    const commandResponse = {
      message_id: 'msg-1',
      raw_record_id: 'raw-1',
      conversation_id: 'chat-1',
      provider_chat_id: 'chat-1',
      provider_message_id: null,
      channel_kind: 'telegram_user',
      status: 'queued',
      command_id: 'command-1',
      provider: 'telegram',
    }
    const fetchMock = vi
      .fn()
      .mockResolvedValueOnce(
        new Response(JSON.stringify(commandResponse), {
          status: 200,
          headers: { 'Content-Type': 'application/json' },
        })
      )
      .mockResolvedValueOnce(
        new Response(JSON.stringify({ ...commandResponse, provider_message_id: 'provider-reply-1' }), {
          status: 200,
          headers: { 'Content-Type': 'application/json' },
        })
      )
      .mockResolvedValueOnce(
        new Response(JSON.stringify({ ...commandResponse, command_id: 'command-forward-1' }), {
          status: 200,
          headers: { 'Content-Type': 'application/json' },
        })
      )
    vi.stubGlobal('fetch', fetchMock)

    await expect(sendTelegramBusinessMessage({
      account_id: 'account-1',
      provider_chat_id: 'chat-1',
      text: 'hello',
    })).resolves.toEqual(commandResponse)
    await expect(replyToTelegramBusinessMessage({
      message_id: 'msg-1',
      text: 'reply',
    })).resolves.toMatchObject({ provider_message_id: 'provider-reply-1' })
    await expect(forwardTelegramBusinessMessage({
      message_id: 'msg-1',
      provider_chat_id: 'chat-2',
    })).resolves.toMatchObject({ command_id: 'command-forward-1' })

    const [sendUrl, sendInit] = fetchMock.mock.calls[0]
    expect(sendUrl).toContain('/api/v1/communications/conversations/chat-1/messages')
    expect(JSON.parse(sendInit.body as string)).toEqual({ account_id: 'account-1', text: 'hello' })

    const [replyUrl, replyInit] = fetchMock.mock.calls[1]
    expect(replyUrl).toContain('/api/v1/communications/messages/msg-1/reply')
    expect(JSON.parse(replyInit.body as string)).toEqual({ text: 'reply' })

    const [forwardUrl, forwardInit] = fetchMock.mock.calls[2]
    expect(forwardUrl).toContain('/api/v1/communications/messages/msg-1/forward')
    expect(JSON.parse(forwardInit.body as string)).toEqual({ conversation_id: 'chat-2' })
  })
})
```

### `frontend/src/domains/communications/api/telegramBusinessApi.ts`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/domains/communications/api/telegramBusinessApi.ts`
- Size bytes / Размер в байтах: `15152`
- Included characters / Включено символов: `12000`
- Truncated / Обрезано: `yes`

```typescript
import { ApiClient } from '../../../platform/api/ApiClient'
import type {
  TelegramChatDetailResponse,
  TelegramChatListResponse,
  TelegramChatMemberListResponse,
  TelegramChatActionResponse,
  TelegramChatSearchResponse,
  TelegramMediaSearchResponse,
  TelegramMessageListResponse,
  TelegramMessageSearchResponse,
  TelegramTopicListResponse,
} from '../../../shared/communications/types/telegram'
import type {
  TelegramForwardChainResponse,
  TelegramLifecycleResponse,
  TelegramMessageTombstoneListResponse,
  TelegramMessageVersionListResponse,
  TelegramReactionListResponse,
  TelegramReactionRequest,
  TelegramReactionResponse,
  TelegramReplyChainResponse,
  TelegramMessage,
  TelegramProviderKind,
} from '../../../shared/communications/types/telegram'
import type { TelegramRawMessageResponse } from '../../../shared/communications/types/telegramRawEvidence'
import type {
  TelegramTopicCreateRequest,
  TelegramTopicLifecycleResponse,
} from '../../../shared/communications/types/telegramTopics'
import type { AttachmentPreviewResponse } from '../types/attachments'
import type {
  CommunicationProviderMessageCommandResponse,
  CommunicationMessageSummary,
  CommunicationMessagesResponse,
  MessagePinToggleResponse,
} from '../types/communications'

export async function fetchTelegramBusinessChats(accountId?: string, limit = 50): Promise<TelegramChatListResponse> {
  const params = new URLSearchParams({ limit: String(Math.trunc(limit)) })
  if (accountId?.trim()) params.set('account_id', accountId.trim())
  return ApiClient.instance.get<TelegramChatListResponse>(
    `/api/v1/communications/conversations?${params.toString()}`,
    'Communication conversations request failed'
  )
}

export async function fetchTelegramBusinessChatDetail(conversationId: string): Promise<TelegramChatDetailResponse> {
  return ApiClient.instance.get<TelegramChatDetailResponse>(
    `/api/v1/communications/conversations/${encodeURIComponent(conversationId)}`,
    'Communication conversation detail request failed'
  )
}

export async function fetchTelegramBusinessChatMembers(
  conversationId: string,
  limit = 50,
  query?: string,
  role?: string,
  cursor?: string
): Promise<TelegramChatMemberListResponse> {
  const params = new URLSearchParams({ limit: String(Math.trunc(limit)) })
  if (query?.trim()) params.set('query', query.trim())
  if (role?.trim()) params.set('role', role.trim())
  if (cursor?.trim()) params.set('cursor', cursor.trim())
  return ApiClient.instance.get<TelegramChatMemberListResponse>(
    `/api/v1/communications/conversations/${encodeURIComponent(conversationId)}/members?${params.toString()}`,
    'Communication conversation members request failed'
  )
}

export async function fetchTelegramBusinessMessages(
  accountId?: string,
  providerChatId?: string,
  limit = 50
): Promise<TelegramMessageListResponse> {
  const params = new URLSearchParams({ limit: String(Math.trunc(limit)), channel_kind: 'telegram' })
  if (accountId?.trim()) params.set('account_id', accountId.trim())
  if (providerChatId?.trim()) params.set('conversation_id', providerChatId.trim())
  const response = await ApiClient.instance.get<CommunicationMessagesResponse>(
    `/api/v1/communications/messages?${params.toString()}`,
    'Communication messages request failed'
  )
  return { items: response.items.map(communicationMessageToTelegramMessage) }
}

export async function searchTelegramBusinessChats(params: {
  q: string
  account_id?: string
  limit?: number
}): Promise<TelegramChatSearchResponse> {
  const query = new URLSearchParams({ q: params.q.trim() })
  if (params.account_id?.trim()) query.set('account_id', params.account_id.trim())
  if (params.limit != null) query.set('limit', String(Math.trunc(params.limit)))
  return ApiClient.instance.get<TelegramChatSearchResponse>(
    `/api/v1/communications/conversations/search?${query.toString()}`,
    'Communication conversation search failed'
  )
}

export async function searchTelegramBusinessMessages(params: {
  q: string
  account_id?: string
  provider_chat_id?: string
  limit?: number
}): Promise<TelegramMessageSearchResponse> {
  const query = new URLSearchParams({ q: params.q.trim() })
  if (params.account_id?.trim()) query.set('account_id', params.account_id.trim())
  if (params.provider_chat_id?.trim()) query.set('provider_chat_id', params.provider_chat_id.trim())
  if (params.limit != null) query.set('limit', String(Math.trunc(params.limit)))
  return ApiClient.instance.get<TelegramMessageSearchResponse>(
    `/api/v1/communications/search/messages?${query.toString()}`,
    'Communication message search failed'
  )
}

export async function searchTelegramBusinessMedia(params: {
  q?: string
  account_id?: string
  provider_chat_id?: string
  kind?: string
  limit?: number
}): Promise<TelegramMediaSearchResponse> {
  const query = new URLSearchParams()
  if (params.q?.trim()) query.set('q', params.q.trim())
  if (params.account_id?.trim()) query.set('account_id', params.account_id.trim())
  if (params.provider_chat_id?.trim()) query.set('provider_chat_id', params.provider_chat_id.trim())
  if (params.kind?.trim()) query.set('kind', params.kind.trim())
  if (params.limit != null) query.set('limit', String(Math.trunc(params.limit)))
  return ApiClient.instance.get<TelegramMediaSearchResponse>(
    `/api/v1/communications/search/media?${query.toString()}`,
    'Communication media search failed'
  )
}

export async function fetchTelegramBusinessPinnedMessages(params: {
  telegram_chat_id: string
  limit?: number
}): Promise<TelegramMessageListResponse> {
  const query = new URLSearchParams()
  if (params.limit != null) query.set('limit', String(Math.trunc(params.limit)))
  return ApiClient.instance.get<TelegramMessageListResponse>(
    `/api/v1/communications/conversations/${encodeURIComponent(params.telegram_chat_id)}/pinned-messages?${query.toString()}`,
    'Communication pinned messages request failed'
  )
}

export async function sendTelegramBusinessMessage(request: {
  account_id: string
  provider_chat_id: string
  text: string
}): Promise<CommunicationProviderMessageCommandResponse> {
  return ApiClient.instance.post<CommunicationProviderMessageCommandResponse>(
    `/api/v1/communications/conversations/${encodeURIComponent(request.provider_chat_id)}/messages`,
    { account_id: request.account_id, text: request.text },
    'Communication message send failed'
  )
}

export async function replyToTelegramBusinessMessage(params: {
  message_id: string
  text: string
}): Promise<CommunicationProviderMessageCommandResponse> {
  return ApiClient.instance.post<CommunicationProviderMessageCommandResponse>(
    `/api/v1/communications/messages/${encodeURIComponent(params.message_id)}/reply`,
    { text: params.text },
    'Communication reply failed'
  )
}

export async function forwardTelegramBusinessMessage(params: {
  message_id: string
  provider_chat_id: string
}): Promise<CommunicationProviderMessageCommandResponse> {
  return ApiClient.instance.post<CommunicationProviderMessageCommandResponse>(
    `/api/v1/communications/messages/${encodeURIComponent(params.message_id)}/forward`,
    { conversation_id: params.provider_chat_id },
    'Communication forward failed'
  )
}

export async function editTelegramBusinessMessage(params: {
  message_id: string
  command_id?: string
  account_id: string
  provider_chat_id: string
  provider_message_id: string
  new_text: string
}): Promise<TelegramLifecycleResponse> {
  return ApiClient.instance.patch<TelegramLifecycleResponse>(
    `/api/v1/communications/messages/${encodeURIComponent(params.message_id)}`,
    params,
    'Communication message edit failed'
  )
}

export async function deleteTelegramBusinessMessage(params: {
  message_id: string
  command_id?: string
  account_id: string
  provider_chat_id: string
  provider_message_id: string
  reason_class?: string
  actor_class?: string
  is_provider_delete?: boolean
}): Promise<TelegramLifecycleResponse> {
  return ApiClient.instance.deleteWithBody<TelegramLifecycleResponse>(
    `/api/v1/communications/messages/${encodeURIComponent(params.message_id)}`,
    params,
    'Communication message delete failed'
  )
}

export async function restoreTelegramBusinessMessageVisibility(params: {
  message_id: string
  command_id?: string
  account_id: string
  provider_chat_id: string
  provider_message_id: string
  reason?: string
}): Promise<TelegramLifecycleResponse> {
  return ApiClient.instance.post<TelegramLifecycleResponse>(
    `/api/v1/communications/messages/${encodeURIComponent(params.message_id)}/restore-visibility`,
    params,
    'Communication message restore failed'
  )
}

export async function pinTelegramBusinessMessage(params: {
  message_id: string
}): Promise<MessagePinToggleResponse> {
  return ApiClient.instance.post<MessagePinToggleResponse>(
    `/api/v1/communications/messages/${encodeURIComponent(params.message_id)}/pin`,
    {},
    'Communication message pin failed'
  )
}

export async function markTelegramBusinessMessageRead(params: {
  message_id: string
  account_id: string
  provider_chat_id: string
}): Promise<TelegramChatActionResponse> {
  return ApiClient.instance.post<TelegramChatActionResponse>(
    `/api/v1/communications/messages/${encodeURIComponent(params.message_id)}/mark-read`,
    { account_id: params.account_id, provider_chat_id: params.provider_chat_id },
    'Communication message mark read failed'
  )
}

export async function fetchTelegramBusinessMessageVersions(messageId: string): Promise<TelegramMessageVersionListResponse> {
  return ApiClient.instance.get<TelegramMessageVersionListResponse>(
    `/api/v1/communications/messages/${encodeURIComponent(messageId)}/versions`,
    'Communication message versions failed'
  )
}

export async function fetchTelegramBusinessMessageTombstones(messageId: string): Promise<TelegramMessageTombstoneListResponse> {
  return ApiClient.instance.get<TelegramMessageTombstoneListResponse>(
    `/api/v1/communications/messages/${encodeURIComponent(messageId)}/tombstones`,
    'Communication message tombstones failed'
  )
}

export async function fetchTelegramBusinessReplyChain(messageId: string): Promise<TelegramReplyChainResponse> {
  return ApiClient.instance.get<TelegramReplyChainResponse>(
    `/api/v1/communications/messages/${encodeURIComponent(messageId)}/reply-chain`,
    'Communication reply chain failed'
  )
}

export async function fetchTelegramBusinessForwardChain(messageId: string): Promise<TelegramForwardChainResponse> {
  return ApiClient.instance.get<TelegramForwardChainResponse>(
    `/api/v1/communications/messages/${encodeURIComponent(messageId)}/forward-chain`,
    'Communication forward chain failed'
  )
}

export async function fetchTelegramBusinessReactions(messageId: string): Promise<TelegramReactionListResponse> {
  return ApiClient.instance.get<TelegramReactionListResponse>(
    `/api/v1/communications/messages/${encodeURIComponent(messageId)}/reactions`,
    'Communication reactions failed'
  )
}

export async function addTelegramBusinessReaction(messageId: string, request: TelegramReactionRequest): Promise<TelegramReactionResponse> {
  return ApiClient.instance.post<TelegramReactionResponse>(
    `/api/v1/communications/messages/${encodeURIComponent(messageId)}/reactions`,
    request,
    'Communication reaction add failed'
  )
}

export async function removeTelegramBusinessReaction(messageId: string, request: TelegramReactionRequest): Promise<TelegramReactionResponse> {
  const params = new URLSearchParams({
    account_id: request.account_id,
    provider_chat_id: request.provider_chat_id,
    provider_message_id: request.provider_message_id,
    reaction_emoji: request.reaction_emoji,
    sender_id: request.sender_id,
  })
  if (request.sender_display_name) params.set('sender_display_name', request.sender_display_name)
  if (request.command_id) params.set('command_id', request.command_id)
  return ApiClient.instance.delete<TelegramReac
```
_Source file truncated after 12000 characters. / Исходный файл обрезан после 12000 символов._

### `frontend/src/domains/communications/api/threadApi.ts`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/domains/communications/api/threadApi.ts`
- Size bytes / Размер в байтах: `995`
- Included characters / Включено символов: `995`
- Truncated / Обрезано: `no`

```typescript
import {
  fetchCommunicationThreadMessagesConnect,
  fetchCommunicationThreadsConnect,
  translateCommunicationThreadConnect
} from './connectCommunications'
import type { ThreadListResponse, ThreadMessagesResponse } from '../types/communications'
import type { ThreadTranslationResponse } from '../types/multilingual'

export async function fetchThreads(
  accountId?: string,
  limit = 50,
  cursor?: string | null
): Promise<ThreadListResponse> {
  return fetchCommunicationThreadsConnect(accountId, limit, cursor ?? undefined)
}

export async function fetchThreadMessages(
  accountId: string,
  subject: string,
  limit = 50
): Promise<ThreadMessagesResponse> {
  return fetchCommunicationThreadMessagesConnect(accountId, subject, limit)
}

export async function translateThread(
  accountId: string,
  subject: string,
  targetLanguage: string,
  limit = 50
): Promise<ThreadTranslationResponse> {
  return translateCommunicationThreadConnect(accountId, subject, targetLanguage, limit)
}
```

### `frontend/src/domains/communications/api/whatsappBusinessApi.test.ts`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/domains/communications/api/whatsappBusinessApi.test.ts`
- Size bytes / Размер в байтах: `17189`
- Included characters / Включено символов: `12000`
- Truncated / Обрезано: `yes`

```typescript
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { ApiClient } from '../../../platform/api'
import {
  addWhatsappBusinessReaction,
  archiveWhatsappBusinessConversation,
  deleteWhatsappBusinessMessage,
  editWhatsappBusinessMessage,
  fetchWhatsappBusinessReactions,
  fetchWhatsappWebBusinessConversationDetail,
  fetchWhatsappWebBusinessConversationMembers,
  fetchWhatsappWebBusinessConversations,
  fetchWhatsappWebBusinessMessages,
  fetchWhatsappWebBusinessPinnedMessages,
  forwardWhatsappBusinessMessage,
  markWhatsappBusinessConversationRead,
  markWhatsappBusinessConversationUnread,
  muteWhatsappBusinessConversation,
  pinWhatsappBusinessConversation,
  pinWhatsappBusinessMessage,
  replyToWhatsappBusinessMessage,
  removeWhatsappBusinessReaction,
  sendWhatsappBusinessMessage,
  searchWhatsappWebBusinessMedia,
  searchWhatsappWebBusinessMessages,
  unarchiveWhatsappBusinessConversation,
  unmuteWhatsappBusinessConversation,
  unpinWhatsappBusinessConversation,
} from './whatsappBusinessApi'

describe('WhatsApp business API', () => {
  beforeEach(() => {
    ApiClient.resetForTests()
    ApiClient.init('http://127.0.0.1:8080', 'test-secret')
  })

  afterEach(() => {
    vi.unstubAllGlobals()
    ApiClient.resetForTests()
  })

  it('reads projected whatsapp conversations from the provider-neutral route', async () => {
    const fetchMock = vi.fn().mockResolvedValueOnce(
      new Response(
        JSON.stringify({
          items: [
            {
              conversation_id: 'wa-chat-1',
              account_id: 'whatsapp-account-1',
              provider_chat_id: 'wa-chat-1',
              title: 'Family',
              last_message_at: '2026-06-20T11:00:00Z',
              metadata: { channel_kind: 'whatsapp_web' },
              created_at: '2026-06-20T11:00:00Z',
              updated_at: '2026-06-20T11:00:01Z',
            },
            {
              conversation_id: 'tg-chat-1',
              account_id: 'telegram-account-1',
              provider_chat_id: 'tg-chat-1',
              title: 'Telegram',
              last_message_at: '2026-06-20T11:00:00Z',
              metadata: { channel_kind: 'telegram_user' },
              created_at: '2026-06-20T11:00:00Z',
              updated_at: '2026-06-20T11:00:01Z',
            },
          ],
        }),
        { status: 200, headers: { 'Content-Type': 'application/json' } }
      )
    )
    vi.stubGlobal('fetch', fetchMock)

    const response = await fetchWhatsappWebBusinessConversations('whatsapp-account-1', 10)

    expect(response.items).toEqual([
      expect.objectContaining({
        conversation_id: 'wa-chat-1',
        provider_chat_id: 'wa-chat-1',
      }),
    ])
    const [url] = fetchMock.mock.calls[0]
    expect(url).toContain('/api/v1/communications/conversations?')
    expect(url).toContain('account_id=whatsapp-account-1')
    expect(url).toContain('channel_kind=whatsapp_web')
  })

  it('uses provider-neutral whatsapp conversation detail and member routes', async () => {
    const fetchMock = vi
      .fn()
      .mockResolvedValueOnce(
        new Response(
          JSON.stringify({
            item: {
              conversation_id: 'wa-chat-1',
              account_id: 'whatsapp-account-1',
              provider_chat_id: 'wa-chat-1',
              title: 'Family',
              last_message_at: '2026-06-20T11:00:00Z',
              metadata: { channel_kind: 'whatsapp_web' },
              created_at: '2026-06-20T11:00:00Z',
              updated_at: '2026-06-20T11:00:01Z',
            },
          }),
          { status: 200, headers: { 'Content-Type': 'application/json' } }
        )
      )
      .mockResolvedValueOnce(
        new Response(
          JSON.stringify({
            items: [],
            next_cursor: null,
          }),
          { status: 200, headers: { 'Content-Type': 'application/json' } }
        )
      )
    vi.stubGlobal('fetch', fetchMock)

    await fetchWhatsappWebBusinessConversationDetail('wa-chat-1')
    await fetchWhatsappWebBusinessConversationMembers('wa-chat-1', 25, 'bea', 'admin', '50')

    const [detailUrl] = fetchMock.mock.calls[0]
    expect(detailUrl).toContain('/api/v1/communications/conversations/wa-chat-1')

    const [membersUrl] = fetchMock.mock.calls[1]
    expect(membersUrl).toContain('/api/v1/communications/conversations/wa-chat-1/members?')
    expect(membersUrl).toContain('limit=25')
    expect(membersUrl).toContain('query=bea')
    expect(membersUrl).toContain('role=admin')
    expect(membersUrl).toContain('cursor=50')
  })

  it('adapts canonical Communication messages to WhatsApp message DTOs', async () => {
    const fetchMock = vi.fn().mockResolvedValueOnce(
      new Response(
        JSON.stringify({
          items: [
            {
              message_id: 'wa-msg-1',
              raw_record_id: 'wa-raw-1',
              account_id: 'whatsapp-account-1',
              provider_record_id: 'provider-wa-1',
              subject: 'Family',
              sender: 'whatsapp:+100000000',
              recipients: [],
              body_text_preview: 'hello from whatsapp',
              occurred_at: '2026-06-20T11:00:00Z',
              projected_at: '2026-06-20T11:00:01Z',
              channel_kind: 'whatsapp_web',
              conversation_id: 'wa-chat-1',
              sender_display_name: 'Bea',
              delivery_state: 'received',
              workflow_state: 'new',
              importance_score: null,
              ai_category: null,
              ai_summary: null,
              ai_summary_generated_at: null,
              message_metadata: { source: 'fixture' },
              attachment_count: 0,
              local_state: 'active',
              local_state_changed_at: null,
            },
          ],
          next_cursor: null,
          has_more: false,
        }),
        { status: 200, headers: { 'Content-Type': 'application/json' } }
      )
    )
    vi.stubGlobal('fetch', fetchMock)

    const response = await fetchWhatsappWebBusinessMessages('whatsapp-account-1', 'wa-chat-1', 10)

    expect(response.items).toEqual([
      {
        message_id: 'wa-msg-1',
        raw_record_id: 'wa-raw-1',
        account_id: 'whatsapp-account-1',
        provider_message_id: 'provider-wa-1',
        provider_chat_id: 'wa-chat-1',
        chat_title: 'Family',
        sender: 'whatsapp:+100000000',
        sender_display_name: 'Bea',
        text: 'hello from whatsapp',
        occurred_at: '2026-06-20T11:00:00Z',
        projected_at: '2026-06-20T11:00:01Z',
        channel_kind: 'whatsapp_web',
        delivery_state: 'received',
        metadata: { source: 'fixture' },
      },
    ])
    const [url] = fetchMock.mock.calls[0]
    expect(url).toContain('/api/v1/communications/messages?')
    expect(url).toContain('channel_kind=whatsapp_web')
    expect(url).toContain('conversation_id=wa-chat-1')
  })

  it('uses provider-neutral whatsapp message search with channel scoping', async () => {
    const fetchMock = vi.fn().mockResolvedValueOnce(
      new Response(
        JSON.stringify({
          query: 'hello',
          items: [],
          total: 0,
        }),
        { status: 200, headers: { 'Content-Type': 'application/json' } }
      )
    )
    vi.stubGlobal('fetch', fetchMock)

    await searchWhatsappWebBusinessMessages({
      q: 'hello',
      account_id: 'whatsapp-account-1',
      provider_chat_id: 'wa-chat-1',
      limit: 20,
    })

    const [url] = fetchMock.mock.calls[0]
    expect(url).toContain('/api/v1/communications/search/messages?')
    expect(url).toContain('channel_kind=whatsapp_web')
    expect(url).toContain('account_id=whatsapp-account-1')
    expect(url).toContain('provider_chat_id=wa-chat-1')
  })

  it('uses provider-neutral whatsapp media search and pinned message routes', async () => {
    const fetchMock = vi
      .fn()
      .mockResolvedValueOnce(
        new Response(JSON.stringify({ items: [] }), {
          status: 200,
          headers: { 'Content-Type': 'application/json' },
        })
      )
      .mockResolvedValueOnce(
        new Response(JSON.stringify({ items: [] }), {
          status: 200,
          headers: { 'Content-Type': 'application/json' },
        })
      )
    vi.stubGlobal('fetch', fetchMock)

    await searchWhatsappWebBusinessMedia({
      q: 'invoice',
      account_id: 'whatsapp-account-1',
      provider_chat_id: 'wa-chat-1',
      kind: 'document',
      limit: 10,
    })
    await fetchWhatsappWebBusinessPinnedMessages({
      conversation_id: 'wa-chat-1',
      limit: 10,
    })

    const [mediaUrl] = fetchMock.mock.calls[0]
    expect(mediaUrl).toContain('/api/v1/communications/search/media?')
    expect(mediaUrl).toContain('channel_kind=whatsapp_web')
    expect(mediaUrl).toContain('kind=document')

    const [pinnedUrl] = fetchMock.mock.calls[1]
    expect(pinnedUrl).toContain('/api/v1/communications/conversations/wa-chat-1/pinned-messages?')
    expect(pinnedUrl).toContain('limit=10')
  })

  it('uses provider-neutral whatsapp message command routes', async () => {
    const fetchMock = vi.fn()
    for (let index = 0; index < 7; index += 1) {
      fetchMock.mockResolvedValueOnce(
        new Response(JSON.stringify({ status: 'queued', pinned: true }), {
          status: 200,
          headers: { 'Content-Type': 'application/json' },
        })
      )
    }
    vi.stubGlobal('fetch', fetchMock)

    await sendWhatsappBusinessMessage({
      account_id: 'whatsapp-account-1',
      provider_chat_id: 'wa-chat-1',
      text: 'hello from panel',
    })
    await replyToWhatsappBusinessMessage({
      message_id: 'wa-msg-1',
      text: 'reply text',
    })
    await forwardWhatsappBusinessMessage({
      message_id: 'wa-msg-1',
      provider_chat_id: 'wa-chat-2',
    })
    await editWhatsappBusinessMessage({
      message_id: 'wa-msg-1',
      account_id: 'whatsapp-account-1',
      provider_chat_id: 'wa-chat-1',
      provider_message_id: 'provider-wa-1',
      new_text: 'edited text',
    })
    await deleteWhatsappBusinessMessage({
      message_id: 'wa-msg-1',
      account_id: 'whatsapp-account-1',
      provider_chat_id: 'wa-chat-1',
      provider_message_id: 'provider-wa-1',
      reason_class: 'deleted_by_owner',
      actor_class: 'owner',
      is_provider_delete: false,
    })
    await pinWhatsappBusinessMessage({
      message_id: 'wa-msg-1',
    })
    await pinWhatsappBusinessConversation({
      conversation_id: 'wa-chat-1',
    })

    const [sendUrl, sendInit] = fetchMock.mock.calls[0]
    expect(sendUrl).toContain('/api/v1/communications/conversations/wa-chat-1/messages')
    expect(sendInit.method).toBe('POST')
    expect(JSON.parse(sendInit.body as string)).toEqual({
      account_id: 'whatsapp-account-1',
      text: 'hello from panel',
    })

    const [replyUrl, replyInit] = fetchMock.mock.calls[1]
    expect(replyUrl).toContain('/api/v1/communications/messages/wa-msg-1/reply')
    expect(replyInit.method).toBe('POST')
    expect(JSON.parse(replyInit.body as string)).toEqual({
      text: 'reply text',
    })

    const [forwardUrl, forwardInit] = fetchMock.mock.calls[2]
    expect(forwardUrl).toContain('/api/v1/communications/messages/wa-msg-1/forward')
    expect(forwardInit.method).toBe('POST')
    expect(JSON.parse(forwardInit.body as string)).toEqual({
      conversation_id: 'wa-chat-2',
    })

    const [editUrl, editInit] = fetchMock.mock.calls[3]
    expect(editUrl).toContain('/api/v1/communications/messages/wa-msg-1')
    expect(editInit.method).toBe('PATCH')
    expect(JSON.parse(editInit.body as string)).toMatchObject({
      account_id: 'whatsapp-account-1',
      provider_chat_id: 'wa-chat-1',
      provider_message_id: 'provider-wa-1',
      new_text: 'edited text',
    })

    const [deleteUrl, deleteInit] = fetchMock.mock.calls[4]
    expect(deleteUrl).toContain('/api/v1/communications/messages/wa-msg-1')
    expect(deleteInit.method).toBe('DELETE')
    expect(J
```
_Source file truncated after 12000 characters. / Исходный файл обрезан после 12000 символов._

### `frontend/src/domains/communications/api/whatsappBusinessApi.ts`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/domains/communications/api/whatsappBusinessApi.ts`
- Size bytes / Размер в байтах: `13751`
- Included characters / Включено символов: `12000`
- Truncated / Обрезано: `yes`

```typescript
import { ApiClient } from '../../../platform/api/ApiClient'
import type {
  WhatsappWebMessage,
  WhatsAppLifecycleResponse,
  WhatsappWebMessageListResponse,
  WhatsappWebMessageSearchResponse,
  WhatsappWebMediaSearchResponse,
} from '../../../shared/communications/types/whatsapp'
import type { TelegramChatMemberListResponse } from '../../../shared/communications/types/telegramMembers'
import type {
  CommunicationMessageSummary,
  CommunicationMessagesResponse,
  CommunicationProviderMessageCommandResponse,
  ConversationPinToggleResponse,
  MessagePinToggleResponse,
} from '../types/communications'
import type {
  TelegramReactionListResponse,
  TelegramReactionRequest,
  TelegramReactionResponse,
} from '../../../shared/communications/types/telegram'
import type {
  CommunicationProviderConversation,
  CommunicationProviderConversationDetailResponse,
  CommunicationProviderConversationListResponse,
  CommunicationProviderMessageListResponse,
} from '../types/providerChannels'

export async function fetchWhatsappWebBusinessConversations(
  accountId?: string,
  limit = 50
): Promise<CommunicationProviderConversationListResponse> {
  const params = new URLSearchParams({
    limit: String(Math.trunc(limit)),
    channel_kind: 'whatsapp_web',
  })
  if (accountId?.trim()) {
    params.set('account_id', accountId.trim())
  }
  const response = await ApiClient.instance.get<CommunicationProviderConversationListResponse>(
    `/api/v1/communications/conversations?${params.toString()}`,
    'Communication WhatsApp conversations request failed'
  )
  return {
    items: response.items.filter(isWhatsappConversation),
  }
}

export async function fetchWhatsappWebBusinessConversationDetail(
  conversationId: string
): Promise<CommunicationProviderConversationDetailResponse> {
  return ApiClient.instance.get<CommunicationProviderConversationDetailResponse>(
    `/api/v1/communications/conversations/${encodeURIComponent(conversationId)}`,
    'Communication WhatsApp conversation detail request failed'
  )
}

export async function fetchWhatsappWebBusinessConversationMembers(
  conversationId: string,
  limit = 50,
  query?: string,
  role?: string,
  cursor?: string
): Promise<TelegramChatMemberListResponse> {
  const params = new URLSearchParams({ limit: String(Math.trunc(limit)) })
  if (query?.trim()) params.set('query', query.trim())
  if (role?.trim()) params.set('role', role.trim())
  if (cursor?.trim()) params.set('cursor', cursor.trim())
  return ApiClient.instance.get<TelegramChatMemberListResponse>(
    `/api/v1/communications/conversations/${encodeURIComponent(conversationId)}/members?${params.toString()}`,
    'Communication WhatsApp conversation members request failed'
  )
}

export async function fetchWhatsappWebBusinessMessages(
  accountId?: string,
  providerChatId?: string,
  limit = 50
): Promise<WhatsappWebMessageListResponse> {
  const params = new URLSearchParams({
    limit: String(Math.trunc(limit)),
    channel_kind: 'whatsapp_web',
  })
  if (accountId?.trim()) {
    params.set('account_id', accountId.trim())
  }
  if (providerChatId?.trim()) {
    params.set('conversation_id', providerChatId.trim())
  }
  const response = await ApiClient.instance.get<CommunicationMessagesResponse>(
    `/api/v1/communications/messages?${params.toString()}`,
    'Communication WhatsApp messages request failed'
  )
  return { items: response.items.map(communicationMessageToWhatsappWebMessage) }
}

export async function searchWhatsappWebBusinessMessages(params: {
  q: string
  account_id?: string
  provider_chat_id?: string
  limit?: number
}): Promise<WhatsappWebMessageSearchResponse> {
  const query = new URLSearchParams({
    q: params.q.trim(),
    channel_kind: 'whatsapp_web',
  })
  if (params.account_id?.trim()) {
    query.set('account_id', params.account_id.trim())
  }
  if (params.provider_chat_id?.trim()) {
    query.set('provider_chat_id', params.provider_chat_id.trim())
  }
  if (params.limit != null) {
    query.set('limit', String(Math.trunc(params.limit)))
  }
  return ApiClient.instance.get<WhatsappWebMessageSearchResponse>(
    `/api/v1/communications/search/messages?${query.toString()}`,
    'Communication WhatsApp message search failed'
  )
}

export async function searchWhatsappWebBusinessMedia(params: {
  q?: string
  account_id?: string
  provider_chat_id?: string
  kind?: string
  limit?: number
}): Promise<WhatsappWebMediaSearchResponse> {
  const query = new URLSearchParams({ channel_kind: 'whatsapp_web' })
  if (params.q?.trim()) {
    query.set('q', params.q.trim())
  }
  if (params.account_id?.trim()) {
    query.set('account_id', params.account_id.trim())
  }
  if (params.provider_chat_id?.trim()) {
    query.set('provider_chat_id', params.provider_chat_id.trim())
  }
  if (params.kind?.trim()) {
    query.set('kind', params.kind.trim())
  }
  if (params.limit != null) {
    query.set('limit', String(Math.trunc(params.limit)))
  }
  return ApiClient.instance.get<WhatsappWebMediaSearchResponse>(
    `/api/v1/communications/search/media?${query.toString()}`,
    'Communication WhatsApp media search failed'
  )
}

export async function fetchWhatsappWebBusinessPinnedMessages(params: {
  conversation_id: string
  limit?: number
}): Promise<CommunicationProviderMessageListResponse> {
  const query = new URLSearchParams()
  if (params.limit != null) {
    query.set('limit', String(Math.trunc(params.limit)))
  }
  return ApiClient.instance.get<CommunicationProviderMessageListResponse>(
    `/api/v1/communications/conversations/${encodeURIComponent(params.conversation_id)}/pinned-messages?${query.toString()}`,
    'Communication WhatsApp pinned messages request failed'
  )
}

export async function sendWhatsappBusinessMessage(request: {
  account_id: string
  provider_chat_id: string
  text: string
}): Promise<CommunicationProviderMessageCommandResponse> {
  return ApiClient.instance.post<CommunicationProviderMessageCommandResponse>(
    `/api/v1/communications/conversations/${encodeURIComponent(request.provider_chat_id)}/messages`,
    { account_id: request.account_id, text: request.text },
    'Communication WhatsApp message send failed'
  )
}

export async function replyToWhatsappBusinessMessage(params: {
  message_id: string
  text: string
}): Promise<CommunicationProviderMessageCommandResponse> {
  return ApiClient.instance.post<CommunicationProviderMessageCommandResponse>(
    `/api/v1/communications/messages/${encodeURIComponent(params.message_id)}/reply`,
    { text: params.text },
    'Communication WhatsApp reply failed'
  )
}

export async function forwardWhatsappBusinessMessage(params: {
  message_id: string
  provider_chat_id: string
}): Promise<CommunicationProviderMessageCommandResponse> {
  return ApiClient.instance.post<CommunicationProviderMessageCommandResponse>(
    `/api/v1/communications/messages/${encodeURIComponent(params.message_id)}/forward`,
    { conversation_id: params.provider_chat_id },
    'Communication WhatsApp forward failed'
  )
}

export async function editWhatsappBusinessMessage(params: {
  message_id: string
  command_id?: string
  account_id: string
  provider_chat_id: string
  provider_message_id: string
  new_text: string
}): Promise<WhatsAppLifecycleResponse> {
  return ApiClient.instance.patch<WhatsAppLifecycleResponse>(
    `/api/v1/communications/messages/${encodeURIComponent(params.message_id)}`,
    params,
    'Communication WhatsApp message edit failed'
  )
}

export async function deleteWhatsappBusinessMessage(params: {
  message_id: string
  command_id?: string
  account_id: string
  provider_chat_id: string
  provider_message_id: string
  reason_class?: string
  actor_class?: string
  is_provider_delete?: boolean
}): Promise<WhatsAppLifecycleResponse> {
  return ApiClient.instance.deleteWithBody<WhatsAppLifecycleResponse>(
    `/api/v1/communications/messages/${encodeURIComponent(params.message_id)}`,
    params,
    'Communication WhatsApp message delete failed'
  )
}

export async function pinWhatsappBusinessMessage(params: {
  message_id: string
}): Promise<MessagePinToggleResponse> {
  return ApiClient.instance.post<MessagePinToggleResponse>(
    `/api/v1/communications/messages/${encodeURIComponent(params.message_id)}/pin`,
    {},
    'Communication WhatsApp message pin failed'
  )
}

export async function pinWhatsappBusinessConversation(params: {
  conversation_id: string
}): Promise<ConversationPinToggleResponse> {
  return ApiClient.instance.post<ConversationPinToggleResponse>(
    `/api/v1/communications/conversations/${encodeURIComponent(params.conversation_id)}/pin`,
    {},
    'Communication WhatsApp conversation pin failed'
  )
}

export async function unpinWhatsappBusinessConversation(params: {
  conversation_id: string
}): Promise<ConversationPinToggleResponse> {
  return ApiClient.instance.post<ConversationPinToggleResponse>(
    `/api/v1/communications/conversations/${encodeURIComponent(params.conversation_id)}/unpin`,
    {},
    'Communication WhatsApp conversation unpin failed'
  )
}

export async function archiveWhatsappBusinessConversation(params: {
  conversation_id: string
}): Promise<ConversationPinToggleResponse> {
  return ApiClient.instance.post<ConversationPinToggleResponse>(
    `/api/v1/communications/conversations/${encodeURIComponent(params.conversation_id)}/archive`,
    {},
    'Communication WhatsApp conversation archive failed'
  )
}

export async function unarchiveWhatsappBusinessConversation(params: {
  conversation_id: string
}): Promise<ConversationPinToggleResponse> {
  return ApiClient.instance.post<ConversationPinToggleResponse>(
    `/api/v1/communications/conversations/${encodeURIComponent(params.conversation_id)}/unarchive`,
    {},
    'Communication WhatsApp conversation unarchive failed'
  )
}

export async function muteWhatsappBusinessConversation(params: {
  conversation_id: string
}): Promise<ConversationPinToggleResponse> {
  return ApiClient.instance.post<ConversationPinToggleResponse>(
    `/api/v1/communications/conversations/${encodeURIComponent(params.conversation_id)}/mute`,
    {},
    'Communication WhatsApp conversation mute failed'
  )
}

export async function unmuteWhatsappBusinessConversation(params: {
  conversation_id: string
}): Promise<ConversationPinToggleResponse> {
  return ApiClient.instance.post<ConversationPinToggleResponse>(
    `/api/v1/communications/conversations/${encodeURIComponent(params.conversation_id)}/unmute`,
    {},
    'Communication WhatsApp conversation unmute failed'
  )
}

export async function markWhatsappBusinessConversationRead(params: {
  conversation_id: string
}): Promise<ConversationPinToggleResponse> {
  return ApiClient.instance.post<ConversationPinToggleResponse>(
    `/api/v1/communications/conversations/${encodeURIComponent(params.conversation_id)}/read`,
    {},
    'Communication WhatsApp conversation mark-read failed'
  )
}

export async function markWhatsappBusinessConversationUnread(params: {
  conversation_id: string
}): Promise<ConversationPinToggleResponse> {
  return ApiClient.instance.post<ConversationPinToggleResponse>(
    `/api/v1/communications/conversations/${encodeURIComponent(params.conversation_id)}/unread`,
    {},
    'Communication WhatsApp conversation mark-unread failed'
  )
}

export async function fetchWhatsappBusinessReactions(
  messageId: string
): Promise<TelegramReactionListResponse> {
  return ApiClient.instance.get<TelegramReactionListResponse>(
    `/api/v1/communications/messages/${encodeURIComponent(messageId)}/reactions`,
    'Communication WhatsApp reactions failed'
  )
}

export async function addWhatsappBusinessReaction(
  messageId: string,
  request: TelegramReactionRequest
): Promise<TelegramReactionResponse> {
  return ApiClient.instance.post<TelegramReactionResponse>(
    `/api/v1/communications/messages/${encodeURIComponent(messageId)}/reactions`,
    request,
    'Communication WhatsApp reaction add failed'
  )
}

export async function removeWhatsappB
```
_Source file truncated after 12000 characters. / Исходный файл обрезан после 12000 символов._

### `frontend/src/domains/communications/components/AttachmentSearchPanel.boundary.test.ts`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/domains/communications/components/AttachmentSearchPanel.boundary.test.ts`
- Size bytes / Размер в байтах: `1294`
- Included characters / Включено символов: `1294`
- Truncated / Обрезано: `no`

```typescript
import { describe, expect, it } from 'vitest'
import { readFileSync } from 'node:fs'

describe('AttachmentSearchPanel boundary', () => {
  it('uses Vee/Zod forms, TanStack Query and TanStack Table without direct fetch', () => {
    const source = readFileSync(
      new URL('./AttachmentSearchPanel.vue', import.meta.url),
      'utf8'
    )

    expect(source).toContain("from 'vee-validate'")
    expect(source).toContain('../forms/attachmentSearchForm')
    expect(source).toContain('setFieldValue')
    expect(source).toContain('useAttachmentSearchQuery')
    expect(source).toContain('useVueTable')
    expect(source).toContain('getCoreRowModel')
    expect(source).toContain('useVirtualizer')
    expect(source).toContain('fetchNextPage')
    expect(source).toContain('hasNextPage')
    expect(source).toContain('useAttachmentSearchResultPrefetch')
    expect(source).toContain('@mouseenter="handleResultPrefetch(tableRows[virtualRow.index].original)"')
    expect(source).toContain('@focus="handleResultPrefetch(tableRows[virtualRow.index].original)"')
    expect(source).toContain('accountId: string | null')
    expect(source).toContain('@submit.prevent="submitSearch"')
    expect(source).not.toContain('../api/communications')
    expect(source).not.toMatch(/\bfetch\s*\(/)
  })
})
```

### `frontend/src/domains/communications/components/BilingualReplyPanel.boundary.test.ts`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/domains/communications/components/BilingualReplyPanel.boundary.test.ts`
- Size bytes / Размер в байтах: `964`
- Included characters / Включено символов: `964`
- Truncated / Обрезано: `no`

```typescript
import { describe, expect, it } from 'vitest'
import { readFileSync } from 'node:fs'

describe('BilingualReplyPanel boundary', () => {
  it('uses Vee/Zod forms and a TanStack mutation without direct API calls', () => {
    const source = readFileSync(
      new URL('./BilingualReplyPanel.vue', import.meta.url),
      'utf8'
    )

    expect(source).toContain("from 'vee-validate'")
    expect(source).toContain('../forms/bilingualReplyFlowForm')
    expect(source).toContain('usePrepareBilingualReplyFlowMutation')
    expect(source).toContain('setFieldValue')
    expect(source).toContain('Original')
    expect(source).toContain('Translation')
    expect(source).toContain('Reply in Russian')
    expect(source).toContain('Back Translation')
    expect(source).toContain('sendBilingualReply')
    expect(source).toContain('@submit.prevent="submitBilingualReply"')
    expect(source).not.toContain('../api/')
    expect(source).not.toContain('fetch(')
  })
})
```

### `frontend/src/domains/communications/components/BulkActionsBar.boundary.test.ts`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/domains/communications/components/BulkActionsBar.boundary.test.ts`
- Size bytes / Размер в байтах: `1154`
- Included characters / Включено символов: `1154`
- Truncated / Обрезано: `no`

```typescript
import { describe, expect, it } from 'vitest'
import { readFileSync } from 'node:fs'

describe('BulkActionsBar mail workflow boundary', () => {
  it('exposes local metadata bulk actions supported by the mail API', () => {
    const source = readFileSync(new URL('./BulkActionsBar.vue', import.meta.url), 'utf8')

    expect(source).toContain("'pin'")
    expect(source).toContain("'unpin'")
    expect(source).toContain("'important'")
    expect(source).toContain("'not_important'")
    expect(source).toContain("'add_label'")
    expect(source).toContain("'remove_label'")
    expect(source).toContain("'snooze'")
  })

  it('emits payload-backed commands for label and snooze operations', () => {
    const source = readFileSync(new URL('./BulkActionsBar.vue', import.meta.url), 'utf8')

    expect(source).toContain('type BulkActionCommand')
    expect(source).toContain('BulkMessageActionRequest')
    expect(source).toContain("command: { action: 'add_label', label: 'Follow up' }")
    expect(source).toContain("command: { action: 'remove_label', label: 'Follow up' }")
    expect(source).toContain('snooze_until: nextBusinessMorningIso()')
  })
})
```

### `frontend/src/domains/communications/components/CommunicationFolderStrip.boundary.test.ts`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/domains/communications/components/CommunicationFolderStrip.boundary.test.ts`
- Size bytes / Размер в байтах: `3995`
- Included characters / Включено символов: `3995`
- Truncated / Обрезано: `no`

```typescript
import { describe, expect, it } from 'vitest'
import { readFileSync } from 'node:fs'

describe('CommunicationFolderStrip folder management boundary', () => {
	it('uses Zod/Vee folder forms and TanStack folder mutations', () => {
		const source = readFileSync(
			new URL('./CommunicationFolderStrip.vue', import.meta.url),
			'utf8'
		)

		expect(source).toContain("from 'vee-validate'")
		expect(source).toContain("from '@tanstack/vue-virtual'")
		expect(source).toContain('../forms/mailFolderForm')
		expect(source).toContain('useCommunicationFoldersQuery')
		expect(source).toContain('useVirtualizer')
		expect(source).toContain('horizontal: true')
		expect(source).toContain('virtualFolders')
		expect(source).toContain('folderVirtualTotalSize')
		expect(source).toContain('v-for="virtualFolder in virtualFolders"')
		expect(source).toContain('folderRows[virtualFolder.index]')
		expect(source).toContain('hasNextPage')
		expect(source).toContain('isFetchingNextPage')
		expect(source).toContain('fetchNextPage')
		expect(source).toContain('handleFolderVirtualScroll')
		expect(source).toContain('@scroll="handleFolderVirtualScroll"')
		expect(source).toContain('useCopyMessageToFolderMutation')
		expect(source).toContain('useCreateCommunicationFolderMutation')
		expect(source).toContain('useUpdateCommunicationFolderMutation')
		expect(source).toContain('useDeleteCommunicationFolderMutation')
		expect(source).toContain('useMoveMessageToFolderMutation')
		expect(source).toContain('useCommunicationFolderReorder')
		expect(source).toContain('orderCommunicationFolderDisplayRows')
		expect(source).toContain('createChildFolderDraft')
		expect(source).toContain('mailFolderHierarchyDeleteImpact')
		expect(source).toContain('mailFolderParentPathOptions')
		expect(source).toContain('splitCommunicationFolderName')
		expect(source).toContain('validateCommunicationFolderParentPath')
		expect(source).toContain('const orderedFolders = computed(() => folderRows.value.map((row) => row.folder))')
		expect(source).toContain('const parentPathOptions = computed(() => mailFolderParentPathOptions(orderedFolders.value, editingFolder.value))')
		expect(source).toContain('const folderPathPreview = computed(() => composeCommunicationFolderName(parentPath.value, leafName.value))')
		expect(source).toContain('function openCreateChildDialog(folder: CommunicationFolder)')
		expect(source).toContain(':description="folderDialogDescription"')
		expect(source).toContain(':description="deleteDialogDescription"')
		expect(source).toContain('list="mail-folder-parent-path-options"')
		expect(source).toContain('<span>Folder name</span>')
		expect(source).toContain('class="mail-folder-path-preview"')
		expect(source).toContain('Add child folder under')
		expect(source).toContain('class="mail-folder-delete-impact"')
		expect(source).toContain('activeId: string')
		expect(source).toContain('select: [folderId: string]')
		expect(source).toContain("emit('select', folderRows[virtualFolder.index].folder.folder_id)")
		expect(source).toContain('@dragover="handleFolderDragOver"')
		expect(source).toContain('@drop.prevent="handleFolderDrop($event, folderRows[virtualFolder.index].folder)"')
		expect(source).toContain('@dragstart="folderReorder.handleDragStart($event, folderRows[virtualFolder.index].folder)"')
		expect(source).toContain('@dragend="folderReorder.handleDragEnd"')
		expect(source).toContain('folderReorder.canHandleDragOver(event)')
		expect(source).toContain('folderIndent(folderRows[virtualFolder.index])')
		expect(source).toContain('@drop.prevent="handleFolderDrop($event, folderRows[virtualFolder.index].folder)"')
		expect(source).toContain('@dragstart="folderReorder.handleDragStart($event, folderRows[virtualFolder.index].folder)"')
		expect(source).toContain('parseCommunicationMessageDragPayload')
		expect(source).toContain('folderReorder.handleDrop(event, folder)')
		expect(source).not.toContain('../api/communications')
		expect(source).not.toContain('ApiClient')
	})
})
```

### `frontend/src/domains/communications/components/CommunicationList.boundary.test.ts`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/domains/communications/components/CommunicationList.boundary.test.ts`
- Size bytes / Размер в байтах: `1331`
- Included characters / Включено символов: `1331`
- Truncated / Обрезано: `no`

```typescript
import { describe, expect, it } from 'vitest'
import { readFileSync } from 'node:fs'

describe('CommunicationList keyboard multi-select boundary', () => {
  it('supports keyboard selection without direct API access', () => {
    const source = readFileSync(new URL('./CommunicationList.vue', import.meta.url), 'utf8')

    expect(source).toContain('handleKeydown')
    expect(source).toContain('@keydown="handleKeydown"')
    expect(source).toContain('tabindex="0"')
    expect(source).toContain('role="listbox"')
    expect(source).toContain('aria-multiselectable="true"')
    expect(source).toContain("event.code === 'Space'")
    expect(source).toContain("event.key.toLowerCase() === 'a'")
    expect(source).toContain('event.metaKey || event.ctrlKey')
    expect(source).toContain("event.key === 'Escape'")
    expect(source).toContain("event.key === 'ArrowDown'")
    expect(source).toContain("event.key === 'ArrowUp'")
    expect(source).toContain("emit('toggleSelection', current.message_id, event.shiftKey)")
    expect(source).toContain("emit('selectVisible', visibleMessageIds.value)")
    expect(source).toContain("emit('clearSelection')")
    expect(source).toContain("emit('toggleSelection', next.message_id, true)")
    expect(source).not.toMatch(/\bfetch\s*\(/)
    expect(source).not.toContain('ApiClient')
  })
})
```

### `frontend/src/domains/communications/components/CommunicationListItem.boundary.test.ts`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/domains/communications/components/CommunicationListItem.boundary.test.ts`
- Size bytes / Размер в байтах: `644`
- Included characters / Включено символов: `644`
- Truncated / Обрезано: `no`

```typescript
import { describe, expect, it } from 'vitest'
import { readFileSync } from 'node:fs'

describe('CommunicationListItem drag payload boundary', () => {
  it('serializes the full selected message set when dragging a selected row', () => {
    const source = readFileSync(new URL('./CommunicationListItem.vue', import.meta.url), 'utf8')

    expect(source).toContain('selectedMessageIds: string[]')
    expect(source).toContain('createCommunicationMessageDragPayload(props.message.message_id, props.selectedMessageIds)')
    expect(source).toContain('role="option"')
    expect(source).toContain(':aria-selected="isChecked || isSelected"')
  })
})
```

### `frontend/src/domains/communications/components/CommunicationViewer.boundary.test.ts`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/domains/communications/components/CommunicationViewer.boundary.test.ts`
- Size bytes / Размер в байтах: `2148`
- Included characters / Включено символов: `2148`
- Truncated / Обрезано: `no`

```typescript
import { describe, expect, it } from 'vitest'
import { readFileSync } from 'node:fs'

describe('CommunicationViewer boundary', () => {
  it('passes bilingual reply send events from MessageBodyTab', () => {
    const source = readFileSync(new URL('./CommunicationViewer.vue', import.meta.url), 'utf8')

    expect(source).toContain('send-bilingual-reply')
    expect(source).toContain('MessageBodyTab')
    expect(source).toContain('@send-bilingual-reply')
    expect(source).toContain('useMessageAiStateQuery')
    expect(source).toContain('useUpdateMessageAiStateMutation')
    expect(source).toContain('currentAiState')
    expect(source).toContain('transitionAiState')
    expect(source).toContain('exportMessage')
    expect(source).toContain('@export-message')
    expect(source).toContain('markMessageRead')
    expect(source).toContain('markMessageUnread')
    expect(source).toContain('deleteFromProvider')
    expect(source).toContain('@mark-message-read')
    expect(source).toContain('@mark-message-unread')
    expect(source).toContain('@delete-from-provider')
    expect(source).toContain('addLabel')
    expect(source).toContain('removeLabel')
    expect(source).toContain('snoozeMessage')
    expect(source).toContain('replyAll')
    expect(source).toContain('forwardMessage')
    expect(source).toContain('redirectMessage')
    expect(source).toContain('@add-label')
    expect(source).toContain('@remove-label')
    expect(source).toContain('@snooze-message')
    expect(source).toContain('@reply-all')
    expect(source).toContain('@forward-message')
    expect(source).toContain('@redirect-message')
    expect(source).toContain('ai-state-panel')
    expect(source).toContain('AI state')
    expect(source).toContain("aiState === 'REVIEW_REQUIRED'")
    expect(source).toContain("review_reason: 'Manual review requested from Mail UI'")
    expect(source).toContain("aiState === 'FAILED'")
    expect(source).toContain("last_error: 'Manual failure recorded from Mail UI'")
    expect(source).toContain("transitionAiState('FAILED')")
    expect(source).not.toContain('fetch(')
    expect(source).not.toContain('ApiClient')
  })
})
```

### `frontend/src/domains/communications/components/CommunicationsActionBar.boundary.test.ts`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/domains/communications/components/CommunicationsActionBar.boundary.test.ts`
- Size bytes / Размер в байтах: `1601`
- Included characters / Включено символов: `1601`
- Truncated / Обрезано: `no`

```typescript
import { describe, expect, it } from 'vitest'
import { readFileSync } from 'node:fs'

describe('CommunicationsActionBar export boundary', () => {
  it('surfaces the latest message export as a browser download link', () => {
    const source = readFileSync(new URL('./CommunicationsActionBar.vue', import.meta.url), 'utf8')

    expect(source).toContain('lastMessageExport')
    expect(source).toContain('messageExportDownloadHref')
    expect(source).toContain('download')
    expect(source).toContain('Export ready')
    expect(source).toContain('MailResourceOverviewStrip')
    expect(source).toContain('MailSyncSettingsStrip')
    expect(source).toContain('../../../shared/mailSync/MailSyncSettingsStrip.vue')
    expect(source).toContain('MailCertificateStrip')
    expect(source).toContain('hasMoreDrafts')
    expect(source).toContain('isLoadingMoreDrafts')
    expect(source).toContain('loadMoreDrafts')
    expect(source).toContain('syncSettings')
    expect(source).toContain('updateSyncSettings')
    expect(source).toContain('subscriptions')
    expect(source).toContain('topSenders')
    expect(source).toContain('blockers')
    expect(source).toContain('hasMoreSubscriptions')
    expect(source).toContain('isLoadingMoreSubscriptions')
    expect(source).toContain('hasMoreTopSenders')
    expect(source).toContain('isLoadingMoreTopSenders')
    expect(source).toContain('loadMoreSubscriptions')
    expect(source).toContain('loadMoreTopSenders')
    expect(source).not.toContain('../api/')
    expect(source).not.toContain('fetch(')
    expect(source).not.toContain('ApiClient')
  })
})
```

### `frontend/src/domains/communications/components/CommunicationsCallsPanel.boundary.test.ts`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/domains/communications/components/CommunicationsCallsPanel.boundary.test.ts`
- Size bytes / Размер в байтах: `1023`
- Included characters / Включено символов: `1023`
- Truncated / Обрезано: `no`

```typescript
import { describe, expect, it } from 'vitest'
import { readFileSync } from 'node:fs'

describe('CommunicationsCallsPanel boundary', () => {
  it('uses provider-neutral calls queries and supports calls plus meetings modes', () => {
    const source = readFileSync(new URL('./CommunicationsCallsPanel.vue', import.meta.url), 'utf8')

    expect(source).toContain('useProviderCallsQuery')
    expect(source).toContain('useProviderCallTranscriptQuery')
    expect(source).toContain("mode: 'calls' | 'meetings'")
    expect(source).toContain("mode === 'meetings'")
    expect(source).toContain("props.mode === 'meetings' ? 'zoom' : undefined")
    expect(source).toContain("meetingProvider(call) === 'zoom'")
    expect(source).toContain('meetingParticipants(selectedCall).length')
    expect(source).toContain('meetingRecordingRefs(selectedCall).length')
    expect(source).toContain("{{ t('Recording references') }}")
    expect(source).toContain("{{ t('Open join URL') }}")
    expect(source).not.toContain('fetch(')
  })
})
```

### `frontend/src/domains/communications/components/CommunicationsConversationList.boundary.test.ts`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/domains/communications/components/CommunicationsConversationList.boundary.test.ts`
- Size bytes / Размер в байтах: `1150`
- Included characters / Включено символов: `1150`
- Truncated / Обрезано: `no`

```typescript
import { describe, expect, it } from 'vitest'
import { readFileSync } from 'node:fs'

describe('CommunicationsConversationList boundary', () => {
	it('renders server-backed thread rows and exposes thread pagination controls', () => {
		const source = readFileSync(
			new URL('./CommunicationsConversationList.vue', import.meta.url),
			'utf8'
		)

		expect(source).toContain('threads: CommunicationThreadSummary[]')
		expect(source).toContain('selectedThreadId: string')
		expect(source).toContain('accountId: string')
		expect(source).toContain('v-for="thread in threads"')
		expect(source).toContain('thread.message_count')
		expect(source).toContain('useThreadMessagesPrefetch')
		expect(source).toContain('handleThreadPrefetch')
		expect(source).toContain('@mouseenter="handleThreadPrefetch(thread)"')
		expect(source).toContain('@focus="handleThreadPrefetch(thread)"')
		expect(source).toContain("selectThread: [thread: CommunicationThreadSummary]")
		expect(source).toContain("loadMoreThreads: []")
		expect(source).toContain('hasThreadNextPage')
		expect(source).not.toMatch(/\bfetch\s*\(/)
		expect(source).not.toContain('ApiClient')
	})
})
```

### `frontend/src/domains/communications/components/CommunicationsDetailPane.boundary.test.ts`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/domains/communications/components/CommunicationsDetailPane.boundary.test.ts`
- Size bytes / Размер в байтах: `2147`
- Included characters / Включено символов: `2147`
- Truncated / Обрезано: `no`

```typescript
import { describe, expect, it } from 'vitest'
import { readFileSync } from 'node:fs'

describe('CommunicationsDetailPane boundary', () => {
  it('forwards bilingual reply send events from the mail viewer', () => {
    const source = readFileSync(
      new URL('./CommunicationsDetailPane.vue', import.meta.url),
      'utf8'
    )

    expect(source).toContain('sendBilingualReply')
    expect(source).toContain('exportMessage')
    expect(source).toContain('addLabel')
    expect(source).toContain('removeLabel')
    expect(source).toContain('snoozeMessage')
    expect(source).toContain('replyAll')
    expect(source).toContain('forwardMessage')
    expect(source).toContain('redirectMessage')
    expect(source).toContain('markMessageRead')
    expect(source).toContain('markMessageUnread')
    expect(source).toContain('deleteFromProvider')
    expect(source).toContain('CommunicationViewer')
    expect(source).toContain('@send-bilingual-reply')
    expect(source).toContain('@export-message')
    expect(source).toContain('@add-label')
    expect(source).toContain('@remove-label')
    expect(source).toContain('@snooze-message')
    expect(source).toContain('@mark-message-read')
    expect(source).toContain('@mark-message-unread')
    expect(source).toContain('@delete-from-provider')
    expect(source).toContain('@reply-all')
    expect(source).toContain('@forward-message')
    expect(source).toContain('@redirect-message')
  })

  it('renders selected threads through the conversation timeline instead of the single-message viewer', () => {
    const source = readFileSync(
      new URL('./CommunicationsDetailPane.vue', import.meta.url),
      'utf8'
    )

    expect(source).toContain('ThreadConversationView')
    expect(source).toContain('selectedThread')
    expect(source).toContain('threadMessages')
    expect(source).toContain('isThreadReplySending')
    expect(source).toContain(':is-sending-reply="isThreadReplySending"')
    expect(source).toContain('@open-message')
    expect(source).toContain('@reply-to-message')
    expect(source).toContain('@save-reply-draft')
    expect(source).toContain('@send-reply')
  })
})
```

### `frontend/src/domains/communications/components/CommunicationsListPane.boundary.test.ts`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/domains/communications/components/CommunicationsListPane.boundary.test.ts`
- Size bytes / Размер в байтах: `1508`
- Included characters / Включено символов: `1508`
- Truncated / Обрезано: `no`

```typescript
import { describe, expect, it } from 'vitest'
import { readFileSync } from 'node:fs'

describe('CommunicationsListPane folder browsing boundary', () => {
  it('can force the virtualized CommunicationList for custom folder message results', () => {
    const source = readFileSync(
      new URL('./CommunicationsListPane.vue', import.meta.url),
      'utf8'
    )

    expect(source).toContain('isFolderMode: boolean')
    expect(source).toContain('accountId: string')
    expect(source).toContain('threads: CommunicationThreadSummary[]')
    expect(source).toContain('selectedThreadId: string')
    expect(source).toContain('hasThreadNextPage: boolean')
    expect(source).toContain('selectThread: [thread: CommunicationThreadSummary]')
    expect(source).toContain('@load-more-threads="emit(')
    expect(source).toContain('v-else-if="!isFolderMode && (navigatorMode ===')
    expect(source).toContain('<CommunicationList')
    expect(source).toContain(':account-id="accountId"')
    expect(source).toContain('@load-more="emit(')
  })

  it('forwards mail list keyboard multi-select commands', () => {
    const source = readFileSync(
      new URL('./CommunicationsListPane.vue', import.meta.url),
      'utf8'
    )

    expect(source).toContain('selectVisible: [messageIds: string[]]')
    expect(source).toContain('clearSelection: []')
    expect(source).toContain('@select-visible="emit(\'selectVisible\', $event)"')
    expect(source).toContain('@clear-selection="emit(\'clearSelection\')"')
  })
})
```

### `frontend/src/domains/communications/components/ComposeDrawer.boundary.test.ts`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/domains/communications/components/ComposeDrawer.boundary.test.ts`
- Size bytes / Размер в байтах: `2950`
- Included characters / Включено символов: `2950`
- Truncated / Обрезано: `no`

```typescript
import { readFileSync } from 'node:fs'
import { dirname, resolve } from 'node:path'
import { fileURLToPath } from 'node:url'
import { describe, expect, it } from 'vitest'

const componentPath = resolve(dirname(fileURLToPath(import.meta.url)), 'ComposeDrawer.vue')

describe('ComposeDrawer boundaries', () => {
	it('uses communications query mutations instead of calling the API client directly', () => {
		const source = readFileSync(componentPath, 'utf8')

		expect(source).not.toContain("../api/communications")
		expect(source).toContain('useSendMailMutation')
		expect(source).toContain('useSaveDraftMutation')
		expect(source).toContain('useDeleteDraftMutation')
		expect(source).toContain('useComposeDraftAutosave')
	})

	it('uses the shared Reka-backed Sheet primitive instead of a custom overlay', () => {
		const source = readFileSync(componentPath, 'utf8')

		expect(source).toContain("import Sheet from '../../../shared/ui/Sheet.vue'")
		expect(source).toContain('<Sheet')
		expect(source).toContain('content-class="compose-drawer"')
		expect(source).toContain('@update:open="handleSheetOpenChange"')
		expect(source).not.toContain('compose-drawer-overlay')
		expect(source).not.toContain('@click.self="handleClose"')
	})

	it('uses a dedicated rich editor for HTML compose while preserving source mode', () => {
		const source = readFileSync(componentPath, 'utf8')

		expect(source).toContain('RichComposeEditor')
		expect(source).toContain("setBodyFormat('html', 'rich')")
		expect(source).toContain("setBodyFormat('html', 'source')")
		expect(source).toContain('html-body-editor')
	})

	it('delegates template selection and rendering to the template picker component', () => {
		const source = readFileSync(componentPath, 'utf8')

		expect(source).toContain('ComposeTemplatePicker')
		expect(source).toContain('@apply="applyRenderedTemplate"')
		expect(source).toContain('@saved=')
		expect(source).toContain('@deleted=')
	})

	it('delegates persona signature selection to the signature picker component', () => {
		const source = readFileSync(componentPath, 'utf8')

		expect(source).toContain('ComposeSignaturePicker')
		expect(source).toContain('@apply="applySignature"')
	})

	it('stages dropped compose attachments without pretending provider send supports them', () => {
		const source = readFileSync(componentPath, 'utf8')

		expect(source).toContain('stagedAttachments')
		expect(source).toContain('handleAttachmentFiles')
		expect(source).toContain('attachmentInput')
		expect(source).toContain('@attachments-dropped="handleAttachmentFiles"')
		expect(source).toContain('Attachment upload is not connected to provider send yet')
		expect(source).toContain('removeStagedAttachment')
	})

	it('keeps compose styling in the component-owned stylesheet', () => {
		const source = readFileSync(componentPath, 'utf8')

		expect(source).toContain("import './ComposeDrawer.css'")
		expect(source).not.toContain('<style scoped>')
	})
})
```

### `frontend/src/domains/communications/components/ComposeSignaturePicker.boundary.test.ts`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/domains/communications/components/ComposeSignaturePicker.boundary.test.ts`
- Size bytes / Размер в байтах: `657`
- Included characters / Включено символов: `657`
- Truncated / Обрезано: `no`

```typescript
import { readFileSync } from 'node:fs'
import { dirname, resolve } from 'node:path'
import { fileURLToPath } from 'node:url'
import { describe, expect, it } from 'vitest'

const componentPath = resolve(dirname(fileURLToPath(import.meta.url)), 'ComposeSignaturePicker.vue')

describe('ComposeSignaturePicker boundaries', () => {
	it('uses the personas query hook and emits selected signatures', () => {
		const source = readFileSync(componentPath, 'utf8')

		expect(source).not.toContain('../api/communications')
		expect(source).toContain('usePersonasQuery')
		expect(source).toContain("emit('apply'")
		expect(source).toContain('persona.signature')
	})
})
```

### `frontend/src/domains/communications/components/ComposeTemplatePicker.boundary.test.ts`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/domains/communications/components/ComposeTemplatePicker.boundary.test.ts`
- Size bytes / Размер в байтах: `3208`
- Included characters / Включено символов: `3208`
- Truncated / Обрезано: `no`

```typescript
import { readFileSync } from 'node:fs'
import { dirname, resolve } from 'node:path'
import { fileURLToPath } from 'node:url'
import { describe, expect, it } from 'vitest'

const componentPath = resolve(dirname(fileURLToPath(import.meta.url)), 'ComposeTemplatePicker.vue')

describe('ComposeTemplatePicker boundaries', () => {
	it('uses TanStack Query hooks for template list and render operations', () => {
		const source = readFileSync(componentPath, 'utf8')

		expect(source).not.toContain('../api/communications')
		expect(source).toContain('useRichTemplatesQuery')
		expect(source).toContain('useRenderRichTemplateMutation')
		expect(source).toContain('useCreateRichTemplateMutation')
		expect(source).toContain("emit('apply'")
		expect(source).toContain("emit('saved'")
		expect(source).toContain("emit('deleted'")
		expect(source).toContain('updateSelectedTemplate')
		expect(source).toContain('templateId: template.template_id')
		expect(source).toContain('useDeleteRichTemplateMutation')
		expect(source).toContain('usePreviewRichTemplateMailMergeMutation')
		expect(source).toContain('deleteSelectedTemplate')
		expect(source).toContain('missingTemplateVariables')
		expect(source).toContain('parseTemplateMailMergePreviewRows')
		expect(source).toContain('stringifyTemplateMailMergePreviewRows')
		expect(source).toContain('resolveTemplateVariableValues')
		expect(source).toContain('storedTemplateDiagnosticMessages')
		expect(source).toContain('orderedTemplates')
		expect(source).toContain('templateDiagnosticCount')
		expect(source).toContain('templateUpdatedLabel')
		expect(source).toContain('TemplateRecipientMappingPanel')
		expect(source).toContain('TemplateSaveForm')
		expect(source).toContain('templateLibraryCategoryOptions')
		expect(source).toContain('selectedCategory')
		expect(source).toContain('deriveTemplateLibraryCategories')
		expect(source).toContain('applyTemplateRecipientMapping')
		expect(source).toContain('buildTemplateRecipientPreviewRows')
		expect(source).toContain('recipientPreviewSummary')
		expect(source).toContain('suggestTemplateSaveName')
		expect(source).toContain("import './ComposeTemplatePicker.css'")
		expect(source).toContain('const isSameTemplate = Boolean(template && previousTemplate && template.template_id === previousTemplate.template_id)')
		expect(source).toContain('preserveExisting: isSameTemplate')
		expect(source).toContain('Mail merge preview')
		expect(source).toContain('@build-preview="buildPreviewRowsFromRecipients"')
		expect(source).toContain('previewMailMergeMutation')
		expect(source).toContain('previewRowsText')
		expect(source).toContain('previewResult')
		expect(source).toContain('mergeValidationMessage')
		expect(source).toContain('selectedTemplateValidationMessage')
		expect(source).toContain('template-diagnostics')
		expect(source).toContain('templateContentDiagnostics')
		expect(source).toContain('saveValidationMessage')
		expect(source).toContain("openSaveTemplate('duplicate')")
		expect(source).toContain('Save copy')
		expect(source).toContain('closeSaveTemplate')
		expect(source).toContain('result.rendered.unresolved_variables')
		expect(source).toContain('result.rendered.malformed_placeholders')
	})
})
```

### `frontend/src/domains/communications/components/DraftStrip.boundary.test.ts`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/domains/communications/components/DraftStrip.boundary.test.ts`
- Size bytes / Размер в байтах: `866`
- Included characters / Включено символов: `866`
- Truncated / Обрезано: `no`

```typescript
import { describe, expect, it } from 'vitest'
import { readFileSync } from 'node:fs'

describe('DraftStrip virtualization boundary', () => {
  it('renders drafts through TanStack Virtual without direct API access', () => {
    const source = readFileSync(new URL('./DraftStrip.vue', import.meta.url), 'utf8')

    expect(source).toContain("from '@tanstack/vue-virtual'")
    expect(source).toContain('useVirtualizer')
    expect(source).toContain('draftVirtualizer')
    expect(source).toContain('virtualDraftRows')
    expect(source).toContain('draftVirtualTotalSize')
    expect(source).toContain('v-for="virtualRow in virtualDraftRows"')
    expect(source).toContain('hasMore')
    expect(source).toContain('loadMore')
    expect(source).toContain("emit('loadMore')")
    expect(source).not.toContain('../api/')
    expect(source).not.toContain('fetch(')
  })
})
```

### `frontend/src/domains/communications/components/MailCertificateStrip.boundary.test.ts`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/domains/communications/components/MailCertificateStrip.boundary.test.ts`
- Size bytes / Размер в байтах: `876`
- Included characters / Включено символов: `876`
- Truncated / Обрезано: `no`

```typescript
import { describe, expect, it } from 'vitest'
import { readFileSync } from 'node:fs'

describe('MailCertificateStrip boundary', () => {
  it('renders certificate inventory and metadata-only creation without direct API access', () => {
    const source = readFileSync(new URL('./MailCertificateStrip.vue', import.meta.url), 'utf8')

    expect(source).toContain('useMailCertificatesQuery')
    expect(source).toContain('useExpiringMailCertificatesQuery')
    expect(source).toContain('useCreateMailCertificateMutation')
    expect(source).toContain('certificateVeeValidationSchema')
    expect(source).toContain('Expiring certificates')
    expect(source).toContain('Add certificate')
    expect(source).toContain('Storage reference')
    expect(source).not.toContain('../api/')
    expect(source).not.toContain('fetch(')
    expect(source).not.toContain('ApiClient')
  })

})
```
