import { create } from '@bufbuild/protobuf'
import { createClient, type Client } from '@connectrpc/connect'

import {
	MailAccountDeleteService,
	MailAccountLifecycleCommandV1Schema,
	type MailAccountLifecycleReceiptV1,
	MailAccountLifecycleRetryService,
	MailAccountLifecycleRetryV1Schema,
	MailAccountLifecycleStatusRequestV1Schema,
	MailAccountLifecycleStatusService,
	MailAccountRetireService,
} from '../../../gen/makosh/mail/account_lifecycle/v1/client_pb'
import { createBrowserGatewayConnectTransport } from '../../../platform/gateway/browserGatewayConnect'

type LifecycleCommandInputV1 = {
	operationId?: string
	connectionId: string
	expectedLifecycleRevision: bigint
}

let retireClient: Client<typeof MailAccountRetireService> | null = null
let deleteClient: Client<typeof MailAccountDeleteService> | null = null
let retryClient: Client<typeof MailAccountLifecycleRetryService> | null = null
let statusClient: Client<typeof MailAccountLifecycleStatusService> | null = null

export async function retireMailAccount(
	input: LifecycleCommandInputV1,
): Promise<MailAccountLifecycleReceiptV1> {
	return getRetireClient().retire(lifecycleCommand(input))
}

export async function deleteMailAccount(
	input: LifecycleCommandInputV1,
): Promise<MailAccountLifecycleReceiptV1> {
	return getDeleteClient().delete(lifecycleCommand(input))
}

export async function retryMailAccountLifecycle(input: {
	operationId: string
	connectionId: string
	expectedLifecycleRevision: bigint
}): Promise<MailAccountLifecycleReceiptV1> {
	validateIdentifier(input.operationId, 'mail lifecycle operation ID')
	validateIdentifier(input.connectionId, 'mail connection ID')
	return getRetryClient().retry(create(MailAccountLifecycleRetryV1Schema, input))
}

export async function getMailAccountLifecycleStatus(input: {
	operationId: string
	connectionId: string
}): Promise<MailAccountLifecycleReceiptV1> {
	validateIdentifier(input.operationId, 'mail lifecycle operation ID')
	validateIdentifier(input.connectionId, 'mail connection ID')
	return getStatusClient().get(create(MailAccountLifecycleStatusRequestV1Schema, input))
}

function lifecycleCommand(input: LifecycleCommandInputV1) {
	validateIdentifier(input.connectionId, 'mail connection ID')
	if (input.expectedLifecycleRevision < 0n) {
		throw new RangeError('mail lifecycle revision is invalid')
	}
	return create(MailAccountLifecycleCommandV1Schema, {
		operationId: input.operationId ?? crypto.randomUUID(),
		connectionId: input.connectionId,
		expectedLifecycleRevision: input.expectedLifecycleRevision,
	})
}

function getRetireClient(): Client<typeof MailAccountRetireService> {
	retireClient ??= createClient(MailAccountRetireService, createBrowserGatewayConnectTransport())
	return retireClient
}

function getDeleteClient(): Client<typeof MailAccountDeleteService> {
	deleteClient ??= createClient(MailAccountDeleteService, createBrowserGatewayConnectTransport())
	return deleteClient
}

function getRetryClient(): Client<typeof MailAccountLifecycleRetryService> {
	retryClient ??= createClient(
		MailAccountLifecycleRetryService,
		createBrowserGatewayConnectTransport(),
	)
	return retryClient
}

function getStatusClient(): Client<typeof MailAccountLifecycleStatusService> {
	statusClient ??= createClient(
		MailAccountLifecycleStatusService,
		createBrowserGatewayConnectTransport(),
	)
	return statusClient
}

function validateIdentifier(value: string, label: string): void {
	if (value.trim().length === 0) throw new RangeError(`${label} is invalid`)
}

export function resetMailAccountLifecycleClientsForTests(): void {
	retireClient = null
	deleteClient = null
	retryClient = null
	statusClient = null
}
