import { createClient, type Client } from '@connectrpc/connect'
import {
  ReviewTaskCandidateCommandService,
  ReviewTaskCandidateQueryService
} from '../../gen/makosh/review/task_candidate/v1/task_candidate_pb'
import { createBrowserGatewayConnectTransport } from '../gateway/browserGatewayConnect'

let commandClient: Client<typeof ReviewTaskCandidateCommandService> | null = null
let queryClient: Client<typeof ReviewTaskCandidateQueryService> | null = null

export function getReviewTaskCandidateCommandClient(): Client<typeof ReviewTaskCandidateCommandService> {
  commandClient ??= createClient(ReviewTaskCandidateCommandService, createBrowserGatewayConnectTransport())
  return commandClient
}

export function getReviewTaskCandidateQueryClient(): Client<typeof ReviewTaskCandidateQueryService> {
  queryClient ??= createClient(ReviewTaskCandidateQueryService, createBrowserGatewayConnectTransport())
  return queryClient
}

export function resetReviewTaskCandidateClientsForTests(): void {
  commandClient = null
  queryClient = null
}
