export type TelegramMediaVisibilityGate = {
	setVisible: (visible: boolean) => void
	openNow: () => void
	stop: () => void
}

export function createTelegramMediaVisibilityGate(
	open: () => void,
	delayMillis: number,
): TelegramMediaVisibilityGate {
	const normalizedDelay = Number.isFinite(delayMillis) ? Math.max(0, delayMillis) : 0
	let opened = false
	let timer: ReturnType<typeof setTimeout> | undefined

	function clearTimer(): void {
		if (timer === undefined) return
		globalThis.clearTimeout(timer)
		timer = undefined
	}

	function openOnce(): void {
		if (opened) return
		opened = true
		clearTimer()
		open()
	}

	return {
		setVisible(visible) {
			if (opened) return
			if (!visible) {
				clearTimer()
				return
			}
			if (normalizedDelay === 0) {
				openOnce()
				return
			}
			if (timer !== undefined) return
			timer = globalThis.setTimeout(openOnce, normalizedDelay)
		},
		openNow: openOnce,
		stop: clearTimer,
	}
}
