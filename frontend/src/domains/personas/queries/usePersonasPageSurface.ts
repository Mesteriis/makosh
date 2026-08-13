import { computed, ref } from 'vue'
import { useI18n } from '../../../platform/i18n'
import {
  useIdentityCandidatesQuery,
  usePersonasQuery,
  useRelationshipsQuery,
  useReviewIdentityCandidateMutation
} from './usePersonasQuery'
import { usePersonasStore } from '../stores/personas'
import type {
  EnrichedPersona,
  PersonaDirectoryFilter,
  PersonaPanelProfile,
  PersonaIdentity,
  PersonaIdentityCandidate,
  PersonaIdentityReviewState,
  PersonaItem,
  PersonaWorkspaceSection
} from '../types/persona'

export function usePersonasPageSurface() {
  const { t } = useI18n()
  const store = usePersonasStore()
  const personaSearchQuery = ref('')
  const directoryFilter = ref<PersonaDirectoryFilter>('all')
  const activeSection = ref<PersonaWorkspaceSection>('overview')
  const unavailableActionError = ref('')
  const activeReviewCandidateId = ref<string | null>(null)

  const personasQuery = usePersonasQuery()
  const identityCandidatesQuery = useIdentityCandidatesQuery()
  const reviewIdentityCandidateMutation = useReviewIdentityCandidateMutation()

  const personas = computed(() => personasQuery.data.value ?? [])
  const ownerPersona = computed<PersonaPanelProfile | null>(() => null)
  const filteredPersonas = computed(() => {
    const query = personaSearchQuery.value.trim().toLowerCase()
    if (!query) return personas.value

    return personas.value.filter((persona) => {
      if (directoryFilter.value === 'address_book' && !persona.is_address_book) {
        return false
      }

      return [
        persona.display_name,
        persona.email_address,
        persona.language,
        persona.preferred_channel,
        persona.tone,
        persona.writing_style
      ].some((value) => value?.toLowerCase().includes(query))
    })
  })

  const personaList = computed<PersonaItem[]>(() =>
    filteredPersonas.value.map((persona) => ({
      persona_id: persona.persona_id,
      name: persona.display_name,
      role: persona.preferred_channel || t('Persona'),
      company: persona.email_address || t('No email'),
      status: persona.last_interaction_at ? t('Active') : undefined,
      channel: persona.preferred_channel ?? undefined
    }))
  )

  const selectedPersona = computed<PersonaPanelProfile | null>(() => {
    const selected = filteredPersonas.value[store.selectedPersonaIndex] ?? filteredPersonas.value[0]
    if (!selected) return null

    return {
      ...selected,
      is_owner: ownerPersona.value?.persona_id === selected.persona_id
    }
  })

  const selectedPersonaId = computed(() => selectedPersona.value?.persona_id ?? null)
  const relationshipsQuery = useRelationshipsQuery(selectedPersonaId)
  const identityTraces = computed<PersonaIdentity[]>(() => [])
  const relationships = computed(() => relationshipsQuery.data.value ?? [])
  const suggestedIdentityCandidates = computed(() =>
    (identityCandidatesQuery.data.value ?? []).filter(
      (item: PersonaIdentityCandidate) => item.review_state === 'suggested'
    )
  )
  const confirmedMergeIdentityCandidates = computed(() =>
    (identityCandidatesQuery.data.value ?? []).filter(
      (item: PersonaIdentityCandidate) =>
        item.candidate_kind === 'merge_personas' && item.review_state === 'user_confirmed'
    )
  )
  const directoryCount = computed(() => personas.value.length)
  const pendingReviewCount = computed(() => suggestedIdentityCandidates.value.length)
  const selectedPersonaRelationships = relationships
  const isLoading = computed(() => personasQuery.isLoading.value)
  const isRefreshing = computed(
    () =>
      personasQuery.isFetching.value ||
      identityCandidatesQuery.isFetching.value ||
      relationshipsQuery.isFetching.value
  )
  const actionError = computed(() => unavailableActionError.value)
  const settingOwnerPersonaId = computed(() => null)
  const reviewingCandidateId = computed(() => activeReviewCandidateId.value)
  const assigningTraceId = computed(() => null)

  function identityConfidence(item: PersonaIdentityCandidate | PersonaIdentity): string {
    return `${Math.round(item.confidence * 100)}%`
  }

  function languageLabel(language: string | null | undefined): string {
    if (!language) return t('Not set')
    const labels: Record<string, string> = {
      ru: t('Russian'),
      en: t('English')
    }
    return labels[language.toLowerCase()] ?? language
  }

  function trustScoreLabel(score: number | null | undefined): string {
    if (score === null || score === undefined) return t('No score')
    return `${score}/100`
  }

  function personaInitials(persona: Pick<EnrichedPersona, 'display_name' | 'email_address'>): string {
    const source = persona.display_name || persona.email_address || '?'
    return source
      .split(/\s+/)
      .filter(Boolean)
      .slice(0, 2)
      .map((part) => part.slice(0, 1))
      .join('')
  }

  function traceTitle(trace: PersonaIdentity): string {
    return trace.identity_value || trace.identity_type
  }

  function traceKindLabel(trace: PersonaIdentity): string {
    return identityKindLabel(trace.identity_type)
  }

  function candidateTitle(candidate: PersonaIdentityCandidate): string {
    if (candidate.candidate_kind === 'attach_email_address' && candidate.email_address) {
      return candidate.email_address
    }

    if (candidate.candidate_kind === 'merge_personas') {
      return t('Possible duplicate persona')
    }

    return candidate.candidate_kind
  }

  function candidateKindLabel(candidate: PersonaIdentityCandidate): string {
    const labels: Record<string, string> = {
      attach_email_address: t('Email candidate'),
      merge_personas: t('Merge candidate'),
      split_persona: t('Split candidate')
    }
    return labels[candidate.candidate_kind] ?? candidate.candidate_kind
  }

  function formatDateTime(value: string | null | undefined): string {
    if (!value) return t('Never')
    const date = new Date(value)
    if (Number.isNaN(date.getTime())) return t('Unknown')
    return new Intl.DateTimeFormat(undefined, {
      month: 'short',
      day: 'numeric',
      hour: '2-digit',
      minute: '2-digit'
    }).format(date)
  }

  function selectPersona(index: number): void {
    store.selectPersona(index)
  }

  async function refresh(): Promise<void> {
    await Promise.all([
      personasQuery.refetch(),
      identityCandidatesQuery.refetch(),
      relationshipsQuery.refetch()
    ])
  }

  function setOwnerPersona(_persona: EnrichedPersona): void {
    unavailableActionError.value = 'persons_owner_profile_not_configured'
  }

  function toggleAddressBookMembership(
    _persona: EnrichedPersona,
    _value: boolean
  ): void {
    unavailableActionError.value = 'persons_address_book_membership_retired'
  }

  async function setIdentityCandidateReview(
    candidate: PersonaIdentityCandidate,
    state: PersonaIdentityReviewState
  ): Promise<void> {
    unavailableActionError.value = ''
    activeReviewCandidateId.value = candidate.identity_candidate_id
    try {
      await reviewIdentityCandidateMutation.mutateAsync({
        candidateId: candidate.identity_candidate_id,
        reviewState: state
      })
    } catch (error) {
      unavailableActionError.value = error instanceof Error ? error.message : 'review_action_failed'
    } finally {
      activeReviewCandidateId.value = null
    }
  }

  function assignTraceToOwner(_trace: PersonaIdentity): void {
    unavailableActionError.value = 'identity_resolution_projection_unavailable'
  }

  function assignTraceToSelectedPersona(_trace: PersonaIdentity): void {
    unavailableActionError.value = 'identity_resolution_projection_unavailable'
  }

  function isSettingOwner(personaId: string): boolean {
    void personaId
    return false
  }

  function isReviewingCandidate(candidateId: string): boolean {
    return activeReviewCandidateId.value === candidateId
  }

  function isAssigningTrace(traceId: string): boolean {
    void traceId
    return false
  }

  function splitConfirmedIdentityMerge(candidate: PersonaIdentityCandidate) {
    return setIdentityCandidateReview(candidate, 'suggested')
  }

  function splitCandidateForConfirmedMerge(): PersonaIdentityCandidate | null {
    return null
  }

  return {
    activeSection,
    actionError,
    assigningTraceId,
    assignTraceToOwner,
    assignTraceToSelectedPersona,
    candidateKindLabel,
    candidateTitle,
    confirmedMergeIdentityCandidates,
    directoryCount,
    directoryFilter,
    filteredPersonas,
    formatDateTime,
    identityConfidence,
    identityTraces,
    isAssigningTrace,
    isLoading,
    isRefreshing,
    isReviewingCandidate,
    isSettingOwner,
    languageLabel,
    ownerPersona,
    pendingReviewCount,
    personaInitials,
    personaList,
    personaSearchQuery,
    personas,
    refresh,
    relationships,
    reviewingCandidateId,
    selectedPersona,
    selectedPersonaRelationships,
    selectedPersonaId,
    selectPersona,
    setIdentityCandidateReview,
    setOwnerPersona,
    settingOwnerPersonaId,
    splitCandidateForConfirmedMerge,
    splitConfirmedIdentityMerge,
    store,
    suggestedIdentityCandidates,
    toggleAddressBookMembership,
    traceKindLabel,
    traceTitle,
    trustScoreLabel
  }
}

function identityKindLabel(kind: string): string {
  const labels: Record<string, string> = {
    email: 'Email',
    phone: 'Phone',
    telegram: 'Telegram',
    whatsapp: 'WhatsApp',
    social: 'Social',
    name: 'Name',
    organization: 'Organization'
  }
  return labels[kind] ?? kind
}
