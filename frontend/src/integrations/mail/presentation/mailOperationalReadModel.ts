import {
	MailFolderKindV1,
	MailMessageFlagV1,
	type MailFolderV1,
	type MailMessageSummaryV1,
	type MailThreadV1,
} from '../../../gen/hermes/mail/operational/v1/client_pb'
import type {
	MailAccountConnection as MailOperationalConnection,
} from '../queries/mailAccountConnections'

export type MailOperationalReadStatus = 'blocked' | 'empty' | 'error' | 'loading' | 'ready'

export type MailOperationalReadModel = {
	canQuery: boolean
	status: MailOperationalReadStatus
	statusMessage: string
	connections: readonly MailOperationalConnectionOption[]
	selectedConnectionId: string
	folders: readonly MailFolderRow[]
	threads: readonly MailThreadRow[]
	messages: readonly MailMessageRow[]
	detail: MailMessageDetailCard | null
	hasMoreFolders: boolean
	hasMoreThreads: boolean
	hasMoreMessages: boolean
}

export type MailOperationalConnectionOption = {
	id: string
	label: string
}

export type MailFolderRow = {
	id: string
	label: string
	meta: string
	selected: boolean
}

export type MailThreadRow = {
	id: string
	subject: string
	snippet: string
	meta: string
	selected: boolean
	unread: boolean
}

export type MailMessageRow = {
	id: string
	subject: string
	sender: string
	snippet: string
	meta: string
	selected: boolean
	unread: boolean
	hasAttachments: boolean
}

export type MailMessageDetailCard = {
	id: string
	subject: string
	sender: string
	recipients: string
	snippet: string
	meta: string
	folders: string
	flags: string
	evidenceState: string
	contentState: string
	observationAnchorId: Uint8Array
	isRead: boolean
	isStarred: boolean
	isTrashed: boolean
	projectionRevision: string
}

export function buildMailConnectionOptions(
	connections: readonly MailOperationalConnection[],
): readonly MailOperationalConnectionOption[] {
	return connections.map((connection) => ({
		id: connection.connectionId,
		label: connection.connectionId,
	}))
}

export function buildMailFolderRows(
	folders: readonly MailFolderV1[],
	selectedFolderId: string,
): readonly MailFolderRow[] {
	return folders.map((folder) => ({
		id: folder.folderId,
		label: folder.displayName || folderKindLabel(folder.kind),
		meta: `${folder.unreadMessages} unread · ${folder.totalMessages} total`,
		selected: folder.folderId === selectedFolderId,
	}))
}

export function buildMailThreadRows(
	threads: readonly MailThreadV1[],
	selectedThreadId: string,
): readonly MailThreadRow[] {
	return threads.map((thread) => ({
		id: thread.providerThreadId,
		subject: thread.subject || '(No subject)',
		snippet: thread.latestSnippet || 'No operational snippet.',
		meta: `${formatUnixSeconds(thread.latestAtUnixSeconds)} · ${thread.messageCount} messages`,
		selected: thread.providerThreadId === selectedThreadId,
		unread: thread.unreadCount > 0n,
	}))
}

export function buildMailMessageRows(
	messages: readonly MailMessageSummaryV1[],
	selectedMessageId: string,
): readonly MailMessageRow[] {
	return messages.map((message) => ({
		id: message.messageId,
		subject: message.subject || '(No subject)',
		sender: message.sender || 'Unknown sender',
		snippet: message.snippet || 'No operational snippet.',
		meta: formatUnixSeconds(message.sentAtUnixSeconds),
		selected: message.messageId === selectedMessageId,
		unread: !message.flag.includes(MailMessageFlagV1.MAIL_MESSAGE_FLAG_READ),
		hasAttachments: message.hasAttachments,
	}))
}

export function buildMailMessageDetail(
	message: MailMessageSummaryV1 | undefined,
): MailMessageDetailCard | null {
	if (!message) return null
	return {
		id: message.messageId,
		subject: message.subject || '(No subject)',
		sender: message.sender || 'Unknown sender',
		recipients: message.recipient.join(', ') || 'No recipient projection.',
		snippet: message.snippet || 'No operational snippet.',
		meta: `${formatUnixSeconds(message.sentAtUnixSeconds)} · revision ${message.projectionRevision}`,
		folders: message.folderId.join(', ') || 'No projected folder.',
		flags: message.flag.map(messageFlagLabel).filter(Boolean).join(', ') || 'No provider flags.',
		evidenceState: message.observationAnchorId.length === 16
			? 'Canonical evidence linked'
			: 'Canonical evidence unavailable',
		contentState: message.hasPlainText
			? 'Authorized body content is Communications-owned and is not part of this Mail projection.'
			: 'No plain-text content was observed.',
		observationAnchorId: new Uint8Array(message.observationAnchorId),
		isRead: message.flag.includes(MailMessageFlagV1.MAIL_MESSAGE_FLAG_READ),
		isStarred: message.flag.includes(MailMessageFlagV1.MAIL_MESSAGE_FLAG_STARRED),
		isTrashed: message.flag.includes(MailMessageFlagV1.MAIL_MESSAGE_FLAG_TRASHED),
		projectionRevision: (message.projectionRevision ?? 0n).toString(),
	}
}

export function filterMailMessageRows(
	messages: readonly MailMessageRow[],
	searchQuery: string,
): readonly MailMessageRow[] {
	const query = searchQuery.trim().toLocaleLowerCase()
	if (!query) return messages
	return messages.filter((message) =>
		[message.sender, message.subject, message.snippet].some((value) =>
			value.toLocaleLowerCase().includes(query),
		),
	)
}

function folderKindLabel(kind: MailFolderKindV1): string {
	if (kind === MailFolderKindV1.MAIL_FOLDER_KIND_INBOX) return 'Inbox'
	if (kind === MailFolderKindV1.MAIL_FOLDER_KIND_SENT) return 'Sent'
	if (kind === MailFolderKindV1.MAIL_FOLDER_KIND_DRAFTS) return 'Drafts'
	if (kind === MailFolderKindV1.MAIL_FOLDER_KIND_TRASH) return 'Trash'
	if (kind === MailFolderKindV1.MAIL_FOLDER_KIND_SPAM) return 'Spam'
	if (kind === MailFolderKindV1.MAIL_FOLDER_KIND_ARCHIVE) return 'Archive'
	return 'Provider folder'
}

function messageFlagLabel(flag: MailMessageFlagV1): string {
	if (flag === MailMessageFlagV1.MAIL_MESSAGE_FLAG_READ) return 'Read'
	if (flag === MailMessageFlagV1.MAIL_MESSAGE_FLAG_STARRED) return 'Starred'
	if (flag === MailMessageFlagV1.MAIL_MESSAGE_FLAG_DRAFT) return 'Draft'
	if (flag === MailMessageFlagV1.MAIL_MESSAGE_FLAG_SENT) return 'Sent'
	if (flag === MailMessageFlagV1.MAIL_MESSAGE_FLAG_TRASHED) return 'Trashed'
	if (flag === MailMessageFlagV1.MAIL_MESSAGE_FLAG_SPAM) return 'Spam'
	return ''
}

function formatUnixSeconds(value: bigint | undefined): string {
	if (value === undefined) return 'Unknown time'
	const milliseconds = Number(value) * 1_000
	if (!Number.isSafeInteger(milliseconds) || milliseconds <= 0) return 'Unknown time'
	return new Intl.DateTimeFormat('en', {
		dateStyle: 'medium',
		timeStyle: 'short',
	}).format(new Date(milliseconds))
}
