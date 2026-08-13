import { createClient, type Client } from '@connectrpc/connect'
import {
  RelationshipsCommandService,
  RelationshipsQueryService
} from '../../gen/makosh/relationships/client/v1/relationships_pb'
import { createBrowserGatewayConnectTransport } from '../gateway/browserGatewayConnect'

let commandClient: Client<typeof RelationshipsCommandService> | null = null
let queryClient: Client<typeof RelationshipsQueryService> | null = null

export function getRelationshipsCommandClient(): Client<typeof RelationshipsCommandService> {
  commandClient ??= createClient(RelationshipsCommandService, createBrowserGatewayConnectTransport())
  return commandClient
}

export function getRelationshipsQueryClient(): Client<typeof RelationshipsQueryService> {
  queryClient ??= createClient(RelationshipsQueryService, createBrowserGatewayConnectTransport())
  return queryClient
}

export function resetRelationshipsClientsForTests(): void {
  commandClient = null
  queryClient = null
}
