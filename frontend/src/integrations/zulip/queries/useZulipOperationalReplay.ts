import { computed, ref, shallowRef } from 'vue'

import type { ClientModuleBootstrapV1 } from '../../../gen/makosh/gateway/v1/client_bootstrap_pb'
import type { ZulipOperationalReplayFrameV1 } from '../../../gen/makosh/zulip/operational/realtime/v1/client_pb'
import { replayZulipOperationalEvents } from '../api/zulipOperationalReplayGateway'
import {
	buildZulipOperationalReplayModel,
	type ZulipOperationalReplayModel,
} from '../presentation/zulipOperationalReplayModel'
import type { ZulipOperationalReadState } from '../presentation/zulipOperationalReadModel'
import { zulipOperationalReplayAccounts } from './zulipOperationalAccounts'

export function useZulipOperationalReplay(input: {
	canReplay: () => boolean
	modules: () => readonly ClientModuleBootstrapV1[]
}) {
	const state = ref<ZulipOperationalReadState>('blocked')
	const statusMessage = ref('')
	const selectedAccountId = ref('')
	const earliestSequence = ref<bigint>()
	const latestSequence = ref<bigint>()
	const nextSequence = ref(0n)
	const resetRequired = ref(false)
	const frames = shallowRef<readonly ZulipOperationalReplayFrameV1[]>([])
	let generation = 0

	const accounts = computed(() => zulipOperationalReplayAccounts(input.modules()))
	const model = computed<ZulipOperationalReplayModel>(() => (
		buildZulipOperationalReplayModel({
			canReplay: input.canReplay(),
			state: state.value,
			statusMessage: statusMessage.value,
			accounts: accounts.value,
			selectedAccountId: selectedAccountId.value,
			earliestSequence: earliestSequence.value,
			latestSequence: latestSequence.value,
			nextSequence: nextSequence.value,
			resetRequired: resetRequired.value,
			frames: frames.value,
		})
	))

	async function reconcile(): Promise<void> {
		const available = accounts.value
		if (!input.canReplay()) {
			clear('Zulip operational realtime capability is not admitted.')
			return
		}
		if (available.length === 0) {
			clear('No admitted Zulip realtime account is available in effective integration settings.')
			state.value = 'empty'
			return
		}
		if (!available.some((account) => account.accountId === selectedAccountId.value)) {
			selectedAccountId.value = available[0]!.accountId
		}
		await refresh()
	}

	async function refresh(): Promise<void> {
		if (!readyForReplay()) return
		const token = ++generation
		begin('Loading Zulip replay window…')
		resetReplay()
		try {
			const response = await replayZulipOperationalEvents({
				accountId: selectedAccountId.value,
			})
			if (!current(token)) return
			applyResponse(response, false)
			finish()
		} catch (error) {
			fail(error, token)
		}
	}

	async function selectAccount(accountId: string): Promise<void> {
		if (!accounts.value.some((account) => account.accountId === accountId)) return
		selectedAccountId.value = accountId
		await refresh()
	}

	async function loadMore(): Promise<void> {
		if (!readyForReplay() || resetRequired.value) return
		if (latestSequence.value === undefined || nextSequence.value >= latestSequence.value) return
		const token = ++generation
		begin('Loading more Zulip replay frames…')
		try {
			const response = await replayZulipOperationalEvents({
				accountId: selectedAccountId.value,
				afterSequence: nextSequence.value,
			})
			if (!current(token)) return
			applyResponse(response, true)
			finish()
		} catch (error) {
			fail(error, token)
		}
	}

	function applyResponse(
		response: Awaited<ReturnType<typeof replayZulipOperationalEvents>>,
		append: boolean,
	): void {
		earliestSequence.value = response.earliestAvailableSequence
		latestSequence.value = response.latestAvailableSequence
		nextSequence.value = response.nextSequence
		resetRequired.value = response.resetRequired
		frames.value = append
			? appendUniqueFrames(frames.value, response.frame)
			: response.frame
	}

	function readyForReplay(): boolean {
		if (!input.canReplay()) {
			clear('Zulip operational realtime capability is not admitted.')
			return false
		}
		if (!selectedAccountId.value) {
			clear('Select an admitted Zulip realtime account.')
			state.value = 'empty'
			return false
		}
		return true
	}

	function begin(message: string): void {
		state.value = 'loading'
		statusMessage.value = message
	}

	function finish(): void {
		if (resetRequired.value) {
			state.value = 'error'
			statusMessage.value = 'Replay cursor is outside retention. Explicit refresh from the current window is required.'
			return
		}
		state.value = frames.value.length === 0 ? 'empty' : 'ready'
		statusMessage.value = frames.value.length === 0
			? 'No Zulip realtime frames are available for this account.'
			: ''
	}

	function clear(message: string): void {
		generation += 1
		selectedAccountId.value = ''
		resetReplay()
		state.value = 'blocked'
		statusMessage.value = message
	}

	function resetReplay(): void {
		earliestSequence.value = undefined
		latestSequence.value = undefined
		nextSequence.value = 0n
		resetRequired.value = false
		frames.value = []
	}

	function fail(error: unknown, token: number): void {
		if (!current(token)) return
		state.value = 'error'
		statusMessage.value = error instanceof Error
			? error.message
			: 'Zulip operational replay is unavailable.'
	}

	function current(token: number): boolean {
		return token === generation
	}

	return {
		model,
		loadMore,
		reconcile,
		refresh,
		selectAccount,
	}
}

function appendUniqueFrames(
	current: readonly ZulipOperationalReplayFrameV1[],
	next: readonly ZulipOperationalReplayFrameV1[],
): readonly ZulipOperationalReplayFrameV1[] {
	const existing = new Set(current.map((frame) => frame.sequence))
	return [...current, ...next.filter((frame) => !existing.has(frame.sequence))]
}
