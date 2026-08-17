import type { ClientModuleBootstrapV1 } from '../../../gen/makosh/gateway/v1/client_bootstrap_pb'
import {
	MailAccountReadinessV1,
	MailProviderPathReadinessV1,
	type MailAccountStatusV1,
} from '../../../gen/makosh/mail/account/v1/client_pb'

const MAIL_MODULE_ID = 'makosh-mail-runtime'
const MAIL_ACCOUNT_CATALOG_CAPABILITY = 'mail.account.catalog.query.v1'
const MAX_CONNECTION_ID_BYTES = 512
const textEncoder = new TextEncoder()

export type MailAccountConnection = {
	connectionId: string
	deliveryReady: boolean
	registrationId: string
	syncReady: boolean
}

export function mailAccountConnections(
	modules: readonly ClientModuleBootstrapV1[],
	accounts: readonly MailAccountStatusV1[],
): readonly MailAccountConnection[] {
	const registration = modules.find((module) =>
		module.moduleId === MAIL_MODULE_ID
		&& module.sectionsEnabled
		&& module.capabilityIds.includes(MAIL_ACCOUNT_CATALOG_CAPABILITY)
	)
	if (!registration) return []

	const connections = new Map<string, MailAccountConnection>()
	for (const account of accounts) {
		const connectionId = account.connectionId.trim()
		if (
			!activeAccount(account.readiness)
			|| !validConnectionId(connectionId)
			|| connections.has(connectionId)
		) continue
		connections.set(connectionId, {
			connectionId,
			deliveryReady: account.deliveryReadiness
				=== MailProviderPathReadinessV1.MAIL_PROVIDER_PATH_READINESS_READY,
			registrationId: registration.registrationId,
			syncReady: account.syncReadiness
				=== MailProviderPathReadinessV1.MAIL_PROVIDER_PATH_READINESS_READY,
		})
	}
	return [...connections.values()].sort(
		(left, right) => left.connectionId.localeCompare(right.connectionId),
	)
}

export function mailAccountConnectionFingerprint(
	connections: readonly MailAccountConnection[],
): string {
	return connections
		.map((connection) => `${connection.registrationId}:${connection.connectionId}`)
		.join('|')
}

export function mailConnectionCredentialRequired(
	connections: readonly MailAccountConnection[],
	connectionId: string,
): boolean {
	const connection = connections.find(candidate => candidate.connectionId === connectionId)
	return connection !== undefined && !connection.syncReady
}

function activeAccount(readiness: MailAccountReadinessV1): boolean {
	return readiness === MailAccountReadinessV1.MAIL_ACCOUNT_READINESS_CONFIGURATION_ONLY
		|| readiness === MailAccountReadinessV1.MAIL_ACCOUNT_READINESS_PENDING_RESTART
		|| readiness === MailAccountReadinessV1.MAIL_ACCOUNT_READINESS_READY
		|| readiness === MailAccountReadinessV1.MAIL_ACCOUNT_READINESS_DEGRADED
}

function validConnectionId(value: string): boolean {
	if (!value || textEncoder.encode(value).length > MAX_CONNECTION_ID_BYTES) return false
	for (let index = 0; index < value.length; index += 1) {
		const code = value.charCodeAt(index)
		if (code <= 0x1f || code === 0x7f) return false
	}
	return true
}
