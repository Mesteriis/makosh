import { computed, ref, shallowRef } from 'vue'
import type { ClientModuleBootstrapV1 } from '../../../gen/makosh/gateway/v1/client_bootstrap_pb'
import type { TelegramAccountResponse } from '../../../gen/makosh/telegram/v1/client_pb'
import {
	listTelegramAccounts,
	replayTelegramAccount,
	restartTelegramAccount,
	retireTelegramAccount,
} from '../api/telegramLifecycleGateway'

const TELEGRAM_MODULE_ID = 'makosh-telegram-runtime'

type TelegramAccountManagementPorts = {
	list(): Promise<readonly TelegramAccountResponse[]>
	restart(accountId: string, expectedRuntimeEpoch: bigint): Promise<{ reconfigurationId: string; state: string }>
	replay(accountId: string, fromSequence: bigint): Promise<{ operationId: string; state: string }>
	retire(accountId: string): Promise<string>
}

export function useTelegramAccountManagement(
	module: () => ClientModuleBootstrapV1 | null,
	ports: TelegramAccountManagementPorts = defaultPorts(),
) {
	const account = shallowRef<TelegramAccountResponse | null>(null)
	const accounts = shallowRef<readonly TelegramAccountResponse[]>([])
	const accountId = ref('')
	const busy = ref(false)
	const message = ref('')
	const messageTone = ref<'neutral' | 'success' | 'error'>('neutral')
	const ownedModule = computed(() => module()?.moduleId === TELEGRAM_MODULE_ID ? module() : null)
	const canManage = computed(() => hasCapability('telegram.lifecycle.v1'))
	const canReconfigure = computed(() => hasCapability('telegram.reconfiguration.v1')
		&& (account.value?.runtimeEpoch ?? 0n) > 0n)
	const stateLabel = computed(() => account.value?.runtimeState || account.value?.state || 'No account')

	async function refresh(): Promise<void> {
		if (!canManage.value) {
			account.value = null
			accounts.value = []
			message.value = 'Telegram lifecycle capability is not admitted.'
			messageTone.value = 'neutral'
			return
		}
		await run(async () => {
			accounts.value = await ports.list()
			if (!accounts.value.some((candidate) => candidate.accountId === accountId.value)) {
				accountId.value = accounts.value[0]?.accountId ?? ''
			}
			account.value = accounts.value.find(
				(candidate) => candidate.accountId === accountId.value,
			) ?? null
			message.value = account.value
				? `Telegram account ${account.value.accountId} status refreshed.`
				: 'No Telegram accounts are provisioned in the integration runtime.'
			messageTone.value = account.value ? 'success' : 'neutral'
		}, 'Telegram account status is unavailable.')
	}

	function selectAccount(nextAccountId: string): void {
		accountId.value = nextAccountId
		account.value = accounts.value.find(
			(candidate) => candidate.accountId === nextAccountId,
		) ?? null
	}

	async function restart(): Promise<void> {
		const current = account.value
		if (!current || !canReconfigure.value) return
		await run(async () => {
			const receipt = await ports.restart(current.accountId, current.runtimeEpoch)
			message.value = `Telegram reconfiguration ${receipt.reconfigurationId} is ${receipt.state}.`
		}, 'Telegram account restart failed.')
	}

	async function replay(): Promise<void> {
		const current = account.value
		if (!current || !canManage.value) return
		await run(async () => {
			const receipt = await ports.replay(current.accountId, 0n)
			message.value = `Telegram replay ${receipt.operationId} is ${receipt.state || 'accepted'}.`
		}, 'Telegram replay failed.')
	}

	async function retire(): Promise<void> {
		const current = account.value
		if (!current || !canManage.value) return
		await run(async () => {
			const operationId = await ports.retire(current.accountId)
			message.value = `Telegram retire operation ${operationId} accepted.`
		}, 'Telegram account retirement failed.')
	}

	async function run(action: () => Promise<void>, failure: string): Promise<void> {
		busy.value = true
		message.value = ''
		messageTone.value = 'success'
		try {
			await action()
		} catch {
			message.value = failure
			messageTone.value = 'error'
		} finally {
			busy.value = false
		}
	}

	function hasCapability(capabilityId: string): boolean {
		return ownedModule.value?.capabilityIds.includes(capabilityId) ?? false
	}

	return {
		account,
		accounts,
		accountId,
		busy,
		message,
		messageTone,
		stateLabel,
		canManage,
		canReconfigure,
		refresh,
		selectAccount,
		restart,
		replay,
		retire,
	}
}

function defaultPorts(): TelegramAccountManagementPorts {
	return {
		list: listTelegramAccounts,
		restart: restartTelegramAccount,
		replay: replayTelegramAccount,
		retire: retireTelegramAccount,
	}
}
