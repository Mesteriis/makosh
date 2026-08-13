import { createClient, type Client } from '@connectrpc/connect'
import {
  ProjectsCommandService,
  ProjectsQueryService
} from '../../gen/makosh/projects/client/v1/projects_pb'
import { createBrowserGatewayConnectTransport } from '../gateway/browserGatewayConnect'

let commandClient: Client<typeof ProjectsCommandService> | null = null
let queryClient: Client<typeof ProjectsQueryService> | null = null

export function getProjectsCommandClient(): Client<typeof ProjectsCommandService> {
  commandClient ??= createClient(ProjectsCommandService, createBrowserGatewayConnectTransport())
  return commandClient
}

export function getProjectsQueryClient(): Client<typeof ProjectsQueryService> {
  queryClient ??= createClient(ProjectsQueryService, createBrowserGatewayConnectTransport())
  return queryClient
}

export function resetProjectsClientsForTests(): void {
  commandClient = null
  queryClient = null
}
