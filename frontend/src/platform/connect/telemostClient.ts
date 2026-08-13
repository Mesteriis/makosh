import { createClient, type Client } from '@connectrpc/connect'
import { YandexTelemostQueryService } from '../../gen/makosh/telemost/v1/telemost_pb'
import { createBrowserGatewayConnectTransport } from '../gateway/browserGatewayConnect'

let client: Client<typeof YandexTelemostQueryService> | null = null

export function getTelemostQueryClient(): Client<typeof YandexTelemostQueryService> {
  client ??= createClient(
    YandexTelemostQueryService,
    createBrowserGatewayConnectTransport(),
  )
  return client
}

export function resetTelemostClientForTests(): void {
  client = null
}
