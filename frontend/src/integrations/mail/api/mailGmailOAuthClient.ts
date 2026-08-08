import { create } from '@bufbuild/protobuf'
import { createClient, type Client } from '@connectrpc/connect'

import {
	CompleteGmailOAuthRequestV1Schema,
	GetGmailOAuthStatusRequestV1Schema,
	type GmailOAuthOperationStatusV1,
	type GmailOAuthStartedV1,
	GmailOAuthAuthorityV1,
	GmailOAuthCompleteService,
	GmailOAuthQueryService,
	GmailOAuthStartService,
	type MailAcceptedV1,
	StartGmailOAuthRequestV1Schema,
} from '../../../gen/makosh/mail/v1/client_pb'
import { createBrowserGatewayConnectTransport } from '../../../platform/gateway/browserGatewayConnect'

export class MailGmailOAuthClientV1 {
	constructor(
		private readonly startClient: Client<typeof GmailOAuthStartService> = createClient(
			GmailOAuthStartService,
			createBrowserGatewayConnectTransport(),
		),
		private readonly completeClient: Client<typeof GmailOAuthCompleteService> = createClient(
			GmailOAuthCompleteService,
			createBrowserGatewayConnectTransport(),
		),
		private readonly queryClient: Client<typeof GmailOAuthQueryService> = createClient(
			GmailOAuthQueryService,
			createBrowserGatewayConnectTransport(),
		),
	) {}

	async start(
		operationId: string,
		connectionId: string,
		authority: 'operational' | 'permanent-delete' = 'operational',
	): Promise<GmailOAuthStartedV1> {
		validateOperationId(operationId)
		validateOperationId(connectionId)
		return this.startClient.start(create(
			StartGmailOAuthRequestV1Schema,
			{
				operationId,
				connectionId,
				authority: authority === 'permanent-delete'
					? GmailOAuthAuthorityV1.GMAIL_OAUTH_AUTHORITY_PERMANENT_DELETE
					: GmailOAuthAuthorityV1.GMAIL_OAUTH_AUTHORITY_OPERATIONAL,
			},
		))
	}

	async complete(input: {
		operationId: string
		connectionId: string
		setupId: string
		state: string
		authorizationCode: string
	}): Promise<MailAcceptedV1> {
		validateOperationId(input.operationId)
		for (const value of [input.connectionId, input.setupId, input.state, input.authorizationCode]) {
			if (value.trim().length === 0) throw new Error('Gmail OAuth completion input is invalid')
		}
		return this.completeClient.complete(create(
			CompleteGmailOAuthRequestV1Schema,
			input,
		))
	}

	async status(
		operationId: string,
		connectionId: string,
	): Promise<GmailOAuthOperationStatusV1 | undefined> {
		validateOperationId(operationId)
		validateOperationId(connectionId)
		return (await this.queryClient.getOperationStatus(create(
			GetGmailOAuthStatusRequestV1Schema,
			{ operationId, connectionId },
		))).status
	}
}

function validateOperationId(value: string): void {
	if (value.trim().length === 0 || value.length > 128) {
		throw new Error('Gmail OAuth operation id is invalid')
	}
}
