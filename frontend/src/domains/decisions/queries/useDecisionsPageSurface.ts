import { computed } from 'vue'
import { useDecisionsStore } from '../stores/decisions'

export function useDecisionsPageSurface() {
  const store = useDecisionsStore()
  return {
    decisions: computed(() => store.decisions),
    selectedDecision: computed(() => store.selectedDecision),
    alternatives: computed(() => store.alternatives),
    evidence: computed(() => store.evidence),
    drafts: computed(() => store.drafts),
    terminal: computed(() => store.terminal),
    error: computed(() => store.error),
    isLoading: computed(() => store.isLoading),
    loadAll: store.loadAll,
    select: store.select,
    createDecision: store.createDecision,
    addAlternative: store.addAlternative,
    decide: store.decide,
    cancel: store.cancel
  }
}
