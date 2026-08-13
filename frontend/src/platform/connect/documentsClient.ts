import { createClient, type Client } from '@connectrpc/connect'
import {
  DocumentsCommandService,
  DocumentsQueryService
} from '../../gen/makosh/documents/client/v1/documents_pb'
import { createBrowserGatewayConnectTransport } from '../gateway/browserGatewayConnect'

let commandClient: Client<typeof DocumentsCommandService> | null = null
let queryClient: Client<typeof DocumentsQueryService> | null = null

export function getDocumentsCommandClient(): Client<typeof DocumentsCommandService> {
  commandClient ??= createClient(DocumentsCommandService, createBrowserGatewayConnectTransport())
  return commandClient
}

export function getDocumentsQueryClient(): Client<typeof DocumentsQueryService> {
  queryClient ??= createClient(DocumentsQueryService, createBrowserGatewayConnectTransport())
  return queryClient
}

export function resetDocumentsClientsForTests(): void {
  commandClient = null
  queryClient = null
}
