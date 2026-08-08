<script setup lang="ts">
import { useI18n } from '../../../platform/i18n'
import { Avatar, Icon, SearchInput, ToggleGroup } from '../../../shared/ui'
import type { EnrichedPersona, PersonaDirectoryFilter, PersonaPanelProfile } from '../types/persona'
import {
  isOwnerPersona,
  personaDirectoryRowClass,
  personaInitials
} from './personaWorkspaceElements'

const props = withDefaults(defineProps<{
  ownerPersona: PersonaPanelProfile | null
  selectedPersona: PersonaPanelProfile | null
  filteredPersonas: readonly EnrichedPersona[]
  directoryCount: number
  searchQuery: string
  directoryFilter?: PersonaDirectoryFilter
  isLoading?: boolean
}>(), {
  directoryFilter: 'all',
  isLoading: false
})

const emit = defineEmits<{
  selectPersona: [index: number]
  'update:directoryFilter': [value: PersonaDirectoryFilter]
  'update:searchQuery': [value: string]
}>()

const { t } = useI18n()

const directoryFilterItems: readonly {
  value: PersonaDirectoryFilter
  label: string
  icon: string
}[] = [
  { value: 'all', label: t('All'), icon: 'tabler:users' },
  { value: 'address_book', label: t('Contacts'), icon: 'tabler:address-book' }
]

function updateDirectoryFilter(value: string | string[]): void {
  if (value === 'all' || value === 'address_book') {
    emit('update:directoryFilter', value)
  }
}
</script>

<template>
  <aside class="personas-side-panel" :aria-label="t('Persona directory')">
    <section class="personas-directory-panel">
      <header>
        <div>
          <span>{{ t('Directory') }}</span>
          <strong>{{ filteredPersonas.length }}/{{ directoryCount }}</strong>
        </div>
        <SearchInput
          :model-value="searchQuery"
          :aria-label="t('Search personas')"
          :clear-label="t('Clear search')"
          :placeholder="t('Search personas, companies, emails...')"
          @update:model-value="emit('update:searchQuery', $event)"
        />
        <ToggleGroup
          class="personas-directory-filter makosh-toggle-group--tabs"
          :model-value="directoryFilter"
          :items="directoryFilterItems"
          :aria-label="t('Directory filter')"
          @update:model-value="updateDirectoryFilter"
        />
      </header>

      <div v-if="isLoading" class="personas-loading">
        <Icon icon="tabler:loader-2" />
        <span>{{ t('Loading') }}</span>
      </div>

      <div v-else-if="filteredPersonas.length === 0" class="personas-empty-compact">
        <Icon icon="tabler:users-off" />
        <span>{{ t('No personas yet') }}</span>
      </div>

      <div v-else class="personas-directory-list">
        <button
          v-for="(persona, index) in filteredPersonas"
          :key="persona.persona_id"
          type="button"
          class="personas-directory-row"
          :class="personaDirectoryRowClass(persona, selectedPersona, ownerPersona, index)"
          @click="emit('selectPersona', index)"
        >
          <Avatar
            size="md"
            :fallback="personaInitials(persona)"
            :alt="persona.display_name"
          />
          <span>
            <strong>{{ persona.display_name }}</strong>
            <small>{{ persona.email_address || t('No email') }}</small>
          </span>
          <em v-if="isOwnerPersona(persona, ownerPersona)">{{ t('Me') }}</em>
          <Icon
            v-else-if="persona.is_address_book"
            icon="tabler:address-book"
            class="personas-directory-address-book-icon"
            :aria-label="t('Contacts')"
          />
        </button>
      </div>
    </section>
  </aside>
</template>
