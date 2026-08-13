<script setup lang="ts">
import { computed } from 'vue'
import PersonasPage from '../presentation/PersonasPage.vue'
import type { PersonasPageActions, PersonasPageModel } from '../presentation/personasPageModel'
import { usePersonasPageSurface } from '../queries/usePersonasPageSurface'

const surface = usePersonasPageSurface()

const model = computed<PersonasPageModel>(() => ({
	ownerPersona: surface.ownerPersona.value,
	selectedPersona: surface.selectedPersona.value,
	filteredPersonas: surface.filteredPersonas.value,
	directoryCount: surface.directoryCount.value,
	pendingReviewCount: surface.pendingReviewCount.value,
	directoryFilter: surface.directoryFilter.value,
	activeSection: surface.activeSection.value,
	searchQuery: surface.personaSearchQuery.value,
	suggestedIdentityCandidates: surface.suggestedIdentityCandidates.value,
	identityTraces: surface.identityTraces.value,
	selectedPersonaRelationships: surface.selectedPersonaRelationships.value,
	isLoading: surface.isLoading.value,
	isRefreshing: surface.isRefreshing.value,
	actionError: surface.actionError.value,
	settingOwnerPersonaId: surface.settingOwnerPersonaId.value,
	reviewingCandidateId: surface.reviewingCandidateId.value,
	assigningTraceId: surface.assigningTraceId.value,
}))

const actions: PersonasPageActions = {
	refresh: surface.refresh,
	selectPersona: surface.selectPersona,
	setOwner: surface.setOwnerPersona,
	reviewCandidate: surface.setIdentityCandidateReview,
	assignTraceToOwner: surface.assignTraceToOwner,
	assignTraceToSelectedPersona: surface.assignTraceToSelectedPersona,
	toggleAddressBook: surface.toggleAddressBookMembership,
	setDirectoryFilter: (value) => { surface.directoryFilter.value = value },
	setActiveSection: (value) => { surface.activeSection.value = value },
	setSearchQuery: (value) => { surface.personaSearchQuery.value = value },
}
</script>

<template>
	<PersonasPage :model="model" :actions="actions" />
</template>
