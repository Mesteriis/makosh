import { create } from '@bufbuild/protobuf'
import { createClient, type Client } from '@connectrpc/connect'

import {
	MailAccountCredentialBindingService,
	MailBindCredentialRequestV1Schema,
	type MailCredentialBindingReceiptV1,
	MailCredentialPurposeV1,
} from '../../../gen/makosh/mail/account/v1/client_pb'
import { createBrowserGatewayConnectTransport } from '../../../platform/gateway/browserGatewayConnect'

let client: Client<typeof MailAccountCredentialBindingService> | null = null

export type BindMailCredentialInputV1 = {
	connectionId: string
	purpose:
		| MailCredentialPurposeV1.MAIL_CREDENTIAL_PURPOSE_IMAP_PASSWORD
		| MailCredentialPurposeV1.MAIL_CREDENTIAL_PURPOSE_SMTP_PASSWORD
	expectedBindingRevision: bigint
	credentialRevision: bigint
}

export async function bindMailCredential(
	input: BindMailCredentialInputV1,
): Promise<MailCredentialBindingReceiptV1> {
	if (input.connectionId.trim().length === 0
		|| input.expectedBindingRevision < 0n
		|| input.credentialRevision <= 0n) {
		throw new Error('mail credential binding input is invalid')
	}
	return getMailCredentialBindingConnectClient().bind(create(
		MailBindCredentialRequestV1Schema,
		input,
	))
}

export function getMailCredentialBindingConnectClient(): Client<
	typeof MailAccountCredentialBindingService
> {
	client ??= createClient(
		MailAccountCredentialBindingService,
		createBrowserGatewayConnectTransport(),
	)
	return client
}

export function resetMailCredentialBindingConnectClientForTests(): void {
	client = null
}
