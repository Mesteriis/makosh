import { computed } from 'vue'
import { useObligationsStore } from '../stores/obligations'

export function useObligationsPageSurface() {
  const store = useObligationsStore()

  return {
    obligations: computed(() => store.obligations),
    evidenceByObligation: computed(() => store.evidenceByObligation),
    openObligations: computed(() => store.openObligations),
    completedObligations: computed(() => store.completedObligations),
    error: computed(() => store.error),
    isLoading: computed(() => store.isLoading),
    mutatingObligationId: computed(() => store.mutatingObligationId),
    loadObligations: store.loadAll,
    loadEvidence: store.loadEvidence,
    updateObligation: store.updateObligation,
    setObligationState: store.setObligationState,
    addEvidence: store.addEvidence,
    removeEvidence: store.removeEvidence,
    store
  }
}
