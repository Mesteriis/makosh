import { createClient, type Client } from '@connectrpc/connect'

import { CallTranscriptionCommandService } from '../../gen/makosh/call_transcription/v1/transcription_pb'
import { createBrowserGatewayConnectTransport } from '../gateway/browserGatewayConnect'

let client: Client<typeof CallTranscriptionCommandService> | null = null

export function getCallTranscriptionCommandClient(): Client<typeof CallTranscriptionCommandService> {
	client ??= createClient(CallTranscriptionCommandService, createBrowserGatewayConnectTransport())
	return client
}

export function resetCallTranscriptionCommandClientForTests(): void {
	client = null
}
