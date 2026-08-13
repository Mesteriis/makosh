<script setup lang="ts">
import { onMounted, reactive, ref } from 'vue'
import {
  KnowledgeNoteStateV1,
  KnowledgeSourceStateV1,
  type KnowledgeNoteV1
} from '../../../gen/makosh/knowledge/client/v1/knowledge_pb'
import { Button, Card, Input } from '../../../shared/ui'
import { useKnowledgePageSurface } from '../queries/useKnowledgePageSurface'
import { hex } from '../stores/knowledge'

const surface = useKnowledgePageSurface()
const title = ref('')
const body = ref('')
const query = ref('')
const sourceDrafts = reactive<Record<string, { owner: string; record: string; digest: string }>>({})

onMounted(() => { void surface.loadNotes() })

async function createNote(): Promise<void> {
  const normalizedTitle = title.value.trim()
  const normalizedBody = body.value.trim()
  if (!normalizedTitle || !normalizedBody) return
  await surface.createNote(normalizedTitle, normalizedBody)
  title.value = ''
  body.value = ''
}

async function addSource(note: KnowledgeNoteV1): Promise<void> {
  const key = hex(note.noteId)
  const value = draft(note)
  if (!value.owner.trim()) return
  await surface.addSource(note, value.owner.trim(), parseHex(value.record, 16), 1n, parseHex(value.digest, 32))
  sourceDrafts[key] = { owner: '', record: '', digest: '' }
}

function draft(note: KnowledgeNoteV1): { owner: string; record: string; digest: string } {
  const key = hex(note.noteId)
  return sourceDrafts[key] ??= { owner: '', record: '', digest: '' }
}

function isMutating(note: KnowledgeNoteV1): boolean {
  return surface.mutatingNoteId.value === hex(note.noteId)
}

function parseHex(value: string, size: number): Uint8Array {
  const normalized = value.trim().toLowerCase()
  if (!new RegExp(`^[0-9a-f]{${size * 2}}$`).test(normalized)) throw new Error('knowledge_public_id_invalid')
  return Uint8Array.from(normalized.match(/.{2}/g) ?? [], (byte) => Number.parseInt(byte, 16))
}
</script>

<template>
  <main class="knowledge-workspace" aria-label="Knowledge">
    <header class="knowledge-workspace__header">
      <div>
        <p class="knowledge-workspace__eyebrow">OWNER KNOWLEDGE</p>
        <h1>Knowledge</h1>
        <p>{{ surface.activeNotes.value.length }} active · {{ surface.archivedNotes.value.length }} archived</p>
      </div>
      <Button variant="secondary" icon="tabler:refresh" :loading="surface.isLoading.value" @click="surface.loadNotes">Refresh</Button>
    </header>

    <form class="knowledge-workspace__search" @submit.prevent="surface.search(query)">
      <Input v-model="query" aria-label="Search knowledge" placeholder="Search owner notes…" />
      <Button type="submit" variant="outline" icon="tabler:search">Search</Button>
      <Button v-if="surface.searchQuery.value" type="button" variant="ghost" @click="query = ''; surface.loadNotes()">Clear</Button>
    </form>

    <form class="knowledge-workspace__create" @submit.prevent="createNote">
      <Input v-model="title" aria-label="Knowledge note title" placeholder="Note title" />
      <textarea v-model="body" aria-label="Knowledge note body" placeholder="Verified or owner-authored knowledge…" />
      <Button type="submit" icon="tabler:plus" :disabled="!title.trim() || !body.trim()">Create note</Button>
    </form>

    <p v-if="surface.error.value" class="knowledge-workspace__error" role="alert">{{ surface.error.value }}</p>
    <p v-if="surface.isLoading.value && surface.notes.value.length === 0" aria-live="polite">Loading Knowledge…</p>
    <p v-else-if="surface.notes.value.length === 0" class="knowledge-workspace__empty">No matching notes.</p>

    <section v-else class="knowledge-workspace__list" aria-label="Knowledge notes">
      <Card v-for="note in surface.notes.value" :key="hex(note.noteId)" class="knowledge-workspace__note">
        <div class="knowledge-workspace__note-header">
          <div>
            <h2>{{ note.title }}</h2>
            <small>Revision {{ note.noteRevision }} · {{ note.origin === 1 ? 'reviewed' : 'owner-authored' }}</small>
          </div>
          <Button
            variant="outline"
            size="sm"
            :disabled="isMutating(note)"
            @click="surface.setNoteState(note, note.state === KnowledgeNoteStateV1.KNOWLEDGE_NOTE_STATE_ACTIVE ? KnowledgeNoteStateV1.KNOWLEDGE_NOTE_STATE_ARCHIVED : KnowledgeNoteStateV1.KNOWLEDGE_NOTE_STATE_ACTIVE)"
          >{{ note.state === KnowledgeNoteStateV1.KNOWLEDGE_NOTE_STATE_ACTIVE ? 'Archive' : 'Restore' }}</Button>
        </div>
        <p class="knowledge-workspace__body">{{ note.body }}</p>

        <div class="knowledge-workspace__sources">
          <Button variant="ghost" size="sm" @click="surface.loadSources(note)">Load public sources</Button>
          <ul v-if="surface.sourcesByNote.value[hex(note.noteId)]?.length">
            <li v-for="source in surface.sourcesByNote.value[hex(note.noteId)]" :key="hex(source.sourceId)">
              <span>{{ source.sourceOwnerId }} · rev {{ source.sourceRevision }}</span>
              <Button
                v-if="source.state === KnowledgeSourceStateV1.KNOWLEDGE_SOURCE_STATE_ACTIVE"
                variant="ghost"
                size="sm"
                :disabled="isMutating(note)"
                @click="surface.removeSource(note, source)"
              >Remove</Button>
            </li>
          </ul>
          <form class="knowledge-workspace__source-add" @submit.prevent="addSource(note)">
            <Input v-model="draft(note).owner" aria-label="Public source owner" placeholder="Source owner" />
            <Input v-model="draft(note).record" aria-label="Public source record ID" placeholder="32 hex record ID" />
            <Input v-model="draft(note).digest" aria-label="Public source evidence digest" placeholder="64 hex evidence digest" />
            <Button type="submit" variant="outline" size="sm" :disabled="isMutating(note)">Attach source</Button>
          </form>
        </div>
      </Card>
    </section>
  </main>
</template>

<style scoped>
.knowledge-workspace { display: grid; gap: 1.25rem; width: min(72rem, 100%); margin: 0 auto; padding: 1.5rem; }
.knowledge-workspace__header, .knowledge-workspace__note-header, .knowledge-workspace__search, .knowledge-workspace__source-add { display: flex; gap: .75rem; align-items: center; }
.knowledge-workspace__header, .knowledge-workspace__note-header { justify-content: space-between; align-items: flex-start; }
.knowledge-workspace__header h1, .knowledge-workspace__note h2 { margin: 0; }
.knowledge-workspace__header p, .knowledge-workspace__note small { color: var(--text-secondary); }
.knowledge-workspace__eyebrow { margin: 0 0 .25rem; font-size: .75rem; font-weight: 700; letter-spacing: .12em; }
.knowledge-workspace__search :deep(.makosh-input-wrapper), .knowledge-workspace__source-add :deep(.makosh-input-wrapper) { flex: 1; }
.knowledge-workspace__create { display: grid; gap: .75rem; padding: 1rem; border: 1px solid var(--border-subtle); border-radius: .75rem; }
.knowledge-workspace__create textarea { min-height: 7rem; resize: vertical; padding: .75rem; border: 1px solid var(--border-subtle); border-radius: .5rem; color: inherit; background: var(--surface-raised); }
.knowledge-workspace__error { padding: .75rem 1rem; color: var(--status-error-text); background: var(--status-error-bg); border-radius: .75rem; }
.knowledge-workspace__empty { padding: 3rem; text-align: center; color: var(--text-secondary); }
.knowledge-workspace__list, .knowledge-workspace__note, .knowledge-workspace__sources { display: grid; gap: 1rem; }
.knowledge-workspace__note { padding: 1rem; }
.knowledge-workspace__body { white-space: pre-wrap; }
.knowledge-workspace__sources ul { display: grid; gap: .5rem; margin: 0; padding: 0; list-style: none; }
.knowledge-workspace__sources li { display: flex; justify-content: space-between; align-items: center; gap: .75rem; }
@media (max-width: 760px) { .knowledge-workspace__header, .knowledge-workspace__note-header, .knowledge-workspace__search, .knowledge-workspace__source-add { align-items: stretch; flex-direction: column; } }
</style>
