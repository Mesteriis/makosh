import type { TelegramAccountResponse } from '../../../gen/makosh/telegram/v1/client_pb'
import type { TelegramAuthorizationStatus } from '../api/telegramAuthorizationGateway'

export type TelegramAccountRow = {
	id: string
	title: string
	detail: string
	selected: boolean
}

export type TelegramAccountAccessModel = {
	accounts: readonly TelegramAccountRow[]
	selectedAccountId: string
	authorizationState: string
	authorizationQrDataUrl: string
	authorizationPasswordHint: string
	password: string
	provisionAccountId: string
	provisionDisplayName: string
	provisionExternalAccountId: string
	statusMessage: string
	pending: boolean
	canAuthorize: boolean
	canManageLifecycle: boolean
	canReconfigure: boolean
}

export function buildTelegramAccountRows(
	accounts: readonly TelegramAccountResponse[],
	selectedAccountId: string,
): readonly TelegramAccountRow[] {
	return accounts.map((account) => ({
		id: account.accountId,
		title: account.displayName || account.externalAccountId || account.accountId,
		detail: [account.state, account.runtimeState]
			.filter(Boolean)
			.join(' · ') || 'provisioned',
		selected: account.accountId === selectedAccountId,
	}))
}

export function authorizationView(status: TelegramAuthorizationStatus | null): {
	state: string
	qrLink: string
	passwordHint: string
} {
	return {
		state: status?.state || 'unknown',
		qrLink: status?.qrLink || '',
		passwordHint: status?.passwordHint || '',
	}
}
