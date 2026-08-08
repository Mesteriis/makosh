import { create } from '@bufbuild/protobuf'
import { createClient, type Client } from '@connectrpc/connect'

import {
	GetAccountStatusQuerySchema,
	type ZulipAccountStatusV1,
	ZulipOperationalQueryService,
	ZulipOperationalQueryV1Schema,
} from '../../../gen/makosh/zulip/operational/v1/client_pb'
import { createBrowserGatewayConnectTransport } from '../../../platform/gateway/browserGatewayConnect'

let client: Client<typeof ZulipOperationalQueryService> | null = null

export async function getZulipAccountStatus(accountId: string): Promise<ZulipAccountStatusV1> {
	if (accountId.trim().length === 0) throw new Error('zulip account id is invalid')
	const response = await getZulipAccountStatusConnectClient().query(create(
		ZulipOperationalQueryV1Schema,
		{
			query: {
				case: 'getAccountStatus',
				value: create(GetAccountStatusQuerySchema, { accountId }),
			},
		},
	))
	if (response.response.case !== 'accountStatus') {
		throw new Error('zulip account status is unavailable')
	}
	return response.response.value
}

export function getZulipAccountStatusConnectClient(): Client<
	typeof ZulipOperationalQueryService
> {
	client ??= createClient(
		ZulipOperationalQueryService,
		createBrowserGatewayConnectTransport(),
	)
	return client
}

export function resetZulipAccountStatusConnectClientForTests(): void {
	client = null
}
