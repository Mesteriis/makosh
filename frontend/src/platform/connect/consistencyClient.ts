import { createClient, type Client } from '@connectrpc/connect'
import { ConsistencyQueryService } from '../../gen/makosh/consistency/v1/consistency_pb'
import { createBrowserGatewayConnectTransport } from '../gateway/browserGatewayConnect'

let client: Client<typeof ConsistencyQueryService> | null = null

export function getConsistencyQueryClient(): Client<typeof ConsistencyQueryService> {
  client ??= createClient(ConsistencyQueryService, createBrowserGatewayConnectTransport())
  return client
}

export function resetConsistencyClientForTests(): void {
  client = null
}
