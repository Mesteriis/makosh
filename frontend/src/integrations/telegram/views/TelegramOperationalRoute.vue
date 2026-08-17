<script setup lang="ts">
import { onBeforeUnmount, onMounted, watch } from 'vue'
import type { ProviderAccountNavigationSnapshot } from '../../../shared/ui/shell/providerAccountNavigation'
import TelegramOperationalPage from '../presentation/TelegramOperationalPage.vue'
import TelegramCommandWorkbench from '../presentation/TelegramCommandWorkbench.vue'
import TelegramMessageInspector from '../presentation/TelegramMessageInspector.vue'
import TelegramOperationRetryPanel from '../presentation/TelegramOperationRetryPanel.vue'
import { useTelegramOperationalPage } from '../queries/useTelegramOperationalPage'
import { useTelegramAccountAccess } from '../queries/useTelegramAccountAccess'
import { useTelegramChatCommands } from '../queries/useTelegramChatCommands'
import { useTelegramDiscovery } from '../queries/useTelegramDiscovery'
import { useTelegramMediaCommands } from '../queries/useTelegramMediaCommands'
import { useTelegramMessageCommands } from '../queries/useTelegramMessageCommands'
import { useTelegramMessageInspector } from '../queries/useTelegramMessageInspector'
import { useTelegramOperationRetry } from '../queries/useTelegramOperationRetry'
import { useTelegramTopicCommands } from '../queries/useTelegramTopicCommands'
import { telegramAccountNavigation } from '../presentation/telegramAccountNavigation'
import { canStartTelegramOperationalLane } from '../presentation/telegramAccountAccessModel'
import type { TelegramDiscoveryResultRow } from '../presentation/telegramDiscoveryModel'
import { EMPTY_PROVIDER_SOURCE_NAMES } from '../../../shared/identity/providerSourceIdentity'

const props = defineProps<{
	canAuthorize: boolean
	canManageLifecycle: boolean
	canQuery: boolean
	canReplay: boolean
	canReconfigure: boolean
	canSend: boolean
	navigationAccountId?: string
	senderPersonaNames?: ReadonlyMap<string, string>
}>()
const emit = defineEmits<{
	accountNavigationChange: [snapshot: ProviderAccountNavigationSnapshot]
}>()
const surface = useTelegramOperationalPage(
	() => props.canSend,
	() => props.canReplay,
	() => props.senderPersonaNames ?? EMPTY_PROVIDER_SOURCE_NAMES,
)
const accountAccess = useTelegramAccountAccess({
	canAuthorize: () => props.canAuthorize,
	canManageLifecycle: () => props.canManageLifecycle,
	canReconfigure: () => props.canReconfigure,
})
const discovery = useTelegramDiscovery({
	accountId: () => accountAccess.selectedAccountId.value,
	canQuery: () => props.canQuery,
	selectedChatId: () => surface.model.value.selectedChatId,
	senderPersonaNames: () => props.senderPersonaNames ?? EMPTY_PROVIDER_SOURCE_NAMES,
})
const commandTarget = {
	accountId: () => accountAccess.selectedAccountId.value,
	canCommand: () => props.canSend,
	providerChatId: () => surface.model.value.selectedChatId,
}
const messageCommands = useTelegramMessageCommands({
	...commandTarget,
	providerMessageId: () => surface.model.value.selectedProviderMessageId,
})
const chatCommands = useTelegramChatCommands(commandTarget)
const topicCommands = useTelegramTopicCommands(commandTarget)
const mediaCommands = useTelegramMediaCommands(commandTarget)
const messageInspector = useTelegramMessageInspector({
	accountId: () => accountAccess.selectedAccountId.value,
	canQuery: () => props.canQuery,
	messageId: () => surface.model.value.selectedMessageId,
	providerChatId: () => surface.model.value.selectedChatId,
	providerMessageId: () => surface.model.value.selectedProviderMessageId,
	senderPersonaNames: () => props.senderPersonaNames ?? EMPTY_PROVIDER_SOURCE_NAMES,
})
const operationRetry = useTelegramOperationRetry(() => props.canManageLifecycle)
let operationalLaneStateKey = ''
let operationalLaneGeneration = 0

async function reconcileOperationalLane(): Promise<void> {
	const accountModel = accountAccess.model.value
	// Account access and its watcher update before refreshAccounts resumes.
	// Bind the operational surface first so that the watcher cannot start the
	// selected lane with the composable's initial empty account ID.
	surface.updateAccountId(accountModel.selectedAccountId)
	const nextLaneKey = canStartTelegramOperationalLane(accountModel)
		? accountModel.selectedAccountId
		: ''
	const nextLaneStateKey = nextLaneKey
		? `active:${nextLaneKey}`
		: `blocked:${accountModel.selectedAccountId}:${accountModel.authorizationState}`
	if (nextLaneStateKey === operationalLaneStateKey) return
	const generation = ++operationalLaneGeneration
	operationalLaneStateKey = nextLaneStateKey
	if (!nextLaneKey) {
		surface.stopRealtime()
		if (accountModel.selectedAccountId) {
			await surface.loadCachedProjection()
			if (generation !== operationalLaneGeneration) return
			surface.suspendForAuthorization(accountModel.authorizationState)
		}
		return
	}
	await surface.loadChats()
	if (generation !== operationalLaneGeneration) return
	await surface.startRealtime()
}

async function refreshAccounts(): Promise<void> {
	await accountAccess.refresh()
	const accountId = accountAccess.selectedAccountId.value
	updateAccountId(accountId)
	await reconcileOperationalLane()
}

async function selectAccount(accountId: string): Promise<void> {
	surface.stopRealtime()
	accountAccess.selectAccount(accountId)
	updateAccountId(accountId)
	await reconcileOperationalLane()
}

async function selectChat(providerChatId: string): Promise<void> {
	await surface.selectChat(providerChatId)
}

async function selectSearchResult(result: TelegramDiscoveryResultRow): Promise<void> {
	await surface.selectChat(result.providerChatId)
	if (result.kind === 'message') {
		surface.selectMessage(result.id, result.providerMessageId)
	}
}

function updateAccountId(accountId: string): void {
	accountAccess.selectAccount(accountId)
	surface.updateAccountId(accountId)
}

watch(
	accountAccess.model,
	(model) => {
		emit('accountNavigationChange', telegramAccountNavigation(model))
		void reconcileOperationalLane()
	},
	{ immediate: true },
)

watch(
	() => props.navigationAccountId,
	(accountId) => {
		if (accountId === undefined || accountId === accountAccess.selectedAccountId.value) return
		void selectAccount(accountId)
	},
)

onMounted(() => {
	void refreshAccounts()
})

onBeforeUnmount(() => {
	surface.stopRealtime()
})
</script>

<template>
	<TelegramOperationalPage
		:account-access="accountAccess.model.value"
		:discovery="discovery.model.value"
		:model="surface.model.value"
		@provision-account="accountAccess.provision"
		@refresh-accounts="refreshAccounts"
		@refresh-chat-context="discovery.refreshChatContext"
		@replay-account="accountAccess.replay"
		@restart-account="accountAccess.restart"
		@retire-account="accountAccess.retire"
		@select-account="selectAccount"
		@submit-authorization-password="accountAccess.submitPassword"
		@load="surface.loadChats"
		@load-more-chats="surface.loadMoreChats"
		@load-older-messages="surface.loadOlderMessages"
		@begin-reply="surface.beginReply"
		@cancel-reply="surface.cancelReply"
		@search="discovery.search"
		@select-search-result="selectSearchResult"
		@select-chat="selectChat"
		@select-message="surface.selectMessage"
		@send="surface.send"
		@update-authorization-password="accountAccess.updatePassword"
		@update-draft="surface.updateDraft"
		@update-search-query="discovery.updateQuery"
		@update-provision-account-id="accountAccess.updateProvisionAccountId"
		@update-provision-display-name="accountAccess.updateProvisionDisplayName"
		@update-provision-external-account-id="accountAccess.updateProvisionExternalAccountId"
	>
		<template #inspector>
			<TelegramMessageInspector
				:model="messageInspector.model.value"
				@inspect="messageInspector.inspect"
			/>
			<TelegramOperationRetryPanel
				:model="operationRetry.model.value"
				@retry="operationRetry.retry"
				@update-operation-id="operationRetry.updateOperationId"
			/>
		</template>
		<template #commands>
			<TelegramCommandWorkbench
				:chat="chatCommands.model.value"
				:media="mediaCommands.model.value"
				:message="messageCommands.model.value"
				:topic="topicCommands.model.value"
				@chat-add-to-folder="chatCommands.addToFolder"
				@chat-archive="chatCommands.archive"
				@chat-join="chatCommands.join"
				@chat-leave="chatCommands.leave"
				@chat-mark-unread="chatCommands.markUnread"
				@chat-mute="chatCommands.mute"
				@chat-remove-from-folder="chatCommands.removeFromFolder"
				@chat-reassign-folders="chatCommands.reassignFolders"
				@media-download="mediaCommands.downloadFile"
				@media-send="mediaCommands.sendMedia"
				@message-delete="messageCommands.remove"
				@message-edit="messageCommands.edit"
				@message-forward="messageCommands.forward"
				@message-pin="messageCommands.pin"
				@message-react="messageCommands.react"
				@message-reply="messageCommands.reply"
				@message-restore="messageCommands.restore"
				@topic-close="topicCommands.closeTopic"
				@topic-create="topicCommands.createTopic"
				@topic-participants="topicCommands.refreshParticipants"
				@topic-refresh="topicCommands.refreshTopics"
				@topic-search="topicCommands.searchMessages"
				@update-chat-folder-id="chatCommands.updateFolderId"
				@update-chat-target-folder-ids="chatCommands.updateTargetFolderIds"
				@update-media-blob-ref="mediaCommands.updateBlobRef"
				@update-media-backup-class="mediaCommands.updateBackupClass"
				@update-media-caption="mediaCommands.updateCaption"
				@update-media-declared-size="mediaCommands.updateDeclaredSize"
				@update-media-filename="mediaCommands.updateFilename"
				@update-media-kind="mediaCommands.updateMediaKind"
				@update-media-provider-file-id="mediaCommands.updateProviderFileId"
				@update-media-reference-id-hex="mediaCommands.updateReferenceIdHex"
				@update-message-emoji="messageCommands.updateEmoji"
				@update-message-restore-reason="messageCommands.updateRestoreReason"
				@update-message-target-chat-id="messageCommands.updateTargetChatId"
				@update-message-text="messageCommands.updateText"
				@update-topic-id="topicCommands.updateTopicId"
				@update-topic-search-query="topicCommands.updateProviderSearchQuery"
				@update-topic-title="topicCommands.updateTopicTitle"
			/>
		</template>
	</TelegramOperationalPage>
</template>
