import { createClient, type Client } from '@connectrpc/connect'

import { CallTranscriptTicketService } from '../../gen/makosh/call_transcription/v1/transcription_pb'
import { createBrowserGatewayConnectTransport } from '../gateway/browserGatewayConnect'

let client: Client<typeof CallTranscriptTicketService> | null = null

export function getCallTranscriptTicketClient(): Client<typeof CallTranscriptTicketService> {
	client ??= createClient(CallTranscriptTicketService, createBrowserGatewayConnectTransport())
	return client
}

export function resetCallTranscriptTicketClientForTests(): void {
	client = null
}
