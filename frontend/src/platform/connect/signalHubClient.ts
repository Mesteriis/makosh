import { createClient } from '@connectrpc/connect'
import { createConnectTransport } from '@connectrpc/connect-web'
import type { Client } from '@connectrpc/connect'
import { ApiClient } from '../api/ApiClient'
import { SignalHubService } from '../../gen/makosh/signal_hub/v1/signal_hub_pb'

let signalHubClient: Client<typeof SignalHubService> | null = null

function createSignalHubConnectClient(): Client<typeof SignalHubService> {
	const apiClient = ApiClient.instance
	const secret = apiClient.getSecret()
	const transport = createConnectTransport({
		baseUrl: apiClient.getBaseUrl(),
		useBinaryFormat: false,
		fetch: (input, init) => {
			const headers = new Headers(init?.headers)
			headers.set('X-Макошь-Secret', secret)
			return fetch(input, {
				...init,
				headers
			})
		}
	})

	return createClient(SignalHubService, transport)
}

export function getSignalHubConnectClient(): Client<typeof SignalHubService> {
	if (!signalHubClient) {
		signalHubClient = createSignalHubConnectClient()
	}

	return signalHubClient
}

export function resetSignalHubConnectClientForTests(): void {
	signalHubClient = null
}
