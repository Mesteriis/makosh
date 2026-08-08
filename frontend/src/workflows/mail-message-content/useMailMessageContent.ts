import { computed, ref } from 'vue'

import { readCanonicalCommunicationContent } from '../../domains/communications/queries/canonicalCommunicationsContent'
import type { CanonicalCommunicationContent } from '../../domains/communications/queries/canonicalCommunicationsContent'
import { resolveCanonicalMessageIdForEvidence } from '../../domains/communications/queries/canonicalCommunicationsRead'
import { decodeCanonicalCommunicationContent } from '../../domains/communications/presentation/canonicalCommunicationContentModel'

export type MailMessageContentStatus = 'idle' | 'loading' | 'ready' | 'unavailable'

type MailMessageContentPorts = {
	resolveMessageId(evidenceId: Uint8Array): Promise<Uint8Array>
	readContent(messageId: Uint8Array, signal?: AbortSignal): Promise<CanonicalCommunicationContent>
}

export function useMailMessageContent(input: {
	canRead: () => boolean
}, ports: MailMessageContentPorts = {
	resolveMessageId: resolveCanonicalMessageIdForEvidence,
	readContent: readCanonicalCommunicationContent,
}) {
	const status = ref<MailMessageContentStatus>('idle')
	const statusMessage = ref('')
	const bodyText = ref('')
	const bodyFormat = ref<'text' | 'html'>('text')
	let generation = 0
	let activeRequest: AbortController | null = null

	const model = computed(() => ({
		status: status.value,
		statusMessage: statusMessage.value,
		bodyText: bodyText.value,
		bodyFormat: bodyFormat.value,
	}))

	async function open(evidenceId: Uint8Array | undefined): Promise<void> {
		const requestGeneration = ++generation
		activeRequest?.abort()
		activeRequest = null
		bodyText.value = ''
		bodyFormat.value = 'text'
		if (!evidenceId || evidenceId.byteLength !== 16 || !input.canRead()) {
			status.value = evidenceId ? 'unavailable' : 'idle'
			statusMessage.value = evidenceId
				? 'Canonical message content capability is unavailable.'
				: ''
			return
		}
		activeRequest = new AbortController()
		status.value = 'loading'
		statusMessage.value = 'Loading message body…'
		try {
			const messageId = await ports.resolveMessageId(evidenceId)
			const content = await ports.readContent(
				messageId,
				activeRequest.signal,
			)
			if (requestGeneration !== generation) return
			bodyText.value = decodeCanonicalCommunicationContent(content.bytes)
			bodyFormat.value = content.mediaType === 'text/html' ? 'html' : 'text'
			status.value = 'ready'
			statusMessage.value = ''
		} catch (error) {
			if (requestGeneration !== generation || isAbort(error)) return
			status.value = 'unavailable'
			statusMessage.value = 'Message body is unavailable.'
			bodyText.value = ''
		} finally {
			if (requestGeneration === generation) activeRequest = null
		}
	}

	return { model, open }
}

function isAbort(error: unknown): boolean {
	return error instanceof DOMException && error.name === 'AbortError'
}
