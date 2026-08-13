import { beforeEach, describe, expect, it, vi } from 'vitest'
import { PersonMatchCandidateDecisionV1 } from '../../../gen/makosh/review/person_match_candidate/v1/person_match_candidate_pb'
import {
  RelationshipParticipantKindV1,
  RelationshipStateV1,
  RelationshipTypeV1
} from '../../../gen/makosh/relationships/client/v1/relationships_pb'

const clients = vi.hoisted(() => ({
  persons: {
    listDirectory: vi.fn(),
    getProfile: vi.fn(),
    listSourceLinks: vi.fn()
  },
  reviewQuery: {
    list: vi.fn(),
    get: vi.fn()
  },
  reviewCommand: {
    decide: vi.fn()
  },
  relationshipsQuery: {
    listForParticipant: vi.fn()
  }
}))

vi.mock('../../../platform/connect/personsClient', () => ({
  getPersonsQueryClient: () => clients.persons
}))

vi.mock('../../../platform/connect/reviewPersonMatchCandidateClient', () => ({
  getReviewPersonMatchCandidateQueryClient: () => clients.reviewQuery,
  getReviewPersonMatchCandidateCommandClient: () => clients.reviewCommand
}))

vi.mock('../../../platform/connect/relationshipsClient', () => ({
  getRelationshipsQueryClient: () => clients.relationshipsQuery
}))

import { fetchPersonas, fetchRelationships, reviewIdentityCandidate } from './personas'

describe('Personas admitted adapter', () => {
  beforeEach(() => vi.clearAllMocks())

  it('hydrates every directory entry through exact profile and source-link RPCs', async () => {
    clients.persons.listDirectory.mockResolvedValue({
      persons: [{ personId: id(1), lifecycle: 2, personRevision: 3n, displayName: 'Directory' }]
    })
    clients.persons.getProfile.mockResolvedValue({
      personId: id(1),
      logicalOwnerId: 'owner-1',
      lifecycle: 2,
      personRevision: 3n,
      ownerProfile: { displayName: 'Profile' },
      createdAt: { unixSeconds: 10n, nanos: 0 },
      updatedAt: { unixSeconds: 20n, nanos: 0 }
    })
    clients.persons.listSourceLinks.mockResolvedValue({
      sourceLinks: [{ claims: { normalizedEmails: ['public@example.invalid'] } }]
    })

    const result = await fetchPersonas(1)

    expect(clients.persons.getProfile).toHaveBeenCalledWith({ logicalOwnerId: '', personId: id(1) })
    expect(clients.persons.listSourceLinks).toHaveBeenCalledWith({
      logicalOwnerId: '', personId: id(1), limit: 200
    })
    expect(result.items[0]?.identity).toEqual({
      display_name: 'Profile', email_address: 'public@example.invalid'
    })
  })

  it('performs Review Get then exact merge Decide with snapshot revisions and digest', async () => {
    clients.reviewQuery.get.mockResolvedValue({
      reviewId: id(9),
      reviewRevision: 7n,
      evidence: { firstPersonId: id(1), secondPersonId: id(2) }
    })
    clients.persons.getProfile
      .mockResolvedValueOnce({ personId: id(1), logicalOwnerId: 'owner-1', personRevision: 11n })
      .mockResolvedValueOnce({ personId: id(2), logicalOwnerId: 'owner-1', personRevision: 12n })
    clients.reviewCommand.decide.mockResolvedValue({})

    await reviewIdentityCandidate(hex(id(9)), 'user_confirmed')

    expect(clients.reviewQuery.get).toHaveBeenCalledWith({ logicalOwnerId: '', reviewId: id(9) })
    const request = clients.reviewCommand.decide.mock.calls[0]?.[0]
    expect(request.expectedReviewRevision).toBe(7n)
    expect(request.decision).toBe(PersonMatchCandidateDecisionV1.PERSON_MATCH_CANDIDATE_DECISION_APPROVE)
    expect(request.approvedAction?.action.case).toBe('merge')
    expect(request.approvedAction?.action.value).toMatchObject({
      sourcePersonId: id(1), expectedSourcePersonRevision: 11n,
      targetPersonId: id(2), expectedTargetPersonRevision: 12n
    })
    expect(request.approvedActionDigest).toHaveLength(32)
  })

  it('loads confirmed typed Relationships for the selected Person without scores', async () => {
    clients.relationshipsQuery.listForParticipant.mockResolvedValue({
      relationships: [{
        relationshipId: id(7),
        source: {
          kind: RelationshipParticipantKindV1.RELATIONSHIP_PARTICIPANT_KIND_PERSON,
          publicId: id(1)
        },
        target: {
          kind: RelationshipParticipantKindV1.RELATIONSHIP_PARTICIPANT_KIND_ORGANIZATION,
          publicId: id(2)
        },
        relationshipType: RelationshipTypeV1.RELATIONSHIP_TYPE_MEMBER_OF,
        state: RelationshipStateV1.RELATIONSHIP_STATE_CONFIRMED,
        validFrom: { unixSeconds: 10n, nanos: 0 },
        relationshipRevision: 3n
      }]
    })

    const result = await fetchRelationships({
      entityKind: 'persona', entityId: hex(id(1)), limit: 5
    })

    expect(clients.relationshipsQuery.listForParticipant).toHaveBeenCalledWith({
      logicalOwnerId: '',
      participant: {
        kind: RelationshipParticipantKindV1.RELATIONSHIP_PARTICIPANT_KIND_PERSON,
        publicId: id(1)
      },
      limit: 5
    })
    expect(result.items[0]).toEqual({
      relationship_id: hex(id(7)),
      source_entity_id: hex(id(1)),
      source_entity_kind: 'persona',
      target_entity_id: hex(id(2)),
      target_entity_kind: 'organization',
      relationship_type: 'member_of',
      state: 'confirmed',
      valid_from: '1970-01-01T00:00:10.000Z',
      valid_until: null,
      relationship_revision: 3
    })
  })
})

function id(seed: number): Uint8Array {
  return new Uint8Array(16).fill(seed)
}

function hex(value: Uint8Array): string {
  return Array.from(value, (byte) => byte.toString(16).padStart(2, '0')).join('')
}
