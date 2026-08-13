export type PersonaType = 'human' | 'ai_agent' | 'organization_proxy' | 'system'

export interface PersonaProfile {
  persona_id: string
  display_name: string
  email_address: string | null
  language: string | null
  tone: string | null
  trust_score: number | null
  avg_response_hours: number | null
  preferred_channel: string | null
  last_interaction_at: string | null
  interaction_count: number
  frequent_topics: string[]
  writing_style: string | null
  persona_metadata: Record<string, unknown>
  is_favorite: boolean
  is_address_book: boolean
  notes: string | null
  linked_projects: string[]
  linked_documents: string[]
  created_at: string
  updated_at: string
}

export type EnrichedPersona = PersonaProfile

export interface OwnerPersona {
  persona_id: string
  display_name: string
  email_address: string | null
  persona_type: PersonaType
  is_self: boolean
  is_address_book: boolean
  created_at: string
  updated_at: string
}

export interface PersonaReadModel {
  persona_id: string
  persona_type: PersonaType
  is_self: boolean
  is_address_book: boolean
  identity: {
    display_name: string
    email_address: string | null
  }
  communication: {
    primary_email: string | null
  }
  created_at: string
  updated_at: string
}

export type PersonaPanelProfile = EnrichedPersona & {
  is_owner: boolean
}

export type PersonaWorkspaceSection =
  | 'overview'
  | 'communications'
  | 'context'
  | 'tasks'
  | 'documents'
  | 'notes'
  | 'relationships'
  | 'timeline'
  | 'dossier'

export type PersonaDirectoryFilter = 'all' | 'address_book'

export interface PersonaDossier {
  persona: PersonaProfile
  summary: string
  source_refs: string[]
  generated_at: string
}

export interface PersonaIdentityCandidate {
  identity_candidate_id: string
  candidate_kind: string
  left_persona_id: string
  right_persona_id: string | null
  email_address: string | null
  evidence_summary: string
  confidence: number
  review_state: string
  generated_at: string
  reviewed_at: string | null
  updated_at: string
}

export type PersonaIdentityReviewState = 'suggested' | 'user_confirmed' | 'user_rejected'

export interface PersonaIdentity {
  id: string
  persona_id: string | null
  identity_type: string
  identity_value: string
  source: string
  confidence: number
  last_verified_at: string | null
  status: string
  metadata: Record<string, unknown>
  created_at: string
  updated_at: string
}

export interface Relationship {
  relationship_id: string
  source_entity_id: string
  source_entity_kind: string
  target_entity_id: string
  target_entity_kind: string
  relationship_type: 'family' | 'friend' | 'colleague' | 'reports_to' | 'member_of' | 'partner'
  state: 'confirmed' | 'ended'
  valid_from: string
  valid_until: string | null
  relationship_revision: number
}

export interface PersonaItem {
  persona_id: string
  name: string
  role: string
  company: string
  channel?: string
  status?: string
}

export interface PersonaOption {
  persona_id: string
  name: string
  company: string
}
