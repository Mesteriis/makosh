<script setup lang="ts">
import { onMounted, reactive, ref } from 'vue'
import {
  DocumentCustodyStateV1,
  DocumentSourceStateV1,
  DocumentStateV1,
  type DocumentV1
} from '../../../gen/makosh/documents/client/v1/documents_pb'
import { Button, Card, Input, Select } from '../../../shared/ui'
import { useDocumentsPageSurface } from '../queries/useDocumentsPageSurface'
import { hex, parseDigest } from '../stores/documents'

const surface = useDocumentsPageSurface()
const query = ref('')
const createDraft = reactive({
  title: '', description: '', mediaType: '', originalFileName: '', declaredSize: '0', contentDigest: ''
})
const editDraft = reactive({ title: '', description: '', mediaType: '', originalFileName: '' })
const sourceDraft = reactive({
  sourceOwnerId: '', sourceRecordId: '', sourceRevision: '1', evidenceDigest: ''
})
const stateOptions = [
  { value: String(DocumentStateV1.DOCUMENT_STATE_ACTIVE), label: 'Active' },
  { value: String(DocumentStateV1.DOCUMENT_STATE_ARCHIVED), label: 'Archived' }
]

onMounted(() => { void surface.loadDocuments() })

async function createDocument(): Promise<void> {
  if (!createDraft.title.trim() || !createDraft.mediaType.trim() || !createDraft.originalFileName.trim()) return
  await surface.createDocument({
    title: createDraft.title.trim(),
    description: createDraft.description.trim(),
    mediaType: createDraft.mediaType.trim(),
    originalFileName: createDraft.originalFileName.trim(),
    declaredSize: BigInt(createDraft.declaredSize),
    contentSha256: parseDigest(createDraft.contentDigest)
  })
  Object.assign(createDraft, {
    title: '', description: '', mediaType: '', originalFileName: '', declaredSize: '0', contentDigest: ''
  })
}

async function selectDocument(document: DocumentV1): Promise<void> {
  await surface.select(document)
  const selected = surface.selectedDocument.value
  if (selected) Object.assign(editDraft, {
    title: selected.title,
    description: selected.description,
    mediaType: selected.mediaType,
    originalFileName: selected.originalFileName
  })
}

async function updateDocument(document: DocumentV1): Promise<void> {
  await surface.updateDocument(document, {
    title: editDraft.title.trim(),
    description: editDraft.description.trim(),
    mediaType: editDraft.mediaType.trim(),
    originalFileName: editDraft.originalFileName.trim()
  })
}

async function addSource(document: DocumentV1): Promise<void> {
  if (!sourceDraft.sourceOwnerId.trim() || !sourceDraft.sourceRecordId.trim()) return
  await surface.addSource(document, {
    sourceOwnerId: sourceDraft.sourceOwnerId.trim(),
    sourceRecordId: sourceDraft.sourceRecordId.trim(),
    sourceRevision: BigInt(sourceDraft.sourceRevision),
    evidenceDigest: parseDigest(sourceDraft.evidenceDigest)
  })
  Object.assign(sourceDraft, {
    sourceOwnerId: '', sourceRecordId: '', sourceRevision: '1', evidenceDigest: ''
  })
}

function custodyLabel(value: DocumentCustodyStateV1): string {
  if (value === DocumentCustodyStateV1.DOCUMENT_CUSTODY_STATE_BOUND) return 'Bound to owner custody'
  if (value === DocumentCustodyStateV1.DOCUMENT_CUSTODY_STATE_RELEASED) return 'Released'
  return 'Awaiting custody'
}
</script>

<template>
  <main class="documents-workspace" aria-label="Documents">
    <header class="documents-workspace__header">
      <div>
        <p class="documents-workspace__eyebrow">OWNER DOCUMENTS</p>
        <h1>Documents</h1>
        <p>{{ surface.activeDocuments.value.length }} active · {{ surface.archivedDocuments.value.length }} archived</p>
      </div>
      <Button variant="secondary" icon="tabler:refresh" :loading="surface.isLoading.value" @click="surface.loadDocuments">Refresh</Button>
    </header>

    <form class="documents-workspace__search" @submit.prevent="surface.search(query)">
      <Input v-model="query" aria-label="Search Documents" placeholder="Search owner documents…" />
      <Button type="submit" variant="outline" icon="tabler:search">Search</Button>
      <Button v-if="surface.searchQuery.value" type="button" variant="ghost" @click="query = ''; surface.loadDocuments()">Clear</Button>
    </form>

    <form class="documents-workspace__create" @submit.prevent="createDocument">
      <Input v-model="createDraft.title" aria-label="Document title" placeholder="Title" />
      <Input v-model="createDraft.description" aria-label="Document description" placeholder="Description" />
      <Input v-model="createDraft.mediaType" aria-label="Document media type" placeholder="application/pdf" />
      <Input v-model="createDraft.originalFileName" aria-label="Original file name" placeholder="filename.pdf" />
      <Input v-model="createDraft.declaredSize" aria-label="Declared size" type="number" min="0" placeholder="Size in bytes" />
      <Input v-model="createDraft.contentDigest" aria-label="Content digest" placeholder="64-character SHA-256" />
      <Button type="submit" icon="tabler:file-plus" :disabled="!createDraft.title.trim()">Create document</Button>
    </form>

    <p v-if="surface.error.value" class="documents-workspace__error" role="alert">{{ surface.error.value }}</p>
    <p v-if="surface.isLoading.value && surface.documents.value.length === 0" aria-live="polite">Loading Documents…</p>
    <p v-else-if="surface.documents.value.length === 0" class="documents-workspace__empty">No matching documents.</p>

    <section v-else class="documents-workspace__layout">
      <div class="documents-workspace__list" aria-label="Documents directory">
        <Card
          v-for="document in surface.documents.value"
          :key="hex(document.documentId)"
          class="documents-workspace__document"
          :selected="surface.selectedDocument.value !== undefined && hex(surface.selectedDocument.value.documentId) === hex(document.documentId)"
          @click="selectDocument(document)"
        >
          <h2>{{ document.title }}</h2>
          <p>{{ document.mediaType }} · {{ document.originalFileName }}</p>
          <small>{{ custodyLabel(document.custodyState) }} · revision {{ document.documentRevision }}</small>
          <Select
            :model-value="String(document.state)"
            :options="stateOptions"
            aria-label="Document lifecycle state"
            :disabled="surface.mutatingDocumentId.value === hex(document.documentId)"
            @click.stop
            @update:model-value="surface.setDocumentState(document, Number($event) as DocumentStateV1)"
          />
        </Card>
      </div>

      <Card v-if="surface.selectedDocument.value" class="documents-workspace__detail">
        <h2>{{ surface.selectedDocument.value.title }}</h2>
        <p>{{ custodyLabel(surface.selectedDocument.value.custodyState) }}</p>
        <dl>
          <div><dt>Media</dt><dd>{{ surface.selectedDocument.value.mediaType }}</dd></div>
          <div><dt>File</dt><dd>{{ surface.selectedDocument.value.originalFileName }}</dd></div>
          <div><dt>Size</dt><dd>{{ surface.selectedDocument.value.declaredSize }} bytes</dd></div>
          <div><dt>SHA-256</dt><dd>{{ hex(surface.selectedDocument.value.contentSha256) }}</dd></div>
        </dl>

        <form @submit.prevent="updateDocument(surface.selectedDocument.value)">
          <Input v-model="editDraft.title" aria-label="Edit document title" placeholder="Title" />
          <Input v-model="editDraft.description" aria-label="Edit document description" placeholder="Description" />
          <Input v-model="editDraft.mediaType" aria-label="Edit media type" placeholder="Media type" />
          <Input v-model="editDraft.originalFileName" aria-label="Edit file name" placeholder="File name" />
          <Button type="submit" variant="outline">Save metadata</Button>
        </form>

        <section>
          <h3>Public source provenance</h3>
          <ul>
            <li v-for="source in surface.sources.value" :key="hex(source.sourceId)">
              <span>{{ source.sourceOwnerId }} · {{ source.sourceRecordId }} · revision {{ source.sourceRevision }}</span>
              <Button
                v-if="source.state === DocumentSourceStateV1.DOCUMENT_SOURCE_STATE_ACTIVE"
                size="sm"
                variant="ghost"
                @click="surface.removeSource(surface.selectedDocument.value!, source)"
              >Remove</Button>
            </li>
          </ul>
          <form @submit.prevent="addSource(surface.selectedDocument.value)">
            <Input v-model="sourceDraft.sourceOwnerId" aria-label="Public source owner" placeholder="Source owner" />
            <Input v-model="sourceDraft.sourceRecordId" aria-label="Public source record" placeholder="Source record" />
            <Input v-model="sourceDraft.sourceRevision" aria-label="Public source revision" type="number" min="1" placeholder="Revision" />
            <Input v-model="sourceDraft.evidenceDigest" aria-label="Public evidence digest" placeholder="64-character evidence digest" />
            <Button type="submit" variant="outline" size="sm">Add source</Button>
          </form>
        </section>
      </Card>
    </section>
  </main>
</template>

<style scoped>
.documents-workspace { display: grid; gap: 1.25rem; width: min(78rem, 100%); margin: 0 auto; padding: 1.5rem; }
.documents-workspace__header, .documents-workspace__search, .documents-workspace__create, .documents-workspace__detail form { display: flex; gap: .75rem; align-items: center; }
.documents-workspace__header { justify-content: space-between; align-items: flex-start; }
.documents-workspace__header h1, .documents-workspace__document h2, .documents-workspace__detail h2, .documents-workspace__detail h3 { margin: 0; }
.documents-workspace__eyebrow { margin: 0 0 .25rem; font-size: .75rem; font-weight: 700; letter-spacing: .12em; }
.documents-workspace__header p, .documents-workspace__document p, .documents-workspace__document small { color: var(--text-secondary); }
.documents-workspace__search :deep(.makosh-input-wrapper), .documents-workspace__create :deep(.makosh-input-wrapper), .documents-workspace__detail :deep(.makosh-input-wrapper) { flex: 1; }
.documents-workspace__create { flex-wrap: wrap; padding: 1rem; border: 1px solid var(--border-subtle); border-radius: .75rem; }
.documents-workspace__layout { display: grid; grid-template-columns: minmax(18rem, 2fr) minmax(22rem, 3fr); gap: 1rem; align-items: start; }
.documents-workspace__list, .documents-workspace__detail { display: grid; gap: .75rem; }
.documents-workspace__document { display: grid; gap: .5rem; padding: 1rem; cursor: pointer; }
.documents-workspace__detail { padding: 1rem; }
.documents-workspace__detail dl { display: grid; gap: .5rem; }
.documents-workspace__detail dl div { display: grid; grid-template-columns: 6rem 1fr; gap: .5rem; }
.documents-workspace__detail dt { color: var(--text-secondary); }
.documents-workspace__detail dd { margin: 0; overflow-wrap: anywhere; }
.documents-workspace__detail section { display: grid; gap: .5rem; padding-top: .75rem; border-top: 1px solid var(--border-subtle); }
.documents-workspace__detail ul { display: grid; gap: .375rem; margin: 0; padding: 0; list-style: none; }
.documents-workspace__detail li { display: flex; justify-content: space-between; gap: .75rem; align-items: center; }
.documents-workspace__error { padding: .75rem 1rem; color: var(--status-error-text); background: var(--status-error-bg); border-radius: .75rem; }
.documents-workspace__empty { padding: 3rem; text-align: center; color: var(--text-secondary); }
@media (max-width: 820px) { .documents-workspace__layout { grid-template-columns: 1fr; } .documents-workspace__header, .documents-workspace__search, .documents-workspace__create, .documents-workspace__detail form { align-items: stretch; flex-direction: column; } }
</style>
