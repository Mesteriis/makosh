<script setup lang="ts">
import { ref } from 'vue'
import { useResponsiveWorkspaceInspector } from '../../../shared/ui/shell/useResponsiveWorkspaceInspector'
import TelegramAccountAccessPanel from './TelegramAccountAccessPanel.vue'
import type { TelegramAccountAccessModel } from './telegramAccountAccessModel'
import TelegramDiscoveryPanel from './TelegramDiscoveryPanel.vue'
import TelegramNewMessageDialog from './TelegramNewMessageDialog.vue'
import type { TelegramDiscoveryModel } from './telegramDiscoveryModel'
import type { TelegramOperationalPageModel } from './telegramOperationalPageModel'
import TelegramWorkspaceChatList from './TelegramWorkspaceChatList.vue'
import TelegramWorkspaceThread from './TelegramWorkspaceThread.vue'
import TelegramWorkspaceToolbar from './TelegramWorkspaceToolbar.vue'
import './telegramOperationalPage.css'
import '../../../shared/ui/shell/providerOperationalWorkspace.css'

defineProps<{
	accountAccess: TelegramAccountAccessModel
	discovery: TelegramDiscoveryModel
	model: TelegramOperationalPageModel
}>()

const emit = defineEmits<{
	load: []
	provisionAccount: []
	refreshAccounts: []
	refreshChatContext: []
	replayAccount: []
	restartAccount: []
	retireAccount: []
	search: []
	selectAccount: [accountId: string]
	selectChat: [providerChatId: string]
	selectMessage: [messageId: string, providerMessageId: string]
	send: []
	submitAuthorizationPassword: []
	updateAuthorizationPassword: [value: string]
	updateDraft: [value: string]
	updateSearchQuery: [value: string]
	updateProvisionAccountId: [value: string]
	updateProvisionDisplayName: [value: string]
	updateProvisionExternalAccountId: [value: string]
}>()

const accountDialogOpen = ref(false)
const composeOpen = ref(false)
const discoveryDialogOpen = ref(false)
const inspectorTab = ref<'actions' | 'context'>('context')
const inspectorVisible = useResponsiveWorkspaceInspector()

function openDiscovery(): void {
	discoveryDialogOpen.value = true
	emit('search')
}
</script>

<template>
	<section class="telegram-operational-page">
		<TelegramWorkspaceToolbar
			:account-access="accountAccess"
			:discovery="discovery"
			:model="model"
			@add-account="accountDialogOpen = true"
			@compose="composeOpen = true"
			@load="emit('load')"
			@open-search="openDiscovery"
			@toggle-inspector="inspectorVisible = !inspectorVisible"
			@update-search-query="emit('updateSearchQuery', $event)"
		/>

		<section class="telegram-action-rail" aria-label="Telegram actions">
			<div>
				<button
					type="button"
					:disabled="model.status === 'loading' || accountAccess.authorizationState !== 'ready'"
					@click="emit('load')"
				>Sync Chats</button>
				<button type="button" :disabled="!model.selectedChatId" @click="emit('refreshChatContext')">Sync History</button>
			</div>
			<nav aria-label="Chat groups">
				<button type="button" class="active">All <span>{{ model.chats.length }}</span></button>
				<button type="button" disabled>Unread</button>
				<button type="button" disabled>Mentions</button>
				<button type="button" disabled>Archived</button>
			</nav>
			<button type="button" :class="{ active: inspectorVisible }" @click="inspectorVisible = !inspectorVisible">
				Details
			</button>
		</section>

		<div class="telegram-workspace-grid" :class="{ 'inspector-hidden': !inspectorVisible }" :aria-busy="model.status === 'loading'">
			<TelegramWorkspaceChatList :model="model" @select-chat="emit('selectChat', $event)" />
			<TelegramWorkspaceThread
				:model="model"
				@refresh-context="emit('refreshChatContext')"
				@select-message="(messageId, providerMessageId) => emit('selectMessage', messageId, providerMessageId)"
				@send="emit('send')"
				@update-draft="emit('updateDraft', $event)"
			/>

			<aside v-if="inspectorVisible" class="telegram-workspace-inspector" aria-label="Telegram details">
				<header>
					<div><h2>Details</h2><p>{{ model.selectedChatTitle || 'No chat selected' }}</p></div>
					<button type="button" aria-label="Close details" @click="inspectorVisible = false">×</button>
				</header>
				<nav aria-label="Telegram inspector sections">
					<button
						type="button"
						:class="{ active: inspectorTab === 'context' }"
						@click="inspectorTab = 'context'"
					>Context</button>
					<button
						type="button"
						:class="{ active: inspectorTab === 'actions' }"
						@click="inspectorTab = 'actions'"
					>Actions</button>
				</nav>
				<div class="telegram-workspace-inspector__body">
					<slot v-if="inspectorTab === 'context'" name="inspector" />
					<slot v-else name="commands" />
				</div>
			</aside>
		</div>

		<TelegramNewMessageDialog
			:open="composeOpen"
			:model="model"
			@close="composeOpen = false"
			@select-chat="emit('selectChat', $event)"
			@send="emit('send')"
			@update-draft="emit('updateDraft', $event)"
		/>

		<div v-if="accountDialogOpen" class="telegram-workspace-dialog" role="dialog" aria-modal="true" aria-label="Telegram accounts">
			<button type="button" class="telegram-workspace-dialog__backdrop" aria-label="Close Telegram accounts" @click="accountDialogOpen = false" />
			<section class="telegram-workspace-dialog__surface">
				<header><div><span>Telegram</span><h2>Accounts & authorization</h2></div><button type="button" @click="accountDialogOpen = false">×</button></header>
				<TelegramAccountAccessPanel
					:model="accountAccess"
					@provision="emit('provisionAccount')"
					@refresh="emit('refreshAccounts')"
					@replay="emit('replayAccount')"
					@restart="emit('restartAccount')"
					@retire="emit('retireAccount')"
					@select-account="emit('selectAccount', $event)"
					@submit-password="emit('submitAuthorizationPassword')"
					@update-password="emit('updateAuthorizationPassword', $event)"
					@update-provision-account-id="emit('updateProvisionAccountId', $event)"
					@update-provision-display-name="emit('updateProvisionDisplayName', $event)"
					@update-provision-external-account-id="emit('updateProvisionExternalAccountId', $event)"
				/>
			</section>
		</div>

		<div v-if="discoveryDialogOpen" class="telegram-workspace-dialog" role="dialog" aria-modal="true" aria-label="Telegram search">
			<button type="button" class="telegram-workspace-dialog__backdrop" aria-label="Close Telegram search" @click="discoveryDialogOpen = false" />
			<section class="telegram-workspace-dialog__surface">
				<header><div><span>Telegram</span><h2>Search & context</h2></div><button type="button" @click="discoveryDialogOpen = false">×</button></header>
				<TelegramDiscoveryPanel
					:model="discovery"
					@refresh-context="emit('refreshChatContext')"
					@search="emit('search')"
					@select-chat="emit('selectChat', $event)"
					@update-query="emit('updateSearchQuery', $event)"
				/>
			</section>
		</div>
	</section>
</template>
	search: []
