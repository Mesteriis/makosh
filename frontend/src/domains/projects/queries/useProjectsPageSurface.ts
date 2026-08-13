import { computed } from 'vue'
import { useProjectsStore } from '../stores/projects'

export function useProjectsPageSurface() {
  const store = useProjectsStore()
  return {
    projects: computed(() => store.projects),
    selectedProject: computed(() => store.selectedProject),
    outcomes: computed(() => store.outcomes),
    references: computed(() => store.references),
    activeProjects: computed(() => store.activeProjects),
    completedProjects: computed(() => store.completedProjects),
    error: computed(() => store.error),
    isLoading: computed(() => store.isLoading),
    mutatingProjectId: computed(() => store.mutatingProjectId),
    loadProjects: store.loadAll,
    select: store.select,
    createProject: store.createProject,
    updateProject: store.updateProject,
    setProjectState: store.setProjectState,
    addOutcome: store.addOutcome,
    updateOutcome: store.updateOutcome,
    setOutcomeState: store.setOutcomeState,
    removeOutcome: store.removeOutcome,
    addReference: store.addReference,
    removeReference: store.removeReference,
    store
  }
}
