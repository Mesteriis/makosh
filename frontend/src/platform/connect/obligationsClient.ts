import { createClient, type Client } from '@connectrpc/connect'
import {
  ObligationsCommandService,
  ObligationsQueryService
} from '../../gen/makosh/obligations/client/v1/obligations_pb'
import { createBrowserGatewayConnectTransport } from '../gateway/browserGatewayConnect'

let commandClient: Client<typeof ObligationsCommandService> | null = null
let queryClient: Client<typeof ObligationsQueryService> | null = null

export function getObligationsCommandClient(): Client<typeof ObligationsCommandService> {
  commandClient ??= createClient(ObligationsCommandService, createBrowserGatewayConnectTransport())
  return commandClient
}

export function getObligationsQueryClient(): Client<typeof ObligationsQueryService> {
  queryClient ??= createClient(ObligationsQueryService, createBrowserGatewayConnectTransport())
  return queryClient
}

export function resetObligationsClientsForTests(): void {
  commandClient = null
  queryClient = null
}
