import { computed, ref } from 'vue'
import { defineStore } from 'pinia'
import {
  ReviewDispositionV1,
  type ReviewAttentionSummaryV1
} from '../../../gen/makosh/review/attention/client/v1/client_pb'
import {
  PersonMatchCandidateDecisionV1,
  PersonMatchCandidateStateV1,
  type PersonMatchCandidateSummaryV1
} from '../../../gen/makosh/review/person_match_candidate/v1/person_match_candidate_pb'
import {
  ReviewTaskCandidateDecisionV1,
  ReviewTaskCandidateErrorCodeV1,
  ReviewTaskCandidateStateV1,
  type ReviewTaskCandidateSummaryV1
} from '../../../gen/makosh/review/task_candidate/v1/task_candidate_pb'
import {
  ReviewNoteCandidateDecisionV1,
  ReviewNoteCandidateErrorCodeV1,
  ReviewNoteCandidateStateV1,
  type ReviewNoteCandidateSummaryV1
} from '../../../gen/makosh/review/note_candidate/v1/note_candidate_pb'
import {
  ReviewObligationCandidateDecisionV1,
  ReviewObligationCandidateErrorCodeV1,
  ReviewObligationCandidateStateV1,
  type ReviewObligationCandidateSummaryV1
} from '../../../gen/makosh/review/obligation_candidate/v1/obligation_candidate_pb'
import {
  getReviewAttentionCommandClient,
  getReviewAttentionQueryClient
} from '../../../platform/connect/reviewAttentionClient'
import {
  getReviewPersonMatchCandidateCommandClient,
  getReviewPersonMatchCandidateQueryClient
} from '../../../platform/connect/reviewPersonMatchCandidateClient'
import {
  getReviewTaskCandidateCommandClient,
  getReviewTaskCandidateQueryClient
} from '../../../platform/connect/reviewTaskCandidateClient'
import {
  getReviewNoteCandidateCommandClient,
  getReviewNoteCandidateQueryClient
} from '../../../platform/connect/reviewNoteCandidateClient'
import {
  getReviewObligationCandidateCommandClient,
  getReviewObligationCandidateQueryClient
} from '../../../platform/connect/reviewObligationCandidateClient'
import type { PersonMatchApprovalV1 } from '../types/review'

const PAGE_LIMIT = 50

export const useReviewStore = defineStore('review', () => {
  const attention = ref<ReviewAttentionSummaryV1[]>([])
  const personMatchCandidates = ref<PersonMatchCandidateSummaryV1[]>([])
  const taskCandidates = ref<ReviewTaskCandidateSummaryV1[]>([])
  const noteCandidates = ref<ReviewNoteCandidateSummaryV1[]>([])
  const obligationCandidates = ref<ReviewObligationCandidateSummaryV1[]>([])
  const error = ref('')
  const reviewingItemKey = ref<string | null>(null)

  const totalPendingCount = computed(() =>
    attention.value.filter((item) => item.disposition === ReviewDispositionV1.REVIEW_DISPOSITION_PENDING).length
    + personMatchCandidates.value.filter((item) => item.state === PersonMatchCandidateStateV1.PERSON_MATCH_CANDIDATE_STATE_PENDING).length
    + taskCandidates.value.filter((item) => item.state === ReviewTaskCandidateStateV1.REVIEW_TASK_CANDIDATE_STATE_PENDING).length
    + noteCandidates.value.filter((item) => item.state === ReviewNoteCandidateStateV1.REVIEW_NOTE_CANDIDATE_STATE_PENDING).length
    + obligationCandidates.value.filter((item) => item.state === ReviewObligationCandidateStateV1.REVIEW_OBLIGATION_CANDIDATE_STATE_PENDING).length
  )

  async function loadAll(): Promise<void> {
    error.value = ''
    try {
      const [attentionResult, personResult, taskResult, noteResult, obligationResult] = await Promise.all([
        getReviewAttentionQueryClient().query({
          protocolMajor: 1,
          operation: { case: 'list', value: { limit: PAGE_LIMIT, cursor: new Uint8Array() } }
        }),
        getReviewPersonMatchCandidateQueryClient().list({
          logicalOwnerId: '',
          limit: PAGE_LIMIT
        }),
        getReviewTaskCandidateQueryClient().list({
          protocolMajor: 1,
          state: ReviewTaskCandidateStateV1.REVIEW_TASK_CANDIDATE_STATE_PENDING,
          afterReviewId: new Uint8Array(),
          limit: PAGE_LIMIT
        }),
        getReviewNoteCandidateQueryClient().list({
          protocolMajor: 1,
          state: ReviewNoteCandidateStateV1.REVIEW_NOTE_CANDIDATE_STATE_PENDING,
          afterReviewId: new Uint8Array(),
          limit: PAGE_LIMIT
        }),
        getReviewObligationCandidateQueryClient().list({
          protocolMajor: 1,
          state: ReviewObligationCandidateStateV1.REVIEW_OBLIGATION_CANDIDATE_STATE_PENDING,
          afterReviewId: new Uint8Array(),
          limit: PAGE_LIMIT
        })
      ])
      if (attentionResult.errorCode || attentionResult.result.case !== 'page') {
        throw new Error(attentionResult.errorCode || 'review_attention_invalid_response')
      }
      assertTaskResult(taskResult.error)
      assertNoteResult(noteResult.error)
      assertObligationResult(obligationResult.error)
      attention.value = attentionResult.result.value.attention
      personMatchCandidates.value = personResult.candidates
      taskCandidates.value = taskResult.reviews
      noteCandidates.value = noteResult.reviews
      obligationCandidates.value = obligationResult.reviews
    } catch (cause) {
      error.value = message(cause)
    }
  }

  async function resolveAttention(
    item: ReviewAttentionSummaryV1,
    resolution: 'reviewed' | 'dismissed'
  ): Promise<void> {
    await run(`attention:${hex(item.attentionId)}`, async () => {
      const response = await getReviewAttentionCommandClient().execute({
        protocolMajor: 1,
        operationId: randomId16(),
        sourceEvidenceId: item.sourceEvidenceId,
        expectedRevision: item.revision,
        operation: resolution === 'reviewed'
          ? { case: 'markReviewed', value: {} }
          : { case: 'dismiss', value: {} }
      })
      if (response.errorCode || !response.attention) {
        throw new Error(response.errorCode || 'review_attention_invalid_response')
      }
      replace(attention.value, response.attention, (value) => value.attentionId)
    })
  }

  async function decideTaskCandidate(
    item: ReviewTaskCandidateSummaryV1,
    decision: ReviewTaskCandidateDecisionV1
  ): Promise<void> {
    await run(`task:${hex(item.reviewId)}`, async () => {
      const response = await getReviewTaskCandidateCommandClient().decide({
        protocolMajor: 1,
        operationId: randomId16(),
        reviewId: item.reviewId,
        expectedReviewRevision: item.reviewRevision,
        decision
      })
      assertTaskResult(response.error)
      if (!response.review) throw new Error('review_task_invalid_response')
      replace(taskCandidates.value, response.review, (value) => value.reviewId)
    })
  }

  async function decideNoteCandidate(
    item: ReviewNoteCandidateSummaryV1,
    decision: ReviewNoteCandidateDecisionV1
  ): Promise<void> {
    await run(`note:${hex(item.reviewId)}`, async () => {
      const response = await getReviewNoteCandidateCommandClient().decide({
        protocolMajor: 1,
        operationId: randomId16(),
        reviewId: item.reviewId,
        expectedReviewRevision: item.reviewRevision,
        decision
      })
      assertNoteResult(response.error)
      if (!response.review) throw new Error('review_note_invalid_response')
      replace(noteCandidates.value, response.review, (value) => value.reviewId)
    })
  }

  async function decideObligationCandidate(
    item: ReviewObligationCandidateSummaryV1,
    decision: ReviewObligationCandidateDecisionV1
  ): Promise<void> {
    await run(`obligation:${hex(item.reviewId)}`, async () => {
      const response = await getReviewObligationCandidateCommandClient().decide({
        protocolMajor: 1,
        operationId: randomId16(),
        reviewId: item.reviewId,
        expectedReviewRevision: item.reviewRevision,
        decision
      })
      assertObligationResult(response.error)
      if (!response.review) throw new Error('review_obligation_invalid_response')
      replace(obligationCandidates.value, response.review, (value) => value.reviewId)
    })
  }

  async function decidePersonMatchCandidate(
    item: PersonMatchCandidateSummaryV1,
    decision: PersonMatchCandidateDecisionV1,
    approval?: PersonMatchApprovalV1
  ): Promise<void> {
    await run(`person-match:${hex(item.reviewId)}`, async () => {
      if (decision === PersonMatchCandidateDecisionV1.PERSON_MATCH_CANDIDATE_DECISION_APPROVE && !approval) {
        throw new Error('review_person_match_approval_required')
      }
      const response = await getReviewPersonMatchCandidateCommandClient().decide({
        protocolMajor: 1,
        operationId: randomId16(),
        reviewId: item.reviewId,
        expectedReviewRevision: item.reviewRevision,
        decision,
        approvedAction: approval?.approvedAction,
        approvedActionDigest: approval?.approvedActionDigest ?? new Uint8Array()
      })
      replace(personMatchCandidates.value, response, (value) => value.reviewId)
    })
  }

  async function run(key: string, operation: () => Promise<void>): Promise<void> {
    reviewingItemKey.value = key
    error.value = ''
    try {
      await operation()
    } catch (cause) {
      error.value = message(cause)
      throw cause
    } finally {
      reviewingItemKey.value = null
    }
  }

  return {
    attention,
    personMatchCandidates,
    taskCandidates,
    noteCandidates,
    obligationCandidates,
    error,
    reviewingItemKey,
    totalPendingCount,
    loadAll,
    resolveAttention,
    decidePersonMatchCandidate,
    decideTaskCandidate,
    decideNoteCandidate,
    decideObligationCandidate
  }
})

function assertTaskResult(error: ReviewTaskCandidateErrorCodeV1): void {
  if (error !== ReviewTaskCandidateErrorCodeV1.REVIEW_TASK_CANDIDATE_ERROR_CODE_UNSPECIFIED) {
    throw new Error(`review_task_error_${error}`)
  }
}

function assertNoteResult(error: ReviewNoteCandidateErrorCodeV1): void {
  if (error !== ReviewNoteCandidateErrorCodeV1.REVIEW_NOTE_CANDIDATE_ERROR_CODE_UNSPECIFIED) {
    throw new Error(`review_note_error_${error}`)
  }
}

function assertObligationResult(error: ReviewObligationCandidateErrorCodeV1): void {
  if (error !== ReviewObligationCandidateErrorCodeV1.REVIEW_OBLIGATION_CANDIDATE_ERROR_CODE_UNSPECIFIED) {
    throw new Error(`review_obligation_error_${error}`)
  }
}

function replace<T>(items: T[], updated: T, id: (value: T) => Uint8Array): void {
  const index = items.findIndex((item) => sameBytes(id(item), id(updated)))
  if (index === -1) items.push(updated)
  else items[index] = updated
}

function sameBytes(left: Uint8Array, right: Uint8Array): boolean {
  return left.length === right.length && left.every((value, index) => value === right[index])
}

function randomId16(): Uint8Array {
  return globalThis.crypto.getRandomValues(new Uint8Array(16))
}

function hex(value: Uint8Array): string {
  return Array.from(value, (byte) => byte.toString(16).padStart(2, '0')).join('')
}

function message(cause: unknown): string {
  return cause instanceof Error ? cause.message : 'review_unavailable'
}
