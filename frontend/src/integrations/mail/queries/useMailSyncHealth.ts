import { computed, ref, shallowRef } from 'vue'

import type {
	MailSyncRunV1,
	MailSyncStatusV1,
} from '../../../gen/makosh/mail/sync_health/v1/client_pb'
import {
	getMailSyncStatus,
	listMailSyncRuns,
} from '../api/mailSyncHealthGateway'
import {
	buildMailSyncHealthModel,
	type MailSyncHealthState,
} from '../presentation/mailSyncHealthModel'
import type { MailAccountConnection } from './mailAccountConnections'

export function useMailSyncHealth(input: {
	canQuery: () => boolean
	connections: () => readonly MailAccountConnection[]
}) {
	const state = ref<MailSyncHealthState>('blocked')
	const statusMessage = ref('')
	const selectedConnectionId = ref('')
	const status = shallowRef<MailSyncStatusV1>()
	const runs = shallowRef<readonly MailSyncRunV1[]>([])
	const nextCursor = ref('')
	let generation = 0

	const connections = computed(input.connections)
	const model = computed(() => buildMailSyncHealthModel({
		canQuery: input.canQuery(),
		state: state.value,
		statusMessage: statusMessage.value,
		connections: connections.value,
		selectedConnectionId: selectedConnectionId.value,
		status: status.value,
		runs: runs.value,
		hasMoreRuns: Boolean(nextCursor.value),
	}))

	async function reconcile(): Promise<void> {
		const available = connections.value
		if (!input.canQuery()) {
			clear('Mail sync health capability is not admitted.')
			return
		}
		if (available.length === 0) {
			clear('No admitted Mail connection exposes sync health.')
			state.value = 'empty'
			return
		}
		if (!available.some((connection) => connection.connectionId === selectedConnectionId.value)) {
			selectedConnectionId.value = available[0]!.connectionId
		}
		await refresh()
	}

	async function refresh(): Promise<void> {
		if (!readyForQuery()) return
		const token = ++generation
		const connectionId = selectedConnectionId.value
		state.value = 'loading'
		statusMessage.value = 'Loading persisted Mail sync health…'
		status.value = undefined
		runs.value = []
		nextCursor.value = ''
		try {
			const [nextStatus, page] = await Promise.all([
				getMailSyncStatus(connectionId),
				listMailSyncRuns({ connectionId }),
			])
			if (!current(token)) return
			status.value = nextStatus
			runs.value = page.item
			nextCursor.value = page.nextCursor ?? ''
			state.value = 'ready'
			statusMessage.value = page.item.length === 0
				? 'No persisted sync runs are available for this connection.'
				: ''
		} catch (error) {
			fail(error, token, 'Mail sync health is unavailable.')
		}
	}

	async function selectConnection(connectionId: string): Promise<void> {
		if (!connections.value.some((connection) => connection.connectionId === connectionId)) return
		selectedConnectionId.value = connectionId
		await refresh()
	}

	async function loadMore(): Promise<void> {
		const cursor = nextCursor.value
		if (!cursor || !readyForQuery()) return
		const token = ++generation
		const connectionId = selectedConnectionId.value
		state.value = 'loading'
		statusMessage.value = 'Loading the next Mail sync run page…'
		try {
			const page = await listMailSyncRuns({ connectionId, cursor })
			if (!current(token)) return
			runs.value = appendUniqueRuns(runs.value, page.item)
			nextCursor.value = page.nextCursor ?? ''
			state.value = 'ready'
			statusMessage.value = ''
		} catch (error) {
			fail(error, token, 'Mail sync history could not be extended.')
		}
	}

	function readyForQuery(): boolean {
		if (!input.canQuery()) {
			clear('Mail sync health capability is not admitted.')
			return false
		}
		if (!selectedConnectionId.value) {
			clear('Select an admitted Mail connection.')
			state.value = 'empty'
			return false
		}
		return true
	}

	function clear(message: string): void {
		generation += 1
		selectedConnectionId.value = ''
		status.value = undefined
		runs.value = []
		nextCursor.value = ''
		state.value = 'blocked'
		statusMessage.value = message
	}

	function fail(error: unknown, token: number, fallback: string): void {
		if (!current(token)) return
		state.value = 'error'
		statusMessage.value = error instanceof Error ? error.message : fallback
	}

	function current(token: number): boolean {
		return token === generation
	}

	return {
		loadMore,
		model,
		reconcile,
		refresh,
		selectConnection,
	}
}

function appendUniqueRuns(
	current: readonly MailSyncRunV1[],
	next: readonly MailSyncRunV1[],
): readonly MailSyncRunV1[] {
	const known = new Set(current.map((run) => run.operationId))
	const appended = [...current]
	for (const run of next) {
		if (known.has(run.operationId)) continue
		known.add(run.operationId)
		appended.push(run)
	}
	return appended
}
