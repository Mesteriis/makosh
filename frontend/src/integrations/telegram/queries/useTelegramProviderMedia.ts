import { onBeforeUnmount, onMounted, ref, type Ref } from 'vue'
import {
	loadTelegramProviderMedia,
	readCachedTelegramProviderMedia,
	type TelegramProviderMediaPriority,
} from '../api/telegramProviderMediaGateway'

export { telegramMediaDeliveryForKind } from '../api/telegramProviderMediaGateway'
import type { TelegramMediaCacheClass } from '../api/telegramMediaMemoryCache'
import { createTelegramMediaVisibilityGate } from './telegramMediaVisibilityGate'

const MEDIA_SCROLL_ROOT = '.telegram-thread-messages, .telegram-workspace-chat-list__items'
const MEDIA_VIEWPORT_MARGIN_PX = 320

export function useTelegramProviderMedia(input: {
	accountId: () => string
	providerFileId: () => string
	contentType: () => string
	scopeKey: () => string
	priority: () => TelegramProviderMediaPriority
	cacheClass?: () => TelegramMediaCacheClass
	delivery?: () => 'inline' | 'range'
	autoLoad?: () => boolean
	autoLoadDelayMillis?: () => number
	viewportMarginPx?: () => number
}, sharedRoot?: Ref<HTMLElement | undefined>): {
	root: Ref<HTMLElement | undefined>
	url: Ref<string>
	loading: Ref<boolean>
	failed: Ref<boolean>
	requestNow: () => void
} {
	const root = sharedRoot ?? ref<HTMLElement>()
	const cacheClass = input.cacheClass?.() ?? 'media'
	const cached = readCachedTelegramProviderMedia(input.accountId(), input.providerFileId(), cacheClass)
	const url = ref(cached?.url ?? '')
	const loading = ref(false)
	const failed = ref(false)
	let observer: IntersectionObserver | undefined
	let fallbackScrollRoot: Element | undefined
	let fallbackFrame = 0
	let active = true
	let loadStarted = Boolean(cached)
	const visibilityGate = createTelegramMediaVisibilityGate(
		() => { void load() },
		input.autoLoadDelayMillis?.() ?? 0,
	)

	async function load(): Promise<void> {
		if (loadStarted || url.value || failed.value || !input.accountId() || !input.providerFileId()) return
		loadStarted = true
		loading.value = true
		failed.value = false
		try {
			const artifact = await loadTelegramProviderMedia(
				input.accountId(),
				input.providerFileId(),
				input.contentType(),
				input.scopeKey(),
				input.priority(),
				cacheClass,
				input.delivery?.() ?? 'inline',
			)
			if (active) url.value = artifact.url
		} catch (error) {
			if (active && (!(error instanceof Error) || error.name !== 'AbortError')) {
				failed.value = true
				loadStarted = false
			}
		} finally {
			if (active) loading.value = false
		}
	}

	onMounted(() => {
		if (input.autoLoad?.() === false) return
		if (!('IntersectionObserver' in globalThis)) {
			fallbackScrollRoot = root.value?.closest(MEDIA_SCROLL_ROOT) ?? undefined
			fallbackScrollRoot?.addEventListener('scroll', scheduleFallbackVisibilityCheck, { passive: true })
			globalThis.addEventListener('resize', scheduleFallbackVisibilityCheck, { passive: true })
			scheduleFallbackVisibilityCheck()
			return
		}
		const scrollRoot = root.value?.closest(MEDIA_SCROLL_ROOT)
		observer = new IntersectionObserver((entries) => {
			visibilityGate.setVisible(entries.some(entry => entry.isIntersecting))
		}, {
			root: scrollRoot instanceof Element ? scrollRoot : null,
			rootMargin: `${input.viewportMarginPx?.() ?? MEDIA_VIEWPORT_MARGIN_PX}px 0px`,
		})
		if (root.value) observer.observe(root.value)
	})

	onBeforeUnmount(() => {
		active = false
		visibilityGate.stop()
		observer?.disconnect()
		fallbackScrollRoot?.removeEventListener('scroll', scheduleFallbackVisibilityCheck)
		globalThis.removeEventListener('resize', scheduleFallbackVisibilityCheck)
		if (fallbackFrame) globalThis.cancelAnimationFrame(fallbackFrame)
	})

	function scheduleFallbackVisibilityCheck(): void {
		if (fallbackFrame) return
		fallbackFrame = globalThis.requestAnimationFrame(() => {
			fallbackFrame = 0
			const target = root.value
			visibilityGate.setVisible(Boolean(target && isNearVisibleViewport(
				target,
				fallbackScrollRoot,
				input.viewportMarginPx?.() ?? MEDIA_VIEWPORT_MARGIN_PX,
			)))
		})
	}

	function requestNow(): void {
		if (failed.value) failed.value = false
		void load()
	}

	return { root, url, loading, failed, requestNow }
}

function isNearVisibleViewport(target: Element, scrollRoot: Element | undefined, marginPx: number): boolean {
	const targetRect = target.getBoundingClientRect()
	const viewport = scrollRoot?.getBoundingClientRect() ?? {
		top: 0,
		bottom: globalThis.innerHeight,
	}
	return targetRect.bottom >= viewport.top - marginPx
		&& targetRect.top <= viewport.bottom + marginPx
}
