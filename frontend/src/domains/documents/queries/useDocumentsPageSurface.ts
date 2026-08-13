import { computed } from 'vue'
import { useDocumentsStore } from '../stores/documents'

export function useDocumentsPageSurface() {
  const store = useDocumentsStore()
  return {
    documents: computed(() => store.documents),
    selectedDocument: computed(() => store.selectedDocument),
    sources: computed(() => store.sources),
    searchQuery: computed(() => store.searchQuery),
    error: computed(() => store.error),
    isLoading: computed(() => store.isLoading),
    mutatingDocumentId: computed(() => store.mutatingDocumentId),
    activeDocuments: computed(() => store.activeDocuments),
    archivedDocuments: computed(() => store.archivedDocuments),
    loadDocuments: store.loadAll,
    search: store.search,
    select: store.select,
    createDocument: store.createDocument,
    updateDocument: store.updateDocument,
    setDocumentState: store.setDocumentState,
    addSource: store.addSource,
    removeSource: store.removeSource,
    store
  }
}
