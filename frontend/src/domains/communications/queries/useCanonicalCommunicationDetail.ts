import { computed, ref, shallowRef } from 'vue'

import type {
	AttachmentAnchorSummaryV1,
	ConversationSummaryV1,
	EvidenceSummaryV1,
	MessageReferenceSummaryV1,
	MessageSummaryV1,
	ObservedParticipantSummaryV1,
} from '../../../gen/makosh/communications/query/v1/query_pb'
import {
	buildCanonicalCommunicationDetailModel,
	type CanonicalCommunicationDetailStatus,
} from '../presentation/canonicalCommunicationDetailModel'
import { bytesKey } from '../presentation/canonicalCommunicationsPageModel'
import { loadCanonicalCommunicationDetail } from './canonicalCommunicationsDetail'
import {
	listCanonicalConversationParticipants,
	listCanonicalMessageAttachmentAnchors,
	listCanonicalMessageEvidence,
	listCanonicalMessageReferences,
} from './canonicalCommunicationsRead'

export function useCanonicalCommunicationDetail() {
	const status = ref<CanonicalCommunicationDetailStatus>('idle')
	const statusMessage = ref('')
	const message = ref<MessageSummaryV1>()
	const conversation = ref<ConversationSummaryV1>()
	const participants = ref<readonly ObservedParticipantSummaryV1[]>([])
	const attachments = ref<readonly AttachmentAnchorSummaryV1[]>([])
	const references = ref<readonly MessageReferenceSummaryV1[]>([])
	const evidence = ref<readonly EvidenceSummaryV1[]>([])
	const participantCursor = shallowRef<Uint8Array>(new Uint8Array())
	const attachmentCursor = shallowRef<Uint8Array>(new Uint8Array())
	const referenceCursor = shallowRef<Uint8Array>(new Uint8Array())
	const evidenceCursor = shallowRef<Uint8Array>(new Uint8Array())
	const loadingMore = ref(false)
	let generation = 0

	const model = computed(() => buildCanonicalCommunicationDetailModel({
		status: status.value,
		statusMessage: statusMessage.value,
		message: message.value,
		conversation: conversation.value,
		participants: participants.value,
		attachments: attachments.value,
		references: references.value,
		evidence: evidence.value,
		hasMoreParticipants: participantCursor.value.byteLength > 0,
		hasMoreAttachments: attachmentCursor.value.byteLength > 0,
		hasMoreReferences: referenceCursor.value.byteLength > 0,
		hasMoreEvidence: evidenceCursor.value.byteLength > 0,
		loadingMore: loadingMore.value,
	}))

	async function open(messageId: Uint8Array): Promise<void> {
		const requestGeneration = ++generation
		resetRows()
		status.value = 'loading'
		statusMessage.value = 'Loading exact canonical message detail…'
		try {
			const detail = await loadCanonicalCommunicationDetail(messageId)
			if (requestGeneration !== generation) return
			message.value = detail.message
			conversation.value = detail.conversation
			participants.value = detail.participants.items
			participantCursor.value = detail.participants.nextCursor
			attachments.value = detail.attachments.items
			attachmentCursor.value = detail.attachments.nextCursor
			references.value = detail.references.items
			referenceCursor.value = detail.references.nextCursor
			evidence.value = detail.evidence.items
			evidenceCursor.value = detail.evidence.nextCursor
			status.value = 'ready'
			statusMessage.value = ''
		} catch {
			if (requestGeneration !== generation) return
			status.value = 'error'
			statusMessage.value = 'Canonical message detail is temporarily unavailable.'
		}
	}

	function close(): void {
		generation += 1
		status.value = 'idle'
		statusMessage.value = ''
		resetRows()
	}

	function attachmentAnchorIdForKey(attachmentKey: string): Uint8Array | undefined {
		const attachment = attachments.value.find(
			(candidate) => bytesKey(candidate.attachmentAnchorId) === attachmentKey,
		)
		return attachment ? new Uint8Array(attachment.attachmentAnchorId) : undefined
	}

	async function loadMoreParticipants(): Promise<void> {
		const current = message.value
		if (!current || participantCursor.value.byteLength === 0) return
		await append(async () => {
			const page = await listCanonicalConversationParticipants(
				current.conversationId,
				100,
				participantCursor.value,
			)
			participants.value = appendUnique(
				participants.value,
				page.items,
				(item) => bytesKey(item.participantId),
			)
			participantCursor.value = page.nextCursor
		})
	}

	async function loadMoreAttachments(): Promise<void> {
		const current = message.value
		if (!current || attachmentCursor.value.byteLength === 0) return
		await append(async () => {
			const page = await listCanonicalMessageAttachmentAnchors(
				current.messageId,
				100,
				attachmentCursor.value,
			)
			attachments.value = appendUnique(
				attachments.value,
				page.items,
				(item) => bytesKey(item.attachmentAnchorId),
			)
			attachmentCursor.value = page.nextCursor
		})
	}

	async function loadMoreReferences(): Promise<void> {
		const current = message.value
		if (!current || referenceCursor.value.byteLength === 0) return
		await append(async () => {
			const page = await listCanonicalMessageReferences(
				current.messageId,
				100,
				referenceCursor.value,
			)
			references.value = appendUnique(
				references.value,
				page.items,
				(item) => `${bytesKey(item.evidenceId)}-${item.kind}`,
			)
			referenceCursor.value = page.nextCursor
		})
	}

	async function loadMoreEvidence(): Promise<void> {
		const current = message.value
		if (!current || evidenceCursor.value.byteLength === 0) return
		await append(async () => {
			const page = await listCanonicalMessageEvidence(
				current.messageId,
				100,
				evidenceCursor.value,
			)
			evidence.value = appendUnique(
				evidence.value,
				page.items,
				(item) => bytesKey(item.evidenceId),
			)
			evidenceCursor.value = page.nextCursor
		})
	}

	async function append(loadPage: () => Promise<void>): Promise<void> {
		if (loadingMore.value) return
		const requestGeneration = generation
		loadingMore.value = true
		statusMessage.value = ''
		try {
			await loadPage()
		} catch {
			if (requestGeneration === generation) {
				statusMessage.value = 'The next canonical detail page is temporarily unavailable.'
			}
		} finally {
			if (requestGeneration === generation) loadingMore.value = false
		}
	}

	function resetRows(): void {
		message.value = undefined
		conversation.value = undefined
		participants.value = []
		attachments.value = []
		references.value = []
		evidence.value = []
		participantCursor.value = new Uint8Array()
		attachmentCursor.value = new Uint8Array()
		referenceCursor.value = new Uint8Array()
		evidenceCursor.value = new Uint8Array()
		loadingMore.value = false
	}

	return {
		attachmentAnchorIdForKey,
		close,
		loadMoreAttachments,
		loadMoreEvidence,
		loadMoreParticipants,
		loadMoreReferences,
		model,
		open,
	}
}

function appendUnique<T>(
	current: readonly T[],
	next: readonly T[],
	key: (item: T) => string,
): readonly T[] {
	const keys = new Set(current.map(key))
	return [...current, ...next.filter((item) => !keys.has(key(item)))]
}
