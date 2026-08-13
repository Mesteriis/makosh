import { createClient, type Client } from '@connectrpc/connect'
import {
  KnowledgeCommandService,
  KnowledgeQueryService
} from '../../gen/makosh/knowledge/client/v1/knowledge_pb'
import { createBrowserGatewayConnectTransport } from '../gateway/browserGatewayConnect'

let commandClient: Client<typeof KnowledgeCommandService> | null = null
let queryClient: Client<typeof KnowledgeQueryService> | null = null

export function getKnowledgeCommandClient(): Client<typeof KnowledgeCommandService> {
  commandClient ??= createClient(KnowledgeCommandService, createBrowserGatewayConnectTransport())
  return commandClient
}

export function getKnowledgeQueryClient(): Client<typeof KnowledgeQueryService> {
  queryClient ??= createClient(KnowledgeQueryService, createBrowserGatewayConnectTransport())
  return queryClient
}

export function resetKnowledgeClientsForTests(): void {
  commandClient = null
  queryClient = null
}
