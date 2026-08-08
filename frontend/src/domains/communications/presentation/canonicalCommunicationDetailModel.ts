import type {
	AttachmentAnchorSummaryV1,
	ConversationSummaryV1,
	EvidenceSummaryV1,
	MessageReferenceSummaryV1,
	MessageSummaryV1,
	ObservedParticipantSummaryV1,
} from '../../../gen/makosh/communications/query/v1/query_pb'
import { bytesKey } from './canonicalCommunicationsPageModel'

export type CanonicalCommunicationDetailStatus = 'idle' | 'loading' | 'ready' | 'error'

export type CanonicalCommunicationDetailModel = {
	status: CanonicalCommunicationDetailStatus
	statusMessage: string
	messageLabel: string
	conversationLabel: string
	directionLabel: string
	bodyStateLabel: string
	lifecycleLabel: string
	observedRangeLabel: string
	participants: readonly CanonicalDetailRow[]
	attachments: readonly CanonicalAttachmentDetailRow[]
	references: readonly CanonicalDetailRow[]
	evidence: readonly CanonicalDetailRow[]
	hasMoreParticipants: boolean
	hasMoreAttachments: boolean
	hasMoreReferences: boolean
	hasMoreEvidence: boolean
	loadingMore: boolean
}

export type CanonicalDetailRow = {
	key: string
	primaryLabel: string
	secondaryLabel: string
	metaLabel: string
}

export type CanonicalAttachmentDetailRow = CanonicalDetailRow & {
	previewEligible: boolean
	previewLabel: 'Preview' | 'Unavailable'
}

const ATTACHMENT_SAFETY_STATE_SAFE_FOR_DELIVERY_V1 = 5

export function buildCanonicalCommunicationDetailModel(input: {
	status: CanonicalCommunicationDetailStatus
	statusMessage: string
	message?: MessageSummaryV1
	conversation?: ConversationSummaryV1
	participants: readonly ObservedParticipantSummaryV1[]
	attachments: readonly AttachmentAnchorSummaryV1[]
	references: readonly MessageReferenceSummaryV1[]
	evidence: readonly EvidenceSummaryV1[]
	hasMoreParticipants: boolean
	hasMoreAttachments: boolean
	hasMoreReferences: boolean
	hasMoreEvidence: boolean
	loadingMore: boolean
}): CanonicalCommunicationDetailModel {
	const { message, conversation } = input
	return {
		status: input.status,
		statusMessage: input.statusMessage,
		messageLabel: message ? identifierLabel('Message', message.messageId) : '',
		conversationLabel: conversation
			? identifierLabel('Conversation', conversation.conversationId)
			: '',
		directionLabel: message ? `Direction ${message.direction}` : '',
		bodyStateLabel: message ? `Body state ${message.bodyState}` : '',
		lifecycleLabel: message ? `Lifecycle ${message.lifecycleState}` : '',
		observedRangeLabel: message
			? observedRange(message.firstObservedAtUnixSeconds, message.lastObservedAtUnixSeconds)
			: '',
		participants: input.participants.map((participant) => ({
			key: bytesKey(participant.participantId),
			primaryLabel: identifierLabel('Participant', participant.participantId),
			secondaryLabel: identifierLabel('Evidence', participant.lastEvidenceId),
			metaLabel: observedRange(
				participant.firstObservedAtUnixSeconds,
				participant.lastObservedAtUnixSeconds,
			),
		})),
		attachments: input.attachments.map((attachment) => ({
			key: bytesKey(attachment.attachmentAnchorId),
			primaryLabel: attachment.hasFilename
				? attachment.filename
				: identifierLabel('Attachment', attachment.attachmentAnchorId),
			secondaryLabel: attachment.hasDescriptor
				? `${attachment.mediaType || 'Unknown media'} · ${attachment.declaredBytes} bytes`
				: 'Descriptor unavailable',
			metaLabel: `State ${attachment.state}`,
			previewEligible: attachment.state === ATTACHMENT_SAFETY_STATE_SAFE_FOR_DELIVERY_V1,
			previewLabel: attachment.state === ATTACHMENT_SAFETY_STATE_SAFE_FOR_DELIVERY_V1
				? 'Preview'
				: 'Unavailable',
		})),
		references: input.references.map((reference) => ({
			key: `${bytesKey(reference.evidenceId)}-${reference.kind}`,
			primaryLabel: `Reference kind ${reference.kind}`,
			secondaryLabel: reference.targetMessageId.byteLength > 0
				? identifierLabel('Target message', reference.targetMessageId)
				: 'Target remains unresolved',
			metaLabel: formatUnixSeconds(reference.observedAtUnixSeconds),
		})),
		evidence: input.evidence.map((evidence) => ({
			key: bytesKey(evidence.evidenceId),
			primaryLabel: identifierLabel('Evidence', evidence.evidenceId),
			secondaryLabel: `Source ${evidence.provider} · kind ${evidence.kind} · direction ${evidence.direction}`,
			metaLabel: formatUnixSeconds(evidence.observedAtUnixSeconds),
		})),
		hasMoreParticipants: input.hasMoreParticipants,
		hasMoreAttachments: input.hasMoreAttachments,
		hasMoreReferences: input.hasMoreReferences,
		hasMoreEvidence: input.hasMoreEvidence,
		loadingMore: input.loadingMore,
	}
}

function identifierLabel(kind: string, value: Uint8Array): string {
	const key = bytesKey(value)
	return `${kind} ${key ? `#${key.slice(0, 12)}` : 'unavailable'}`
}

function observedRange(first: bigint, last: bigint): string {
	return first === last
		? formatUnixSeconds(last)
		: `${formatUnixSeconds(first)} — ${formatUnixSeconds(last)}`
}

function formatUnixSeconds(value: bigint): string {
	const milliseconds = Number(value) * 1_000
	if (!Number.isSafeInteger(milliseconds)) return 'Time unavailable'
	const date = new Date(milliseconds)
	if (Number.isNaN(date.getTime())) return 'Time unavailable'
	return new Intl.DateTimeFormat(undefined, {
		dateStyle: 'medium',
		timeStyle: 'short',
	}).format(date)
}
