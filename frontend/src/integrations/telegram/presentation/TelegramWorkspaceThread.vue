<script setup lang="ts">
import { computed, nextTick, onBeforeUnmount, onUpdated, ref, watch } from 'vue'
import { Icon } from '@/shared/ui'
import type { TelegramOperationalPageModel } from './telegramOperationalPageModel'
import {
	initialTelegramHistoryScrollTop,
	shouldPrefetchTelegramHistory,
} from './telegramHistoryViewport'
import TelegramProviderMedia from './TelegramProviderMedia.vue'

const props = defineProps<{ model: TelegramOperationalPageModel }>()

const emit = defineEmits<{
	refreshContext: []
	loadOlderMessages: []
	openActions: []
	openSearch: []
	replySelected: []
	cancelReply: []
	selectMessage: [messageId: string, providerMessageId: string]
	send: []
	updateDraft: [value: string]
}>()

const emojiOpen = ref(false)
const threadMessages = ref<HTMLElement>()
const selectedChatDetail = computed(() =>
	props.model.chats.find(chat => chat.id === props.model.selectedChatId)?.detail || 'Telegram chat')
const quickEmoji = ['👍', '❤️', '😂', '🔥', '👏', '🙏'] as const
const scrollPositionByChat = new Map<string, number>()
const initializedChats = new Set<string>()
let activeThreadKey = ''
let olderRequestPending = false
let olderScrollAnchor: { key: string; scrollHeight: number; scrollTop: number } | undefined
let pendingInitialThreadKey = ''
let initialLayoutFrame = 0

function appendEmoji(emoji: string): void {
	emit('updateDraft', `${props.model.draft}${emoji}`)
	emojiOpen.value = false
}

function handleComposerKeydown(event: KeyboardEvent): void {
	if (!(event.ctrlKey || event.metaKey) || event.key !== 'Enter' || !props.model.draft.trim()) return
	event.preventDefault()
	emit('send')
}

function threadKey(): string {
	return `${props.model.accountId}:${props.model.selectedChatId || 'none'}`
}

function saveThreadScroll(): void {
	if (activeThreadKey && threadMessages.value) {
		scrollPositionByChat.set(activeThreadKey, threadMessages.value.scrollTop)
	}
}

function requestOlderMessages(): void {
	const container = threadMessages.value
	if (!container || olderRequestPending || props.model.historyPending || !props.model.hasOlderMessages) return
	olderRequestPending = true
	olderScrollAnchor = {
		key: threadKey(),
		scrollHeight: container.scrollHeight,
		scrollTop: container.scrollTop,
	}
	emit('loadOlderMessages')
}

function handleThreadScroll(): void {
	saveThreadScroll()
	if ((threadMessages.value?.scrollTop ?? Number.POSITIVE_INFINITY) <= 160) requestOlderMessages()
}

function prefetchHistoryUntilScrollable(): void {
	const container = threadMessages.value
	if (!container || !shouldPrefetchTelegramHistory(
		props.model.hasOlderMessages,
		container.scrollHeight,
		container.clientHeight,
	)) return
	requestOlderMessages()
}

watch(threadKey, async (nextKey, previousKey) => {
	if (previousKey && threadMessages.value) {
		scrollPositionByChat.set(previousKey, threadMessages.value.scrollTop)
	}
	activeThreadKey = nextKey
	await nextTick()
	const container = threadMessages.value
	if (!container) return
	const saved = scrollPositionByChat.get(nextKey)
	if (saved !== undefined) {
		pendingInitialThreadKey = ''
		initializedChats.add(nextKey)
		container.scrollTop = saved
		return
	}
	pendingInitialThreadKey = nextKey
	scheduleInitialThreadPosition()
}, { immediate: true })

function scheduleInitialThreadPosition(): void {
	if (initialLayoutFrame) return
	initialLayoutFrame = globalThis.requestAnimationFrame(() => {
		initialLayoutFrame = globalThis.requestAnimationFrame(() => {
			initialLayoutFrame = 0
			const key = threadKey()
			const container = threadMessages.value
			if (
				pendingInitialThreadKey !== key
				|| !container
				|| olderRequestPending
			) return
			const target = initialTelegramHistoryScrollTop(
				props.model.messages.length,
				container.scrollHeight,
			)
			if (target === undefined) return
			container.scrollTop = target
			if (!props.model.historyPending) {
				initializedChats.add(key)
				pendingInitialThreadKey = ''
			}
		})
	})
}

onUpdated(() => {
	if (pendingInitialThreadKey === threadKey()) scheduleInitialThreadPosition()
})

watch(() => props.model.historyPending, async (pending, wasPending) => {
	if (pending || !wasPending) return
	olderRequestPending = false
	await nextTick()
	const container = threadMessages.value
	if (olderScrollAnchor?.key === threadKey() && container) {
		container.scrollTop = olderScrollAnchor.scrollTop
			+ (container.scrollHeight - olderScrollAnchor.scrollHeight)
		scrollPositionByChat.set(olderScrollAnchor.key, container.scrollTop)
		olderScrollAnchor = undefined
		prefetchHistoryUntilScrollable()
		return
	}
	const key = threadKey()
	if (container && !initializedChats.has(key)) {
		initializedChats.add(key)
		container.scrollTop = scrollPositionByChat.get(key) ?? container.scrollHeight
	}
	prefetchHistoryUntilScrollable()
})

onBeforeUnmount(() => {
	if (initialLayoutFrame) globalThis.cancelAnimationFrame(initialLayoutFrame)
	saveThreadScroll()
})
</script>

<template>
	<main class="telegram-workspace-thread" aria-label="Telegram message thread">
		<template v-if="model.selectedChatId">
			<header class="telegram-thread-header">
				<div class="telegram-thread-header__avatar">
					<TelegramProviderMedia
						v-if="model.selectedChatAvatarProviderFileId"
						:key="model.selectedChatAvatarProviderFileId"
						:account-id="model.accountId"
						:provider-file-id="model.selectedChatAvatarProviderFileId"
						:alt="`${model.selectedChatTitle} avatar`"
						content-type="image/jpeg"
						kind="image"
						cache-class="avatar"
						:scope-key="`${model.accountId}:${model.selectedChatId}`"
					>{{ model.selectedChatTitle.slice(0, 1).toLocaleUpperCase() }}</TelegramProviderMedia>
					<template v-else>{{ model.selectedChatTitle.slice(0, 1).toLocaleUpperCase() }}</template>
				</div>
				<div>
					<h2>{{ model.selectedChatTitle }}</h2>
					<p>{{ selectedChatDetail }}</p>
				</div>
				<nav aria-label="Chat actions">
					<button type="button" title="Search" @click="emit('openSearch')"><Icon icon="tabler:search" size="1rem" /></button>
					<button type="button" title="Refresh context" @click="emit('refreshContext')">
						<Icon icon="tabler:refresh" size="1rem" />
					</button>
					<button type="button" title="Message and chat actions" @click="emit('openActions')"><Icon icon="tabler:dots-vertical" size="1rem" /></button>
				</nav>
			</header>

			<nav class="telegram-thread-tabs" aria-label="Telegram thread sections">
				<button type="button" class="active">Messages <span>{{ model.messages.length }}</span></button>
			</nav>

			<section ref="threadMessages" class="telegram-thread-messages" @scroll.passive="handleThreadScroll">
				<div v-if="model.historyPending && model.messages.length === 0" class="telegram-thread-messages__loading" role="status">
					<Icon icon="tabler:loader-2" size="1.25rem" />
					<span>Loading messages…</span>
				</div>
				<button
					v-if="model.hasOlderMessages"
					type="button"
					class="telegram-thread-load-older"
					:disabled="model.historyPending"
					@click="requestOlderMessages"
				>
					<Icon icon="tabler:arrow-up" size="1rem" />
					{{ model.historyPending ? 'Loading history…' : 'Load older messages' }}
				</button>
				<article
					v-for="message in model.messages"
					:key="`${message.providerMessageId}:${message.id}`"
					class="telegram-thread-message"
					:class="{ outgoing: message.outgoing, selected: message.selected }"
					role="button"
					tabindex="0"
					:aria-pressed="message.selected"
					@click="emit('selectMessage', message.id, message.providerMessageId)"
					@keydown.enter="emit('selectMessage', message.id, message.providerMessageId)"
				>
					<span class="telegram-thread-message__sender">{{ message.sender }}</span>
					<p v-if="message.body">{{ message.body }}</p>
					<TelegramProviderMedia
						v-if="message.media?.providerFileId"
						:key="`${message.id}:${message.media.providerFileId}:${message.media.previewProviderFileId}`"
						class="telegram-thread-message__provider-media"
						:account-id="model.accountId"
						:provider-file-id="message.media.providerFileId"
						:preview-provider-file-id="message.media.previewProviderFileId"
						:preview-inline-data="message.media.previewInlineData"
						:content-type="message.media.contentType"
						:preview-content-type="message.media.previewContentType"
						:kind="message.media.renderKind"
						:alt="message.media.filename"
						:scope-key="`${model.accountId}:${model.selectedChatId}`"
					>
						<span class="telegram-thread-message__media">
							<Icon icon="tabler:loader-2" size="1rem" />
							<span><strong>{{ message.media.filename }}</strong><small>Loading {{ message.media.kind }}…</small></span>
						</span>
					</TelegramProviderMedia>
					<span v-else-if="message.media" class="telegram-thread-message__media">
						<Icon icon="tabler:paperclip" size="1rem" />
						<span><strong>{{ message.media.filename }}</strong><small>{{ message.media.kind }}</small></span>
					</span>
					<footer>
						<time>{{ message.meta }}</time>
						<Icon v-if="message.outgoing" icon="tabler:checks" size="0.85rem" />
					</footer>
					<span v-if="message.selected" class="telegram-thread-message__selected-actions">
						<span>Message selected</span>
						<span>
							<button type="button" @click.stop="emit('replySelected')">Reply</button>
							<button type="button" @click.stop="emit('openActions')">More actions</button>
						</span>
					</span>
				</article>
				<p v-if="!model.historyPending && model.messages.length === 0" class="telegram-thread-messages__empty">
					No projected messages in this chat.
				</p>
			</section>

			<div v-if="model.replyToProviderMessageId" class="telegram-thread-reply-banner">
				<span><strong>Replying to {{ model.replyToSender || 'message' }}</strong><small>{{ model.replyToBody || 'Attachment' }}</small></span>
				<button type="button" aria-label="Cancel reply" @click="emit('cancelReply')">×</button>
			</div>
			<form class="telegram-thread-composer" @submit.prevent="emit('send')">
				<button type="button" title="Media actions" @click="emit('openActions')"><Icon icon="tabler:paperclip" size="1.1rem" /></button>
				<textarea
					rows="1"
					placeholder="Write a message…"
					:value="model.draft"
					:disabled="!model.canSend || model.sendPending"
					@input="emit('updateDraft', ($event.target as HTMLTextAreaElement).value)"
					@keydown="handleComposerKeydown"
				/>
				<div class="telegram-thread-composer__emoji">
					<button type="button" title="Emoji" :aria-expanded="emojiOpen" @click="emojiOpen = !emojiOpen"><Icon icon="tabler:mood-smile" size="1.1rem" /></button>
					<div v-if="emojiOpen" class="telegram-thread-emoji-picker" role="menu" aria-label="Emoji">
						<button v-for="emoji in quickEmoji" :key="emoji" type="button" role="menuitem" @click="appendEmoji(emoji)">{{ emoji }}</button>
					</div>
				</div>
				<button
					type="submit"
					class="telegram-thread-composer__send"
					title="Send"
					:disabled="!model.canSend || !model.draft.trim() || model.sendPending"
				>
					<Icon icon="tabler:send" size="1.1rem" />
				</button>
				<small v-if="model.sendMessage">{{ model.sendMessage }}</small>
			</form>
		</template>

		<section v-else class="telegram-workspace-thread__empty">
			<Icon icon="tabler:brand-telegram" size="2.25rem" />
			<h2>Select a Telegram chat</h2>
			<p>Choose a conversation to inspect messages and compose replies.</p>
		</section>
	</main>
</template>
