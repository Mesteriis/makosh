import { create } from '@bufbuild/protobuf'
import { createClient, type Client } from '@connectrpc/connect'

import {
	ZulipAccountLifecycleCommandV1Schema,
	type ZulipAccountLifecycleReceiptV1,
	ZulipAccountLifecycleService,
	ZulipBindCredentialV1Schema,
	ZulipRetireAccountV1Schema,
} from '../../../gen/makosh/zulip/account/v1/client_pb'
import { createBrowserGatewayConnectTransport } from '../../../platform/gateway/browserGatewayConnect'

let lifecycleClient: Client<typeof ZulipAccountLifecycleService> | null = null

export async function bindZulipCredential(input: {
	accountId: string
	expectedBindingRevision: bigint
	credentialRevision: bigint
}): Promise<ZulipAccountLifecycleReceiptV1> {
	return getZulipAccountLifecycleConnectClient().apply(create(
		ZulipAccountLifecycleCommandV1Schema,
		{
			command: {
				case: 'bindCredential',
				value: create(ZulipBindCredentialV1Schema, input),
			},
		},
	))
}

export async function retireZulipAccount(input: {
	accountId: string
	expectedBindingRevision: bigint
}): Promise<ZulipAccountLifecycleReceiptV1> {
	if (input.accountId.trim().length === 0 || input.expectedBindingRevision <= 0n) {
		throw new Error('zulip retirement input is invalid')
	}
	return getZulipAccountLifecycleConnectClient().apply(create(
		ZulipAccountLifecycleCommandV1Schema,
		{
			command: {
				case: 'retireAccount',
				value: create(ZulipRetireAccountV1Schema, input),
			},
		},
	))
}

export function getZulipAccountLifecycleConnectClient(): Client<
	typeof ZulipAccountLifecycleService
> {
	if (!lifecycleClient) {
		lifecycleClient = createClient(
			ZulipAccountLifecycleService,
			createBrowserGatewayConnectTransport(),
		)
	}
	return lifecycleClient
}

export function resetZulipAccountLifecycleConnectClientForTests(): void {
	lifecycleClient = null
}
