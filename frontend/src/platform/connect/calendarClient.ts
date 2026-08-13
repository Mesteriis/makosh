import { createClient, type Client } from '@connectrpc/connect'
import {
  CalendarCommandService,
  CalendarQueryService
} from '../../gen/makosh/calendar/client/v1/calendar_pb'
import { createBrowserGatewayConnectTransport } from '../gateway/browserGatewayConnect'

let commandClient: Client<typeof CalendarCommandService> | null = null
let queryClient: Client<typeof CalendarQueryService> | null = null

export function getCalendarCommandClient(): Client<typeof CalendarCommandService> {
  commandClient ??= createClient(CalendarCommandService, createBrowserGatewayConnectTransport())
  return commandClient
}

export function getCalendarQueryClient(): Client<typeof CalendarQueryService> {
  queryClient ??= createClient(CalendarQueryService, createBrowserGatewayConnectTransport())
  return queryClient
}

export function resetCalendarClientsForTests(): void {
  commandClient = null
  queryClient = null
}
