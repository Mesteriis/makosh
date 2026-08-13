import { createClient, type Client } from '@connectrpc/connect'
import {
  ReviewPersonMatchCandidateCommandService,
  ReviewPersonMatchCandidateQueryService
} from '../../gen/makosh/review/person_match_candidate/v1/person_match_candidate_pb'
import { createBrowserGatewayConnectTransport } from '../gateway/browserGatewayConnect'

let commandClient: Client<typeof ReviewPersonMatchCandidateCommandService> | null = null
let queryClient: Client<typeof ReviewPersonMatchCandidateQueryService> | null = null

export function getReviewPersonMatchCandidateCommandClient(): Client<typeof ReviewPersonMatchCandidateCommandService> {
  commandClient ??= createClient(ReviewPersonMatchCandidateCommandService, createBrowserGatewayConnectTransport())
  return commandClient
}

export function getReviewPersonMatchCandidateQueryClient(): Client<typeof ReviewPersonMatchCandidateQueryService> {
  queryClient ??= createClient(ReviewPersonMatchCandidateQueryService, createBrowserGatewayConnectTransport())
  return queryClient
}

export function resetReviewPersonMatchCandidateClientsForTests(): void {
  commandClient = null
  queryClient = null
}
