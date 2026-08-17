const HISTORY_VIEWPORT_FILL_TOLERANCE_PX = 1

export function shouldPrefetchTelegramHistory(
	hasOlderMessages: boolean,
	scrollHeight: number,
	clientHeight: number,
): boolean {
	return hasOlderMessages
		&& scrollHeight <= clientHeight + HISTORY_VIEWPORT_FILL_TOLERANCE_PX
}

export function initialTelegramHistoryScrollTop(
	messageCount: number,
	scrollHeight: number,
	savedScrollTop?: number,
): number | undefined {
	if (messageCount <= 0) return undefined
	return savedScrollTop ?? scrollHeight
}
