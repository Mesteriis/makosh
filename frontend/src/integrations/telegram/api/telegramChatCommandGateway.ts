import type { TelegramOperationResponse } from '../../../gen/makosh/telegram/v1/client_pb'
import { getTelegramOperationalConnectClient } from './telegramOperationalClient'

export type TelegramChatTarget = {
	accountId: string
	providerChatId: string
	operationId: string
}

export async function setTelegramChatUnread(
	target: TelegramChatTarget,
	unread: boolean,
	readThroughProviderMessageId?: string,
): Promise<TelegramOperationResponse> {
	return getTelegramOperationalConnectClient().executeCommand({
		command: {
			case: 'markUnread',
			value: {
				...normalizeTarget(target),
				unread,
				readThroughProviderMessageId: optionalIdentifier(readThroughProviderMessageId),
			},
		},
	})
}

export async function setTelegramChatArchived(
	target: TelegramChatTarget,
	archived: boolean,
): Promise<TelegramOperationResponse> {
	return getTelegramOperationalConnectClient().executeCommand({
		command: {
			case: 'archive',
			value: { ...normalizeTarget(target), archived },
		},
	})
}

export async function setTelegramChatMuted(
	target: TelegramChatTarget,
	muted: boolean,
): Promise<TelegramOperationResponse> {
	return getTelegramOperationalConnectClient().executeCommand({
		command: {
			case: 'mute',
			value: { ...normalizeTarget(target), muted },
		},
	})
}

export async function joinTelegramChat(
	target: TelegramChatTarget,
): Promise<TelegramOperationResponse> {
	return getTelegramOperationalConnectClient().executeCommand({
		command: { case: 'join', value: normalizeTarget(target) },
	})
}

export async function leaveTelegramChat(
	target: TelegramChatTarget,
): Promise<TelegramOperationResponse> {
	return getTelegramOperationalConnectClient().executeCommand({
		command: { case: 'leave', value: normalizeTarget(target) },
	})
}

export async function addTelegramChatToFolder(
	target: TelegramChatTarget,
	providerFolderId: bigint,
): Promise<TelegramOperationResponse> {
	return executeFolderCommand('addChatToFolder', target, providerFolderId)
}

export async function removeTelegramChatFromFolder(
	target: TelegramChatTarget,
	providerFolderId: bigint,
): Promise<TelegramOperationResponse> {
	return executeFolderCommand('removeChatFromFolder', target, providerFolderId)
}

export async function reassignTelegramChatFolders(
	target: TelegramChatTarget,
	targetProviderFolderIds: readonly bigint[],
): Promise<TelegramOperationResponse> {
	const uniqueFolderIds = new Set(targetProviderFolderIds)
	if (
		targetProviderFolderIds.length === 0
		|| targetProviderFolderIds.length > 64
		|| uniqueFolderIds.size !== targetProviderFolderIds.length
		|| targetProviderFolderIds.some((folderId) => folderId <= 0n)
	) {
		throw new RangeError('Telegram target folder IDs must be 1-64 unique positive integers')
	}
	return getTelegramOperationalConnectClient().executeCommand({
		command: {
			case: 'reassignChatFolders',
			value: {
				...normalizeTarget(target),
				targetProviderFolderIds: [...targetProviderFolderIds],
			},
		},
	})
}

async function executeFolderCommand(
	command: 'addChatToFolder' | 'removeChatFromFolder',
	target: TelegramChatTarget,
	providerFolderId: bigint,
): Promise<TelegramOperationResponse> {
	if (providerFolderId < 0n) {
		throw new RangeError('Telegram folder ID must be non-negative')
	}
	const value = { ...normalizeTarget(target), providerFolderId }
	if (command === 'addChatToFolder') {
		return getTelegramOperationalConnectClient().executeCommand({
			command: { case: 'addChatToFolder', value },
		})
	}
	return getTelegramOperationalConnectClient().executeCommand({
		command: { case: 'removeChatFromFolder', value },
	})
}

function normalizeTarget(target: TelegramChatTarget): TelegramChatTarget {
	return {
		accountId: requireIdentifier('account ID', target.accountId),
		providerChatId: requireIdentifier('chat ID', target.providerChatId),
		operationId: requireIdentifier('operation ID', target.operationId),
	}
}

function optionalIdentifier(value?: string): string | undefined {
	const normalized = value?.trim()
	return normalized || undefined
}

function requireIdentifier(label: string, value: string): string {
	const normalized = value.trim()
	if (!normalized) {
		throw new RangeError(`${label} is required`)
	}
	return normalized
}
