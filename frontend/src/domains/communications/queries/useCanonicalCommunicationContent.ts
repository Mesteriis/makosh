import { computed, ref } from 'vue'

import {
	decodeCanonicalCommunicationContent,
	type CanonicalCommunicationContentStatus,
} from '../presentation/canonicalCommunicationContentModel'
import { readCanonicalCommunicationContent } from './canonicalCommunicationsContent'
import type { CanonicalCommunicationContent } from './canonicalCommunicationsContent'

type CanonicalCommunicationContentReader = (
	messageId: Uint8Array,
	signal?: AbortSignal,
) => Promise<CanonicalCommunicationContent>

export function useCanonicalCommunicationContent(
	readContent: CanonicalCommunicationContentReader = readCanonicalCommunicationContent,
) {
	const status = ref<CanonicalCommunicationContentStatus>('idle')
	const statusMessage = ref('')
	const bodyText = ref('')
	let generation = 0
	let activeRequest: AbortController | null = null

	const model = computed(() => ({
		status: status.value,
		statusMessage: statusMessage.value,
		bodyText: bodyText.value,
	}))

	async function open(messageId: Uint8Array): Promise<void> {
		const requestGeneration = ++generation
		activeRequest?.abort()
		activeRequest = new AbortController()
		status.value = 'loading'
		statusMessage.value = 'Loading canonical message content…'
		bodyText.value = ''
		try {
			const content = await readContent(messageId, activeRequest.signal)
			if (requestGeneration !== generation) return
			bodyText.value = decodeCanonicalCommunicationContent(content.bytes)
			status.value = 'ready'
			statusMessage.value = ''
		} catch (error) {
			if (requestGeneration !== generation || isAbort(error)) return
			status.value = 'unavailable'
			statusMessage.value = 'Canonical message content is unavailable.'
			bodyText.value = ''
		} finally {
			if (requestGeneration === generation) activeRequest = null
		}
	}

	function close(): void {
		generation += 1
		activeRequest?.abort()
		activeRequest = null
		status.value = 'idle'
		statusMessage.value = ''
		bodyText.value = ''
	}

	return { close, model, open }
}

function isAbort(error: unknown): boolean {
	return error instanceof DOMException && error.name === 'AbortError'
}
