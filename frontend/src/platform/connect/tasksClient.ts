import { createClient, type Client } from '@connectrpc/connect'
import {
  TasksCommandService,
  TasksQueryService
} from '../../gen/makosh/tasks/client/v1/tasks_pb'
import { createBrowserGatewayConnectTransport } from '../gateway/browserGatewayConnect'

let commandClient: Client<typeof TasksCommandService> | null = null
let queryClient: Client<typeof TasksQueryService> | null = null

export function getTasksCommandClient(): Client<typeof TasksCommandService> {
  commandClient ??= createClient(TasksCommandService, createBrowserGatewayConnectTransport())
  return commandClient
}

export function getTasksQueryClient(): Client<typeof TasksQueryService> {
  queryClient ??= createClient(TasksQueryService, createBrowserGatewayConnectTransport())
  return queryClient
}

export function resetTasksClientsForTests(): void {
  commandClient = null
  queryClient = null
}
