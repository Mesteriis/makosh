import { computed } from 'vue'
import { useKnowledgeStore } from '../stores/knowledge'

export function useKnowledgePageSurface() {
  const store = useKnowledgeStore()
  return {
    notes: computed(() => store.notes),
    activeNotes: computed(() => store.activeNotes),
    archivedNotes: computed(() => store.archivedNotes),
    sourcesByNote: computed(() => store.sourcesByNote),
    searchQuery: computed(() => store.searchQuery),
    error: computed(() => store.error),
    isLoading: computed(() => store.isLoading),
    mutatingNoteId: computed(() => store.mutatingNoteId),
    loadNotes: store.loadAll,
    search: store.search,
    loadSources: store.loadSources,
    createNote: store.createNote,
    updateNote: store.updateNote,
    setNoteState: store.setNoteState,
    addSource: store.addSource,
    removeSource: store.removeSource,
    store
  }
}
