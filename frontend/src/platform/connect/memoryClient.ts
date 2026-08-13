import { createClient, type Client } from '@connectrpc/connect'
import { MemoryQueryService } from '../../gen/makosh/memory/v1/memory_pb'
import { createBrowserGatewayConnectTransport } from '../gateway/browserGatewayConnect'

let client: Client<typeof MemoryQueryService> | null = null

export function getMemoryQueryClient(): Client<typeof MemoryQueryService> {
  client ??= createClient(MemoryQueryService, createBrowserGatewayConnectTransport())
  return client
}

export function resetMemoryClientForTests(): void {
  client = null
}
