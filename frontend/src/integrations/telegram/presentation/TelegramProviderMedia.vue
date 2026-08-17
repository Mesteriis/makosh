<script setup lang="ts">
import { computed, ref } from 'vue'
import {
	telegramMediaDeliveryForKind,
	useTelegramProviderMedia,
} from '../queries/useTelegramProviderMedia'
import TelegramTgsSticker from './TelegramTgsSticker.vue'
import {
	fallbackVideoPreviewTime,
	inlineVideoPreviewDataUrl,
	shouldRewindFallbackPreview,
} from './telegramVideoPreview'

const props = withDefaults(defineProps<{
	accountId: string
	providerFileId: string
	previewProviderFileId?: string
	previewInlineData?: Uint8Array
	contentType?: string
	previewContentType?: string
	kind: 'animation' | 'audio' | 'file' | 'image' | 'tgs' | 'video'
	alt?: string
	scopeKey: string
	priority?: 'interactive' | 'background'
	cacheClass?: 'avatar' | 'media'
}>(), {
	contentType: '',
	previewProviderFileId: '',
	previewInlineData: () => new Uint8Array(),
	previewContentType: 'image/jpeg',
	alt: '',
	priority: 'interactive',
	cacheClass: 'media',
})

const root = ref<HTMLElement>()
const { url, loading, failed, requestNow } = useTelegramProviderMedia({
	accountId: () => props.accountId,
	providerFileId: () => props.providerFileId,
	contentType: () => props.contentType,
	scopeKey: () => props.scopeKey,
	priority: () => props.priority,
	cacheClass: () => props.cacheClass,
	delivery: () => telegramMediaDeliveryForKind(props.kind),
	autoLoad: () => ['animation', 'image', 'tgs', 'video'].includes(props.kind),
	autoLoadDelayMillis: () => ['animation', 'tgs', 'video'].includes(props.kind) ? 2_000 : 0,
	viewportMarginPx: () => ['animation', 'tgs', 'video'].includes(props.kind) ? 0 : 320,
}, root)
const { url: previewUrl, failed: previewFailed } = useTelegramProviderMedia({
	accountId: () => props.accountId,
	providerFileId: () => ['animation', 'tgs', 'video'].includes(props.kind) ? props.previewProviderFileId : '',
	contentType: () => props.previewContentType,
	scopeKey: () => props.scopeKey,
	priority: () => 'background',
	cacheClass: () => 'media',
	viewportMarginPx: () => 320,
}, root)
const displayPreviewUrl = computed(() => previewUrl.value || inlineVideoPreviewDataUrl(props.previewInlineData))
const unavailable = computed(() => props.kind === 'image'
	? failed.value
	: props.kind === 'video'
		? failed.value && !displayPreviewUrl.value && (!props.previewProviderFileId || previewFailed.value)
		: failed.value)
const fallbackPreviewTime = ref(0)

function requestMedia(event?: Event): void {
	if (props.kind === 'image' || url.value) return
	event?.stopPropagation()
	requestNow()
}

function revealFallbackVideoFrame(event: Event): void {
	if (displayPreviewUrl.value) return
	const video = event.currentTarget as HTMLVideoElement
	const previewTime = fallbackVideoPreviewTime(video.duration)
	if (previewTime === 0) return
	fallbackPreviewTime.value = previewTime
	video.currentTime = previewTime
}

function rewindFallbackVideoFrame(event: Event): void {
	const video = event.currentTarget as HTMLVideoElement
	if (shouldRewindFallbackPreview(video.currentTime, fallbackPreviewTime.value)) {
		video.currentTime = 0
	}
	fallbackPreviewTime.value = 0
}
</script>

<template>
	<span
		ref="root"
		class="telegram-provider-media"
		:class="{
			failed: unavailable,
			video: kind === 'video',
			animation: kind === 'animation',
			tgs: kind === 'tgs',
			audio: kind === 'audio',
			file: kind === 'file',
		}"
		:role="kind !== 'image' && !url ? 'button' : undefined"
		:tabindex="kind !== 'image' && !url ? 0 : undefined"
		@keydown.enter="requestMedia"
	>
		<img v-if="kind === 'image' && url" :src="url" :alt="alt" loading="lazy" decoding="async" />
		<video
			v-else-if="kind === 'video' && url"
			:src="url"
			:poster="displayPreviewUrl || undefined"
			aria-label="Telegram video"
			controls
			preload="auto"
			playsinline
			@click.stop
			@loadedmetadata="revealFallbackVideoFrame"
			@play="rewindFallbackVideoFrame"
		/>
		<video
			v-else-if="kind === 'animation' && url"
			:src="url"
			:poster="displayPreviewUrl || undefined"
			aria-label="Telegram animation"
			autoplay
			loop
			muted
			playsinline
			preload="auto"
			@click.stop
		/>
		<TelegramTgsSticker
			v-else-if="kind === 'tgs' && url"
			:url="url"
			:alt="alt"
			@click.stop
		/>
		<template v-else-if="kind === 'video' || kind === 'animation' || kind === 'tgs'">
			<img v-if="displayPreviewUrl" class="telegram-provider-media__preview" :src="displayPreviewUrl" :alt="alt" loading="lazy" decoding="async" />
			<slot v-else />
			<button
				type="button"
				class="telegram-provider-media__play"
				:class="{ loading, retry: failed }"
				:disabled="loading"
				:aria-label="loading
					? `Loading Telegram ${kind}`
					: failed ? `Retry Telegram ${kind}` : `Load Telegram ${kind}`"
				@click="requestMedia"
			>
				<span v-if="loading" class="telegram-provider-media__spinner" aria-hidden="true" />
				<span v-else-if="failed" aria-hidden="true">↻</span>
				<span v-else aria-hidden="true">▶</span>
			</button>
		</template>
		<audio
			v-else-if="kind === 'audio' && url"
			:src="url"
			aria-label="Telegram audio"
			controls
			preload="metadata"
			@click.stop
		/>
		<button
			v-else-if="kind === 'audio'"
			type="button"
			class="telegram-provider-media__load"
			aria-label="Load Telegram audio"
			@click="requestMedia"
		>
			<span aria-hidden="true">▶</span>
			<span>{{ alt || 'Load audio' }}</span>
		</button>
		<a
			v-else-if="kind === 'file' && url"
			class="telegram-provider-media__download"
			:href="url"
			:download="alt || undefined"
			@click.stop
		>{{ alt || 'Open file' }}</a>
		<button
			v-else-if="kind === 'file'"
			type="button"
			class="telegram-provider-media__load"
			aria-label="Load Telegram file"
			@click="requestMedia"
		>
			<span aria-hidden="true">↓</span>
			<span>{{ alt || 'Load file' }}</span>
		</button>
		<slot v-else />
	</span>
</template>
