import type {
	CredentialBinding,
	TelegramAccountResponse,
	TelegramOperationResponse,
	TelegramReconfigurationResponseV1,
} from '../../../gen/makosh/telegram/v1/client_pb'
import { getTelegramLifecycleConnectClient } from './telegramLifecycleClient'
import { getTelegramReconfigurationConnectClient } from './telegramReconfigurationClient'

const REPLAY_LIMIT = 100

export type ProvisionTelegramAccountInput = {
	accountId: string
	displayName: string
	externalAccountId: string
	credentials: readonly CredentialBinding[]
}

export async function listTelegramAccounts(): Promise<readonly TelegramAccountResponse[]> {
	const response = await getTelegramLifecycleConnectClient().execute({
		request: { case: 'listAccounts', value: {} },
	})
	if (response.response.case !== 'accounts') {
		throw new Error('Telegram account projection is unavailable')
	}
	return response.response.value.account
}

export async function provisionTelegramAccount(
	input: ProvisionTelegramAccountInput,
): Promise<TelegramAccountResponse> {
	const response = await getTelegramLifecycleConnectClient().execute({
		request: {
			case: 'provision',
			value: {
				accountId: requireIdentifier('account ID', input.accountId),
				displayName: requireIdentifier('display name', input.displayName),
				externalAccountId: input.externalAccountId.trim(),
				credential: [...input.credentials],
				qrAuthorized: false,
			},
		},
	})
	if (response.response.case !== 'account') {
		throw new Error('Telegram account provisioning result is unavailable')
	}
	return response.response.value
}

export async function restartTelegramAccount(
	accountId: string,
	expectedRuntimeEpoch: bigint,
	reconfigurationId: string = crypto.randomUUID(),
): Promise<TelegramReconfigurationResponseV1> {
	if (expectedRuntimeEpoch <= 0n) throw new RangeError('runtime epoch is required')
	return getTelegramReconfigurationConnectClient().execute({
		request: {
			case: 'begin',
			value: {
				reconfigurationId: requireIdentifier('reconfiguration ID', reconfigurationId),
				accountId: requireIdentifier('account ID', accountId),
				expectedRuntimeEpoch,
			},
		},
	})
}

export async function retireTelegramAccount(accountId: string): Promise<string> {
	return executeAccountAction('retireAccount', accountId)
}

export async function replayTelegramAccount(
	accountId: string,
	afterSequence: bigint,
): Promise<TelegramOperationResponse> {
	const response = await getTelegramLifecycleConnectClient().execute({
		request: {
			case: 'replay',
			value: {
				accountId: requireIdentifier('account ID', accountId),
				afterSequence,
				limit: REPLAY_LIMIT,
			},
		},
	})
	if (response.response.case !== 'operation') {
		throw new Error('Telegram replay result is unavailable')
	}
	return response.response.value
}

export async function retryTelegramOperation(
	operationId: string,
	nowUnixSeconds: bigint,
): Promise<TelegramOperationResponse> {
	const response = await getTelegramLifecycleConnectClient().execute({
		request: {
			case: 'retry',
			value: {
				operationId: requireIdentifier('operation ID', operationId),
				nowUnixSeconds,
				nextAttemptAtUnixSeconds: nowUnixSeconds,
			},
		},
	})
	if (response.response.case !== 'operation') {
		throw new Error('Telegram retry result is unavailable')
	}
	return response.response.value
}

async function executeAccountAction(
	action: 'retireAccount',
	accountId: string,
): Promise<string> {
	const response = await getTelegramLifecycleConnectClient().execute({
		request: {
			case: action,
			value: { accountId: requireIdentifier('account ID', accountId) },
		},
	})
	if (response.response.case !== 'accepted') {
		throw new Error(`Telegram ${action} was not accepted`)
	}
	return response.response.value.operationId
}

function requireIdentifier(label: string, value: string): string {
	const normalized = value.trim()
	if (!normalized) {
		throw new RangeError(`${label} is required`)
	}
	return normalized
}
