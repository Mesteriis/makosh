import { beforeEach, describe, expect, it, vi } from 'vitest'

import { MailCompositionModeV1 } from '../../../gen/makosh/mail/composition/v1/client_pb'
import { getMailCompositionCommandConnectClient } from './mailCompositionCommandClient'
import { getMailCompositionQueryConnectClient } from './mailCompositionQueryClient'
import {
	deleteMailDraft,
	listMailDrafts,
	previewMailTemplate,
	upsertMailDraft,
	upsertMailSignature,
	upsertMailTemplate,
} from './mailCompositionGateway'

vi.mock('./mailCompositionCommandClient', () => ({
	getMailCompositionCommandConnectClient: vi.fn(),
}))
vi.mock('./mailCompositionQueryClient', () => ({
	getMailCompositionQueryConnectClient: vi.fn(),
}))

const mutate = vi.fn()
const query = vi.fn()

describe('Mail composition Gateway adapter', () => {
	beforeEach(() => {
		mutate.mockReset()
		query.mockReset()
		vi.mocked(getMailCompositionCommandConnectClient).mockReturnValue({ mutate } as never)
		vi.mocked(getMailCompositionQueryConnectClient).mockReturnValue({ query } as never)
	})

	it('maps a complete draft to the exact generated command contract', async () => {
		mutate.mockResolvedValue({ entityId: 'draft-1', revision: 1n })
		await upsertMailDraft({
			connectionId: ' primary ',
			draftId: ' draft-1 ',
			mode: MailCompositionModeV1.MAIL_COMPOSITION_MODE_REPLY,
			providerConversationId: ' thread-1 ',
			inReplyToProviderMessageId: ' message-1 ',
			toRecipients: [' owner@example.test '],
			ccRecipients: [' cc@example.test '],
			bccRecipients: [' private@example.test '],
			subject: 'Subject',
			textBody: 'Body',
			templateId: ' template-1 ',
			signatureId: ' signature-1 ',
		})
		expect(mutate).toHaveBeenCalledTimes(1)
		expect(mutate.mock.calls[0]?.[0]).toMatchObject({
			command: {
				case: 'upsertDraft',
				value: {
					draft: {
						connectionId: 'primary',
						draftId: 'draft-1',
						toRecipient: ['owner@example.test'],
						ccRecipient: ['cc@example.test'],
						bccRecipient: ['private@example.test'],
						templateId: 'template-1',
						signatureId: 'signature-1',
					},
				},
			},
		})
	})

	it('keeps optimistic revisions on updates and deletes', async () => {
		mutate.mockResolvedValue({ entityId: 'draft-1', revision: 4n })
		await upsertMailDraft({
			connectionId: 'primary',
			draftId: 'draft-1',
			expectedRevision: 3n,
			mode: MailCompositionModeV1.MAIL_COMPOSITION_MODE_NEW,
			toRecipients: ['owner@example.test'],
			ccRecipients: [],
			bccRecipients: [],
			subject: '',
			textBody: 'Body',
		})
		await deleteMailDraft('primary', 'draft-1', 4n)
		expect(mutate.mock.calls[0]?.[0]).toMatchObject({
			command: { value: { expectedRevision: 3n } },
		})
		expect(mutate.mock.calls[1]?.[0]).toMatchObject({
			command: { case: 'deleteDraft', value: { expectedRevision: 4n } },
		})
	})

	it('queries scoped pages and previews typed template values', async () => {
		query
			.mockResolvedValueOnce({
				response: { case: 'drafts', value: { item: [], nextCursor: 'next' } },
			})
			.mockResolvedValueOnce({
				response: {
					case: 'templatePreview',
					value: { templateId: 'template-1', ready: true },
				},
			})
		await expect(listMailDrafts('primary')).resolves.toMatchObject({ nextCursor: 'next' })
		await expect(previewMailTemplate({
			connectionId: 'primary',
			templateId: 'template-1',
			values: { owner: 'AVM' },
		})).resolves.toMatchObject({ ready: true })
		expect(query.mock.calls[1]?.[0]).toMatchObject({
			query: {
				case: 'previewTemplate',
				value: { value: [{ name: 'owner', value: 'AVM' }] },
			},
		})
	})

	it('keeps templates and signatures in separate commands', async () => {
		mutate.mockResolvedValue({ revision: 1n })
		await upsertMailTemplate({
			connectionId: 'primary',
			templateId: 'template-1',
			name: 'Welcome',
			subjectTemplate: 'Hello {{owner}}',
			textBodyTemplate: 'Hi {{owner}}',
			variables: ['owner'],
		})
		await upsertMailSignature({
			connectionId: 'primary',
			signatureId: 'signature-1',
			name: 'Personal',
			textBody: 'Regards',
			isDefault: true,
		})
		expect(mutate.mock.calls[0]?.[0].command.case).toBe('upsertTemplate')
		expect(mutate.mock.calls[1]?.[0].command.case).toBe('upsertSignature')
	})
})
