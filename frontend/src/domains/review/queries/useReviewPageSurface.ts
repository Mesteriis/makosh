import { computed } from 'vue'
import { useReviewStore } from '../stores/review'

export function useReviewPageSurface() {
  const store = useReviewStore()

  const attention = computed(() => store.attention)
  const personMatchCandidates = computed(() => store.personMatchCandidates)
  const taskCandidates = computed(() => store.taskCandidates)
  const noteCandidates = computed(() => store.noteCandidates)
  const obligationCandidates = computed(() => store.obligationCandidates)

  async function loadReviewWorkspace(): Promise<void> {
    await store.loadAll()
  }

  return {
    attention,
    personMatchCandidates,
    taskCandidates,
    noteCandidates,
    obligationCandidates,
    loadReviewWorkspace,
    resolveAttention: store.resolveAttention,
    decidePersonMatchCandidate: store.decidePersonMatchCandidate,
    decideTaskCandidate: store.decideTaskCandidate,
    decideNoteCandidate: store.decideNoteCandidate,
    decideObligationCandidate: store.decideObligationCandidate,
    store
  }
}
