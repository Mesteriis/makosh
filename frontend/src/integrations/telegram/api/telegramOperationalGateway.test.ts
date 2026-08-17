import { beforeEach, describe, expect, it, vi } from 'vitest'
import { Code, ConnectError } from '@connectrpc/connect'

import {
	loadTelegramChats,
	loadTelegramMessagePage,
	loadTelegramMessages,
	sendTelegramText,
} from './telegramOperationalGateway'
import { getTelegramOperationalConnectClient } from './telegramOperationalClient'

vi.mock('./telegramOperationalClient', () => ({
	getTelegramOperationalConnectClient: vi.fn(),
}))

const executeQuery = vi.fn()
const executeCommand = vi.fn()

describe('Telegram operational Gateway adapter', () => {
	beforeEach(() => {
		executeQuery.mockReset()
		executeCommand.mockReset()
		vi.mocked(getTelegramOperationalConnectClient).mockReturnValue({
			executeQuery,
			executeCommand,
		} as never)
	})

	it('reads provider-owned chat and message projections through exact generated queries', async () => {
		executeQuery
			.mockRejectedValueOnce(new ConnectError('runtime busy', Code.Unavailable))
			.mockResolvedValueOnce({
				response: { case: 'chats', value: { chat: [{ providerChatId: 'chat-1' }] } },
			})
			.mockResolvedValueOnce({
				response: { case: 'historyPage', value: { page: { item: [] } } },
			})
			.mockResolvedValueOnce({
				response: {
					case: 'cachedMessages',
					value: { item: [{ messageId: 'message-1', text: 'Loaded message' }] },
				},
			})

		await expect(loadTelegramChats(' account-1 ')).resolves.toHaveLength(1)
		await expect(loadTelegramMessages('account-1', 'chat-1')).resolves.toHaveLength(1)

		expect(executeQuery).toHaveBeenNthCalledWith(2, {
			query: {
				case: 'loadChats',
				value: { accountId: 'account-1', limit: 50 },
			},
		})
		expect(executeQuery).toHaveBeenNthCalledWith(3, {
			query: {
				case: 'loadHistory',
				value: {
					accountId: 'account-1',
					providerChatId: 'chat-1',
					fromMessageId: undefined,
					mode: 'latest',
					limit: 100,
				},
			},
		})
		expect(executeQuery).toHaveBeenNthCalledWith(4, {
			query: {
				case: 'cachedMessages',
				value: { accountId: 'account-1', providerChatId: 'chat-1', limit: 500 },
			},
		})
	})

	it('loads older history from the provider cursor before refreshing the larger cache window', async () => {
		executeQuery
			.mockResolvedValueOnce({
				response: {
					case: 'historyPage',
					value: { page: { item: [], nextFromMessageId: 41n, hasMore: true } },
				},
			})
			.mockResolvedValueOnce({
				response: {
					case: 'cachedMessages',
					value: { item: [{ messageId: 'message-1', text: 'Loaded message' }] },
				},
			})

		await expect(loadTelegramMessagePage('account-1', 'chat-1', 42n)).resolves.toMatchObject({
			nextFromMessageId: 41n,
			hasMore: true,
			messages: [{ messageId: 'message-1' }],
		})
		expect(executeQuery).toHaveBeenNthCalledWith(1, {
			query: {
				case: 'loadHistory',
				value: {
					accountId: 'account-1',
					providerChatId: 'chat-1',
					fromMessageId: 42n,
					mode: 'older',
					limit: 100,
				},
			},
		})
		expect(executeQuery).toHaveBeenNthCalledWith(2, {
			query: {
				case: 'cachedMessages',
				value: { accountId: 'account-1', providerChatId: 'chat-1', limit: 500 },
			},
		})
	})

	it('keeps an older provider page even when it is outside the latest durable cache window', async () => {
		executeQuery
			.mockResolvedValueOnce({
				response: {
					case: 'historyPage',
					value: {
						page: {
							item: [{
								accountId: 'account-1',
								providerChatId: 'chat-1',
								providerMessageId: 'older-1',
								senderId: '7',
								text: 'Older message',
								observedAtUnixSeconds: 1n,
							}],
							nextFromMessageId: 40n,
							hasMore: true,
						},
					},
				},
			})
			.mockResolvedValueOnce({
				response: {
					case: 'cachedMessages',
					value: { item: [{ messageId: 'latest-1', providerMessageId: 'latest-1', text: 'Latest message' }] },
				},
			})

		await expect(loadTelegramMessagePage('account-1', 'chat-1', 42n)).resolves.toMatchObject({
			messages: expect.arrayContaining([
				expect.objectContaining({ messageId: 'latest-1' }),
				expect.objectContaining({
					messageId: 'telegram:account-1:chat-1:older-1',
					text: 'Older message',
				}),
			]),
		})
	})

	it('keeps the fresh provider sender identity when the durable cache still has a generic label', async () => {
		executeQuery
			.mockResolvedValueOnce({
				response: {
					case: 'historyPage',
					value: {
						page: {
							item: [{
								providerMessageId: 'provider-message-1',
								senderId: '7',
								senderDisplayName: 'Fresh provider name',
							}],
						},
					},
				},
			})
			.mockResolvedValueOnce({
				response: {
					case: 'cachedMessages',
					value: {
						item: [{
							messageId: 'message-1',
							providerMessageId: 'provider-message-1',
							senderId: '7',
							senderDisplayName: 'Telegram user',
							text: 'Loaded message',
						}],
					},
				},
			})

		await expect(loadTelegramMessagePage('account-1', 'chat-1')).resolves.toMatchObject({
			messages: [{ senderDisplayName: 'Fresh provider name' }],
		})
	})

	it('does not render stale provider rows that contain neither text nor media', async () => {
		executeQuery
			.mockResolvedValueOnce({
				response: {
					case: 'historyPage',
					value: { page: { item: [], hasMore: false } },
				},
			})
			.mockResolvedValueOnce({
				response: {
					case: 'cachedMessages',
					value: {
						item: [
							{ messageId: 'message-complete', text: 'Loaded message' },
							{ messageId: 'message-stale', text: '', media: undefined },
						],
					},
				},
			})

		await expect(loadTelegramMessagePage('account-1', 'chat-1')).resolves.toMatchObject({
			messages: [{ messageId: 'message-complete', text: 'Loaded message' }],
		})
	})

	it('sends text through the provider command contract', async () => {
		executeCommand.mockResolvedValue({ operationId: 'operation-1', state: 'accepted' })

		await expect(sendTelegramText({
			accountId: 'account-1',
			providerChatId: 'chat-1',
			text: ' Hello ',
			operationId: 'operation-1',
		})).resolves.toMatchObject({ state: 'accepted' })

		expect(executeCommand).toHaveBeenCalledWith({
			command: {
				case: 'sendText',
				value: {
					accountId: 'account-1',
					providerChatId: 'chat-1',
					text: 'Hello',
					operationId: 'operation-1',
				},
			},
		})
	})

	it('rejects missing identifiers before transport', async () => {
		await expect(loadTelegramChats(' ')).rejects.toThrow('account ID is required')
		await expect(sendTelegramText({
			accountId: 'account-1',
			providerChatId: '',
			text: 'Hello',
			operationId: 'operation-1',
		})).rejects.toThrow('chat ID is required')
		expect(executeQuery).not.toHaveBeenCalled()
		expect(executeCommand).not.toHaveBeenCalled()
	})
})
