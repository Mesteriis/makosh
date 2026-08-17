<script setup lang="ts">
import { Icon } from '@/shared/ui'
import type { TelegramOperationalPageModel } from './telegramOperationalPageModel'
import TelegramProviderMedia from './TelegramProviderMedia.vue'

defineProps<{ model: TelegramOperationalPageModel }>()

const emit = defineEmits<{
	selectChat: [providerChatId: string]
	loadMore: []
}>()

function handleScroll(event: Event): void {
	const target = event.currentTarget as HTMLElement
	if (target.scrollHeight - target.scrollTop - target.clientHeight <= 240) emit('loadMore')
}
</script>

<template>
	<aside class="telegram-workspace-chat-list" aria-label="Telegram chats">
		<header>
			<div>
				<strong>Chats</strong>
				<small>{{ model.chats.length }} conversations</small>
			</div>
			<button type="button" title="Chat list options"><Icon icon="tabler:dots" size="1rem" /></button>
		</header>

		<p v-if="model.statusMessage" class="telegram-workspace-chat-list__status" :role="model.status === 'error' ? 'alert' : 'status'">
			{{ model.statusMessage }}
		</p>

		<div class="telegram-workspace-chat-list__items" @scroll.passive="handleScroll">
			<button
				v-for="chat in model.chats"
				:key="`${chat.id}:${chat.avatarProviderFileId}`"
				type="button"
				class="telegram-workspace-chat"
				:class="{ selected: chat.selected }"
				:aria-pressed="chat.selected"
				@click="emit('selectChat', chat.id)"
			>
				<span class="telegram-workspace-chat__avatar">
					<TelegramProviderMedia
						v-if="chat.avatarProviderFileId"
						:key="chat.avatarProviderFileId"
						:account-id="model.accountId"
						:provider-file-id="chat.avatarProviderFileId"
						:alt="`${chat.title} avatar`"
						content-type="image/jpeg"
						kind="image"
						:scope-key="`${model.accountId}:chat-list`"
						priority="background"
						cache-class="avatar"
					>{{ chat.title.slice(0, 1).toLocaleUpperCase() }}</TelegramProviderMedia>
					<template v-else>{{ chat.title.slice(0, 1).toLocaleUpperCase() }}</template>
				</span>
				<span class="telegram-workspace-chat__body">
					<strong>{{ chat.title }}</strong>
					<small>{{ chat.detail }}</small>
				</span>
				<span v-if="chat.selected" class="telegram-workspace-chat__selected" />
			</button>
			<button
				v-if="model.hasMoreChats"
				type="button"
				class="telegram-workspace-chat-list__load-more"
				:disabled="model.chatPending"
				@click="emit('loadMore')"
			>
				{{ model.chatPending ? 'Loading more chats…' : 'Load more chats' }}
			</button>

			<section v-if="model.status !== 'loading' && model.chats.length === 0" class="telegram-workspace-chat-list__empty">
				<Icon icon="tabler:messages" size="1.75rem" />
				<strong>No Telegram chats</strong>
				<small>Sync the selected account to load provider conversations.</small>
			</section>
		</div>
	</aside>
</template>
