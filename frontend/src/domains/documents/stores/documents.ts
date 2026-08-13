import { computed, ref } from 'vue'
import { defineStore } from 'pinia'
import {
  DocumentStateV1,
  type DocumentSourceV1,
  type DocumentV1,
  type TimestampV1
} from '../../../gen/makosh/documents/client/v1/documents_pb'
import {
  getDocumentsCommandClient,
  getDocumentsQueryClient
} from '../../../platform/connect/documentsClient'

const PAGE_LIMIT = 50

export const useDocumentsStore = defineStore('documents-owner', () => {
  const documents = ref<DocumentV1[]>([])
  const selectedDocument = ref<DocumentV1>()
  const sources = ref<DocumentSourceV1[]>([])
  const searchQuery = ref('')
  const error = ref('')
  const isLoading = ref(false)
  const mutatingDocumentId = ref<string | null>(null)

  const activeDocuments = computed(() => documents.value.filter((document) =>
    document.state === DocumentStateV1.DOCUMENT_STATE_ACTIVE
  ))
  const archivedDocuments = computed(() => documents.value.filter((document) =>
    document.state === DocumentStateV1.DOCUMENT_STATE_ARCHIVED
  ))

  async function loadAll(): Promise<void> {
    await loadPages('')
  }

  async function search(query: string): Promise<void> {
    await loadPages(query.trim())
  }

  async function select(document: DocumentV1): Promise<void> {
    isLoading.value = true
    error.value = ''
    try {
      selectedDocument.value = await getDocumentsQueryClient().get({
        logicalOwnerId: '', documentId: document.documentId
      })
      await loadSources(selectedDocument.value)
    } catch (cause) {
      error.value = message(cause)
      throw cause
    } finally {
      isLoading.value = false
    }
  }

  async function createDocument(input: {
    title: string
    description: string
    mediaType: string
    originalFileName: string
    declaredSize: bigint
    contentSha256: Uint8Array
  }): Promise<void> {
    await run(null, async () => {
      const result = await getDocumentsCommandClient().create({
        operationId: randomId16(),
        logicalOwnerId: '',
        ...input,
        createdAt: timestamp(new Date())
      })
      replaceResult(result.document)
    })
  }

  async function updateDocument(document: DocumentV1, input: {
    title?: string
    description?: string
    mediaType?: string
    originalFileName?: string
  }): Promise<void> {
    await mutate(document, () => getDocumentsCommandClient().update({
      operationId: randomId16(),
      documentId: document.documentId,
      logicalOwnerId: '',
      expectedDocumentRevision: document.documentRevision,
      ...input,
      updatedAt: timestamp(new Date())
    }))
  }

  async function setDocumentState(document: DocumentV1, state: DocumentStateV1): Promise<void> {
    await mutate(document, () => getDocumentsCommandClient().setState({
      operationId: randomId16(),
      documentId: document.documentId,
      logicalOwnerId: '',
      expectedDocumentRevision: document.documentRevision,
      state,
      changedAt: timestamp(new Date())
    }))
  }

  async function addSource(document: DocumentV1, input: {
    sourceOwnerId: string
    sourceRecordId: string
    sourceRevision: bigint
    evidenceDigest: Uint8Array
  }): Promise<void> {
    await mutate(document, () => getDocumentsCommandClient().addSource({
      operationId: randomId16(),
      documentId: document.documentId,
      logicalOwnerId: '',
      expectedDocumentRevision: document.documentRevision,
      ...input,
      changedAt: timestamp(new Date())
    }), true)
  }

  async function removeSource(document: DocumentV1, source: DocumentSourceV1): Promise<void> {
    await mutate(document, () => getDocumentsCommandClient().removeSource({
      operationId: randomId16(),
      documentId: document.documentId,
      logicalOwnerId: '',
      expectedDocumentRevision: document.documentRevision,
      sourceId: source.sourceId,
      changedAt: timestamp(new Date())
    }), true)
  }

  async function loadPages(query: string): Promise<void> {
    isLoading.value = true
    error.value = ''
    searchQuery.value = query
    try {
      const next: DocumentV1[] = []
      let cursor: Uint8Array<ArrayBufferLike> = new Uint8Array()
      for (let page = 0; page < 100; page += 1) {
        const result = query
          ? await getDocumentsQueryClient().search({
              logicalOwnerId: '', query, afterDocumentId: cursor, limit: PAGE_LIMIT
            })
          : await getDocumentsQueryClient().list({
              logicalOwnerId: '', afterDocumentId: cursor, limit: PAGE_LIMIT
            })
        next.push(...result.documents)
        if (result.nextAfterDocumentId.length === 0) break
        cursor = result.nextAfterDocumentId
      }
      documents.value = next
      if (selectedDocument.value) {
        const refreshed = next.find((document) => sameBytes(
          document.documentId, selectedDocument.value!.documentId
        ))
        if (refreshed) selectedDocument.value = refreshed
      }
    } catch (cause) {
      error.value = message(cause)
      throw cause
    } finally {
      isLoading.value = false
    }
  }

  async function loadSources(document: DocumentV1): Promise<void> {
    const next: DocumentSourceV1[] = []
    let cursor: Uint8Array<ArrayBufferLike> = new Uint8Array()
    for (let page = 0; page < 100; page += 1) {
      const result = await getDocumentsQueryClient().listSources({
        logicalOwnerId: '', documentId: document.documentId, afterSourceId: cursor, limit: PAGE_LIMIT
      })
      next.push(...result.sources)
      if (result.nextAfterSourceId.length === 0) break
      cursor = result.nextAfterSourceId
    }
    sources.value = next
  }

  async function mutate(
    document: DocumentV1,
    operation: () => Promise<{ document?: DocumentV1 }>,
    reloadSources = false
  ): Promise<void> {
    await run(hex(document.documentId), async () => {
      const result = await operation()
      const updated = replaceResult(result.document)
      if (reloadSources) await loadSources(updated)
    })
  }

  async function run(documentId: string | null, operation: () => Promise<void>): Promise<void> {
    mutatingDocumentId.value = documentId
    error.value = ''
    try {
      await operation()
    } catch (cause) {
      error.value = message(cause)
      throw cause
    } finally {
      mutatingDocumentId.value = null
    }
  }

  function replaceResult(document: DocumentV1 | undefined): DocumentV1 {
    if (!document) throw new Error('documents_invalid_response')
    const index = documents.value.findIndex((value) => sameBytes(value.documentId, document.documentId))
    if (index === -1) documents.value.push(document)
    else documents.value[index] = document
    documents.value.sort((left, right) => compareBytes(left.documentId, right.documentId))
    if (selectedDocument.value && sameBytes(selectedDocument.value.documentId, document.documentId)) {
      selectedDocument.value = document
    }
    return document
  }

  return {
    documents, selectedDocument, sources, searchQuery, error, isLoading, mutatingDocumentId,
    activeDocuments, archivedDocuments, loadAll, search, select, createDocument, updateDocument,
    setDocumentState, addSource, removeSource
  }
})

export function timestamp(value: Date): TimestampV1 {
  const milliseconds = value.getTime()
  return {
    $typeName: 'makosh.documents.client.v1.TimestampV1',
    unixSeconds: BigInt(Math.floor(milliseconds / 1_000)),
    nanos: Math.trunc(milliseconds % 1_000) * 1_000_000
  }
}

export function hex(value: Uint8Array): string {
  return Array.from(value, (byte) => byte.toString(16).padStart(2, '0')).join('')
}

export function parseDigest(value: string): Uint8Array {
  const normalized = value.trim().toLowerCase()
  if (!/^[0-9a-f]{64}$/.test(normalized)) throw new Error('documents_invalid_digest')
  return Uint8Array.from(normalized.match(/../g) ?? [], (byte) => Number.parseInt(byte, 16))
}

function randomId16(): Uint8Array {
  return globalThis.crypto.getRandomValues(new Uint8Array(16))
}

function sameBytes(left: Uint8Array, right: Uint8Array): boolean {
  return left.length === right.length && left.every((value, index) => value === right[index])
}

function compareBytes(left: Uint8Array, right: Uint8Array): number {
  const length = Math.min(left.length, right.length)
  for (let index = 0; index < length; index += 1) {
    const comparison = (left[index] ?? 0) - (right[index] ?? 0)
    if (comparison !== 0) return comparison
  }
  return left.length - right.length
}

function message(cause: unknown): string {
  return cause instanceof Error ? cause.message : 'documents_unavailable'
}
