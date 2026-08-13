import { createClient, type Client } from '@connectrpc/connect'
import { TimelineQueryService } from '../../gen/makosh/timeline/v1/timeline_pb'
import { createBrowserGatewayConnectTransport } from '../gateway/browserGatewayConnect'

let client: Client<typeof TimelineQueryService> | null = null

export function getTimelineQueryClient(): Client<typeof TimelineQueryService> {
  client ??= createClient(TimelineQueryService, createBrowserGatewayConnectTransport())
  return client
}

export function resetTimelineClientForTests(): void {
  client = null
}
