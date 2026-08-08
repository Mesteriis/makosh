import { computed, onBeforeUnmount, ref } from 'vue'

import type { TelegramAccountResponse } from '../../../gen/makosh/telegram/v1/client_pb'
import {
	getTelegramAuthorizationStatus,
	submitTelegramAuthorizationPassword,
} from '../api/telegramAuthorizationGateway'
import type { TelegramAuthorizationStatus } from '../api/telegramAuthorizationGateway'
import {
	listTelegramAccounts,
	provisionTelegramAccount,
	replayTelegramAccount,
	restartTelegramAccount,
	retireTelegramAccount,
} from '../api/telegramLifecycleGateway'
import {
	authorizationView,
	buildTelegramAccountRows,
} from '../presentation/telegramAccountAccessModel'
import type { TelegramAccountAccessModel } from '../presentation/telegramAccountAccessModel'
import { telegramQrDataUrl } from '../linking/telegramQrArtifact'

export function useTelegramAccountAccess(capabilities: {
	canAuthorize: () => boolean
	canManageLifecycle: () => boolean
	canReconfigure: () => boolean
}) {
	const accounts = ref<readonly TelegramAccountResponse[]>([])
	const selectedAccountId = ref('')
	const authorization = ref<TelegramAuthorizationStatus | null>(null)
	const authorizationQrDataUrl = ref('')
	const password = ref('')
	const provisionAccountId = ref('')
	const provisionDisplayName = ref('')
	const provisionExternalAccountId = ref('')
	const statusMessage = ref('')
	const pending = ref(false)

	const model = computed<TelegramAccountAccessModel>(() => {
		const authorizationState = authorizationView(authorization.value)
		return {
			accounts: buildTelegramAccountRows(accounts.value, selectedAccountId.value),
			selectedAccountId: selectedAccountId.value,
			authorizationState: authorizationState.state,
			authorizationQrDataUrl: authorizationQrDataUrl.value,
			authorizationPasswordHint: authorizationState.passwordHint,
			password: password.value,
			provisionAccountId: provisionAccountId.value,
			provisionDisplayName: provisionDisplayName.value,
			provisionExternalAccountId: provisionExternalAccountId.value,
			statusMessage: statusMessage.value,
			pending: pending.value,
			canAuthorize: capabilities.canAuthorize(),
			canManageLifecycle: capabilities.canManageLifecycle(),
			canReconfigure: capabilities.canReconfigure(),
		}
	})

	async function refresh(): Promise<void> {
		pending.value = true
		statusMessage.value = ''
		try {
			const [nextAccounts, nextAuthorization] = await Promise.all([
				capabilities.canManageLifecycle() ? listTelegramAccounts() : Promise.resolve([]),
				capabilities.canAuthorize() ? getTelegramAuthorizationStatus() : Promise.resolve(null),
			])
			accounts.value = nextAccounts
			authorization.value = nextAuthorization
			authorizationQrDataUrl.value = nextAuthorization?.qrLink
				? await telegramQrDataUrl(nextAuthorization.qrLink)
				: ''
			if (!selectedAccountId.value && nextAccounts[0]) {
				selectedAccountId.value = nextAccounts[0].accountId
			}
			statusMessage.value = nextAccounts.length === 0
				? 'No Telegram accounts are provisioned.'
				: `${nextAccounts.length} Telegram account${nextAccounts.length === 1 ? '' : 's'} available.`
		} catch (error) {
			authorizationQrDataUrl.value = ''
			statusMessage.value = message(error, 'Telegram account access is unavailable.')
		} finally {
			pending.value = false
		}
	}

	async function provision(): Promise<void> {
		await runLifecycleAction(async () => {
			const account = await provisionTelegramAccount({
				accountId: provisionAccountId.value,
				displayName: provisionDisplayName.value,
				externalAccountId: provisionExternalAccountId.value,
				credentials: [],
			})
			selectedAccountId.value = account.accountId
			statusMessage.value = `Telegram account ${account.accountId} provisioned.`
			await refresh()
		})
	}

	async function restart(): Promise<void> {
		if (!capabilities.canReconfigure()) {
			statusMessage.value = 'Telegram reconfiguration capability is not admitted.'
			return
		}
		await runSelectedAccountAction(async (accountId) => {
			const account = accounts.value.find((candidate) => candidate.accountId === accountId)
			if (!account || account.runtimeEpoch <= 0n) {
				throw new Error('Telegram runtime epoch is unavailable')
			}
			const result = await restartTelegramAccount(
				accountId,
				account.runtimeEpoch,
			)
			return `Reconfiguration ${result.reconfigurationId} is ${result.state}.`
		})
	}

	async function replay(): Promise<void> {
		await runSelectedAccountAction(async (accountId) => {
			const operation = await replayTelegramAccount(accountId, 0n)
			return `Replay operation ${operation.operationId} is ${operation.state || 'accepted'}.`
		})
	}

	async function retire(): Promise<void> {
		const accountId = requireSelectedAccount()
		if (!window.confirm(`Retire Telegram account ${accountId}? Provider state will be fenced.`)) {
			return
		}
		await runLifecycleAction(async () => {
			const operationId = await retireTelegramAccount(accountId)
			statusMessage.value = `Retire operation ${operationId} accepted.`
			await refresh()
		})
	}

	async function submitPassword(): Promise<void> {
		if (!capabilities.canAuthorize()) {
			statusMessage.value = 'Telegram authorization capability is not admitted.'
			return
		}
		pending.value = true
		try {
			await submitTelegramAuthorizationPassword(password.value)
			password.value = ''
			authorization.value = await getTelegramAuthorizationStatus()
			authorizationQrDataUrl.value = authorization.value.qrLink
				? await telegramQrDataUrl(authorization.value.qrLink)
				: ''
			statusMessage.value = 'Telegram authorization password accepted.'
		} catch (error) {
			statusMessage.value = message(error, 'Telegram authorization failed.')
		} finally {
			pending.value = false
		}
	}

	function selectAccount(accountId: string): void {
		selectedAccountId.value = accountId
	}

	function updatePassword(value: string): void {
		password.value = value
	}

	function updateProvisionAccountId(value: string): void {
		provisionAccountId.value = value
	}

	function updateProvisionDisplayName(value: string): void {
		provisionDisplayName.value = value
	}

	function updateProvisionExternalAccountId(value: string): void {
		provisionExternalAccountId.value = value
	}

	async function runSelectedAccountAction(
		action: (accountId: string) => Promise<string>,
	): Promise<void> {
		await runLifecycleAction(async () => {
			statusMessage.value = await action(requireSelectedAccount())
			await refresh()
		})
	}

	async function runLifecycleAction(action: () => Promise<void>): Promise<void> {
		if (!capabilities.canManageLifecycle()) {
			statusMessage.value = 'Telegram lifecycle capability is not admitted.'
			return
		}
		pending.value = true
		try {
			await action()
		} catch (error) {
			statusMessage.value = message(error, 'Telegram lifecycle action failed.')
		} finally {
			pending.value = false
		}
	}

	function requireSelectedAccount(): string {
		if (!selectedAccountId.value) {
			throw new RangeError('Telegram account selection is required')
		}
		return selectedAccountId.value
	}

	onBeforeUnmount(() => {
		authorizationQrDataUrl.value = ''
		password.value = ''
	})

	return {
		model,
		selectedAccountId,
		refresh,
		provision,
		restart,
		replay,
		retire,
		submitPassword,
		selectAccount,
		updatePassword,
		updateProvisionAccountId,
		updateProvisionDisplayName,
		updateProvisionExternalAccountId,
	}
}

function message(error: unknown, fallback: string): string {
	return error instanceof Error ? error.message : fallback
}
