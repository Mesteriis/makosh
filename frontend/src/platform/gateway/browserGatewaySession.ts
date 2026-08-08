import { create } from '@bufbuild/protobuf'
import { createClient, type Client } from '@connectrpc/connect'

import {
	BrowserGatewayAccessModeV1,
	BrowserSessionService,
	BrowserSessionStatusRequestV1Schema,
} from '../../gen/makosh/gateway/v1/browser_session_pb'
import { createBrowserGatewayConnectTransport } from './browserGatewayConnect'

export type BrowserGatewaySessionStatus = {
	accessMode:
		| BrowserGatewayAccessModeV1.PAIRED
		| BrowserGatewayAccessModeV1.LAN_DEVELOPMENT
		| BrowserGatewayAccessModeV1.LOCAL_DEVELOPMENT
	expiresAtUnixMillis: bigint
}

export async function fetchBrowserGatewaySessionStatus(
	client: Client<typeof BrowserSessionService> = createClient(
		BrowserSessionService,
		createBrowserGatewayConnectTransport(),
	),
): Promise<BrowserGatewaySessionStatus> {
	const response = await client.getStatus(create(BrowserSessionStatusRequestV1Schema))
	if (response.major !== 1
		|| (response.accessMode !== BrowserGatewayAccessModeV1.PAIRED
			&& response.accessMode !== BrowserGatewayAccessModeV1.LAN_DEVELOPMENT
			&& response.accessMode !== BrowserGatewayAccessModeV1.LOCAL_DEVELOPMENT)) {
		throw new Error('Unsupported browser Gateway session contract')
	}
	return {
		accessMode: response.accessMode,
		expiresAtUnixMillis: response.sessionExpiresAtUnixMillis,
	}
}
