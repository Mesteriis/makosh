import {
  PersonLifecycleV1,
  PersonsQueryService,
  type PersonDirectoryEntryV1,
  type PersonProfileResultV1
} from '../../../gen/makosh/persons/v1/persons_pb'
import {
  PersonMatchCandidateDecisionV1,
  PersonMatchCandidateStateV1,
  ReviewPersonMatchCandidateQueryService,
  type PersonMatchCandidateSummaryV1
} from '../../../gen/makosh/review/person_match_candidate/v1/person_match_candidate_pb'
import {
  RelationshipParticipantKindV1,
  RelationshipStateV1,
  RelationshipTypeV1,
  type RelationshipV1
} from '../../../gen/makosh/relationships/client/v1/relationships_pb'
import { getPersonsQueryClient } from '../../../platform/connect/personsClient'
import {
  getReviewPersonMatchCandidateCommandClient,
  getReviewPersonMatchCandidateQueryClient
} from '../../../platform/connect/reviewPersonMatchCandidateClient'
import { getRelationshipsQueryClient } from '../../../platform/connect/relationshipsClient'
import type {
  EnrichedPersona,
  OwnerPersona,
  PersonaDossier,
  PersonaIdentityCandidate,
  PersonaIdentity,
  PersonaIdentityReviewState,
  PersonaReadModel,
  Relationship
} from '../types/persona'

export type PersonaListResponse = { items: PersonaReadModel[] }
export type OwnerPersonaResponse = { owner_persona: OwnerPersona | null }
export type PersonaIdentityCandidateListResponse = { items: PersonaIdentityCandidate[] }
export type PersonaIdentityTraceListResponse = { items: PersonaIdentity[] }
export type RelationshipListResponse = { items: Relationship[] }

export async function fetchPersonas(limit = 50): Promise<PersonaListResponse> {
  const client = getPersonsQueryClient()
  const result = await client.listDirectory({ logicalOwnerId: '', limit })
  const items = await Promise.all(result.persons.map(async (entry) => {
    const [profile, sources] = await Promise.all([
      client.getProfile({ logicalOwnerId: '', personId: entry.personId }),
      client.listSourceLinks({ logicalOwnerId: '', personId: entry.personId, limit: 200 })
    ])
    return directoryEntry(entry, profile, sources.sourceLinks[0]?.claims?.normalizedEmails[0] ?? null)
  }))
  return { items }
}

export async function fetchOwnerPersona(): Promise<OwnerPersonaResponse> {
  return { owner_persona: null }
}

export async function setOwnerPersona(_personaId: string): Promise<OwnerPersonaResponse> {
  throw unavailable('persons_owner_profile_not_configured')
}

export async function updatePersonaAddressBookMembership(
  _personaId: string,
  _isAddressBook: boolean
): Promise<OwnerPersona> {
  throw unavailable('persons_address_book_membership_retired')
}

export async function fetchPersonaDossier(_personaId: string): Promise<PersonaDossier> {
  throw unavailable('persons_dossier_unavailable')
}

export async function fetchIdentityCandidates(limit = 50): Promise<PersonaIdentityCandidateListResponse> {
  const result = await getReviewPersonMatchCandidateQueryClient().list({
    logicalOwnerId: '',
    limit
  })
  return { items: result.candidates.map(reviewCandidate) }
}

export async function reviewIdentityCandidate(
  identityCandidateId: string,
  reviewState: PersonaIdentityReviewState
): Promise<void> {
  if (reviewState === 'suggested') throw unavailable('review_action_requires_decision')
  const reviewId = unhex16(identityCandidateId)
  const review = await getReviewPersonMatchCandidateQueryClient().get({
    logicalOwnerId: '',
    reviewId
  })
  const approve = reviewState === 'user_confirmed'
  let approvedAction: {
    action: {
      case: 'merge'
      value: {
        sourcePersonId: Uint8Array
        expectedSourcePersonRevision: bigint
        targetPersonId: Uint8Array
        expectedTargetPersonRevision: bigint
      }
    }
  } | undefined
  let approvedActionDigest = new Uint8Array()
  if (approve) {
    const evidence = review.evidence
    if (!evidence) throw unavailable('review_evidence_unavailable')
    const persons = getPersonsQueryClient()
    const [source, target] = await Promise.all([
      persons.getProfile({ logicalOwnerId: '', personId: evidence.firstPersonId }),
      persons.getProfile({ logicalOwnerId: '', personId: evidence.secondPersonId })
    ])
    if (!source.logicalOwnerId || source.logicalOwnerId !== target.logicalOwnerId) {
      throw unavailable('review_owner_snapshot_conflict')
    }
    approvedAction = {
      action: {
        case: 'merge',
        value: {
          sourcePersonId: source.personId,
          expectedSourcePersonRevision: source.personRevision,
          targetPersonId: target.personId,
          expectedTargetPersonRevision: target.personRevision
        }
      }
    }
    approvedActionDigest = await mergeActionDigest(
      source.logicalOwnerId,
      source.personId,
      source.personRevision,
      target.personId,
      target.personRevision
    )
  }
  await getReviewPersonMatchCandidateCommandClient().decide({
    protocolMajor: 1,
    operationId: randomId16(),
    reviewId,
    expectedReviewRevision: review.reviewRevision,
    decision: approve
      ? PersonMatchCandidateDecisionV1.PERSON_MATCH_CANDIDATE_DECISION_APPROVE
      : PersonMatchCandidateDecisionV1.PERSON_MATCH_CANDIDATE_DECISION_REJECT,
    approvedAction,
    approvedActionDigest
  })
}

export async function fetchIdentityTraces(_limit = 50): Promise<PersonaIdentityTraceListResponse> {
  return { items: [] }
}

export async function assignIdentityTrace(_traceId: string, _personaId: string): Promise<void> {
  throw unavailable('identity_resolution_projection_unavailable')
}

export function normalizePersonaReadModel(persona: PersonaReadModel | OwnerPersona): EnrichedPersona {
  const displayName = 'identity' in persona ? persona.identity.display_name : persona.display_name
  const emailAddress = 'identity' in persona ? persona.identity.email_address : persona.email_address
  return {
    persona_id: persona.persona_id,
    display_name: displayName,
    email_address: emailAddress,
    language: null,
    tone: null,
    trust_score: null,
    avg_response_hours: null,
    preferred_channel: emailAddress ? 'mail' : null,
    last_interaction_at: null,
    interaction_count: 0,
    frequent_topics: [],
    writing_style: null,
    persona_metadata: {},
    is_favorite: false,
    is_address_book: persona.is_address_book ?? false,
    notes: null,
    linked_projects: [],
    linked_documents: [],
    created_at: persona.created_at,
    updated_at: persona.updated_at
  }
}

export function normalizeOwnerPersona(persona: PersonaReadModel | OwnerPersona): OwnerPersona {
  const normalized = normalizePersonaReadModel(persona)
  return {
    persona_id: normalized.persona_id,
    display_name: normalized.display_name,
    email_address: normalized.email_address,
    persona_type: 'human',
    is_self: false,
    is_address_book: normalized.is_address_book,
    created_at: normalized.created_at,
    updated_at: normalized.updated_at
  }
}

export async function fetchRelationships(params: {
  entityKind: 'persona'
  entityId: string
  limit?: number
}): Promise<RelationshipListResponse> {
  const result = await getRelationshipsQueryClient().listForParticipant({
    logicalOwnerId: '',
    participant: {
      kind: RelationshipParticipantKindV1.RELATIONSHIP_PARTICIPANT_KIND_PERSON,
      publicId: unhex16(params.entityId)
    },
    limit: params.limit ?? 50
  })
  return { items: result.relationships.map(relationshipReadModel) }
}

function relationshipReadModel(value: RelationshipV1): Relationship {
  return {
    relationship_id: hex(value.relationshipId),
    source_entity_id: hex(value.source?.publicId ?? new Uint8Array()),
    source_entity_kind: relationshipParticipantKind(value.source?.kind),
    target_entity_id: hex(value.target?.publicId ?? new Uint8Array()),
    target_entity_kind: relationshipParticipantKind(value.target?.kind),
    relationship_type: relationshipType(value.relationshipType),
    state: value.state === RelationshipStateV1.RELATIONSHIP_STATE_ENDED ? 'ended' : 'confirmed',
    valid_from: wireTimestamp(value.validFrom),
    valid_until: value.validUntil == null ? null : wireTimestamp(value.validUntil),
    relationship_revision: Number(value.relationshipRevision)
  }
}

function relationshipParticipantKind(value: RelationshipParticipantKindV1 | undefined): 'persona' | 'organization' {
  return value === RelationshipParticipantKindV1.RELATIONSHIP_PARTICIPANT_KIND_ORGANIZATION
    ? 'organization'
    : 'persona'
}

function relationshipType(value: RelationshipTypeV1): Relationship['relationship_type'] {
  const values: Partial<Record<RelationshipTypeV1, Relationship['relationship_type']>> = {
    [RelationshipTypeV1.RELATIONSHIP_TYPE_FAMILY]: 'family',
    [RelationshipTypeV1.RELATIONSHIP_TYPE_FRIEND]: 'friend',
    [RelationshipTypeV1.RELATIONSHIP_TYPE_COLLEAGUE]: 'colleague',
    [RelationshipTypeV1.RELATIONSHIP_TYPE_REPORTS_TO]: 'reports_to',
    [RelationshipTypeV1.RELATIONSHIP_TYPE_MEMBER_OF]: 'member_of',
    [RelationshipTypeV1.RELATIONSHIP_TYPE_PARTNER]: 'partner'
  }
  return values[value] ?? 'colleague'
}

function directoryEntry(
  person: PersonDirectoryEntryV1,
  profile: PersonProfileResultV1,
  sourceEmail: string | null
): PersonaReadModel {
  const createdAt = wireTimestamp(profile.createdAt)
  const updatedAt = wireTimestamp(profile.updatedAt)
  const email = profile.ownerProfile?.normalizedEmails?.[0] ?? sourceEmail
  return {
    persona_id: hex(person.personId),
    persona_type: 'human',
    is_self: false,
    is_address_book: person.lifecycle !== PersonLifecycleV1.PERSON_LIFECYCLE_ARCHIVED,
    identity: {
      display_name: profile.ownerProfile?.displayName || person.displayName || 'Unnamed person',
      email_address: email
    },
    communication: { primary_email: email },
    created_at: createdAt,
    updated_at: updatedAt
  }
}

function reviewCandidate(candidate: PersonMatchCandidateSummaryV1): PersonaIdentityCandidate {
  const evidence = candidate.evidence
  const state = candidate.state === PersonMatchCandidateStateV1.PERSON_MATCH_CANDIDATE_STATE_PENDING
    ? 'suggested'
    : candidate.state === PersonMatchCandidateStateV1.PERSON_MATCH_CANDIDATE_STATE_APPROVED
      ? 'user_confirmed'
      : 'user_rejected'
  return {
    identity_candidate_id: hex(candidate.reviewId),
    candidate_kind: 'person_match',
    left_persona_id: hex(evidence?.firstPersonId ?? new Uint8Array()),
    right_persona_id: hex(evidence?.secondPersonId ?? new Uint8Array()) || null,
    email_address: null,
    evidence_summary: 'Public provider identity match',
    confidence: 1,
    review_state: state,
    generated_at: millis(evidence?.observedAtUnixMillis),
    reviewed_at: candidate.decidedAtUnixMillis == null ? null : millis(candidate.decidedAtUnixMillis),
    updated_at: millis(candidate.decidedAtUnixMillis ?? evidence?.observedAtUnixMillis)
  }
}

function hex(value: Uint8Array): string {
  return Array.from(value, (byte) => byte.toString(16).padStart(2, '0')).join('')
}

function millis(value: bigint | undefined): string {
  const numeric = value == null ? 0 : Number(value)
  return new Date(Number.isSafeInteger(numeric) ? numeric : 0).toISOString()
}

function wireTimestamp(value: { unixSeconds: bigint; nanos: number } | undefined): string {
  if (!value) return new Date(0).toISOString()
  const millis = Number(value.unixSeconds) * 1_000 + Math.trunc(value.nanos / 1_000_000)
  return new Date(Number.isSafeInteger(millis) ? millis : 0).toISOString()
}

function unhex16(value: string): Uint8Array {
  if (!/^[0-9a-f]{32}$/.test(value)) throw unavailable('invalid_public_id')
  return Uint8Array.from(value.match(/.{2}/g) ?? [], (part) => Number.parseInt(part, 16))
}

function randomId16(): Uint8Array {
  return globalThis.crypto.getRandomValues(new Uint8Array(16))
}

async function mergeActionDigest(
  owner: string,
  source: Uint8Array,
  sourceRevision: bigint,
  target: Uint8Array,
  targetRevision: bigint
): Promise<Uint8Array<ArrayBuffer>> {
  const encoder = new TextEncoder()
  const chunks = [
    encoder.encode('makosh.persons.confirmed-action.v1'),
    lengthPrefixed(encoder.encode('merge-persons')),
    lengthPrefixed(encoder.encode(owner)),
    lengthPrefixed(source),
    u64(sourceRevision),
    lengthPrefixed(target),
    u64(targetRevision)
  ]
  const bytes = new Uint8Array(chunks.reduce((total, chunk) => total + chunk.length, 0))
  let offset = 0
  for (const chunk of chunks) {
    bytes.set(chunk, offset)
    offset += chunk.length
  }
  return new Uint8Array(await globalThis.crypto.subtle.digest('SHA-256', bytes))
}

function lengthPrefixed(value: Uint8Array): Uint8Array {
  const bytes = new Uint8Array(8 + value.length)
  new DataView(bytes.buffer).setBigUint64(0, BigInt(value.length))
  bytes.set(value, 8)
  return bytes
}

function u64(value: bigint): Uint8Array {
  if (value <= 0n) throw unavailable('invalid_revision')
  const bytes = new Uint8Array(8)
  new DataView(bytes.buffer).setBigUint64(0, value)
  return bytes
}

function unavailable(code: string): Error {
  return new Error(code)
}

void PersonsQueryService
void ReviewPersonMatchCandidateQueryService
void (null as PersonProfileResultV1 | null)
