const MAX_FALLBACK_PREVIEW_TIME_SECONDS = 0.25
const MIN_FALLBACK_PREVIEW_TIME_SECONDS = 0.01

export function inlineVideoPreviewDataUrl(bytes: Uint8Array): string {
	if (bytes.byteLength === 0) return ''
	let binary = ''
	for (const byte of bytes) binary += String.fromCharCode(byte)
	return `data:image/jpeg;base64,${globalThis.btoa(binary)}`
}

export function fallbackVideoPreviewTime(durationSeconds: number): number {
	if (!Number.isFinite(durationSeconds) || durationSeconds <= 0) return 0
	return Math.min(
		MAX_FALLBACK_PREVIEW_TIME_SECONDS,
		Math.max(MIN_FALLBACK_PREVIEW_TIME_SECONDS, durationSeconds / 20),
	)
}

export function shouldRewindFallbackPreview(
	currentTimeSeconds: number,
	previewTimeSeconds: number,
): boolean {
	return previewTimeSeconds > 0
		&& Number.isFinite(currentTimeSeconds)
		&& Math.abs(currentTimeSeconds - previewTimeSeconds) <= 0.1
}
