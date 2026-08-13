import { createClient, type Client } from '@connectrpc/connect'
import {
  ReviewObligationCandidateCommandService,
  ReviewObligationCandidateQueryService
} from '../../gen/makosh/review/obligation_candidate/v1/obligation_candidate_pb'
import { createBrowserGatewayConnectTransport } from '../gateway/browserGatewayConnect'

let commandClient: Client<typeof ReviewObligationCandidateCommandService> | null = null
let queryClient: Client<typeof ReviewObligationCandidateQueryService> | null = null

export function getReviewObligationCandidateCommandClient(): Client<typeof ReviewObligationCandidateCommandService> {
  commandClient ??= createClient(ReviewObligationCandidateCommandService, createBrowserGatewayConnectTransport())
  return commandClient
}

export function getReviewObligationCandidateQueryClient(): Client<typeof ReviewObligationCandidateQueryService> {
  queryClient ??= createClient(ReviewObligationCandidateQueryService, createBrowserGatewayConnectTransport())
  return queryClient
}

export function resetReviewObligationCandidateClientsForTests(): void {
  commandClient = null
  queryClient = null
}
