import { computed } from 'vue'
import { useProjectsStore } from '../stores/projects'

export function useProjectsQuery() {
  const store = useProjectsStore()
  return {
    data: computed(() => store.projects),
    error: computed(() => store.error ? new Error(store.error) : null),
    isLoading: computed(() => store.isLoading),
    refetch: store.loadAll
  }
}

export function useProjectQuery() {
  const store = useProjectsStore()
  return {
    data: computed(() => store.selectedProject),
    outcomes: computed(() => store.outcomes),
    references: computed(() => store.references),
    isLoading: computed(() => store.isLoading)
  }
}
