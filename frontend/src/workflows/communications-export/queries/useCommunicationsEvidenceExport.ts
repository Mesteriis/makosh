import { computed, onUnmounted, ref, watch, type ComputedRef } from 'vue'

import { EvidenceExportStatusV1 } from '../../../gen/makosh/communications_export/v1/export_pb'
import { downloadBytesFile } from '../../../shared/file/downloadBytes'
import {
	getCommunicationsEvidenceExportStatus,
	openCommunicationsEvidenceExportRealtime,
	readCommunicationsEvidenceExport,
	startCommunicationsEvidenceExport,
	type CommunicationsEvidenceExportRealtimeBindingV1,
} from '../api/communicationsEvidenceExport'
import type {
	CommunicationsEvidenceExportPanelModel,
	CommunicationsEvidenceExportPanelStatus,
} from '../presentation/communicationsEvidenceExportPanelModel'

const CANONICAL_ID_BYTES = 16
const MAX_MESSAGES = 64

export function useCommunicationsEvidenceExport(
	canExport: () => boolean,
): {
	model: ComputedRef<CommunicationsEvidenceExportPanelModel>
	addMessage: (messageId: Uint8Array) => void
	clear: () => void
	start: () => Promise<void>
	refresh: () => Promise<void>
	download: () => Promise<void>
} {
	const selected = ref<Uint8Array[]>([])
	const exportId = ref<Uint8Array>()
	const status = ref<CommunicationsEvidenceExportPanelStatus>('idle')
	const statusMessage = ref('')
	const completedItems = ref(0)
	const requestedItems = ref(0)
	let generation = 0
	let realtimeBinding: CommunicationsEvidenceExportRealtimeBindingV1 | undefined

	const model = computed<CommunicationsEvidenceExportPanelModel>(() => ({
		available: canExport(),
		busy: ['starting', 'downloading'].includes(status.value),
		canAddCandidate: canExport() && selected.value.length < MAX_MESSAGES,
		canDownload: canExport() && status.value === 'ready',
		canRefresh: canExport() && Boolean(exportId.value) && ['pending', 'materializing'].includes(status.value),
		selectedCount: selected.value.length,
		progressLabel: requestedItems.value > 0
			? `${completedItems.value} / ${requestedItems.value}`
			: `${selected.value.length} selected`,
		status: canExport() ? status.value : 'unavailable',
		statusMessage: canExport()
			? statusMessage.value
			: 'Evidence export is not admitted for this runtime.',
	}))

	watch(
		canExport,
		(available) => {
			if (available) return
			generation += 1
			selected.value = []
			exportId.value = undefined
			status.value = 'idle'
			statusMessage.value = ''
		},
	)
	onUnmounted(() => {
		generation += 1
		realtimeBinding?.close()
	})

	function addMessage(messageId: Uint8Array): void {
		if (!canExport() || !validId(messageId) || selected.value.length >= MAX_MESSAGES) return
		const key = bytesKey(messageId)
		if (selected.value.some((candidate) => bytesKey(candidate) === key)) {
			statusMessage.value = 'The open canonical message is already selected.'
			return
		}
		selected.value = [...selected.value, new Uint8Array(messageId)]
		resetJob()
		statusMessage.value = 'Canonical message added to the evidence export.'
	}

	function clear(): void {
		generation += 1
		selected.value = []
		resetJob()
		statusMessage.value = ''
	}

	async function start(): Promise<void> {
		if (!canExport() || selected.value.length === 0 || status.value === 'starting') return
		const currentGeneration = ++generation
		const operationId = crypto.getRandomValues(new Uint8Array(CANONICAL_ID_BYTES))
		status.value = 'starting'
		statusMessage.value = 'Submitting an owner-local evidence export…'
		completedItems.value = 0
		requestedItems.value = selected.value.length
		realtimeBinding?.close()
		realtimeBinding = openCommunicationsEvidenceExportRealtime({
			onStatus: response => {
				if (currentGeneration !== generation) return
				applyStatusResponse(response)
			},
			onUnavailable: () => {
				if (currentGeneration !== generation || isTerminal(status.value)) return
				statusMessage.value = 'Live export status is unavailable. Refresh its owner-local snapshot.'
			},
		})
		try {
			await realtimeBinding.ready
			if (currentGeneration !== generation) return
			exportId.value = await startCommunicationsEvidenceExport(selected.value, operationId)
			realtimeBinding.attachExport(exportId.value)
			status.value = 'pending'
			statusMessage.value = 'Communications is preparing the canonical evidence snapshot.'
		} catch {
			if (currentGeneration !== generation) return
			realtimeBinding?.close()
			realtimeBinding = undefined
			status.value = 'error'
			statusMessage.value = 'Evidence export could not be started.'
		}
	}

	async function refresh(): Promise<void> {
		if (!exportId.value || !canExport()) return
		const currentGeneration = generation
		try {
			await applyStatus(currentGeneration)
		} catch {
			if (currentGeneration !== generation) return
			status.value = 'error'
			statusMessage.value = 'Evidence export status is temporarily unavailable.'
		}
	}

	async function download(): Promise<void> {
		if (!exportId.value || status.value !== 'ready' || !canExport()) return
		status.value = 'downloading'
		statusMessage.value = 'Issuing a one-use artifact read…'
		try {
			const bytes = await readCommunicationsEvidenceExport(exportId.value)
			downloadBytesFile(
				`makosh-communications-evidence-${bytesKey(exportId.value).slice(0, 12)}.jsonl`,
				bytes,
				'application/x-ndjson',
			)
			status.value = 'ready'
			statusMessage.value = 'Evidence artifact downloaded. A new one-use read will be issued if needed.'
		} catch {
			status.value = 'error'
			statusMessage.value = 'Evidence artifact download failed closed.'
		}
	}

	async function applyStatus(currentGeneration: number): Promise<boolean> {
		const currentExportId = exportId.value
		if (!currentExportId) return true
		const response = await getCommunicationsEvidenceExportStatus(currentExportId)
		if (currentGeneration !== generation) return true
		return applyStatusResponse(response)
	}

	function applyStatusResponse(response: {
		status: EvidenceExportStatusV1
		completedItems: number
		requestedItems: number
	}): boolean {
		completedItems.value = response.completedItems
		requestedItems.value = response.requestedItems
		switch (response.status) {
			case EvidenceExportStatusV1.EVIDENCE_EXPORT_STATUS_PENDING_SOURCE:
				status.value = 'pending'
				statusMessage.value = 'Communications is preparing the canonical evidence snapshot.'
				return false
			case EvidenceExportStatusV1.EVIDENCE_EXPORT_STATUS_MATERIALIZING:
				status.value = 'materializing'
				statusMessage.value = 'The workflow is materializing the bounded JSONL artifact.'
				return false
			case EvidenceExportStatusV1.EVIDENCE_EXPORT_STATUS_READY:
				status.value = 'ready'
				statusMessage.value = 'Evidence artifact is ready for a one-use authenticated download.'
				return true
			case EvidenceExportStatusV1.EVIDENCE_EXPORT_STATUS_REJECTED:
				status.value = 'rejected'
				statusMessage.value = 'Evidence export was rejected by the current canonical policy.'
				return true
			default:
				throw new Error('Unknown export status')
		}
	}

	function resetJob(): void {
		generation += 1
		realtimeBinding?.close()
		realtimeBinding = undefined
		exportId.value = undefined
		status.value = 'idle'
		completedItems.value = 0
		requestedItems.value = 0
	}

	return { model, addMessage, clear, start, refresh, download }
}

function validId(value: Uint8Array): boolean {
	return value.byteLength === CANONICAL_ID_BYTES && value.some((byte) => byte !== 0)
}

function bytesKey(value: Uint8Array): string {
	return Array.from(value, (byte) => byte.toString(16).padStart(2, '0')).join('')
}

function isTerminal(status: CommunicationsEvidenceExportPanelStatus): boolean {
	return ['ready', 'rejected', 'error', 'idle', 'unavailable'].includes(status)
}
