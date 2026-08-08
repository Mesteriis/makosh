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

- Chunk ID / ID чанка: `148-source-frontend-part-008`
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

### `frontend/src/domains/communications/queries/telegramBusinessQueries.ts`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/domains/communications/queries/telegramBusinessQueries.ts`
- Size bytes / Размер в байтах: `18776`
- Included characters / Включено символов: `12000`
- Truncated / Обрезано: `yes`

```typescript
import { useInfiniteQuery, useMutation, useQuery, useQueryClient } from '@tanstack/vue-query'
import { computed, toValue, type MaybeRefOrGetter } from 'vue'
import {
  addTelegramBusinessReaction,
  deleteTelegramBusinessMessage,
  editTelegramBusinessMessage,
  fetchTelegramBusinessChatDetail,
  fetchTelegramBusinessChatMembers,
  fetchTelegramBusinessChats,
  fetchTelegramBusinessForwardChain,
  fetchTelegramBusinessMessageTombstones,
  fetchTelegramBusinessMessageVersions,
  fetchTelegramBusinessMessages,
  fetchTelegramBusinessPinnedMessages,
  fetchTelegramBusinessRawEvidence,
  fetchTelegramBusinessReactions,
  fetchTelegramBusinessReplyChain,
  fetchTelegramBusinessTopicMessages,
  fetchTelegramBusinessTopics,
  forwardTelegramBusinessMessage,
  markTelegramBusinessMessageRead,
  pinTelegramBusinessMessage,
  previewTelegramBusinessAttachment,
  removeTelegramBusinessReaction,
  replyToTelegramBusinessMessage,
  restoreTelegramBusinessMessageVisibility,
  searchTelegramBusinessChats,
  searchTelegramBusinessMedia,
  searchTelegramBusinessMessages,
  searchTelegramBusinessTopics,
  sendTelegramBusinessMessage,
} from '../api/telegramBusinessApi'
import type {
  TelegramChat,
  TelegramChatDetailResponse,
  TelegramChatListResponse,
  TelegramChatMember,
  TelegramChatMemberListResponse,
  TelegramChatSearchResponse,
  TelegramForwardChainResponse,
  TelegramLifecycleResponse,
  TelegramMediaSearchResponse,
  TelegramMessage,
  TelegramMessageListResponse,
  TelegramMessageSearchResponse,
  TelegramMessageTombstoneListResponse,
  TelegramMessageVersionListResponse,
  TelegramReactionListResponse,
  TelegramReactionRequest,
  TelegramReactionResponse,
  TelegramReplyChainResponse,
  TelegramTopicListResponse,
} from '../../../shared/communications/types/telegram'
import type { TelegramRawMessageResponse } from '../../../shared/communications/types/telegramRawEvidence'
import type { AttachmentPreviewResponse } from '../types/attachments'
import type {
  CommunicationProviderMessageCommandResponse,
  MessagePinToggleResponse,
} from '../types/communications'

export const telegramBusinessQueryKeys = {
  chats: ['communications', 'telegram', 'chats'] as const,
  chatDetail: ['communications', 'telegram', 'chat-detail'] as const,
  chatMembers: ['communications', 'telegram', 'chat-members'] as const,
  messages: ['communications', 'telegram', 'messages'] as const,
  topics: ['communications', 'telegram', 'topics'] as const,
  topicMessages: ['communications', 'telegram', 'topic-messages'] as const,
  search: ['communications', 'telegram', 'search'] as const,
}

export function useTelegramChatsQuery(
  accountId?: MaybeRefOrGetter<string | undefined>,
  limit: MaybeRefOrGetter<number> = 50
) {
  return useQuery<TelegramChat[]>({
    queryKey: computed(() => [
      ...telegramBusinessQueryKeys.chats,
      toValue(accountId) ?? 'all',
      toValue(limit),
    ]),
    queryFn: async () => {
      const response: TelegramChatListResponse = await fetchTelegramBusinessChats(
        toValue(accountId),
        toValue(limit)
      )
      return response.items
    },
  })
}

export function useTelegramChatDetailQuery(
  telegramChatId: MaybeRefOrGetter<string | null | undefined>
) {
  return useQuery<TelegramChat | null>({
    queryKey: computed(() => [
      ...telegramBusinessQueryKeys.chatDetail,
      toValue(telegramChatId) ?? 'none',
    ]),
    queryFn: async () => {
      const value = toValue(telegramChatId)
      if (!value) return null
      const response: TelegramChatDetailResponse = await fetchTelegramBusinessChatDetail(value)
      return response.item
    },
    enabled: computed(() => Boolean(toValue(telegramChatId))),
  })
}

export function useTelegramChatMembersQuery(
  telegramChatId: MaybeRefOrGetter<string | null | undefined>,
  limit: MaybeRefOrGetter<number> = 50,
  query: MaybeRefOrGetter<string | null | undefined> = '',
  role: MaybeRefOrGetter<string | null | undefined> = ''
) {
  return useInfiniteQuery<
    TelegramChatMemberListResponse,
    Error,
    TelegramChatMember[],
    readonly unknown[],
    string | null
  >({
    queryKey: computed(() => [
      ...telegramBusinessQueryKeys.chatMembers,
      toValue(telegramChatId) ?? 'none',
      toValue(limit),
      normalizeTelegramBusinessQueryValue(toValue(query)),
      normalizeTelegramBusinessQueryValue(toValue(role)),
    ]),
    initialPageParam: null,
    queryFn: async ({ pageParam }) => {
      const value = toValue(telegramChatId)
      if (!value) return { items: [], next_cursor: null }
      return fetchTelegramBusinessChatMembers(
        value,
        toValue(limit),
        normalizeTelegramBusinessQueryValue(toValue(query)) || undefined,
        normalizeTelegramBusinessQueryValue(toValue(role)) || undefined,
        pageParam ?? undefined
      )
    },
    getNextPageParam: (lastPage) => lastPage.next_cursor ?? undefined,
    select: (data) => data.pages.flatMap((page) => page.items),
    enabled: computed(() => Boolean(toValue(telegramChatId))),
  })
}

export function useTelegramMessagesQuery(
  accountId?: MaybeRefOrGetter<string | null | undefined>,
  providerChatId?: MaybeRefOrGetter<string | null | undefined>,
  limit: MaybeRefOrGetter<number> = 50
) {
  return useQuery<TelegramMessage[]>({
    queryKey: computed(() => [
      ...telegramBusinessQueryKeys.messages,
      toValue(accountId) ?? 'all',
      toValue(providerChatId) ?? 'all',
      toValue(limit),
    ]),
    queryFn: async () => {
      const response = await fetchTelegramBusinessMessages(
        toValue(accountId) ?? undefined,
        toValue(providerChatId) ?? undefined,
        toValue(limit)
      )
      return response.items
    },
    enabled: computed(() => {
      const providerChatIdValue = toValue(providerChatId)
      if (providerChatIdValue === null) return false
      return providerChatIdValue === undefined || Boolean(toValue(accountId) && providerChatIdValue)
    }),
  })
}

export function useTelegramDialogSearchQuery(params: {
  q: MaybeRefOrGetter<string>
  accountId?: MaybeRefOrGetter<string | null | undefined>
  limit?: MaybeRefOrGetter<number>
}) {
  return useQuery<TelegramChatSearchResponse>({
    queryKey: computed(() => [
      ...telegramBusinessQueryKeys.search,
      'dialogs',
      toValue(params.q).trim(),
      toValue(params.accountId) ?? 'all',
      toValue(params.limit) ?? 20,
    ]),
    queryFn: () =>
      searchTelegramBusinessChats({
        q: toValue(params.q),
        account_id: toValue(params.accountId) ?? undefined,
        limit: toValue(params.limit) ?? 20,
      }),
    enabled: computed(() => toValue(params.q).trim().length >= 2),
  })
}

export function useTelegramMessageSearchQuery(params: {
  q: MaybeRefOrGetter<string>
  accountId?: MaybeRefOrGetter<string | null | undefined>
  providerChatId?: MaybeRefOrGetter<string | null | undefined>
  limit?: MaybeRefOrGetter<number>
}) {
  return useQuery<TelegramMessageSearchResponse>({
    queryKey: computed(() => [
      ...telegramBusinessQueryKeys.search,
      'messages',
      toValue(params.q).trim(),
      toValue(params.accountId) ?? 'all',
      toValue(params.providerChatId) ?? 'all',
      toValue(params.limit) ?? 50,
    ]),
    queryFn: () =>
      searchTelegramBusinessMessages({
        q: toValue(params.q),
        account_id: toValue(params.accountId) ?? undefined,
        provider_chat_id: toValue(params.providerChatId) ?? undefined,
        limit: toValue(params.limit) ?? 50,
      }),
    enabled: computed(() => toValue(params.q).trim().length >= 2),
  })
}

export function useTelegramMediaSearchQuery(params: {
  q?: MaybeRefOrGetter<string | null | undefined>
  accountId?: MaybeRefOrGetter<string | null | undefined>
  providerChatId?: MaybeRefOrGetter<string | null | undefined>
  kind?: MaybeRefOrGetter<string | null | undefined>
  limit?: MaybeRefOrGetter<number>
}) {
  return useQuery<TelegramMediaSearchResponse>({
    queryKey: computed(() => [
      ...telegramBusinessQueryKeys.search,
      'media',
      toValue(params.q)?.trim() ?? '',
      toValue(params.accountId) ?? 'all',
      toValue(params.providerChatId) ?? 'all',
      toValue(params.kind) ?? 'all',
      toValue(params.limit) ?? 100,
    ]),
    queryFn: () =>
      searchTelegramBusinessMedia({
        q: toValue(params.q) ?? undefined,
        account_id: toValue(params.accountId) ?? undefined,
        provider_chat_id: toValue(params.providerChatId) ?? undefined,
        kind: toValue(params.kind) ?? undefined,
        limit: toValue(params.limit) ?? 100,
      }),
    enabled: computed(() => Boolean(toValue(params.accountId) && toValue(params.providerChatId))),
  })
}

export function useTelegramPinnedMessagesQuery(params: {
  telegramChatId?: MaybeRefOrGetter<string | null | undefined>
  limit?: MaybeRefOrGetter<number>
}) {
  return useQuery<TelegramMessageListResponse>({
    queryKey: computed(() => [
      ...telegramBusinessQueryKeys.chats,
      toValue(params.telegramChatId) ?? 'none',
      'pinned-messages',
      toValue(params.limit) ?? 100,
    ]),
    queryFn: () =>
      fetchTelegramBusinessPinnedMessages({
        telegram_chat_id: toValue(params.telegramChatId) as string,
        limit: toValue(params.limit) ?? 100,
      }),
    enabled: computed(() => Boolean(toValue(params.telegramChatId))),
  })
}

export function useTelegramTopicsQuery(
  telegramChatId: MaybeRefOrGetter<string | null | undefined>,
  limit: MaybeRefOrGetter<number> = 100
) {
  return useQuery<TelegramTopicListResponse>({
    queryKey: computed(() => [
      ...telegramBusinessQueryKeys.topics,
      toValue(telegramChatId) ?? 'none',
      toValue(limit),
    ]),
    queryFn: async () => {
      const chatId = toValue(telegramChatId)
      if (!chatId) return { telegram_chat_id: '', items: [] }
      return fetchTelegramBusinessTopics(chatId, toValue(limit))
    },
    enabled: computed(() => Boolean(toValue(telegramChatId))),
  })
}

export function useTelegramTopicMessagesQuery(
  topicId: MaybeRefOrGetter<string | null | undefined>,
  limit: MaybeRefOrGetter<number> = 50
) {
  return useQuery<TelegramMessageListResponse>({
    queryKey: computed(() => [
      ...telegramBusinessQueryKeys.topicMessages,
      toValue(topicId) ?? 'none',
      toValue(limit),
    ]),
    queryFn: async () => {
      const tid = toValue(topicId)
      if (!tid) return { items: [] }
      return fetchTelegramBusinessTopicMessages(tid, toValue(limit))
    },
    enabled: computed(() => Boolean(toValue(topicId))),
  })
}

export function useTelegramTopicSearchQuery(
  telegramChatId: MaybeRefOrGetter<string | null | undefined>,
  q: MaybeRefOrGetter<string>,
  limit: MaybeRefOrGetter<number> = 50
) {
  return useQuery<TelegramTopicListResponse>({
    queryKey: computed(() => [
      ...telegramBusinessQueryKeys.search,
      'topics',
      toValue(telegramChatId) ?? 'none',
      toValue(q).trim(),
      toValue(limit),
    ]),
    queryFn: async () => {
      const chatId = toValue(telegramChatId)
      if (!chatId) return { telegram_chat_id: '', items: [] }
      return searchTelegramBusinessTopics(chatId, toValue(q), toValue(limit))
    },
    enabled: computed(() => Boolean(toValue(telegramChatId)) && toValue(q).trim().length >= 2),
  })
}

export function useTelegramMessageVersionsQuery(
  messageId: MaybeRefOrGetter<string | null | undefined>,
  enabled: MaybeRefOrGetter<boolean> = true
) {
  return useQuery<TelegramMessageVersionListResponse>({
    queryKey: computed(() => ['communications', 'messages', toValue(messageId), 'versions']),
    queryFn: () => fetchTelegramBusinessMessageVersions(toValue(messageId) as string),
    enabled: computed(() => Boolean(toValue(messageId)) && Boolean(toValue(enabled))),
  })
}

export function useTelegramMessageTombstonesQuery(
  messageId: MaybeRefOrGetter<string | null | undefined>,
  enabled: MaybeRefOrGetter<boolean> = true
) {
  return useQuery<TelegramMessageTombstoneListResponse>({
    queryK
```
_Source file truncated after 12000 characters. / Исходный файл обрезан после 12000 символов._

### `frontend/src/domains/communications/queries/threadInfiniteQuery.boundary.test.ts`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/domains/communications/queries/threadInfiniteQuery.boundary.test.ts`
- Size bytes / Размер в байтах: `1098`
- Included characters / Включено символов: `1098`
- Truncated / Обрезано: `no`

```typescript
import { describe, expect, it } from 'vitest'
import { readFileSync } from 'node:fs'

describe('thread infinite query boundary', () => {
	it('uses TanStack infinite query cursor loading for thread server state', () => {
		const source = readFileSync(
			new URL('./mailCoreQueries.ts', import.meta.url),
			'utf8'
		)

		expect(source).toContain('useInfiniteQuery')
		expect(source).toContain('fetchThreads(toValue(accountId), 50, pageParam)')
		expect(source).toContain('getNextPageParam: (lastPage) => lastPage.next_cursor ?? undefined')
		expect(source).toContain('select: (data) => data.pages.flatMap((page) => page.items)')
	})

	it('keeps thread message loading behind a dedicated query hook', () => {
		const source = readFileSync(
			new URL('./mailCoreQueries.ts', import.meta.url),
			'utf8'
		)

		expect(source).toContain('useThreadMessagesQuery')
		expect(source).toContain('fetchThreadMessages(')
		expect(source).toContain('communications-thread-messages')
		expect(source).toContain('enabled: computed(() => Boolean(toValue(accountId)?.trim() && toValue(subject)?.trim()))')
	})
})
```

### `frontend/src/domains/communications/queries/threadTranslationMutation.boundary.test.ts`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/domains/communications/queries/threadTranslationMutation.boundary.test.ts`
- Size bytes / Размер в байтах: `665`
- Included characters / Включено символов: `665`
- Truncated / Обрезано: `no`

```typescript
import { describe, expect, it } from 'vitest'
import { readFileSync } from 'node:fs'

describe('thread translation mutation boundary', () => {
  it('routes thread translation through TanStack mutation and the communications API client', () => {
    const source = readFileSync(new URL('./mailActionQueries.ts', import.meta.url), 'utf8')

    expect(source).toContain('translateThread')
    expect(source).toContain('ThreadTranslationResponse')
    expect(source).toContain('export function useTranslateThreadMutation()')
    expect(source).toContain('useMutation<')
    expect(source).toContain('translateThread(accountId, subject, targetLanguage, limit)')
  })
})
```

### `frontend/src/domains/communications/queries/useCommunicationsQuery.ts`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/domains/communications/queries/useCommunicationsQuery.ts`
- Size bytes / Размер в байтах: `245`
- Included characters / Включено символов: `245`
- Truncated / Обрезано: `no`

```typescript
export * from './mailActionQueries'
export * from './callQueries'
export * from './mailCoreQueries'
export * from './mailOperationQueries'
export * from './mailWorkspaceQueries'
export type { NullableQueryParam, QueryParam } from './queryTypes'
```

### `frontend/src/domains/communications/queries/whatsappBusinessQueries.ts`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/domains/communications/queries/whatsappBusinessQueries.ts`
- Size bytes / Размер в байтах: `14950`
- Included characters / Включено символов: `12000`
- Truncated / Обрезано: `yes`

```typescript
import { useInfiniteQuery, useMutation, useQuery, useQueryClient } from '@tanstack/vue-query'
import { computed, toValue, type MaybeRefOrGetter } from 'vue'
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
} from '../api/whatsappBusinessApi'
import type {
  WhatsAppLifecycleResponse,
  WhatsappWebMediaSearchResponse,
  WhatsappWebMessage,
  WhatsappWebMessageSearchResponse,
} from '../../../shared/communications/types/whatsapp'
import type {
  TelegramChatMember,
  TelegramChatMemberListResponse,
} from '../../../shared/communications/types/telegramMembers'
import type {
  TelegramReactionListResponse,
  TelegramReactionRequest,
  TelegramReactionResponse,
} from '../../../shared/communications/types/telegram'
import type { CommunicationProviderConversation } from '../types/providerChannels'
import type {
  CommunicationProviderMessageListResponse,
} from '../types/providerChannels'
import type {
  CommunicationProviderMessageCommandResponse,
  ConversationPinToggleResponse,
  MessagePinToggleResponse,
} from '../types/communications'

export const whatsappBusinessQueryKeys = {
  conversations: ['communications', 'whatsapp', 'conversations'] as const,
  conversationDetail: ['communications', 'whatsapp', 'conversation-detail'] as const,
  chatMembers: ['communications', 'whatsapp', 'chat-members'] as const,
  messages: ['communications', 'whatsapp', 'messages'] as const,
  search: ['communications', 'whatsapp', 'search'] as const,
}

export function useWhatsappBusinessConversationsQuery(
  accountId?: MaybeRefOrGetter<string | null | undefined>,
  limit: MaybeRefOrGetter<number> = 100
) {
  return useQuery<CommunicationProviderConversation[]>({
    queryKey: computed(() => [
      ...whatsappBusinessQueryKeys.conversations,
      toValue(accountId) ?? 'all',
      toValue(limit),
    ]),
    queryFn: async () => {
      const response = await fetchWhatsappWebBusinessConversations(
        toValue(accountId) ?? undefined,
        toValue(limit)
      )
      return response.items
    },
  })
}

export function useWhatsappBusinessMessagesQuery(
  accountId?: MaybeRefOrGetter<string | null | undefined>,
  providerChatId?: MaybeRefOrGetter<string | null | undefined>,
  limit: MaybeRefOrGetter<number> = 100
) {
  return useQuery<WhatsappWebMessage[]>({
    queryKey: computed(() => [
      ...whatsappBusinessQueryKeys.messages,
      toValue(accountId) ?? 'all',
      toValue(providerChatId) ?? 'all',
      toValue(limit),
    ]),
    queryFn: async () => {
      const response = await fetchWhatsappWebBusinessMessages(
        toValue(accountId) ?? undefined,
        toValue(providerChatId) ?? undefined,
        toValue(limit)
      )
      return response.items
    },
    enabled: computed(() => {
      const providerChatIdValue = toValue(providerChatId)
      if (providerChatIdValue === null) return false
      return providerChatIdValue === undefined || Boolean(toValue(accountId) && providerChatIdValue)
    }),
  })
}

export function useWhatsappConversationDetailQuery(
  conversationId: MaybeRefOrGetter<string | null | undefined>
) {
  return useQuery<CommunicationProviderConversation | null>({
    queryKey: computed(() => [
      ...whatsappBusinessQueryKeys.conversationDetail,
      toValue(conversationId) ?? 'none',
    ]),
    queryFn: async () => {
      const value = toValue(conversationId)
      if (!value) return null
      const response = await fetchWhatsappWebBusinessConversationDetail(value)
      return response.item
    },
    enabled: computed(() => Boolean(toValue(conversationId))),
  })
}

export function useWhatsappConversationMembersQuery(
  conversationId: MaybeRefOrGetter<string | null | undefined>,
  limit: MaybeRefOrGetter<number> = 50,
  query: MaybeRefOrGetter<string | null | undefined> = '',
  role: MaybeRefOrGetter<string | null | undefined> = ''
) {
  return useInfiniteQuery<
    TelegramChatMemberListResponse,
    Error,
    TelegramChatMember[],
    readonly unknown[],
    string | null
  >({
    queryKey: computed(() => [
      ...whatsappBusinessQueryKeys.chatMembers,
      toValue(conversationId) ?? 'none',
      toValue(limit),
      normalizeWhatsappQueryValue(toValue(query)),
      normalizeWhatsappQueryValue(toValue(role)),
    ]),
    initialPageParam: null,
    queryFn: async ({ pageParam }) => {
      const value = toValue(conversationId)
      if (!value) return { items: [], next_cursor: null }
      return fetchWhatsappWebBusinessConversationMembers(
        value,
        toValue(limit),
        normalizeWhatsappQueryValue(toValue(query)) || undefined,
        normalizeWhatsappQueryValue(toValue(role)) || undefined,
        pageParam ?? undefined
      )
    },
    getNextPageParam: (lastPage) => lastPage.next_cursor ?? undefined,
    select: (data) => data.pages.flatMap((page) => page.items),
    enabled: computed(() => Boolean(toValue(conversationId))),
  })
}

export function useWhatsappMessageSearchQuery(params: {
  q: MaybeRefOrGetter<string>
  accountId?: MaybeRefOrGetter<string | null | undefined>
  providerChatId?: MaybeRefOrGetter<string | null | undefined>
  limit?: MaybeRefOrGetter<number>
}) {
  return useQuery<WhatsappWebMessageSearchResponse>({
    queryKey: computed(() => [
      ...whatsappBusinessQueryKeys.search,
      'messages',
      toValue(params.q).trim(),
      toValue(params.accountId) ?? 'all',
      toValue(params.providerChatId) ?? 'all',
      toValue(params.limit) ?? 50,
    ]),
    queryFn: () =>
      searchWhatsappWebBusinessMessages({
        q: toValue(params.q),
        account_id: toValue(params.accountId) ?? undefined,
        provider_chat_id: toValue(params.providerChatId) ?? undefined,
        limit: toValue(params.limit) ?? 50,
      }),
    enabled: computed(() => toValue(params.q).trim().length >= 2),
  })
}

export function useWhatsappMediaSearchQuery(params: {
  q?: MaybeRefOrGetter<string | null | undefined>
  accountId?: MaybeRefOrGetter<string | null | undefined>
  providerChatId?: MaybeRefOrGetter<string | null | undefined>
  kind?: MaybeRefOrGetter<string | null | undefined>
  limit?: MaybeRefOrGetter<number>
}) {
  return useQuery<WhatsappWebMediaSearchResponse>({
    queryKey: computed(() => [
      ...whatsappBusinessQueryKeys.search,
      'media',
      toValue(params.q)?.trim() ?? '',
      toValue(params.accountId) ?? 'all',
      toValue(params.providerChatId) ?? 'all',
      toValue(params.kind) ?? 'all',
      toValue(params.limit) ?? 100,
    ]),
    queryFn: () =>
      searchWhatsappWebBusinessMedia({
        q: toValue(params.q) ?? undefined,
        account_id: toValue(params.accountId) ?? undefined,
        provider_chat_id: toValue(params.providerChatId) ?? undefined,
        kind: toValue(params.kind) ?? undefined,
        limit: toValue(params.limit) ?? 100,
      }),
    enabled: computed(() => Boolean(toValue(params.accountId) && toValue(params.providerChatId))),
  })
}

export function useWhatsappPinnedMessagesQuery(params: {
  conversationId?: MaybeRefOrGetter<string | null | undefined>
  limit?: MaybeRefOrGetter<number>
}) {
  return useQuery<CommunicationProviderMessageListResponse>({
    queryKey: computed(() => [
      ...whatsappBusinessQueryKeys.conversations,
      toValue(params.conversationId) ?? 'none',
      'pinned-messages',
      toValue(params.limit) ?? 100,
    ]),
    queryFn: () =>
      fetchWhatsappWebBusinessPinnedMessages({
        conversation_id: toValue(params.conversationId) as string,
        limit: toValue(params.limit) ?? 100,
      }),
    enabled: computed(() => Boolean(toValue(params.conversationId))),
  })
}

export function useWhatsappMessageReactionsQuery(
  messageId: MaybeRefOrGetter<string | null | undefined>,
  enabled: MaybeRefOrGetter<boolean> = true
) {
  return useQuery<TelegramReactionListResponse>({
    queryKey: computed(() => ['communications', 'messages', toValue(messageId), 'reactions']),
    queryFn: () => fetchWhatsappBusinessReactions(toValue(messageId) as string),
    enabled: computed(() => Boolean(toValue(messageId)) && Boolean(toValue(enabled))),
  })
}

function normalizeWhatsappQueryValue(value: string | null | undefined): string {
  return value?.trim() ?? ''
}

function useInvalidateWhatsappBusinessState() {
  const queryClient = useQueryClient()
  return () => {
    queryClient.invalidateQueries({ queryKey: whatsappBusinessQueryKeys.messages })
    queryClient.invalidateQueries({ queryKey: whatsappBusinessQueryKeys.conversations })
    queryClient.invalidateQueries({ queryKey: whatsappBusinessQueryKeys.conversationDetail })
    queryClient.invalidateQueries({ queryKey: ['communications', 'messages'] })
  }
}

export function useSendWhatsappMessageMutation() {
  const invalidate = useInvalidateWhatsappBusinessState()
  return useMutation<
    CommunicationProviderMessageCommandResponse,
    Error,
    { account_id: string; provider_chat_id: string; text: string }
  >({
    mutationFn: sendWhatsappBusinessMessage,
    onSuccess: invalidate,
  })
}

export function useReplyWhatsappMessageMutation() {
  const invalidate = useInvalidateWhatsappBusinessState()
  return useMutation<
    CommunicationProviderMessageCommandResponse,
    Error,
    {
      message_id: string
      account_id?: string
      provider_chat_id?: string
      reply_to_provider_message_id?: string
      text: string
    }
  >({
    mutationFn: (request) =>
      replyToWhatsappBusinessMessage({ message_id: request.message_id, text: request.text }),
    onSuccess: invalidate,
  })
}

export function useForwardWhatsappMessageMutation() {
  const invalidate = useInvalidateWhatsappBusinessState()
  return useMutation<
    CommunicationProviderMessageCommandResponse,
    Error,
    {
      message_id: string
      account_id?: string
      provider_chat_id: string
      from_provider_chat_id?: string
      from_provider_message_id?: string
    }
  >({
    mutationFn: (request) =>
      forwardWhatsappBusinessMessage({
        message_id: request.message_id,
        provider_chat_id: request.provider_chat_id,
      }),
    onSuccess: invalidate,
  })
}

export function useEditWhatsappMessageMutation() {
  const invalidate = useInvalidateWhatsappBusinessState()
  return useMutation<
    WhatsAppLifecycleResponse,
    Error,
    Parameters<typeof editWhatsappBusinessMessage>[0]
  >({
    mutationFn: editWhatsappBusinessMessage,
    onSuccess: invalidate,
  })
}

export function useDeleteWhatsappMessageMutation() {
  const invalidate = useInvalidateWhatsappBusinessState()
  return useMutation<
    WhatsAppLifecycleResponse,
    Error,
    Parameters<typeof deleteWhatsappBusinessMessage>[0]
  >({
    mutationFn: deleteWhatsappBusinessMessage,
    onSuccess: invalidate,
  })
}

export function usePinWhatsappMessageMutation() {
  const invalidate = useInvalidateWhatsappBusinessState()
  return useMutation<MessagePinToggleResponse, Error, { message_id: string }>({
    mutationFn: pinWhatsappBusinessMessage,
    onSuccess: invalidate,
  })
}

export function useAddWhatsappReactionMutation() {
  const invalidate = useInvalidateWhatsappBusinessState()
  return useMutation<
    TelegramReactionResponse,
    Error,
    {
```
_Source file truncated after 12000 characters. / Исходный файл обрезан после 12000 символов._

### `frontend/src/domains/communications/stores/communications.test.ts`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/domains/communications/stores/communications.test.ts`
- Size bytes / Размер в байтах: `4840`
- Included characters / Включено символов: `4840`
- Truncated / Обрезано: `no`

```typescript
import { beforeEach, describe, expect, it } from 'vitest'
import { createPinia, setActivePinia } from 'pinia'
import {
	communicationSectionWorkflowState,
	communicationWorkflowStateSectionId,
	useCommunicationsStore
} from './communications'
import type { CommunicationMessageSummary, MailSyncStatus } from '../types/communications'

beforeEach(() => {
	setActivePinia(createPinia())
})

describe('communication section workflow mapping', () => {
	it('maps UI section ids to backend workflow states', () => {
		expect(communicationSectionWorkflowState('unified')).toBe('')
		expect(communicationSectionWorkflowState('inbox')).toBe('new')
		expect(communicationSectionWorkflowState('needs_reply')).toBe('needs_action')
		expect(communicationSectionWorkflowState('waiting')).toBe('waiting')
		expect(communicationSectionWorkflowState('done')).toBe('done')
		expect(communicationSectionWorkflowState('archived')).toBe('archived')
	})

	it('maps backend workflow states back to UI section ids', () => {
		expect(communicationWorkflowStateSectionId('')).toBe('unified')
		expect(communicationWorkflowStateSectionId('new')).toBe('inbox')
		expect(communicationWorkflowStateSectionId('needs_action')).toBe('needs_reply')
		expect(communicationWorkflowStateSectionId('waiting')).toBe('waiting')
		expect(communicationWorkflowStateSectionId('done')).toBe('done')
		expect(communicationWorkflowStateSectionId('archived')).toBe('archived')
	})
})

describe('communications multi-select state', () => {
	it('toggles selected message ids and clears selections', () => {
		const store = useCommunicationsStore()

		store.toggleMessageSelection('msg-1')
		store.toggleMessageSelection('msg-2')
		store.toggleMessageSelection('msg-1')

		expect(store.selectedMessageIds).toEqual(['msg-2'])
		expect(store.selectedMessageIdSet.has('msg-2')).toBe(true)

		store.clearMessageSelection()

		expect(store.selectedMessageIds).toEqual([])
		expect(store.selectedMessageIdSet.size).toBe(0)
	})

	it('selects a visible message range from the last selection anchor', () => {
		const store = useCommunicationsStore()
		store.setMessages([
			messageSummary('msg-1'),
			messageSummary('msg-2'),
			messageSummary('msg-3'),
			messageSummary('msg-4')
		])

		store.toggleMessageSelection('msg-1')
		store.toggleMessageSelection('msg-4', true)

		expect(store.selectedMessageIds).toEqual(['msg-1', 'msg-2', 'msg-3', 'msg-4'])
	})

	it('selects the current visible message id set for keyboard select-all', () => {
		const store = useCommunicationsStore()
		store.setMessages([
			messageSummary('msg-1'),
			messageSummary('msg-2'),
			messageSummary('msg-3')
		])

		store.selectVisibleMessages(['msg-2', 'msg-1', 'msg-2', ''])

		expect(store.selectedMessageIds).toEqual(['msg-2', 'msg-1'])

		store.toggleMessageSelection('msg-3', true)

		expect(store.selectedMessageIds).toEqual(['msg-1', 'msg-2', 'msg-3'])
	})
})

describe('communications mail account selection', () => {
	it('selects the first synced account when no account is selected', () => {
		const store = useCommunicationsStore()

		store.setMailSyncStatuses([
			mailSyncStatus('account-1'),
			mailSyncStatus('account-2')
		])

		expect(store.selectedMailAccountId).toBe('account-1')
	})

	it('preserves an explicit selected account when sync statuses refresh', () => {
		const store = useCommunicationsStore()
		store.setSelectedMailAccountId('account-2')

		store.setMailSyncStatuses([
			mailSyncStatus('account-1'),
			mailSyncStatus('account-2')
		])

		expect(store.selectedMailAccountId).toBe('account-2')
	})
})

function messageSummary(messageId: string): CommunicationMessageSummary {
	return {
		message_id: messageId,
		raw_record_id: `raw-${messageId}`,
		account_id: 'account-1',
		provider_record_id: `provider-${messageId}`,
		subject: `Subject ${messageId}`,
		sender: 'sender@example.com',
		recipients: ['recipient@example.com'],
		body_text_preview: 'Preview',
		occurred_at: null,
		projected_at: '2026-06-14T00:00:00Z',
		channel_kind: 'email',
		conversation_id: null,
		sender_display_name: null,
		delivery_state: 'received',
		workflow_state: 'new',
		importance_score: null,
		ai_category: null,
		ai_summary: null,
		ai_summary_generated_at: null,
		message_metadata: {},
		attachment_count: 0,
		local_state: 'active',
		local_state_changed_at: null
	}
}

function mailSyncStatus(accountId: string): MailSyncStatus {
	return {
		account_id: accountId,
		status: 'idle',
		phase: 'idle',
		progress_mode: 'none',
		progress_percent: null,
		processed_messages: 0,
		estimated_total_messages: null,
		current_batch_size: 0,
		last_started_at: null,
		last_completed_at: null,
		next_run_at: null,
		last_error_code: null,
		last_error_message: null,
		last_fetched_messages: 0,
		last_projected_messages: 0,
		last_upserted_persons: 0,
		last_upserted_organizations: 0
	}
}
```

### `frontend/src/domains/communications/stores/communications.ts`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/domains/communications/stores/communications.ts`
- Size bytes / Размер в байтах: `16730`
- Included characters / Включено символов: `12000`
- Truncated / Обрезано: `yes`

```typescript
import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import { format, formatDistanceToNow } from 'date-fns'
import type {
  CommunicationMessageSummary,
  CommunicationMessageDetailResponse,
  WorkflowState,
  LocalMessageState,
  MailSyncStatus,
  MailboxHealth,
  CommunicationDraft,
  ComposeFormModel,
  NavigatorMode,
  InspectorMode,
  MessageContextTab,
  MessageExportResponse,
  CommunicationMessageInsight,
  CommunicationSectionId,
  CommunicationThreadSummary,
  ProjectItem,
  TaskItem
} from '../types/communications'

const emptyComposeForm: ComposeFormModel = {
  mode: 'compose',
  draftId: '',
  accountId: '',
  toText: '',
  ccText: '',
  bccText: '',
  subject: '',
  body: '',
  bodyHtml: null,
  bodyFormat: 'plain',
  scheduledSendAt: '',
  undoSendSeconds: null,
  inReplyTo: null
}

export const useCommunicationsStore = defineStore('communications-ui', () => {
  // --- Message list state ---
  const communicationMessages = ref<CommunicationMessageSummary[]>([])
  const selectedCommunicationDetail = ref<CommunicationMessageDetailResponse | null>(null)
  const communicationsError = ref('')
  const isCommunicationsLoading = ref(false)
  const selectedConversationIndex = ref(-1)
  const selectedCommunicationMessageId = ref('')
  const selectedMessageIds = ref<string[]>([])
  const selectionAnchorMessageId = ref('')
  const selectedMessageIdSet = computed(() => new Set(selectedMessageIds.value))

  // --- Filters ---
  const mailStateFilter = ref<WorkflowState | ''>('')
  const mailLocalStateFilter = ref<LocalMessageState>('active')
  const mailStateCounts = ref<{ state: string; count: number }[]>([])
  const isMailStateTransitioning = ref(false)

  // --- AI ---
  const isAiAnswerSubmitting = ref(false)
  const aiAnalysisResult = ref<Record<string, unknown> | null>(null)

  // --- Drafts ---
  const drafts = ref<CommunicationDraft[]>([])

  // --- Health ---
  const mailboxHealth = ref<MailboxHealth | null>(null)

  // --- Senders ---
  const topSenders = ref<{ sender: string; message_count: number }[]>([])

  // --- Threads ---
  const threads = ref<CommunicationThreadSummary[]>([])
  const selectedThread = ref<CommunicationThreadSummary | null>(null)
  const selectedThreadId = computed(() => selectedThread.value?.thread_id ?? '')

  // --- Resources ---
  const mailResources = ref<Record<string, unknown>>({})
  const mailResourceSummary = ref<Record<string, number>>({})

  // --- Message insight ---
  const mailMessageInsight = ref<CommunicationMessageInsight | null>(null)

  // --- Action status ---
  const isMailActionRunning = ref(false)
  const mailActionStatus = ref('')
  const mailActionError = ref('')
  const lastMessageExport = ref<MessageExportResponse | null>(null)

  // --- Sync ---
  const mailSyncStatuses = ref<MailSyncStatus[]>([])
  const selectedMailSyncSettings = ref<{ account_id: string; sync_enabled: boolean; batch_size: number; poll_interval_seconds: number } | null>(null)
  const lastMailSyncRuns = ref<Record<string, unknown>[]>([])
  const isMailSyncBusy = ref(false)
  const mailSyncStatusMessage = ref('')
  const mailSyncError = ref('')

  // --- Compose ---
  const isComposeOpen = ref(false)
  const composeForm = ref<ComposeFormModel>({ ...emptyComposeForm })
  const selectedMailAccountId = ref('')
  const isSendReviewOpen = ref(false)
  const isSendingMessage = ref(false)
  const composeSendError = ref('')
  const composeStatusMessage = ref('')
  const lastSendResponse = ref<Record<string, unknown> | null>(null)

  // --- UI state ---
  const messageSearchQuery = ref('')
  const communicationsNavigatorMode = ref<NavigatorMode>('threads')
  const expandedCommunicationContactKey = ref<string | null>(null)
  const communicationsInspectorMode = ref<InspectorMode>('context')
  const activeMessageContextTab = ref<MessageContextTab>('message')
  const communicationProjects = ref<ProjectItem[]>([])
  const communicationTasks = ref<TaskItem[]>([])

  // --- Derived: selected communication ---
  const selectedCommunication = computed(() => {
    const idx = selectedConversationIndex.value
    if (idx >= 0 && idx < communicationMessages.value.length) {
      return communicationMessages.value[idx]
    }
    return null
  })

  // --- Actions ---

  function setMessages(messages: CommunicationMessageSummary[]) {
    communicationMessages.value = messages
    const visibleIds = new Set(messages.map((message) => message.message_id))
    selectedMessageIds.value = selectedMessageIds.value.filter((messageId) => visibleIds.has(messageId))
    if (selectionAnchorMessageId.value && !visibleIds.has(selectionAnchorMessageId.value)) {
      selectionAnchorMessageId.value = ''
    }
  }

  function selectMessage(index: number) {
    clearSelectedThread()
    selectedConversationIndex.value = index
    if (index >= 0 && index < communicationMessages.value.length) {
      selectedCommunicationMessageId.value = communicationMessages.value[index].message_id
      return
    }
    selectedCommunicationMessageId.value = ''
  }

  function selectMessageId(messageId: string) {
    clearSelectedThread()
    selectedCommunicationMessageId.value = messageId
    selectedConversationIndex.value = communicationMessages.value.findIndex((message) => message.message_id === messageId)
  }

  function toggleMessageSelection(messageId: string, extendRange = false) {
    const normalized = messageId.trim()
    if (!normalized) return
    if (extendRange && selectionAnchorMessageId.value) {
      selectMessageRange(selectionAnchorMessageId.value, normalized)
      return
    }
    if (selectedMessageIdSet.value.has(normalized)) {
      selectedMessageIds.value = selectedMessageIds.value.filter((id) => id !== normalized)
      if (selectionAnchorMessageId.value === normalized) selectionAnchorMessageId.value = ''
      return
    }
    selectedMessageIds.value = [...selectedMessageIds.value, normalized]
    selectionAnchorMessageId.value = normalized
  }

  function selectMessageRange(anchorMessageId: string, targetMessageId: string) {
    const anchorIndex = communicationMessages.value.findIndex((message) => message.message_id === anchorMessageId)
    const targetIndex = communicationMessages.value.findIndex((message) => message.message_id === targetMessageId)
    if (anchorIndex < 0 || targetIndex < 0) {
      toggleMessageSelection(targetMessageId)
      return
    }

    const [start, end] = anchorIndex < targetIndex
      ? [anchorIndex, targetIndex]
      : [targetIndex, anchorIndex]
    const existing = new Set(selectedMessageIds.value)
    for (const message of communicationMessages.value.slice(start, end + 1)) {
      existing.add(message.message_id)
    }
    selectedMessageIds.value = communicationMessages.value
      .map((message) => message.message_id)
      .filter((id) => existing.has(id))
  }

  function clearMessageSelection() {
    selectedMessageIds.value = []
    selectionAnchorMessageId.value = ''
  }

  function selectVisibleMessages(messageIds: string[]) {
    const uniqueIds = [...new Set(messageIds.map((messageId) => messageId.trim()).filter(Boolean))]
    selectedMessageIds.value = uniqueIds
    selectionAnchorMessageId.value = uniqueIds[0] ?? ''
  }

  function setMessageDetail(detail: CommunicationMessageDetailResponse | null) {
    selectedCommunicationDetail.value = detail
  }

  function setCommunicationsError(error: string) {
    communicationsError.value = error
  }

  function setCommunicationsLoading(loading: boolean) {
    isCommunicationsLoading.value = loading
  }

  function setStateFilter(state: WorkflowState | '') {
    mailStateFilter.value = state
  }

  function setLocalStateFilter(state: LocalMessageState) {
    mailLocalStateFilter.value = state
  }

  function setStateCounts(counts: { state: string; count: number }[]) {
    mailStateCounts.value = counts
  }

  function setMailSyncStatuses(statuses: MailSyncStatus[]) {
    mailSyncStatuses.value = statuses
    if (!selectedMailAccountId.value) {
      selectedMailAccountId.value = statuses[0]?.account_id ?? ''
    }
  }

  function setMailSyncStatusMessage(msg: string) {
    mailSyncStatusMessage.value = msg
  }

  function setMailSyncError(err: string) {
    mailSyncError.value = err
  }

  function setIsMailSyncBusy(busy: boolean) {
    isMailSyncBusy.value = busy
  }

  function setDrafts(draftList: CommunicationDraft[]) {
    drafts.value = draftList
  }

  function setMailboxHealth(health: MailboxHealth | null) {
    mailboxHealth.value = health
  }

  function setTopSenders(senders: { sender: string; message_count: number }[]) {
    topSenders.value = senders
  }

  function setThreads(threadList: CommunicationThreadSummary[]) {
    threads.value = threadList
  }

  function selectThread(thread: CommunicationThreadSummary) {
    selectedThread.value = thread
    selectedConversationIndex.value = -1
    selectedCommunicationMessageId.value = ''
    selectedCommunicationDetail.value = null
    mailMessageInsight.value = null
  }

  function clearSelectedThread() {
    selectedThread.value = null
  }

  function setCommunicationMessageInsight(insight: CommunicationMessageInsight | null) {
    mailMessageInsight.value = insight
  }

  function setIsMailActionRunning(running: boolean) {
    isMailActionRunning.value = running
  }

  function setMailActionStatus(status: string) {
    mailActionStatus.value = status
  }

  function setMailActionError(error: string) {
    mailActionError.value = error
  }

  function setLastMessageExport(exported: MessageExportResponse | null) {
    lastMessageExport.value = exported
  }

  // --- Compose actions ---

  function openCompose(form: ComposeFormModel) {
    composeForm.value = { ...form }
    isComposeOpen.value = true
  }

  function closeCompose() {
    isComposeOpen.value = false
    composeForm.value = { ...emptyComposeForm }
    isSendReviewOpen.value = false
    composeSendError.value = ''
    composeStatusMessage.value = ''
  }

  function updateComposeForm(partial: Partial<ComposeFormModel>) {
    composeForm.value = { ...composeForm.value, ...partial }
  }

  function setComposeStatusMessage(msg: string) {
    composeStatusMessage.value = msg
  }

  function setComposeSendError(err: string) {
    composeSendError.value = err
  }

  function setIsSendingMessage(sending: boolean) {
    isSendingMessage.value = sending
  }

  function openSendReview() {
    isSendReviewOpen.value = true
  }

  function closeSendReview() {
    isSendReviewOpen.value = false
  }

  // --- Search ---

  function setMessageSearchQuery(query: string) {
    messageSearchQuery.value = query
  }

  // --- Navigator ---

  function setNavigatorMode(mode: NavigatorMode) {
    communicationsNavigatorMode.value = mode
  }

  function setExpandedContactKey(key: string | null) {
    expandedCommunicationContactKey.value = key
  }

  // --- Inspector ---

  function setInspectorMode(mode: InspectorMode) {
    communicationsInspectorMode.value = mode
  }

  function setActiveMessageContextTab(tab: MessageContextTab) {
    activeMessageContextTab.value = tab
  }

  // --- Projects & Tasks ---

  function setCommunicationProjects(projects: ProjectItem[]) {
    communicationProjects.value = projects
  }

  function setCommunicationTasks(tasks: TaskItem[]) {
    communicationTasks.value = tasks
  }

  // --- Account ---

  function setSelectedMailAccountId(accountId: string) {
    selectedMailAccountId.value = accountId
  }

  return {
    // State
    communicationMessages, selectedCommunicationDetail, communicationsError, isCommunicationsLoading,
    selectedConversationIndex, selectedCommunicationMessageId, selectedMessageIds,
    mailStateFilter, mailLocalStateFilter, mailStateCounts, isMailStateTransitioning,
    isAiAnswerSubmitting, aiAnalysisResult,
    drafts, mailboxHealth, topSenders, threads, selectedThread, selectedThreadId,
    mailResources, mailResourceSummary, mailMessageInsight,
    isMailActionRunning, mailActionStatus, mailAction
```
_Source file truncated after 12000 characters. / Исходный файл обрезан после 12000 символов._

### `frontend/src/domains/communications/types/aiState.ts`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/domains/communications/types/aiState.ts`
- Size bytes / Размер в байтах: `480`
- Included characters / Включено символов: `480`
- Truncated / Обрезано: `no`

```typescript
export type CommunicationAiState =
  | 'NEW'
  | 'PROCESSING'
  | 'PROCESSED'
  | 'REVIEW_REQUIRED'
  | 'FAILED'
  | 'ARCHIVED'

export type CommunicationAiStateRecord = {
  message_id: string
  ai_state: CommunicationAiState
  review_reason: string | null
  last_error: string | null
  created_at: string
  updated_at: string
}

export type CommunicationAiStateTransitionRequest = {
  ai_state: CommunicationAiState
  review_reason?: string | null
  last_error?: string | null
}
```

### `frontend/src/domains/communications/types/attachments.ts`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/domains/communications/types/attachments.ts`
- Size bytes / Размер в байтах: `2370`
- Included characters / Включено символов: `2370`
- Truncated / Обрезано: `no`

```typescript
export type AttachmentScanStatus = 'not_scanned' | 'clean' | 'suspicious' | 'malicious' | 'failed'

export type AttachmentSearchRequest = {
  account_id?: string
  q?: string
  content_type?: string
  scan_status?: AttachmentScanStatus
  cursor?: string | null
  limit?: number
}

export type AttachmentSearchResult = {
  attachment_id: string
  message_id: string
  raw_record_id: string
  account_id: string
  message_subject: string
  sender: string
  occurred_at: string | null
  blob_id: string
  provider_attachment_id: string
  filename: string | null
  content_type: string
  size_bytes: number
  sha256: string
  disposition: 'attachment' | 'inline' | 'unknown'
  scan_status: AttachmentScanStatus
  scan_engine: string | null
  scan_checked_at: string | null
  scan_summary: string | null
  storage_kind: string
  storage_path: string
  created_at: string
  updated_at: string
}

export type AttachmentSearchResponse = {
  items: AttachmentSearchResult[]
  next_cursor: string | null
  has_more: boolean
}

export type ArchiveInspectionEntry = {
  name: string
  normalized_path: string
  compressed_size: number
  uncompressed_size: number
  is_dir: boolean
  is_nested_archive: boolean
}

export type ArchiveInspectionReport = {
  archive_kind: 'zip'
  entry_count: number
  total_uncompressed_bytes: number
  has_nested_archive: boolean
  entries: ArchiveInspectionEntry[]
}

export type AttachmentArchiveInspectionResponse = {
  attachment_id: string
  message_id: string
  filename: string | null
  content_type: string
  scan_status: AttachmentScanStatus
  report: ArchiveInspectionReport
}

export type AttachmentPreviewResponse = {
  attachment_id: string
  message_id: string
  filename: string | null
  content_type: string
  scan_status: AttachmentScanStatus
  preview_kind: 'text' | 'image' | 'audio' | 'video' | 'pdf'
  text: string
  data_url: string | null
  truncated: boolean
  byte_count: number
  max_preview_bytes: number
}

export type AttachmentTranslationRequest = {
  target_language: string
  source_text: string
}

export type AttachmentTranslationResponse = {
  attachment_id: string
  message_id: string
  filename: string | null
  original_language: string
  confidence: number
  translated: boolean
  text: string | null
  target: string
  model: string | null
  reason: string | null
  source: 'caller_provided_extracted_text'
}
```

### `frontend/src/domains/communications/types/bilingualReplyFlow.ts`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/domains/communications/types/bilingualReplyFlow.ts`
- Size bytes / Размер в байтах: `932`
- Included characters / Включено символов: `932`
- Truncated / Обрезано: `no`

```typescript
export const bilingualReplyToneOptions = [
  'formal',
  'business',
  'friendly',
  'short',
  'detailed'
] as const

export type BilingualReplyTone = typeof bilingualReplyToneOptions[number]

export type BilingualReplyFlowRequest = {
  reply_text_ru: string
  tone: BilingualReplyTone
}

export type BilingualOriginal = {
  language: string
  confidence: number
  text: string
}

export type BilingualTranslationStep = {
  target: string
  translated: boolean
  text: string | null
  model: string | null
  reason: string | null
}

export type BilingualReplyDraft = {
  language: 'ru'
  tone: BilingualReplyTone
  text: string
}

export type BilingualReplyFlowResponse = {
  message_id: string
  subject: string
  tone: BilingualReplyTone
  reply_language: 'ru'
  send_ready: boolean
  original: BilingualOriginal
  translation: BilingualTranslationStep
  reply: BilingualReplyDraft
  back_translation: BilingualTranslationStep
}
```

### `frontend/src/domains/communications/types/certificates.ts`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/domains/communications/types/certificates.ts`
- Size bytes / Размер в байтах: `2414`
- Included characters / Включено символов: `2414`
- Truncated / Обрезано: `no`

```typescript
export type CertificateType =
  | 'smime'
  | 'pgp'
  | 'pdf_sign'
  | 'cades'
  | 'xades'
  | 'gost_sign'
  | 'unknown'

export type CertificateProvider =
  | 'fnmt'
  | 'dnie'
  | 'cryptopro'
  | 'gost'
  | 'apple_keychain'
  | 'pkcs12'
  | 'yubikey'
  | 'usb_token'
  | 'other'

export type CertificateStorageKind =
  | 'os_keychain'
  | 'encrypted_vault'
  | 'pkcs12_file'
  | 'pfx_file'
  | 'smart_card'
  | 'usb_token'
  | 'external_vault'

export type CertificateTrustStatus =
  | 'trusted'
  | 'untrusted'
  | 'expired'
  | 'revoked'
  | 'pending_verification'
  | 'self_signed'

export type MailCertificate = {
  cert_id: string
  owner_name: string
  issuer: string
  serial_number: string | null
  fingerprint_sha256: string | null
  valid_from: string | null
  valid_until: string | null
  cert_type: CertificateType
  provider: CertificateProvider
  storage_kind: CertificateStorageKind
  storage_ref: string | null
  trust_status: CertificateTrustStatus
  is_revoked: boolean
  usage: string[]
  linked_message_id: string | null
  metadata: Record<string, unknown>
  created_at: string
  updated_at: string
}

export type MailCertificateListResponse = {
  items: MailCertificate[]
}

export type MailCertificateCreateRequest = {
  cert_id: string
  owner_name: string
  issuer: string
  serial_number?: string | null
  fingerprint_sha256?: string | null
  valid_from?: string | null
  valid_until?: string | null
  cert_type?: CertificateType
  provider?: CertificateProvider
  storage_kind?: CertificateStorageKind
  storage_ref?: string | null
  trust_status?: CertificateTrustStatus
  is_revoked?: boolean
  usage?: string[]
  linked_message_id?: string | null
  metadata?: Record<string, unknown>
}

export const certificateTypeOptions: CertificateType[] = [
  'smime',
  'pgp',
  'pdf_sign',
  'cades',
  'xades',
  'gost_sign',
  'unknown'
]

export const certificateProviderOptions: CertificateProvider[] = [
  'fnmt',
  'dnie',
  'cryptopro',
  'gost',
  'apple_keychain',
  'pkcs12',
  'yubikey',
  'usb_token',
  'other'
]

export const certificateStorageKindOptions: CertificateStorageKind[] = [
  'os_keychain',
  'encrypted_vault',
  'pkcs12_file',
  'pfx_file',
  'smart_card',
  'usb_token',
  'external_vault'
]

export const certificateTrustStatusOptions: CertificateTrustStatus[] = [
  'trusted',
  'untrusted',
  'expired',
  'revoked',
  'pending_verification',
  'self_signed'
]
```

### `frontend/src/domains/communications/types/communications.ts`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/domains/communications/types/communications.ts`
- Size bytes / Размер в байтах: `15884`
- Included characters / Включено символов: `12000`
- Truncated / Обрезано: `yes`

```typescript
// --- Re-exported API types from Svelte reference ---

import type { MailCertificate } from './certificates'

export type LocalMessageState = 'active' | 'trash' | 'all'

export type WorkflowState = 'new' | 'reviewed' | 'needs_action' | 'waiting' | 'done' | 'archived' | 'muted' | 'spam'

export type CommunicationMessageSummary = {
  message_id: string
  raw_record_id: string
  observation_id?: string | null
  account_id: string
  provider_record_id: string
  subject: string
  sender: string
  recipients: string[]
  body_text_preview: string
  occurred_at: string | null
  projected_at: string
  channel_kind: string
  conversation_id: string | null
  sender_display_name: string | null
  delivery_state: string
  workflow_state: WorkflowState
  importance_score: number | null
  ai_category: string | null
  ai_summary: string | null
  ai_summary_generated_at: string | null
  message_metadata: Record<string, unknown>
  attachment_count: number
  local_state: LocalMessageState
  local_state_changed_at: string | null
}

export type CommunicationMessagesResponse = {
  items: CommunicationMessageSummary[]
  next_cursor: string | null
  has_more: boolean
}

export type CommunicationAttachment = {
  attachment_id: string
  message_id: string
  raw_record_id: string
  blob_id: string
  provider_attachment_id: string
  filename: string | null
  content_type: string
  size_bytes: number
  sha256: string
  disposition: 'attachment' | 'inline' | 'unknown'
  scan_status: 'not_scanned' | 'clean' | 'suspicious' | 'malicious' | 'failed'
  scan_engine: string | null
  scan_checked_at: string | null
  scan_summary: string | null
  scan_metadata: Record<string, unknown>
  storage_kind: string
  storage_path: string
  created_at: string
  updated_at: string
}

export type CommunicationMessageDetailItem = {
  message_id: string
  raw_record_id: string
  observation_id?: string | null
  account_id: string
  provider_record_id: string
  subject: string
  sender: string
  recipients: string[]
  body_text: string
  body_html: string | null
  occurred_at: string | null
  projected_at: string
  channel_kind: string
  conversation_id: string | null
  sender_display_name: string | null
  delivery_state: string
  workflow_state: WorkflowState
  importance_score: number | null
  ai_category: string | null
  ai_summary: string | null
  ai_summary_generated_at: string | null
  message_metadata: Record<string, unknown>
  local_state: LocalMessageState
  local_state_changed_at: string | null
  local_state_reason: string | null
}

export type CommunicationMessageDetailResponse = {
  message: CommunicationMessageDetailItem
  attachments: CommunicationAttachment[]
}

export type WorkflowStateCountItem = {
  state: string
  count: number
}

export type WorkflowStateCountsResponse = {
  counts: WorkflowStateCountItem[]
}

export type WorkflowStateTransitionRequest = {
  workflow_state: WorkflowState
}

export type LocalMessageStateResponse = {
  message_id: string
  local_state: LocalMessageState
  provider_deleted?: boolean
}

export type MailSyncSettings = {
  account_id: string
  sync_enabled: boolean
  batch_size: number
  poll_interval_seconds: number
  updated_at: string
}

export type MailSyncSettingsUpdate = {
  sync_enabled: boolean
  batch_size: number
  poll_interval_seconds: number
}

export type MailSyncStatus = {
  account_id: string
  status: string
  phase: string
  progress_mode: 'none' | 'determinate' | 'indeterminate' | string
  progress_percent: number | null
  processed_messages: number
  estimated_total_messages: number | null
  current_batch_size: number
  last_started_at: string | null
  last_completed_at: string | null
  next_run_at: string | null
  last_error_code: string | null
  last_error_message: string | null
  last_fetched_messages: number
  last_projected_messages: number
  last_upserted_persons: number
  last_upserted_organizations: number
}

export type MailSyncStatusListResponse = {
  items: MailSyncStatus[]
}

export type MailSyncRunResponse = {
  run_id: string
  account_id: string
  trigger: string
  status: string
  phase: string
  progress_mode: 'none' | 'determinate' | 'indeterminate' | string
  progress_percent: number | null
  processed_messages: number
  estimated_total_messages: number | null
  current_batch_size: number
  fetched_messages: number
  projected_messages: number
  upserted_persons: number
  upserted_organizations: number
  checkpoint_before_present: boolean
  checkpoint_after_present: boolean
  checkpoint_saved: boolean
  failure_reason: { code: string; message: string } | null
  started_at: string
  completed_at: string | null
  next_run_at: string | null
}

export type CommunicationThread = {
  thread_id: string
  account_id: string
  subject: string
  message_count: number
  participant_count: number
  first_message_at: string | null
  last_message_at: string | null
  last_activity_at: string
  has_open_action: boolean
  has_attachments: boolean
  dominant_workflow_state: string
}

export type CommunicationThreadSummary = Pick<
  CommunicationThread,
  | 'thread_id'
  | 'subject'
  | 'message_count'
  | 'participant_count'
  | 'last_activity_at'
  | 'has_open_action'
  | 'has_attachments'
  | 'dominant_workflow_state'
>

export type ThreadMessage = {
  message_id: string
  provider_record_id: string
  account_id: string
  subject: string
  sender: string
  sender_display_name: string | null
  body_text: string
  occurred_at: string | null
  projected_at: string
  workflow_state: string
  importance_score: number | null
  ai_category: string | null
  ai_summary: string | null
  delivery_state: string
  attachment_count: number
  attachments: CommunicationAttachment[]
}

export type ThreadListResponse = {
  items: CommunicationThread[]
  next_cursor: string | null
  has_more: boolean
}
export type ThreadMessagesResponse = { items: ThreadMessage[] }

export type ProviderCall = {
  call_id: string
  account_id: string
  provider_call_id: string
  provider_chat_id: string
  direction: string
  call_state: string
  started_at: string | null
  ended_at: string | null
  transcription_policy_id: string | null
  metadata: Record<string, unknown>
  created_at: string
  updated_at: string
}

export type ProviderCallListResponse = {
  items: ProviderCall[]
}

export type ProviderCallTranscript = {
  transcript_id: string
  call_id: string
  account_id: string
  provider_chat_id: string
  transcript_status: string
  stt_provider: string
  source_audio_ref: string | null
  language_code: string | null
  transcript_text: string
  segments: unknown
  provenance: Record<string, unknown>
  created_at: string
  updated_at: string
}

export type ProviderCallTranscriptResponse = {
  transcript: ProviderCallTranscript | null
}

export type MessageAnalyzeResponse = {
  message_id: string
  analyzed: boolean
  category: string | null
  summary: string | null
  summary_contract: {
    key_points: string[]
    action_items: string[]
    risks: string[]
    deadlines: string[]
    event_candidates: CommunicationKnowledgeCandidate[]
    persona_candidates: CommunicationKnowledgeCandidate[]
    organization_candidates: CommunicationKnowledgeCandidate[]
    document_candidates: CommunicationKnowledgeCandidate[]
    agreement_candidates: CommunicationKnowledgeCandidate[]
  }
  importance_score: number | null
  workflow_state: string
  source: string
  confidence: number | null
  evidence: string[]
}

export type CommunicationKnowledgeCandidate = {
  title: string
  evidence: string
}

export type WorkflowActionKind =
  | 'reply'
  | 'create_task'
  | 'create_note'
  | 'create_document'
  | 'create_event'
  | 'link_document'
  | 'create_contact'
  | 'archive'

export type WorkflowActionSource = {
  kind: 'communication_message'
  id: string
}

export type WorkflowActionRequest = {
  command_id: string
  action: WorkflowActionKind
  source?: WorkflowActionSource
  input?: {
    title?: string
    body?: string
    email?: string
    display_name?: string
    starts_at?: string
    ends_at?: string
    due_at?: string
    document_id?: string
  }
}

export type WorkflowActionResponse = {
  command_id: string
  event_id: string
  action: WorkflowActionKind
  status: 'created' | 'updated' | 'linked' | 'opened' | 'archived' | 'noop'
  target: {
    kind: 'compose' | 'message' | 'task' | 'document' | 'calendar_event' | 'person'
    id: string | null
  }
  provenance: {
    source_kind?: string
    source_id?: string
    confidence: number | null
    evidence: string[]
  }
}

export type MailboxHealth = {
  total_messages: number
  unread: number
  needs_action: number
  waiting: number
  done: number
  archived: number
  spam: number
  important: number
  with_attachments: number
  average_importance: number
  oldest_message_days: number | null
}

export type SenderStats = {
  sender: string
  message_count: number
  avg_importance: number
  last_message_days: number | null
}

export type SenderStatsListResponse = {
  items: SenderStats[]
  next_cursor: string | null
  has_more: boolean
}

export type WorkflowStateTransitionResponse = {
  message_id: string
  workflow_state: string
  previous_state: string
}

export type MessageExplainResponse = {
  reasons: string[]
}

export type SmartCcResponse = {
  suggestions: string[]
}

export type MessagePinToggleResponse = {
  message_id: string
  pinned: boolean
}

export type ConversationPinToggleResponse = {
  conversation_id: string
  provider_chat_id: string
  channel_kind: string
  action: string
  status: string
  command_id: string
  provider: string
  active: boolean
}

export type CommunicationProviderMessageCommandResponse = {
  message_id: string
  raw_record_id: string
  conversation_id: string
  provider_chat_id: string
  provider_message_id: string | null
  channel_kind: string
  status: string
  command_id: string
  provider: string
}

export type MessageImportantToggleResponse = {
  message_id: string
  important: boolean
}

export type MessageExportResponse = {
  content_type: string
  content: string
  filename: string
}
export type MessageExportFormat = 'md' | 'eml' | 'json'

export type MessageAuthResult = {
  result: string
  domain?: string | null
  ip?: string | null
  selector?: string | null
  policy?: string | null
}

export type MessageAuthCheckResponse = {
  auth: {
    spf: MessageAuthResult | null
    dkim: MessageAuthResult | null
    dmarc: MessageAuthResult | null
    raw_headers: string[]
  }
  risk: {
    has_spf: boolean
    spf_pass: boolean
    has_dkim: boolean
    dkim_pass: boolean
    has_dmarc: boolean
    dmarc_pass: boolean
    is_spoofed: boolean
    risk_summary: string
  }
}

export type SignatureDetection = {
  has_signature: boolean
  signature_type: string | null
  signer_info: string | null
  is_valid: boolean | null
  cert_expiry_warning: string | null
}

export type LanguageDetection = {
  language: string
  confidence: number
  script: string | null
}

export type TranslationResponse = {
  translated: boolean
  text?: string
  target?: string
  model?: string
  reason?: string
}

export type AiReplyResponse = {
  subject?: string
  body?: string
  tone?: string
  language?: string
  generated?: boolean
  reason?: string
}

export type AiReplyVariantsRequest = {
  languages?: string[]
  tones?: string[]
}

export type AiReplyVariantsResponse = {
  variants: AiReplyResponse[]
}

export type ExtractedTask = {
  title: string
  due_date: string | null
  assignee: string | null
  priority: string | null
  source: string
}

export type ExtractedNote = {
  title: string
  content: string
  tags: string[]
  source: string
}

export type ExtractTasksResponse = { tasks: ExtractedTask[] }
export type ExtractNotesResponse = { notes: ExtractedNote[] }

export type CommunicationSearchResponse = {
  results: { object_id: string; object_kind: string; title: string }[]
}

export type SubscriptionSource = {
  sender: string
  message_count: number
  first_seen: string
  last_seen: string
  is_newsletter: boole
```
_Source file truncated after 12000 characters. / Исходный файл обрезан после 12000 символов._

### `frontend/src/domains/communications/types/folders.ts`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/domains/communications/types/folders.ts`
- Size bytes / Размер в байтах: `1379`
- Included characters / Включено символов: `1379`
- Truncated / Обрезано: `no`

```typescript
import type { LocalMessageState, WorkflowState } from './communications'

export type CommunicationFolder = {
  folder_id: string
  account_id: string | null
  name: string
  description: string | null
  color: string | null
  sort_order: number
  message_count: number
  created_at: string
  updated_at: string
}

export type CommunicationFolderListResponse = {
  items: CommunicationFolder[]
  next_cursor: string | null
  has_more: boolean
}

export type CommunicationFolderInput = {
  folder_id?: string
  account_id?: string | null
  name: string
  description?: string | null
  color?: string | null
  sort_order?: number
}

export type CommunicationFolderUpdate = Partial<CommunicationFolderInput>

export type FolderDeleteResponse = {
  deleted: boolean
}

export type FolderMessageOperation = 'copy' | 'move'

export type FolderMessageActionResponse = {
  operation: FolderMessageOperation
  folder_id: string
  message_id: string
  message: FolderMessage
}

export type FolderMessage = {
  folder_id: string
  message_id: string
  account_id: string
  subject: string
  sender: string
  occurred_at: string | null
  projected_at: string
  workflow_state: WorkflowState
  local_state: LocalMessageState
  added_at: string
  attachment_count: number
}

export type FolderMessageListResponse = {
  items: FolderMessage[]
  next_cursor: string | null
  has_more: boolean
}
```

### `frontend/src/domains/communications/types/mailOperations.ts`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/domains/communications/types/mailOperations.ts`
- Size bytes / Размер в байтах: `3320`
- Included characters / Включено символов: `3320`
- Truncated / Обрезано: `no`

```typescript
export type CommunicationDraft = {
  draft_id: string
  account_id: string
  persona_id: string | null
  to_recipients: string[]
  cc_recipients: string[]
  bcc_recipients: string[]
  subject: string
  body_text: string
  body_html: string | null
  in_reply_to: string | null
  references: string[]
  status: 'draft' | 'scheduled' | 'sending' | 'sent' | 'failed'
  scheduled_send_at: string | null
  send_attempts: number
  last_error: string | null
  metadata: Record<string, unknown>
  created_at: string
  updated_at: string
}

export type DraftListResponse = {
  items: CommunicationDraft[]
  next_cursor: string | null
  has_more: boolean
}

export type DraftUpsertRequest = {
  draft_id: string
  account_id: string
  persona_id?: string | null
  to_recipients: string[]
  cc_recipients?: string[]
  bcc_recipients?: string[]
  subject: string
  body_text: string
  body_html?: string | null
  in_reply_to?: string | null
  references?: string[]
  status?: 'draft' | 'scheduled' | 'sending' | 'sent' | 'failed'
  scheduled_send_at?: string | null
  metadata?: Record<string, unknown>
}

export type DraftDeleteResponse = { deleted: boolean }

export type SendCommunicationRequest = {
  account_id: string
  to: string[]
  cc?: string[]
  bcc?: string[]
  subject: string
  body_text: string
  body_html?: string | null
  in_reply_to?: string | null
  references?: string[]
  draft_id?: string | null
  scheduled_send_at?: string | null
  undo_send_seconds?: number | null
  confirmed_provider_write: boolean
}

export type SendCommunicationResponse = {
  message_id: string
  outbox_id: string | null
  accepted: string[]
  accepted_recipients: string[]
  transport: 'smtp' | 'local' | 'outbox' | string
  status: 'sent' | 'queued' | 'scheduled' | string
  scheduled_send_at: string | null
  undo_deadline_at: string | null
  failure_reason: string | null
}

export type RedirectMessageRequest = {
  to: string[]
  cc?: string[]
  bcc?: string[]
  confirmed_provider_write?: boolean
}

export type CommunicationOutboxStatus = 'queued' | 'scheduled' | 'sending' | 'sent' | 'failed' | 'canceled'

export type CommunicationOutboxItem = {
  outbox_id: string
  account_id: string
  draft_id: string | null
  to_recipients: string[]
  cc_recipients: string[]
  bcc_recipients: string[]
  subject: string
  body_text: string
  body_html: string | null
  status: CommunicationOutboxStatus
  scheduled_send_at: string | null
  undo_deadline_at: string | null
  send_attempts: number
  claimed_at: string | null
  sent_at: string | null
  provider_message_id: string | null
  last_error: string | null
  metadata: Record<string, unknown>
  created_at: string
  updated_at: string
}

export type OutboxListResponse = {
  items: CommunicationOutboxItem[]
  next_cursor: string | null
  has_more: boolean
}

export type BulkMessageAction =
  | 'mark_read'
  | 'mark_unread'
  | 'archive'
  | 'trash'
  | 'restore'
  | 'pin'
  | 'unpin'
  | 'important'
  | 'not_important'
  | 'add_label'
  | 'remove_label'
  | 'snooze'

export type BulkMessageActionRequest = {
  action: BulkMessageAction
  message_ids: string[]
  label?: string
  snooze_until?: string
}

export type BulkMessageActionResponse = {
  action: BulkMessageAction
  requested_count: number
  matched_count: number
  updated_count: number
  not_found: string[]
}
```

### `frontend/src/domains/communications/types/multilingual.ts`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/domains/communications/types/multilingual.ts`
- Size bytes / Размер в байтах: `361`
- Included characters / Включено символов: `361`
- Truncated / Обрезано: `no`

```typescript
export type ThreadTranslationItem = {
  message_id: string
  original_language: string
  confidence: number
  translated: boolean
  text: string | null
  target: string
  model: string | null
  reason: string | null
}

export type ThreadTranslationResponse = {
  account_id: string
  subject: string
  target_language: string
  items: ThreadTranslationItem[]
}
```

### `frontend/src/domains/communications/types/personas.ts`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/domains/communications/types/personas.ts`
- Size bytes / Размер в байтах: `303`
- Included characters / Включено символов: `303`
- Truncated / Обрезано: `no`

```typescript
export type CommunicationPersona = {
  persona_id: string
  account_id: string
  name: string
  display_name: string
  signature: string
  default_language: string | null
  default_tone: string | null
  is_default: boolean
  metadata: Record<string, unknown>
  created_at: string
  updated_at: string
}
```

### `frontend/src/domains/communications/types/providerChannels.ts`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/domains/communications/types/providerChannels.ts`
- Size bytes / Размер в байтах: `2426`
- Included characters / Включено символов: `2426`
- Truncated / Обрезано: `no`

```typescript
export type CommunicationProviderChannelKind = 'telegram_user' | 'telegram_bot' | 'whatsapp_web' | string

export type CommunicationProviderConversation = {
  telegram_chat_id?: string
  conversation_id?: string
  account_id: string
  provider_chat_id: string
  chat_kind?: string
  title: string
  username?: string | null
  sync_state?: string
  last_message_at: string | null
  metadata: Record<string, unknown>
  created_at: string
  updated_at: string
}

export type CommunicationProviderConversationListResponse = {
  items: CommunicationProviderConversation[]
}

export type CommunicationProviderConversationDetailResponse = {
  item: CommunicationProviderConversation
}

export type CommunicationProviderMessage = {
  message_id: string
  raw_record_id: string
  account_id: string
  provider_record_id?: string
  provider_message_id?: string
  provider_chat_id?: string | null
  conversation_id?: string | null
  chat_title?: string
  sender: string
  sender_display_name: string | null
  text?: string
  body_text_preview?: string
  occurred_at: string | null
  projected_at: string
  channel_kind: CommunicationProviderChannelKind
  delivery_state: string
  metadata?: Record<string, unknown>
  message_metadata?: Record<string, unknown>
}

export type CommunicationProviderMessageListResponse = {
  items: CommunicationProviderMessage[]
  next_cursor?: string | null
  has_more?: boolean
}

export type CommunicationProviderMessageSearchResponse = {
  query: string
  items: CommunicationProviderMessage[]
  total: number
}

export type CommunicationProviderTopic = {
  topic_id: string
  telegram_chat_id: string
  account_id: string
  provider_topic_id: number
  provider_chat_id: string
  title: string
  icon_emoji: string | null
  is_pinned: boolean
  is_closed: boolean
  unread_count: number
  last_message_at: string | null
  metadata: Record<string, unknown>
  created_at: string
  updated_at: string
}

export type CommunicationProviderTopicListResponse = {
  telegram_chat_id: string
  items: CommunicationProviderTopic[]
}

export type CommunicationRawEvidenceResponse = {
  raw_record: {
    raw_record_id: string
    provider_kind: string
    provider_account_id: string
    provider_message_id: string
    source_uri: string | null
    occurred_at: string
    ingested_at: string
    payload: Record<string, unknown>
    headers: Record<string, string>
    provenance: Record<string, unknown>
  }
}
```

### `frontend/src/domains/communications/types/readReceipts.ts`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/domains/communications/types/readReceipts.ts`
- Size bytes / Размер в байтах: `695`
- Included characters / Включено символов: `695`
- Truncated / Обрезано: `no`

```typescript
export type CommunicationReadReceiptKind = 'read'

export type CommunicationReadReceipt = {
  receipt_id: string
  account_id: string
  outbox_id: string | null
  provider_message_id: string
  recipient: string
  receipt_kind: CommunicationReadReceiptKind
  read_at: string
  source_kind: string
  provider_record_id: string | null
  raw_record_id: string | null
  metadata: Record<string, unknown>
  created_at: string
}

export type NewCommunicationReadReceipt = {
  receipt_id?: string
  account_id: string
  provider_message_id: string
  recipient: string
  read_at: string
  source_kind?: string
  provider_record_id?: string
  raw_record_id?: string
  metadata?: Record<string, unknown>
}
```

### `frontend/src/domains/communications/types/savedSearches.ts`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/domains/communications/types/savedSearches.ts`
- Size bytes / Размер в байтах: `980`
- Included characters / Включено символов: `980`
- Truncated / Обрезано: `no`

```typescript
import type { LocalMessageState, WorkflowState } from './communications'

export type CommunicationSavedSearch = {
  saved_search_id: string
  name: string
  description: string | null
  account_id: string | null
  query: string
  workflow_state: WorkflowState | null
  local_state: LocalMessageState
  channel_kind: string | null
  is_smart_folder: boolean
  sort_order: number
  message_count: number
  created_at: string
  updated_at: string
}

export type SavedSearchListResponse = {
  items: CommunicationSavedSearch[]
  next_cursor: string | null
  has_more: boolean
}

export type SavedSearchInput = {
  name: string
  description?: string | null
  account_id?: string | null
  query?: string
  workflow_state?: WorkflowState | null
  local_state?: LocalMessageState
  channel_kind?: string | null
  is_smart_folder?: boolean
  sort_order?: number
}

export type SavedSearchUpdate = Partial<SavedSearchInput>

export type SavedSearchDeleteResponse = {
  deleted: boolean
}
```

### `frontend/src/domains/communications/types/templates.ts`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/domains/communications/types/templates.ts`
- Size bytes / Размер в байтах: `1629`
- Included characters / Включено символов: `1629`
- Truncated / Обрезано: `no`

```typescript
export type CommunicationTemplate = {
  template_id: string
  name: string
  subject_template: string
  body_template: string
  variables: string[]
  placeholder_variables: string[]
  undeclared_variables: string[]
  unused_variables: string[]
  malformed_placeholders: string[]
  language: string | null
  created_at: string
  updated_at: string
}

export type RichTemplateRenderRequest = {
  template_id: string
  variables: Record<string, string>
}

export type RichTemplateUpsertRequest = {
  template_id?: string
  name: string
  subject_template: string
  body_template: string
  variables: string[]
  language: string | null
}

export type RichTemplateUpsertResponse = {
  saved: boolean
  template: CommunicationTemplate
}

export type RichTemplateDeleteResponse = {
  template_id: string
  deleted: boolean
}

export type RichTemplateRenderResponse = {
  template_id: string
  variables: Record<string, string>
  rendered: {
    subject: string
    body: string
    missing_variables: string[]
    unresolved_variables: string[]
    malformed_placeholders: string[]
  }
}

export type RichTemplateMailMergePreviewRow = {
  row_id: string
  variables: Record<string, string>
}

export type RichTemplateMailMergePreviewRequest = {
  template_id: string
  rows: RichTemplateMailMergePreviewRow[]
}

export type RichTemplateMailMergePreviewItem = {
  row_id: string
  ready: boolean
  rendered: RichTemplateRenderResponse['rendered']
}

export type RichTemplateMailMergePreviewResponse = {
  template_id: string
  row_count: number
  ready_count: number
  blocked_count: number
  items: RichTemplateMailMergePreviewItem[]
}
```

### `frontend/src/domains/communications/views/CommunicationsPage.boundary.test.ts`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/domains/communications/views/CommunicationsPage.boundary.test.ts`
- Size bytes / Размер в байтах: `13212`
- Included characters / Включено символов: `12000`
- Truncated / Обрезано: `yes`

```typescript
import { describe, expect, it } from 'vitest'
import { readFileSync } from 'node:fs'

function scriptSetupSource(source: string): string {
	const match = source.match(/<script setup lang="ts">([\s\S]*?)<\/script>/)
	return match?.[1] ?? ''
}

describe('CommunicationsPage folder management integration', () => {
	it('keeps page orchestration in a view-level controller instead of the Vue component', () => {
		const source = readFileSync(
			new URL('./CommunicationsPage.vue', import.meta.url),
			'utf8'
		)
		const scriptSource = scriptSetupSource(source)
		const controllerSource = readFileSync(
			new URL('./useCommunicationsPageController.ts', import.meta.url),
			'utf8'
		)
		const resourceOverviewSource = readFileSync(
			new URL('./useMailResourceOverview.ts', import.meta.url),
			'utf8'
		)

		expect(source).toContain('useCommunicationsPageController')
		expect(source).toContain("../../../shared/mailSetup/AccountSetupModal.vue")
		expect(source).toContain("import CommunicationsCallsPanel from '../components/CommunicationsCallsPanel.vue'")
		expect(source).toContain('@generate-ai-reply="handleGenerateAiReply"')
		expect(source).toContain('@apply-ai-reply="handleApplyAiReply"')
		expect(source).toContain('@review-security="handleReviewSecurity"')
		expect(source).toContain('@review-recipients="handleReviewRecipients"')
		expect(source).toContain('@reply-all="handleReplyAll"')
		expect(source).toContain('@forward-message="handleForwardMessage"')
		expect(source).toContain('@redirect-message="handleRedirectMessage"')
		expect(source).toContain('@mark-message-read="handleMarkMessageRead"')
		expect(source).toContain('@mark-message-unread="handleMarkMessageUnread"')
		expect(source).toContain('@delete-from-provider="handleDeleteFromProvider"')
		expect(scriptSource).toContain('handleGenerateAiReply,')
		expect(scriptSource).toContain('handleApplyAiReply,')
		expect(scriptSource).toContain('handleReviewSecurity,')
		expect(scriptSource).toContain('handleReviewRecipients,')
		expect(scriptSource).toContain('handleReplyAll,')
		expect(scriptSource).toContain('handleForwardMessage,')
		expect(scriptSource).toContain('handleRedirectMessage,')
		expect(scriptSource).toContain('handleMarkMessageRead,')
		expect(scriptSource).toContain('handleMarkMessageUnread,')
		expect(scriptSource).toContain('handleDeleteFromProvider,')
		expect(source).not.toContain('useMailListQuery')
		expect(source).not.toContain('useBulkMessageActionMutation')
		expect(source).not.toContain('watch(')
		expect(source).not.toContain('onMounted')
		expect(controllerSource).toContain('useMailListQuery')
		expect(controllerSource).toContain('useFolderMailList')
		expect(controllerSource).toContain('useThreadReplyActions')
		expect(controllerSource).toContain('useMailSyncActions')
		expect(controllerSource).toContain('useSelectedMessageActions')
		expect(controllerSource).toContain('handleBulkAction')
		expect(controllerSource).toContain('useMailResourceOverview')
		expect(resourceOverviewSource).toContain('useSubscriptionsQuery')
		expect(resourceOverviewSource).toContain('useTopSendersQuery')
		expect(resourceOverviewSource).toContain('useCommunicationBlockersQuery')
		expect(resourceOverviewSource).toContain('handleLoadMoreSubscriptions')
		expect(resourceOverviewSource).toContain('handleLoadMoreTopSenders')
		expect(controllerSource).toContain('handleGenerateAiReply')
		expect(controllerSource).toContain('handleApplyAiReply')
		expect(controllerSource).toContain('handleReviewSecurity')
		expect(controllerSource).toContain('handleReviewRecipients')
		expect(controllerSource).toContain('handleReplyAll')
		expect(controllerSource).toContain('handleForwardMessage')
		expect(controllerSource).toContain('handleRedirectMessage')
		expect(controllerSource).toContain('handleMarkMessageRead')
		expect(controllerSource).toContain('handleMarkMessageUnread')
		expect(controllerSource).toContain('handleDeleteFromProvider')
		expect(controllerSource).not.toContain("from '../components/")
		expect(controllerSource).not.toContain('fetch(')
		expect(controllerSource).not.toContain('ApiClient')
	})

	it('routes provider-neutral calls and meetings sections into dedicated communication surfaces', () => {
		const source = readFileSync(
			new URL('./CommunicationsPage.vue', import.meta.url),
			'utf8'
		)

		expect(source).toContain("v-else-if=\"nav.activeCommunicationSection === 'calls'\"")
		expect(source).toContain("v-else-if=\"nav.activeCommunicationSection === 'meetings'\"")
		expect(source).toContain('<CommunicationsCallsPanel')
		expect(source).toContain('mode="calls"')
		expect(source).toContain('mode="meetings"')
	})

	it('keeps selected-message side-effect orchestration in a focused controller helper', () => {
		const source = readFileSync(
			new URL('./useSelectedMessageActions.ts', import.meta.url),
			'utf8'
		)
		const controllerSource = readFileSync(
			new URL('./useCommunicationsPageController.ts', import.meta.url),
			'utf8'
		)

		expect(source).toContain('useGenerateAiReplyMutation')
		expect(source).toContain('useReviewMessageSecurityMutation')
		expect(source).toContain('useReviewMessageRecipientsMutation')
		expect(source).toContain('useRedirectMessageMutation')
		expect(source).toContain('runSelectedMessageAction')
		expect(source).toContain('handleGenerateAiReply')
		expect(source).toContain('handleApplyAiReply')
		expect(source).toContain('handleReviewSecurity')
		expect(source).toContain('handleReviewRecipients')
		expect(source).toContain('handleRedirectMessage')
		expect(source).toContain('handleExportMessage')
		expect(source).toContain('handleAddLabel')
		expect(source).toContain('handleSnoozeMessage')
		expect(source).not.toContain("from '../components/")
		expect(source).not.toContain('fetch(')
		expect(source).not.toContain('ApiClient')
		expect(controllerSource).not.toContain('useGenerateAiReplyMutation')
		expect(controllerSource).not.toContain('useReviewMessageSecurityMutation')
		expect(controllerSource).not.toContain('useReviewMessageRecipientsMutation')
		expect(controllerSource).not.toContain('useRedirectMessageMutation')
	})

	it('keeps thread reply send/draft orchestration in a focused controller helper', () => {
		const source = readFileSync(
			new URL('./useThreadReplyActions.ts', import.meta.url),
			'utf8'
		)

		expect(source).toContain('useSaveDraftMutation')
		expect(source).toContain('useSendMailMutation')
		expect(source).toContain('buildComposeDraftPayload')
		expect(source).toContain('composeFormToSendRequest')
		expect(source).toContain('threadReplyComposeForm')
		expect(source).toContain('handleReplyToThreadMessage')
		expect(source).toContain('handleSaveThreadReplyDraft')
		expect(source).toContain('handleSendThreadReply')
		expect(source).not.toContain("from '../components/")
		expect(source).not.toContain('fetch(')
		expect(source).not.toContain('ApiClient')
	})

	it('keeps mail sync side-effect orchestration in a focused controller helper', () => {
		const source = readFileSync(
			new URL('./useMailSyncActions.ts', import.meta.url),
			'utf8'
		)

		expect(source).toContain('useRunMailSyncNowMutation')
		expect(source).toContain('handleSyncNow')
		expect(source).toContain('useMailSyncSettingsQuery')
		expect(source).toContain('useUpdateMailSyncSettingsMutation')
		expect(source).toContain('handleUpdateSyncSettings')
		expect(source).toContain('clearSyncStatus')
		expect(source).toContain('loadInitialData')
		expect(source).not.toContain("from '../components/")
		expect(source).not.toContain('fetch(')
		expect(source).not.toContain('ApiClient')
	})

	it('renders the custom folder management strip with the selected account context', () => {
		const source = readFileSync(
			new URL('./CommunicationsPage.vue', import.meta.url),
			'utf8'
		)
		const controllerSource = readFileSync(
			new URL('./useCommunicationsPageController.ts', import.meta.url),
			'utf8'
		)

		expect(source).toContain('CommunicationFolderStrip')
		expect(source).toContain('savedSearchChannelKind')
		expect(source).toContain('AttachmentSearchPanel')
		expect(source).toContain(':account-id="store.selectedMailAccountId || null"')
		expect(source).toContain('activeFolderId')
		expect(source).toContain(':active-id="activeFolderId"')
		expect(source).toContain('@select="handleFolderSelect"')
		expect(source).toContain(':current-channel-kind="savedSearchChannelKind || \'email\'"')
		expect(source).toContain(':is-folder-mode="Boolean(activeFolderId)"')
		expect(source).toContain(':messages="visibleMailList"')
		expect(source).toContain(':account-id="store.selectedMailAccountId"')
		expect(controllerSource).toContain('const savedSearchChannelKind = ref<string>()')
		expect(controllerSource).toContain('savedSearchChannelKind,')
	})

	it('routes mail list keyboard selection commands into the communications store', () => {
		const source = readFileSync(
			new URL('./CommunicationsPage.vue', import.meta.url),
			'utf8'
		)

		expect(source).toContain('@select-visible="store.selectVisibleMessages"')
		expect(source).toContain('@clear-selection="store.clearMessageSelection"')
	})

	it('wires server-backed thread pagination into the navigator list', () => {
		const source = readFileSync(
			new URL('./CommunicationsPage.vue', import.meta.url),
			'utf8'
		)

		expect(source).toContain('threads="store.threads"')
		expect(source).toContain(':has-thread-next-page="hasThreadNextPage"')
		expect(source).toContain(':is-fetching-thread-next-page="isFetchingThreadNextPage"')
		expect(source).toContain(':selected-thread-id="store.selectedThreadId"')
		expect(source).toContain('@select-thread="handleSelectThread"')
		expect(source).toContain('@load-more-threads="handleLoadMoreThreads"')
	})

	it('loads selected thread messages into the detail conversation view', () => {
		const source = readFileSync(
			new URL('./CommunicationsPage.vue', import.meta.url),
			'utf8'
		)
		const controllerSource = readFileSync(
			new URL('./useCommunicationsPageController.ts', import.meta.url),
			'utf8'
		)

		expect(controllerSource).toContain('useThreadMessagesQuery')
		expect(source).toContain('selectedThreadMessages')
		expect(source).toContain(':selected-thread="store.selectedThread"')
		expect(source).toContain(':thread-messages="selectedThreadMessages"')
		expect(source).toContain('@open-thread-message="handleOpenThreadMessage"')
		expect(source).toContain('@reply-to-thread-message="handleReplyToThreadMessage"')
		expect(source).toContain('@save-thread-reply-draft="handleSaveThreadReplyDraft"')
		expect(source).toContain('@send-thread-reply="handleSendThreadReply"')
		expect(source).toContain(':is-thread-reply-sending="isThreadReplySending"')
	})

	it('surfaces outbox delivery status through query-backed strip wiring', () => {
		const source = readFileSync(
			new URL('./CommunicationsPage.vue', import.meta.url),
			'utf8'
		)
		const controllerSource = readFileSync(
			new URL('./useCommunicationsPageController.ts', import.meta.url),
			'utf8'
		)

		expect(source).toContain('OutboxStatusStrip')
		expect(controllerSource).toContain('useOutboxStatusStrip')
		expect(source).toContain(':items="outboxItems"')
		expect(source).toContain(':error-message="outboxErrorMessage"')
		expect(source).toContain(':has-more="hasMoreOutboxItems"')
		expect(source).toContain('@load-more="loadMoreOutboxItems"')
		expect(source).toContain('@prefetch-more="prefetchMoreOutboxItems"')
		expect(source).toContain('@undo="undoOutbox"')
	})

  it('routes bilingual reply send actions from message detail into compose', () => {
    const source = readFileSync(
      new URL('./CommunicationsPage.vue', import.meta.url),
      'utf8'
    )

    expect(source).toContain('@send-bilingual-reply="handleBilingualReplySend"')
    expect(source).toContain('handleBilingualReplySend')
  })

	it('wires format-aware message export into the detail pane and action bar', () => {
		const source = readFileSync(
			new URL('./CommunicationsPage.vue', import.meta.url),
			'utf8'
```
_Source file truncated after 12000 characters. / Исходный файл обрезан после 12000 символов._

### `frontend/src/domains/communications/views/useCommunicationsPageController.ts`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/domains/communications/views/useCommunicationsPageController.ts`
- Size bytes / Размер в байтах: `15218`
- Included characters / Включено символов: `12000`
- Truncated / Обрезано: `yes`

```typescript
import { computed, onMounted, ref, watch } from 'vue'
import {
  useBulkMessageActionMutation,
  useConversationsQuery,
  useDeleteDraftMutation,
  useDraftsQuery,
  useMailboxHealthQuery,
  useMailListQuery,
  useMessageQuery,
  useStateCountsQuery,
  useSyncStatusesQuery,
  useThreadMessagesQuery
} from '../queries/useCommunicationsQuery'
import { useFolderMailList } from '../queries/folderMailList'
import { useOutboxStatusStrip } from '../queries/outboxStatusStrip'
import { useMailResourceOverview } from './useMailResourceOverview'
import { draftToComposeForm } from '../helpers/communicationPageModels'
import {
  communicationSectionWorkflowState,
  communicationWorkflowStateSectionId,
  useCommunicationsStore
} from '../stores/communications'
import type {
  BulkMessageActionRequest,
  CommunicationSectionId,
  CommunicationDraft,
  CommunicationThreadSummary
} from '../types/communications'
import type { CommunicationSavedSearch } from '../types/savedSearches'
import { useMailSyncActions } from './useMailSyncActions'
import { useThreadReplyActions } from './useThreadReplyActions'
import { useSelectedMessageActions } from './useSelectedMessageActions'

type BulkActionCommand = Omit<BulkMessageActionRequest, 'message_ids'>

export function useCommunicationsPageController() {
  const store = useCommunicationsStore()
  const isAccountSetupOpen = ref(false)
  const inspectorVisible = ref(true)
  const activeSavedSearchId = ref('')
  const activeFolderId = ref('')
  const savedSearchChannelKind = ref<string>()

  const {
    data: mailListData,
    error: mailListError,
    isLoading: isMailListLoading,
    isFetchingNextPage,
    hasNextPage,
    fetchNextPage,
    refetch: refetchMailList
  } = useMailListQuery(
    () => store.selectedMailAccountId || undefined,
    () => store.mailStateFilter || undefined,
    () => savedSearchChannelKind.value,
    () => store.messageSearchQuery || undefined,
    () => store.mailLocalStateFilter
  )
  const {
    data: messageDetailData,
    refetch: refetchMessageDetail
  } = useMessageQuery(() => store.selectedCommunicationMessageId || null)
  const {
    data: stateCountsData,
    refetch: refetchStateCounts
  } = useStateCountsQuery(() => store.selectedMailAccountId || undefined, () => store.mailLocalStateFilter)
  const {
    data: syncStatusesData,
    refetch: refetchSyncStatuses
  } = useSyncStatusesQuery()
  const {
    data: draftsData,
    refetch: refetchDrafts,
    hasNextPage: hasDraftNextPage,
    isFetchingNextPage: isFetchingDraftNextPage,
    fetchNextPage: fetchNextDraftPage
  } = useDraftsQuery(() => store.selectedMailAccountId || undefined)
  const {
    data: mailboxHealthData,
    refetch: refetchMailboxHealth
  } = useMailboxHealthQuery(() => store.selectedMailAccountId || undefined)
  const resourceOverview = useMailResourceOverview(() => store.selectedMailAccountId || undefined)
  const {
    data: conversationsData,
    isLoading: isThreadListLoading,
    hasNextPage: hasThreadNextPage,
    isFetchingNextPage: isFetchingThreadNextPage,
    fetchNextPage: fetchNextThreadPage
  } = useConversationsQuery(() => store.selectedMailAccountId || undefined)
  const {
    data: threadMessagesData,
    isLoading: isSelectedThreadLoading,
    error: selectedThreadMessagesError
  } = useThreadMessagesQuery(
    () => store.selectedMailAccountId || null,
    () => store.selectedThread?.subject ?? null
  )
  const deleteDraftMutation = useDeleteDraftMutation()
  const bulkMessageActionMutation = useBulkMessageActionMutation()
  const {
    outboxItems,
    outboxErrorMessage,
    isOutboxLoading,
    isLoadingMoreOutbox,
    hasMoreOutboxItems,
    isUndoingOutbox,
    undoOutbox,
    loadMoreOutboxItems,
    prefetchMoreOutboxItems
  } = useOutboxStatusStrip(() => store.selectedMailAccountId || undefined, {
    onStatus: (message) => store.setMailActionStatus(message),
    onError: (message) => store.setMailActionError(message)
  })

  const mailList = computed(() => mailListData.value ?? [])
  const messageDetail = computed(() => messageDetailData.value ?? null)
  const stateCounts = computed(() => stateCountsData.value ?? [])
  const drafts = computed(() => draftsData.value ?? [])
  const hasMoreDrafts = computed(() => Boolean(hasDraftNextPage.value))
  const isLoadingMoreDrafts = computed(() => isFetchingDraftNextPage.value)
  const mailboxHealth = computed(() => mailboxHealthData.value ?? null)
  const {
    areResourcesLoading,
    blockers,
    handleLoadMoreSubscriptions,
    handleLoadMoreTopSenders,
    hasMoreSubscriptions,
    hasMoreTopSenders,
    isLoadingMoreSubscriptions,
    isLoadingMoreTopSenders,
    subscriptions,
    topSenders
  } = resourceOverview
  const selectedThreadMessages = computed(() => threadMessagesData.value?.items ?? [])
  const selectedThreadErrorMessage = computed(() => {
    if (!selectedThreadMessagesError.value) return ''
    return selectedThreadMessagesError.value instanceof Error
      ? selectedThreadMessagesError.value.message
      : 'Failed to load conversation'
  })
  const hasRail = computed(() => inspectorVisible.value && messageDetail.value !== null)
  const selectedBulkCount = computed(() => store.selectedMessageIds.length)
  const isBulkActionRunning = computed(() => bulkMessageActionMutation.isPending.value)
  const mailListErrorMessage = computed(() => {
    if (!mailListError.value) return ''
    return mailListError.value instanceof Error ? mailListError.value.message : 'Failed to load messages'
  })
  const folderMail = useFolderMailList(() => activeFolderId.value)
  const visibleMailList = computed(() => activeFolderId.value ? folderMail.messages.value : mailList.value)
  const visibleMailListErrorMessage = computed(() => activeFolderId.value ? folderMail.errorMessage.value : mailListErrorMessage.value)
  const isVisibleMailListLoading = computed(() => activeFolderId.value ? folderMail.isLoading.value : isMailListLoading.value)
  const isNavigatorListLoading = computed(() =>
    !activeFolderId.value && store.communicationsNavigatorMode === 'threads'
      ? isThreadListLoading.value
      : isVisibleMailListLoading.value
  )
  const hasVisibleNextPage = computed(() => activeFolderId.value ? Boolean(folderMail.hasNextPage.value) : Boolean(hasNextPage.value))
  const isFetchingVisibleNextPage = computed(() => activeFolderId.value ? folderMail.isFetchingNextPage.value : isFetchingNextPage.value)
  const activeSectionId = computed<CommunicationSectionId>(() =>
    communicationWorkflowStateSectionId(store.mailStateFilter)
  )

  watch(visibleMailList, (items) => store.setMessages(items))
  watch(messageDetailData, (detail) => store.setMessageDetail(detail ?? null))
  watch(stateCountsData, (counts) => store.setStateCounts(counts ?? []))
  watch(syncStatusesData, (statuses) => store.setMailSyncStatuses(statuses ?? []))
  watch(draftsData, (items) => store.setDrafts(items ?? []))
  watch(mailboxHealthData, (health) => store.setMailboxHealth(health ?? null))
  watch(conversationsData, (threads) => {
    store.setThreads((threads ?? []).map((thread) => ({
      thread_id: thread.thread_id,
      subject: thread.subject,
      message_count: thread.message_count,
      participant_count: thread.participant_count,
      last_activity_at: thread.last_activity_at,
      has_open_action: thread.has_open_action,
      has_attachments: thread.has_attachments,
      dominant_workflow_state: thread.dominant_workflow_state
    })))
  })

  function resetSelectedMessageContext() {
    store.selectMessage(-1)
    store.clearSelectedThread()
    store.setMessageDetail(null)
    store.setCommunicationMessageInsight(null)
  }

  function handleSearchQueryUpdate(query: string) {
    activeSavedSearchId.value = ''
    activeFolderId.value = ''
    store.setMessageSearchQuery(query)
    resetSelectedMessageContext()
  }

  function handleLoadMoreMessages() {
    if (activeFolderId.value) {
      if (folderMail.hasNextPage.value && !folderMail.isFetchingNextPage.value) void folderMail.fetchNextPage()
      return
    }
    if (!hasNextPage.value || isFetchingNextPage.value) return
    void fetchNextPage()
  }

  function handleLoadMoreThreads() {
    if (!hasThreadNextPage.value || isFetchingThreadNextPage.value) return
    void fetchNextThreadPage()
  }

  function handleLoadMoreDrafts() {
    if (!hasDraftNextPage.value || isFetchingDraftNextPage.value) return
    void fetchNextDraftPage()
  }

  function handleSelectMessage(index: number) {
    store.selectMessage(index)
    store.setActiveMessageContextTab('message')
    store.setCommunicationMessageInsight(null)
  }

  function handleSelectThread(thread: CommunicationThreadSummary) {
    store.selectThread(thread)
    store.setActiveMessageContextTab('message')
  }

  function handleOpenThreadMessage(messageId: string) {
    store.selectMessageId(messageId)
    store.setActiveMessageContextTab('message')
    store.setCommunicationMessageInsight(null)
  }

  const {
    handleReplyToThreadMessage,
    handleSaveThreadReplyDraft,
    handleSendThreadReply,
    isThreadReplySending
  } = useThreadReplyActions(store)

  async function handleBulkAction(command: BulkActionCommand) {
    const messageIds = [...store.selectedMessageIds]
    if (messageIds.length === 0) return
    store.setIsMailActionRunning(true)
    try {
      const result = await bulkMessageActionMutation.mutateAsync({
        ...command,
        message_ids: messageIds
      })
      store.setMailActionStatus(`${result.updated_count} messages updated`)
      store.clearMessageSelection()
      await Promise.all([refetchMailList(), refetchStateCounts()])
    } catch (e) {
      store.setMailActionError(e instanceof Error ? e.message : 'Bulk action failed')
    } finally {
      store.setIsMailActionRunning(false)
    }
  }

  const {
    handleAddLabel,
    handleDeleteFromProvider,
    handleAnalyze,
    handleApplyAiReply,
    handleBilingualReplySend,
    handleCreateNote,
    handleCreateTask,
    handleExportMessage,
    handleMarkMessageRead,
    handleMarkMessageUnread,
    handleForwardMessage,
    handleGenerateAiReply,
    handleMute,
    handleNewMessage,
    handleRedirectMessage,
    handleRemoveLabel,
    handleReply,
    handleReplyAll,
    handleReviewRecipients,
    handleReviewSecurity,
    handleSnoozeMessage,
    handleToggleImportant,
    handleTogglePin,
    handleTranslate
  } = useSelectedMessageActions(store, {
    getMessageDetail: () => messageDetail.value?.message ?? null,
    refetchMessageDetail
  })

  function selectSection(sectionId: CommunicationSectionId) {
    const workflowState = communicationSectionWorkflowState(sectionId)
    if (workflowState === null) return
    activeSavedSearchId.value = ''
    activeFolderId.value = ''
    savedSearchChannelKind.value = undefined
    store.setStateFilter(workflowState)
    store.setLocalStateFilter('active')
    resetSelectedMessageContext()
  }

  function handleSavedSearchSelect(savedSearch: CommunicationSavedSearch) {
    activeSavedSearchId.value = savedSearch.saved_search_id
    activeFolderId.value = ''
    savedSearchChannelKind.value = savedSearch.channel_kind ?? undefined
    store.setMessageSearchQuery(savedSearch.query)
    store.setStateFilter(savedSearch.workflow_state ?? '')
    store.setLocalStateFilter(savedSearch.local_state)
    resetSelectedMessageContext()
  }

  function handleSavedSearchDeleted(savedSearch: CommunicationSavedSearch) {
    if (activeSavedSearchId.value !== savedSearch.saved_search_id) return
    activeSavedSearchId.value = ''
    savedSearchChannelKind.value = undefined
    store.setMessageSearchQuery('')
    store.setStateFilter('')
    store.setLocalStateFilter('active')
    resetSelectedMessageContext()
  }

  function handleFolderSelect(folderId: string) {
    activeFolderId.value = activeFolderId.value === folderId ? '' : folderId
    activeSavedSearchId.value = ''
    resetSelectedMessageContext()
  }

  function handleFolderDeleted() {
    active
```
_Source file truncated after 12000 characters. / Исходный файл обрезан после 12000 символов._

### `frontend/src/domains/communications/views/useMailResourceOverview.ts`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/domains/communications/views/useMailResourceOverview.ts`
- Size bytes / Размер в байтах: `2183`
- Included characters / Включено символов: `2183`
- Truncated / Обрезано: `no`

```typescript
import { computed } from 'vue'
import {
  useCommunicationBlockersQuery,
  useSubscriptionsQuery,
  useTopSendersQuery
} from '../queries/mailWorkspaceQueries'
import type { QueryParam } from '../queries/queryTypes'

export function useMailResourceOverview(accountId?: QueryParam<string>) {
  const {
    data: subscriptionsData,
    isLoading: isSubscriptionsLoading,
    hasNextPage: hasSubscriptionsNextPage,
    isFetchingNextPage: isFetchingSubscriptionsNextPage,
    fetchNextPage: fetchNextSubscriptionsPage
  } = useSubscriptionsQuery(accountId)
  const {
    data: topSendersData,
    isLoading: isTopSendersLoading,
    hasNextPage: hasTopSendersNextPage,
    isFetchingNextPage: isFetchingTopSendersNextPage,
    fetchNextPage: fetchNextTopSendersPage
  } = useTopSendersQuery(accountId)
  const {
    data: blockersData,
    isLoading: isBlockersLoading
  } = useCommunicationBlockersQuery()

  const subscriptions = computed(() => subscriptionsData.value ?? [])
  const topSenders = computed(() => topSendersData.value ?? [])
  const blockers = computed(() => blockersData.value ?? [])
  const hasMoreSubscriptions = computed(() => Boolean(hasSubscriptionsNextPage.value))
  const hasMoreTopSenders = computed(() => Boolean(hasTopSendersNextPage.value))
  const isLoadingMoreSubscriptions = computed(() => isFetchingSubscriptionsNextPage.value)
  const isLoadingMoreTopSenders = computed(() => isFetchingTopSendersNextPage.value)
  const areResourcesLoading = computed(() =>
    isSubscriptionsLoading.value || isTopSendersLoading.value || isBlockersLoading.value
  )

  function handleLoadMoreSubscriptions() {
    if (!hasSubscriptionsNextPage.value || isFetchingSubscriptionsNextPage.value) return
    void fetchNextSubscriptionsPage()
  }

  function handleLoadMoreTopSenders() {
    if (!hasTopSendersNextPage.value || isFetchingTopSendersNextPage.value) return
    void fetchNextTopSendersPage()
  }

  return {
    areResourcesLoading,
    blockers,
    handleLoadMoreSubscriptions,
    handleLoadMoreTopSenders,
    hasMoreSubscriptions,
    hasMoreTopSenders,
    isLoadingMoreSubscriptions,
    isLoadingMoreTopSenders,
    subscriptions,
    topSenders
  }
}
```

### `frontend/src/domains/communications/views/useMailSyncActions.ts`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/domains/communications/views/useMailSyncActions.ts`
- Size bytes / Размер в байтах: `2739`
- Included characters / Включено символов: `2739`
- Truncated / Обрезано: `no`

```typescript
import {
  useMailSyncSettingsQuery,
  useRunMailSyncNowMutation,
  useUpdateMailSyncSettingsMutation
} from '../../../shared/mailSync/runtimeQueries'
import type { useCommunicationsStore } from '../stores/communications'
import type { MailSyncSettingsUpdate } from '../types/communications'

type CommunicationsStore = ReturnType<typeof useCommunicationsStore>
type RefetchHandler = () => Promise<unknown>

type MailSyncRefetches = {
  refetchMailList: RefetchHandler
  refetchMailboxHealth: RefetchHandler
  refetchStateCounts: RefetchHandler
  refetchSyncStatuses: RefetchHandler
}

export function useMailSyncActions(store: CommunicationsStore, refetches: MailSyncRefetches) {
  const runMailSyncNowMutation = useRunMailSyncNowMutation()
  const updateMailSyncSettingsMutation = useUpdateMailSyncSettingsMutation()
  const {
    data: selectedMailSyncSettings,
    isLoading: isSyncSettingsLoading
  } = useMailSyncSettingsQuery(() => store.selectedMailAccountId || null)

  async function handleSyncNow() {
    const accountId = store.selectedMailAccountId
    if (!accountId) return
    store.setIsMailSyncBusy(true)
    store.setMailSyncStatusMessage('Syncing...')
    try {
      await runMailSyncNowMutation.mutateAsync(accountId)
      store.setMailSyncStatusMessage('Sync completed')
      await Promise.all([
        refetches.refetchMailList(),
        refetches.refetchStateCounts(),
        refetches.refetchSyncStatuses(),
        refetches.refetchMailboxHealth()
      ])
    } catch (e) {
      store.setMailSyncError(e instanceof Error ? e.message : 'Sync failed')
    } finally {
      store.setIsMailSyncBusy(false)
    }
  }

  function clearSyncStatus() {
    store.setMailSyncStatusMessage('')
    store.setMailSyncError('')
  }

  async function handleUpdateSyncSettings(settings: MailSyncSettingsUpdate) {
    const accountId = store.selectedMailAccountId
    if (!accountId) return
    store.setMailSyncStatusMessage('Saving sync settings...')
    store.setMailSyncError('')
    try {
      await updateMailSyncSettingsMutation.mutateAsync({ accountId, settings })
      store.setMailSyncStatusMessage('Sync settings saved')
      await refetches.refetchSyncStatuses()
    } catch (e) {
      store.setMailSyncError(e instanceof Error ? e.message : 'Sync settings update failed')
    }
  }

  async function loadInitialData() {
    await Promise.all([
      refetches.refetchSyncStatuses(),
      refetches.refetchMailboxHealth(),
      refetches.refetchStateCounts()
    ])
  }

  return {
    clearSyncStatus,
    handleUpdateSyncSettings,
    handleSyncNow,
    isSyncSettingsLoading,
    isSyncSettingsSaving: updateMailSyncSettingsMutation.isPending,
    selectedMailSyncSettings,
    loadInitialData
  }
}
```

### `frontend/src/domains/communications/views/useSelectedMessageActions.ts`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/domains/communications/views/useSelectedMessageActions.ts`
- Size bytes / Размер в байтах: `11746`
- Included characters / Включено символов: `11746`
- Truncated / Обрезано: `no`

```typescript
import {
  useAddMessageLabelMutation,
  useAnalyzeMessageMutation,
  useExportMessageMutation,
  useDeleteMessageFromProviderMutation,
  useMarkMessageReadMutation,
  useMarkMessageUnreadMutation,
  useExtractMessageNotesMutation,
  useExtractMessageTasksMutation,
  useGenerateAiReplyMutation,
  useRedirectMessageMutation,
  useRemoveMessageLabelMutation,
  useReviewMessageRecipientsMutation,
  useReviewMessageSecurityMutation,
  useSnoozeMessageMutation,
  useToggleMessageImportantMutation,
  useToggleMessageMuteMutation,
  useToggleMessagePinMutation,
  useTranslateMessageMutation
} from '../queries/useCommunicationsQuery'
import { splitComposeRecipients } from '../forms/composeValidation'
import {
  emptyCommunicationMessageInsight,
  forwardComposeForm,
  newComposeForm,
  replyAllComposeForm,
  replyComposeForm
} from '../helpers/communicationPageModels'
import type { useCommunicationsStore } from '../stores/communications'
import type {
  AiReplyResponse,
  CommunicationMessageDetailItem,
  MessageExportFormat
} from '../types/communications'
import type { BilingualReplyFlowResponse } from '../types/bilingualReplyFlow'

type CommunicationsStore = ReturnType<typeof useCommunicationsStore>
type RefetchHandler = () => Promise<unknown>

type SelectedMessageActionOptions = {
  getMessageDetail: () => CommunicationMessageDetailItem | null
  refetchMessageDetail: RefetchHandler
}

export function useSelectedMessageActions(
  store: CommunicationsStore,
  deps: SelectedMessageActionOptions
) {
  const togglePinMutation = useToggleMessagePinMutation()
  const toggleImportantMutation = useToggleMessageImportantMutation()
  const toggleMuteMutation = useToggleMessageMuteMutation()
  const exportMessageMutation = useExportMessageMutation()
  const markMessageReadMutation = useMarkMessageReadMutation()
  const markMessageUnreadMutation = useMarkMessageUnreadMutation()
  const deleteMessageFromProviderMutation = useDeleteMessageFromProviderMutation()
  const generateAiReplyMutation = useGenerateAiReplyMutation()
  const reviewSecurityMutation = useReviewMessageSecurityMutation()
  const reviewRecipientsMutation = useReviewMessageRecipientsMutation()
  const redirectMessageMutation = useRedirectMessageMutation()
  const addLabelMutation = useAddMessageLabelMutation()
  const removeLabelMutation = useRemoveMessageLabelMutation()
  const snoozeMessageMutation = useSnoozeMessageMutation()
  const analyzeMessageMutation = useAnalyzeMessageMutation()
  const translateMessageMutation = useTranslateMessageMutation()
  const extractMessageTasksMutation = useExtractMessageTasksMutation()
  const extractMessageNotesMutation = useExtractMessageNotesMutation()

  function handleReply() {
    if (!store.selectedCommunication) return
    store.openCompose(replyComposeForm(store.selectedCommunication, store.selectedMailAccountId, `draft-${Date.now()}`))
  }

  function handleReplyAll() {
    if (!store.selectedCommunication) return
    store.openCompose(replyAllComposeForm(store.selectedCommunication, store.selectedMailAccountId, `draft-${Date.now()}`))
  }

  function handleForwardMessage() {
    if (!store.selectedCommunication) return
    store.openCompose(forwardComposeForm(store.selectedCommunication, store.selectedMailAccountId, `draft-${Date.now()}`))
  }

  async function handleRedirectMessage(recipientsText: string) {
    await runSelectedMessageAction(async (messageId) => {
      const to = splitComposeRecipients(recipientsText)
      if (to.length === 0) {
        throw new Error('Redirect recipient is required')
      }
      const result = await redirectMessageMutation.mutateAsync({
        messageId,
        request: { to, confirmed_provider_write: true }
      })
      return result.status === 'sent' ? 'Redirected' : `Redirect ${result.status}`
    })
  }

  function handleBilingualReplySend(response: BilingualReplyFlowResponse): void {
    const detail = deps.getMessageDetail()
    if (!detail || !response.send_ready) return
    store.openCompose({
      mode: 'reply',
      draftId: `draft-${Date.now()}`,
      accountId: detail.account_id || store.selectedMailAccountId || '',
      toText: detail.sender,
      ccText: '',
      bccText: '',
      subject: response.subject,
      body: response.reply.text,
      bodyHtml: null,
      bodyFormat: 'plain',
      scheduledSendAt: '',
      undoSendSeconds: null,
      inReplyTo: detail.provider_record_id || null
    })
  }

  function handleNewMessage() {
    store.openCompose(newComposeForm(store.selectedMailAccountId || '', `draft-${Date.now()}`))
  }

  async function runSelectedMessageAction(action: (messageId: string) => Promise<string>) {
    const messageId = store.selectedCommunicationMessageId
    if (!messageId) return
    store.setIsMailActionRunning(true)
    store.setLastMessageExport(null)
    try {
      store.setMailActionStatus(await action(messageId))
    } catch (e) {
      store.setMailActionError(e instanceof Error ? e.message : 'Message action failed')
    } finally {
      store.setIsMailActionRunning(false)
    }
  }

  async function handleTogglePin() {
    await runSelectedMessageAction(async (messageId) => {
      const result = await togglePinMutation.mutateAsync(messageId)
      return result.pinned ? 'Pinned' : 'Unpinned'
    })
  }

  async function handleToggleImportant() {
    await runSelectedMessageAction(async (messageId) => {
      const result = await toggleImportantMutation.mutateAsync(messageId)
      return result.important ? 'Marked important' : 'Unmarked important'
    })
  }

  async function handleMute() {
    await runSelectedMessageAction(async (messageId) => {
      await toggleMuteMutation.mutateAsync(messageId)
      return 'Muted'
    })
  }

  async function handleExportMessage(format: MessageExportFormat) {
    await runSelectedMessageAction(async (messageId) => {
      const exported = await exportMessageMutation.mutateAsync({ messageId, format })
      store.setLastMessageExport(exported)
      return `Exported ${exported.filename}`
    })
  }

  async function handleMarkMessageRead() {
    await runSelectedMessageAction(async (messageId) => {
      await markMessageReadMutation.mutateAsync(messageId)
      await deps.refetchMessageDetail()
      return 'Marked as read'
    })
  }

  async function handleMarkMessageUnread() {
    await runSelectedMessageAction(async (messageId) => {
      await markMessageUnreadMutation.mutateAsync(messageId)
      await deps.refetchMessageDetail()
      return 'Marked as unread'
    })
  }

  async function handleDeleteFromProvider() {
    await runSelectedMessageAction(async (messageId) => {
      await deleteMessageFromProviderMutation.mutateAsync(messageId)
      await deps.refetchMessageDetail()
      return 'Deleted in provider mode'
    })
  }

  async function handleAddLabel(label: string) {
    await runSelectedMessageAction(async (messageId) => {
      await addLabelMutation.mutateAsync({ messageId, label })
      await deps.refetchMessageDetail()
      return `Added label ${label}`
    })
  }

  async function handleRemoveLabel(label: string) {
    await runSelectedMessageAction(async (messageId) => {
      await removeLabelMutation.mutateAsync({ messageId, label })
      await deps.refetchMessageDetail()
      return `Removed label ${label}`
    })
  }

  async function handleSnoozeMessage(until: string) {
    await runSelectedMessageAction(async (messageId) => {
      await snoozeMessageMutation.mutateAsync({ messageId, until })
      await deps.refetchMessageDetail()
      return 'Snoozed'
    })
  }

  async function handleAnalyze() {
    await runSelectedMessageAction(async (messageId) => {
      await analyzeMessageMutation.mutateAsync(messageId)
      await deps.refetchMessageDetail()
      return 'Analyzed'
    })
  }

  async function handleTranslate() {
    await runSelectedMessageAction(async (messageId) => {
      const result = await translateMessageMutation.mutateAsync({ messageId, targetLanguage: 'en' })
      store.setCommunicationMessageInsight({
        ...(store.mailMessageInsight ?? emptyCommunicationMessageInsight(messageId)),
        translation: result
      })
      return 'Translated'
    })
  }

  async function handleGenerateAiReply(replyOptions: { tone: string; language: string }) {
    await runSelectedMessageAction(async (messageId) => {
      const result = await generateAiReplyMutation.mutateAsync({
        messageId,
        tone: replyOptions.tone,
        language: replyOptions.language
      })
      store.setCommunicationMessageInsight({
        ...(store.mailMessageInsight ?? emptyCommunicationMessageInsight(messageId)),
        aiReply: result
      })
      return result.generated === false ? result.reason || 'AI reply not generated' : 'AI reply generated'
    })
  }

  function handleApplyAiReply(response: AiReplyResponse) {
    const detail = deps.getMessageDetail()
    if (!detail || !response.body) return
    store.openCompose({
      mode: 'reply',
      draftId: `draft-${Date.now()}`,
      accountId: detail.account_id || store.selectedMailAccountId || '',
      toText: detail.sender,
      ccText: '',
      bccText: '',
      subject: response.subject || (detail.subject.startsWith('Re:') ? detail.subject : `Re: ${detail.subject}`),
      body: response.body,
      bodyHtml: null,
      bodyFormat: 'plain',
      scheduledSendAt: '',
      undoSendSeconds: null,
      inReplyTo: detail.provider_record_id || null
    })
  }

  async function handleReviewSecurity() {
    await runSelectedMessageAction(async (messageId) => {
      const result = await reviewSecurityMutation.mutateAsync(messageId)
      store.setCommunicationMessageInsight({
        ...(store.mailMessageInsight ?? emptyCommunicationMessageInsight(messageId)),
        auth: result.auth,
        signature: result.signature
      })
      return result.auth.risk.risk_summary
    })
  }

  async function handleReviewRecipients() {
    await runSelectedMessageAction(async (messageId) => {
      const result = await reviewRecipientsMutation.mutateAsync(messageId)
      store.setCommunicationMessageInsight({
        ...(store.mailMessageInsight ?? emptyCommunicationMessageInsight(messageId)),
        smartCc: result
      })
      return `${result.suggestions.length} recipient suggestions`
    })
  }

  async function handleCreateTask() {
    await runSelectedMessageAction(async (messageId) => {
      const result = await extractMessageTasksMutation.mutateAsync(messageId)
      store.setCommunicationMessageInsight({
        ...(store.mailMessageInsight ?? emptyCommunicationMessageInsight(messageId)),
        tasks: result.tasks
      })
      return `Extracted ${result.tasks.length} tasks`
    })
  }

  async function handleCreateNote() {
    await runSelectedMessageAction(async (messageId) => {
      const result = await extractMessageNotesMutation.mutateAsync(messageId)
      store.setCommunicationMessageInsight({
        ...(store.mailMessageInsight ?? emptyCommunicationMessageInsight(messageId)),
        notes: result.notes
      })
      return `Extracted ${result.notes.length} notes`
    })
  }

  return {
    handleAddLabel,
    handleDeleteFromProvider,
    handleAnalyze,
    handleApplyAiReply,
    handleBilingualReplySend,
    handleCreateNote,
    handleCreateTask,
    handleExportMessage,
    handleMarkMessageRead,
    handleMarkMessageUnread,
    handleForwardMessage,
    handleGenerateAiReply,
    handleMute,
    handleNewMessage,
    handleRedirectMessage,
    handleRemoveLabel,
    handleReply,
    handleReplyAll,
    handleReviewRecipients,
    handleReviewSecurity,
    handleSnoozeMessage,
    handleToggleImportant,
    handleTogglePin,
    handleTranslate
  }
}
```
