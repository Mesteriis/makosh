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

- Chunk ID / ID чанка: `142-source-frontend-part-002`
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

### `frontend/src/domains/communications/api/aiState.ts`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/domains/communications/api/aiState.ts`
- Size bytes / Размер в байтах: `811`
- Included characters / Включено символов: `811`
- Truncated / Обрезано: `no`

```typescript
import { ApiClient } from '../../../platform/api/ApiClient'
import type {
  CommunicationAiStateRecord,
  CommunicationAiStateTransitionRequest
} from '../types/aiState'

export async function fetchMessageAiState(messageId: string): Promise<CommunicationAiStateRecord> {
  return ApiClient.instance.get<CommunicationAiStateRecord>(
    `/api/v1/communications/messages/${encodeURIComponent(messageId)}/ai-state`,
    'Message AI state request failed'
  )
}

export async function updateMessageAiState(
  messageId: string,
  request: CommunicationAiStateTransitionRequest
): Promise<CommunicationAiStateRecord> {
  return ApiClient.instance.put<CommunicationAiStateRecord>(
    `/api/v1/communications/messages/${encodeURIComponent(messageId)}/ai-state`,
    request,
    'Message AI state update failed'
  )
}
```

### `frontend/src/domains/communications/api/attachmentApi.ts`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/domains/communications/api/attachmentApi.ts`
- Size bytes / Размер в байтах: `1143`
- Included characters / Включено символов: `1143`
- Truncated / Обрезано: `no`

```typescript
import {
  inspectAttachmentArchiveConnect,
  previewAttachmentConnect,
  searchAttachmentsConnect,
  translateAttachmentConnect
} from './connectCommunications'
import type {
  AttachmentArchiveInspectionResponse,
  AttachmentPreviewResponse,
  AttachmentSearchRequest,
  AttachmentSearchResponse,
  AttachmentTranslationRequest,
  AttachmentTranslationResponse
} from '../types/attachments'

export async function searchAttachments(
  request: AttachmentSearchRequest = {}
): Promise<AttachmentSearchResponse> {
  return searchAttachmentsConnect(request)
}

export async function inspectAttachmentArchive(
  attachmentId: string
): Promise<AttachmentArchiveInspectionResponse> {
  return inspectAttachmentArchiveConnect(attachmentId)
}

export async function previewAttachment(
  attachmentId: string
): Promise<AttachmentPreviewResponse> {
  return previewAttachmentConnect(attachmentId)
}

export async function translateAttachment(
  attachmentId: string,
  request: AttachmentTranslationRequest
): Promise<AttachmentTranslationResponse> {
  return translateAttachmentConnect(attachmentId, request.target_language, request.source_text)
}
```

### `frontend/src/domains/communications/api/attachmentImportApi.test.ts`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/domains/communications/api/attachmentImportApi.test.ts`
- Size bytes / Размер в байтах: `1564`
- Included characters / Включено символов: `1564`
- Truncated / Обрезано: `no`

```typescript
import { afterEach, describe, expect, it, vi } from 'vitest'
import { ApiClient } from '../../../platform/api/ApiClient'
import { importCommunicationAttachment } from './attachmentImportApi'

describe('Communication attachment import API', () => {
  afterEach(() => {
    vi.unstubAllGlobals()
    ApiClient.resetForTests()
  })

  it('imports local attachments through the Communication import endpoint', async () => {
    ApiClient.init('http://127.0.0.1:8080', 'test-secret')
    const fetchMock = vi.fn().mockResolvedValue(
      new Response(
        JSON.stringify({
          attachment_id: 'att-import:1',
          blob_id: 'blob:1',
          content_type: 'text/plain',
          size_bytes: 5,
          sha256: 'sha256:abc',
          scan_status: 'not_scanned',
          storage_kind: 'local_fs',
          storage_path: 'sha256/abc',
        }),
        { status: 200 }
      )
    )
    vi.stubGlobal('fetch', fetchMock)

    const response = await importCommunicationAttachment({
      account_id: 'telegram-1',
      channel_kind: 'telegram',
      filename: 'note.txt',
      content_type: 'text/plain',
      content_base64: 'aGVsbG8=',
    })

    expect(response.attachment_id).toBe('att-import:1')
    expect(fetchMock).toHaveBeenCalledOnce()
    const [url, init] = fetchMock.mock.calls[0]
    expect(url).toContain('/api/v1/communications/attachments/import')
    expect(init.method).toBe('POST')
    expect(JSON.parse(init.body as string)).toMatchObject({
      account_id: 'telegram-1',
      content_base64: 'aGVsbG8=',
    })
  })
})
```

### `frontend/src/domains/communications/api/attachmentImportApi.ts`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/domains/communications/api/attachmentImportApi.ts`
- Size bytes / Размер в байтах: `946`
- Included characters / Включено символов: `946`
- Truncated / Обрезано: `no`

```typescript
import { ApiClient } from '../../../platform/api/ApiClient'

export type CommunicationAttachmentImportRequest = {
  account_id?: string
  channel_kind?: string
  filename?: string
  content_type?: string
  content_base64: string
  source_kind?: string
  metadata?: Record<string, unknown>
}

export type CommunicationAttachmentImportResponse = {
  attachment_id: string
  account_id?: string | null
  channel_kind?: string | null
  blob_id: string
  filename?: string | null
  content_type: string
  size_bytes: number
  sha256: string
  scan_status: string
  storage_kind: string
  storage_path: string
}

export async function importCommunicationAttachment(
  request: CommunicationAttachmentImportRequest
): Promise<CommunicationAttachmentImportResponse> {
  return ApiClient.instance.post<CommunicationAttachmentImportResponse>(
    '/api/v1/communications/attachments/import',
    request,
    'Communication attachment import failed'
  )
}
```

### `frontend/src/domains/communications/api/bilingualReplyFlow.test.ts`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/domains/communications/api/bilingualReplyFlow.test.ts`
- Size bytes / Размер в байтах: `2230`
- Included characters / Включено символов: `2202`
- Truncated / Обрезано: `no`

```typescript
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { ApiClient } from '../../../platform/api/ApiClient'
import { prepareBilingualReplyFlow } from './bilingualReplyFlow'

describe('bilingual reply flow API', () => {
  beforeEach(() => {
    ApiClient.resetForTests()
    ApiClient.init('http://127.0.0.1:8080', 'test-secret')
  })

  afterEach(() => {
    vi.unstubAllGlobals()
    ApiClient.resetForTests()
  })

  it('posts Russian reply text and tone to the protected review endpoint', async () => {
    const fetchMock = vi.fn().mockResolvedValue(
      new Response(
        JSON.stringify({
          message_id: 'mail_message:1',
          subject: 'Re: Contrato',
          tone: 'business',
          reply_language: 'ru',
          send_ready: false,
          original: {
            language: 'es',
            confidence: 0.7,
            text: 'Hola equipo'
          },
          translation: {
            target: 'ru',
            translated: false,
            text: null,
            model: null,
            reason: 'translation runtime unavailable'
          },
          reply: {
            language: 'ru',
            tone: 'business',
            text: 'Спасибо.'
          },
          back_translation: {
            target: 'es',
            translated: false,
            text: null,
            model: null,
            reason: 'translation runtime unavailable'
          }
        }),
        { status: 200, headers: { 'Content-Type': 'application/json' } }
      )
    )
    vi.stubGlobal('fetch', fetchMock)

    const response = await prepareBilingualReplyFlow('mail_message:1', {
      reply_text_ru: 'Спасибо.',
      tone: 'business'
    })

    expect(response.reply.text).toBe('Спасибо.')
    expect(fetchMock).toHaveBeenCalledOnce()
    const [url, init] = fetchMock.mock.calls[0]
    expect(url).toBe(
      'http://127.0.0.1:8080/api/v1/communications/messages/mail_message%3A1/bilingual-reply-flow'
    )
    expect(init.method).toBe('POST')
    expect(init.headers['X-Макошь-Secret']).toBe('test-secret')
    expect(JSON.parse(init.body as string)).toEqual({
      reply_text_ru: 'Спасибо.',
      tone: 'business'
    })
  })
})
```

### `frontend/src/domains/communications/api/bilingualReplyFlow.ts`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/domains/communications/api/bilingualReplyFlow.ts`
- Size bytes / Размер в байтах: `538`
- Included characters / Включено символов: `538`
- Truncated / Обрезано: `no`

```typescript
import { ApiClient } from '../../../platform/api/ApiClient'
import type {
  BilingualReplyFlowRequest,
  BilingualReplyFlowResponse
} from '../types/bilingualReplyFlow'

export async function prepareBilingualReplyFlow(
  messageId: string,
  request: BilingualReplyFlowRequest
): Promise<BilingualReplyFlowResponse> {
  return ApiClient.instance.post<BilingualReplyFlowResponse>(
    `/api/v1/communications/messages/${encodeURIComponent(messageId)}/bilingual-reply-flow`,
    request,
    'Bilingual reply flow preparation failed'
  )
}
```

### `frontend/src/domains/communications/api/callApi.test.ts`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/domains/communications/api/callApi.test.ts`
- Size bytes / Размер в байтах: `1279`
- Included characters / Включено символов: `1279`
- Truncated / Обрезано: `no`

```typescript
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { ApiClient } from '../../../platform/api/ApiClient'
import { fetchProviderCallTranscript, fetchProviderCalls } from './callApi'

describe('communications call API', () => {
  beforeEach(() => {
    ApiClient.resetForTests()
    ApiClient.init('http://127.0.0.1:8080', 'test-secret')
  })

  afterEach(() => {
    vi.unstubAllGlobals()
    ApiClient.resetForTests()
  })

  it('uses provider-neutral calls routes', async () => {
    const ok = (body: unknown) =>
      new Response(JSON.stringify(body), {
        status: 200,
        headers: { 'Content-Type': 'application/json' },
      })
    const fetchMock = vi
      .fn()
      .mockResolvedValueOnce(ok({ items: [] }))
      .mockResolvedValueOnce(ok({ transcript: null }))
    vi.stubGlobal('fetch', fetchMock)

    await fetchProviderCalls(' zoom-live-1 ', 12, 'zoom')
    await fetchProviderCallTranscript('call-1')

    expect(fetchMock.mock.calls[0][0]).toContain('/api/v1/calls?limit=12&account_id=zoom-live-1&provider=zoom')
    expect(fetchMock.mock.calls[1][0]).toContain('/api/v1/calls/call-1/transcript')
    expect(fetchMock.mock.calls[0][1].method).toBe('GET')
    expect(fetchMock.mock.calls[1][1].method).toBe('GET')
  })
})
```

### `frontend/src/domains/communications/api/callApi.ts`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/domains/communications/api/callApi.ts`
- Size bytes / Размер в байтах: `951`
- Included characters / Включено символов: `951`
- Truncated / Обрезано: `no`

```typescript
import { ApiClient } from '../../../platform/api/ApiClient'
import type {
  ProviderCallListResponse,
  ProviderCallTranscriptResponse,
} from '../types/communications'

export async function fetchProviderCalls(
  accountId?: string,
  limit = 50,
  provider?: string
): Promise<ProviderCallListResponse> {
  const params = new URLSearchParams()
  params.set('limit', String(limit))
  if (accountId?.trim()) params.set('account_id', accountId.trim())
  if (provider?.trim()) params.set('provider', provider.trim())
  return ApiClient.instance.get<ProviderCallListResponse>(
    `/api/v1/calls?${params.toString()}`,
    'Provider calls request failed'
  )
}

export async function fetchProviderCallTranscript(
  callId: string
): Promise<ProviderCallTranscriptResponse> {
  return ApiClient.instance.get<ProviderCallTranscriptResponse>(
    `/api/v1/calls/${encodeURIComponent(callId)}/transcript`,
    'Provider call transcript request failed'
  )
}
```

### `frontend/src/domains/communications/api/certificateApi.ts`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/domains/communications/api/certificateApi.ts`
- Size bytes / Размер в байтах: `1041`
- Included characters / Включено символов: `1041`
- Truncated / Обрезано: `no`

```typescript
import { ApiClient } from '../../../platform/api/ApiClient'
import type {
  MailCertificate,
  MailCertificateCreateRequest,
  MailCertificateListResponse
} from '../types/certificates'

export async function fetchMailCertificates(): Promise<MailCertificateListResponse> {
  return ApiClient.instance.get<MailCertificateListResponse>(
    '/api/v1/communications/certificates',
    'Mail certificates request failed'
  )
}

export async function fetchExpiringMailCertificates(days = 90): Promise<MailCertificateListResponse> {
  const safeDays = Math.min(Math.max(Math.trunc(days), 1), 3650)
  return ApiClient.instance.get<MailCertificateListResponse>(
    `/api/v1/communications/certificates/expiring?days=${safeDays}`,
    'Expiring mail certificates request failed'
  )
}

export async function createMailCertificate(
  request: MailCertificateCreateRequest
): Promise<MailCertificate> {
  return ApiClient.instance.post<MailCertificate>(
    '/api/v1/communications/certificates',
    request,
    'Mail certificate save failed'
  )
}
```

### `frontend/src/domains/communications/api/communications.test.ts`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/domains/communications/api/communications.test.ts`
- Size bytes / Размер в байтах: `45949`
- Included characters / Включено символов: `12000`
- Truncated / Обрезано: `yes`

```typescript
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { ApiClient } from '../../../platform/api/ApiClient'
import {
  analyzeMessage,
  addMessageLabel,
  bulkMessageAction,
  createDraft,
  createSavedSearch,
  detectMessageLanguage,
  deleteMessageFromProvider,
  deleteDraft,
  deleteSavedSearch,
  deleteRichTemplate,
  extractMessageNotes,
  extractMessageTasks,
  fetchCommunicationMessages,
  fetchDrafts,
  fetchCommunicationBlockers,
  fetchPersonas,
  fetchMessageAuth,
  fetchMessageExplain,
  fetchMailboxHealth,
  fetchMessageSignature,
  fetchMessageSmartCc,
  fetchMessageStateCounts,
  generateAiReply,
  generateAiReplyVariants,
  fetchSubscriptions,
  fetchTopSenders,
  fetchRichTemplates,
  fetchOutboxItems,
  runWorkflowAction,
  searchEmails,
  sendEmail,
  exportMessage,
  fetchThreadMessages,
  fetchThreads,
  previewRichTemplateMailMerge,
  renderRichTemplate,
  restoreMessage,
  snoozeMessage,
  markMessageRead,
  toggleMessageImportant,
  toggleMessageMute,
  toggleMessagePin,
  translateMessage,
  saveRichTemplate,
  fetchSavedSearches,
  transitionMessageWorkflowState,
  trashMessage,
  undoOutboxItem,
  updateSavedSearch
} from './communications'

describe('communications API', () => {
  beforeEach(() => {
    ApiClient.resetForTests()
    ApiClient.init('http://127.0.0.1:8080', 'test-secret')
  })

  afterEach(() => {
    vi.unstubAllGlobals()
    ApiClient.resetForTests()
  })

  it('passes cursor pagination parameters to the mail messages endpoint', async () => {
    const fetchMock = vi.fn().mockResolvedValue(
      new Response(JSON.stringify({
        items: [],
        nextCursor: 'next-cursor',
        hasMore: true
      }), {
        status: 200,
        headers: { 'Content-Type': 'application/json' }
      })
    )
    vi.stubGlobal('fetch', fetchMock)

    const response = await fetchCommunicationMessages(
      'account-1',
      'new',
      'email',
      'quarterly update',
      'active',
      50,
      'cursor:value'
    )

    expect(response.next_cursor).toBe('next-cursor')
    expect(response.has_more).toBe(true)
    expect(fetchMock).toHaveBeenCalledOnce()
    const [url] = fetchMock.mock.calls[0]
    expect(url).toBe('http://127.0.0.1:8080/makosh.communications.v1.CommunicationsService/ListMessages')
    expect(JSON.parse(decodeBody(fetchMock.mock.calls[0][1].body))).toMatchObject({
      accountId: 'account-1',
      workflowState: 'new',
      channelKind: 'email',
      query: 'quarterly update',
      localState: 'active',
      limit: 50,
      cursor: 'cursor:value'
    })
  })

  it('routes workflow transition, state counts and search through CommunicationsService', async () => {
    const fetchMock = vi
      .fn()
      .mockResolvedValueOnce(
        new Response(JSON.stringify({
          messageId: 'msg-1',
          workflowState: 'reviewed',
          previousState: 'new'
        }), {
          status: 200,
          headers: { 'Content-Type': 'application/json' }
        })
      )
      .mockResolvedValueOnce(
        new Response(JSON.stringify({
          counts: [{ state: 'reviewed', count: 3 }]
        }), {
          status: 200,
          headers: { 'Content-Type': 'application/json' }
        })
      )
      .mockResolvedValueOnce(
        new Response(JSON.stringify({
          results: [{ objectId: 'msg-1', objectKind: 'communication_message', title: 'Result' }]
        }), {
          status: 200,
          headers: { 'Content-Type': 'application/json' }
        })
      )
    vi.stubGlobal('fetch', fetchMock)

    const transition = await transitionMessageWorkflowState('msg-1', 'reviewed')
    const counts = await fetchMessageStateCounts('account-1', 'active')
    const search = await searchEmails('invoice', 12)

    expect(transition.previous_state).toBe('new')
    expect(counts.counts[0]).toEqual({ state: 'reviewed', count: 3 })
    expect(search.results[0].object_kind).toBe('communication_message')
    expect(fetchMock.mock.calls[0][0]).toBe(
      'http://127.0.0.1:8080/makosh.communications.v1.CommunicationsService/TransitionMessageWorkflowState'
    )
    expect(fetchMock.mock.calls[1][0]).toBe(
      'http://127.0.0.1:8080/makosh.communications.v1.CommunicationsService/ListMessageWorkflowStateCounts'
    )
    expect(fetchMock.mock.calls[2][0]).toBe(
      'http://127.0.0.1:8080/makosh.communications.v1.CommunicationsService/SearchMessages'
    )
  })

  it('routes message language, translation and extraction through CommunicationsService', async () => {
    const fetchMock = vi
      .fn()
      .mockResolvedValueOnce(
        new Response(JSON.stringify({
          language: 'es',
          confidence: 0.88,
          script: 'latin'
        }), {
          status: 200,
          headers: { 'Content-Type': 'application/json' }
        })
      )
      .mockResolvedValueOnce(
        new Response(JSON.stringify({
          translated: false,
          target: 'en',
          reason: 'no LLM configured'
        }), {
          status: 200,
          headers: { 'Content-Type': 'application/json' }
        })
      )
      .mockResolvedValueOnce(
        new Response(JSON.stringify({
          tasks: [{
            title: 'Reply by Friday',
            dueDate: 'Friday',
            assignee: null,
            priority: 'normal',
            source: 'heuristic'
          }]
        }), {
          status: 200,
          headers: { 'Content-Type': 'application/json' }
        })
      )
      .mockResolvedValueOnce(
        new Response(JSON.stringify({
          notes: [{
            title: 'Contract update',
            content: 'Contains legal review items',
            tags: ['legal'],
            source: 'heuristic'
          }]
        }), {
          status: 200,
          headers: { 'Content-Type': 'application/json' }
        })
      )
    vi.stubGlobal('fetch', fetchMock)

    const language = await detectMessageLanguage('msg-1')
    const translation = await translateMessage('msg-1', 'en')
    const tasks = await extractMessageTasks('msg-1')
    const notes = await extractMessageNotes('msg-1')

    expect(language.language).toBe('es')
    expect(translation.reason).toBe('no LLM configured')
    expect(tasks.tasks[0].title).toBe('Reply by Friday')
    expect(notes.notes[0].tags).toEqual(['legal'])
    expect(fetchMock.mock.calls[0][0]).toBe(
      'http://127.0.0.1:8080/makosh.communications.v1.CommunicationsService/DetectMessageLanguage'
    )
    expect(fetchMock.mock.calls[1][0]).toBe(
      'http://127.0.0.1:8080/makosh.communications.v1.CommunicationsService/TranslateMessage'
    )
    expect(fetchMock.mock.calls[2][0]).toBe(
      'http://127.0.0.1:8080/makosh.communications.v1.CommunicationsService/ExtractMessageTasks'
    )
    expect(fetchMock.mock.calls[3][0]).toBe(
      'http://127.0.0.1:8080/makosh.communications.v1.CommunicationsService/ExtractMessageNotes'
    )
  })

  it('routes analyze, explain and smart cc through CommunicationsService', async () => {
    const fetchMock = vi
      .fn()
      .mockResolvedValueOnce(
        new Response(JSON.stringify({
          messageId: 'msg-1',
          analyzed: true,
          category: 'follow_up',
          summary: 'Needs a reply',
          summaryContract: {
            keyPoints: ['Asks for updated contract'],
            actionItems: ['Reply this week'],
            risks: [],
            deadlines: ['2026-06-30'],
            eventCandidates: [],
            personaCandidates: [{ title: 'Alice', evidence: 'from Alice <alice@example.com>' }],
            organizationCandidates: [],
            documentCandidates: [],
            agreementCandidates: []
          },
          importanceScore: 81,
          workflowState: 'needs_action',
          source: 'local_heuristic',
          evidence: ['Contains request']
        }), {
          status: 200,
          headers: { 'Content-Type': 'application/json' }
        })
      )
      .mockResolvedValueOnce(
        new Response(JSON.stringify({
          reasons: ['Contains request']
        }), {
          status: 200,
          headers: { 'Content-Type': 'application/json' }
        })
      )
      .mockResolvedValueOnce(
        new Response(JSON.stringify({
          suggestions: ['sales@example.com']
        }), {
          status: 200,
          headers: { 'Content-Type': 'application/json' }
        })
      )
    vi.stubGlobal('fetch', fetchMock)

    const analyzed = await analyzeMessage('msg-1')
    const explained = await fetchMessageExplain('msg-1')
    const smartCc = await fetchMessageSmartCc('msg-1')

    expect(analyzed.summary_contract.action_items).toEqual(['Reply this week'])
    expect(analyzed.summary_contract.persona_candidates[0]).toEqual({
      title: 'Alice',
      evidence: 'from Alice <alice@example.com>'
    })
    expect(explained.reasons).toEqual(['Contains request'])
    expect(smartCc.suggestions).toEqual(['sales@example.com'])
    expect(fetchMock.mock.calls[0][0]).toBe(
      'http://127.0.0.1:8080/makosh.communications.v1.CommunicationsService/AnalyzeMessage'
    )
    expect(fetchMock.mock.calls[1][0]).toBe(
      'http://127.0.0.1:8080/makosh.communications.v1.CommunicationsService/GetMessageExplain'
    )
    expect(fetchMock.mock.calls[2][0]).toBe(
      'http://127.0.0.1:8080/makosh.communications.v1.CommunicationsService/GetMessageSmartCc'
    )
  })

  it('routes export, auth and signature through CommunicationsService', async () => {
    const fetchMock = vi
      .fn()
      .mockResolvedValueOnce(
        new Response(JSON.stringify({
          contentType: 'application/json',
          content: '{"message_id":"msg-1"}',
          filename: 'message_msg-1.json'
        }), {
          status: 200,
          headers: { 'Content-Type': 'application/json' }
        })
      )
      .mockResolvedValueOnce(
        new Response(JSON.stringify({
          auth: {
            spf: { result: 'pass', domain: 'alice@example.com' },
            rawHeaders: ['Received-SPF: pass']
          },
          risk: {
            hasSpf: true,
            spfPass: true,
            hasDkim: false,
            dkimPass: false,
            hasDmarc: false,
            dmarcPass: false,
            isSpoofed: false,
            riskSummary: 'Authentication checks passed'
          }
        }), {
          status: 200,
          headers: { 'Content-Type': 'application/json' }
        })
      )
      .mockResolvedValueOnce(
        new Response(JSON.stringify({
          hasSignature: false
        }), {
          status: 200,
          headers: { 'Content-Type': 'application/json' }
        })
      )
    vi.stubGlobal('fetch', fetchMock)

    const exported = await exportMessage('msg-1', 'json')
    const auth = await fetchMessageAuth('msg-1')
    const signature = await fetchMessageSignature('msg-1')

    expect(exported.filename).toBe('message_msg-1.json')
    expect(auth.auth.spf?.domain).toBe('alice@example.com')
    expect(signature.has_signature).toBe(false)
    expect(fetchMock.mock.calls[0][0]).toBe(
      'http://127.0.0.1:8080/makosh.communications.v1.CommunicationsService/GetMessageExport'
    )
    expect(fetchMock.mock.calls[1][0]).toBe(
      'http://127.0.0.1:8080/makosh.communications.v1.CommunicationsService/GetMessageAuth'
    )
    expect(fetchMock.mock.calls[2][0]).toBe(
      'http://127.0.0.1:8080/makosh.communications.v1.CommunicationsService/GetMessageSignature'
    )
  })

  it('routes ai reply and ai reply variants through CommunicationsService', async () => {
    const fetchMock = vi
      .fn()
      .mockResolvedValueOnce(
        new Response(JSON.stringify({
          subject: 'Re: Quarterly update',
          body: 'Generated reply',
          tone: 'business',
          language: 'en',
          generated: true
        }), {
          status: 200,
          headers: { 'Content-Type': 'application/json' }
        })
      )
      .mockResolvedValueOnce(
        new Response(JSON.stringi
```
_Source file truncated after 12000 characters. / Исходный файл обрезан после 12000 символов._

### `frontend/src/domains/communications/api/communications.ts`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/domains/communications/api/communications.ts`
- Size bytes / Размер в байтах: `2427`
- Included characters / Включено символов: `2427`
- Truncated / Обрезано: `no`

```typescript
export {
  fetchCommunicationMessages,
  fetchCommunicationMessage,
  transitionMessageWorkflowState,
  fetchMessageStateCounts,
  trashMessage,
  restoreMessage,
  bulkMessageAction,
  analyzeMessage,
  runWorkflowAction,
  fetchMessageExplain,
  fetchMessageSmartCc,
  markMessageRead,
  deleteMessageFromProvider,
  toggleMessagePin,
  toggleMessageImportant,
  toggleMessageMute,
  snoozeMessage,
  addMessageLabel,
  exportMessage,
  fetchMessageAuth,
  fetchMessageSignature,
  detectMessageLanguage,
  translateMessage,
  createDraft,
  deleteDraft,
  fetchDrafts,
  searchEmails,
  fetchSubscriptions,
  fetchCommunicationBlockers,
  fetchPersonas,
  fetchRichTemplates,
  saveRichTemplate,
  deleteRichTemplate,
  renderRichTemplate,
  previewRichTemplateMailMerge,
  fetchTopSenders,
  fetchMailboxHealth
} from './messageApi'
export {
  fetchSavedSearches,
  createSavedSearch,
  updateSavedSearch,
  deleteSavedSearch
} from './savedSearchApi'
export {
  fetchCommunicationFolders,
  createCommunicationFolder,
  updateCommunicationFolder,
  deleteCommunicationFolder,
  fetchFolderMessages,
  copyMessageToFolder,
  moveMessageToFolder
} from './folderApi'
export {
  fetchMailSyncStatus,
  fetchMailSyncSettings,
  updateMailSyncSettings,
  runMailSyncNow,
  runMailFullResync
} from '../../../shared/mailSync/syncApi'
export { sendEmail, redirectMessage } from './sendApi'
export {
  fetchOutboxItems,
  undoOutboxItem
} from './outboxApi'
export {
  fetchThreads,
  fetchThreadMessages,
  translateThread
} from './threadApi'
export {
  fetchProviderCalls,
  fetchProviderCallTranscript
} from './callApi'
export {
  fetchCommunicationMessagesConnect,
  fetchCommunicationMessageConnect,
  fetchCommunicationThreadsConnect,
  fetchCommunicationThreadMessagesConnect,
  fetchCommunicationDraftsConnect,
  fetchCommunicationOutboxConnect,
  sendCommunicationConnect
} from './connectCommunications'
export {
  searchAttachments,
  inspectAttachmentArchive,
  previewAttachment,
  translateAttachment
} from './attachmentApi'
export {
  createMailCertificate,
  fetchExpiringMailCertificates,
  fetchMailCertificates
} from './certificateApi'
export {
  fetchMessageAiState,
  updateMessageAiState
} from './aiState'
export {
  generateAiReply,
  generateAiReplyVariants,
  extractMessageTasks,
  extractMessageNotes
} from './messageApi'
export type { CommunicationSearchResponse } from '../types/communications'
```

### `frontend/src/domains/communications/api/communicationsAttachmentsFolders.test.ts`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/domains/communications/api/communicationsAttachmentsFolders.test.ts`
- Size bytes / Размер в байтах: `13040`
- Included characters / Включено символов: `12000`
- Truncated / Обрезано: `yes`

```typescript
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { ApiClient } from '../../../platform/api/ApiClient'
import { resetCommunicationsConnectClientForTests } from '../../../platform/connect/communicationsClient'
import {
  copyMessageToFolder,
  createCommunicationFolder,
  fetchFolderMessages,
  fetchCommunicationFolders,
  inspectAttachmentArchive,
  moveMessageToFolder,
  previewAttachment,
  redirectMessage,
  searchAttachments,
  translateAttachment,
  translateThread
} from './communications'

describe('communications API attachment and folder helpers', () => {
  beforeEach(() => {
    ApiClient.resetForTests()
    ApiClient.init('http://127.0.0.1:8080', 'test-secret')
    resetCommunicationsConnectClientForTests()
  })

  afterEach(() => {
    vi.unstubAllGlobals()
    resetCommunicationsConnectClientForTests()
    ApiClient.resetForTests()
  })

  it('searches attachment metadata with cursor filters', async () => {
    const fetchMock = vi.fn().mockResolvedValue(
      new Response(JSON.stringify({ items: [], nextCursor: null, hasMore: false }), {
        status: 200,
        headers: { 'Content-Type': 'application/json' }
      })
    )
    vi.stubGlobal('fetch', fetchMock)

    await searchAttachments({
      account_id: 'account-1',
      q: 'invoice pdf',
      content_type: 'pdf',
      scan_status: 'not_scanned',
      limit: 25,
      cursor: 'cursor:value'
    })

    expect(fetchMock).toHaveBeenCalledOnce()
    const [url, init] = fetchMock.mock.calls[0]
    expect(url).toBe(
      'http://127.0.0.1:8080/makosh.communications.v1.CommunicationsService/SearchAttachments'
    )
    expect(init.method).toBe('POST')
    expect(JSON.parse(decodeBody(init.body))).toEqual({
      accountId: 'account-1',
      query: 'invoice pdf',
      contentType: 'pdf',
      scanStatus: 'not_scanned',
      cursor: 'cursor:value',
      limit: 25
    })
  })

  it('posts attachment translation requests with provided extracted text', async () => {
    const fetchMock = vi.fn().mockResolvedValue(
      new Response(
        JSON.stringify({
          attachmentId: 'mail_attachment:1',
          messageId: 'msg-1',
          filename: 'contract.txt',
          originalLanguage: 'es',
          confidence: 0.91,
          translated: false,
          text: null,
          target: 'en',
          model: null,
          reason: 'translation runtime unavailable',
          source: 'caller_provided_extracted_text'
        }),
        {
          status: 200,
          headers: { 'Content-Type': 'application/json' }
        }
      )
    )
    vi.stubGlobal('fetch', fetchMock)

    await translateAttachment('mail_attachment:1', {
      target_language: 'en',
      source_text: 'Hola equipo'
    })

    expect(fetchMock).toHaveBeenCalledOnce()
    const [url, init] = fetchMock.mock.calls[0]
    expect(url).toBe(
      'http://127.0.0.1:8080/makosh.communications.v1.CommunicationsService/TranslateAttachment'
    )
    expect(init.method).toBe('POST')
    expect(JSON.parse(decodeBody(init.body))).toEqual({
      attachmentId: 'mail_attachment:1',
      targetLanguage: 'en',
      sourceText: 'Hola equipo'
    })
  })

  it('fetches attachment archive inspection reports by attachment id', async () => {
    const fetchMock = vi.fn().mockResolvedValue(
      new Response(
        JSON.stringify({
          attachmentId: 'mail_attachment:1',
          messageId: 'msg-1',
          filename: 'archive.zip',
          contentType: 'application/zip',
          scanStatus: 'not_scanned',
          report: {
            archiveKind: 'zip',
            entryCount: 1,
            totalUncompressedBytes: 5,
            hasNestedArchive: false,
            entries: []
          }
        }),
        {
          status: 200,
          headers: { 'Content-Type': 'application/json' }
        }
      )
    )
    vi.stubGlobal('fetch', fetchMock)

    const report = await inspectAttachmentArchive('mail_attachment:1')

    expect(report.report.archive_kind).toBe('zip')
    expect(fetchMock).toHaveBeenCalledOnce()
    const [url, init] = fetchMock.mock.calls[0]
    expect(url).toBe(
      'http://127.0.0.1:8080/makosh.communications.v1.CommunicationsService/GetAttachmentArchiveInspection'
    )
    expect(init.method).toBe('POST')
    expect(JSON.parse(decodeBody(init.body))).toEqual({
      attachmentId: 'mail_attachment:1'
    })
  })

  it('fetches safe attachment previews by attachment id', async () => {
    const fetchMock = vi.fn().mockResolvedValue(
      new Response(
        JSON.stringify({
          attachmentId: 'mail_attachment:1',
          messageId: 'msg-1',
          filename: 'notes.txt',
          contentType: 'text/plain',
          scanStatus: 'clean',
          previewKind: 'text',
          text: 'First line',
          dataUrl: null,
          truncated: false,
          byteCount: 10,
          maxPreviewBytes: 65536
        }),
        {
          status: 200,
          headers: { 'Content-Type': 'application/json' }
        }
      )
    )
    vi.stubGlobal('fetch', fetchMock)

    const preview = await previewAttachment('mail_attachment:1')

    expect(preview.preview_kind).toBe('text')
    expect(preview.text).toBe('First line')
    expect(preview.data_url).toBeNull()
    expect(fetchMock).toHaveBeenCalledOnce()
    const [url, init] = fetchMock.mock.calls[0]
    expect(url).toBe(
      'http://127.0.0.1:8080/makosh.communications.v1.CommunicationsService/GetAttachmentPreview'
    )
    expect(init.method).toBe('POST')
    expect(JSON.parse(decodeBody(init.body))).toEqual({
      attachmentId: 'mail_attachment:1'
    })
  })

  it('maps pdf attachment previews from connect responses', async () => {
    const fetchMock = vi.fn().mockResolvedValue(
      new Response(
        JSON.stringify({
          attachmentId: 'mail_attachment:pdf',
          messageId: 'msg-pdf',
          filename: 'spec.pdf',
          contentType: 'application/pdf',
          scanStatus: 'clean',
          previewKind: 'pdf',
          text: '',
          dataUrl: 'data:application/pdf;base64,JVBERi0x',
          truncated: false,
          byteCount: 8,
          maxPreviewBytes: 16777216
        }),
        {
          status: 200,
          headers: { 'Content-Type': 'application/json' }
        }
      )
    )
    vi.stubGlobal('fetch', fetchMock)

    const preview = await previewAttachment('mail_attachment:pdf')

    expect(preview.preview_kind).toBe('pdf')
    expect(preview.data_url).toBe('data:application/pdf;base64,JVBERi0x')
  })

  it('manages custom folders and local folder message actions', async () => {
    const fetchMock = vi
      .fn()
      .mockResolvedValueOnce(
        new Response(JSON.stringify({
          items: [{
            folderId: 'mail_folder:1',
            accountId: 'account-1',
            name: 'Clients',
            description: null,
            color: '#3b82f6',
            sortOrder: 10,
            messageCount: 3,
            createdAt: '2026-06-14T00:00:00Z',
            updatedAt: '2026-06-14T00:00:00Z'
          }],
          page: { nextCursor: '', hasMore: false }
        }), {
          status: 200,
          headers: { 'Content-Type': 'application/json' }
        })
      )
      .mockResolvedValueOnce(
        new Response(JSON.stringify({ item: { folderId: 'mail_folder:1', accountId: 'account-1', name: 'Clients', color: '#3b82f6', sortOrder: 10, messageCount: 0, createdAt: '2026-06-14T00:00:00Z', updatedAt: '2026-06-14T00:00:00Z' } }), {
          status: 200,
          headers: { 'Content-Type': 'application/json' }
        })
      )
      .mockResolvedValueOnce(
        new Response(JSON.stringify({ item: { operation: 'copy', folderId: 'mail_folder:1', messageId: 'msg-1', message: { folderId: 'mail_folder:1', messageId: 'msg-1', accountId: 'account-1', subject: 'Clients note', sender: 'Ada <ada@example.com>', projectedAt: '2026-06-14T00:00:00Z', workflowState: 'new', localState: 'active', addedAt: '2026-06-14T00:00:00Z', attachmentCount: 0 } } }), {
          status: 200,
          headers: { 'Content-Type': 'application/json' }
        })
      )
      .mockResolvedValueOnce(
        new Response(JSON.stringify({ item: { operation: 'move', folderId: 'mail_folder:1', messageId: 'msg-1', message: { folderId: 'mail_folder:1', messageId: 'msg-1', accountId: 'account-1', subject: 'Clients note', sender: 'Ada <ada@example.com>', projectedAt: '2026-06-14T00:00:00Z', workflowState: 'new', localState: 'active', addedAt: '2026-06-14T00:00:00Z', attachmentCount: 0 } } }), {
          status: 200,
          headers: { 'Content-Type': 'application/json' }
        })
      )
      .mockResolvedValueOnce(
        new Response(JSON.stringify({ items: [], page: { nextCursor: '', hasMore: false } }), {
          status: 200,
          headers: { 'Content-Type': 'application/json' }
        })
      )
    vi.stubGlobal('fetch', fetchMock)

    const folders = await fetchCommunicationFolders('account-1', 50)
    await createCommunicationFolder({
      name: 'Clients',
      account_id: 'account-1',
      color: '#3b82f6',
      sort_order: 10
    })
    await copyMessageToFolder('mail_folder:1', 'msg-1')
    await moveMessageToFolder('mail_folder:1', 'msg-1')
    await fetchFolderMessages('mail_folder:1', 25, 'cursor:value')

    expect(fetchMock).toHaveBeenCalledTimes(5)
    expect(folders.items[0].message_count).toBe(3)
    expect(fetchMock.mock.calls[0][0]).toBe(
      'http://127.0.0.1:8080/makosh.communications.v1.CommunicationsService/ListFolders'
    )
    expect(fetchMock.mock.calls[1][1].method).toBe('POST')
    expect(fetchMock.mock.calls[1][0]).toBe(
      'http://127.0.0.1:8080/makosh.communications.v1.CommunicationsService/CreateFolder'
    )
    expect(JSON.parse(decodeBody(fetchMock.mock.calls[1][1].body))).toEqual({
      name: 'Clients',
      accountId: 'account-1',
      color: '#3b82f6',
      sortOrder: 10
    })
    expect(fetchMock.mock.calls[2][0]).toBe(
      'http://127.0.0.1:8080/makosh.communications.v1.CommunicationsService/CopyMessageToFolder'
    )
    expect(fetchMock.mock.calls[2][1].method).toBe('POST')
    expect(fetchMock.mock.calls[3][0]).toBe(
      'http://127.0.0.1:8080/makosh.communications.v1.CommunicationsService/MoveMessageToFolder'
    )
    expect(fetchMock.mock.calls[4][0]).toBe(
      'http://127.0.0.1:8080/makosh.communications.v1.CommunicationsService/ListFolderMessages'
    )
    expect(JSON.parse(decodeBody(fetchMock.mock.calls[4][1].body))).toMatchObject({
      folderId: 'mail_folder:1',
      page: {
        limit: 25,
        cursor: 'cursor:value'
      }
    })
  })

  it('posts thread translation requests with account and subject filters', async () => {
    const fetchMock = vi.fn().mockResolvedValue(
      new Response(JSON.stringify({
        accountId: 'account-1',
        subject: 'Thread Translation',
        targetLanguage: 'en',
        items: []
      }), {
        status: 200,
        headers: { 'Content-Type': 'application/json' }
      })
    )
    vi.stubGlobal('fetch', fetchMock)

    await translateThread('account-1', 'Thread Translation', 'en')

    expect(fetchMock).toHaveBeenCalledOnce()
    const [url, init] = fetchMock.mock.calls[0]
    expect(url).toBe(
      'http://127.0.0.1:8080/makosh.communications.v1.CommunicationsService/TranslateThread'
    )
    expect(init.method).toBe('POST')
    expect(JSON.parse(decodeBody(init.body))).toEqual({
      accountId: 'account-1',
      subject: 'Thread Translation',
      targetLanguage: 'en',
      limit: 50
    })
  })

  it('posts redirect requests to the message redirect endpoint', async () => {
    const fetchMock = vi.fn().mockResolvedValue(
      new Response(JSON.stringify({
        messageId: 'outbox-1',
        outboxId: 'outbox-1',
        accepted: ['redirect@example.com'],
        acceptedRecipients: ['redirect@example.com'],
        transport: 'outbox',
        status: 'queued',
        scheduledSendAt: null
```
_Source file truncated after 12000 characters. / Исходный файл обрезан после 12000 символов._

### `frontend/src/domains/communications/api/connect/collections.ts`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/domains/communications/api/connect/collections.ts`
- Size bytes / Размер в байтах: `17818`
- Included characters / Включено символов: `12000`
- Truncated / Обрезано: `yes`

```typescript
import { getCommunicationsConnectClient } from '../../../../platform/connect/communicationsClient'
import type {
  CommunicationDraft,
  CommunicationOutboxItem,
  DraftDeleteResponse,
  DraftListResponse,
  DraftUpsertRequest,
  OutboxListResponse,
  RedirectMessageRequest,
  SendCommunicationRequest,
  SendCommunicationResponse,
  ThreadListResponse,
  ThreadMessagesResponse
} from '../../types/communications'
import type {
  AttachmentArchiveInspectionResponse,
  AttachmentPreviewResponse,
  AttachmentSearchRequest,
  AttachmentSearchResponse,
  AttachmentTranslationResponse
} from '../../types/attachments'
import type {
  CommunicationSavedSearch as DomainCommunicationSavedSearch,
  SavedSearchDeleteResponse,
  SavedSearchInput,
  SavedSearchListResponse,
  SavedSearchUpdate
} from '../../types/savedSearches'
import type {
  CommunicationFolder,
  CommunicationFolderInput,
  CommunicationFolderListResponse,
  CommunicationFolderUpdate,
  FolderDeleteResponse,
  FolderMessageActionResponse,
  FolderMessageListResponse
} from '../../types/folders'
import type { ThreadTranslationResponse } from '../../types/multilingual'
import {
  emptyStringToNull,
  mapAttachment,
  mapDraftItem,
  mapFolderItem,
  mapFolderMessageActionResult,
  mapFolderMessageItem,
  mapOutboxItem,
  mapSavedSearchItem,
  normalizeAttachmentDisposition,
  normalizeAttachmentScanStatus,
  toNumber
} from './mapping'

export async function fetchCommunicationThreadsConnect(
  accountId?: string,
  limit?: number,
  cursor?: string
): Promise<ThreadListResponse> {
  const response = await getCommunicationsConnectClient().listThreads({
    accountId,
    cursor,
    limit: limit ?? 0
  })

  return {
    items: response.items.map((item) => ({
      thread_id: item.threadId,
      account_id: item.accountId,
      subject: item.subject,
      message_count: toNumber(item.messageCount),
      participant_count: toNumber(item.participantCount),
      first_message_at: item.firstMessageAt ?? null,
      last_message_at: item.lastMessageAt ?? null,
      last_activity_at: item.lastActivityAt,
      has_open_action: item.hasOpenAction,
      has_attachments: item.hasAttachments,
      dominant_workflow_state: item.dominantWorkflowState
    })),
    next_cursor: response.nextCursor ?? null,
    has_more: response.hasMore
  }
}

export async function fetchCommunicationThreadMessagesConnect(
  accountId: string,
  subject: string,
  limit?: number
): Promise<ThreadMessagesResponse> {
  const response = await getCommunicationsConnectClient().listThreadMessages({
    accountId,
    subject,
    limit: limit ?? 0
  })

  return {
    items: response.items.map((item) => ({
      message_id: item.messageId,
      provider_record_id: item.providerRecordId,
      account_id: item.accountId,
      subject: item.subject,
      sender: item.sender,
      sender_display_name: item.senderDisplayName ?? null,
      body_text: item.bodyText,
      occurred_at: item.occurredAt ?? null,
      projected_at: item.projectedAt,
      workflow_state: item.workflowState,
      importance_score: item.importanceScore ?? null,
      ai_category: item.aiCategory ?? null,
      ai_summary: item.aiSummary ?? null,
      delivery_state: item.deliveryState,
      attachment_count: toNumber(item.attachmentCount),
      attachments: item.attachments.map(mapAttachment)
    }))
  }
}

export async function translateCommunicationThreadConnect(
  accountId: string,
  subject: string,
  targetLanguage: string,
  limit?: number
): Promise<ThreadTranslationResponse> {
  const response = await getCommunicationsConnectClient().translateThread({
    accountId,
    subject,
    targetLanguage,
    limit: limit ?? 0
  })

  return {
    account_id: response.accountId,
    subject: response.subject,
    target_language: response.targetLanguage,
    items: response.items.map((item) => ({
      message_id: item.messageId,
      original_language: item.originalLanguage,
      confidence: item.confidence,
      translated: item.translated,
      text: item.text ?? null,
      target: item.target,
      model: item.model ?? null,
      reason: item.reason ?? null
    }))
  }
}

export async function searchAttachmentsConnect(
  request: AttachmentSearchRequest = {}
): Promise<AttachmentSearchResponse> {
  const response = await getCommunicationsConnectClient().searchAttachments({
    accountId: request.account_id,
    query: request.q,
    contentType: request.content_type,
    scanStatus: request.scan_status,
    cursor: request.cursor ?? undefined,
    limit: request.limit ?? 0
  })

  return {
    items: response.items.map((item) => ({
      attachment_id: item.attachmentId,
      message_id: item.messageId,
      raw_record_id: item.rawRecordId,
      account_id: item.accountId,
      message_subject: item.messageSubject,
      sender: item.sender,
      occurred_at: item.occurredAt ?? null,
      blob_id: item.blobId,
      provider_attachment_id: item.providerAttachmentId,
      filename: item.filename ?? null,
      content_type: item.contentType,
      size_bytes: toNumber(item.sizeBytes),
      sha256: item.sha256,
      disposition: normalizeAttachmentDisposition(item.disposition),
      scan_status: normalizeAttachmentScanStatus(item.scanStatus),
      scan_engine: item.scanEngine ?? null,
      scan_checked_at: item.scanCheckedAt ?? null,
      scan_summary: item.scanSummary ?? null,
      storage_kind: item.storageKind,
      storage_path: item.storagePath,
      created_at: item.createdAt,
      updated_at: item.updatedAt
    })),
    next_cursor: response.nextCursor ?? null,
    has_more: response.hasMore
  }
}

export async function inspectAttachmentArchiveConnect(
  attachmentId: string
): Promise<AttachmentArchiveInspectionResponse> {
  const response = await getCommunicationsConnectClient().getAttachmentArchiveInspection({
    attachmentId
  })

  return {
    attachment_id: response.attachmentId,
    message_id: response.messageId,
    filename: response.filename ?? null,
    content_type: response.contentType,
    scan_status: normalizeAttachmentScanStatus(response.scanStatus),
    report: {
      archive_kind: 'zip',
      entry_count: toNumber(response.report?.entryCount ?? 0),
      total_uncompressed_bytes: toNumber(response.report?.totalUncompressedBytes ?? 0),
      has_nested_archive: response.report?.hasNestedArchive ?? false,
      entries: (response.report?.entries ?? []).map((entry) => ({
        name: entry.name,
        normalized_path: entry.normalizedPath,
        compressed_size: toNumber(entry.compressedSize),
        uncompressed_size: toNumber(entry.uncompressedSize),
        is_dir: entry.isDir,
        is_nested_archive: entry.isNestedArchive
      }))
    }
  }
}

export async function previewAttachmentConnect(
  attachmentId: string
): Promise<AttachmentPreviewResponse> {
  const response = await getCommunicationsConnectClient().getAttachmentPreview({ attachmentId })
  return {
    attachment_id: response.attachmentId,
    message_id: response.messageId,
    filename: response.filename ?? null,
    content_type: response.contentType,
    scan_status: normalizeAttachmentScanStatus(response.scanStatus),
    preview_kind:
      response.previewKind === 'image'
        ? 'image'
        : response.previewKind === 'audio'
          ? 'audio'
          : response.previewKind === 'video'
            ? 'video'
            : response.previewKind === 'pdf'
              ? 'pdf'
              : 'text',
    text: response.text,
    data_url: response.dataUrl ?? null,
    truncated: response.truncated,
    byte_count: toNumber(response.byteCount),
    max_preview_bytes: toNumber(response.maxPreviewBytes)
  }
}

export async function translateAttachmentConnect(
  attachmentId: string,
  targetLanguage: string,
  sourceText: string
): Promise<AttachmentTranslationResponse> {
  const response = await getCommunicationsConnectClient().translateAttachment({
    attachmentId,
    targetLanguage,
    sourceText
  })

  return {
    attachment_id: response.attachmentId,
    message_id: response.messageId,
    filename: response.filename ?? null,
    original_language: response.originalLanguage,
    confidence: response.confidence,
    translated: response.translated,
    text: response.text ?? null,
    target: response.target,
    model: response.model ?? null,
    reason: response.reason ?? null,
    source: 'caller_provided_extracted_text'
  }
}

export async function fetchCommunicationDraftsConnect(
  accountId?: string,
  status?: string,
  limit?: number,
  cursor?: string
): Promise<DraftListResponse> {
  const response = await getCommunicationsConnectClient().listDrafts({
    accountId,
    status,
    page: { limit: limit ?? 0, cursor: cursor ?? '' }
  })
  return {
    items: response.items.map(mapDraftItem),
    next_cursor: emptyStringToNull(response.page?.nextCursor),
    has_more: response.page?.hasMore ?? false
  }
}

export async function fetchCommunicationSavedSearchesConnect(
  smartFolder?: boolean,
  accountId?: string,
  limit?: number,
  cursor?: string
): Promise<SavedSearchListResponse> {
  const response = await getCommunicationsConnectClient().listSavedSearches({
    accountId,
    smartFolder,
    page: { limit: limit ?? 0, cursor: cursor ?? '' }
  })
  return {
    items: response.items.map(mapSavedSearchItem),
    next_cursor: emptyStringToNull(response.page?.nextCursor),
    has_more: response.page?.hasMore ?? false
  }
}

export async function createCommunicationSavedSearchConnect(
  request: SavedSearchInput
): Promise<DomainCommunicationSavedSearch> {
  const response = await getCommunicationsConnectClient().createSavedSearch({
    name: request.name,
    description: request.description ?? undefined,
    accountId: request.account_id ?? undefined,
    query: request.query ?? undefined,
    workflowState: request.workflow_state ?? undefined,
    localState: request.local_state ?? undefined,
    channelKind: request.channel_kind ?? undefined,
    isSmartFolder: request.is_smart_folder ?? undefined,
    sortOrder: request.sort_order ?? undefined
  })
  return mapSavedSearchItem(response.item)
}

export async function updateCommunicationSavedSearchConnect(
  savedSearchId: string,
  request: SavedSearchUpdate
): Promise<DomainCommunicationSavedSearch> {
  const response = await getCommunicationsConnectClient().updateSavedSearch({
    savedSearchId,
    name: request.name ?? undefined,
    description: request.description ?? undefined,
    accountId: request.account_id ?? undefined,
    query: request.query ?? undefined,
    workflowState: request.workflow_state ?? undefined,
    localState: request.local_state ?? undefined,
    channelKind: request.channel_kind ?? undefined,
    isSmartFolder: request.is_smart_folder ?? undefined,
    sortOrder: request.sort_order ?? undefined
  })
  return mapSavedSearchItem(response.item)
}

export async function deleteCommunicationSavedSearchConnect(
  savedSearchId: string
): Promise<SavedSearchDeleteResponse> {
  const response = await getCommunicationsConnectClient().deleteSavedSearch({ savedSearchId })
  return { deleted: response.deleted }
}

export async function fetchCommunicationOutboxConnect(
  accountId?: string,
  status?: string,
  limit?: number,
  cursor?: string
): Promise<OutboxListResponse> {
  const response = await getCommunicationsConnectClient().listOutbox({
    accountId,
    status,
    page: { limit: limit ?? 0, cursor: cursor ?? '' }
  })
  return {
    items: response.items.map(mapOutboxItem),
    next_cursor: emptyStringToNull(response.page?.nextCursor),
    has_more: response.page?.hasMore ?? false
  }
}

export async function fetchCommunicationFoldersConnect(
  accountId?: string,
  limit?: number,
  cursor?: string
): Promise<CommunicationFolderListResponse> {
  const response = await getCommunicationsConnectClient().listFolders({
    accountId,
    page: { limit: limit ?? 0, cursor: cursor ?? '' }
  })
  return {
    items: response.items.map
```
_Source file truncated after 12000 characters. / Исходный файл обрезан после 12000 символов._

### `frontend/src/domains/communications/api/connect/insights.ts`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/domains/communications/api/connect/insights.ts`
- Size bytes / Размер в байтах: `9336`
- Included characters / Включено символов: `9336`
- Truncated / Обрезано: `no`

```typescript
import { getCommunicationsConnectClient } from '../../../../platform/connect/communicationsClient'
import type {
  CommunicationArchitectureBlocker,
  CommunicationPersona,
  CommunicationSearchResponse,
  CommunicationTemplate,
  LanguageDetection,
  MailboxHealth,
  RichTemplateDeleteResponse,
  RichTemplateMailMergePreviewRequest,
  RichTemplateMailMergePreviewResponse,
  RichTemplateRenderRequest,
  RichTemplateRenderResponse,
  RichTemplateUpsertRequest,
  RichTemplateUpsertResponse,
  SenderStatsListResponse,
  SubscriptionListResponse,
  TranslationResponse,
  WorkflowStateCountsResponse
} from '../../types/communications'
import { mapRichTemplate, parseJsonObject, toNumber } from './mapping'
import { postCommunicationsConnectJson } from './shared'

export async function fetchMessageStateCountsConnect(
  accountId?: string,
  localState?: string
): Promise<WorkflowStateCountsResponse> {
  const response = await getCommunicationsConnectClient().listMessageWorkflowStateCounts({
    accountId,
    localState
  })

  return {
    counts: response.counts.map((item) => ({
      state: item.state,
      count: toNumber(item.count)
    }))
  }
}

export async function fetchSubscriptionsConnect(
  accountId?: string,
  limit?: number,
  cursor?: string
): Promise<SubscriptionListResponse> {
  const response = await getCommunicationsConnectClient().listSubscriptions({
    accountId,
    cursor,
    limit: limit ?? 0
  })

  return {
    items: response.items.map((item) => ({
      sender: item.sender,
      message_count: toNumber(item.messageCount),
      first_seen: item.firstSeen,
      last_seen: item.lastSeen,
      is_newsletter: item.isNewsletter,
      has_unsubscribe: item.hasUnsubscribe
    })),
    next_cursor: response.nextCursor ?? null,
    has_more: response.hasMore
  }
}

export async function fetchMailboxHealthConnect(accountId?: string): Promise<MailboxHealth> {
  const response = await getCommunicationsConnectClient().getMailboxHealth({ accountId })
  return {
    total_messages: toNumber(response.item?.totalMessages ?? 0),
    unread: toNumber(response.item?.unread ?? 0),
    needs_action: toNumber(response.item?.needsAction ?? 0),
    waiting: toNumber(response.item?.waiting ?? 0),
    done: toNumber(response.item?.done ?? 0),
    archived: toNumber(response.item?.archived ?? 0),
    spam: toNumber(response.item?.spam ?? 0),
    important: toNumber(response.item?.important ?? 0),
    with_attachments: toNumber(response.item?.withAttachments ?? 0),
    average_importance: response.item?.averageImportance ?? 0,
    oldest_message_days: response.item?.oldestMessageDays ?? null
  }
}

export async function fetchTopSendersConnect(
  accountId?: string,
  limit?: number,
  cursor?: string
): Promise<SenderStatsListResponse> {
  const response = await getCommunicationsConnectClient().listTopSenders({
    accountId,
    cursor,
    limit: limit ?? 0
  })

  return {
    items: response.items.map((item) => ({
      sender: item.sender,
      message_count: toNumber(item.messageCount),
      avg_importance: item.avgImportance,
      last_message_days: item.lastMessageDays ?? null
    })),
    next_cursor: response.nextCursor ?? null,
    has_more: response.hasMore
  }
}

export async function fetchCommunicationBlockersConnect(): Promise<CommunicationArchitectureBlocker[]> {
  const response = await getCommunicationsConnectClient().listCommunicationBlockers({})
  return response.items.map((item) => ({
    section: item.section,
    feature: item.feature,
    reason: item.reason,
    resolution: item.resolution
  }))
}

export async function fetchCommunicationPersonasConnect(): Promise<{ items: CommunicationPersona[] }> {
  const response = await postCommunicationsConnectJson<{
    items: Array<{
      personaId: string
      accountId: string
      name: string
      displayName: string
      signature: string
      defaultLanguage?: string
      defaultTone?: string
      isDefault: boolean
      metadataJson: string
      createdAt: string
      updatedAt: string
    }>
  }>('ListCommunicationPersonas', {})

  return {
    items: response.items.map((item) => ({
      persona_id: item.personaId,
      account_id: item.accountId,
      name: item.name,
      display_name: item.displayName,
      signature: item.signature,
      default_language: item.defaultLanguage ?? null,
      default_tone: item.defaultTone ?? null,
      is_default: item.isDefault,
      metadata: parseJsonObject(item.metadataJson),
      created_at: item.createdAt,
      updated_at: item.updatedAt
    }))
  }
}

export async function fetchRichTemplatesConnect(): Promise<{ templates: CommunicationTemplate[] }> {
  const response = await postCommunicationsConnectJson<{
    templates: Array<Parameters<typeof mapRichTemplate>[0]>
  }>('ListRichTemplates', {})
  return { templates: response.templates.map(mapRichTemplate) }
}

export async function saveRichTemplateConnect(
  request: RichTemplateUpsertRequest
): Promise<RichTemplateUpsertResponse> {
  const response = await postCommunicationsConnectJson<{
    saved: boolean
    template?: Parameters<typeof mapRichTemplate>[0]
  }>('UpsertRichTemplate', {
    templateId: request.template_id ?? undefined,
    name: request.name,
    subjectTemplate: request.subject_template,
    bodyTemplate: request.body_template,
    variables: request.variables,
    language: request.language ?? ''
  })

  return {
    saved: response.saved,
    template: mapRichTemplate(response.template)
  }
}

export async function deleteRichTemplateConnect(
  templateId: string
): Promise<RichTemplateDeleteResponse> {
  const response = await postCommunicationsConnectJson<{ templateId: string; deleted: boolean }>(
    'DeleteRichTemplate',
    { templateId }
  )
  return { template_id: response.templateId, deleted: response.deleted }
}

export async function renderRichTemplateConnect(
  request: RichTemplateRenderRequest
): Promise<RichTemplateRenderResponse> {
  const response = await postCommunicationsConnectJson<{
    templateId: string
    variables: Record<string, string>
    rendered?: {
      subject: string
      body: string
      missingVariables: string[]
      unresolvedVariables: string[]
      malformedPlaceholders: string[]
    }
  }>('RenderRichTemplate', {
    templateId: request.template_id,
    variables: request.variables
  })

  return {
    template_id: response.templateId,
    variables: response.variables,
    rendered: {
      subject: response.rendered?.subject ?? '',
      body: response.rendered?.body ?? '',
      missing_variables: response.rendered?.missingVariables ?? [],
      unresolved_variables: response.rendered?.unresolvedVariables ?? [],
      malformed_placeholders: response.rendered?.malformedPlaceholders ?? []
    }
  }
}

export async function previewRichTemplateMailMergeConnect(
  request: RichTemplateMailMergePreviewRequest
): Promise<RichTemplateMailMergePreviewResponse> {
  const response = await postCommunicationsConnectJson<{
    templateId: string
    rowCount: number | bigint
    readyCount: number | bigint
    blockedCount: number | bigint
    items: Array<{
      rowId: string
      ready: boolean
      rendered?: {
        subject: string
        body: string
        missingVariables: string[]
        unresolvedVariables: string[]
        malformedPlaceholders: string[]
      }
    }>
  }>('PreviewRichTemplateMailMerge', {
    templateId: request.template_id,
    rows: request.rows.map((row) => ({ rowId: row.row_id, variables: row.variables }))
  })

  return {
    template_id: response.templateId,
    row_count: toNumber(response.rowCount),
    ready_count: toNumber(response.readyCount),
    blocked_count: toNumber(response.blockedCount),
    items: response.items.map((item) => ({
      row_id: item.rowId,
      ready: item.ready,
      rendered: {
        subject: item.rendered?.subject ?? '',
        body: item.rendered?.body ?? '',
        missing_variables: item.rendered?.missingVariables ?? [],
        unresolved_variables: item.rendered?.unresolvedVariables ?? [],
        malformed_placeholders: item.rendered?.malformedPlaceholders ?? []
      }
    }))
  }
}

export async function searchMessagesConnect(
  query: string,
  limit?: number
): Promise<CommunicationSearchResponse> {
  const response = await getCommunicationsConnectClient().searchMessages({ query, limit: limit ?? 0 })
  return {
    results: response.results.map((item) => ({
      object_id: item.objectId,
      object_kind: item.objectKind,
      title: item.title
    }))
  }
}

export async function detectMessageLanguageConnect(messageId: string): Promise<LanguageDetection> {
  const response = await getCommunicationsConnectClient().detectMessageLanguage({ messageId })
  return {
    language: response.language,
    confidence: response.confidence,
    script: response.script ?? null
  }
}

export async function translateMessageConnect(
  messageId: string,
  targetLanguage: string
): Promise<TranslationResponse> {
  const response = await getCommunicationsConnectClient().translateMessage({
    messageId,
    targetLanguage
  })
  return {
    translated: response.translated,
    text: response.text ?? undefined,
    target: response.target ?? undefined,
    model: response.model ?? undefined,
    reason: response.reason ?? undefined
  }
}
```

### `frontend/src/domains/communications/api/connect/mapping.ts`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/domains/communications/api/connect/mapping.ts`
- Size bytes / Размер в байтах: `15344`
- Included characters / Включено символов: `12000`
- Truncated / Обрезано: `yes`

```typescript
import type {
  BulkMessageActionResponse,
  CommunicationAttachment,
  CommunicationDraft,
  CommunicationKnowledgeCandidate,
  CommunicationMessageSummary,
  CommunicationOutboxItem,
  CommunicationTemplate,
  DraftListResponse,
  MessageAnalyzeResponse
} from '../../types/communications'
import type { AttachmentScanStatus, AttachmentSearchResponse } from '../../types/attachments'
import type { CommunicationSavedSearch as DomainCommunicationSavedSearch } from '../../types/savedSearches'
import type { CommunicationFolder, FolderMessage, FolderMessageActionResponse } from '../../types/folders'

export function mapMessageSummary(item: {
  messageId: string
  rawRecordId: string
  observationId?: string
  accountId: string
  providerRecordId: string
  subject: string
  sender: string
  recipients: string[]
  bodyText: string
  occurredAt?: string
  projectedAt: string
  channelKind: string
  conversationId?: string
  senderDisplayName?: string
  deliveryState: string
  messageMetadataJson: string
  workflowState: string
  importanceScore?: number
  aiCategory?: string
  aiSummary?: string
  aiSummaryGeneratedAt?: string
  localState: string
  localStateChangedAt?: string
  attachmentCount: number | bigint
}): CommunicationMessageSummary {
  return {
    message_id: item.messageId,
    raw_record_id: item.rawRecordId,
    observation_id: item.observationId ?? null,
    account_id: item.accountId,
    provider_record_id: item.providerRecordId,
    subject: item.subject,
    sender: item.sender,
    recipients: item.recipients,
    body_text_preview: textPreview(item.bodyText, 240),
    occurred_at: item.occurredAt ?? null,
    projected_at: item.projectedAt,
    channel_kind: item.channelKind,
    conversation_id: item.conversationId ?? null,
    sender_display_name: item.senderDisplayName ?? null,
    delivery_state: item.deliveryState,
    workflow_state: normalizeWorkflowState(item.workflowState),
    importance_score: item.importanceScore ?? null,
    ai_category: item.aiCategory ?? null,
    ai_summary: item.aiSummary ?? null,
    ai_summary_generated_at: item.aiSummaryGeneratedAt ?? null,
    message_metadata: parseJsonObject(item.messageMetadataJson),
    attachment_count: toNumber(item.attachmentCount),
    local_state: normalizeLocalState(item.localState),
    local_state_changed_at: item.localStateChangedAt ?? null
  }
}

export function mapAttachment(item: {
  attachmentId: string
  messageId: string
  rawRecordId: string
  blobId: string
  providerAttachmentId: string
  filename?: string
  contentType: string
  sizeBytes: number | bigint
  sha256: string
  disposition: string
  scanStatus: string
  scanEngine?: string
  scanCheckedAt?: string
  scanSummary?: string
  scanMetadataJson: string
  storageKind: string
  storagePath: string
  createdAt: string
  updatedAt: string
}): CommunicationAttachment {
  return {
    attachment_id: item.attachmentId,
    message_id: item.messageId,
    raw_record_id: item.rawRecordId,
    blob_id: item.blobId,
    provider_attachment_id: item.providerAttachmentId,
    filename: item.filename ?? null,
    content_type: item.contentType,
    size_bytes: toNumber(item.sizeBytes),
    sha256: item.sha256,
    disposition: normalizeDisposition(item.disposition),
    scan_status: normalizeScanStatus(item.scanStatus),
    scan_engine: item.scanEngine ?? null,
    scan_checked_at: item.scanCheckedAt ?? null,
    scan_summary: item.scanSummary ?? null,
    scan_metadata: parseJsonObject(item.scanMetadataJson),
    storage_kind: item.storageKind,
    storage_path: item.storagePath,
    created_at: item.createdAt,
    updated_at: item.updatedAt
  }
}

export function mapOutboxItem(item: {
  outboxId: string
  accountId: string
  draftId?: string
  toRecipients: string[]
  ccRecipients: string[]
  bccRecipients: string[]
  subject: string
  bodyText: string
  bodyHtml?: string
  status: string
  scheduledSendAt?: string
  undoDeadlineAt?: string
  sendAttempts: number
  claimedAt?: string
  sentAt?: string
  providerMessageId?: string
  lastError?: string
  metadataJson: string
  createdAt: string
  updatedAt: string
} | undefined): CommunicationOutboxItem {
  if (!item) {
    throw new Error('CommunicationsService returned an empty outbox item')
  }
  return {
    outbox_id: item.outboxId,
    account_id: item.accountId,
    draft_id: item.draftId ?? null,
    to_recipients: item.toRecipients,
    cc_recipients: item.ccRecipients,
    bcc_recipients: item.bccRecipients,
    subject: item.subject,
    body_text: item.bodyText,
    body_html: item.bodyHtml ?? null,
    status: normalizeOutboxStatus(item.status),
    scheduled_send_at: item.scheduledSendAt ?? null,
    undo_deadline_at: item.undoDeadlineAt ?? null,
    send_attempts: item.sendAttempts,
    claimed_at: item.claimedAt ?? null,
    sent_at: item.sentAt ?? null,
    provider_message_id: item.providerMessageId ?? null,
    last_error: item.lastError ?? null,
    metadata: parseJsonObject(item.metadataJson),
    created_at: item.createdAt,
    updated_at: item.updatedAt
  }
}

export function mapDraftItem(item: {
  draftId: string
  accountId: string
  personaId?: string
  toRecipients: string[]
  ccRecipients: string[]
  bccRecipients: string[]
  subject: string
  bodyText: string
  bodyHtml?: string
  inReplyTo?: string
  references: string[]
  status: string
  scheduledSendAt?: string
  sendAttempts: number
  lastError?: string
  metadataJson: string
  createdAt: string
  updatedAt: string
} | undefined): CommunicationDraft {
  if (!item) {
    throw new Error('CommunicationsService returned an empty draft item')
  }
  return {
    draft_id: item.draftId,
    account_id: item.accountId,
    persona_id: item.personaId ?? null,
    to_recipients: item.toRecipients,
    cc_recipients: item.ccRecipients,
    bcc_recipients: item.bccRecipients,
    subject: item.subject,
    body_text: item.bodyText,
    body_html: item.bodyHtml ?? null,
    in_reply_to: item.inReplyTo ?? null,
    references: item.references,
    status: normalizeDraftStatus(item.status),
    scheduled_send_at: item.scheduledSendAt ?? null,
    send_attempts: item.sendAttempts,
    last_error: item.lastError ?? null,
    metadata: parseJsonObject(item.metadataJson),
    created_at: item.createdAt,
    updated_at: item.updatedAt
  }
}

export function mapSavedSearchItem(item: {
  savedSearchId: string
  name: string
  description?: string
  accountId?: string
  query: string
  workflowState?: string
  localState: string
  channelKind?: string
  isSmartFolder: boolean
  sortOrder: number
  messageCount: number | bigint
  createdAt: string
  updatedAt: string
} | undefined): DomainCommunicationSavedSearch {
  if (!item) {
    throw new Error('CommunicationsService returned an empty saved search item')
  }
  return {
    saved_search_id: item.savedSearchId,
    name: item.name,
    description: item.description ?? null,
    account_id: item.accountId ?? null,
    query: item.query,
    workflow_state: item.workflowState ? normalizeWorkflowState(item.workflowState) : null,
    local_state: normalizeLocalState(item.localState),
    channel_kind: item.channelKind ?? null,
    is_smart_folder: item.isSmartFolder,
    sort_order: item.sortOrder,
    message_count: toNumber(item.messageCount),
    created_at: item.createdAt,
    updated_at: item.updatedAt
  }
}

export function mapFolderItem(item: {
  folderId: string
  accountId?: string
  name: string
  description?: string
  color?: string
  sortOrder: number
  messageCount: number | bigint
  createdAt: string
  updatedAt: string
} | undefined): CommunicationFolder {
  if (!item) {
    throw new Error('CommunicationsService returned an empty folder item')
  }
  return {
    folder_id: item.folderId,
    account_id: item.accountId ?? null,
    name: item.name,
    description: item.description ?? null,
    color: item.color ?? null,
    sort_order: item.sortOrder,
    message_count: toNumber(item.messageCount),
    created_at: item.createdAt,
    updated_at: item.updatedAt
  }
}

export function mapFolderMessageItem(item: {
  folderId: string
  messageId: string
  accountId: string
  subject: string
  sender: string
  occurredAt?: string
  projectedAt: string
  workflowState: string
  localState: string
  addedAt: string
  attachmentCount: number | bigint
} | undefined): FolderMessage {
  if (!item) {
    throw new Error('CommunicationsService returned an empty folder message item')
  }
  return {
    folder_id: item.folderId,
    message_id: item.messageId,
    account_id: item.accountId,
    subject: item.subject,
    sender: item.sender,
    occurred_at: item.occurredAt ?? null,
    projected_at: item.projectedAt,
    workflow_state: normalizeWorkflowState(item.workflowState),
    local_state: normalizeLocalState(item.localState),
    added_at: item.addedAt,
    attachment_count: toNumber(item.attachmentCount)
  }
}

export function mapFolderMessageActionResult(item: {
  operation: string
  folderId: string
  messageId: string
  message?: {
    folderId: string
    messageId: string
    accountId: string
    subject: string
    sender: string
    occurredAt?: string
    projectedAt: string
    workflowState: string
    localState: string
    addedAt: string
    attachmentCount: number | bigint
  }
} | undefined): FolderMessageActionResponse {
  if (!item) {
    throw new Error('CommunicationsService returned an empty folder action result')
  }
  return {
    operation: item.operation === 'move' ? 'move' : 'copy',
    folder_id: item.folderId,
    message_id: item.messageId,
    message: mapFolderMessageItem(item.message)
  }
}

export function mapRichTemplate(item: {
  templateId: string
  name: string
  subjectTemplate: string
  bodyTemplate: string
  variables: string[]
  placeholderVariables: string[]
  undeclaredVariables: string[]
  unusedVariables: string[]
  malformedPlaceholders: string[]
  language?: string
  createdAt: string
  updatedAt: string
} | undefined): CommunicationTemplate {
  if (!item) {
    throw new Error('CommunicationsService returned an empty rich template item')
  }
  return {
    template_id: item.templateId,
    name: item.name,
    subject_template: item.subjectTemplate,
    body_template: item.bodyTemplate,
    variables: item.variables,
    placeholder_variables: item.placeholderVariables,
    undeclared_variables: item.undeclaredVariables,
    unused_variables: item.unusedVariables,
    malformed_placeholders: item.malformedPlaceholders,
    language: item.language ?? null,
    created_at: item.createdAt,
    updated_at: item.updatedAt
  }
}

export function mapMessageSummaryContract(item: {
  keyPoints: string[]
  actionItems: string[]
  risks: string[]
  deadlines: string[]
  eventCandidates: { title: string; evidence: string }[]
  personaCandidates: { title: string; evidence: string }[]
  organizationCandidates: { title: string; evidence: string }[]
  documentCandidates: { title: string; evidence: string }[]
  agreementCandidates: { title: string; evidence: string }[]
} | undefined): MessageAnalyzeResponse['summary_contract'] {
  return {
    key_points: item?.keyPoints ?? [],
    action_items: item?.actionItems ?? [],
    risks: item?.risks ?? [],
    deadlines: item?.deadlines ?? [],
    event_candidates: mapKnowledgeCandidates(item?.eventCandidates),
    persona_candidates: mapKnowledgeCandidates(item?.personaCandidates),
    organization_candidates: mapKnowledgeCandidates(item?.organizationCandidates),
    document_candidates: mapKnowledgeCandidates(item?.documentCandidates),
    agreement_candidates: mapKnowledgeCandidates(item?.agreementCandidates)
  }
}

export function mapKnowledgeCandidates(
  items: { title: string; evidence: string }[] | undefined
): CommunicationKnowledgeCandidate[] {
  return (items ?? []).map((item) => ({
    title: item.title,
    evidence: item.evidence
  }))
}

export function parseJsonObject(value: string | undefined): Record<string, unknown> {
  if (!value || value.trim().length === 0) {
    return
```
_Source file truncated after 12000 characters. / Исходный файл обрезан после 12000 символов._

### `frontend/src/domains/communications/api/connect/messageLifecycle.ts`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/domains/communications/api/connect/messageLifecycle.ts`
- Size bytes / Размер в байтах: `15243`
- Included characters / Включено символов: `12000`
- Truncated / Обрезано: `yes`

```typescript
import { getCommunicationsConnectClient } from '../../../../platform/connect/communicationsClient'
import type {
  AiReplyResponse,
  AiReplyVariantsResponse,
  BulkMessageActionRequest,
  BulkMessageActionResponse,
  CommunicationMessageDetailResponse,
  CommunicationMessagesResponse,
  ExtractNotesResponse,
  ExtractTasksResponse,
  LocalMessageStateResponse,
  MessageAnalyzeResponse,
  MessageAuthCheckResponse,
  MessageExplainResponse,
  MessageExportResponse,
  MessageImportantToggleResponse,
  MessagePinToggleResponse,
  SignatureDetection,
  SmartCcResponse,
  WorkflowActionRequest,
  WorkflowActionResponse,
  WorkflowState,
  WorkflowStateTransitionResponse
} from '../../types/communications'
import {
  mapAttachment,
  mapMessageSummary,
  mapMessageSummaryContract,
  normalizeBulkMessageAction,
  normalizeLocalState,
  normalizeWorkflowState,
  parseJsonObject
} from './mapping'
import { postCommunicationsConnectJson } from './shared'

export type ConnectCommunicationMessagesRequest = {
  account_id?: string
  workflow_state?: string
  channel_kind?: string
  conversation_id?: string
  query?: string
  match_mode?: 'all' | 'any'
  local_state?: string
  cursor?: string
  limit?: number
}

export async function fetchCommunicationMessagesConnect(
  request: ConnectCommunicationMessagesRequest = {}
): Promise<CommunicationMessagesResponse> {
  const response = await getCommunicationsConnectClient().listMessages({
    accountId: request.account_id,
    workflowState: request.workflow_state,
    channelKind: request.channel_kind,
    conversationId: request.conversation_id,
    query: request.query,
    matchMode: request.match_mode,
    localState: request.local_state,
    cursor: request.cursor,
    limit: request.limit ?? 0
  })

  return {
    items: response.items.map(mapMessageSummary),
    next_cursor: response.nextCursor ?? null,
    has_more: response.hasMore
  }
}

export async function fetchCommunicationMessageConnect(
  messageId: string
): Promise<CommunicationMessageDetailResponse> {
  const response = await getCommunicationsConnectClient().getMessage({ messageId })

  return {
    message: {
      message_id: response.item?.messageId ?? messageId,
      raw_record_id: response.item?.rawRecordId ?? '',
      observation_id: response.item?.observationId ?? null,
      account_id: response.item?.accountId ?? '',
      provider_record_id: response.item?.providerRecordId ?? '',
      subject: response.item?.subject ?? '',
      sender: response.item?.sender ?? '',
      recipients: response.item?.recipients ?? [],
      body_text: response.item?.bodyText ?? '',
      body_html: null,
      occurred_at: response.item?.occurredAt ?? null,
      projected_at: response.item?.projectedAt ?? '',
      channel_kind: response.item?.channelKind ?? '',
      conversation_id: response.item?.conversationId ?? null,
      sender_display_name: response.item?.senderDisplayName ?? null,
      delivery_state: response.item?.deliveryState ?? '',
      workflow_state: normalizeWorkflowState(response.item?.workflowState),
      importance_score: response.item?.importanceScore ?? null,
      ai_category: response.item?.aiCategory ?? null,
      ai_summary: response.item?.aiSummary ?? null,
      ai_summary_generated_at: response.item?.aiSummaryGeneratedAt ?? null,
      message_metadata: parseJsonObject(response.item?.messageMetadataJson),
      local_state: normalizeLocalState(response.item?.localState),
      local_state_changed_at: response.item?.localStateChangedAt ?? null,
      local_state_reason: response.item?.localStateReason ?? null
    },
    attachments: response.attachments.map(mapAttachment)
  }
}

export async function transitionMessageWorkflowStateConnect(
  messageId: string,
  workflowState: WorkflowState
): Promise<WorkflowStateTransitionResponse> {
  const response = await getCommunicationsConnectClient().transitionMessageWorkflowState({
    messageId,
    workflowState
  })

  return {
    message_id: response.messageId,
    workflow_state: normalizeWorkflowState(response.workflowState),
    previous_state: response.previousState
  }
}

export async function trashMessageConnect(messageId: string): Promise<LocalMessageStateResponse> {
  const response = await getCommunicationsConnectClient().trashMessage({ messageId })
  return {
    message_id: response.messageId,
    local_state: normalizeLocalState(response.localState),
    provider_deleted: response.providerDeleted ?? undefined
  }
}

export async function restoreMessageConnect(messageId: string): Promise<LocalMessageStateResponse> {
  const response = await getCommunicationsConnectClient().restoreMessage({ messageId })
  return {
    message_id: response.messageId,
    local_state: normalizeLocalState(response.localState),
    provider_deleted: response.providerDeleted ?? undefined
  }
}

export async function markMessageReadConnect(messageId: string): Promise<Record<string, unknown>> {
  const response = await getCommunicationsConnectClient().markMessageRead({ messageId })
  return {
    message_id: response.messageId,
    marked_read: response.markedRead,
    workflow_state: normalizeWorkflowState(response.workflowState)
  }
}

export async function deleteMessageFromProviderConnect(
  messageId: string
): Promise<LocalMessageStateResponse> {
  const response = await getCommunicationsConnectClient().deleteMessageFromProvider({ messageId })
  return {
    message_id: response.messageId,
    local_state: normalizeLocalState(response.localState),
    provider_deleted: response.providerDeleted ?? undefined
  }
}

export async function bulkMessageActionConnect(
  request: BulkMessageActionRequest
): Promise<BulkMessageActionResponse> {
  const response = await getCommunicationsConnectClient().bulkMessageAction({
    action: request.action,
    messageIds: request.message_ids,
    label: request.label ?? undefined,
    snoozeUntil: request.snooze_until ?? undefined
  })

  return {
    action: normalizeBulkMessageAction(response.action),
    requested_count: Number(response.requestedCount),
    matched_count: Number(response.matchedCount),
    updated_count: Number(response.updatedCount),
    not_found: response.notFound
  }
}

export async function toggleMessagePinConnect(messageId: string): Promise<MessagePinToggleResponse> {
  const response = await getCommunicationsConnectClient().toggleMessagePin({ messageId })
  return { message_id: response.messageId, pinned: response.pinned }
}

export async function toggleMessageImportantConnect(
  messageId: string
): Promise<MessageImportantToggleResponse> {
  const response = await getCommunicationsConnectClient().toggleMessageImportant({ messageId })
  return { message_id: response.messageId, important: response.important }
}

export async function toggleMessageMuteConnect(messageId: string): Promise<MessagePinToggleResponse> {
  const response = await getCommunicationsConnectClient().toggleMessageMute({ messageId })
  return {
    message_id: response.messageId,
    pinned: response.muted
  }
}

export async function snoozeMessageConnect(
  messageId: string,
  snoozeUntil: string
): Promise<Record<string, unknown>> {
  return postCommunicationsConnectJson<Record<string, unknown>>('SnoozeMessage', {
    messageId,
    until: snoozeUntil
  })
}

export async function addMessageLabelConnect(
  messageId: string,
  label: string
): Promise<Record<string, unknown>> {
  return postCommunicationsConnectJson<Record<string, unknown>>('AddMessageLabel', {
    messageId,
    label
  })
}

export async function removeMessageLabelConnect(
  messageId: string,
  label: string
): Promise<Record<string, unknown>> {
  return postCommunicationsConnectJson<Record<string, unknown>>('RemoveMessageLabel', {
    messageId,
    label
  })
}

export async function analyzeMessageConnect(messageId: string): Promise<MessageAnalyzeResponse> {
  const response = await getCommunicationsConnectClient().analyzeMessage({ messageId })
  return {
    message_id: response.messageId,
    analyzed: response.analyzed,
    category: response.category ?? null,
    summary: response.summary ?? null,
    summary_contract: mapMessageSummaryContract(response.summaryContract),
    importance_score: response.importanceScore ?? null,
    workflow_state: response.workflowState,
    source: response.source,
    confidence: response.confidence ?? null,
    evidence: response.evidence
  }
}

export async function runWorkflowActionConnect(
  request: WorkflowActionRequest
): Promise<WorkflowActionResponse> {
  const response = await postCommunicationsConnectJson<{
    commandId: string
    eventId: string
    action: string
    status: 'created' | 'updated' | 'linked' | 'opened' | 'archived' | 'noop'
    target?: { kind: 'compose' | 'message' | 'task' | 'document' | 'calendar_event' | 'person'; id?: string }
    provenance?: { sourceKind?: string; sourceId?: string; confidence?: number; evidence: string[] }
  }>('RunWorkflowAction', {
    commandId: request.command_id,
    action: request.action,
    source: request.source ? { kind: request.source.kind, id: request.source.id } : undefined,
    input: request.input
      ? {
          title: request.input.title,
          body: request.input.body,
          email: request.input.email,
          displayName: request.input.display_name,
          startsAt: request.input.starts_at,
          endsAt: request.input.ends_at,
          dueAt: request.input.due_at,
          documentId: request.input.document_id
        }
      : undefined
  })

  return {
    command_id: response.commandId,
    event_id: response.eventId,
    action: response.action as WorkflowActionResponse['action'],
    status: response.status,
    target: { kind: response.target?.kind ?? 'message', id: response.target?.id ?? null },
    provenance: {
      source_kind: response.provenance?.sourceKind,
      source_id: response.provenance?.sourceId,
      confidence: response.provenance?.confidence ?? null,
      evidence: response.provenance?.evidence ?? []
    }
  }
}

export async function fetchMessageExplainConnect(messageId: string): Promise<MessageExplainResponse> {
  const response = await getCommunicationsConnectClient().getMessageExplain({ messageId })
  return { reasons: response.reasons }
}

export async function fetchMessageSmartCcConnect(messageId: string): Promise<SmartCcResponse> {
  const response = await getCommunicationsConnectClient().getMessageSmartCc({ messageId })
  return { suggestions: response.suggestions }
}

export async function exportMessageConnect(
  messageId: string,
  format: 'md' | 'eml' | 'json'
): Promise<MessageExportResponse> {
  const response = await getCommunicationsConnectClient().getMessageExport({ messageId, format })
  return {
    content_type: response.contentType,
    content: response.content,
    filename: response.filename
  }
}

export async function fetchMessageAuthConnect(messageId: string): Promise<MessageAuthCheckResponse> {
  const response = await getCommunicationsConnectClient().getMessageAuth({ messageId })
  return {
    auth: {
      spf: response.auth?.spf
        ? {
            result: response.auth.spf.result,
            domain: response.auth.spf.domain ?? null,
            ip: response.auth.spf.ip ?? null,
            selector: response.auth.spf.selector ?? null,
            policy: response.auth.spf.policy ?? null
          }
        : null,
      dkim: response.auth?.dkim
        ? {
            result: response.auth.dkim.result,
            domain: response.auth.dkim.domain ?? null,
            ip: response.auth.dkim.ip ?? null,
            selector: response.auth.dkim.selector ?? null,
            policy: response.auth.dkim.policy ?? null
          }
        : null,
      dmarc: response.auth?.dmarc
        ? {
            result: response.auth.dmarc.result,
            domain: response.auth.dmarc.domain ?? null,
            ip: response.auth.dmarc.ip ?? null,
            selector: response.auth.dmarc.selector ?? null,

```
_Source file truncated after 12000 characters. / Исходный файл обрезан после 12000 символов._

### `frontend/src/domains/communications/api/connect/shared.ts`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/domains/communications/api/connect/shared.ts`
- Size bytes / Размер в байтах: `696`
- Included characters / Включено символов: `696`
- Truncated / Обрезано: `no`

```typescript
import { ApiClient } from '../../../../platform/api/ApiClient'

export async function postCommunicationsConnectJson<T>(
  method: string,
  body: Record<string, unknown>
): Promise<T> {
  const apiClient = ApiClient.instance
  const response = await fetch(
    `${apiClient.getBaseUrl()}/makosh.communications.v1.CommunicationsService/${method}`,
    {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
        'X-Макошь-Secret': apiClient.getSecret()
      },
      body: JSON.stringify(body)
    }
  )
  if (!response.ok) {
    throw new Error(`CommunicationsService/${method} failed with status ${response.status}`)
  }
  return (await response.json()) as T
}
```

### `frontend/src/domains/communications/api/connectCommunications.test.ts`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/domains/communications/api/connectCommunications.test.ts`
- Size bytes / Размер в байтах: `61306`
- Included characters / Включено символов: `12000`
- Truncated / Обрезано: `yes`

```typescript
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { ApiClient } from '../../../platform/api/ApiClient'
import { resetCommunicationsConnectClientForTests } from '../../../platform/connect/communicationsClient'
import {
  analyzeMessageConnect,
  addMessageLabelConnect,
  bulkMessageActionConnect,
  copyMessageToFolderConnect,
  createCommunicationDraftConnect,
  createCommunicationFolderConnect,
  createCommunicationSavedSearchConnect,
  deleteMessageFromProviderConnect,
  detectMessageLanguageConnect,
  deleteCommunicationFolderConnect,
  deleteCommunicationSavedSearchConnect,
  deleteCommunicationDraftConnect,
  extractMessageNotesConnect,
  extractMessageTasksConnect,
  fetchCommunicationDraftsConnect,
  fetchCommunicationBlockersConnect,
  fetchCommunicationPersonasConnect,
  fetchCommunicationFoldersConnect,
  fetchCommunicationMessageConnect,
  fetchCommunicationMessagesConnect,
  fetchCommunicationOutboxConnect,
  fetchCommunicationSavedSearchesConnect,
  fetchMessageAuthConnect,
  fetchMessageExplainConnect,
  fetchMailboxHealthConnect,
  fetchMessageSignatureConnect,
  generateAiReplyConnect,
  generateAiReplyVariantsConnect,
  markMessageReadConnect,
  fetchMessageSmartCcConnect,
  fetchMessageStateCountsConnect,
  fetchSubscriptionsConnect,
  fetchFolderMessagesConnect,
  fetchCommunicationThreadMessagesConnect,
  fetchCommunicationThreadsConnect,
  fetchRichTemplatesConnect,
  fetchTopSendersConnect,
  moveMessageToFolderConnect,
  redirectMessageConnect,
  removeMessageLabelConnect,
  restoreMessageConnect,
  searchMessagesConnect,
  saveRichTemplateConnect,
  sendCommunicationConnect,
  snoozeMessageConnect,
  trashMessageConnect,
  toggleMessageImportantConnect,
  toggleMessageMuteConnect,
  toggleMessagePinConnect,
  translateMessageConnect,
  transitionMessageWorkflowStateConnect,
  translateCommunicationThreadConnect,
  updateCommunicationFolderConnect,
  updateCommunicationSavedSearchConnect,
  undoCommunicationOutboxItemConnect,
  exportMessageConnect,
  deleteRichTemplateConnect,
  previewRichTemplateMailMergeConnect,
  runWorkflowActionConnect,
  renderRichTemplateConnect
} from './connectCommunications'

describe('communications ConnectRPC API', () => {
  beforeEach(() => {
    ApiClient.resetForTests()
    ApiClient.init('http://127.0.0.1:8080', 'test-secret')
    resetCommunicationsConnectClientForTests()
  })

  afterEach(() => {
    vi.unstubAllGlobals()
    resetCommunicationsConnectClientForTests()
    ApiClient.resetForTests()
  })

  it('lists communications messages through the protected ConnectRPC client', async () => {
    const fetchMock = vi.fn().mockResolvedValue(
      new Response(JSON.stringify({
        items: [{
          messageId: 'msg-1',
          rawRecordId: 'raw-1',
          observationId: 'obs-1',
          accountId: 'account-1',
          providerRecordId: 'provider-1',
          subject: 'Quarterly update',
          sender: 'Ada <ada@example.com>',
          recipients: ['bob@example.com'],
          bodyText: 'Long body text for preview',
          occurredAt: '2026-06-23T10:00:00Z',
          projectedAt: '2026-06-23T10:01:00Z',
          channelKind: 'email',
          conversationId: 'thread-1',
          senderDisplayName: 'Ada',
          deliveryState: 'received',
          messageMetadataJson: '{"tag":"important"}',
          workflowState: 'needs_action',
          importanceScore: 91,
          aiCategory: 'follow_up',
          aiSummary: 'Needs a reply',
          aiSummaryGeneratedAt: '2026-06-23T10:02:00Z',
          localState: 'active',
          localStateChangedAt: '2026-06-23T10:03:00Z',
          attachmentCount: 2
        }],
        nextCursor: 'next-cursor',
        hasMore: true
      }), {
        status: 200,
        headers: { 'Content-Type': 'application/json' }
      })
    )
    vi.stubGlobal('fetch', fetchMock)

    const response = await fetchCommunicationMessagesConnect({
      account_id: 'account-1',
      workflow_state: 'needs_action',
      limit: 10
    })

    expect(response.items[0]).toMatchObject({
      message_id: 'msg-1',
      raw_record_id: 'raw-1',
      observation_id: 'obs-1',
      body_text_preview: 'Long body text for preview',
      workflow_state: 'needs_action',
      attachment_count: 2
    })
    expect(response.next_cursor).toBe('next-cursor')
    expect(response.has_more).toBe(true)
    expect(fetchMock).toHaveBeenCalledOnce()
    const [url, options] = fetchMock.mock.calls[0]
    expect(url).toBe('http://127.0.0.1:8080/makosh.communications.v1.CommunicationsService/ListMessages')
    expect(options.method).toBe('POST')
    expect(new Headers(options.headers).get('X-Макошь-Secret')).toBe('test-secret')
    expect(JSON.parse(decodeBody(options.body))).toMatchObject({
      accountId: 'account-1',
      workflowState: 'needs_action',
      limit: 10
    })
  })

  it('maps workflow transitions, state counts and search through CommunicationsService', async () => {
    const fetchMock = vi
      .fn()
      .mockResolvedValueOnce(
        new Response(JSON.stringify({
          messageId: 'msg-1',
          workflowState: 'done',
          previousState: 'needs_action'
        }), {
          status: 200,
          headers: { 'Content-Type': 'application/json' }
        })
      )
      .mockResolvedValueOnce(
        new Response(JSON.stringify({
          counts: [
            { state: 'new', count: 2 },
            { state: 'done', count: 5 }
          ]
        }), {
          status: 200,
          headers: { 'Content-Type': 'application/json' }
        })
      )
      .mockResolvedValueOnce(
        new Response(JSON.stringify({
          results: [
            {
              objectId: 'msg-1',
              objectKind: 'communication_message',
              title: '[Ada] Quarterly update'
            }
          ]
        }), {
          status: 200,
          headers: { 'Content-Type': 'application/json' }
        })
      )
    vi.stubGlobal('fetch', fetchMock)

    const transition = await transitionMessageWorkflowStateConnect('msg-1', 'done')
    const counts = await fetchMessageStateCountsConnect('account-1', 'active')
    const search = await searchMessagesConnect('quarterly', 15)

    expect(transition).toEqual({
      message_id: 'msg-1',
      workflow_state: 'done',
      previous_state: 'needs_action'
    })
    expect(counts.counts).toEqual([
      { state: 'new', count: 2 },
      { state: 'done', count: 5 }
    ])
    expect(search.results[0]).toEqual({
      object_id: 'msg-1',
      object_kind: 'communication_message',
      title: '[Ada] Quarterly update'
    })
    expect(fetchMock.mock.calls[0][0]).toBe(
      'http://127.0.0.1:8080/makosh.communications.v1.CommunicationsService/TransitionMessageWorkflowState'
    )
    expect(fetchMock.mock.calls[1][0]).toBe(
      'http://127.0.0.1:8080/makosh.communications.v1.CommunicationsService/ListMessageWorkflowStateCounts'
    )
    expect(fetchMock.mock.calls[2][0]).toBe(
      'http://127.0.0.1:8080/makosh.communications.v1.CommunicationsService/SearchMessages'
    )
    expect(JSON.parse(decodeBody(fetchMock.mock.calls[0][1].body))).toMatchObject({
      messageId: 'msg-1',
      workflowState: 'done'
    })
    expect(JSON.parse(decodeBody(fetchMock.mock.calls[1][1].body))).toMatchObject({
      accountId: 'account-1',
      localState: 'active'
    })
    expect(JSON.parse(decodeBody(fetchMock.mock.calls[2][1].body))).toMatchObject({
      query: 'quarterly',
      limit: 15
    })
  })

  it('maps subscriptions, mailbox health, top senders and blockers through CommunicationsService', async () => {
    const fetchMock = vi
      .fn()
      .mockResolvedValueOnce(
        new Response(JSON.stringify({
          items: [{
            sender: 'news@example.com',
            messageId: 'ignored',
            messageCount: 7,
            firstSeen: '2026-06-01T00:00:00Z',
            lastSeen: '2026-06-23T00:00:00Z',
            isNewsletter: true,
            hasUnsubscribe: true
          }],
          nextCursor: 'sub-cursor',
          hasMore: true
        }), {
          status: 200,
          headers: { 'Content-Type': 'application/json' }
        })
      )
      .mockResolvedValueOnce(
        new Response(JSON.stringify({
          item: {
            totalMessages: 42,
            unread: 5,
            needsAction: 3,
            waiting: 2,
            done: 20,
            archived: 10,
            spam: 2,
            important: 6,
            withAttachments: 9,
            averageImportance: 51.5,
            oldestMessageDays: 14.25
          }
        }), {
          status: 200,
          headers: { 'Content-Type': 'application/json' }
        })
      )
      .mockResolvedValueOnce(
        new Response(JSON.stringify({
          items: [{
            sender: 'ada@example.com',
            messageCount: 11,
            avgImportance: 77.5,
            lastMessageDays: 1.5
          }],
          nextCursor: 'sender-cursor',
          hasMore: false
        }), {
          status: 200,
          headers: { 'Content-Type': 'application/json' }
        })
      )
      .mockResolvedValueOnce(
        new Response(JSON.stringify({
          items: [{
            section: '§16-17',
            feature: 'Outbox tracking',
            reason: 'Needs provider callback wiring',
            resolution: 'Connect callback/runtime ingestion'
          }]
        }), {
          status: 200,
          headers: { 'Content-Type': 'application/json' }
        })
      )
    vi.stubGlobal('fetch', fetchMock)

    const subscriptions = await fetchSubscriptionsConnect('account-1', 25, 'cursor:value')
    const health = await fetchMailboxHealthConnect('account-1')
    const senders = await fetchTopSendersConnect('account-1', 10, 'sender:cursor')
    const blockers = await fetchCommunicationBlockersConnect()

    expect(subscriptions).toEqual({
      items: [{
        sender: 'news@example.com',
        message_count: 7,
        first_seen: '2026-06-01T00:00:00Z',
        last_seen: '2026-06-23T00:00:00Z',
        is_newsletter: true,
        has_unsubscribe: true
      }],
      next_cursor: 'sub-cursor',
      has_more: true
    })
    expect(health.total_messages).toBe(42)
    expect(health.oldest_message_days).toBe(14.25)
    expect(senders.items[0]).toEqual({
      sender: 'ada@example.com',
      message_count: 11,
      avg_importance: 77.5,
      last_message_days: 1.5
    })
    expect(blockers).toEqual([{
      section: '§16-17',
      feature: 'Outbox tracking',
      reason: 'Needs provider callback wiring',
      resolution: 'Connect callback/runtime ingestion'
    }])
    expect(fetchMock.mock.calls[0][0]).toBe(
      'http://127.0.0.1:8080/makosh.communications.v1.CommunicationsService/ListSubscriptions'
    )
    expect(fetchMock.mock.calls[1][0]).toBe(
      'http://127.0.0.1:8080/makosh.communications.v1.CommunicationsService/GetMailboxHealth'
    )
    expect(fetchMock.mock.calls[2][0]).toBe(
      'http://127.0.0.1:8080/makosh.communications.v1.CommunicationsService/ListTopSenders'
    )
    expect(fetchMock.mock.calls[3][0]).toBe(
      'http://127.0.0.1:8080/makosh.communications.v1.CommunicationsService/ListCommunicationBlockers'
    )
  })

  it('maps personas and rich templates through CommunicationsService', async () => {
    const fetchMock = vi
      .fn()
      .mockResolvedValueOnce(
        new Response(JSON.stringify({
          items: [{
            personaId: 'persona-1',
            accountId: 'account-1',
            name: 'Owner',
            displayName: 'Owner Persona',
            signature: 'Regards',
            defaultLanguage: 'en',
            defaultTone: 'warm',
            isDefault: true,
            metadataJson: '{"role":"owner"}',
            createdAt: '2026-06-23T00:00:00Z',
            updatedAt: '2026-06-23T00:00:00Z'
          }]
        }), {
          status: 200,
          headers: { 'Content-Type': '
```
_Source file truncated after 12000 characters. / Исходный файл обрезан после 12000 символов._

### `frontend/src/domains/communications/api/connectCommunications.ts`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/domains/communications/api/connectCommunications.ts`
- Size bytes / Размер в байтах: `2242`
- Included characters / Включено символов: `2242`
- Truncated / Обрезано: `no`

```typescript
export type { ConnectCommunicationMessagesRequest } from './connect/messageLifecycle'

export {
  addMessageLabelConnect,
  analyzeMessageConnect,
  bulkMessageActionConnect,
  deleteMessageFromProviderConnect,
  exportMessageConnect,
  extractMessageNotesConnect,
  extractMessageTasksConnect,
  fetchCommunicationMessageConnect,
  fetchCommunicationMessagesConnect,
  fetchMessageAuthConnect,
  fetchMessageExplainConnect,
  fetchMessageSignatureConnect,
  fetchMessageSmartCcConnect,
  generateAiReplyConnect,
  generateAiReplyVariantsConnect,
  markMessageReadConnect,
  removeMessageLabelConnect,
  restoreMessageConnect,
  runWorkflowActionConnect,
  snoozeMessageConnect,
  toggleMessageImportantConnect,
  toggleMessageMuteConnect,
  toggleMessagePinConnect,
  transitionMessageWorkflowStateConnect,
  trashMessageConnect
} from './connect/messageLifecycle'

export {
  deleteRichTemplateConnect,
  detectMessageLanguageConnect,
  fetchCommunicationBlockersConnect,
  fetchCommunicationPersonasConnect,
  fetchMailboxHealthConnect,
  fetchMessageStateCountsConnect,
  fetchRichTemplatesConnect,
  fetchSubscriptionsConnect,
  fetchTopSendersConnect,
  previewRichTemplateMailMergeConnect,
  renderRichTemplateConnect,
  saveRichTemplateConnect,
  searchMessagesConnect,
  translateMessageConnect
} from './connect/insights'

export {
  copyMessageToFolderConnect,
  createCommunicationDraftConnect,
  createCommunicationFolderConnect,
  createCommunicationSavedSearchConnect,
  deleteCommunicationDraftConnect,
  deleteCommunicationFolderConnect,
  deleteCommunicationSavedSearchConnect,
  fetchCommunicationDraftsConnect,
  fetchCommunicationFoldersConnect,
  fetchCommunicationOutboxConnect,
  fetchCommunicationSavedSearchesConnect,
  fetchCommunicationThreadMessagesConnect,
  fetchCommunicationThreadsConnect,
  fetchFolderMessagesConnect,
  inspectAttachmentArchiveConnect,
  moveMessageToFolderConnect,
  previewAttachmentConnect,
  redirectMessageConnect,
  searchAttachmentsConnect,
  sendCommunicationConnect,
  translateAttachmentConnect,
  translateCommunicationThreadConnect,
  undoCommunicationOutboxItemConnect,
  updateCommunicationFolderConnect,
  updateCommunicationSavedSearchConnect
} from './connect/collections'
```

### `frontend/src/domains/communications/api/folderApi.ts`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/domains/communications/api/folderApi.ts`
- Size bytes / Размер в байтах: `1871`
- Included characters / Включено символов: `1871`
- Truncated / Обрезано: `no`

```typescript
import {
  copyMessageToFolderConnect,
  createCommunicationFolderConnect,
  deleteCommunicationFolderConnect,
  fetchCommunicationFoldersConnect,
  fetchFolderMessagesConnect,
  moveMessageToFolderConnect,
  updateCommunicationFolderConnect
} from './connectCommunications'
import type {
  FolderDeleteResponse,
  FolderMessageActionResponse,
  FolderMessageListResponse,
  CommunicationFolder,
  CommunicationFolderInput,
  CommunicationFolderListResponse,
  CommunicationFolderUpdate
} from '../types/folders'

export async function fetchCommunicationFolders(
  accountId?: string,
  limit = 500,
  cursor?: string | null
): Promise<CommunicationFolderListResponse> {
  return fetchCommunicationFoldersConnect(accountId, limit, cursor ?? undefined)
}

export async function createCommunicationFolder(request: CommunicationFolderInput): Promise<CommunicationFolder> {
  return createCommunicationFolderConnect(request)
}

export async function updateCommunicationFolder(
  folderId: string,
  request: CommunicationFolderUpdate
): Promise<CommunicationFolder> {
  return updateCommunicationFolderConnect(folderId, request)
}

export async function deleteCommunicationFolder(folderId: string): Promise<FolderDeleteResponse> {
  return deleteCommunicationFolderConnect(folderId)
}

export async function fetchFolderMessages(
  folderId: string,
  limit = 250,
  cursor?: string | null
): Promise<FolderMessageListResponse> {
  return fetchFolderMessagesConnect(folderId, limit, cursor ?? undefined)
}

export async function copyMessageToFolder(
  folderId: string,
  messageId: string
): Promise<FolderMessageActionResponse> {
  return copyMessageToFolderConnect(folderId, messageId)
}

export async function moveMessageToFolder(
  folderId: string,
  messageId: string
): Promise<FolderMessageActionResponse> {
  return moveMessageToFolderConnect(folderId, messageId)
}
```

### `frontend/src/domains/communications/api/messageApi.ts`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/domains/communications/api/messageApi.ts`
- Size bytes / Размер в байтах: `9547`
- Included characters / Включено символов: `9547`
- Truncated / Обрезано: `no`

```typescript
import {
  analyzeMessageConnect,
  bulkMessageActionConnect,
  addMessageLabelConnect,
  createCommunicationDraftConnect,
  deleteMessageFromProviderConnect,
  detectMessageLanguageConnect,
  deleteCommunicationDraftConnect,
  extractMessageNotesConnect,
  extractMessageTasksConnect,
  fetchCommunicationMessageConnect,
  fetchCommunicationMessagesConnect,
  fetchCommunicationDraftsConnect,
  fetchCommunicationBlockersConnect,
  fetchCommunicationPersonasConnect,
  fetchMessageAuthConnect,
  fetchMessageExplainConnect,
  fetchMailboxHealthConnect,
  fetchMessageSignatureConnect,
  fetchMessageSmartCcConnect,
  fetchMessageStateCountsConnect,
  fetchSubscriptionsConnect,
  fetchTopSendersConnect,
  fetchRichTemplatesConnect,
  generateAiReplyConnect,
  generateAiReplyVariantsConnect,
  markMessageReadConnect,
  restoreMessageConnect,
  runWorkflowActionConnect,
  searchMessagesConnect,
  saveRichTemplateConnect,
  snoozeMessageConnect,
  trashMessageConnect,
  toggleMessageImportantConnect,
  toggleMessageMuteConnect,
  toggleMessagePinConnect,
  translateMessageConnect,
  transitionMessageWorkflowStateConnect,
  exportMessageConnect,
  deleteRichTemplateConnect,
  previewRichTemplateMailMergeConnect,
  renderRichTemplateConnect
} from './connectCommunications'
import type {
  CommunicationMessagesResponse,
  CommunicationMessageDetailResponse,
  WorkflowState,
  LocalMessageState,
  WorkflowStateCountsResponse,
  WorkflowStateTransitionResponse,
  LocalMessageStateResponse,
  MessageAnalyzeResponse,
  WorkflowActionRequest,
  WorkflowActionResponse,
  DraftListResponse,
  CommunicationDraft,
  DraftDeleteResponse,
  DraftUpsertRequest,
  MailboxHealth,
  SenderStatsListResponse,
  MessageExplainResponse,
  SmartCcResponse,
  MessagePinToggleResponse,
  MessageImportantToggleResponse,
  MessageExportResponse,
  MessageAuthCheckResponse,
  SignatureDetection,
  LanguageDetection,
  TranslationResponse,
  AiReplyResponse,
  AiReplyVariantsRequest,
  AiReplyVariantsResponse,
  ExtractTasksResponse,
  ExtractNotesResponse,
  CommunicationSearchResponse,
  SubscriptionListResponse,
  CommunicationArchitectureBlocker,
  CommunicationPersona,
  CommunicationTemplate,
  RichTemplateDeleteResponse,
  RichTemplateMailMergePreviewRequest,
  RichTemplateMailMergePreviewResponse,
  RichTemplateRenderRequest,
  RichTemplateRenderResponse,
  RichTemplateUpsertRequest,
  RichTemplateUpsertResponse,
  BulkMessageActionRequest,
  BulkMessageActionResponse
} from '../types/communications'

export async function fetchCommunicationMessages(
  accountId?: string,
  workflowState?: WorkflowState,
  channelKind?: string,
  query?: string,
  localState?: LocalMessageState,
  limit = 250,
  cursor?: string | null
): Promise<CommunicationMessagesResponse> {
  return fetchCommunicationMessagesConnect({
    account_id: accountId,
    workflow_state: workflowState,
    channel_kind: channelKind,
    query,
    local_state: localState,
    limit,
    cursor: cursor ?? undefined
  })
}

export async function fetchCommunicationMessage(messageId: string): Promise<CommunicationMessageDetailResponse> {
  return fetchCommunicationMessageConnect(messageId)
}

export async function transitionMessageWorkflowState(
  messageId: string,
  workflowState: WorkflowState
): Promise<WorkflowStateTransitionResponse> {
  return transitionMessageWorkflowStateConnect(messageId, workflowState)
}

export async function fetchMessageStateCounts(
  accountId?: string,
  localState?: LocalMessageState
): Promise<WorkflowStateCountsResponse> {
  return fetchMessageStateCountsConnect(accountId, localState)
}

export async function trashMessage(messageId: string): Promise<LocalMessageStateResponse> {
  return trashMessageConnect(messageId)
}

export async function restoreMessage(messageId: string): Promise<LocalMessageStateResponse> {
  return restoreMessageConnect(messageId)
}

export async function markMessageRead(messageId: string): Promise<Record<string, unknown>> {
  return markMessageReadConnect(messageId)
}

export async function deleteMessageFromProvider(messageId: string): Promise<LocalMessageStateResponse> {
  return deleteMessageFromProviderConnect(messageId)
}

export async function bulkMessageAction(
  request: BulkMessageActionRequest
): Promise<BulkMessageActionResponse> {
  return bulkMessageActionConnect(request)
}

export async function analyzeMessage(messageId: string): Promise<MessageAnalyzeResponse> {
  return analyzeMessageConnect(messageId)
}

export async function runWorkflowAction(
  request: WorkflowActionRequest
): Promise<WorkflowActionResponse> {
  return runWorkflowActionConnect(request)
}

export async function searchEmails(query: string, limit = 20): Promise<CommunicationSearchResponse> {
  return searchMessagesConnect(query, limit)
}

export async function fetchDrafts(
  accountId?: string,
  status?: string,
  limit = 100,
  cursor?: string | null
): Promise<DraftListResponse> {
  return fetchCommunicationDraftsConnect(accountId, status, limit, cursor ?? undefined)
}

export async function createDraft(draft: DraftUpsertRequest): Promise<CommunicationDraft> {
  return createCommunicationDraftConnect(draft)
}

export async function deleteDraft(draftId: string): Promise<DraftDeleteResponse> {
  return deleteCommunicationDraftConnect(draftId)
}

export async function fetchMessageExplain(messageId: string): Promise<MessageExplainResponse> {
  return fetchMessageExplainConnect(messageId)
}

export async function fetchMessageSmartCc(messageId: string): Promise<SmartCcResponse> {
  return fetchMessageSmartCcConnect(messageId)
}

export async function toggleMessagePin(messageId: string): Promise<MessagePinToggleResponse> {
  return toggleMessagePinConnect(messageId)
}

export async function toggleMessageImportant(messageId: string): Promise<MessageImportantToggleResponse> {
  return toggleMessageImportantConnect(messageId)
}

export async function toggleMessageMute(messageId: string): Promise<MessagePinToggleResponse> {
  return toggleMessageMuteConnect(messageId)
}

export async function snoozeMessage(
  messageId: string,
  until: string
): Promise<Record<string, unknown>> {
  return snoozeMessageConnect(messageId, until)
}

export async function addMessageLabel(
  messageId: string,
  label: string
): Promise<Record<string, unknown>> {
  return addMessageLabelConnect(messageId, label)
}

export async function exportMessage(
  messageId: string,
  format: 'md' | 'eml' | 'json'
): Promise<MessageExportResponse> {
  return exportMessageConnect(messageId, format)
}

export async function fetchMessageAuth(messageId: string): Promise<MessageAuthCheckResponse> {
  return fetchMessageAuthConnect(messageId)
}

export async function fetchMessageSignature(messageId: string): Promise<SignatureDetection> {
  return fetchMessageSignatureConnect(messageId)
}

export async function detectMessageLanguage(messageId: string): Promise<LanguageDetection> {
  return detectMessageLanguageConnect(messageId)
}

export async function translateMessage(
  messageId: string,
  targetLanguage: string
): Promise<TranslationResponse> {
  return translateMessageConnect(messageId, targetLanguage)
}

export async function generateAiReply(
  messageId: string,
  request: { tone?: string; language?: string; context?: string } = {}
): Promise<AiReplyResponse> {
  return generateAiReplyConnect(messageId, request)
}

export async function generateAiReplyVariants(
  messageId: string,
  request: AiReplyVariantsRequest = {}
): Promise<AiReplyVariantsResponse> {
  return generateAiReplyVariantsConnect(messageId, request)
}

export async function extractMessageTasks(messageId: string): Promise<ExtractTasksResponse> {
  return extractMessageTasksConnect(messageId)
}

export async function extractMessageNotes(messageId: string): Promise<ExtractNotesResponse> {
  return extractMessageNotesConnect(messageId)
}

export async function fetchSubscriptions(
  accountId?: string,
  limit = 50,
  cursor?: string | null
): Promise<SubscriptionListResponse> {
  return fetchSubscriptionsConnect(accountId, limit, cursor ?? undefined)
}

export async function fetchMailboxHealth(accountId?: string): Promise<MailboxHealth> {
  return fetchMailboxHealthConnect(accountId)
}

export async function fetchTopSenders(
  accountId?: string,
  limit = 20,
  cursor?: string | null
): Promise<SenderStatsListResponse> {
  return fetchTopSendersConnect(accountId, limit, cursor ?? undefined)
}

export async function fetchPersonas(): Promise<{ items: CommunicationPersona[] }> {
  return fetchCommunicationPersonasConnect()
}

export async function fetchRichTemplates(): Promise<{ templates: CommunicationTemplate[] }> {
  return fetchRichTemplatesConnect()
}

export async function saveRichTemplate(
  request: RichTemplateUpsertRequest
): Promise<RichTemplateUpsertResponse> {
  return saveRichTemplateConnect(request)
}

export async function deleteRichTemplate(templateId: string): Promise<RichTemplateDeleteResponse> {
  return deleteRichTemplateConnect(templateId)
}

export async function renderRichTemplate(
  request: RichTemplateRenderRequest
): Promise<RichTemplateRenderResponse> {
  return renderRichTemplateConnect(request)
}

export async function previewRichTemplateMailMerge(
  request: RichTemplateMailMergePreviewRequest
): Promise<RichTemplateMailMergePreviewResponse> {
  return previewRichTemplateMailMergeConnect(request)
}

export async function fetchCommunicationBlockers(): Promise<CommunicationArchitectureBlocker[]> {
  return fetchCommunicationBlockersConnect()
}
```

### `frontend/src/domains/communications/api/outboxApi.ts`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/domains/communications/api/outboxApi.ts`
- Size bytes / Размер в байтах: `640`
- Included characters / Включено символов: `640`
- Truncated / Обрезано: `no`

```typescript
import { fetchCommunicationOutboxConnect, undoCommunicationOutboxItemConnect } from './connectCommunications'
import type { CommunicationOutboxItem, CommunicationOutboxStatus, OutboxListResponse } from '../types/communications'

export async function fetchOutboxItems(
  accountId?: string,
  status?: CommunicationOutboxStatus,
  limit = 100,
  cursor?: string | null
): Promise<OutboxListResponse> {
  return fetchCommunicationOutboxConnect(accountId, status, limit, cursor ?? undefined)
}

export async function undoOutboxItem(outboxId: string): Promise<CommunicationOutboxItem> {
  return undoCommunicationOutboxItemConnect(outboxId)
}
```

### `frontend/src/domains/communications/api/providerChannels.boundary.test.ts`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/domains/communications/api/providerChannels.boundary.test.ts`
- Size bytes / Размер в байтах: `2613`
- Included characters / Включено символов: `2613`
- Truncated / Обрезано: `no`

```typescript
import { readFileSync } from 'node:fs'
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { ApiClient } from '../../../platform/api/ApiClient'
import {
  fetchCommunicationConversationDetail,
  fetchCommunicationMessages,
  fetchCommunicationRawEvidence,
  searchCommunicationMessages,
} from './providerChannels'

describe('communications provider-neutral channel API boundary', () => {
  beforeEach(() => {
    ApiClient.resetForTests()
    ApiClient.init('http://127.0.0.1:8080', 'test-secret')
  })

  afterEach(() => {
    vi.unstubAllGlobals()
    ApiClient.resetForTests()
  })

  it('does not embed provider-control integration routes in domain clients', () => {
    const source = readFileSync(new URL('./providerChannels.ts', import.meta.url), 'utf8')

    expect(source).not.toMatch(/\/api\/v1\/integrations\/.*\/provider-(commands|search|media|sync)/)
  })

  it('uses provider-neutral Communications routes for business reads and evidence', async () => {
    const fetchMock = vi
      .fn()
      .mockResolvedValueOnce(new Response(JSON.stringify({ item: { conversation_id: 'conv-1' } }), {
        status: 200,
        headers: { 'Content-Type': 'application/json' },
      }))
      .mockResolvedValueOnce(new Response(JSON.stringify({ items: [] }), {
        status: 200,
        headers: { 'Content-Type': 'application/json' },
      }))
      .mockResolvedValueOnce(new Response(JSON.stringify({ query: 'alpha', items: [], total: 0 }), {
        status: 200,
        headers: { 'Content-Type': 'application/json' },
      }))
      .mockResolvedValueOnce(new Response(JSON.stringify({ raw_record: { raw_record_id: 'raw-1' } }), {
        status: 200,
        headers: { 'Content-Type': 'application/json' },
      }))
    vi.stubGlobal('fetch', fetchMock)

    await fetchCommunicationConversationDetail('conv-1')
    await fetchCommunicationMessages({ conversationId: 'conv-1', channelKind: 'telegram_user' })
    await searchCommunicationMessages({ q: 'alpha', account_id: 'acct-1', provider_chat_id: 'chat-1' })
    await fetchCommunicationRawEvidence('msg-1')

    expect(String(fetchMock.mock.calls[0][0])).toContain('/api/v1/communications/conversations/conv-1')
    expect(String(fetchMock.mock.calls[1][0])).toContain('/api/v1/communications/messages?')
    expect(String(fetchMock.mock.calls[1][0])).toContain('conversation_id=conv-1')
    expect(String(fetchMock.mock.calls[2][0])).toContain('/api/v1/communications/search/messages?')
    expect(String(fetchMock.mock.calls[3][0])).toContain('/api/v1/communications/messages/msg-1/raw-evidence')
  })
})
```

### `frontend/src/domains/communications/api/providerChannels.ts`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/domains/communications/api/providerChannels.ts`
- Size bytes / Размер в байтах: `6142`
- Included characters / Включено символов: `6142`
- Truncated / Обрезано: `no`

```typescript
import { ApiClient } from '../../../platform/api/ApiClient'
import type {
  CommunicationProviderConversationDetailResponse,
  CommunicationProviderConversationListResponse,
  CommunicationProviderMessageListResponse,
  CommunicationProviderMessageSearchResponse,
  CommunicationProviderTopicListResponse,
  CommunicationRawEvidenceResponse,
} from '../types/providerChannels'

export async function fetchCommunicationConversations(
  accountId?: string,
  limit = 50
): Promise<CommunicationProviderConversationListResponse> {
  const params = new URLSearchParams({ limit: String(Math.trunc(limit)) })
  if (accountId?.trim()) {
    params.set('account_id', accountId.trim())
  }
  return ApiClient.instance.get<CommunicationProviderConversationListResponse>(
    `/api/v1/communications/conversations?${params.toString()}`,
    'Communication conversations request failed'
  )
}

export async function searchCommunicationConversations(params: {
  q: string
  account_id?: string
  limit?: number
}): Promise<CommunicationProviderConversationListResponse & { query: string; total: number }> {
  const query = new URLSearchParams({ q: params.q.trim() })
  if (params.account_id?.trim()) {
    query.set('account_id', params.account_id.trim())
  }
  if (params.limit != null) {
    query.set('limit', String(Math.trunc(params.limit)))
  }
  return ApiClient.instance.get<CommunicationProviderConversationListResponse & { query: string; total: number }>(
    `/api/v1/communications/conversations/search?${query.toString()}`,
    'Communication conversation search failed'
  )
}

export async function fetchCommunicationConversationDetail(
  conversationId: string
): Promise<CommunicationProviderConversationDetailResponse> {
  return ApiClient.instance.get<CommunicationProviderConversationDetailResponse>(
    `/api/v1/communications/conversations/${encodeURIComponent(conversationId)}`,
    'Communication conversation detail request failed'
  )
}

export async function fetchCommunicationConversationMembers(
  conversationId: string,
  limit = 50,
  query?: string,
  role?: string,
  cursor?: string
) {
  const params = new URLSearchParams({ limit: String(Math.trunc(limit)) })
  if (query?.trim()) params.set('query', query.trim())
  if (role?.trim()) params.set('role', role.trim())
  if (cursor?.trim()) params.set('cursor', cursor.trim())
  return ApiClient.instance.get(
    `/api/v1/communications/conversations/${encodeURIComponent(conversationId)}/members?${params.toString()}`,
    'Communication conversation members request failed'
  )
}

export async function fetchCommunicationMessages(params: {
  accountId?: string
  conversationId?: string
  channelKind?: string
  limit?: number
} = {}): Promise<CommunicationProviderMessageListResponse> {
  const query = new URLSearchParams({ limit: String(Math.trunc(params.limit ?? 50)) })
  if (params.accountId?.trim()) {
    query.set('account_id', params.accountId.trim())
  }
  if (params.conversationId?.trim()) {
    query.set('conversation_id', params.conversationId.trim())
  }
  if (params.channelKind?.trim()) {
    query.set('channel_kind', params.channelKind.trim())
  }
  return ApiClient.instance.get<CommunicationProviderMessageListResponse>(
    `/api/v1/communications/messages?${query.toString()}`,
    'Communication messages request failed'
  )
}

export async function searchCommunicationMessages(params: {
  q: string
  account_id?: string
  provider_chat_id?: string
  limit?: number
}): Promise<CommunicationProviderMessageSearchResponse> {
  const query = new URLSearchParams({ q: params.q.trim() })
  if (params.account_id?.trim()) {
    query.set('account_id', params.account_id.trim())
  }
  if (params.provider_chat_id?.trim()) {
    query.set('provider_chat_id', params.provider_chat_id.trim())
  }
  if (params.limit != null) {
    query.set('limit', String(Math.trunc(params.limit)))
  }
  return ApiClient.instance.get<CommunicationProviderMessageSearchResponse>(
    `/api/v1/communications/search/messages?${query.toString()}`,
    'Communication message search failed'
  )
}

export async function fetchCommunicationPinnedMessages(params: {
  conversationId: string
  limit?: number
}): Promise<CommunicationProviderMessageListResponse> {
  const query = new URLSearchParams()
  if (params.limit != null) {
    query.set('limit', String(Math.trunc(params.limit)))
  }
  return ApiClient.instance.get<CommunicationProviderMessageListResponse>(
    `/api/v1/communications/conversations/${encodeURIComponent(params.conversationId)}/pinned-messages?${query.toString()}`,
    'Communication pinned messages request failed'
  )
}

export async function fetchCommunicationRawEvidence(
  messageId: string
): Promise<CommunicationRawEvidenceResponse> {
  return ApiClient.instance.get<CommunicationRawEvidenceResponse>(
    `/api/v1/communications/messages/${encodeURIComponent(messageId)}/raw-evidence`,
    'Communication raw evidence request failed'
  )
}

export async function fetchCommunicationTopics(
  conversationId: string,
  limit = 100
): Promise<CommunicationProviderTopicListResponse> {
  return ApiClient.instance.get<CommunicationProviderTopicListResponse>(
    `/api/v1/communications/conversations/${encodeURIComponent(conversationId)}/topics?limit=${limit}`,
    'Communication topics request failed'
  )
}

export async function fetchCommunicationTopicMessages(
  topicId: string,
  limit = 50
): Promise<CommunicationProviderMessageListResponse> {
  return ApiClient.instance.get<CommunicationProviderMessageListResponse>(
    `/api/v1/communications/topics/${encodeURIComponent(topicId)}/messages?limit=${limit}`,
    'Communication topic messages request failed'
  )
}

export async function searchCommunicationTopics(
  conversationId: string,
  q: string,
  limit = 50
): Promise<CommunicationProviderTopicListResponse> {
  const params = new URLSearchParams({
    q: q.trim(),
    telegram_chat_id: conversationId.trim(),
    limit: String(limit),
  })
  return ApiClient.instance.get<CommunicationProviderTopicListResponse>(
    `/api/v1/communications/topics/search?${params}`,
    'Communication topic search failed'
  )
}
```

### `frontend/src/domains/communications/api/readReceipts.test.ts`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/domains/communications/api/readReceipts.test.ts`
- Size bytes / Размер в байтах: `1555`
- Included characters / Включено символов: `1555`
- Truncated / Обрезано: `no`

```typescript
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { ApiClient } from '../../../platform/api/ApiClient'
import { recordReadReceipt } from './readReceipts'

describe('communication read receipt API', () => {
  beforeEach(() => {
    ApiClient.resetForTests()
    ApiClient.init('http://127.0.0.1:8080', 'test-secret')
  })

  afterEach(() => {
    vi.unstubAllGlobals()
    ApiClient.resetForTests()
  })

  it('posts read receipts through the protected communications API', async () => {
    const fetchMock = vi.fn().mockResolvedValue(
      new Response(
        JSON.stringify({
          receipt_id: 'mail_read_receipt:1',
          outbox_id: 'outbox-1',
          receipt_kind: 'read'
        }),
        {
          status: 200,
          headers: { 'Content-Type': 'application/json' }
        }
      )
    )
    vi.stubGlobal('fetch', fetchMock)

    await recordReadReceipt({
      account_id: 'account-1',
      provider_message_id: 'provider-message-1',
      recipient: 'reader@example.com',
      read_at: '2026-06-15T10:00:00Z',
      source_kind: 'mdn'
    })

    expect(fetchMock).toHaveBeenCalledOnce()
    const [url, init] = fetchMock.mock.calls[0]
    expect(url).toContain('/api/v1/communications/read-receipts')
    expect(init.method).toBe('POST')
    expect(JSON.parse(init.body as string)).toEqual({
      account_id: 'account-1',
      provider_message_id: 'provider-message-1',
      recipient: 'reader@example.com',
      read_at: '2026-06-15T10:00:00Z',
      source_kind: 'mdn'
    })
  })
})
```
