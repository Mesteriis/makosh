<script setup lang="ts">
import { ChatInput } from '@/shared/ui'
import type { TelegramOperationalPageModel } from './telegramOperationalPageModel'
import './telegramNewMessageDialog.css'

defineProps<{
	open: boolean
	model: TelegramOperationalPageModel
}>()

const emit = defineEmits<{
	close: []
	selectChat: [providerChatId: string]
	send: []
	updateDraft: [value: string]
}>()
</script>


<template>
	<div v-if="open" class="telegram-new-message-dialog" role="dialog" aria-modal="true" aria-labelledby="telegram-new-message-title">
		<button
			type="button"
			class="telegram-new-message-dialog__backdrop"
			aria-label="Close new Telegram message"
			@click="emit('close')"
		/>
		<section class="telegram-new-message-dialog__surface">
			<header>
				<div>
					<span>Telegram</span>
					<h2 id="telegram-new-message-title">New message</h2>
					<p>Choose an existing provider conversation and compose a message without leaving the workspace.</p>
				</div>
				<button type="button" aria-label="Close new Telegram message" @click="emit('close')">×</button>
			</header>
			<div class="telegram-new-message">
			<label for="telegram-new-message-chat">
				Conversation
				<select
					id="telegram-new-message-chat"
					:value="model.selectedChatId"
					:disabled="model.chats.length === 0 || model.sendPending"
					@change="emit('selectChat', ($event.target as HTMLSelectElement).value)"
				>
					<option value="">Select a Telegram chat</option>
					<option v-for="chat in model.chats" :key="chat.id" :value="chat.id">
						{{ chat.title }} — {{ chat.detail }}
					</option>
				</select>
			</label>

			<div v-if="model.chats.length === 0" class="telegram-new-message__empty" role="status">
				<span class="telegram-new-message__empty-mark" aria-hidden="true">+</span>
				<strong>No synchronized chats</strong>
				<p>Sync the Telegram account first. The composer never invents a recipient or sends to an unverified provider target.</p>
			</div>
			<ChatInput
				v-else
				id="telegram-new-message-body"
				:model-value="model.draft"
				label="Message"
				placeholder="Write a Telegram message…"
				send-label="Send message"
				:helper="model.sendMessage || 'Ctrl+Enter or Meta+Enter sends the message.'"
				:disabled="!model.canSend || !model.selectedChatId"
				:loading="model.sendPending"
				:show-attach="false"
				:max-length="4096"
				@submit="emit('send')"
				@update:model-value="emit('updateDraft', $event)"
			/>
			</div>
		</section>
	</div>
</template>
