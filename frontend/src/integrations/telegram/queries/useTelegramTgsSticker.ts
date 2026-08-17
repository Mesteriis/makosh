import lottie, { type AnimationItem } from 'lottie-web/build/player/lottie_light'
import { onBeforeUnmount, onMounted, ref, watch, type Ref } from 'vue'

const MAX_TGS_JSON_BYTES = 8 * 1024 * 1024

export function useTelegramTgsSticker(
	url: () => string,
	container: Ref<HTMLElement | undefined>,
) {
	const failed = ref(false)
	let animation: AnimationItem | undefined
	let abort: AbortController | undefined
	let generation = 0

	onMounted(() => { void renderSticker() })
	watch(url, () => { void renderSticker() }, { flush: 'post' })

	onBeforeUnmount(() => {
		generation += 1
		abort?.abort()
		animation?.destroy()
	})

	async function renderSticker(): Promise<void> {
		const currentGeneration = ++generation
		abort?.abort()
		animation?.destroy()
		animation = undefined
		failed.value = false
		if (!url() || !container.value) return

		abort = new AbortController()
		try {
			const response = await fetch(url(), { signal: abort.signal })
			if (!response.ok) throw new Error('tgs_fetch_failed')
			const compressed = await response.arrayBuffer()
			const stream = new Blob([compressed]).stream().pipeThrough(new DecompressionStream('gzip'))
			const text = await new Response(stream).text()
			if (!text || text.length > MAX_TGS_JSON_BYTES) throw new Error('tgs_payload_invalid')
			const data = JSON.parse(text) as Record<string, unknown>
			if (!Array.isArray(data.layers) || hasExternalAssets(data.assets)) {
				throw new Error('tgs_animation_invalid')
			}
			if (currentGeneration !== generation || !container.value) return
			animation = lottie.loadAnimation({
				container: container.value,
				renderer: 'svg',
				loop: true,
				autoplay: true,
				animationData: data,
			})
		} catch (error) {
			if (currentGeneration === generation && (!(error instanceof Error) || error.name !== 'AbortError')) {
				failed.value = true
			}
		}
	}

	return { failed }
}

function hasExternalAssets(value: unknown): boolean {
	if (!Array.isArray(value)) return false
	return value.some((asset) => {
		if (!asset || typeof asset !== 'object') return false
		const candidate = asset as Record<string, unknown>
		return (typeof candidate.p === 'string' && candidate.p.length > 0)
			|| (typeof candidate.u === 'string' && candidate.u.length > 0)
	})
}
