import type {
	TelegramAttachmentProjection,
	TelegramCommandRecordProjection,
	TelegramFileSnapshotProjection,
	TelegramMessageMutationProjection,
	TelegramMessageProjection,
	TelegramMessageReferencesProjection,
	TelegramMessageTombstoneProjection,
	TelegramMessageVersionProjection,
	TelegramReactionObservationProjection,
	TelegramReactionSummaryProjection,
} from '../../../gen/makosh/telegram/v1/client_pb'
import { getTelegramOperationalConnectClient } from './telegramOperationalClient'

const PAGE_LIMIT = 100

export type TelegramMessageInspection = {
	message?: TelegramMessageProjection
	versions: readonly TelegramMessageVersionProjection[]
	tombstones: readonly TelegramMessageTombstoneProjection[]
	mutations: readonly TelegramMessageMutationProjection[]
	references?: TelegramMessageReferencesProjection
	replyChain: readonly TelegramMessageProjection[]
	forwardChain: readonly TelegramMessageProjection[]
	attachment?: TelegramAttachmentProjection
	file?: TelegramFileSnapshotProjection
	reactions: readonly TelegramReactionObservationProjection[]
	reactionSummary: readonly TelegramReactionSummaryProjection[]
	commands: readonly TelegramCommandRecordProjection[]
	pinned: boolean
}

export async function inspectTelegramMessage(input: {
	accountId: string
	providerChatId: string
	messageId: string
	providerMessageId: string
}): Promise<TelegramMessageInspection> {
	const accountId = requireIdentifier('account ID', input.accountId)
	const providerChatId = requireIdentifier('chat ID', input.providerChatId)
	const messageId = requireIdentifier('message ID', input.messageId)
	const providerMessageId = requireIdentifier('provider message ID', input.providerMessageId)
	const client = getTelegramOperationalConnectClient()
	const [
		message,
		versions,
		tombstones,
		mutations,
		references,
		replyChain,
		forwardChain,
		attachment,
		reactions,
		reactionSummary,
		commands,
		pinnedMessages,
	] = await Promise.all([
		client.executeQuery({
			query: { case: 'messageById', value: { accountId, messageId } },
		}),
		client.executeQuery({
			query: { case: 'messageVersions', value: { accountId, messageId } },
		}),
		client.executeQuery({
			query: { case: 'messageTombstones', value: { accountId, messageId } },
		}),
		client.executeQuery({
			query: { case: 'messageMutations', value: { accountId, messageId } },
		}),
		client.executeQuery({
			query: { case: 'messageReferences', value: { accountId, messageId } },
		}),
		client.executeQuery({
			query: {
				case: 'replyChain',
				value: { accountId, providerChatId, providerMessageId, limit: PAGE_LIMIT },
			},
		}),
		client.executeQuery({
			query: {
				case: 'forwardChain',
				value: { accountId, providerChatId, providerMessageId, limit: PAGE_LIMIT },
			},
		}),
		client.executeQuery({
			query: {
				case: 'attachmentForMessage',
				value: { accountId, providerChatId, providerMessageId },
			},
		}),
		client.executeQuery({
			query: {
				case: 'reactions',
				value: { accountId, providerChatId, providerMessageId },
			},
		}),
		client.executeQuery({
			query: {
				case: 'reactionSummary',
				value: { accountId, providerChatId, providerMessageId },
			},
		}),
		client.executeQuery({
			query: {
				case: 'commands',
				value: {
					accountId,
					providerChatId,
					providerMessageId,
					commandKind: [],
					limit: PAGE_LIMIT,
				},
			},
		}),
		client.executeQuery({
			query: {
				case: 'pinnedMessages',
				value: { accountId, providerChatId, limit: PAGE_LIMIT },
			},
		}),
	])
	if (
		message.response.case !== 'cachedMessages'
		|| versions.response.case !== 'messageVersions'
		|| tombstones.response.case !== 'messageTombstones'
		|| mutations.response.case !== 'messageMutations'
		|| references.response.case !== 'messageReferences'
		|| replyChain.response.case !== 'replyChain'
		|| forwardChain.response.case !== 'forwardChain'
		|| attachment.response.case !== 'attachment'
		|| reactions.response.case !== 'reactions'
		|| reactionSummary.response.case !== 'reactionSummary'
		|| commands.response.case !== 'commands'
		|| pinnedMessages.response.case !== 'cachedMessages'
	) {
		throw new Error('Telegram message inspection is incomplete')
	}
	const selectedAttachment = attachment.response.value.attachment
	return {
		message: message.response.value.item[0],
		versions: versions.response.value.item,
		tombstones: tombstones.response.value.item,
		mutations: mutations.response.value.item,
		references: references.response.value.references,
		replyChain: replyChain.response.value.item,
		forwardChain: forwardChain.response.value.item,
		attachment: selectedAttachment,
		file: selectedAttachment?.providerFileId
			? await loadTelegramFile(accountId, selectedAttachment.providerFileId)
			: undefined,
		reactions: reactions.response.value.reaction,
		reactionSummary: reactionSummary.response.value.summary,
		commands: commands.response.value.record,
		pinned: pinnedMessages.response.value.item.some(
			(item) => item.providerMessageId === providerMessageId,
		),
	}
}

async function loadTelegramFile(
	accountId: string,
	providerFileId: string,
): Promise<TelegramFileSnapshotProjection | undefined> {
	const response = await getTelegramOperationalConnectClient().executeQuery({
		query: { case: 'file', value: { accountId, providerFileId } },
	})
	if (response.response.case !== 'file') {
		throw new Error('Telegram file projection is unavailable')
	}
	return response.response.value.file
}

function requireIdentifier(label: string, value: string): string {
	const normalized = value.trim()
	if (!normalized) {
		throw new RangeError(`${label} is required`)
	}
	return normalized
}
