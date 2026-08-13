import { beforeEach, describe, expect, it, vi } from 'vitest'
import { createPinia, setActivePinia } from 'pinia'
import {
  DocumentCustodyStateV1,
  DocumentStateV1,
  type DocumentV1
} from '../../../gen/makosh/documents/client/v1/documents_pb'

const clients = vi.hoisted(() => ({
  query: { list: vi.fn(), search: vi.fn(), get: vi.fn(), listSources: vi.fn() },
  command: {
    create: vi.fn(), update: vi.fn(), setState: vi.fn(),
    attachBlob: vi.fn(), releaseBlob: vi.fn(), addSource: vi.fn(), removeSource: vi.fn()
  }
}))

vi.mock('../../../platform/connect/documentsClient', () => ({
  getDocumentsQueryClient: () => clients.query,
  getDocumentsCommandClient: () => clients.command
}))

import { useDocumentsStore } from './documents'

describe('typed Documents owner store', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
    vi.clearAllMocks()
    clients.query.listSources.mockResolvedValue({ sources: [], nextAfterSourceId: new Uint8Array() })
  })

  it('loads all bounded pages using the exclusive last-returned cursor', async () => {
    clients.query.list
      .mockResolvedValueOnce({ documents: [document(1)], nextAfterDocumentId: id(1) })
      .mockResolvedValueOnce({ documents: [document(2)], nextAfterDocumentId: new Uint8Array() })
    const store = useDocumentsStore()

    await store.loadAll()

    expect(store.documents.map((value) => value.documentId)).toEqual([id(1), id(2)])
    expect(clients.query.list.mock.calls).toEqual([
      [{ logicalOwnerId: '', afterDocumentId: new Uint8Array(), limit: 50 }],
      [{ logicalOwnerId: '', afterDocumentId: id(1), limit: 50 }]
    ])
  })

  it('dispatches lifecycle and source commands with exact current revisions', async () => {
    const initial = document(1)
    const revised = { ...initial, documentRevision: 2n }
    clients.query.list.mockResolvedValue({ documents: [initial], nextAfterDocumentId: new Uint8Array() })
    clients.command.setState.mockResolvedValue({ document: revised })
    clients.command.addSource.mockResolvedValue({ document: { ...revised, documentRevision: 3n } })
    const store = useDocumentsStore()
    await store.loadAll()

    await store.setDocumentState(store.documents[0]!, DocumentStateV1.DOCUMENT_STATE_ARCHIVED)
    await store.addSource(store.documents[0]!, {
      sourceOwnerId: 'public-source', sourceRecordId: 'record-1', sourceRevision: 7n,
      evidenceDigest: new Uint8Array(32).fill(9)
    })

    expect(clients.command.setState.mock.calls[0]?.[0]).toMatchObject({
      documentId: id(1), expectedDocumentRevision: 1n,
      state: DocumentStateV1.DOCUMENT_STATE_ARCHIVED
    })
    expect(clients.command.addSource.mock.calls[0]?.[0]).toMatchObject({
      documentId: id(1), expectedDocumentRevision: 2n,
      sourceOwnerId: 'public-source', sourceRecordId: 'record-1', sourceRevision: 7n
    })
    expect(store.documents[0]?.documentRevision).toBe(3n)
  })
})

function document(seed: number): DocumentV1 {
  return {
    $typeName: 'makosh.documents.client.v1.DocumentV1',
    documentId: id(seed), logicalOwnerId: 'owner-1', title: `Document ${seed}`,
    description: '', mediaType: 'application/pdf', originalFileName: `document-${seed}.pdf`,
    declaredSize: 10n, contentSha256: new Uint8Array(32).fill(seed),
    state: DocumentStateV1.DOCUMENT_STATE_ACTIVE,
    custodyState: DocumentCustodyStateV1.DOCUMENT_CUSTODY_STATE_UNBOUND,
    documentRevision: 1n
  }
}

function id(seed: number): Uint8Array {
  return new Uint8Array(16).fill(seed)
}
