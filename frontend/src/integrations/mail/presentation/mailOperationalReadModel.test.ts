import { create } from '@bufbuild/protobuf'
import { describe, expect, it } from 'vitest'

import {
	MailFolderKindV1,
	MailFolderV1Schema,
	MailMessageFlagV1,
	MailMessageSummaryV1Schema,
	MailThreadV1Schema,
} from '../../../gen/makosh/mail/operational/v1/client_pb'
import {
	buildMailFolderRows,
	buildMailMessageDetail,
	buildMailMessageRows,
	buildMailThreadRows,
	filterMailMessageRows,
} from './mailOperationalReadModel'

describe('Mail operational read presentation model', () => {
	it('maps folder, thread, and message projections without exposing opaque anchor bytes', () => {
		const folders = buildMailFolderRows([create(MailFolderV1Schema, {
			folderId: 'inbox',
			displayName: '',
			kind: MailFolderKindV1.MAIL_FOLDER_KIND_INBOX,
			totalMessages: 8n,
			unreadMessages: 2n,
		})], 'inbox')
		const threads = buildMailThreadRows([create(MailThreadV1Schema, {
			providerThreadId: 'thread-1',
			subject: 'Clean room',
			latestSnippet: 'Bounded evidence',
			messageCount: 2n,
			unreadCount: 1n,
		})], 'thread-1')
		const message = create(MailMessageSummaryV1Schema, {
			messageId: 'message-1',
			providerThreadId: 'thread-1',
			folderId: ['inbox'],
			subject: 'Clean room',
			sender: 'owner@example.test',
			recipient: ['team@example.test'],
			snippet: 'Bounded evidence',
			flag: [MailMessageFlagV1.MAIL_MESSAGE_FLAG_STARRED],
			hasPlainText: true,
			hasAttachments: true,
			observationAnchorId: new Uint8Array(16).fill(7),
			projectionRevision: 4n,
		})
		const messages = buildMailMessageRows([message], 'message-1')
		const detail = buildMailMessageDetail(message)

		expect(folders[0]).toMatchObject({ label: 'Inbox', selected: true })
		expect(threads[0]).toMatchObject({ subject: 'Clean room', unread: true })
		expect(messages[0]).toMatchObject({
			sender: 'owner@example.test',
			unread: true,
			hasAttachments: true,
		})
		expect(detail).toMatchObject({
			evidenceState: 'Canonical evidence linked',
			flags: 'Starred',
		})
		expect(JSON.stringify(detail)).not.toContain('7,7,7')
		expect(detail?.contentState).toContain('Communications-owned')
	})

	it('filters rendered rows through the presentation model', () => {
		const messages = buildMailMessageRows([
			create(MailMessageSummaryV1Schema, {
				messageId: 'message-1',
				subject: 'Quarterly report',
				sender: 'finance@example.test',
				snippet: 'Attached is the report.',
			}),
			create(MailMessageSummaryV1Schema, {
				messageId: 'message-2',
				subject: 'Lunch',
				sender: 'friend@example.test',
				snippet: 'Tomorrow at noon?',
			}),
		], '')

		expect(filterMailMessageRows(messages, 'report')).toHaveLength(1)
		expect(filterMailMessageRows(messages, 'FINANCE')).toHaveLength(1)
		expect(filterMailMessageRows(messages, '  ')).toBe(messages)
	})
})
