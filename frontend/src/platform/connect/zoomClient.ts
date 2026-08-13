import { createClient, type Client } from '@connectrpc/connect'
import { ZoomQueryService } from '../../gen/makosh/zoom/v1/zoom_pb'
import { createBrowserGatewayConnectTransport } from '../gateway/browserGatewayConnect'

let client: Client<typeof ZoomQueryService> | null = null

export function getZoomQueryClient(): Client<typeof ZoomQueryService> {
  client ??= createClient(ZoomQueryService, createBrowserGatewayConnectTransport())
  return client
}

export function resetZoomClientForTests(): void {
  client = null
}
