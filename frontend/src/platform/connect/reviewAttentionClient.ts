import { createClient, type Client } from '@connectrpc/connect'
import {
  ReviewAttentionCommandService,
  ReviewAttentionQueryService
} from '../../gen/makosh/review/attention/client/v1/client_pb'
import { createBrowserGatewayConnectTransport } from '../gateway/browserGatewayConnect'

let commandClient: Client<typeof ReviewAttentionCommandService> | null = null
let queryClient: Client<typeof ReviewAttentionQueryService> | null = null

export function getReviewAttentionCommandClient(): Client<typeof ReviewAttentionCommandService> {
  commandClient ??= createClient(ReviewAttentionCommandService, createBrowserGatewayConnectTransport())
  return commandClient
}

export function getReviewAttentionQueryClient(): Client<typeof ReviewAttentionQueryService> {
  queryClient ??= createClient(ReviewAttentionQueryService, createBrowserGatewayConnectTransport())
  return queryClient
}

export function resetReviewAttentionClientsForTests(): void {
  commandClient = null
  queryClient = null
}
