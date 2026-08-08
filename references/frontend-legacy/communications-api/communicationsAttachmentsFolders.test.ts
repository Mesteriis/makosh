// Historical pre-clean-room transport specification. Not part of the active test suite.
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

  it('posts attachment translation requests without caller-provided text', async () => {
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
          source: 'durable_extracted_text'
        }),
        {
          status: 200,
          headers: { 'Content-Type': 'application/json' }
        }
      )
    )
    vi.stubGlobal('fetch', fetchMock)

    await translateAttachment('mail_attachment:1', {
      target_language: 'en'
    })

    expect(fetchMock).toHaveBeenCalledOnce()
    const [url, init] = fetchMock.mock.calls[0]
    expect(url).toBe(
      'http://127.0.0.1:8080/makosh.communications.v1.CommunicationsService/TranslateAttachment'
    )
    expect(init.method).toBe('POST')
    expect(JSON.parse(decodeBody(init.body))).toEqual({
      attachmentId: 'mail_attachment:1',
      targetLanguage: 'en'
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
        scheduledSendAt: null,
        undoDeadlineAt: null,
        failureReason: null
      }), {
        status: 200,
        headers: { 'Content-Type': 'application/json' }
      })
    )
    vi.stubGlobal('fetch', fetchMock)

    await redirectMessage('msg-1', {
      to: ['redirect@example.com'],
      cc: ['copy@example.com'],
      confirmed_provider_write: true
    })

    expect(fetchMock).toHaveBeenCalledOnce()
    const [url, init] = fetchMock.mock.calls[0]
    expect(url).toBe(
      'http://127.0.0.1:8080/makosh.communications.v1.CommunicationsService/RedirectMessage'
    )
    expect(init.method).toBe('POST')
    expect(JSON.parse(decodeBody(init.body))).toEqual({
      messageId: 'msg-1',
      toRecipients: ['redirect@example.com'],
      ccRecipients: ['copy@example.com'],
      confirmedProviderWrite: true
    })
  })
})

function decodeBody(body: BodyInit | null | undefined): string {
  if (typeof body === 'string') {
    return body
  }
  if (body instanceof Uint8Array) {
    return new TextDecoder().decode(body)
  }
  return ''
}
