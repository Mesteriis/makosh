import { createClient, type Client } from '@connectrpc/connect'
import {
  PersonsCommandService,
  PersonsQueryService
} from '../../gen/makosh/persons/v1/persons_pb'
import { createBrowserGatewayConnectTransport } from '../gateway/browserGatewayConnect'

let personsCommandClient: Client<typeof PersonsCommandService> | null = null
let personsQueryClient: Client<typeof PersonsQueryService> | null = null

export function getPersonsCommandClient(): Client<typeof PersonsCommandService> {
  personsCommandClient ??= createClient(PersonsCommandService, createBrowserGatewayConnectTransport())
  return personsCommandClient
}

export function getPersonsQueryClient(): Client<typeof PersonsQueryService> {
  personsQueryClient ??= createClient(PersonsQueryService, createBrowserGatewayConnectTransport())
  return personsQueryClient
}

export function resetPersonsClientsForTests(): void {
  personsCommandClient = null
  personsQueryClient = null
}
