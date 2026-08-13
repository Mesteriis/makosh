import { createClient, type Client } from '@connectrpc/connect'
import { GraphQueryService } from '../../gen/makosh/graph/v1/graph_pb'
import { createBrowserGatewayConnectTransport } from '../gateway/browserGatewayConnect'

let client: Client<typeof GraphQueryService> | null = null

export function getGraphQueryClient(): Client<typeof GraphQueryService> {
  client ??= createClient(GraphQueryService, createBrowserGatewayConnectTransport())
  return client
}

export function resetGraphClientForTests(): void {
  client = null
}
