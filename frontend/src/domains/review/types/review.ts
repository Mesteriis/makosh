import type { ReviewAttentionSummaryV1 } from '../../../gen/makosh/review/attention/client/v1/client_pb'
import type {
  PersonMatchCandidateApprovedActionV1,
  PersonMatchCandidateSummaryV1
} from '../../../gen/makosh/review/person_match_candidate/v1/person_match_candidate_pb'
import type { ReviewTaskCandidateSummaryV1 } from '../../../gen/makosh/review/task_candidate/v1/task_candidate_pb'
import type { ReviewNoteCandidateSummaryV1 } from '../../../gen/makosh/review/note_candidate/v1/note_candidate_pb'
import type { ReviewObligationCandidateSummaryV1 } from '../../../gen/makosh/review/obligation_candidate/v1/obligation_candidate_pb'

export type ReviewQueueSnapshot = {
  attention: ReviewAttentionSummaryV1[]
  personMatchCandidates: PersonMatchCandidateSummaryV1[]
  taskCandidates: ReviewTaskCandidateSummaryV1[]
  noteCandidates: ReviewNoteCandidateSummaryV1[]
  obligationCandidates: ReviewObligationCandidateSummaryV1[]
}

export type PersonMatchApprovalV1 = {
  approvedAction: PersonMatchCandidateApprovedActionV1
  approvedActionDigest: Uint8Array
}
