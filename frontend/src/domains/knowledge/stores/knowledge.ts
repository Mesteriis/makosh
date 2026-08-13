import { computed, ref } from 'vue'
import { defineStore } from 'pinia'
import {
  KnowledgeNoteStateV1,
  type KnowledgeNoteV1,
  type KnowledgeSourceV1,
  type TimestampV1
} from '../../../gen/makosh/knowledge/client/v1/knowledge_pb'
import {
  getKnowledgeCommandClient,
  getKnowledgeQueryClient
} from '../../../platform/connect/knowledgeClient'

const PAGE_LIMIT = 50

export const useKnowledgeStore = defineStore('knowledge', () => {
  const notes = ref<KnowledgeNoteV1[]>([])
  const sourcesByNote = ref<Record<string, KnowledgeSourceV1[]>>({})
  const searchQuery = ref('')
  const error = ref('')
  const isLoading = ref(false)
  const mutatingNoteId = ref<string | null>(null)

  const activeNotes = computed(() => notes.value.filter((note) =>
    note.state === KnowledgeNoteStateV1.KNOWLEDGE_NOTE_STATE_ACTIVE
  ))
  const archivedNotes = computed(() => notes.value.filter((note) =>
    note.state === KnowledgeNoteStateV1.KNOWLEDGE_NOTE_STATE_ARCHIVED
  ))

  async function loadAll(): Promise<void> {
    await loadPages('')
  }

  async function search(query: string): Promise<void> {
    const normalized = query.trim()
    searchQuery.value = normalized
    await loadPages(normalized)
  }

  async function loadSources(note: KnowledgeNoteV1): Promise<void> {
    const loaded: KnowledgeSourceV1[] = []
    let cursor: Uint8Array<ArrayBufferLike> = new Uint8Array()
    do {
      const page = await getKnowledgeQueryClient().listSources({
        logicalOwnerId: '',
        noteId: note.noteId,
        afterSourceId: cursor,
        limit: PAGE_LIMIT
      })
      loaded.push(...page.sources)
      cursor = page.nextAfterSourceId
    } while (cursor.length > 0)
    sourcesByNote.value[hex(note.noteId)] = loaded
  }

  async function createNote(title: string, body: string): Promise<void> {
    await run(null, async () => {
      const result = await getKnowledgeCommandClient().create({
        operationId: randomId16(),
        noteId: new Uint8Array(),
        logicalOwnerId: '',
        title,
        body,
        createdAt: timestamp(new Date())
      })
      replaceResult(result.note)
    })
  }

  async function updateNote(note: KnowledgeNoteV1, title: string, body: string): Promise<void> {
    await run(hex(note.noteId), async () => {
      const result = await getKnowledgeCommandClient().update({
        operationId: randomId16(),
        noteId: note.noteId,
        logicalOwnerId: '',
        expectedNoteRevision: note.noteRevision,
        title,
        body,
        updatedAt: timestamp(new Date())
      })
      replaceResult(result.note)
    })
  }

  async function setNoteState(note: KnowledgeNoteV1, state: KnowledgeNoteStateV1): Promise<void> {
    await run(hex(note.noteId), async () => {
      const result = await getKnowledgeCommandClient().setState({
        operationId: randomId16(),
        noteId: note.noteId,
        logicalOwnerId: '',
        expectedNoteRevision: note.noteRevision,
        state,
        changedAt: timestamp(new Date())
      })
      replaceResult(result.note)
    })
  }

  async function addSource(
    note: KnowledgeNoteV1,
    sourceOwnerId: string,
    sourceRecordId: Uint8Array,
    sourceRevision: bigint,
    evidenceDigest: Uint8Array
  ): Promise<void> {
    await run(hex(note.noteId), async () => {
      const result = await getKnowledgeCommandClient().addSource({
        operationId: randomId16(),
        noteId: note.noteId,
        logicalOwnerId: '',
        expectedNoteRevision: note.noteRevision,
        sourceId: new Uint8Array(),
        sourceOwnerId,
        sourceRecordId,
        sourceRevision,
        evidenceDigest,
        changedAt: timestamp(new Date())
      })
      replaceResult(result.note)
      if (result.note) await loadSources(result.note)
    })
  }

  async function removeSource(note: KnowledgeNoteV1, source: KnowledgeSourceV1): Promise<void> {
    await run(hex(note.noteId), async () => {
      const result = await getKnowledgeCommandClient().removeSource({
        operationId: randomId16(),
        noteId: note.noteId,
        logicalOwnerId: '',
        expectedNoteRevision: note.noteRevision,
        sourceId: source.sourceId,
        changedAt: timestamp(new Date())
      })
      replaceResult(result.note)
      if (result.note) await loadSources(result.note)
    })
  }

  async function loadPages(query: string): Promise<void> {
    isLoading.value = true
    error.value = ''
    try {
      const loaded: KnowledgeNoteV1[] = []
      let cursor: Uint8Array<ArrayBufferLike> = new Uint8Array()
      do {
        const page = query
          ? await getKnowledgeQueryClient().search({
              logicalOwnerId: '', query, afterNoteId: cursor, limit: PAGE_LIMIT
            })
          : await getKnowledgeQueryClient().list({
              logicalOwnerId: '', afterNoteId: cursor, limit: PAGE_LIMIT
            })
        loaded.push(...page.notes)
        cursor = page.nextAfterNoteId
      } while (cursor.length > 0)
      notes.value = loaded
    } catch (cause) {
      error.value = message(cause)
    } finally {
      isLoading.value = false
    }
  }

  async function run(noteId: string | null, operation: () => Promise<void>): Promise<void> {
    mutatingNoteId.value = noteId
    error.value = ''
    try {
      await operation()
    } catch (cause) {
      error.value = message(cause)
      throw cause
    } finally {
      mutatingNoteId.value = null
    }
  }

  function replaceResult(note: KnowledgeNoteV1 | undefined): void {
    if (!note) throw new Error('knowledge_invalid_response')
    const index = notes.value.findIndex((value) => sameBytes(value.noteId, note.noteId))
    if (index === -1) notes.value.push(note)
    else notes.value[index] = note
    notes.value.sort((left, right) => compareBytes(left.noteId, right.noteId))
  }

  return {
    notes,
    sourcesByNote,
    searchQuery,
    error,
    isLoading,
    mutatingNoteId,
    activeNotes,
    archivedNotes,
    loadAll,
    search,
    loadSources,
    createNote,
    updateNote,
    setNoteState,
    addSource,
    removeSource
  }
})

function timestamp(value: Date): TimestampV1 {
  const milliseconds = value.getTime()
  return {
    $typeName: 'makosh.knowledge.client.v1.TimestampV1',
    unixSeconds: BigInt(Math.floor(milliseconds / 1_000)),
    nanos: Math.trunc(milliseconds % 1_000) * 1_000_000
  }
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

export function hex(value: Uint8Array): string {
  return Array.from(value, (byte) => byte.toString(16).padStart(2, '0')).join('')
}

function message(cause: unknown): string {
  return cause instanceof Error ? cause.message : 'knowledge_unavailable'
}
