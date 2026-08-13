import { beforeEach, describe, expect, it, vi } from 'vitest'
import { createPinia, setActivePinia } from 'pinia'
import { ReviewDispositionV1 } from '../../../gen/makosh/review/attention/client/v1/client_pb'
import { PersonMatchCandidateStateV1 } from '../../../gen/makosh/review/person_match_candidate/v1/person_match_candidate_pb'
import {
  ReviewTaskCandidateDecisionV1,
  ReviewTaskCandidateStateV1
} from '../../../gen/makosh/review/task_candidate/v1/task_candidate_pb'
import {
  ReviewNoteCandidateDecisionV1,
  ReviewNoteCandidateStateV1
} from '../../../gen/makosh/review/note_candidate/v1/note_candidate_pb'
import {
  ReviewObligationCandidateDecisionV1,
  ReviewObligationCandidateStateV1
} from '../../../gen/makosh/review/obligation_candidate/v1/obligation_candidate_pb'

const clients = vi.hoisted(() => ({
  attentionQuery: { query: vi.fn() },
  attentionCommand: { execute: vi.fn() },
  personQuery: { list: vi.fn() },
  personCommand: { decide: vi.fn() },
  taskQuery: { list: vi.fn() },
  taskCommand: { decide: vi.fn() },
  noteQuery: { list: vi.fn() },
  noteCommand: { decide: vi.fn() },
  obligationQuery: { list: vi.fn() },
  obligationCommand: { decide: vi.fn() }
}))

vi.mock('../../../platform/connect/reviewAttentionClient', () => ({
  getReviewAttentionQueryClient: () => clients.attentionQuery,
  getReviewAttentionCommandClient: () => clients.attentionCommand
}))
vi.mock('../../../platform/connect/reviewPersonMatchCandidateClient', () => ({
  getReviewPersonMatchCandidateQueryClient: () => clients.personQuery,
  getReviewPersonMatchCandidateCommandClient: () => clients.personCommand
}))
vi.mock('../../../platform/connect/reviewTaskCandidateClient', () => ({
  getReviewTaskCandidateQueryClient: () => clients.taskQuery,
  getReviewTaskCandidateCommandClient: () => clients.taskCommand
}))
vi.mock('../../../platform/connect/reviewNoteCandidateClient', () => ({
  getReviewNoteCandidateQueryClient: () => clients.noteQuery,
  getReviewNoteCandidateCommandClient: () => clients.noteCommand
}))
vi.mock('../../../platform/connect/reviewObligationCandidateClient', () => ({
  getReviewObligationCandidateQueryClient: () => clients.obligationQuery,
  getReviewObligationCandidateCommandClient: () => clients.obligationCommand
}))

import { useReviewStore } from './review'

describe('typed Review owner store', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
    vi.clearAllMocks()
    clients.attentionQuery.query.mockResolvedValue({
      result: { case: 'page', value: { attention: [attention()] } },
      errorCode: ''
    })
    clients.personQuery.list.mockResolvedValue({ candidates: [person()] })
    clients.taskQuery.list.mockResolvedValue({ reviews: [task()], error: 0 })
    clients.noteQuery.list.mockResolvedValue({ reviews: [note()], error: 0 })
    clients.obligationQuery.list.mockResolvedValue({ reviews: [obligation()], error: 0 })
  })

  it('loads exactly the five authoritative Review slices through typed clients', async () => {
    const store = useReviewStore()
    await store.loadAll()

    expect(store.error).toBe('')
    expect(store.attention).toHaveLength(1)
    expect(store.personMatchCandidates).toHaveLength(1)
    expect(store.taskCandidates).toHaveLength(1)
    expect(store.noteCandidates).toHaveLength(1)
    expect(store.obligationCandidates).toHaveLength(1)
    expect(store.totalPendingCount).toBe(5)
    expect(clients.taskQuery.list).toHaveBeenCalledWith({
      protocolMajor: 1,
      state: ReviewTaskCandidateStateV1.REVIEW_TASK_CANDIDATE_STATE_PENDING,
      afterReviewId: new Uint8Array(),
      limit: 50
    })
  })

  it('dispatches Task, Note and Obligation decisions with exact revisions', async () => {
    clients.taskCommand.decide.mockResolvedValue({
      review: { ...task(), state: ReviewTaskCandidateStateV1.REVIEW_TASK_CANDIDATE_STATE_REJECTED, reviewRevision: 2n },
      error: 0
    })
    clients.noteCommand.decide.mockResolvedValue({
      review: { ...note(), state: ReviewNoteCandidateStateV1.REVIEW_NOTE_CANDIDATE_STATE_REJECTED, reviewRevision: 2n },
      error: 0
    })
    clients.obligationCommand.decide.mockResolvedValue({
      review: { ...obligation(), state: ReviewObligationCandidateStateV1.REVIEW_OBLIGATION_CANDIDATE_STATE_REJECTED, reviewRevision: 2n },
      error: 0
    })
    const store = useReviewStore()
    await store.loadAll()

    await store.decideTaskCandidate(
      store.taskCandidates[0]!,
      ReviewTaskCandidateDecisionV1.REVIEW_TASK_CANDIDATE_DECISION_REJECT
    )
    await store.decideNoteCandidate(
      store.noteCandidates[0]!,
      ReviewNoteCandidateDecisionV1.REVIEW_NOTE_CANDIDATE_DECISION_REJECT
    )
    await store.decideObligationCandidate(
      store.obligationCandidates[0]!,
      ReviewObligationCandidateDecisionV1.REVIEW_OBLIGATION_CANDIDATE_DECISION_REJECT
    )

    expect(clients.taskCommand.decide.mock.calls[0]?.[0]).toMatchObject({
      reviewId: id(3),
      expectedReviewRevision: 1n,
      decision: ReviewTaskCandidateDecisionV1.REVIEW_TASK_CANDIDATE_DECISION_REJECT
    })
    expect(clients.noteCommand.decide.mock.calls[0]?.[0]).toMatchObject({
      reviewId: id(4),
      expectedReviewRevision: 1n,
      decision: ReviewNoteCandidateDecisionV1.REVIEW_NOTE_CANDIDATE_DECISION_REJECT
    })
    expect(store.taskCandidates[0]?.reviewRevision).toBe(2n)
    expect(store.noteCandidates[0]?.reviewRevision).toBe(2n)
    expect(clients.obligationCommand.decide.mock.calls[0]?.[0]).toMatchObject({
      reviewId: id(5),
      expectedReviewRevision: 1n,
      decision: ReviewObligationCandidateDecisionV1.REVIEW_OBLIGATION_CANDIDATE_DECISION_REJECT
    })
    expect(store.obligationCandidates[0]?.reviewRevision).toBe(2n)
  })
})

function attention() {
  return {
    attentionId: id(1),
    sourceEvidenceId: id(11),
    revision: 1n,
    disposition: ReviewDispositionV1.REVIEW_DISPOSITION_PENDING
  }
}

function person() {
  return {
    reviewId: id(2),
    reviewRevision: 1n,
    state: PersonMatchCandidateStateV1.PERSON_MATCH_CANDIDATE_STATE_PENDING
  }
}

function task() {
  return {
    reviewId: id(3),
    reviewRevision: 1n,
    state: ReviewTaskCandidateStateV1.REVIEW_TASK_CANDIDATE_STATE_PENDING
  }
}

function note() {
  return {
    reviewId: id(4),
    reviewRevision: 1n,
    state: ReviewNoteCandidateStateV1.REVIEW_NOTE_CANDIDATE_STATE_PENDING
  }
}

function obligation() {
  return {
    reviewId: id(5),
    reviewRevision: 1n,
    state: ReviewObligationCandidateStateV1.REVIEW_OBLIGATION_CANDIDATE_STATE_PENDING
  }
}

function id(seed: number): Uint8Array {
  return new Uint8Array(16).fill(seed)
}
