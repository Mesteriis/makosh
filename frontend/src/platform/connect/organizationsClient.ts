import { createClient, type Client } from '@connectrpc/connect'
import {
  OrganizationsCommandService,
  OrganizationsQueryService
} from '../../gen/makosh/organizations/client/v1/organizations_pb'
import { createBrowserGatewayConnectTransport } from '../gateway/browserGatewayConnect'

let commandClient: Client<typeof OrganizationsCommandService> | null = null
let queryClient: Client<typeof OrganizationsQueryService> | null = null

export function getOrganizationsCommandClient(): Client<typeof OrganizationsCommandService> {
  commandClient ??= createClient(OrganizationsCommandService, createBrowserGatewayConnectTransport())
  return commandClient
}

export function getOrganizationsQueryClient(): Client<typeof OrganizationsQueryService> {
  queryClient ??= createClient(OrganizationsQueryService, createBrowserGatewayConnectTransport())
  return queryClient
}

export function resetOrganizationsClientsForTests(): void {
  commandClient = null
  queryClient = null
}
