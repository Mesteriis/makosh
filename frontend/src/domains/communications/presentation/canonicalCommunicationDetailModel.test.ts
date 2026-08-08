import { describe, expect, it } from 'vitest'

import { buildCanonicalCommunicationDetailModel } from './canonicalCommunicationDetailModel'

describe('canonical Communications detail presentation model', () => {
	it('maps metadata-only detail without provider locators or message content', () => {
		const messageId = new Uint8Array(16).fill(1)
		const conversationId = new Uint8Array(16).fill(2)
		const model = buildCanonicalCommunicationDetailModel({
			status: 'ready',
			statusMessage: '',
			message: {
				$typeName: 'makosh.communications.query.v1.MessageSummaryV1',
				messageId,
				conversationId,
				sourceCursorSha256: new Uint8Array(32),
				bodyState: 2,
				lifecycleState: 1,
				firstObservedAtUnixSeconds: 10n,
				lastObservedAtUnixSeconds: 10n,
				lastEvidenceId: new Uint8Array(16),
				direction: 1,
			},
			conversation: {
				$typeName: 'makosh.communications.query.v1.ConversationSummaryV1',
				conversationId,
				accountCursorSha256: new Uint8Array(32),
				conversationCursorSha256: new Uint8Array(32),
				provider: 2,
				firstObservedAtUnixSeconds: 10n,
				lastObservedAtUnixSeconds: 10n,
				lastEvidenceId: new Uint8Array(16),
			},
			participants: [],
			attachments: [],
			references: [],
			evidence: [],
			hasMoreParticipants: false,
			hasMoreAttachments: false,
			hasMoreReferences: false,
			hasMoreEvidence: false,
			loadingMore: false,
		})

		expect(model.messageLabel).toContain('Message #010101010101')
		expect(model.conversationLabel).toContain('Conversation #020202020202')
		expect(model.bodyStateLabel).toBe('Body state 2')
		expect(model).not.toHaveProperty('body')
		expect(model).not.toHaveProperty('providerLocator')
	})

	it('admits preview only for safe-for-delivery attachment evidence', () => {
		const attachment = (state: number) => ({
			$typeName: 'makosh.communications.query.v1.AttachmentAnchorSummaryV1' as const,
			attachmentAnchorId: new Uint8Array(16).fill(state),
			messageId: new Uint8Array(16),
			mediaCursorSha256: new Uint8Array(32),
			state,
			firstObservedAtUnixSeconds: 10n,
			lastObservedAtUnixSeconds: 10n,
			lastEvidenceId: new Uint8Array(16),
			hasDescriptor: true,
			filename: 'attachment.bin',
			hasFilename: true,
			mediaType: 'application/octet-stream',
			declaredBytes: 1n,
			sha256: new Uint8Array(32),
			disposition: 1,
		})
		const model = buildCanonicalCommunicationDetailModel({
			status: 'ready',
			statusMessage: '',
			participants: [],
			attachments: [attachment(5), attachment(6)],
			references: [],
			evidence: [],
			hasMoreParticipants: false,
			hasMoreAttachments: false,
			hasMoreReferences: false,
			hasMoreEvidence: false,
			loadingMore: false,
		})

		expect(model.attachments.map(({ previewEligible, previewLabel }) => ({
			previewEligible,
			previewLabel,
		}))).toEqual([
			{ previewEligible: true, previewLabel: 'Preview' },
			{ previewEligible: false, previewLabel: 'Unavailable' },
		])
	})
})
