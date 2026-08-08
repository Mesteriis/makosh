// --- Task Candidate types ---
export type TaskCandidateReviewState = 'suggested' | 'user_confirmed' | 'user_rejected'

export interface TaskCandidate {
  task_candidate_id: string
  source_kind: 'message' | 'document'
  source_id: string
  project_id: string | null
  title: string
  due_text: string | null
  assignee_label: string | null
  confidence: number
  review_state: TaskCandidateReviewState
  evidence_excerpt: string
  generated_at: string
  reviewed_at: string | null
  updated_at: string
}

export interface TaskCandidateListResponse {
  items: TaskCandidate[]
}

// --- Task types ---
export interface Task {
  task_id: string
  task_candidate_id: string | null
  title: string
  description: string | null
  source_kind: string
  source_id: string
  source_type: string
  project_id: string | null
  status: string
  makosh_status: string
  priority_score: number | null
  risk_score: number | null
  readiness_score: number | null
  area: string | null
  why: string | null
  outcome: string | null
  due_at: string | null
  completed_at: string | null
  archived_at: string | null
  waiting_reason: string | null
  energy_type: string | null
  confidentiality: string
  tags: unknown[]
  task_metadata: Record<string, unknown>
  linked_person_id: string | null
  linked_organization_id: string | null
  created_from_event_id: string | null
  created_by_actor_id: string | null
  created_at: string
  updated_at: string
}

export interface TaskRecordsResponse {
  items: Task[]
}

// --- Decision types ---
export type DecisionEntityKind =
  | 'persona' | 'organization' | 'project' | 'communication'
  | 'document' | 'task' | 'event' | 'decision' | 'obligation' | 'knowledge'

export type DecisionReviewState = 'suggested' | 'user_confirmed' | 'user_rejected'

export interface Decision {
  decision_id: string
  title: string
  status: string
  rationale: string
  alternatives: unknown
  decided_by_entity_kind: DecisionEntityKind | null
  decided_by_entity_id: string | null
  decided_at: string | null
  review_state: DecisionReviewState
  confidence: number
  metadata: Record<string, unknown>
  created_at: string
  updated_at: string
}

export interface DecisionListResponse {
  items: Decision[]
}

export interface DecisionReviewRequest {
  review_state: Exclude<DecisionReviewState, 'suggested'>
}

// --- Obligation types ---
export type ObligationEntityKind =
  | 'persona' | 'organization' | 'project' | 'communication'
  | 'document' | 'task' | 'event' | 'decision' | 'obligation' | 'knowledge'

export type ObligationReviewState = 'suggested' | 'user_confirmed' | 'user_rejected'
export type ObligationRiskState = 'none' | 'watch' | 'at_risk' | 'breached'

export interface Obligation {
  obligation_id: string
  obligated_entity_kind: ObligationEntityKind
  obligated_entity_id: string
  beneficiary_entity_kind: ObligationEntityKind | null
  beneficiary_entity_id: string | null
  statement: string
  status: string
  review_state: ObligationReviewState
  due_at: string | null
  condition: string | null
  risk_state: ObligationRiskState
  confidence: number
  metadata: Record<string, unknown>
  created_at: string
  updated_at: string
}

export interface ObligationListResponse {
  items: Obligation[]
}

export interface ObligationReviewRequest {
  review_state: Exclude<ObligationReviewState, 'suggested'>
}
