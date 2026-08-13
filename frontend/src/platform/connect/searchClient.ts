import { createClient, type Client } from '@connectrpc/connect'
import { SearchQueryService } from '../../gen/makosh/search/v1/search_pb'
import { createBrowserGatewayConnectTransport } from '../gateway/browserGatewayConnect'

let client: Client<typeof SearchQueryService> | null = null

export function getSearchQueryClient(): Client<typeof SearchQueryService> {
  client ??= createClient(SearchQueryService, createBrowserGatewayConnectTransport())
  return client
}

export function resetSearchClientForTests(): void {
  client = null
}
