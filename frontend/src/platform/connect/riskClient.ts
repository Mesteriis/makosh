import { createClient, type Client } from '@connectrpc/connect'
import { RiskQueryService } from '../../gen/makosh/risk/v1/risk_pb'
import { createBrowserGatewayConnectTransport } from '../gateway/browserGatewayConnect'

let client: Client<typeof RiskQueryService> | null = null

export function getRiskQueryClient(): Client<typeof RiskQueryService> {
  client ??= createClient(RiskQueryService, createBrowserGatewayConnectTransport())
  return client
}

export function resetRiskClientForTests(): void {
  client = null
}
