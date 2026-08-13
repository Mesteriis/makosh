import { createClient, type Client } from '@connectrpc/connect'
import {
  ReviewNoteCandidateCommandService,
  ReviewNoteCandidateQueryService
} from '../../gen/makosh/review/note_candidate/v1/note_candidate_pb'
import { createBrowserGatewayConnectTransport } from '../gateway/browserGatewayConnect'

let commandClient: Client<typeof ReviewNoteCandidateCommandService> | null = null
let queryClient: Client<typeof ReviewNoteCandidateQueryService> | null = null

export function getReviewNoteCandidateCommandClient(): Client<typeof ReviewNoteCandidateCommandService> {
  commandClient ??= createClient(ReviewNoteCandidateCommandService, createBrowserGatewayConnectTransport())
  return commandClient
}

export function getReviewNoteCandidateQueryClient(): Client<typeof ReviewNoteCandidateQueryService> {
  queryClient ??= createClient(ReviewNoteCandidateQueryService, createBrowserGatewayConnectTransport())
  return queryClient
}

export function resetReviewNoteCandidateClientsForTests(): void {
  commandClient = null
  queryClient = null
}
