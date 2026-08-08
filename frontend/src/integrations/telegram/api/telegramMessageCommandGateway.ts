import type { TelegramOperationResponse } from '../../../gen/makosh/telegram/v1/client_pb'
import { getTelegramOperationalConnectClient } from './telegramOperationalClient'

export type TelegramMessageTarget = {
	accountId: string
	providerChatId: string
	providerMessageId: string
	operationId: string
}

export async function replyToTelegramMessage(
	target: TelegramMessageTarget,
	text: string,
): Promise<TelegramOperationResponse> {
	const normalized = normalizeTarget(target)
	return getTelegramOperationalConnectClient().executeCommand({
		command: {
			case: 'reply',
			value: {
				operationId: normalized.operationId,
				accountId: normalized.accountId,
				providerChatId: normalized.providerChatId,
				replyToProviderMessageId: normalized.providerMessageId,
				text: requireText(text),
			},
		},
	})
}

export async function forwardTelegramMessage(input: TelegramMessageTarget & {
	targetProviderChatId: string
}): Promise<TelegramOperationResponse> {
	const target = normalizeTarget(input)
	return getTelegramOperationalConnectClient().executeCommand({
		command: {
			case: 'forward',
			value: {
				operationId: target.operationId,
				accountId: target.accountId,
				providerChatId: requireIdentifier('target chat ID', input.targetProviderChatId),
				fromProviderChatId: target.providerChatId,
				fromProviderMessageId: target.providerMessageId,
			},
		},
	})
}

export async function editTelegramMessage(
	target: TelegramMessageTarget,
	text: string,
): Promise<TelegramOperationResponse> {
	return getTelegramOperationalConnectClient().executeCommand({
		command: {
			case: 'edit',
			value: {
				...normalizeTarget(target),
				text: requireText(text),
			},
		},
	})
}

export async function deleteTelegramMessage(
	target: TelegramMessageTarget,
	revoke: boolean,
): Promise<TelegramOperationResponse> {
	return getTelegramOperationalConnectClient().executeCommand({
		command: {
			case: 'delete',
			value: {
				...normalizeTarget(target),
				revoke,
			},
		},
	})
}

export async function restoreTelegramMessageVisibility(
	target: TelegramMessageTarget,
	reason: string,
): Promise<TelegramOperationResponse> {
	return getTelegramOperationalConnectClient().executeCommand({
		command: {
			case: 'restoreVisibility',
			value: {
				...normalizeTarget(target),
				reason: requireIdentifier('restore reason', reason),
			},
		},
	})
}

export async function setTelegramMessageReaction(
	target: TelegramMessageTarget,
	emoji: string,
	active: boolean,
): Promise<TelegramOperationResponse> {
	return getTelegramOperationalConnectClient().executeCommand({
		command: {
			case: 'reaction',
			value: {
				...normalizeTarget(target),
				emoji: requireIdentifier('reaction', emoji),
				active,
			},
		},
	})
}

export async function setTelegramMessagePinned(
	target: TelegramMessageTarget,
	active: boolean,
): Promise<TelegramOperationResponse> {
	return getTelegramOperationalConnectClient().executeCommand({
		command: {
			case: 'pin',
			value: {
				...normalizeTarget(target),
				active,
			},
		},
	})
}

function normalizeTarget(target: TelegramMessageTarget): TelegramMessageTarget {
	return {
		accountId: requireIdentifier('account ID', target.accountId),
		providerChatId: requireIdentifier('chat ID', target.providerChatId),
		providerMessageId: requireIdentifier('message ID', target.providerMessageId),
		operationId: requireIdentifier('operation ID', target.operationId),
	}
}

function requireText(value: string): string {
	const normalized = value.trim()
	if (!normalized) {
		throw new RangeError('Telegram message text is required')
	}
	return normalized
}

function requireIdentifier(label: string, value: string): string {
	const normalized = value.trim()
	if (!normalized) {
		throw new RangeError(`${label} is required`)
	}
	return normalized
}
