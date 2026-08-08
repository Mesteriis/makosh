import type { TelegramOperationResponse } from '../../../gen/makosh/telegram/v1/client_pb'
import { getTelegramOperationalConnectClient } from './telegramOperationalClient'

const PAGE_LIMIT = 100

export type TelegramTopicTarget = {
	accountId: string
	providerChatId: string
	operationId: string
}

export async function requestTelegramMessageSearch(
	target: TelegramTopicTarget,
	query: string,
): Promise<TelegramOperationResponse> {
	return getTelegramOperationalConnectClient().executeCommand({
		command: {
			case: 'searchMessages',
			value: {
				...normalizeTarget(target),
				query: requireIdentifier('search query', query),
				limit: PAGE_LIMIT,
			},
		},
	})
}

export async function requestTelegramParticipants(
	target: TelegramTopicTarget,
): Promise<TelegramOperationResponse> {
	return getTelegramOperationalConnectClient().executeCommand({
		command: {
			case: 'listParticipants',
			value: {
				...normalizeTarget(target),
				filter: '',
				offset: 0,
				limit: PAGE_LIMIT,
			},
		},
	})
}

export async function requestTelegramTopics(
	target: TelegramTopicTarget,
): Promise<TelegramOperationResponse> {
	return getTelegramOperationalConnectClient().executeCommand({
		command: {
			case: 'listTopics',
			value: { ...normalizeTarget(target), limit: PAGE_LIMIT },
		},
	})
}

export async function createTelegramTopic(
	target: TelegramTopicTarget,
	title: string,
): Promise<TelegramOperationResponse> {
	return getTelegramOperationalConnectClient().executeCommand({
		command: {
			case: 'createTopic',
			value: {
				...normalizeTarget(target),
				title: requireIdentifier('topic title', title),
			},
		},
	})
}

export async function setTelegramTopicClosed(
	target: TelegramTopicTarget,
	providerTopicId: string,
	isClosed: boolean,
): Promise<TelegramOperationResponse> {
	return getTelegramOperationalConnectClient().executeCommand({
		command: {
			case: 'setTopicClosed',
			value: {
				...normalizeTarget(target),
				providerTopicId: requireIdentifier('topic ID', providerTopicId),
				isClosed,
			},
		},
	})
}

function normalizeTarget(target: TelegramTopicTarget): TelegramTopicTarget {
	return {
		accountId: requireIdentifier('account ID', target.accountId),
		providerChatId: requireIdentifier('chat ID', target.providerChatId),
		operationId: requireIdentifier('operation ID', target.operationId),
	}
}

function requireIdentifier(label: string, value: string): string {
	const normalized = value.trim()
	if (!normalized) {
		throw new RangeError(`${label} is required`)
	}
	return normalized
}
