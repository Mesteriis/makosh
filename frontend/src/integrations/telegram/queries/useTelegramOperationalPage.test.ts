import { beforeEach, describe, expect, it, vi } from 'vitest'

import { useTelegramOperationalPage } from './useTelegramOperationalPage'
import {
	loadTelegramChats,
	loadTelegramMessagePage,
	loadTelegramParticipants,
	readCachedTelegramChats,
	readCachedTelegramMessages,
} from '../api/telegramOperationalGateway'
import { openTelegramOperationalRealtime } from '../api/telegramOperationalRealtime'

vi.mock('../api/telegramOperationalGateway', () => ({
	loadTelegramChats: vi.fn(),
	loadTelegramMessagePage: vi.fn(),
	loadTelegramParticipants: vi.fn(),
	readCachedTelegramChats: vi.fn(),
	readCachedTelegramMessages: vi.fn(),
	sendTelegramText: vi.fn(),
}))
vi.mock('../api/telegramOperationalRealtime', () => ({ openTelegramOperationalRealtime: vi.fn() }))
vi.mock('../api/telegramMessageCommandGateway', () => ({ replyToTelegramMessage: vi.fn() }))

let realtimeInput: {
	onProjectionChanged(latestSequence: bigint): void
	onLive(): void
	onUnavailable(): void
} | undefined

describe('Telegram operational page state', () => {
	beforeEach(() => {
		vi.clearAllMocks()
		vi.mocked(loadTelegramParticipants).mockResolvedValue([])
		realtimeInput = undefined
		vi.mocked(openTelegramOperationalRealtime).mockImplementation((_accountId, input) => {
			realtimeInput = input
			return { close: vi.fn() }
		})
	})

	it('loads chats incrementally instead of requesting the entire catalog', async () => {
		vi.mocked(loadTelegramChats)
			.mockResolvedValueOnce(Array.from({ length: 50 }, (_, index) => chat(`chat-${index}`)) as never)
			.mockResolvedValueOnce(Array.from({ length: 100 }, (_, index) => chat(`chat-${index}`)) as never)
		vi.mocked(loadTelegramMessagePage).mockResolvedValue(page('message-in-first') as never)
		const surface = useTelegramOperationalPage(() => true)
		surface.updateAccountId('account-1')

		await surface.loadChats()
		expect(surface.model.value.chats).toHaveLength(50)
		expect(surface.model.value.hasMoreChats).toBe(true)
		await surface.loadMoreChats()

		expect(loadTelegramChats).toHaveBeenNthCalledWith(1, 'account-1', 50)
		expect(loadTelegramChats).toHaveBeenNthCalledWith(2, 'account-1', 100)
		expect(surface.model.value.chats).toHaveLength(100)
	})

	it('keeps probing partial provider pages until a request adds no chats', async () => {
		vi.mocked(loadTelegramChats)
			.mockResolvedValueOnce(Array.from({ length: 50 }, (_, index) => chat(`chat-${index}`)) as never)
			.mockResolvedValueOnce(Array.from({ length: 80 }, (_, index) => chat(`chat-${index}`)) as never)
			.mockResolvedValueOnce(Array.from({ length: 80 }, (_, index) => chat(`chat-${index}`)) as never)
		vi.mocked(loadTelegramMessagePage).mockResolvedValue(page('message-in-first') as never)
		const surface = useTelegramOperationalPage(() => true)
		surface.updateAccountId('account-1')

		await surface.loadChats()
		await surface.loadMoreChats()
		expect(surface.model.value.hasMoreChats).toBe(true)
		await surface.loadMoreChats()

		expect(loadTelegramChats).toHaveBeenNthCalledWith(2, 'account-1', 100)
		expect(loadTelegramChats).toHaveBeenNthCalledWith(3, 'account-1', 150)
		expect(surface.model.value.chats).toHaveLength(80)
		expect(surface.model.value.hasMoreChats).toBe(false)
	})

	it('does not expose stale messages when an older chat request finishes last', async () => {
		const first = deferred<ReturnType<typeof page>>()
		const second = deferred<ReturnType<typeof page>>()
		vi.mocked(loadTelegramMessagePage).mockImplementation((_accountId, providerChatId) =>
			(providerChatId === 'chat-1' ? first.promise : second.promise) as never)
		const surface = useTelegramOperationalPage(() => true)
		surface.updateAccountId('account-1')

		const firstSelection = surface.selectChat('chat-1')
		expect(surface.model.value.messages).toEqual([])
		const secondSelection = surface.selectChat('chat-2')
		second.resolve(page('message-in-second'))
		await secondSelection
		first.resolve(page('message-in-first'))
		await firstSelection

		expect(surface.model.value.selectedChatId).toBe('chat-2')
		expect(surface.model.value.messages.map(message => message.id)).toEqual(['message-in-second'])
		expect(surface.model.value.historyPending).toBe(false)
	})

	it('restores a chat history immediately while refreshing it in the background', async () => {
		const refreshedFirst = deferred<ReturnType<typeof page>>()
		vi.mocked(loadTelegramMessagePage)
			.mockResolvedValueOnce(page('message-in-first') as never)
			.mockResolvedValueOnce(page('message-in-second') as never)
			.mockReturnValueOnce(refreshedFirst.promise as never)
		const surface = useTelegramOperationalPage(() => true)
		surface.updateAccountId('account-1')

		await surface.selectChat('chat-1')
		await surface.selectChat('chat-2')
		const returning = surface.selectChat('chat-1')

		expect(surface.model.value.selectedChatId).toBe('chat-1')
		expect(surface.model.value.messages.map(message => message.id)).toEqual(['message-in-first'])
		expect(surface.model.value.historyPending).toBe(true)
		refreshedFirst.resolve(page('message-in-first-refreshed'))
		await returning

		expect(surface.model.value.messages.map(message => message.id)).toEqual(['message-in-first-refreshed'])
		expect(surface.model.value.historyPending).toBe(false)
	})

	it('renders provider history before the participant directory finishes loading', async () => {
		const participants = deferred<readonly object[]>()
		vi.mocked(loadTelegramMessagePage).mockResolvedValue(page('message-in-first') as never)
		vi.mocked(loadTelegramParticipants).mockReturnValue(participants.promise as never)
		const surface = useTelegramOperationalPage(() => true)
		surface.updateAccountId('account-1')

		const selection = surface.selectChat('chat-1')
		await Promise.resolve()
		await Promise.resolve()

		expect(surface.model.value.messages.map(message => message.id)).toEqual(['message-in-first'])
		expect(surface.model.value.historyPending).toBe(false)
		participants.resolve([])
		await selection
	})

	it('does not replace a loaded chat with an empty transient refresh', async () => {
		vi.mocked(loadTelegramMessagePage)
			.mockResolvedValueOnce(page('message-in-first') as never)
			.mockResolvedValueOnce({ messages: [], hasMore: false } as never)
		const surface = useTelegramOperationalPage(() => true)
		surface.updateAccountId('account-1')

		await surface.selectChat('chat-1')
		await surface.selectChat('chat-1')

		expect(surface.model.value.messages.map(message => message.id)).toEqual(['message-in-first'])
		expect(surface.model.value.status).toBe('ready')
	})

	it('keeps a short provider history page available for lazy pagination', async () => {
		vi.mocked(loadTelegramMessagePage).mockResolvedValue({
			...page('message-in-first'),
			nextFromMessageId: 90n,
			hasMore: true,
		} as never)
		const surface = useTelegramOperationalPage(() => true)
		surface.updateAccountId('account-1')

		await surface.selectChat('chat-1')

		expect(surface.model.value.messages.map(message => message.id)).toEqual(['message-in-first'])
		expect(surface.model.value.hasOlderMessages).toBe(true)
	})

	it('accumulates older pages instead of replacing already loaded history', async () => {
		vi.mocked(loadTelegramMessagePage)
			.mockResolvedValueOnce({
				...page('message-latest'),
				nextFromMessageId: 90n,
				hasMore: true,
			} as never)
			.mockResolvedValueOnce({
				...page('message-older'),
				nextFromMessageId: 80n,
				hasMore: true,
			} as never)
		const surface = useTelegramOperationalPage(() => true)
		surface.updateAccountId('account-1')

		await surface.selectChat('chat-1')
		await surface.loadOlderMessages()

		expect(surface.model.value.messages.map(message => message.id)).toEqual([
			'message-latest',
			'message-older',
		])
		expect(loadTelegramMessagePage).toHaveBeenNthCalledWith(2, 'account-1', 'chat-1', 90n)
		expect(surface.model.value.hasOlderMessages).toBe(true)
	})

	it('keeps a loaded chat page when realtime observes an empty transient cache snapshot', async () => {
		const loadedChats = Array.from({ length: 50 }, (_, index) => chat(`chat-${index}`))
		vi.mocked(loadTelegramChats).mockResolvedValue(loadedChats as never)
		vi.mocked(loadTelegramMessagePage).mockResolvedValue(page('message-in-first') as never)
		vi.mocked(readCachedTelegramChats).mockResolvedValue([])
		const surface = useTelegramOperationalPage(() => true, () => true)
		surface.updateAccountId('account-1')

		await surface.loadChats()
		await surface.startRealtime()
		realtimeInput?.onProjectionChanged(1n)
		await vi.waitFor(() => expect(readCachedTelegramChats).toHaveBeenCalled())
		surface.stopRealtime()

		expect(surface.model.value.chats).toHaveLength(50)
		expect(surface.model.value.selectedChatId).toBe('chat-0')
	})

	it('uses participant names for generic provider senders', async () => {
		vi.mocked(loadTelegramMessagePage).mockResolvedValue({
			messages: [{
				messageId: 'message-1',
				providerMessageId: 'message-1',
				senderId: '7',
				senderDisplayName: 'Telegram user',
				observedAtUnixSeconds: 1n,
				deliveryState: 'received',
			}],
			hasMore: false,
		} as never)
		vi.mocked(loadTelegramParticipants).mockResolvedValue([{
			providerMemberId: 'user:7',
			displayName: 'Provider nickname',
		} as never])
		const surface = useTelegramOperationalPage(() => true)
		surface.updateAccountId('account-1')

		await surface.selectChat('chat-1')

		expect(surface.model.value.messages[0]?.sender).toBe('Provider nickname')
	})

	it('shows authorization recovery without exposing an operational transport error', () => {
		const surface = useTelegramOperationalPage(() => true)
		surface.updateAccountId('account-1')

		surface.suspendForAuthorization('waiting_qr_scan')

		expect(surface.model.value.status).toBe('empty')
		expect(surface.model.value.statusMessage).toBe('Scan the Telegram QR code to load chats.')
	})

	it('opens the durable chat cache while provider authorization is unavailable', async () => {
		vi.mocked(readCachedTelegramChats).mockResolvedValue([chat('chat-1')] as never)
		vi.mocked(readCachedTelegramMessages).mockResolvedValue(page('cached-message').messages as never)
		const surface = useTelegramOperationalPage(() => true)
		surface.updateAccountId('account-1')

		await surface.loadCachedProjection()

		expect(surface.model.value.chats).toHaveLength(1)
		expect(surface.model.value.messages.map(message => message.id)).toEqual(['cached-message'])
		expect(surface.model.value.statusMessage).toContain('Showing cached Telegram chats')
		expect(loadTelegramMessagePage).not.toHaveBeenCalled()
	})
})

function chat(providerChatId: string): object {
	return { providerChatId, title: providerChatId, kind: 'private' }
}

function page(messageId: string): {
	messages: readonly object[]
	nextFromMessageId?: bigint
	hasMore: boolean
} {
	return {
		messages: [{
			messageId,
			providerMessageId: messageId,
			observedAtUnixSeconds: 1n,
			deliveryState: 'received',
		}],
		hasMore: false,
	}
}

function deferred<T>(): { promise: Promise<T>; resolve(value: T): void } {
	let resolvePromise: (value: T) => void = () => undefined
	const promise = new Promise<T>((resolve) => {
		resolvePromise = resolve
	})
	return { promise, resolve: resolvePromise }
}
