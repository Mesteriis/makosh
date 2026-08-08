import { createClient, type Client } from '@connectrpc/connect'

import { CallTranscriptionQueryService } from '../../gen/makosh/call_transcription/v1/transcription_pb'
import { createBrowserGatewayConnectTransport } from '../gateway/browserGatewayConnect'

let client: Client<typeof CallTranscriptionQueryService> | null = null

export function getCallTranscriptionQueryClient(): Client<typeof CallTranscriptionQueryService> {
	client ??= createClient(CallTranscriptionQueryService, createBrowserGatewayConnectTransport())
	return client
}

export function resetCallTranscriptionQueryClientForTests(): void {
	client = null
}
