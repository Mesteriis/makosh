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
	selectedAccountOperational: boolean
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
	authorizationState = 'unknown',
): readonly TelegramAccountRow[] {
	return accounts.map((account) => {
		const selected = account.accountId === selectedAccountId
		return {
			id: account.accountId,
			title: account.displayName || account.externalAccountId || account.accountId,
			detail: [
				account.state,
				account.runtimeState,
				selected ? `authorization: ${authorizationState}` : '',
			].filter(Boolean).join(' · ') || 'provisioned',
			selected,
		}
	})
}

export function isTelegramAccountOperational(
	accounts: readonly TelegramAccountResponse[],
	selectedAccountId: string,
): boolean {
	const selected = accounts.find(account => account.accountId === selectedAccountId)
	return selected?.state === 'ready'
		&& (selected.runtimeState === 'running' || selected.runtimeState === 'degraded')
}

export function canStartTelegramOperationalLane(
	model: Pick<
		TelegramAccountAccessModel,
		'selectedAccountId' | 'selectedAccountOperational' | 'authorizationState'
	>,
): boolean {
	return Boolean(model.selectedAccountId)
		&& model.selectedAccountOperational
		&& model.authorizationState === 'ready'
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
