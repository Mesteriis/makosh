import { computed } from 'vue'
import { useOrganizationsStore } from '../stores/organizations'

export function useOrganizationsPageSurface() {
  const store = useOrganizationsStore()
  return {
    organizations: computed(() => store.organizations),
    selectedOrganization: computed(() => store.selectedOrganization),
    sources: computed(() => store.sources),
    searchQuery: computed(() => store.searchQuery),
    error: computed(() => store.error),
    isLoading: computed(() => store.isLoading),
    mutatingOrganizationId: computed(() => store.mutatingOrganizationId),
    activeOrganizations: computed(() => store.activeOrganizations),
    archivedOrganizations: computed(() => store.archivedOrganizations),
    loadOrganizations: store.loadAll,
    search: store.search,
    select: store.select,
    createOrganization: store.createOrganization,
    updateOrganization: store.updateOrganization,
    setOrganizationState: store.setOrganizationState,
    addSource: store.addSource,
    removeSource: store.removeSource,
    store
  }
}
