import { createClient, type Client } from '@connectrpc/connect'
import {
  DecisionsCommandService,
  DecisionsQueryService
} from '../../gen/makosh/decisions/client/v1/decisions_pb'
import { createBrowserGatewayConnectTransport } from '../gateway/browserGatewayConnect'

let commandClient: Client<typeof DecisionsCommandService> | null = null
let queryClient: Client<typeof DecisionsQueryService> | null = null

export function getDecisionsCommandClient(): Client<typeof DecisionsCommandService> {
  commandClient ??= createClient(DecisionsCommandService, createBrowserGatewayConnectTransport())
  return commandClient
}

export function getDecisionsQueryClient(): Client<typeof DecisionsQueryService> {
  queryClient ??= createClient(DecisionsQueryService, createBrowserGatewayConnectTransport())
  return queryClient
}

export function resetDecisionsClientsForTests(): void {
  commandClient = null
  queryClient = null
}
