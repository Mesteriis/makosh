import { computed, ref, shallowRef } from 'vue'
import type {
	MailFolderV1,
	MailMessageSummaryV1,
	MailThreadV1,
} from '../../../gen/hermes/mail/operational/v1/client_pb'
import { MailFolderKindV1 } from '../../../gen/hermes/mail/operational/v1/client_pb'
import {
	getMailOperationalMessage,
	listMailOperationalFolders,
	listMailOperationalMessages,
	listMailOperationalThreads,
} from '../api/mailOperationalReadGateway'
import {
	buildMailConnectionOptions,
	buildMailFolderRows,
	buildMailMessageDetail,
	buildMailMessageRows,
	buildMailThreadRows,
	type MailOperationalReadModel,
	type MailOperationalReadStatus,
} from '../presentation/mailOperationalReadModel'
import type { MailAccountConnection } from './mailAccountConnections'

export function useMailOperationalRead(input: {
	canQuery: () => boolean
	connections: () => readonly MailAccountConnection[]
}) {
	const status = ref<MailOperationalReadStatus>('blocked')
	const statusMessage = ref('')
	const selectedConnectionId = ref('')
	const selectedFolderId = ref('')
	const selectedThreadId = ref('')
	const selectedMessageId = ref('')
	const folders = shallowRef<readonly MailFolderV1[]>([])
	const threads = shallowRef<readonly MailThreadV1[]>([])
	const messages = shallowRef<readonly MailMessageSummaryV1[]>([])
	const detail = shallowRef<MailMessageSummaryV1>()
	const folderCursor = ref('')
	const threadCursor = ref('')
	const messageCursor = ref('')
	let generation = 0

	const connections = computed(input.connections)
	const model = computed<MailOperationalReadModel>(() => ({
		canQuery: input.canQuery(),
		status: status.value,
		statusMessage: statusMessage.value,
		connections: buildMailConnectionOptions(connections.value),
		selectedConnectionId: selectedConnectionId.value,
		folders: buildMailFolderRows(folders.value, selectedFolderId.value),
		threads: buildMailThreadRows(threads.value, selectedThreadId.value),
		messages: buildMailMessageRows(messages.value, selectedMessageId.value),
		detail: buildMailMessageDetail(detail.value),
		hasMoreFolders: Boolean(folderCursor.value),
		hasMoreThreads: Boolean(threadCursor.value),
		hasMoreMessages: Boolean(messageCursor.value),
	}))

	async function reconcile(): Promise<void> {
		const available = connections.value
		if (!input.canQuery()) {
			clear('Mail operational query capability is not admitted.')
			return
		}
		if (available.length === 0) {
			clear('No admitted Mail connection is available in effective integration settings.')
			status.value = 'empty'
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
		begin('Loading Mail folders…')
		resetProjection()
		try {
			const page = await listMailOperationalFolders({
				connectionId: selectedConnectionId.value,
			})
			if (!current(token)) return
			folders.value = page.item
			folderCursor.value = page.nextCursor ?? ''
			const initialFolder = preferredFolder(page.item)
			if (!initialFolder) {
				completeEmpty('No operational Mail folders are available for this connection.')
				return
			}
			await loadFolder(initialFolder.folderId, token)
		} catch (error) {
			fail(error, token, 'Mail operational projection is unavailable.')
		}
	}

	async function selectConnection(connectionId: string): Promise<void> {
		if (!connections.value.some((connection) => connection.connectionId === connectionId)) return
		selectedConnectionId.value = connectionId
		await refresh()
	}

	async function selectFolder(folderId: string): Promise<void> {
		if (!folders.value.some((folder) => folder.folderId === folderId)) return
		const token = ++generation
		begin('Loading Mail threads…')
		try {
			await loadFolder(folderId, token)
		} catch (error) {
			fail(error, token, 'Mail threads are unavailable.')
		}
	}

	async function selectThread(providerThreadId: string): Promise<void> {
		if (providerThreadId
			&& !threads.value.some((thread) => thread.providerThreadId === providerThreadId)) return
		const token = ++generation
		begin('Loading Mail messages…')
		try {
			await loadMessages(providerThreadId || undefined, token)
		} catch (error) {
			fail(error, token, 'Mail messages are unavailable.')
		}
	}

	async function selectMessage(messageId: string): Promise<void> {
		if (!messages.value.some((message) => message.messageId === messageId)) return
		const token = ++generation
		begin('Loading Mail message detail…')
		try {
			await loadMessage(messageId, token)
			if (current(token)) completeReady()
		} catch (error) {
			fail(error, token, 'Mail message detail is unavailable.')
		}
	}

	async function loadMoreFolders(): Promise<void> {
		const cursor = folderCursor.value
		if (!cursor || !readyForQuery()) return
		const token = ++generation
		await append(token, async () => {
			const page = await listMailOperationalFolders({
				connectionId: selectedConnectionId.value,
				cursor,
			})
			if (!current(token)) return
			folders.value = appendUnique(folders.value, page.item, (folder) => folder.folderId)
			folderCursor.value = page.nextCursor ?? ''
		}, 'Mail folders could not be extended.')
	}

	async function loadMoreThreads(): Promise<void> {
		const cursor = threadCursor.value
		if (!cursor || !readyForQuery()) return
		const token = ++generation
		await append(token, async () => {
			const page = await listMailOperationalThreads({
				connectionId: selectedConnectionId.value,
				folderId: selectedFolderId.value,
				cursor,
			})
			if (!current(token)) return
			threads.value = appendUnique(
				threads.value,
				page.item,
				(thread) => thread.providerThreadId,
			)
			threadCursor.value = page.nextCursor ?? ''
		}, 'Mail threads could not be extended.')
	}

	async function loadMoreMessages(): Promise<void> {
		const cursor = messageCursor.value
		if (!cursor || !readyForQuery()) return
		const token = ++generation
		await append(token, async () => {
			const page = await listMailOperationalMessages({
				connectionId: selectedConnectionId.value,
				folderId: selectedFolderId.value,
				providerThreadId: selectedThreadId.value || undefined,
				cursor,
			})
			if (!current(token)) return
			messages.value = appendUnique(
				messages.value,
				page.item,
				(message) => message.messageId,
			)
			messageCursor.value = page.nextCursor ?? ''
		}, 'Mail messages could not be extended.')
	}

	async function loadFolder(folderId: string, token: number): Promise<void> {
		selectedFolderId.value = folderId
		selectedThreadId.value = ''
		selectedMessageId.value = ''
		threads.value = []
		messages.value = []
		detail.value = undefined
		threadCursor.value = ''
		messageCursor.value = ''
		const page = await listMailOperationalThreads({
			connectionId: selectedConnectionId.value,
			folderId,
		})
		if (!current(token)) return
		threads.value = page.item
		threadCursor.value = page.nextCursor ?? ''
		await loadMessages(undefined, token)
	}

	async function loadMessages(providerThreadId: string | undefined, token: number): Promise<void> {
		selectedThreadId.value = providerThreadId ?? ''
		selectedMessageId.value = ''
		messages.value = []
		detail.value = undefined
		messageCursor.value = ''
		const page = await listMailOperationalMessages({
			connectionId: selectedConnectionId.value,
			folderId: selectedFolderId.value,
			providerThreadId,
		})
		if (!current(token)) return
		messages.value = page.item
		messageCursor.value = page.nextCursor ?? ''
		const initialMessage = page.item[0]
		if (!initialMessage) {
			completeEmpty('No operational Mail messages are available in this selection.')
			return
		}
		await loadMessage(initialMessage.messageId, token)
		if (current(token)) completeReady()
	}

	async function loadMessage(messageId: string, token: number): Promise<void> {
		selectedMessageId.value = messageId
		const response = await getMailOperationalMessage({
			connectionId: selectedConnectionId.value,
			messageId,
		})
		if (!current(token)) return
		detail.value = response.summary
	}

	async function append(
		token: number,
		work: () => Promise<void>,
		fallback: string,
	): Promise<void> {
		begin('Loading the next Mail page…')
		try {
			await work()
			if (current(token)) completeReady()
		} catch (error) {
			fail(error, token, fallback)
		}
	}

	function readyForQuery(): boolean {
		if (!input.canQuery()) {
			clear('Mail operational query capability is not admitted.')
			return false
		}
		if (!selectedConnectionId.value) {
			clear('Select an admitted Mail connection.')
			status.value = 'empty'
			return false
		}
		return true
	}

	function begin(message: string): void {
		status.value = 'loading'
		statusMessage.value = message
	}

	function completeReady(): void {
		status.value = 'ready'
		statusMessage.value = ''
	}

	function completeEmpty(message: string): void {
		status.value = 'empty'
		statusMessage.value = message
	}

	function clear(message: string): void {
		generation += 1
		resetProjection()
		selectedConnectionId.value = ''
		status.value = 'blocked'
		statusMessage.value = message
	}

	function resetProjection(): void {
		selectedFolderId.value = ''
		selectedThreadId.value = ''
		selectedMessageId.value = ''
		folders.value = []
		threads.value = []
		messages.value = []
		detail.value = undefined
		folderCursor.value = ''
		threadCursor.value = ''
		messageCursor.value = ''
	}

	function fail(error: unknown, token: number, fallback: string): void {
		if (!current(token)) return
		status.value = 'error'
		statusMessage.value = error instanceof Error ? error.message : fallback
	}

	function current(token: number): boolean {
		return token === generation
	}

	return {
		model,
		loadMoreFolders,
		loadMoreMessages,
		loadMoreThreads,
		reconcile,
		refresh,
		selectConnection,
		selectFolder,
		selectMessage,
		selectThread,
	}
}

function preferredFolder(folders: readonly MailFolderV1[]): MailFolderV1 | undefined {
	return folders.find(
		(folder) => folder.kind === MailFolderKindV1.MAIL_FOLDER_KIND_INBOX,
	) ?? folders[0]
}

function appendUnique<T>(
	current: readonly T[],
	next: readonly T[],
	key: (value: T) => string,
): readonly T[] {
	const existing = new Set(current.map(key))
	return [...current, ...next.filter((value) => !existing.has(key(value)))]
}
