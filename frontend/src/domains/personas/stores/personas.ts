import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import type {
  PersonaDossier,
  PersonaIdentityCandidate,
  PersonaIdentity,
  PersonaIdentityReviewState,
  Relationship
} from '../types/persona'
import { reviewIdentityCandidate, assignIdentityTrace } from '../api/personas'

export function formatIdentityTraceKind(kind: string): string {
  const labels: Record<string, string> = {
    email: 'Email',
    phone: 'Phone',
    telegram: 'Telegram',
    whatsapp: 'WhatsApp',
    social: 'Social Profile',
    name: 'Name',
    organization: 'Organization'
  }
  return labels[kind] || kind
}

export function formatIdentityTraceValue(trace: PersonaIdentity): string {
  return trace.identity_value || trace.identity_type
}

export function identityTraceConfidence(trace: PersonaIdentity): string {
  return `${Math.round(trace.confidence * 100)}%`
}

export function formatRelationshipType(type: string): string {
  const labels: Record<string, string> = {
    colleague: 'Colleague',
    manager: 'Manager',
    client: 'Client',
    partner: 'Partner',
    vendor: 'Vendor',
    friend: 'Friend',
    family: 'Family',
    acquaintance: 'Acquaintance',
    competitor: 'Competitor',
    other: 'Other'
  }
  return labels[type] || type
}

export function formatRelationshipScore(score: number): string {
  if (score >= 0.8) return 'Strong'
  if (score >= 0.5) return 'Moderate'
  return 'Weak'
}

export function formatRelationshipEndpoint(kind: string, id: string): string {
  return `${kind}:${id.slice(0, 8)}...`
}

export function personaIdentityPairKey(leftPersonaId: string, rightPersonaId: string): string {
  return leftPersonaId <= rightPersonaId
    ? `${leftPersonaId}:${rightPersonaId}`
    : `${rightPersonaId}:${leftPersonaId}`
}

export function dossierSectionPreview(dossier: PersonaDossier): string[] {
  const words = (dossier.summary || '').split(/\s+/).filter(Boolean)
  return [...new Set(words.slice(0, 10))]
}

export const usePersonasStore = defineStore('personas', () => {
  const selectedPersonaIndex = ref(0)
  const loadedDossierPersonaId = ref<string | null>(null)
  const personaDossier = ref<PersonaDossier | null>(null)
  const personaDossierError = ref('')
  const isPersonaDossierLoading = ref(false)
  const identityCandidatesError = ref('')
  const identityTracesError = ref('')
  const assigningIdentityTraceId = ref<string | null>(null)

  const identityCandidates = ref<PersonaIdentityCandidate[]>([])
  const identityTraces = ref<PersonaIdentity[]>([])
  const relationships = ref<Relationship[]>([])

  const suggestedIdentityCandidates = computed(() =>
    identityCandidates.value.filter((item) => item.review_state === 'suggested')
  )

  function setIdentityCandidates(items: PersonaIdentityCandidate[]) {
    identityCandidates.value = items
  }

  function setIdentityTraces(items: PersonaIdentity[]) {
    identityTraces.value = items
  }

  function setRelationships(items: Relationship[]) {
    relationships.value = items
  }

  function selectPersona(index: number) {
    selectedPersonaIndex.value = index
  }

  function setPersonaDossier(dossier: PersonaDossier | null, error: string) {
    personaDossier.value = dossier
    personaDossierError.value = error
  }

  function setPersonaDossierLoading(loading: boolean) {
    isPersonaDossierLoading.value = loading
  }

  function setLoadedDossierPersonaId(id: string | null) {
    loadedDossierPersonaId.value = id
  }

  async function reviewCandidate(candidate: PersonaIdentityCandidate, state: PersonaIdentityCandidate['review_state']) {
    if (!isReviewableIdentityCandidateState(state)) return
    try {
      await reviewIdentityCandidate(candidate.identity_candidate_id, state)
    } catch (e: any) {
      identityCandidatesError.value = e.message || 'Review failed'
}

function isReviewableIdentityCandidateState(
  state: PersonaIdentityCandidate['review_state']
): state is PersonaIdentityReviewState {
  return state === 'user_confirmed' || state === 'user_rejected'
}
  }

  async function assignTraceToPersona(trace: PersonaIdentity, personaId: string) {
    assigningIdentityTraceId.value = trace.id
    try {
      await assignIdentityTrace(trace.id, personaId)
    } catch (e: any) {
      identityTracesError.value = e.message || 'Assignment failed'
    }
    assigningIdentityTraceId.value = null
  }

  return {
    selectedPersonaIndex,
    personaDossier,
    personaDossierError,
    isPersonaDossierLoading,
    identityCandidatesError,
    identityTracesError,
    assigningIdentityTraceId,
    suggestedIdentityCandidates,
    identityCandidates,
    identityTraces,
    relationships,
    loadedDossierPersonaId,
    setIdentityCandidates,
    setIdentityTraces,
    setRelationships,
    selectPersona,
    setPersonaDossier,
    setPersonaDossierLoading,
    setLoadedDossierPersonaId,
    reviewCandidate,
    assignTraceToPersona
  }
})
