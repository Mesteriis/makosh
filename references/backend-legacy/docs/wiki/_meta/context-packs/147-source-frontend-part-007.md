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

- Chunk ID / ID чанка: `147-source-frontend-part-007`
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

### `frontend/src/domains/communications/queries/mailOperationQueries.ts`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/domains/communications/queries/mailOperationQueries.ts`
- Size bytes / Размер в байтах: `17721`
- Included characters / Включено символов: `12000`
- Truncated / Обрезано: `yes`

```typescript
import { useInfiniteQuery, useMutation, useQueryClient, type InfiniteData } from '@tanstack/vue-query'
import { computed, toValue } from 'vue'
import {
  bulkMessageAction,
  createDraft,
  deleteDraft,
  fetchDrafts,
  fetchOutboxItems,
  redirectMessage,
  sendEmail,
  undoOutboxItem
} from '../api/communications'
import { updateMessageAiState } from '../api/aiState'
import { prepareBilingualReplyFlow } from '../api/bilingualReplyFlow'
import type { BilingualReplyFlowRequest, BilingualReplyFlowResponse } from '../types/bilingualReplyFlow'
import type {
  BulkMessageActionRequest,
  BulkMessageActionResponse,
  DraftListResponse,
  CommunicationDraft,
  CommunicationOutboxItem,
  CommunicationOutboxStatus,
  CommunicationMessageDetailResponse,
  CommunicationMessagesResponse,
  OutboxListResponse,
  RedirectMessageRequest,
  SendCommunicationRequest,
  SendCommunicationResponse
} from '../types/communications'
import type {
  CommunicationAiStateRecord,
  CommunicationAiStateTransitionRequest
} from '../types/aiState'
import {
  applyBulkMessageActionToMailDetail,
  applyBulkMessageActionToMailList,
  markOutboxItemCanceled,
  removeDraftFromDraftList,
  upsertDraftInDraftList
} from './optimisticMailUpdates'
import { communicationMessageQueryKey } from './communicationPrefetch'
import { communicationRealtimeQueryOptions } from './communicationQueryPolicies'
import type { QueryParam } from './queryTypes'
import type { ComposeDraftPayload } from '../forms/composeDraftAutosave'

type BulkMessageActionMutationContext = {
  previousMailLists: Array<[readonly unknown[], InfiniteData<CommunicationMessagesResponse> | undefined]>
  previousMessages: Array<[
    readonly ['communications-message', string],
    CommunicationMessageDetailResponse | null | undefined
  ]>
}

type DraftMutationContext = {
  previousDraftLists: Array<[readonly unknown[], InfiniteData<DraftListResponse> | undefined]>
}

type OutboxMutationContext = {
  previousOutboxLists: Array<[readonly unknown[], InfiniteData<OutboxListResponse> | undefined]>
}

export function useDraftsQuery(accountId?: QueryParam<string>) {
  return useInfiniteQuery<DraftListResponse, Error, CommunicationDraft[], readonly unknown[], string | null>({
    queryKey: computed(() => ['communications-drafts', toValue(accountId)]),
    initialPageParam: null,
    queryFn: async ({ pageParam }) => fetchDrafts(toValue(accountId), undefined, 50, pageParam),
    getNextPageParam: (lastPage) => lastPage.next_cursor ?? undefined,
    select: (data) => data.pages.flatMap((page) => page.items),
    ...communicationRealtimeQueryOptions
  })
}

export function useOutboxQuery(accountId?: QueryParam<string>, status?: QueryParam<CommunicationOutboxStatus>) {
  return useInfiniteQuery<OutboxListResponse, Error, CommunicationOutboxItem[], readonly unknown[], string | null>({
    queryKey: computed(() => ['communications-outbox', toValue(accountId), toValue(status)]),
    initialPageParam: null,
    queryFn: async ({ pageParam }) => fetchOutboxItems(toValue(accountId), toValue(status), 100, pageParam),
    getNextPageParam: (lastPage) => lastPage.next_cursor ?? undefined,
    select: (data) => data.pages.flatMap((page) => page.items),
    ...communicationRealtimeQueryOptions
  })
}

export function useSendMailMutation() {
  const queryClient = useQueryClient()
  return useMutation<SendCommunicationResponse, Error, SendCommunicationRequest, DraftMutationContext>({
    mutationFn: async (request: SendCommunicationRequest) => {
      return sendEmail(request)
    },
    onMutate: async (request) => {
      if (!request.draft_id) return { previousDraftLists: [] }

      await queryClient.cancelQueries({ queryKey: ['communications-drafts'] })
      const previousDraftLists = queryClient.getQueriesData<InfiniteData<DraftListResponse>>({
        queryKey: ['communications-drafts']
      })

      for (const [queryKey, data] of previousDraftLists) {
        queryClient.setQueryData(queryKey, removeDraftFromDraftPages(data, request.draft_id))
      }

      return { previousDraftLists }
    },
    onError: (_error, _request, context) => {
      restoreDraftLists(queryClient, context)
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['communications-list'] })
      queryClient.invalidateQueries({ queryKey: ['communications-drafts'] })
      queryClient.invalidateQueries({ queryKey: ['communications-outbox'] })
    }
  })
}

export function useSaveDraftMutation() {
  const queryClient = useQueryClient()
  return useMutation<CommunicationDraft, Error, ComposeDraftPayload, DraftMutationContext>({
    mutationFn: async (draft: ComposeDraftPayload) => {
      return createDraft(draft)
    },
    onMutate: async (draft) => {
      await queryClient.cancelQueries({ queryKey: ['communications-drafts'] })
      const previousDraftLists = queryClient.getQueriesData<InfiniteData<DraftListResponse>>({
        queryKey: ['communications-drafts']
      })
      const optimisticDraft = optimisticDraftFromPayload(draft, previousDraftLists)

      for (const [queryKey, data] of previousDraftLists) {
        if (!draftQueryMatchesAccount(queryKey, optimisticDraft.account_id)) continue
        queryClient.setQueryData(queryKey, upsertDraftInDraftPages(data, optimisticDraft))
      }

      return { previousDraftLists }
    },
    onError: (_error, _draft, context) => {
      restoreDraftLists(queryClient, context)
    },
    onSuccess: (draft) => {
      for (const [queryKey, data] of queryClient.getQueriesData<InfiniteData<DraftListResponse>>({
        queryKey: ['communications-drafts']
      })) {
        if (!draftQueryMatchesAccount(queryKey, draft.account_id)) continue
        queryClient.setQueryData(queryKey, upsertDraftInDraftPages(data, draft))
      }
      queryClient.invalidateQueries({ queryKey: ['communications-drafts'] })
    },
    onSettled: () => {
      queryClient.invalidateQueries({ queryKey: ['communications-drafts'] })
    }
  })
}

export function useDeleteDraftMutation() {
  const queryClient = useQueryClient()
  return useMutation({
    mutationFn: async (draftId: string) => {
      return deleteDraft(draftId)
    },
    onMutate: async (draftId) => {
      await queryClient.cancelQueries({ queryKey: ['communications-drafts'] })
      const previousDraftLists = queryClient.getQueriesData<InfiniteData<DraftListResponse>>({
        queryKey: ['communications-drafts']
      })

      for (const [queryKey, data] of previousDraftLists) {
        queryClient.setQueryData(queryKey, removeDraftFromDraftPages(data, draftId))
      }

      return { previousDraftLists }
    },
    onError: (_error, _draftId, context) => {
      restoreDraftLists(queryClient, context)
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['communications-drafts'] })
    }
  })
}

export function useUndoOutboxMutation() {
  const queryClient = useQueryClient()
  return useMutation<CommunicationOutboxItem, Error, string, OutboxMutationContext>({
    mutationFn: async (outboxId: string) => {
      return undoOutboxItem(outboxId)
    },
    onMutate: async (outboxId) => {
      await queryClient.cancelQueries({ queryKey: ['communications-outbox'] })
      const previousOutboxLists = queryClient.getQueriesData<InfiniteData<OutboxListResponse>>({
        queryKey: ['communications-outbox']
      })

      for (const [queryKey, data] of previousOutboxLists) {
        queryClient.setQueryData(queryKey, markOutboxItemCanceledInOutboxPage(data, queryKey, outboxId))
      }

      return { previousOutboxLists }
    },
    onError: (_error, _outboxId, context) => {
      restoreOutboxLists(queryClient, context)
    },
    onSuccess: (item) => {
      for (const [queryKey, data] of queryClient.getQueriesData<InfiniteData<OutboxListResponse>>({
        queryKey: ['communications-outbox']
      })) {
        queryClient.setQueryData(queryKey, upsertOutboxItemInOutboxPage(data, queryKey, item))
      }
      queryClient.invalidateQueries({ queryKey: ['communications-outbox'] })
    }
  })
}

export function usePrepareBilingualReplyFlowMutation() {
  return useMutation<
    BilingualReplyFlowResponse,
    Error,
    { messageId: string; request: BilingualReplyFlowRequest }
  >({
    mutationFn: async ({ messageId, request }) => {
      return prepareBilingualReplyFlow(messageId, request)
    }
  })
}

export function useRedirectMessageMutation() {
  const queryClient = useQueryClient()
  return useMutation<SendCommunicationResponse, Error, { messageId: string; request: RedirectMessageRequest }>({
    mutationFn: async ({ messageId, request }) => redirectMessage(messageId, request),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['communications-outbox'] })
    }
  })
}

function restoreDraftLists(
  queryClient: ReturnType<typeof useQueryClient>,
  context: DraftMutationContext | undefined
): void {
  if (!context) return
  for (const [queryKey, data] of context.previousDraftLists) {
    queryClient.setQueryData(queryKey, data)
  }
}

function restoreOutboxLists(
  queryClient: ReturnType<typeof useQueryClient>,
  context: OutboxMutationContext | undefined
): void {
  if (!context) return
  for (const [queryKey, data] of context.previousOutboxLists) {
    queryClient.setQueryData(queryKey, data)
  }
}

function upsertOutboxItemInOutboxPage(
  data: InfiniteData<OutboxListResponse> | undefined,
  queryKey: readonly unknown[],
  item: CommunicationOutboxItem
): InfiniteData<OutboxListResponse> | undefined {
  if (!data) return data
  if (!outboxQueryMatches(queryKey, item)) {
    return removeOutboxItemFromOutboxPage(data, item.outbox_id)
  }

  const existsInAnyPage = data.pages.some((page) =>
    page.items.some((existing) => existing.outbox_id === item.outbox_id)
  )
  let changed = false
  const pages = data.pages.map((page, pageIndex) => {
    const existingIndex = page.items.findIndex((existing) => existing.outbox_id === item.outbox_id)

    if (existingIndex >= 0) {
      const items = page.items.slice()
      items[existingIndex] = item
      changed = true
      return { ...page, items }
    }

    if (!existsInAnyPage && pageIndex === 0) {
      changed = true
      return { ...page, items: [item, ...page.items] }
    }
    return page
  })

  return changed ? { ...data, pages } : data
}

function markOutboxItemCanceledInOutboxPage(
  data: InfiniteData<OutboxListResponse> | undefined,
  queryKey: readonly unknown[],
  outboxId: string
): InfiniteData<OutboxListResponse> | undefined {
  if (!data) return data
  const queryStatus = queryKey[2]
  if (typeof queryStatus === 'string' && queryStatus !== 'canceled') {
    return removeOutboxItemFromOutboxPage(data, outboxId)
  }

  let changed = false
  const pages = data.pages.map((page) => {
    const items = markOutboxItemCanceled(page.items, outboxId) ?? page.items
    if (items === page.items) return page
    changed = true
    return { ...page, items }
  })

  return changed ? { ...data, pages } : data
}

function removeOutboxItemFromOutboxPage(
  data: InfiniteData<OutboxListResponse>,
  outboxId: string
): InfiniteData<OutboxListResponse> {
  let changed = false
  const pages = data.pages.map((page) => {
    const items = page.items.filter((item) => item.outbox_id !== outboxId)
    if (items.length === page.items.length) return page
    changed = true
    return { ...page, items }
  })

  return changed ? { ...data, pages } : data
}

function optimisticDraftFromPayload(
  draft: ComposeDraftPayload,
  draftLists: Array<[readonly unknown[], InfiniteData<DraftListResponse> | undefined]>
): CommunicationDraft {
  const existing = findCachedDraft(draftLists, draft.draft_id)
  const now = new Date().toISOString()

  return {
    draft_id: draft.draft_id,
    account_id: draft.account_id,
    persona_id: existing?.persona_id ?? null,
    to_recipients: draft.to_recipients,
    cc_recipients: draft.cc_recipients,
    bcc_recipients: draft.bcc_recipients,
    subject: dr
```
_Source file truncated after 12000 characters. / Исходный файл обрезан после 12000 символов._

### `frontend/src/domains/communications/queries/mailWorkspaceQueries.ts`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/domains/communications/queries/mailWorkspaceQueries.ts`
- Size bytes / Размер в байтах: `21295`
- Included characters / Включено символов: `12000`
- Truncated / Обрезано: `yes`

```typescript
import { useInfiniteQuery, useMutation, useQuery, useQueryClient, type InfiniteData } from '@tanstack/vue-query'
import { computed, toValue, type MaybeRefOrGetter } from 'vue'
import {
  copyMessageToFolder,
  createMailCertificate,
  createCommunicationFolder,
  createSavedSearch,
  deleteCommunicationFolder,
  deleteRichTemplate,
  deleteSavedSearch,
  fetchFolderMessages,
  fetchCommunicationBlockers,
  fetchExpiringMailCertificates,
  fetchMailCertificates,
  fetchCommunicationFolders,
  fetchRichTemplates,
  fetchSavedSearches,
  fetchSubscriptions,
  fetchTopSenders,
  inspectAttachmentArchive,
  moveMessageToFolder,
  previewAttachment,
  previewRichTemplateMailMerge,
  renderRichTemplate,
  saveRichTemplate,
  searchAttachments,
  translateAttachment,
  updateCommunicationFolder,
  updateSavedSearch
} from '../api/communications'
import type {
  CommunicationTemplate,
  CommunicationArchitectureBlocker,
  RichTemplateDeleteResponse,
  RichTemplateMailMergePreviewRequest,
  RichTemplateMailMergePreviewResponse,
  RichTemplateRenderRequest,
  RichTemplateRenderResponse,
  RichTemplateUpsertRequest,
  RichTemplateUpsertResponse,
  SenderStats,
  SenderStatsListResponse,
  SubscriptionListResponse,
  SubscriptionSource
} from '../types/communications'
import type {
  MailCertificate,
  MailCertificateCreateRequest
} from '../types/certificates'
import type {
  AttachmentArchiveInspectionResponse,
  AttachmentPreviewResponse,
  AttachmentSearchRequest,
  AttachmentSearchResponse,
  AttachmentSearchResult,
  AttachmentTranslationRequest,
  AttachmentTranslationResponse
} from '../types/attachments'
import type {
  FolderDeleteResponse,
  FolderMessage,
  FolderMessageActionResponse,
  FolderMessageListResponse,
  CommunicationFolder,
  CommunicationFolderInput,
  CommunicationFolderListResponse,
  CommunicationFolderUpdate
} from '../types/folders'
import type {
  CommunicationSavedSearch,
  SavedSearchDeleteResponse,
  SavedSearchInput,
  SavedSearchListResponse,
  SavedSearchUpdate
} from '../types/savedSearches'
import type { NullableQueryParam, QueryParam } from './queryTypes'
import {
  communicationDetailQueryOptions,
  communicationRealtimeQueryOptions,
  communicationReferenceQueryOptions
} from './communicationQueryPolicies'
import {
  optimisticFolderFromUpdate,
  removeFolderFromFolderList,
  upsertFolderInFolderList
} from './optimisticFolderUpdates'
import {
  findCachedFolderMessage,
  optimisticFolderMessageForTarget,
  removeFolderMessageFromFolderList,
  upsertFolderMessageInFolderList,
  type FolderMessageListCache
} from './optimisticFolderMessageUpdates'

type FolderMutationContext = {
  previousFolderLists: Array<[readonly unknown[], InfiniteData<CommunicationFolderListResponse> | undefined]>
}

type FolderMessageMutationContext = {
  previousFolderMessageLists: FolderMessageListCache
}

export function useRichTemplatesQuery() {
  return useQuery<CommunicationTemplate[]>({
    queryKey: ['communications-rich-templates'],
    queryFn: async () => {
      const res = await fetchRichTemplates()
      return res.templates
    },
    ...communicationReferenceQueryOptions
  })
}

export function useSubscriptionsQuery(accountId?: QueryParam<string>) {
  return useInfiniteQuery<SubscriptionListResponse, Error, SubscriptionSource[], readonly unknown[], string | null>({
    queryKey: computed(() => ['communications-subscriptions', toValue(accountId)]),
    initialPageParam: null,
    queryFn: async ({ pageParam }) => fetchSubscriptions(toValue(accountId), 25, pageParam),
    getNextPageParam: (lastPage) => lastPage.next_cursor ?? undefined,
    select: (data) => data.pages.flatMap((page) => page.items),
    ...communicationRealtimeQueryOptions
  })
}

export function useTopSendersQuery(accountId?: QueryParam<string>) {
  return useInfiniteQuery<SenderStatsListResponse, Error, SenderStats[], readonly unknown[], string | null>({
    queryKey: computed(() => ['communications-top-senders', toValue(accountId)]),
    initialPageParam: null,
    queryFn: async ({ pageParam }) => fetchTopSenders(toValue(accountId), 25, pageParam),
    getNextPageParam: (lastPage) => lastPage.next_cursor ?? undefined,
    select: (data) => data.pages.flatMap((page) => page.items),
    ...communicationRealtimeQueryOptions
  })
}

export function useCommunicationBlockersQuery() {
  return useQuery<CommunicationArchitectureBlocker[]>({
    queryKey: ['communications-mail-blockers'],
    queryFn: async () => fetchCommunicationBlockers(),
    ...communicationReferenceQueryOptions
  })
}

export function useMailCertificatesQuery() {
  return useQuery<MailCertificate[]>({
    queryKey: ['communications-certificates'],
    queryFn: async () => {
      const res = await fetchMailCertificates()
      return res.items
    },
    ...communicationReferenceQueryOptions
  })
}

export function useExpiringMailCertificatesQuery(days: QueryParam<number> = 90) {
  return useQuery<MailCertificate[]>({
    queryKey: computed(() => ['communications-certificates-expiring', toValue(days)]),
    queryFn: async () => {
      const res = await fetchExpiringMailCertificates(toValue(days))
      return res.items
    },
    ...communicationReferenceQueryOptions
  })
}

export function useCreateMailCertificateMutation() {
  const queryClient = useQueryClient()
  return useMutation<MailCertificate, Error, MailCertificateCreateRequest>({
    mutationFn: async (request) => createMailCertificate(request),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['communications-certificates'] })
      queryClient.invalidateQueries({ queryKey: ['communications-certificates-expiring'] })
    }
  })
}

export function useSavedSearchesQuery(isSmartFolder?: QueryParam<boolean>, accountId?: QueryParam<string>) {
  return useInfiniteQuery<SavedSearchListResponse, Error, CommunicationSavedSearch[], readonly unknown[], string | null>({
    queryKey: computed(() => ['communications-saved-searches', toValue(isSmartFolder), toValue(accountId)]),
    initialPageParam: null,
    queryFn: async ({ pageParam }) => {
      return fetchSavedSearches(toValue(isSmartFolder), toValue(accountId), 100, pageParam)
    },
    getNextPageParam: (lastPage) => lastPage.next_cursor ?? undefined,
    select: (data) => data.pages.flatMap((page) => page.items),
    ...communicationRealtimeQueryOptions
  })
}

export function useCommunicationFoldersQuery(accountId?: QueryParam<string>) {
  return useInfiniteQuery<CommunicationFolderListResponse, Error, CommunicationFolder[], readonly unknown[], string | null>({
    queryKey: computed(() => ['communications-folders', toValue(accountId)]),
    initialPageParam: null,
    queryFn: async ({ pageParam }) => {
      return fetchCommunicationFolders(toValue(accountId), 500, pageParam)
    },
    getNextPageParam: (lastPage) => lastPage.next_cursor ?? undefined,
    select: (data) => data.pages.flatMap((page) => page.items),
    ...communicationRealtimeQueryOptions
  })
}

export function useFolderMessagesQuery(
  folderId: NullableQueryParam<string>,
  enabled: QueryParam<boolean> = true
) {
  return useInfiniteQuery<FolderMessageListResponse, Error, FolderMessage[], readonly unknown[], string | null>({
    queryKey: computed(() => ['communications-folder-messages', toValue(folderId)]),
    initialPageParam: null,
    enabled: computed(() => !!toValue(folderId) && toValue(enabled)),
    queryFn: async ({ pageParam }) => {
      const id = toValue(folderId)
      if (!id) {
        return { items: [], next_cursor: null, has_more: false }
      }
      return fetchFolderMessages(id, 250, pageParam)
    },
    getNextPageParam: (lastPage) => lastPage.next_cursor ?? undefined,
    select: (data) => data.pages.flatMap((page) => page.items),
    ...communicationRealtimeQueryOptions
  })
}

export function useAttachmentSearchQuery(
  request: MaybeRefOrGetter<AttachmentSearchRequest>,
  enabled: QueryParam<boolean> = true
) {
  return useInfiniteQuery<AttachmentSearchResponse, Error, AttachmentSearchResult[], readonly unknown[], string | null>({
    queryKey: computed(() => {
      const value = toValue(request)
      return [
        'communications-attachment-search',
        value.account_id,
        value.q,
        value.content_type,
        value.scan_status,
        value.limit
      ]
    }),
    initialPageParam: null,
    enabled: computed(() => toValue(enabled)),
    queryFn: async ({ pageParam }) => {
      return searchAttachments({ ...toValue(request), cursor: pageParam })
    },
    getNextPageParam: (lastPage) => lastPage.next_cursor ?? undefined,
    select: (data) => data.pages.flatMap((page) => page.items),
    ...communicationDetailQueryOptions
  })
}

export function useAttachmentArchiveInspectionQuery(
  attachmentId: NullableQueryParam<string>,
  enabled: QueryParam<boolean> = true
) {
  return useQuery<AttachmentArchiveInspectionResponse | null>({
    queryKey: computed(() => ['communications-attachment-archive-inspection', toValue(attachmentId)]),
    queryFn: async () => {
      const id = toValue(attachmentId)
      if (!id) return null
      return inspectAttachmentArchive(id)
    },
    enabled: computed(() => Boolean(toValue(attachmentId)) && toValue(enabled)),
    ...communicationDetailQueryOptions
  })
}

export function useAttachmentPreviewQuery(
  attachmentId: NullableQueryParam<string>,
  enabled: QueryParam<boolean> = true
) {
  return useQuery<AttachmentPreviewResponse | null>({
    queryKey: computed(() => ['communications-attachment-preview', toValue(attachmentId)]),
    queryFn: async () => {
      const id = toValue(attachmentId)
      if (!id) return null
      return previewAttachment(id)
    },
    enabled: computed(() => Boolean(toValue(attachmentId)) && toValue(enabled)),
    ...communicationDetailQueryOptions
  })
}

export function useTranslateAttachmentMutation() {
  return useMutation<
    AttachmentTranslationResponse,
    Error,
    { attachmentId: string; request: AttachmentTranslationRequest }
  >({
    mutationFn: async ({ attachmentId, request }) => translateAttachment(attachmentId, request)
  })
}

export function useCreateRichTemplateMutation() {
  const queryClient = useQueryClient()
  return useMutation<RichTemplateUpsertResponse, Error, RichTemplateUpsertRequest>({
    mutationFn: async (request) => saveRichTemplate(request),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['communications-rich-templates'] })
    }
  })
}

export function useDeleteRichTemplateMutation() {
  const queryClient = useQueryClient()
  return useMutation<RichTemplateDeleteResponse, Error, string>({
    mutationFn: async (templateId) => deleteRichTemplate(templateId),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['communications-rich-templates'] })
    }
  })
}

export function useRenderRichTemplateMutation() {
  return useMutation<RichTemplateRenderResponse, Error, RichTemplateRenderRequest>({
    mutationFn: async (request) => renderRichTemplate(request)
  })
}

export function usePreviewRichTemplateMailMergeMutation() {
  return useMutation<
    RichTemplateMailMergePreviewResponse,
    Error,
    RichTemplateMailMergePreviewRequest
  >({
    mutationFn: async (request) => previewRichTemplateMailMerge(request)
  })
}

export function useCreateSavedSearchMutation() {
  const queryClient = useQueryClient()
  return useMutation<CommunicationSavedSearch, Error, SavedSearchInput>({
    mutationFn: async (request: SavedSearchInput) => createSavedSearch(request),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['communications-saved-searches'] })
    }
  })
}

export function useUpdateSavedSearchMutation() {
  const queryClient = useQueryClient()
  return useMutation<
    CommunicationSavedSearch,
    Error,
    { savedSearchId: string; request: SavedSearchUpdate }
  >({
    mutationFn: async ({ savedSearchId, request }) => updateSavedSearch(savedSearchId, request),
    onSuccess: () => {

```
_Source file truncated after 12000 characters. / Исходный файл обрезан после 12000 символов._

### `frontend/src/domains/communications/queries/messageLocalIntelligence.boundary.test.ts`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/domains/communications/queries/messageLocalIntelligence.boundary.test.ts`
- Size bytes / Размер в байтах: `554`
- Included characters / Включено символов: `554`
- Truncated / Обрезано: `no`

```typescript
import { describe, expect, it } from 'vitest'
import { readFileSync } from 'node:fs'

describe('message local intelligence query boundary', () => {
  it('wraps explain and language APIs through TanStack mutations', () => {
    const source = readFileSync(new URL('./mailActionQueries.ts', import.meta.url), 'utf8')

    expect(source).toContain('useExplainMessageMutation')
    expect(source).toContain('useDetectMessageLanguageMutation')
    expect(source).toContain('fetchMessageExplain')
    expect(source).toContain('detectMessageLanguage')
  })

})
```

### `frontend/src/domains/communications/queries/optimisticFolderMessageUpdates.test.ts`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/domains/communications/queries/optimisticFolderMessageUpdates.test.ts`
- Size bytes / Размер в байтах: `3149`
- Included characters / Включено символов: `3149`
- Truncated / Обрезано: `no`

```typescript
import { describe, expect, it } from 'vitest'
import type { InfiniteData } from '@tanstack/vue-query'
import type { FolderMessage, FolderMessageListResponse } from '../types/folders'
import {
	findCachedFolderMessage,
	optimisticFolderMessageForTarget,
	removeFolderMessageFromFolderList,
	upsertFolderMessageInFolderList
} from './optimisticFolderMessageUpdates'

function folderMessage(overrides: Partial<FolderMessage> = {}): FolderMessage {
	return {
		folder_id: 'folder-1',
		message_id: 'message-1',
		account_id: 'account-1',
		subject: 'Status',
		sender: 'sender@example.com',
		occurred_at: '2026-06-15T10:00:00Z',
		projected_at: '2026-06-15T10:01:00Z',
		workflow_state: 'new',
		local_state: 'active',
		added_at: '2026-06-15T10:02:00Z',
		attachment_count: 0,
		...overrides
	}
}

function folderMessageList(items: FolderMessage[]): InfiniteData<FolderMessageListResponse> {
	return {
		pages: [
			{
				items,
				next_cursor: null,
				has_more: false
			}
		],
		pageParams: [null]
	}
}

describe('optimistic folder message updates', () => {
	it('upserts folder messages into matching folder caches in newest-first order', () => {
		const older = folderMessage({
			folder_id: 'folder-2',
			message_id: 'older',
			added_at: '2026-06-15T10:00:00Z'
		})
		const newer = folderMessage({
			folder_id: 'folder-2',
			message_id: 'newer',
			added_at: '2026-06-15T11:00:00Z'
		})

		const updated = upsertFolderMessageInFolderList(
			folderMessageList([older]),
			['communications-folder-messages', 'folder-2'],
			newer
		)

		expect(updated?.pages[0]?.items.map((item) => item.message_id)).toEqual([
			'newer',
			'older'
		])
	})

	it('does not upsert folder messages into unrelated folder caches', () => {
		const data = folderMessageList([folderMessage({ folder_id: 'folder-1' })])

		const updated = upsertFolderMessageInFolderList(
			data,
			['communications-folder-messages', 'folder-1'],
			folderMessage({ folder_id: 'folder-2' })
		)

		expect(updated).toBe(data)
	})

	it('removes moved messages from cached source folder lists', () => {
		const first = folderMessage({ message_id: 'message-1' })
		const second = folderMessage({ message_id: 'message-2' })
		const data = folderMessageList([first, second])

		const removed = removeFolderMessageFromFolderList(data, 'message-1')
		const unchanged = removeFolderMessageFromFolderList(data, 'missing')

		expect(removed?.pages[0]?.items).toEqual([second])
		expect(unchanged).toBe(data)
	})

	it('finds cached folder messages and builds target-folder optimistic rows', () => {
		const source = folderMessage({ folder_id: 'folder-1', message_id: 'message-1' })
		const lists: Array<[readonly unknown[], InfiniteData<FolderMessageListResponse> | undefined]> = [
			[['communications-folder-messages', 'folder-1'], folderMessageList([source])]
		]

		const found = findCachedFolderMessage(lists, 'message-1')
		const target = optimisticFolderMessageForTarget(source, 'folder-2', '2026-06-15T12:00:00Z')

		expect(found).toEqual(source)
		expect(target).toMatchObject({
			folder_id: 'folder-2',
			message_id: 'message-1',
			added_at: '2026-06-15T12:00:00Z'
		})
	})
})
```

### `frontend/src/domains/communications/queries/optimisticFolderMessageUpdates.ts`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/domains/communications/queries/optimisticFolderMessageUpdates.ts`
- Size bytes / Размер в байтах: `2777`
- Included characters / Включено символов: `2777`
- Truncated / Обрезано: `no`

```typescript
import type { InfiniteData } from '@tanstack/vue-query'
import type { FolderMessage, FolderMessageListResponse } from '../types/folders'

export type FolderMessageListCache = Array<[
	readonly unknown[],
	InfiniteData<FolderMessageListResponse> | undefined
]>

export function upsertFolderMessageInFolderList(
	data: InfiniteData<FolderMessageListResponse> | undefined,
	queryKey: readonly unknown[],
	folderMessage: FolderMessage
): InfiniteData<FolderMessageListResponse> | undefined {
	if (!data || !folderMessageMatchesQuery(queryKey, folderMessage)) return data

	let changed = false
	const pages = data.pages.map((page, pageIndex) => {
		const existingIndex = page.items.findIndex((item) => item.message_id === folderMessage.message_id)

		if (existingIndex >= 0) {
			const items = page.items.slice()
			items[existingIndex] = folderMessage
			changed = true
			return {
				...page,
				items: sortFolderMessages(items)
			}
		}

		if (pageIndex === 0) {
			changed = true
			return {
				...page,
				items: sortFolderMessages([folderMessage, ...page.items])
			}
		}

		return page
	})

	return changed ? { ...data, pages } : data
}

export function removeFolderMessageFromFolderList(
	data: InfiniteData<FolderMessageListResponse> | undefined,
	messageId: string
): InfiniteData<FolderMessageListResponse> | undefined {
	if (!data) return data

	let changed = false
	const pages = data.pages.map((page) => {
		const items = page.items.filter((item) => item.message_id !== messageId)
		if (items.length === page.items.length) return page
		changed = true
		return { ...page, items }
	})

	return changed ? { ...data, pages } : data
}

export function findCachedFolderMessage(
	folderMessageLists: FolderMessageListCache,
	messageId: string
): FolderMessage | undefined {
	for (const [, data] of folderMessageLists) {
		for (const page of data?.pages ?? []) {
			const folderMessage = page.items.find((item) => item.message_id === messageId)
			if (folderMessage) return folderMessage
		}
	}
	return undefined
}

export function optimisticFolderMessageForTarget(
	source: FolderMessage,
	folderId: string,
	addedAt: string
): FolderMessage {
	return {
		...source,
		folder_id: folderId,
		added_at: addedAt
	}
}

export function folderMessageMatchesQuery(
	queryKey: readonly unknown[],
	folderMessage: FolderMessage
): boolean {
	const folderId = queryKey[1]
	return typeof folderId !== 'string' || folderId === folderMessage.folder_id
}

function sortFolderMessages(items: FolderMessage[]): FolderMessage[] {
	return items
		.slice()
		.sort((left, right) => {
			const addedAt = Date.parse(right.added_at) - Date.parse(left.added_at)
			if (Number.isFinite(addedAt) && addedAt !== 0) return addedAt
			return left.message_id.localeCompare(right.message_id)
		})
}
```

### `frontend/src/domains/communications/queries/optimisticFolderUpdates.test.ts`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/domains/communications/queries/optimisticFolderUpdates.test.ts`
- Size bytes / Размер в байтах: `3205`
- Included characters / Включено символов: `3205`
- Truncated / Обрезано: `no`

```typescript
import { describe, expect, it } from 'vitest'
import type { InfiniteData } from '@tanstack/vue-query'
import type { CommunicationFolder, CommunicationFolderListResponse } from '../types/folders'
import {
	optimisticFolderFromUpdate,
	removeFolderFromFolderList,
	upsertFolderInFolderList
} from './optimisticFolderUpdates'

function folder(overrides: Partial<CommunicationFolder> = {}): CommunicationFolder {
	return {
		folder_id: 'folder-1',
		account_id: 'account-1',
		name: 'Clients',
		description: null,
		color: null,
		sort_order: 100,
		message_count: 0,
		created_at: '2026-06-15T10:00:00Z',
		updated_at: '2026-06-15T10:00:00Z',
		...overrides
	}
}

function folderList(items: CommunicationFolder[]): InfiniteData<CommunicationFolderListResponse> {
	return {
		pages: [
			{
				items,
				next_cursor: null,
				has_more: false
			}
		],
		pageParams: [null]
	}
}

describe('optimistic folder updates', () => {
	it('upserts folders into matching folder list caches in display order', () => {
		const bravo = folder({ folder_id: 'bravo', name: 'Bravo', sort_order: 200 })
		const charlie = folder({ folder_id: 'charlie', name: 'Charlie', sort_order: 300 })
		const alpha = folder({ folder_id: 'alpha', name: 'Alpha', sort_order: 100 })

		const updated = upsertFolderInFolderList(
			folderList([bravo, charlie]),
			['communications-folders', 'account-1'],
			alpha
		)

		expect(updated?.pages[0]?.items.map((item) => item.folder_id)).toEqual([
			'alpha',
			'bravo',
			'charlie'
		])
	})

	it('does not patch account-scoped folder caches for another account', () => {
		const existing = folder({ folder_id: 'existing', account_id: 'account-2' })
		const data = folderList([existing])

		const updated = upsertFolderInFolderList(
			data,
			['communications-folders', 'account-2'],
			folder({ folder_id: 'foreign', account_id: 'account-1' })
		)

		expect(updated).toBe(data)
	})

	it('removes a cached folder when an update no longer matches the account-scoped query', () => {
		const existing = folder({ folder_id: 'folder-1', account_id: 'account-1' })
		const data = folderList([existing])

		const updated = upsertFolderInFolderList(
			data,
			['communications-folders', 'account-1'],
			{ ...existing, account_id: 'account-2' }
		)

		expect(updated?.pages[0]?.items).toEqual([])
	})

	it('removes folders from cached folder lists without touching unchanged caches', () => {
		const first = folder({ folder_id: 'folder-1' })
		const second = folder({ folder_id: 'folder-2' })
		const data = folderList([first, second])

		const removed = removeFolderFromFolderList(data, 'folder-1')
		const unchanged = removeFolderFromFolderList(data, 'missing')

		expect(removed?.pages[0]?.items).toEqual([second])
		expect(unchanged).toBe(data)
	})

	it('builds optimistic folder updates from cached rows and partial updates', () => {
		const existing = folder({ folder_id: 'folder-1', name: 'Old', color: 'blue' })

		expect(optimisticFolderFromUpdate(existing, {
			account_id: null,
			name: 'New',
			color: null
		}, '2026-06-15T11:00:00Z')).toMatchObject({
			folder_id: 'folder-1',
			account_id: null,
			name: 'New',
			color: null,
			updated_at: '2026-06-15T11:00:00Z'
		})
	})
})
```

### `frontend/src/domains/communications/queries/optimisticFolderUpdates.ts`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/domains/communications/queries/optimisticFolderUpdates.ts`
- Size bytes / Размер в байтах: `2622`
- Included characters / Включено символов: `2622`
- Truncated / Обрезано: `no`

```typescript
import type { InfiniteData } from '@tanstack/vue-query'
import type { CommunicationFolder, CommunicationFolderListResponse, CommunicationFolderUpdate } from '../types/folders'

export function upsertFolderInFolderList(
	data: InfiniteData<CommunicationFolderListResponse> | undefined,
	queryKey: readonly unknown[],
	folder: CommunicationFolder
): InfiniteData<CommunicationFolderListResponse> | undefined {
	if (!data) return data
	if (!folderMatchesQuery(queryKey, folder)) {
		return removeFolderFromFolderList(data, folder.folder_id)
	}

	let changed = false
	const pages = data.pages.map((page, pageIndex) => {
		const existingIndex = page.items.findIndex((item) => item.folder_id === folder.folder_id)

		if (existingIndex >= 0) {
			const items = page.items.slice()
			items[existingIndex] = folder
			changed = true
			return {
				...page,
				items: sortFolders(items)
			}
		}

		if (pageIndex === 0) {
			changed = true
			return {
				...page,
				items: sortFolders([folder, ...page.items])
			}
		}

		return page
	})

	return changed ? { ...data, pages } : data
}

export function removeFolderFromFolderList(
	data: InfiniteData<CommunicationFolderListResponse> | undefined,
	folderId: string
): InfiniteData<CommunicationFolderListResponse> | undefined {
	if (!data) return data

	let changed = false
	const pages = data.pages.map((page) => {
		const items = page.items.filter((item) => item.folder_id !== folderId)
		if (items.length === page.items.length) return page
		changed = true
		return { ...page, items }
	})

	return changed ? { ...data, pages } : data
}

export function optimisticFolderFromUpdate(
	existing: CommunicationFolder,
	update: CommunicationFolderUpdate,
	updatedAt: string
): CommunicationFolder {
	return {
		...existing,
		account_id: typeof update.account_id === 'undefined' ? existing.account_id : update.account_id,
		name: update.name ?? existing.name,
		description: typeof update.description === 'undefined' ? existing.description : update.description,
		color: typeof update.color === 'undefined' ? existing.color : update.color,
		sort_order: update.sort_order ?? existing.sort_order,
		updated_at: updatedAt
	}
}

export function folderMatchesQuery(queryKey: readonly unknown[], folder: CommunicationFolder): boolean {
	const accountId = queryKey[1]
	if (typeof accountId !== 'string' || !accountId.trim()) return true
	return folder.account_id === accountId
}

function sortFolders(folders: CommunicationFolder[]): CommunicationFolder[] {
	return folders
		.slice()
		.sort((left, right) => left.sort_order - right.sort_order || left.name.localeCompare(right.name))
}
```

### `frontend/src/domains/communications/queries/optimisticMailUpdates.test.ts`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/domains/communications/queries/optimisticMailUpdates.test.ts`
- Size bytes / Размер в байтах: `6409`
- Included characters / Включено символов: `6409`
- Truncated / Обрезано: `no`

```typescript
import { describe, expect, it } from 'vitest'
import type { InfiniteData } from '@tanstack/vue-query'
import type {
	CommunicationMessageSummary,
	CommunicationDraft,
	CommunicationOutboxItem,
	CommunicationMessagesResponse
} from '../types/communications'
import {
	applyBulkMessageActionToMailDetail,
	applyBulkMessageActionToMailList,
	markOutboxItemCanceled,
	removeDraftFromDraftList,
	upsertDraftInDraftList,
	upsertOutboxItem
} from './optimisticMailUpdates'

function message(overrides: Partial<CommunicationMessageSummary>): CommunicationMessageSummary {
	return {
		message_id: 'msg-1',
		raw_record_id: 'raw-1',
		account_id: 'account-1',
		provider_record_id: 'provider-1',
		subject: 'Quarterly update',
		sender: 'sender@example.com',
		recipients: ['recipient@example.com'],
		body_text_preview: 'Preview',
		occurred_at: '2026-06-14T10:00:00Z',
		projected_at: '2026-06-14T10:01:00Z',
		channel_kind: 'email',
		conversation_id: 'thread-1',
		sender_display_name: 'Sender',
		delivery_state: 'delivered',
		workflow_state: 'new',
		importance_score: null,
		ai_category: null,
		ai_summary: null,
		ai_summary_generated_at: null,
		message_metadata: {},
		attachment_count: 0,
		local_state: 'active',
		local_state_changed_at: null,
		...overrides
	}
}

function mailList(items: CommunicationMessageSummary[]): InfiniteData<CommunicationMessagesResponse> {
	return {
		pages: [
			{
				items,
				next_cursor: null,
				has_more: false
			}
		],
		pageParams: [null]
	}
}

function draft(overrides: Partial<CommunicationDraft> = {}): CommunicationDraft {
	return {
		draft_id: 'draft-1',
		account_id: 'account-1',
		persona_id: null,
		to_recipients: ['reader@example.com'],
		cc_recipients: [],
		bcc_recipients: [],
		subject: 'Draft subject',
		body_text: 'Draft body',
		body_html: null,
		in_reply_to: null,
		references: [],
		status: 'draft',
		scheduled_send_at: null,
		send_attempts: 0,
		last_error: null,
		metadata: {},
		created_at: '2026-06-15T10:00:00Z',
		updated_at: '2026-06-15T10:00:00Z',
		...overrides
	}
}

function outboxItem(overrides: Partial<CommunicationOutboxItem> = {}): CommunicationOutboxItem {
	return {
		outbox_id: 'outbox-1',
		account_id: 'account-1',
		draft_id: 'draft-1',
		to_recipients: ['reader@example.com'],
		cc_recipients: [],
		bcc_recipients: [],
		subject: 'Queued subject',
		body_text: 'Queued body',
		body_html: null,
		status: 'queued',
		scheduled_send_at: null,
		undo_deadline_at: '2026-06-15T10:05:00Z',
		send_attempts: 0,
		claimed_at: null,
		sent_at: null,
		provider_message_id: null,
		last_error: null,
		metadata: {},
		created_at: '2026-06-15T10:00:00Z',
		updated_at: '2026-06-15T10:00:00Z',
		...overrides
	}
}

describe('optimistic mail updates', () => {
	it('marks selected list messages as reviewed without changing unrelated messages', () => {
		const unread = message({ message_id: 'msg-1', workflow_state: 'new' })
		const unrelated = message({ message_id: 'msg-2', workflow_state: 'needs_action' })

		const updated = applyBulkMessageActionToMailList(mailList([unread, unrelated]), {
			action: 'mark_read',
			message_ids: ['msg-1']
		})

		expect(updated?.pages[0]?.items).toEqual([
			{ ...unread, workflow_state: 'reviewed' },
			unrelated
		])
		expect(updated?.pages[0]?.items[0]).not.toBe(unread)
		expect(updated?.pages[0]?.items[1]).toBe(unrelated)
	})

	it('removes changed messages from filtered list caches they no longer match', () => {
		const first = message({ message_id: 'msg-1', local_state: 'active' })
		const second = message({ message_id: 'msg-2', local_state: 'active' })

		const archived = applyBulkMessageActionToMailList(
			mailList([first, second]),
			{
				action: 'archive',
				message_ids: ['msg-1']
			},
			['communications-list', 'account-1', 'new', 'email', undefined, undefined]
		)
		const trashed = applyBulkMessageActionToMailList(
			mailList([first, second]),
			{
				action: 'trash',
				message_ids: ['msg-2']
			},
			['communications-list', 'account-1', undefined, 'email', undefined, undefined]
		)

		expect(archived?.pages[0]?.items).toEqual([second])
		expect(trashed?.pages[0]?.items).toEqual([first])
	})

	it('updates selected message detail state when the detail cache is present', () => {
		const detail = {
			message: {
				...message({ message_id: 'msg-1', workflow_state: 'reviewed' }),
				body_text: 'Full body',
				body_html: null,
				local_state_reason: null
			},
			attachments: []
		}

		const updated = applyBulkMessageActionToMailDetail(detail, {
			action: 'mark_unread',
			message_ids: ['msg-1']
		})

		expect(updated?.message.workflow_state).toBe('new')
		expect(updated?.attachments).toBe(detail.attachments)
	})

	it('upserts saved drafts while preserving unrelated draft rows', () => {
		const existing = draft({ draft_id: 'draft-1', subject: 'Old subject' })
		const unrelated = draft({ draft_id: 'draft-2', subject: 'Unrelated' })
		const updatedDraft = draft({ draft_id: 'draft-1', subject: 'Updated subject' })
		const newDraft = draft({ draft_id: 'draft-3', subject: 'New subject' })

		const replaced = upsertDraftInDraftList([existing, unrelated], updatedDraft)
		const inserted = upsertDraftInDraftList([existing], newDraft)

		expect(replaced).toEqual([updatedDraft, unrelated])
		expect(replaced?.[1]).toBe(unrelated)
		expect(inserted).toEqual([newDraft, existing])
	})

	it('removes deleted drafts without changing unchanged draft caches', () => {
		const first = draft({ draft_id: 'draft-1' })
		const second = draft({ draft_id: 'draft-2' })
		const drafts = [first, second]

		expect(removeDraftFromDraftList(drafts, 'draft-1')).toEqual([second])
		expect(removeDraftFromDraftList(drafts, 'missing')).toBe(drafts)
	})

	it('upserts and cancel-patches outbox rows for undo-send UX', () => {
		const queued = outboxItem({ outbox_id: 'outbox-1', status: 'queued' })
		const other = outboxItem({ outbox_id: 'outbox-2', status: 'scheduled' })
		const sent = outboxItem({ outbox_id: 'outbox-1', status: 'sent', sent_at: '2026-06-15T10:06:00Z' })

		expect(upsertOutboxItem([queued, other], sent)).toEqual([sent, other])
		expect(upsertOutboxItem([other], queued)).toEqual([queued, other])

		const canceled = markOutboxItemCanceled([queued, other], 'outbox-1')
		expect(canceled?.[0]).toMatchObject({
			outbox_id: 'outbox-1',
			status: 'canceled',
			undo_deadline_at: null
		})
		expect(canceled?.[1]).toBe(other)
	})
})
```

### `frontend/src/domains/communications/queries/optimisticMailUpdates.ts`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/domains/communications/queries/optimisticMailUpdates.ts`
- Size bytes / Размер в байтах: `8103`
- Included characters / Включено символов: `8103`
- Truncated / Обрезано: `no`

```typescript
import type { InfiniteData } from '@tanstack/vue-query'
import type {
	BulkMessageActionRequest,
	CommunicationMessageSummary,
	CommunicationDraft,
	CommunicationOutboxItem,
	LocalMessageState,
	CommunicationMessageDetailItem,
	CommunicationMessageDetailResponse,
	CommunicationMessagesResponse,
	WorkflowState
} from '../types/communications'

const WORKFLOW_STATES = new Set<WorkflowState>([
	'new',
	'reviewed',
	'needs_action',
	'waiting',
	'done',
	'archived',
	'muted',
	'spam'
])

const LOCAL_STATES = new Set<LocalMessageState>(['active', 'trash', 'all'])

type MailListFilters = {
	workflowState?: WorkflowState
	localState?: LocalMessageState
}

export function applyBulkMessageActionToMailList(
	data: InfiniteData<CommunicationMessagesResponse> | undefined,
	request: BulkMessageActionRequest,
	queryKey?: readonly unknown[]
): InfiniteData<CommunicationMessagesResponse> | undefined {
	if (!data) return data

	const targetIds = new Set(request.message_ids)
	if (targetIds.size === 0) return data

	const filters = parseMailListFilters(queryKey)
	let changed = false

	const pages = data.pages.map((page) => {
		let pageChanged = false
		const items: CommunicationMessageSummary[] = []

		for (const item of page.items) {
			if (!targetIds.has(item.message_id)) {
				items.push(item)
				continue
			}

			const updated = applyBulkMessageActionToSummary(item, request)
			if (!isVisibleInMailList(updated, filters)) {
				pageChanged = true
				changed = true
				continue
			}

			items.push(updated)
			if (updated !== item) {
				pageChanged = true
				changed = true
			}
		}

		if (!pageChanged) return page
		return { ...page, items }
	})

	if (!changed) return data
	return { ...data, pages }
}

export function applyBulkMessageActionToMailDetail(
	data: CommunicationMessageDetailResponse | null | undefined,
	request: BulkMessageActionRequest
): CommunicationMessageDetailResponse | null | undefined {
	if (!data || !data.message || !request.message_ids.includes(data.message.message_id)) return data

	const updatedMessage = applyBulkMessageActionToDetailItem(data.message, request)
	if (updatedMessage === data.message) return data
	return { ...data, message: updatedMessage }
}

export function upsertDraftInDraftList(
	drafts: CommunicationDraft[] | undefined,
	draft: CommunicationDraft
): CommunicationDraft[] | undefined {
	if (!drafts) return drafts

	const index = drafts.findIndex((item) => item.draft_id === draft.draft_id)
	if (index === -1) return [draft, ...drafts]

	const next = drafts.slice()
	next[index] = draft
	return next
}

export function removeDraftFromDraftList(
	drafts: CommunicationDraft[] | undefined,
	draftId: string
): CommunicationDraft[] | undefined {
	if (!drafts) return drafts
	const next = drafts.filter((draft) => draft.draft_id !== draftId)
	return next.length === drafts.length ? drafts : next
}

export function upsertOutboxItem(
	items: CommunicationOutboxItem[] | undefined,
	item: CommunicationOutboxItem
): CommunicationOutboxItem[] | undefined {
	if (!items) return items

	const index = items.findIndex((existing) => existing.outbox_id === item.outbox_id)
	if (index === -1) return [item, ...items]

	const next = items.slice()
	next[index] = item
	return next
}

export function markOutboxItemCanceled(
	items: CommunicationOutboxItem[] | undefined,
	outboxId: string
): CommunicationOutboxItem[] | undefined {
	if (!items) return items

	let changed = false
	const next = items.map((item) => {
		if (item.outbox_id !== outboxId) return item
		if (item.status === 'canceled' && item.undo_deadline_at === null) return item
		changed = true
		return {
			...item,
			status: 'canceled' as const,
			undo_deadline_at: null,
			last_error: null
		}
	})

	return changed ? next : items
}

function applyBulkMessageActionToSummary(
	message: CommunicationMessageSummary,
	request: BulkMessageActionRequest
): CommunicationMessageSummary {
	const updated = applyBulkMessageActionToBaseMessage(message, request)
	return updated === message ? message : { ...message, ...updated }
}

function applyBulkMessageActionToDetailItem(
	message: CommunicationMessageDetailItem,
	request: BulkMessageActionRequest
): CommunicationMessageDetailItem {
	const updated = applyBulkMessageActionToBaseMessage(message, request)
	return updated === message ? message : { ...message, ...updated }
}

function applyBulkMessageActionToBaseMessage<T extends {
	workflow_state: WorkflowState
	local_state: LocalMessageState
	message_metadata: Record<string, unknown>
}>(
	message: T,
	request: BulkMessageActionRequest
): T | Partial<T> {
	switch (request.action) {
		case 'mark_read':
			return message.workflow_state === 'reviewed' ? message : { workflow_state: 'reviewed' } as Partial<T>
		case 'mark_unread':
			return message.workflow_state === 'new' ? message : { workflow_state: 'new' } as Partial<T>
		case 'archive':
			return message.workflow_state === 'archived' ? message : { workflow_state: 'archived' } as Partial<T>
		case 'trash':
			return message.local_state === 'trash' ? message : { local_state: 'trash' } as Partial<T>
		case 'restore':
			return message.local_state === 'active' ? message : { local_state: 'active' } as Partial<T>
		case 'pin':
			return applyMetadataUpdate(message, { pinned: true })
		case 'unpin':
			return applyMetadataUpdate(message, { pinned: false })
		case 'important':
			return applyMetadataUpdate(message, { important: true })
		case 'not_important':
			return applyMetadataUpdate(message, { important: false })
		case 'add_label':
			return addLabel(message, request.label)
		case 'remove_label':
			return removeLabel(message, request.label)
		case 'snooze':
			return request.snooze_until
				? applyMetadataUpdate(message, { snooze_until: request.snooze_until })
				: message
	}
}

function applyMetadataUpdate<T extends { message_metadata: Record<string, unknown> }>(
	message: T,
	metadataPatch: Record<string, unknown>
): Partial<T> {
	return {
		message_metadata: {
			...message.message_metadata,
			...metadataPatch
		}
	} as Partial<T>
}

function addLabel<T extends { message_metadata: Record<string, unknown> }>(
	message: T,
	label: string | undefined
): T | Partial<T> {
	const normalized = label?.trim()
	if (!normalized) return message

	const labels = currentLabels(message.message_metadata)
	if (labels.includes(normalized)) return message

	return applyMetadataUpdate(message, {
		labels: [...labels, normalized].sort()
	})
}

function removeLabel<T extends { message_metadata: Record<string, unknown> }>(
	message: T,
	label: string | undefined
): T | Partial<T> {
	const normalized = label?.trim()
	if (!normalized) return message

	const labels = currentLabels(message.message_metadata)
	const nextLabels = labels.filter((value) => value !== normalized)
	if (nextLabels.length === labels.length) return message

	return applyMetadataUpdate(message, { labels: nextLabels })
}

function currentLabels(metadata: Record<string, unknown>): string[] {
	const labels = metadata.labels
	if (!Array.isArray(labels)) return []
	return labels.filter((label): label is string => typeof label === 'string' && label.trim() !== '')
}

function parseMailListFilters(queryKey?: readonly unknown[]): MailListFilters {
	const workflowState = queryKey?.[2]
	const localState = queryKey?.[5]

	return {
		workflowState: isWorkflowState(workflowState) ? workflowState : undefined,
		localState: isLocalState(localState) ? localState : undefined
	}
}

function isVisibleInMailList(
	message: CommunicationMessageSummary,
	filters: MailListFilters
): boolean {
	if (filters.workflowState && message.workflow_state !== filters.workflowState) {
		return false
	}

	if (filters.localState === 'all') return true

	const localState = filters.localState ?? 'active'
	return message.local_state === localState
}

function isWorkflowState(value: unknown): value is WorkflowState {
	return typeof value === 'string' && WORKFLOW_STATES.has(value as WorkflowState)
}

function isLocalState(value: unknown): value is LocalMessageState {
	return typeof value === 'string' && LOCAL_STATES.has(value as LocalMessageState)
}
```

### `frontend/src/domains/communications/queries/outboxInfiniteQuery.boundary.test.ts`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/domains/communications/queries/outboxInfiniteQuery.boundary.test.ts`
- Size bytes / Размер в байтах: `1438`
- Included characters / Включено символов: `1438`
- Truncated / Обрезано: `no`

```typescript
import { describe, expect, it } from 'vitest'
import { readFileSync } from 'node:fs'

describe('outbox infinite query boundary', () => {
	it('uses TanStack infinite query cursor loading for outbox server state', () => {
		const source = readFileSync(
			new URL('./mailOperationQueries.ts', import.meta.url),
			'utf8'
		)

		expect(source).toContain('useInfiniteQuery')
		expect(source).toContain('fetchOutboxItems(toValue(accountId), toValue(status), 100, pageParam)')
		expect(source).toContain('getNextPageParam: (lastPage) => lastPage.next_cursor ?? undefined')
		expect(source).toContain('select: (data) => data.pages.flatMap((page) => page.items)')
	})

	it('exposes load-more state from the outbox strip hook without API ownership in the component', () => {
		const hookSource = readFileSync(
			new URL('./outboxStatusStrip.ts', import.meta.url),
			'utf8'
		)
		const componentSource = readFileSync(
			new URL('../components/OutboxStatusStrip.vue', import.meta.url),
			'utf8'
		)

		expect(hookSource).toContain('hasMoreOutboxItems')
		expect(hookSource).toContain('loadMoreOutboxItems')
		expect(hookSource).toContain('prefetchMoreOutboxItems')
		expect(hookSource).toContain('outboxQuery.fetchNextPage()')
		expect(componentSource).toContain("loadMore: []")
		expect(componentSource).toContain("prefetchMore: []")
		expect(componentSource).not.toContain('fetch(')
		expect(componentSource).not.toContain('ApiClient')
	})
})
```

### `frontend/src/domains/communications/queries/outboxStatusStrip.ts`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/domains/communications/queries/outboxStatusStrip.ts`
- Size bytes / Размер в байтах: `2184`
- Included characters / Включено символов: `2184`
- Truncated / Обрезано: `no`

```typescript
import { computed, type MaybeRefOrGetter } from 'vue'
import {
  useOutboxQuery,
  useUndoOutboxMutation
} from './useCommunicationsQuery'

type OutboxStatusStripOptions = {
  onStatus?: (message: string) => void
  onError?: (message: string) => void
}

export function useOutboxStatusStrip(
  accountId: MaybeRefOrGetter<string | undefined>,
  options: OutboxStatusStripOptions = {}
) {
  const outboxQuery = useOutboxQuery(accountId)
  const undoMutation = useUndoOutboxMutation()
  const outboxItems = computed(() => outboxQuery.data.value ?? [])
  const outboxErrorMessage = computed(() => {
    if (!outboxQuery.error.value) return ''
    return outboxQuery.error.value instanceof Error
      ? outboxQuery.error.value.message
      : 'Failed to load outbox'
  })
  const isUndoingOutbox = computed(() => undoMutation.isPending.value)
  const hasMoreOutboxItems = computed(() => Boolean(outboxQuery.hasNextPage.value))
  const isLoadingMoreOutbox = computed(() => outboxQuery.isFetchingNextPage.value)

  async function undoOutbox(outboxId: string): Promise<void> {
    try {
      await undoMutation.mutateAsync(outboxId)
      options.onStatus?.('Send canceled')
      await outboxQuery.refetch()
    } catch (error) {
      options.onError?.(error instanceof Error ? error.message : 'Undo send failed')
    }
  }

  async function loadMoreOutboxItems(): Promise<void> {
    if (!outboxQuery.hasNextPage.value || outboxQuery.isFetchingNextPage.value) return
    try {
      await outboxQuery.fetchNextPage()
    } catch (error) {
      options.onError?.(error instanceof Error ? error.message : 'Failed to load outbox')
    }
  }

  async function prefetchMoreOutboxItems(): Promise<void> {
    if (!outboxQuery.hasNextPage.value || outboxQuery.isFetchingNextPage.value) return
    try {
      await outboxQuery.fetchNextPage()
    } catch {
      // Prefetch is opportunistic; explicit load-more reports user-facing errors.
    }
  }

  return {
    outboxItems,
    outboxErrorMessage,
    isOutboxLoading: outboxQuery.isLoading,
    isLoadingMoreOutbox,
    hasMoreOutboxItems,
    isUndoingOutbox,
    undoOutbox,
    loadMoreOutboxItems,
    prefetchMoreOutboxItems
  }
}
```

### `frontend/src/domains/communications/queries/queryTypes.ts`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/domains/communications/queries/queryTypes.ts`
- Size bytes / Размер в байтах: `168`
- Included characters / Включено символов: `168`
- Truncated / Обрезано: `no`

```typescript
import type { MaybeRefOrGetter } from 'vue'

export type QueryParam<T> = MaybeRefOrGetter<T | undefined>
export type NullableQueryParam<T> = MaybeRefOrGetter<T | null>
```

### `frontend/src/domains/communications/queries/realtimeMailPatches.ts`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/domains/communications/queries/realtimeMailPatches.ts`
- Size bytes / Размер в байтах: `21122`
- Included characters / Включено символов: `12000`
- Truncated / Обрезано: `yes`

```typescript
import type { InfiniteData } from '@tanstack/vue-query'
import type {
	BulkMessageActionRequest,
	CommunicationDraft,
	CommunicationOutboxItem,
	CommunicationMessageDetailResponse,
	CommunicationMessagesResponse,
	OutboxListResponse,
	MailSyncStatus
} from '../types/communications'
import type {
	FolderMessage,
	FolderMessageListResponse,
	CommunicationFolder,
	CommunicationFolderListResponse
} from '../types/folders'
import type { CommunicationSavedSearch, SavedSearchListResponse } from '../types/savedSearches'
import type { CommunicationAiStateRecord } from '../types/aiState'
import {
	applyBulkMessageActionToMailDetail,
	applyBulkMessageActionToMailList
} from './optimisticMailUpdates'
import {
	aiStateValue,
	folderMessageValue,
	folderValue,
	isRecord,
	normalizeBulkAction,
	normalizeMessageIds,
	nullableNumberValue,
	nullableStringValue,
	numberValue,
	outboxStatusValue,
	savedSearchValue,
	storedEventEnvelope,
	stringValue,
	type AiStatePatchPayload,
	type DraftPatchPayload,
	type FolderMessagePatchPayload,
	type CommunicationMessagePatchPayload,
	type MailRealtimePatchQueryClient,
	type OutboxPatchPayload,
	type SyncPatchPayload
} from './realtimePatchShared'

export type { MailRealtimePatchQueryClient } from './realtimePatchShared'

type AvailableMailRealtimePatchQueryClient = Required<
	Pick<MailRealtimePatchQueryClient, 'getQueriesData' | 'setQueryData'>
>

export function applyMailRealtimePatch(
	eventData: string,
	queryClient: MailRealtimePatchQueryClient
): boolean {
	const { getQueriesData, setQueryData } = queryClient
	if (!getQueriesData || !setQueryData) return false
	const availableQueryClient: AvailableMailRealtimePatchQueryClient = {
		getQueriesData,
		setQueryData
	}

	if (applyAiStateRealtimePatch(eventData, availableQueryClient)) return true
	if (applyOutboxRealtimePatch(eventData, availableQueryClient)) return true
	if (applyDraftRealtimePatch(eventData, availableQueryClient)) return true
	if (applyFolderRealtimePatch(eventData, availableQueryClient)) return true
	if (applyFolderMessageRealtimePatch(eventData, availableQueryClient)) return true
	if (applySavedSearchRealtimePatch(eventData, availableQueryClient)) return true
	if (applySyncRealtimePatch(eventData, availableQueryClient)) return true

	const request = bulkActionRequestFromEvent(eventData)
	if (!request) return false

	let patched = false
	for (const [queryKey, data] of availableQueryClient.getQueriesData<InfiniteData<CommunicationMessagesResponse>>({
		queryKey: ['communications-list']
	})) {
		availableQueryClient.setQueryData(queryKey, () =>
			applyBulkMessageActionToMailList(data, request, queryKey)
		)
		patched = true
	}

	for (const messageId of request.message_ids) {
		const queryKey = ['communications-message', messageId] as const
		availableQueryClient.setQueryData<CommunicationMessageDetailResponse | null | undefined>(queryKey, (data) =>
			applyBulkMessageActionToMailDetail(data, request)
		)
		patched = true
	}

	return patched
}

function applyAiStateRealtimePatch(
	eventData: string,
	queryClient: Required<Pick<MailRealtimePatchQueryClient, 'getQueriesData' | 'setQueryData'>>
): boolean {
	const envelope = storedEventEnvelope(eventData)
	if (envelope?.event?.event_type !== 'mail.ai_state.changed') return false

	const payload = envelope.event.payload as AiStatePatchPayload | undefined
	const messageId = stringValue(payload?.message_id)
	const aiState = aiStateValue(payload?.ai_state)
	if (!messageId || !aiState) return false

	queryClient.setQueryData<CommunicationAiStateRecord | null | undefined>(
		['communications-ai-state', messageId],
		(data) => ({
			message_id: messageId,
			ai_state: aiState,
			review_reason:
				typeof payload?.review_required === 'boolean' && payload.review_required
					? data?.review_reason ?? null
					: data?.review_reason ?? null,
			last_error:
				typeof payload?.failed === 'boolean' && payload.failed
					? data?.last_error ?? null
					: data?.last_error ?? null,
			created_at: data?.created_at ?? new Date().toISOString(),
			updated_at: new Date().toISOString()
		})
	)

	return true
}

function applySyncRealtimePatch(
	eventData: string,
	queryClient: Required<Pick<MailRealtimePatchQueryClient, 'getQueriesData' | 'setQueryData'>>
): boolean {
	const envelope = storedEventEnvelope(eventData)
	const event = envelope?.event
	const eventType = event?.event_type
	if (
		eventType !== 'mail.sync.started' &&
		eventType !== 'mail.sync.progress' &&
		eventType !== 'mail.sync.completed' &&
		eventType !== 'mail.sync.failed' &&
		eventType !== 'mail.sync.skipped'
	) {
		return false
	}

	const payload = event?.payload as SyncPatchPayload | undefined
	const accountId = stringValue(payload?.account_id)
	if (!accountId) return false

	let patched = false
	for (const [queryKey, data] of queryClient.getQueriesData<MailSyncStatus[]>({
		queryKey: ['communications', 'mail', 'sync-statuses']
	})) {
		const updated = patchSyncStatuses(data, accountId, payload)
		if (updated !== data) {
			queryClient.setQueryData(queryKey, updated)
			patched = true
		}
	}

	return patched
}

function patchSyncStatuses(
	items: MailSyncStatus[] | undefined,
	accountId: string,
	payload: SyncPatchPayload | undefined
): MailSyncStatus[] | undefined {
	if (!items || !payload) return items

	let changed = false
	const patched = items.map((item) => {
		if (item.account_id !== accountId) return item
		changed = true
		return {
			...item,
			status: stringValue(payload.status) ?? item.status,
			phase: stringValue(payload.phase) ?? item.phase,
			progress_mode: stringValue(payload.progress_mode) ?? item.progress_mode,
			progress_percent:
				typeof payload.progress_percent === 'undefined'
					? item.progress_percent
					: nullableNumberValue(payload.progress_percent),
			processed_messages: numberValue(payload.processed_messages) ?? item.processed_messages,
			estimated_total_messages:
				typeof payload.estimated_total_messages === 'undefined'
					? item.estimated_total_messages
					: nullableNumberValue(payload.estimated_total_messages),
			current_batch_size: numberValue(payload.current_batch_size) ?? item.current_batch_size,
			next_run_at:
				typeof payload.next_run_at === 'undefined'
					? item.next_run_at
					: nullableStringValue(payload.next_run_at),
			last_error_code:
				typeof payload.error_code === 'undefined'
					? item.last_error_code
					: nullableStringValue(payload.error_code),
			last_fetched_messages: numberValue(payload.fetched_messages) ?? item.last_fetched_messages,
			last_projected_messages:
				numberValue(payload.projected_messages) ?? item.last_projected_messages,
			last_upserted_persons:
				numberValue(payload.upserted_persons) ?? item.last_upserted_persons,
			last_upserted_organizations:
				numberValue(payload.upserted_organizations) ?? item.last_upserted_organizations
		}
	})

	return changed ? patched : items
}

function applyFolderMessageRealtimePatch(
	eventData: string,
	queryClient: Required<Pick<MailRealtimePatchQueryClient, 'getQueriesData' | 'setQueryData'>>
): boolean {
	const envelope = storedEventEnvelope(eventData)
	const event = envelope?.event
	const eventType = event?.event_type
	if (
		eventType !== 'mail.folder_message.copied' &&
		eventType !== 'mail.folder_message.moved'
	) {
		return false
	}

	const payload = event?.payload as FolderMessagePatchPayload | undefined
	const folderMessage = folderMessageValue(payload?.message)
	const messageId = stringValue(payload?.message_id)
	if (!folderMessage || !messageId) return false

	let patched = false
	for (const [queryKey, data] of queryClient.getQueriesData<InfiniteData<FolderMessageListResponse>>({
		queryKey: ['communications-folder-messages']
	})) {
		const updated = patchFolderMessageList(data, queryKey, eventType, folderMessage, messageId)
		if (updated !== data) {
			queryClient.setQueryData(queryKey, updated)
			patched = true
		}
	}

	return patched
}

function patchFolderMessageList(
	data: InfiniteData<FolderMessageListResponse> | undefined,
	queryKey: readonly unknown[],
	eventType: string,
	folderMessage: FolderMessage,
	messageId: string
): InfiniteData<FolderMessageListResponse> | undefined {
	if (!data) return data

	const queryFolderId = typeof queryKey[1] === 'string' ? queryKey[1] : null
	let changed = false
	const pages = data.pages.map((page, pageIndex) => {
		if (queryFolderId === folderMessage.folder_id) {
			const existingIndex = page.items.findIndex((item) => item.message_id === folderMessage.message_id)
			if (existingIndex >= 0) {
				changed = true
				const items = page.items.slice()
				items[existingIndex] = folderMessage
				return {
					...page,
					items: sortFolderMessages(items)
				}
			}

			if (pageIndex === 0) {
				changed = true
				return {
					...page,
					items: sortFolderMessages([folderMessage, ...page.items])
				}
			}
		}

		if (eventType === 'mail.folder_message.moved' && queryFolderId !== folderMessage.folder_id) {
			const updated = page.items.filter((item) => item.message_id !== messageId)
			if (updated.length !== page.items.length) {
				changed = true
				return {
					...page,
					items: updated
				}
			}
		}

		return page
	})

	return changed ? { ...data, pages } : data
}

function sortFolderMessages(items: FolderMessage[]): FolderMessage[] {
	return items
		.slice()
		.sort((left, right) => {
			const addedAt = Date.parse(right.added_at) - Date.parse(left.added_at)
			if (Number.isFinite(addedAt) && addedAt !== 0) return addedAt
			return left.message_id.localeCompare(right.message_id)
		})
}

function applySavedSearchRealtimePatch(
	eventData: string,
	queryClient: Required<Pick<MailRealtimePatchQueryClient, 'getQueriesData' | 'setQueryData'>>
): boolean {
	const envelope = storedEventEnvelope(eventData)
	const event = envelope?.event
	const eventType = event?.event_type
	if (
		eventType !== 'mail.saved_search.created' &&
		eventType !== 'mail.saved_search.updated' &&
		eventType !== 'mail.saved_search.deleted'
	) {
		return false
	}

	const savedSearch = savedSearchValue(event?.payload)
	if (!savedSearch) return false

	let patched = false
	for (const [queryKey, data] of queryClient.getQueriesData<InfiniteData<SavedSearchListResponse>>({
		queryKey: ['communications-saved-searches']
	})) {
		const updated = patchSavedSearchList(data, queryKey, eventType, savedSearch)
		if (updated !== data) {
			queryClient.setQueryData(queryKey, updated)
			patched = true
		}
	}

	return patched
}

function patchSavedSearchList(
	data: InfiniteData<SavedSearchListResponse> | undefined,
	queryKey: readonly unknown[],
	eventType: string,
	savedSearch: CommunicationSavedSearch
): InfiniteData<SavedSearchListResponse> | undefined {
	if (!data) return data

	const matchesQuery = savedSearchMatchesQuery(queryKey, savedSearch)
	if (eventType === 'mail.saved_search.deleted' || !matchesQuery) {
		return removeSavedSearchFromPages(data, savedSearch.saved_search_id)
	}

	let found = false
	let changed = false
	const pages = data.pages.map((page) => {
		const existingIndex = page.items.findIndex((item) => item.saved_search_id === savedSearch.saved_search_id)
		if (existingIndex < 0) return page

		found = true
		changed = true
		const items = page.items.slice()
		items[existingIndex] = savedSearch
		return { ...page, items: sortSavedSearches(items) }
	})

	if (eventType === 'mail.saved_search.created' && !found && pages.length > 0) {
		const [firstPage, ...restPages] = pages
		return {
			...data,
			pages: [{ ...firstPage, items: sortSavedSearches([savedSearch, ...firstPage.items]) }, ...restPages]
		}
	}

	return changed ? { ...data, pages } : data
}

function removeSavedSearchFromPages(
	data: InfiniteData<SavedSearchListResponse>,
	savedSearchId: string
): InfiniteData<SavedSearchListResponse> {
	let changed = false
	const pages = data.pages.map((page) => {
		const items = page.items.filter((item) => item.saved_search_id !== savedSearchId)
		if (items.length === page.items.length) return page
		changed = true
		retu
```
_Source file truncated after 12000 characters. / Исходный файл обрезан после 12000 символов._

### `frontend/src/domains/communications/queries/realtimePatchShared.ts`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/domains/communications/queries/realtimePatchShared.ts`
- Size bytes / Размер в байтах: `75`
- Included characters / Включено символов: `75`
- Truncated / Обрезано: `no`

```typescript
export * from '../../../shared/communications/queries/realtimePatchShared'
```

### `frontend/src/domains/communications/queries/realtimeTelegramMediaPatches.ts`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/domains/communications/queries/realtimeTelegramMediaPatches.ts`
- Size bytes / Размер в байтах: `10512`
- Included characters / Включено символов: `10512`
- Truncated / Обрезано: `no`

```typescript
import type {
  TelegramMediaItem,
  TelegramMediaSearchResponse,
  TelegramMessage,
} from '../../../shared/communications/types/telegram'
import { isRecord, stringValue } from '../../../shared/communications/queries/realtimePatchShared'
import {
  type TelegramEventPayload,
  matchesMessageScope,
} from './realtimeTelegramPatchShared'

export function isTelegramMediaDownloadEvent(eventType: string): boolean {
  return (
    eventType === 'telegram.media.download.started' ||
    eventType === 'telegram.media.download.progress' ||
    eventType === 'telegram.media.download.failed' ||
    eventType === 'telegram.media.downloaded'
  )
}

export function patchTelegramMessageMediaDownloadState(
  message: TelegramMessage,
  eventType: string,
  payload: TelegramEventPayload | undefined,
  snapshot: TelegramMessage | null
): TelegramMessage {
  if (!isTelegramMediaDownloadEvent(eventType) || !payload) return message
  if (eventType === 'telegram.media.downloaded' && snapshot) return snapshot
  if (message.provider_chat_id && stringValue(payload.provider_chat_id) !== message.provider_chat_id) {
    return message
  }
  if (stringValue(payload.provider_message_id) !== message.provider_message_id) return message

  const nextMetadata = patchAttachmentCollection(message.metadata, payload)
  return nextMetadata === message.metadata ? message : { ...message, metadata: nextMetadata }
}

export function patchTelegramMediaSearch(
  queryKey: readonly unknown[],
  response: TelegramMediaSearchResponse | undefined,
  eventType: string,
  payload: TelegramEventPayload | undefined,
  snapshot: TelegramMessage | null
): TelegramMediaSearchResponse | undefined {
  if (!response || !isTelegramMediaDownloadEvent(eventType)) return response
  if (queryKey[0] !== 'communications' || queryKey[1] !== 'telegram' || queryKey[2] !== 'search' || queryKey[3] !== 'media') {
    return response
  }

  if (eventType === 'telegram.media.downloaded' && snapshot) {
    return upsertDownloadedMediaSnapshot(queryKey, response, snapshot)
  }

  if (!payload) return response
  const providerMessageId = stringValue(payload.provider_message_id)
  const providerChatId = stringValue(payload.provider_chat_id)
  if (!providerMessageId || !providerChatId) return response

  const nextItems = response.items.map((item) =>
    patchMediaItemDownloadState(item, providerChatId, providerMessageId, payload)
  )
  return nextItems.some((item, index) => item !== response.items[index])
    ? { ...response, items: nextItems }
    : response
}

function upsertDownloadedMediaSnapshot(
  queryKey: readonly unknown[],
  response: TelegramMediaSearchResponse,
  snapshot: TelegramMessage
): TelegramMediaSearchResponse {
  const query = typeof queryKey[4] === 'string' ? queryKey[4].trim().toLowerCase() : ''
  const accountId = typeof queryKey[5] === 'string' && queryKey[5] !== 'all' ? queryKey[5] : null
  const providerChatId = typeof queryKey[6] === 'string' && queryKey[6] !== 'all' ? queryKey[6] : null
  const kindFilter = typeof queryKey[7] === 'string' && queryKey[7] !== 'all' ? queryKey[7] : null
  const limit = typeof queryKey[8] === 'number' ? queryKey[8] : null

  if (!matchesMessageScope(snapshot, accountId, providerChatId)) return response

  const nextItemsById = new Map(
    response.items.map((item) => [`${item.message_id}:${item.file_name}`, item] as const)
  )
  for (const item of telegramMediaItemsFromMessageSnapshot(snapshot)) {
    if (kindFilter && item.kind !== kindFilter) continue
    if (query && !matchesMediaQuery(item, query)) continue
    nextItemsById.set(`${item.message_id}:${item.file_name}`, item)
  }

  const nextItems = Array.from(nextItemsById.values()).sort(
    (left, right) => (right.occurred_at ?? '').localeCompare(left.occurred_at ?? '')
  )
  return { ...response, items: typeof limit === 'number' ? nextItems.slice(0, limit) : nextItems }
}

function patchMediaItemDownloadState(
  item: TelegramMediaItem,
  providerChatId: string,
  providerMessageId: string,
  payload: TelegramEventPayload
): TelegramMediaItem {
  if (item.provider_chat_id !== providerChatId || item.provider_message_id !== providerMessageId) {
    return item
  }
  if (!attachmentMatchesPayload(item, payload)) return item

  return {
    ...item,
    download_state: stringValue(payload.download_state) ?? item.download_state,
    tdlib_file_id:
      typeof payload.tdlib_file_id === 'number' ? payload.tdlib_file_id : item.tdlib_file_id,
    provider_attachment_id:
      stringValue(payload.provider_attachment_id) ?? item.provider_attachment_id,
    local_path: stringValue(payload.local_path) ?? item.local_path,
    expected_size_bytes:
      typeof payload.expected_size_bytes === 'number'
        ? payload.expected_size_bytes
        : item.expected_size_bytes,
    downloaded_size_bytes:
      typeof payload.downloaded_size_bytes === 'number'
        ? payload.downloaded_size_bytes
        : item.downloaded_size_bytes,
    is_downloading_active:
      typeof payload.is_downloading_active === 'boolean'
        ? payload.is_downloading_active
        : item.is_downloading_active,
    is_downloading_completed:
      typeof payload.is_downloading_completed === 'boolean'
        ? payload.is_downloading_completed
        : item.is_downloading_completed,
    last_error: stringValue(payload.error) ?? item.last_error,
  }
}

function patchAttachmentCollection(
  metadata: Record<string, unknown>,
  payload: TelegramEventPayload
): Record<string, unknown> {
  const attachmentKey = Array.isArray(metadata.attachments)
    ? 'attachments'
    : Array.isArray(metadata.files)
      ? 'files'
      : null
  if (!attachmentKey) return metadata

  const rawAttachments = metadata[attachmentKey]
  if (!Array.isArray(rawAttachments)) return metadata
  const nextAttachments = rawAttachments.map((attachment) =>
    patchAttachmentRecord(attachment, payload)
  )
  return nextAttachments.some((attachment, index) => attachment !== rawAttachments[index])
    ? { ...metadata, [attachmentKey]: nextAttachments }
    : metadata
}

function patchAttachmentRecord(attachment: unknown, payload: TelegramEventPayload): unknown {
  if (!isRecord(attachment) || !attachmentMatchesPayload(attachment, payload)) return attachment

  const nextAttachment = { ...attachment }
  if (typeof payload.tdlib_file_id === 'number') nextAttachment.tdlib_file_id = payload.tdlib_file_id
  if (typeof payload.download_state === 'string') nextAttachment.download_state = payload.download_state
  if (typeof payload.local_path === 'string') nextAttachment.local_path = payload.local_path
  if (typeof payload.provider_attachment_id === 'string') {
    nextAttachment.provider_attachment_id = payload.provider_attachment_id
  }
  if (typeof payload.expected_size_bytes === 'number') {
    nextAttachment.expected_size_bytes = payload.expected_size_bytes
  }
  if (typeof payload.downloaded_size_bytes === 'number') {
    nextAttachment.downloaded_size_bytes = payload.downloaded_size_bytes
  }
  if (typeof payload.is_downloading_active === 'boolean') {
    nextAttachment.is_downloading_active = payload.is_downloading_active
  }
  if (typeof payload.is_downloading_completed === 'boolean') {
    nextAttachment.is_downloading_completed = payload.is_downloading_completed
  }
  if (typeof payload.error === 'string') nextAttachment.last_error = payload.error
  return nextAttachment
}

function attachmentMatchesPayload(
  attachment: Record<string, unknown> | TelegramMediaItem,
  payload: TelegramEventPayload
): boolean {
  const payloadAttachmentId = stringValue(payload.attachment_id) ?? stringValue(payload.provider_attachment_id)
  const attachmentId =
    stringValue((attachment as Record<string, unknown>).attachment_id)
    ?? stringValue((attachment as Record<string, unknown>).provider_attachment_id)
  if (payloadAttachmentId && attachmentId) return payloadAttachmentId === attachmentId

  const payloadTdlibFileId = typeof payload.tdlib_file_id === 'number' ? payload.tdlib_file_id : null
  const attachmentTdlibFileId =
    typeof (attachment as Record<string, unknown>).tdlib_file_id === 'number'
      ? ((attachment as Record<string, unknown>).tdlib_file_id as number)
      : null
  return payloadTdlibFileId !== null && attachmentTdlibFileId === payloadTdlibFileId
}

function telegramMediaItemsFromMessageSnapshot(message: TelegramMessage): TelegramMediaItem[] {
  const rawAttachments = message.metadata?.attachments ?? message.metadata?.files
  if (!Array.isArray(rawAttachments)) return []
  return rawAttachments.flatMap((attachment): TelegramMediaItem[] => {
    if (!isRecord(attachment)) return []
    const fileName = stringValue(attachment.filename) ?? stringValue(attachment.file_name)
    const kind = stringValue(attachment.attachment_type) ?? stringValue(attachment.kind) ?? 'file'
    if (!fileName || !message.provider_chat_id) return []
    return [{
      message_id: message.message_id,
      provider_message_id: message.provider_message_id,
      provider_chat_id: message.provider_chat_id,
      file_name: fileName,
      kind,
      mime_type: stringValue(attachment.content_type) ?? stringValue(attachment.mime_type),
      size_bytes: typeof attachment.size === 'number'
        ? attachment.size
        : typeof attachment.size_bytes === 'number' ? attachment.size_bytes : null,
      occurred_at: message.occurred_at,
      download_state: stringValue(attachment.download_state) ?? 'unknown',
      tdlib_file_id: typeof attachment.tdlib_file_id === 'number' ? attachment.tdlib_file_id : null,
      provider_attachment_id: stringValue(attachment.attachment_id) ?? stringValue(attachment.provider_attachment_id),
      local_path: stringValue(attachment.local_path),
      expected_size_bytes: typeof attachment.expected_size_bytes === 'number' ? attachment.expected_size_bytes : null,
      downloaded_size_bytes: typeof attachment.downloaded_size_bytes === 'number' ? attachment.downloaded_size_bytes : null,
      is_downloading_active: typeof attachment.is_downloading_active === 'boolean' ? attachment.is_downloading_active : null,
      is_downloading_completed: typeof attachment.is_downloading_completed === 'boolean' ? attachment.is_downloading_completed : null,
      last_error: stringValue(attachment.last_error),
    }]
  })
}

function matchesMediaQuery(item: TelegramMediaItem, query: string): boolean {
  return [item.file_name, item.kind, item.provider_message_id, item.mime_type ?? '']
    .join(' ')
    .toLowerCase()
    .includes(query)
}
```

### `frontend/src/domains/communications/queries/realtimeTelegramParticipantPatches.ts`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/domains/communications/queries/realtimeTelegramParticipantPatches.ts`
- Size bytes / Размер в байтах: `5797`
- Included characters / Включено символов: `5797`
- Truncated / Обрезано: `no`

```typescript
import type { TelegramChatMember } from '../../../shared/communications/types/telegram'
import { isRecord, storedEventEnvelope, stringValue } from '../../../shared/communications/queries/realtimePatchShared'

type TelegramChatMembersPage = {
  items: TelegramChatMember[]
  next_cursor: string | null
}

type TelegramChatMembersInfiniteData = {
  pages: TelegramChatMembersPage[]
  pageParams: unknown[]
}

export type TelegramParticipantPatchQueryClient = {
  getQueriesData?: <TData>(filters: { queryKey: readonly unknown[] }) => Array<
    [readonly unknown[], TData | undefined]
  >
  setQueryData?: <TData>(
    queryKey: readonly unknown[],
    updater: TData | ((data: TData | undefined) => TData | undefined)
  ) => unknown
}

export function applyTelegramParticipantRealtimePatch(
  eventData: string,
  queryClient: TelegramParticipantPatchQueryClient
): boolean {
  const { getQueriesData, setQueryData } = queryClient
  if (!getQueriesData || !setQueryData) return false

  const envelope = storedEventEnvelope(eventData)
  const event = envelope?.event
  if (event?.event_type !== 'telegram.participant.updated') return false
  const payload = isRecord(event.payload) ? event.payload : undefined
  const telegramChatId = stringValue(payload?.telegram_chat_id)
  const participant = telegramChatMemberSnapshot(payload?.participant)
  if (!telegramChatId || !participant) return false

  let patched = false
  for (const [queryKey, data] of getQueriesData<TelegramChatMember[] | TelegramChatMembersInfiniteData>({
    queryKey: ['communications', 'telegram', 'chat-members']
  })) {
    if (queryKey[3] !== telegramChatId) continue
    const updated = patchParticipantQuery(queryKey, data, participant)
    if (updated !== data) {
      setQueryData(queryKey, updated)
      patched = true
    }
  }
  return patched
}

function telegramChatMemberSnapshot(value: unknown): TelegramChatMember | null {
  if (!isRecord(value)) return null
  const providerMemberId = stringValue(value.provider_member_id) ?? stringValue(value.sender_id)
  if (!providerMemberId) return null
  return {
    sender_id: stringValue(value.sender_id) ?? providerMemberId,
    sender_display_name: stringValue(value.sender_display_name),
    message_count: typeof value.message_count === 'number' ? value.message_count : 0,
    last_message_at: stringValue(value.last_message_at),
    source: telegramMemberSource(value.source),
    provider_member_id: providerMemberId,
    username: stringValue(value.username),
    role: stringValue(value.role),
    status: stringValue(value.status),
    is_admin: value.is_admin === true,
    is_owner: value.is_owner === true,
    permissions: isRecord(value.permissions) ? value.permissions : {},
    observed_at: stringValue(value.observed_at),
  }
}

function telegramMemberSource(value: unknown): TelegramChatMember['source'] {
  return value === 'tdlib' || value === 'bot_api' || value === 'message_heuristic'
    ? value
    : 'tdlib'
}

function patchParticipantQuery(
  queryKey: readonly unknown[],
  data: TelegramChatMember[] | TelegramChatMembersInfiniteData | undefined,
  participant: TelegramChatMember
): TelegramChatMember[] | TelegramChatMembersInfiniteData | undefined {
  if (!data) return data

  const query = typeof queryKey[5] === 'string' ? queryKey[5].trim().toLowerCase() : ''
  const role = typeof queryKey[6] === 'string' ? queryKey[6].trim().toLowerCase() : ''

  if (Array.isArray(data)) {
    return patchParticipantCollection(data, participant, query, role)
  }

  if (!isInfiniteData(data)) return data
  const nextPages = data.pages.map((page) => ({
    ...page,
    items: patchParticipantCollection(page.items, participant, query, role) ?? page.items,
  }))
  return nextPages.some((page, index) => page.items !== data.pages[index]?.items)
    ? { ...data, pages: nextPages }
    : data
}

function patchParticipantCollection(
  members: TelegramChatMember[] | undefined,
  participant: TelegramChatMember,
  query: string,
  role: string
): TelegramChatMember[] | undefined {
  if (!members) return members
  if (participantIsInactive(participant)) {
    return members.filter((member) => member.provider_member_id !== participant.provider_member_id)
  }
  const participantMatches = participantMatchesFilters(participant, query, role)
  const existingIndex = members.findIndex(
    (member) => member.provider_member_id === participant.provider_member_id
  )
  if (!participantMatches) {
    if (existingIndex < 0) return members
    return members.filter((member) => member.provider_member_id !== participant.provider_member_id)
  }
  if (existingIndex < 0) return [participant, ...members]
  return members.map((member, index) => (index === existingIndex ? participant : member))
}

function participantIsInactive(participant: TelegramChatMember): boolean {
  const status = (participant.status ?? '').trim().toLowerCase()
  const role = (participant.role ?? '').trim().toLowerCase()
  return (
    status === 'left' ||
    status === 'banned' ||
    status === 'absent_exhaustive' ||
    role === 'left' ||
    role === 'banned'
  )
}

function participantMatchesFilters(
  participant: TelegramChatMember,
  query: string,
  role: string
): boolean {
  if (role && (participant.role ?? '').trim().toLowerCase() !== role) return false
  if (!query) return true

  return [
    participant.sender_display_name ?? '',
    participant.sender_id,
    participant.provider_member_id,
    participant.username ?? '',
    participant.role ?? '',
    participant.status ?? '',
    participant.source,
  ]
    .join(' ')
    .toLowerCase()
    .includes(query)
}

function isInfiniteData(value: unknown): value is TelegramChatMembersInfiniteData {
  return isRecord(value) && Array.isArray(value.pages) && Array.isArray(value.pageParams)
}
```

### `frontend/src/domains/communications/queries/realtimeTelegramPatchShared.ts`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/domains/communications/queries/realtimeTelegramPatchShared.ts`
- Size bytes / Размер в байтах: `7739`
- Included characters / Включено символов: `7739`
- Truncated / Обрезано: `no`

```typescript
import type { TelegramChat, TelegramMessage } from '../../../shared/communications/types/telegram'
import { isRecord, stringValue } from '../../../shared/communications/queries/realtimePatchShared'

export const TELEGRAM_TYPING_TTL_MS = 7000

export type TelegramEventPayload = {
  account_id?: unknown
  provider_chat_id?: unknown
  provider_message_id?: unknown
  delivery_state?: unknown
  runtime_kind?: unknown
  status?: unknown
  version_number?: unknown
  reason_class?: unknown
  tombstone_id?: unknown
  reaction_emoji?: unknown
  is_active?: unknown
  scope?: unknown
  synced_count?: unknown
  has_more?: unknown
  tdlib_file_id?: unknown
  download_state?: unknown
  local_path?: unknown
  provider_attachment_id?: unknown
  expected_size_bytes?: unknown
  downloaded_size_bytes?: unknown
  is_downloading_active?: unknown
  is_downloading_completed?: unknown
  error?: unknown
  attachment_id?: unknown
  blob_id?: unknown
  scan_status?: unknown
  command_id?: unknown
  command_kind?: unknown
  retry_count?: unknown
  max_retries?: unknown
  last_error?: unknown
  result_payload?: unknown
  next_attempt_at?: unknown
  last_attempt_at?: unknown
  provider_observed_at?: unknown
  provider_state?: unknown
  reconciliation_status?: unknown
  reconciled_at?: unknown
  dead_lettered_at?: unknown
  completed_at?: unknown
  idempotency_key?: unknown
  target_ref?: unknown
  capability_state?: unknown
  action_class?: unknown
  confirmation_decision?: unknown
  audit_metadata?: unknown
  actor_id?: unknown
  happened_at?: unknown
  created_at?: unknown
  updated_at?: unknown
  action?: unknown
  list_kind?: unknown
  provider_folder_id?: unknown
  order?: unknown
  message_id?: unknown
  is_pinned?: unknown
  is_archived?: unknown
  is_muted?: unknown
  telegram_chat_id?: unknown
  provider_thread_id?: unknown
  sender_id?: unknown
  topic?: unknown
  chat?: unknown
  message?: unknown
  items?: unknown
  payload?: unknown
}

export type TelegramStoredEventEnvelope = {
  event?: {
    event_type?: unknown
    occurred_at?: unknown
    metadata?: unknown
    subject?: unknown
    payload?: unknown
  }
}

export function eventSubjectId(subject: unknown): string | null {
  if (!isRecord(subject)) return null
  return stringValue(subject.id)
}

export function runtimeAccountId(queryKey: readonly unknown[]): string | null {
  if (queryKey[0] !== 'integrations' || queryKey[1] !== 'telegram' || queryKey[2] !== 'runtime') return null
  return typeof queryKey[3] === 'string' ? queryKey[3] : null
}

export function telegramChatSnapshot(value: unknown): TelegramChat | null {
  if (!isRecord(value)) return null
  const telegramChatId = stringValue(value.telegram_chat_id)
  const accountId = stringValue(value.account_id)
  const providerChatId = stringValue(value.provider_chat_id)
  const chatKind = stringValue(value.chat_kind)
  const title = stringValue(value.title)
  const syncState = stringValue(value.sync_state)
  const createdAt = stringValue(value.created_at)
  const updatedAt = stringValue(value.updated_at)
  if (!telegramChatId || !accountId || !providerChatId || !chatKind || !title || !syncState || !createdAt || !updatedAt) {
    return null
  }
  return {
    telegram_chat_id: telegramChatId,
    account_id: accountId,
    provider_chat_id: providerChatId,
    chat_kind: chatKind as TelegramChat['chat_kind'],
    title,
    username: stringValue(value.username),
    sync_state: syncState as TelegramChat['sync_state'],
    last_message_at: stringValue(value.last_message_at),
    metadata: isRecord(value.metadata) ? value.metadata : {},
    created_at: createdAt,
    updated_at: updatedAt,
  }
}

export function telegramMessageSnapshot(value: unknown): TelegramMessage | null {
  if (!isRecord(value)) return null
  const messageId = stringValue(value.message_id)
  const accountId = stringValue(value.account_id)
  const providerMessageId = stringValue(value.provider_message_id)
  const chatTitle = stringValue(value.chat_title)
  const sender = stringValue(value.sender)
  const projectedAt = stringValue(value.projected_at)
  const channelKind = stringValue(value.channel_kind)
  const deliveryState = stringValue(value.delivery_state)
  if (!messageId || !accountId || !providerMessageId || !chatTitle || !sender || !projectedAt || !channelKind || !deliveryState) {
    return null
  }
  return {
    message_id: messageId,
    raw_record_id: stringValue(value.raw_record_id) ?? '',
    account_id: accountId,
    provider_message_id: providerMessageId,
    provider_chat_id: stringValue(value.provider_chat_id),
    chat_title: chatTitle,
    sender,
    sender_display_name: stringValue(value.sender_display_name),
    text: stringValue(value.text) ?? '',
    occurred_at: stringValue(value.occurred_at),
    projected_at: projectedAt,
    channel_kind: channelKind as TelegramMessage['channel_kind'],
    delivery_state: deliveryState,
    metadata: isRecord(value.metadata) ? value.metadata : {},
  }
}

export function messageQueryScope(queryKey: readonly unknown[]): [string | null, string | null, number | null] {
  if (queryKey[0] !== 'communications' || queryKey[1] !== 'telegram' || queryKey[2] !== 'messages') return [null, null, null]
  const accountId = typeof queryKey[3] === 'string' && queryKey[3] !== 'all' && queryKey[3] !== 'none'
    ? queryKey[3]
    : null
  const providerChatId = typeof queryKey[4] === 'string' && queryKey[4] !== 'all' && queryKey[4] !== 'none'
    ? queryKey[4]
    : null
  const limit = typeof queryKey[5] === 'number' ? queryKey[5] : null
  return [accountId, providerChatId, limit]
}

export function chatQueryScope(queryKey: readonly unknown[]): [string | null, number | null] {
  if (queryKey[0] !== 'communications' || queryKey[1] !== 'telegram' || queryKey[2] !== 'chats') return [null, null]
  const accountId = typeof queryKey[3] === 'string' && queryKey[3] !== 'all' ? queryKey[3] : null
  const limit = typeof queryKey[4] === 'number' ? queryKey[4] : null
  return [accountId, limit]
}

export function matchesMessageScope(message: TelegramMessage, accountId: string | null, providerChatId: string | null): boolean {
  if (accountId && message.account_id !== accountId) return false
  if (providerChatId && message.provider_chat_id !== providerChatId) return false
  return true
}

export function matchesChatScope(chat: TelegramChat, accountId: string | null): boolean {
  if (accountId && chat.account_id !== accountId) return false
  return true
}

export function insertMessageByRecency(
  messages: TelegramMessage[],
  nextMessage: TelegramMessage,
  limit: number | null
): TelegramMessage[] {
  const items = [nextMessage, ...messages.filter((m) => m.message_id !== nextMessage.message_id)]
  items.sort((l, r) => messageRecencyKey(r).localeCompare(messageRecencyKey(l)))
  return typeof limit === 'number' ? items.slice(0, limit) : items
}

export function insertChatByRecency(
  chats: TelegramChat[],
  nextChat: TelegramChat,
  limit: number | null
): TelegramChat[] {
  const items = [nextChat, ...chats.filter((c) => c.telegram_chat_id !== nextChat.telegram_chat_id)]
  items.sort((l, r) => chatRecencyKey(r).localeCompare(chatRecencyKey(l)))
  return typeof limit === 'number' ? items.slice(0, limit) : items
}

export function patchPinMetadata(
  metadata: Record<string, unknown>,
  payload: TelegramEventPayload | undefined
): Record<string, unknown> {
  if (typeof payload?.is_pinned !== 'boolean') return metadata
  return { ...metadata, pinned: payload.is_pinned, is_pinned: payload.is_pinned }
}

function messageRecencyKey(m: TelegramMessage): string { return m.occurred_at ?? m.projected_at ?? '' }
function chatRecencyKey(c: TelegramChat): string { return c.last_message_at ?? c.updated_at ?? '' }
```

### `frontend/src/domains/communications/queries/realtimeTelegramPatches.ts`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/domains/communications/queries/realtimeTelegramPatches.ts`
- Size bytes / Размер в байтах: `24655`
- Included characters / Включено символов: `12000`
- Truncated / Обрезано: `yes`

```typescript
import type {
  TelegramChat,
  TelegramChatGroupFilter,
  TelegramMediaSearchResponse,
  TelegramMessage,
  TelegramMessageListResponse,
  TelegramMessageSearchResponse,
  TelegramReactionListResponse,
  TelegramRuntimeStatus
} from '../../../shared/communications/types/telegram'
import type { TelegramTopicListResponse } from '../../../shared/communications/types/telegramTopics'
import { isRecord, storedEventEnvelope, stringValue } from '../../../shared/communications/queries/realtimePatchShared'
import { patchTelegramTopicList } from './realtimeTelegramTopicPatches'
import {
  isTelegramMediaDownloadEvent,
  patchTelegramMediaSearch,
  patchTelegramMessageMediaDownloadState,
} from './realtimeTelegramMediaPatches'
import {
  TELEGRAM_TYPING_TTL_MS,
  type TelegramEventPayload,
  type TelegramStoredEventEnvelope,
  chatQueryScope,
  eventSubjectId,
  insertChatByRecency,
  insertMessageByRecency,
  matchesChatScope,
  matchesMessageScope,
  messageQueryScope,
  patchPinMetadata,
  runtimeAccountId,
  telegramChatSnapshot,
  telegramMessageSnapshot
} from './realtimeTelegramPatchShared'

export type TelegramRealtimePatchQueryClient = {
  getQueriesData?: <TData>(filters: { queryKey: readonly unknown[] }) => Array<
    [readonly unknown[], TData | undefined]
  >
  setQueryData?: <TData>(
    queryKey: readonly unknown[],
    updater: TData | ((data: TData | undefined) => TData | undefined)
  ) => unknown
}

export function applyTelegramRealtimePatch(
  eventData: string,
  queryClient: TelegramRealtimePatchQueryClient
): boolean {
  const { getQueriesData, setQueryData } = queryClient
  if (!getQueriesData || !setQueryData) return false

  const envelope = storedEventEnvelope(eventData) as TelegramStoredEventEnvelope | null
  const eventType = stringValue(envelope?.event?.event_type)
  if (!eventType || !eventType.startsWith('telegram.')) return false

  const occurredAt = stringValue(envelope?.event?.occurred_at)
  const subjectId = eventSubjectId(envelope?.event?.subject)
  const payload = isRecord(envelope?.event?.payload)
    ? (envelope?.event?.payload as TelegramEventPayload)
    : undefined
  const metadata = isRecord(envelope?.event?.metadata)
    ? (envelope?.event?.metadata as Record<string, unknown>)
    : undefined
  const snapshot = telegramMessageSnapshot(payload?.message)
  const chatSnapshot = telegramChatSnapshot(payload?.chat)

  let patched = false
  for (const [queryKey, data] of getQueriesData<TelegramMessage[]>({
    queryKey: ['communications', 'telegram', 'messages']
  })) {
    const updated = patchMessageList(queryKey, data, eventType, subjectId, payload, snapshot)
    if (updated !== data) {
      setQueryData(queryKey, updated)
      patched = true
    }
  }

  for (const [queryKey, data] of getQueriesData<TelegramMessageListResponse>({
    queryKey: ['communications', 'telegram', 'chats']
  })) {
    if (!isTelegramPinnedMessagesQueryKey(queryKey)) continue
    const updated = patchPinnedMessages(queryKey, data, eventType, subjectId, payload, snapshot)
    if (updated !== data) {
      setQueryData(queryKey, updated)
      patched = true
    }
  }

  for (const [queryKey, data] of getQueriesData<TelegramChat[]>({
    queryKey: ['communications', 'telegram', 'chats']
  })) {
    if (isTelegramPinnedMessagesQueryKey(queryKey)) continue
    const updated = patchChatList(queryKey, data, eventType, payload, chatSnapshot, occurredAt)
    if (updated !== data) {
      setQueryData(queryKey, updated)
      patched = true
    }
  }

  for (const [queryKey, data] of getQueriesData<TelegramChat | null>({
    queryKey: ['communications', 'telegram', 'chat-detail']
  })) {
    const updated = patchChatDetail(queryKey, data, eventType, payload, chatSnapshot, occurredAt)
    if (updated !== data) {
      setQueryData(queryKey, updated)
      patched = true
    }
  }

  for (const [queryKey, data] of getQueriesData<TelegramChatGroupFilter[]>({
    queryKey: ['communications', 'telegram', 'folders']
  })) {
    const updated = patchFolderFilters(queryKey, data, eventType, payload, metadata)
    if (updated !== data) {
      setQueryData(queryKey, updated)
      patched = true
    }
  }

  for (const [queryKey, data] of getQueriesData<TelegramMessageSearchResponse>({
    queryKey: ['communications', 'telegram', 'search', 'messages']
  })) {
    const updated = patchMessageSearch(queryKey, data, eventType, subjectId, payload, snapshot)
    if (updated !== data) {
      setQueryData(queryKey, updated)
      patched = true
    }
  }

  for (const [queryKey, data] of getQueriesData<TelegramMediaSearchResponse>({
    queryKey: ['communications', 'telegram', 'search', 'media']
  })) {
    const updated = patchTelegramMediaSearch(queryKey, data, eventType, payload, snapshot)
    if (updated !== data) {
      setQueryData(queryKey, updated)
      patched = true
    }
  }

  for (const [queryKey, data] of getQueriesData<TelegramReactionListResponse>({
    queryKey: ['communications', 'telegram', 'message-reactions']
  })) {
    const updated = patchReactionDetail(queryKey, data, eventType, subjectId, payload)
    if (updated !== data) {
      setQueryData(queryKey, updated)
      patched = true
    }
  }

  for (const [queryKey, data] of getQueriesData<TelegramTopicListResponse>({
    queryKey: ['communications', 'telegram']
  })) {
    const updated = patchTelegramTopicList(queryKey, data, eventType, payload)
    if (updated !== data) {
      setQueryData(queryKey, updated)
      patched = true
    }
  }

  for (const [queryKey, data] of getQueriesData<TelegramRuntimeStatus | null>({
    queryKey: ['integrations', 'telegram', 'runtime']
  })) {
    const updated = patchRuntimeStatus(queryKey, data, eventType, payload, metadata)
    if (updated !== data) {
      setQueryData(queryKey, updated)
      patched = true
    }
  }

  return patched
}

function patchMessageList(
  queryKey: readonly unknown[],
  messages: TelegramMessage[] | undefined,
  eventType: string,
  subjectId: string | null,
  payload: TelegramEventPayload | undefined,
  snapshot: TelegramMessage | null
): TelegramMessage[] | undefined {
  if (!messages) return messages

  if (isTelegramMediaDownloadEvent(eventType)) {
    const nextMessages = messages.map((message) =>
      patchTelegramMessageMediaDownloadState(message, eventType, payload, snapshot)
    )
    return nextMessages.some((message, index) => message !== messages[index])
      ? nextMessages
      : messages
  }

  const targetMessageId = subjectId ?? snapshot?.message_id ?? null
  if (!targetMessageId) return messages
  const [accountId, providerChatId, limit] = messageQueryScope(queryKey)
  if (snapshot && !matchesMessageScope(snapshot, accountId, providerChatId)) {
    return messages
  }

  const patched = messages.map((message) => {
    if (message.message_id !== targetMessageId) return message

    if (eventType === 'telegram.message.created') {
      if (snapshot) return snapshot
      return {
        ...message,
        delivery_state: stringValue(payload?.delivery_state) ?? message.delivery_state,
      }
    }

    if (eventType === 'telegram.message.updated' || eventType === 'telegram.message.edited') {
      const meta = {
        ...(snapshot?.metadata ?? message.metadata),
        lifecycle: {
          ...(snapshot && isRecord(snapshot.metadata.lifecycle) ? snapshot.metadata.lifecycle : {}),
          ...(isRecord(message.metadata.lifecycle) ? message.metadata.lifecycle : {}),
          latest_version_number:
            typeof payload?.version_number === 'number'
              ? payload.version_number
              : snapshot?.metadata.latest_version_number ?? message.metadata.latest_version_number ?? null,
        },
      }
      return { ...(snapshot ?? message), metadata: patchPinMetadata(meta, payload) }
    }

    if (eventType === 'telegram.message.deleted' || eventType === 'telegram.message.visibility_restored') {
      return {
        ...(snapshot ?? message),
        metadata: {
          ...(snapshot?.metadata ?? message.metadata),
          tombstone: {
            reason_class: stringValue(payload?.reason_class),
            tombstone_id: stringValue(payload?.tombstone_id),
            is_visible: eventType === 'telegram.message.visibility_restored',
          },
        },
      }
    }

    if (eventType === 'telegram.media.downloaded' && snapshot) return snapshot
    if (eventType === 'telegram.reaction.changed') {
      const reactionEmoji = stringValue(payload?.reaction_emoji)
      if (!reactionEmoji) return message

      const currentMetadata = snapshot?.metadata ?? message.metadata
      const currentSummary = isRecord(currentMetadata.reaction_summary)
        ? currentMetadata.reaction_summary
        : { reactions: [] as Array<Record<string, unknown>> }
      const currentReactions = Array.isArray(currentSummary.reactions) ? currentSummary.reactions : []
      const existingIndex = currentReactions.findIndex(
        (item) => isRecord(item) && stringValue(item.reaction_emoji) === reactionEmoji
      )
      const isActive = payload?.is_active === true
      const nextReactions = currentReactions.slice()
      if (existingIndex >= 0 && isRecord(nextReactions[existingIndex])) {
        const existing = nextReactions[existingIndex]
        const currentCount = typeof existing.count === 'number' ? existing.count : 0
        nextReactions[existingIndex] = {
          ...existing,
          count: isActive ? currentCount + 1 : Math.max(currentCount - 1, 0),
        }
      } else if (isActive) {
        nextReactions.push({ reaction_emoji: reactionEmoji, count: 1, senders: [] })
      }
      return {
        ...(snapshot ?? message),
        metadata: {
          ...currentMetadata,
          reaction_summary: {
            ...currentSummary,
            reactions: nextReactions.filter(
              (item) => !isRecord(item) || typeof item.count !== 'number' || item.count > 0
            ),
          },
        },
      }
    }

    return message
  })

  const existingIndex = patched.findIndex((message) => message.message_id === targetMessageId)
  if (existingIndex >= 0) return patched
  if ((eventType === 'telegram.message.created' || eventType === 'telegram.media.downloaded') && snapshot) {
    return insertMessageByRecency(messages, snapshot, limit)
  }
  if (eventType === 'telegram.message.updated' && snapshot) {
    return insertMessageByRecency(
      messages,
      { ...snapshot, metadata: patchPinMetadata(snapshot.metadata, payload) },
      limit
    )
  }
  return messages
}

function patchRuntimeStatus(
  queryKey: readonly unknown[],
  status: TelegramRuntimeStatus | null | undefined,
  eventType: string,
  payload: TelegramEventPayload | undefined,
  metadata: Record<string, unknown> | undefined
): TelegramRuntimeStatus | null | undefined {
  if (!status) return status
  const queryAccountId = runtimeAccountId(queryKey)
  const eventAccountId = stringValue(metadata?.account_id)
  if (queryAccountId && eventAccountId && queryAccountId !== eventAccountId) return status

  if (eventType.startsWith('telegram.sync.')) {
    if (!payload) return status
    const scope = stringValue(payload.scope)
    if (!scope) return status
    return {
      ...status,
      status:
        eventType === 'telegram.sync.failed'
          ? 'degraded'
          : eventType === 'telegram.sync.started' || eventType === 'telegram.sync.progress'
            ? 'running'
            : status.status,
      last_sync_scope: scope,
      last_sync_status: stringValue(payload.status),
      last_synced_count: typeof payload.synced_count === 'number' ? payload.synced_count : null,
      last_sync_has_more: typeof payload.has_more === 'boolean' ? payload.has_more : null,
      last_sync_provider_chat_id: stringValue(payload.provider_chat_id),
      updated_at: new Date().toISOString(),
    }
  }

  if (eventType !== 'telegram.command.status_changed' || !payload) return status
  return {
    ...status,
    last_command_
```
_Source file truncated after 12000 characters. / Исходный файл обрезан после 12000 символов._

### `frontend/src/domains/communications/queries/realtimeTelegramTopicPatches.ts`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/domains/communications/queries/realtimeTelegramTopicPatches.ts`
- Size bytes / Размер в байтах: `3385`
- Included characters / Включено символов: `3385`
- Truncated / Обрезано: `no`

```typescript
import type { TelegramTopic, TelegramTopicListResponse } from '../../../shared/communications/types/telegramTopics'
import { isRecord, stringValue } from '../../../shared/communications/queries/realtimePatchShared'
import type { TelegramEventPayload } from './realtimeTelegramPatchShared'

export function patchTelegramTopicList(
  queryKey: readonly unknown[],
  response: TelegramTopicListResponse | undefined,
  eventType: string,
  payload: TelegramEventPayload | undefined
): TelegramTopicListResponse | undefined {
  if (!response || eventType !== 'telegram.topic.updated') return response
  if (!isTopicListQueryKey(queryKey)) return response

  const topic = telegramTopicSnapshot(payload?.topic)
  if (!topic || response.telegram_chat_id !== topic.telegram_chat_id) return response

  const query = topicSearchQuery(queryKey)
  if (query && !topic.title.toLowerCase().includes(query)) {
    const nextItems = response.items.filter((item) => item.topic_id !== topic.topic_id)
    return nextItems.length === response.items.length ? response : { ...response, items: nextItems }
  }

  const limit = typeof queryKey[queryKey.length - 1] === 'number'
    ? queryKey[queryKey.length - 1] as number
    : null
  const nextItems = [topic, ...response.items.filter((item) => item.topic_id !== topic.topic_id)]
  nextItems.sort((left, right) => topicSortKey(right).localeCompare(topicSortKey(left)))

  return {
    ...response,
    items: typeof limit === 'number' ? nextItems.slice(0, limit) : nextItems,
  }
}

function isTopicListQueryKey(queryKey: readonly unknown[]): boolean {
  if (queryKey[0] !== 'communications' || queryKey[1] !== 'telegram') return false
  return queryKey[2] === 'topics' || queryKey[2] === 'topic-search'
}

function topicSearchQuery(queryKey: readonly unknown[]): string {
  if (queryKey[2] !== 'topic-search') return ''
  return typeof queryKey[4] === 'string' ? queryKey[4].trim().toLowerCase() : ''
}

function telegramTopicSnapshot(value: unknown): TelegramTopic | null {
  if (!isRecord(value)) return null
  const topicId = stringValue(value.topic_id)
  const telegramChatId = stringValue(value.telegram_chat_id)
  const accountId = stringValue(value.account_id)
  const providerChatId = stringValue(value.provider_chat_id)
  const title = stringValue(value.title)
  const createdAt = stringValue(value.created_at)
  const updatedAt = stringValue(value.updated_at)
  const providerTopicId = typeof value.provider_topic_id === 'number' ? value.provider_topic_id : null
  if (!topicId || !telegramChatId || !accountId || !providerChatId || !title || !createdAt || !updatedAt || providerTopicId === null) {
    return null
  }

  return {
    topic_id: topicId,
    telegram_chat_id: telegramChatId,
    account_id: accountId,
    provider_topic_id: providerTopicId,
    provider_chat_id: providerChatId,
    title,
    icon_emoji: stringValue(value.icon_emoji),
    is_pinned: value.is_pinned === true,
    is_closed: value.is_closed === true,
    unread_count: typeof value.unread_count === 'number' ? value.unread_count : 0,
    last_message_at: stringValue(value.last_message_at),
    metadata: isRecord(value.metadata) ? value.metadata : {},
    created_at: createdAt,
    updated_at: updatedAt,
  }
}

function topicSortKey(topic: TelegramTopic): string {
  return `${topic.is_pinned ? '1' : '0'}:${topic.last_message_at ?? topic.updated_at}`
}
```

### `frontend/src/domains/communications/queries/realtimeWhatsAppPatches.ts`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/domains/communications/queries/realtimeWhatsAppPatches.ts`
- Size bytes / Размер в байтах: `14945`
- Included characters / Включено символов: `12000`
- Truncated / Обрезано: `yes`

```typescript
import type { WhatsappWebMessage } from '../../../shared/communications/types/whatsapp'
import type { CommunicationProviderConversation } from '../types/providerChannels'
import {
	isRecord,
	storedEventEnvelope,
	stringValue,
} from '../../../shared/communications/queries/realtimePatchShared'

type WhatsAppEventPayload = Record<string, unknown>

export type WhatsAppRealtimePatchQueryClient = {
	getQueriesData?: <TData>(filters: { queryKey: readonly unknown[] }) => Array<
		[readonly unknown[], TData | undefined]
	>
	setQueryData?: <TData>(
		queryKey: readonly unknown[],
		updater: TData | ((data: TData | undefined) => TData | undefined)
	) => unknown
}

export function applyWhatsAppRealtimePatch(
	eventData: string,
	queryClient: WhatsAppRealtimePatchQueryClient
): boolean {
	const { getQueriesData, setQueryData } = queryClient
	if (!getQueriesData || !setQueryData) return false

	const envelope = storedEventEnvelope(eventData)
	const eventType = stringValue(envelope?.event?.event_type)
	if (!eventType || !eventType.startsWith('whatsapp.')) return false

	const payload = isRecord(envelope?.event?.payload)
		? (envelope.event?.payload as WhatsAppEventPayload)
		: undefined
	const snapshot = whatsappMessageSnapshot(payload?.message)
	const conversationSnapshot = whatsappConversationSnapshot(payload)
	let patched = false

	for (const [queryKey, data] of getQueriesData<CommunicationProviderConversation[]>({
		queryKey: ['communications', 'whatsapp', 'conversations'],
	})) {
		const updated = patchConversationList(queryKey, data, eventType, payload, conversationSnapshot)
		if (updated !== data) {
			setQueryData(queryKey, updated)
			patched = true
		}
	}

	for (const [queryKey, data] of getQueriesData<CommunicationProviderConversation | null>({
		queryKey: ['communications', 'whatsapp', 'conversation-detail'],
	})) {
		const updated = patchConversationDetail(queryKey, data, eventType, payload, conversationSnapshot)
		if (updated !== data) {
			setQueryData(queryKey, updated)
			patched = true
		}
	}

	for (const [queryKey, data] of getQueriesData<WhatsappWebMessage[]>({
		queryKey: ['communications', 'whatsapp', 'messages'],
	})) {
		const updated = patchMessageList(queryKey, data, eventType, payload, snapshot)
		if (updated !== data) {
			setQueryData(queryKey, updated)
			patched = true
		}
	}

	return patched
}

function patchConversationList(
	queryKey: readonly unknown[],
	conversations: CommunicationProviderConversation[] | undefined,
	eventType: string,
	payload: WhatsAppEventPayload | undefined,
	snapshot: CommunicationProviderConversation | null
): CommunicationProviderConversation[] | undefined {
	if (!conversations || eventType !== 'whatsapp.dialog.updated' || !payload) return conversations

	const accountId = queryScopeValue(queryKey[3])
	const limit = typeof queryKey[4] === 'number' ? queryKey[4] : null
	if (snapshot && accountId && snapshot.account_id !== accountId) return conversations

	const conversationId =
		stringValue(payload.conversation_id) ??
		snapshot?.conversation_id ??
		snapshot?.provider_chat_id ??
		null
	if (!conversationId) return conversations

	const index = conversations.findIndex((conversation) =>
		conversation.conversation_id === conversationId || conversation.provider_chat_id === conversationId
	)
	if (index < 0) {
		if (!snapshot || (accountId && snapshot.account_id !== accountId)) return conversations
		const next = [snapshot, ...conversations]
		return typeof limit === 'number' ? next.slice(0, limit) : next
	}

	const current = conversations[index]
	const nextConversation = patchExistingConversation(current, payload, snapshot)
	if (nextConversation === current) return conversations

	return conversations.map((conversation, currentIndex) =>
		currentIndex === index ? nextConversation : conversation
	)
}

function patchConversationDetail(
	queryKey: readonly unknown[],
	conversation: CommunicationProviderConversation | null | undefined,
	eventType: string,
	payload: WhatsAppEventPayload | undefined,
	snapshot: CommunicationProviderConversation | null
): CommunicationProviderConversation | null | undefined {
	if (!conversation || eventType !== 'whatsapp.dialog.updated' || !payload) return conversation

	const expectedConversationId =
		typeof queryKey[3] === 'string' && queryKey[3] !== 'none' ? queryKey[3] : null
	const payloadConversationId =
		stringValue(payload.conversation_id) ??
		snapshot?.conversation_id ??
		snapshot?.provider_chat_id ??
		null
	if (!payloadConversationId) return conversation
	if (
		expectedConversationId &&
		expectedConversationId !== payloadConversationId &&
		conversation.provider_chat_id !== payloadConversationId
	) {
		return conversation
	}

	return patchExistingConversation(conversation, payload, snapshot)
}

function patchExistingConversation(
	conversation: CommunicationProviderConversation,
	payload: WhatsAppEventPayload,
	snapshot: CommunicationProviderConversation | null
): CommunicationProviderConversation {
	const metadata = {
		...conversation.metadata,
		...snapshot?.metadata,
	}

	patchConversationMetadataFlag(metadata, 'is_pinned', payload.is_pinned)
	patchConversationMetadataFlag(metadata, 'pinned', payload.is_pinned)
	patchConversationMetadataFlag(metadata, 'is_archived', payload.is_archived)
	patchConversationMetadataFlag(metadata, 'archived', payload.is_archived)
	patchConversationMetadataFlag(metadata, 'is_muted', payload.is_muted)
	patchConversationMetadataFlag(metadata, 'muted', payload.is_muted)
	patchConversationMetadataFlag(metadata, 'is_unread', payload.is_unread)
	if (typeof payload.unread_count === 'number') metadata.unread_count = payload.unread_count
	if (typeof payload.participant_count === 'number') metadata.participant_count = payload.participant_count
	if (typeof payload.chat_kind === 'string') metadata.chat_kind = payload.chat_kind
	if (typeof payload.chat_title === 'string') metadata.provider_label = payload.chat_title

	const nextConversation: CommunicationProviderConversation = {
		...(snapshot ?? conversation),
		conversation_id: snapshot?.conversation_id ?? conversation.conversation_id,
		account_id: snapshot?.account_id ?? conversation.account_id,
		provider_chat_id: snapshot?.provider_chat_id ?? conversation.provider_chat_id,
		chat_kind: snapshot?.chat_kind ?? conversation.chat_kind ?? stringValue(payload.chat_kind) ?? undefined,
		title: snapshot?.title ?? conversation.title,
		last_message_at: snapshot?.last_message_at ?? conversation.last_message_at,
		metadata,
		created_at: snapshot?.created_at ?? conversation.created_at,
		updated_at: snapshot?.updated_at ?? conversation.updated_at,
	}

	return shallowConversationEqual(nextConversation, conversation) ? conversation : nextConversation
}

function patchMessageList(
	queryKey: readonly unknown[],
	messages: WhatsappWebMessage[] | undefined,
	eventType: string,
	payload: WhatsAppEventPayload | undefined,
	snapshot: WhatsappWebMessage | null
): WhatsappWebMessage[] | undefined {
	if (!messages || !payload) return messages

	const accountId = queryScopeValue(queryKey[3])
	const providerChatId = queryScopeValue(queryKey[4])
	const payloadAccountId = stringValue(payload.account_id)
	const payloadProviderChatId =
		stringValue(payload.provider_chat_id) ?? snapshot?.provider_chat_id ?? null

	if (accountId && payloadAccountId && payloadAccountId !== accountId) return messages
	if (providerChatId && payloadProviderChatId && payloadProviderChatId !== providerChatId) {
		return messages
	}

	const messageId =
		stringValue(payload.message_id) ??
		stringValue(payload.raw_message_id) ??
		stringValue(payload.provider_message_id) ??
		snapshot?.message_id ??
		null
	if (!messageId) return messages

	const matchIndex = messages.findIndex((message) => message.message_id === messageId)
	if (matchIndex < 0) {
		if (eventType === 'whatsapp.message.created' && snapshot && matchesQueryScope(snapshot, accountId, providerChatId)) {
			return insertMessage(queryKey, messages, snapshot)
		}
		return messages
	}

	const matched = messages[matchIndex]
	const nextMessage = patchExistingMessage(matched, eventType, payload, snapshot)
	if (nextMessage === matched) return messages

	return messages.map((message, index) => (index === matchIndex ? nextMessage : message))
}

function patchExistingMessage(
	message: WhatsappWebMessage,
	eventType: string,
	payload: WhatsAppEventPayload,
	snapshot: WhatsappWebMessage | null
): WhatsappWebMessage {
	if (eventType === 'whatsapp.message.created' && snapshot) {
		return snapshot
	}

	if (eventType === 'whatsapp.message.updated') {
		const nextMetadata = {
			...(snapshot?.metadata ?? message.metadata),
			lifecycle: {
				...(isRecord(message.metadata.lifecycle) ? message.metadata.lifecycle : {}),
				edited: payload.edited === true,
			},
		}
		return { ...(snapshot ?? message), metadata: nextMetadata }
	}

	if (eventType === 'whatsapp.message.deleted') {
		return {
			...(snapshot ?? message),
			metadata: {
				...(snapshot?.metadata ?? message.metadata),
				tombstone: {
					tombstone_id: stringValue(payload.tombstone_id),
					is_visible: false,
				},
			},
		}
	}

	if (eventType === 'whatsapp.reaction.changed') {
		const reaction = stringValue(payload.reaction)
		if (!reaction) return message
		const reactions = normalizeReactionSummary(message.metadata.reaction_summary)
		const nextReactions = payload.is_active === false
			? reactions.filter((item) => item.reaction !== reaction)
			: upsertReactionSummary(reactions, reaction)
		return {
			...message,
			metadata: {
				...message.metadata,
				reaction_summary: { reactions: nextReactions },
			},
		}
	}

	if (eventType === 'whatsapp.receipt.changed') {
		const deliveryState = stringValue(payload.delivery_state)
		return deliveryState ? { ...message, delivery_state: deliveryState } : message
	}

	return message
}

function insertMessage(
	queryKey: readonly unknown[],
	messages: WhatsappWebMessage[],
	snapshot: WhatsappWebMessage
): WhatsappWebMessage[] {
	const limit = typeof queryKey[5] === 'number' ? queryKey[5] : null
	const nextMessages = [snapshot, ...messages]
	return typeof limit === 'number' ? nextMessages.slice(0, limit) : nextMessages
}

function queryScopeValue(value: unknown): string | null {
	return typeof value === 'string' && value !== 'all' ? value : null
}

function matchesQueryScope(
	message: WhatsappWebMessage,
	accountId: string | null,
	providerChatId: string | null
): boolean {
	if (accountId && message.account_id !== accountId) return false
	if (providerChatId && message.provider_chat_id !== providerChatId) return false
	return true
}

function whatsappMessageSnapshot(value: unknown): WhatsappWebMessage | null {
	if (!isRecord(value)) return null

	const messageId = stringValue(value.message_id)
	const accountId = stringValue(value.account_id)
	const providerMessageId = stringValue(value.provider_message_id)
	if (!messageId || !accountId || !providerMessageId) return null

	return {
		message_id: messageId,
		raw_record_id: stringValue(value.raw_record_id) ?? '',
		account_id: accountId,
		provider_message_id: providerMessageId,
		provider_chat_id: stringValue(value.provider_chat_id),
		chat_title: stringValue(value.chat_title) ?? '',
		sender: stringValue(value.sender) ?? '',
		sender_display_name: stringValue(value.sender_display_name),
		text: stringValue(value.text) ?? '',
		occurred_at: stringValue(value.occurred_at),
		projected_at: stringValue(value.projected_at) ?? new Date().toISOString(),
		channel_kind: 'whatsapp_web',
		delivery_state: stringValue(value.delivery_state) ?? 'received',
		metadata: isRecord(value.metadata) ? value.metadata : {},
	}
}

function whatsappConversationSnapshot(
	value: unknown
): CommunicationProviderConversation | null {
	if (!isRecord(value)) return null

	const conversationId = stringValue(value.conversation_id)
	const accountId = stringValue(value.account_id)
	const providerChatId = stringValue(value.provider_chat_id)
	const title = stringValue(value.chat_title)
	const
```
_Source file truncated after 12000 characters. / Исходный файл обрезан после 12000 символов._

### `frontend/src/domains/communications/queries/resourceOverviewInfiniteQuery.boundary.test.ts`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/domains/communications/queries/resourceOverviewInfiniteQuery.boundary.test.ts`
- Size bytes / Размер в байтах: `831`
- Included characters / Включено символов: `831`
- Truncated / Обрезано: `no`

```typescript
import { describe, expect, it } from 'vitest'
import { readFileSync } from 'node:fs'

describe('resource overview infinite query boundary', () => {
  it('uses TanStack infinite queries for mailbox resource lists', () => {
    const source = readFileSync(new URL('./mailWorkspaceQueries.ts', import.meta.url), 'utf8')

    expect(source).toContain('useInfiniteQuery<SubscriptionListResponse')
    expect(source).toContain('useInfiniteQuery<SenderStatsListResponse')
    expect(source).toContain('fetchSubscriptions(toValue(accountId), 25, pageParam)')
    expect(source).toContain('fetchTopSenders(toValue(accountId), 25, pageParam)')
    expect(source).toContain('getNextPageParam: (lastPage) => lastPage.next_cursor ?? undefined')
    expect(source).toContain('select: (data) => data.pages.flatMap((page) => page.items)')
  })
})
```

### `frontend/src/domains/communications/queries/savedSearchInfiniteQuery.boundary.test.ts`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/domains/communications/queries/savedSearchInfiniteQuery.boundary.test.ts`
- Size bytes / Размер в байтах: `712`
- Included characters / Включено символов: `712`
- Truncated / Обрезано: `no`

```typescript
import { describe, expect, it } from 'vitest'
import { readFileSync } from 'node:fs'

describe('saved-search infinite query boundary', () => {
  it('uses TanStack infinite query cursor loading for saved searches and smart folders', () => {
    const source = readFileSync(
      new URL('./mailWorkspaceQueries.ts', import.meta.url),
      'utf8'
    )

    expect(source).toContain('useInfiniteQuery<')
    expect(source).toContain('fetchSavedSearches(toValue(isSmartFolder), toValue(accountId), 100, pageParam)')
    expect(source).toContain('getNextPageParam: (lastPage) => lastPage.next_cursor ?? undefined')
    expect(source).toContain('select: (data) => data.pages.flatMap((page) => page.items)')
  })
})
```
