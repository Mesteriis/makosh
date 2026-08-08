<script setup lang="ts">
import { ref } from 'vue'
import { useResponsiveWorkspaceInspector } from '../../../shared/ui/shell/useResponsiveWorkspaceInspector'
import type {
	MailCompositionModel,
	MailDraftEditorPatch,
	MailSignatureEditorPatch,
	MailTemplateEditorPatch,
} from './mailCompositionModel'
import MailCompositionPanel from './MailCompositionPanel.vue'
import type { MailDeliveryModel } from './mailDeliveryModel'
import MailDeliveryPanel from './MailDeliveryPanel.vue'
import type { MailMessageFlagModel } from './mailMessageFlagModel'
import type { MailMessageLocationModel } from './mailMessageLocationModel'
import type { MailMessagePermanentDeleteModel } from './mailMessagePermanentDeleteModel'
import type { MailOperationalReadModel } from './mailOperationalReadModel'
import MailSyncHealthPanel from './MailSyncHealthPanel.vue'
import type { MailSyncHealthModel } from './mailSyncHealthModel'
import type { MailSyncModel } from './mailSyncModel'
import MailWorkspaceInspector from './MailWorkspaceInspector.vue'
import MailWorkspaceList from './MailWorkspaceList.vue'
import MailWorkspaceReader from './MailWorkspaceReader.vue'
import MailWorkspaceToolbar from './MailWorkspaceToolbar.vue'
import './mailOperationalPage.css'
import '../../../shared/ui/shell/providerOperationalWorkspace.css'

defineProps<{
	compositionModel: MailCompositionModel
	bodyContentStatus: 'idle' | 'loading' | 'ready' | 'unavailable'
	bodyContentStatusMessage: string
	bodyText: string
	bodyFormat: 'text' | 'html'
	deliveryModel: MailDeliveryModel
	flagModel: MailMessageFlagModel
	locationModel: MailMessageLocationModel
	permanentDeleteModel: MailMessagePermanentDeleteModel
	readModel: MailOperationalReadModel
	syncHealthModel: MailSyncHealthModel
	syncModel: MailSyncModel
}>()

const emit = defineEmits<{
	compositionApplyTemplate: []
	compositionNewDraft: []
	compositionNewSignature: []
	compositionNewTemplate: []
	compositionRefresh: []
	compositionRemoveDraft: []
	compositionRemoveSignature: []
	compositionRemoveTemplate: []
	compositionSaveDraft: []
	compositionSaveSignature: []
	compositionSaveTemplate: []
	compositionSelectConnection: [connectionId: string]
	compositionSelectDraft: [draftId: string]
	compositionSelectSignature: [signatureId: string]
	compositionSelectTemplate: [templateId: string]
	compositionUpdateDraft: [patch: MailDraftEditorPatch]
	compositionUpdateSignature: [patch: MailSignatureEditorPatch]
	compositionUpdateTemplate: [patch: MailTemplateEditorPatch]
	compositionUseSignature: [signatureId: string]
	deliver: []
	flagRefreshStatus: []
	flagSetRead: [targetValue: boolean]
	flagSetStarred: [targetValue: boolean]
	locationArchive: []
	locationMove: []
	locationRefreshStatus: []
	locationRestore: []
	locationSelectTargetFolder: [folderId: string]
	locationTrash: []
	permanentDelete: []
	permanentDeleteRefreshStatus: []
	permanentDeleteSetConfirmed: [confirmed: boolean]
	loadMoreFolders: []
	loadMoreMessages: []
	loadMoreThreads: []
	readRefresh: []
	refreshStatus: []
	selectFolder: [folderId: string]
	selectMessage: [messageId: string]
	selectThread: [providerThreadId: string]
	sync: []
	syncHealthLoadMore: []
	syncHealthRefresh: []
	selectSyncHealthConnection: [connectionId: string]
	updateOperationId: [value: string]
}>()

const composeOpen = ref(false)
const inspectorVisible = useResponsiveWorkspaceInspector()
const searchQuery = ref('')
const syncHealthOpen = ref(false)
</script>

<template>
	<section class="mail-operational-page">
		<MailWorkspaceToolbar
			:read-model="readModel"
			:search-query="searchQuery"
			:sync-model="syncModel"
			@compose="composeOpen = true"
			@refresh="emit('readRefresh')"
			@show-sync-health="syncHealthOpen = true"
			@sync="emit('sync')"
			@toggle-inspector="inspectorVisible = !inspectorVisible"
			@update-search="searchQuery = $event"
		/>

		<div class="mail-workspace-shell" :class="{ 'inspector-hidden': !inspectorVisible }">
			<MailWorkspaceList
				:model="readModel"
				:search-query="searchQuery"
				@load-more="emit('loadMoreMessages')"
				@select-folder="emit('selectFolder', $event)"
				@select-message="emit('selectMessage', $event)"
				@select-thread="emit('selectThread', $event)"
			/>
			<MailWorkspaceReader
				:body-content-status="bodyContentStatus"
				:body-content-status-message="bodyContentStatusMessage"
				:body-text="bodyText"
				:body-format="bodyFormat"
				:detail="readModel.detail"
				:inspector-visible="inspectorVisible"
				@toggle-inspector="inspectorVisible = !inspectorVisible"
			/>
			<MailWorkspaceInspector
				v-if="inspectorVisible"
				:detail="readModel.detail"
				:flag-model="flagModel"
				:location-model="locationModel"
				:permanent-delete-model="permanentDeleteModel"
				@close="inspectorVisible = false"
				@flag-refresh-status="emit('flagRefreshStatus')"
				@flag-set-read="emit('flagSetRead', $event)"
				@flag-set-starred="emit('flagSetStarred', $event)"
				@location-archive="emit('locationArchive')"
				@location-move="emit('locationMove')"
				@location-refresh-status="emit('locationRefreshStatus')"
				@location-restore="emit('locationRestore')"
				@location-select-target-folder="emit('locationSelectTargetFolder', $event)"
				@location-trash="emit('locationTrash')"
				@permanent-delete="emit('permanentDelete')"
				@permanent-delete-refresh-status="emit('permanentDeleteRefreshStatus')"
				@permanent-delete-set-confirmed="emit('permanentDeleteSetConfirmed', $event)"
			/>
		</div>

		<div v-if="syncHealthOpen" class="mail-workspace-dialog" role="dialog" aria-modal="true" aria-label="Mail sync health">
			<button type="button" class="mail-workspace-dialog__backdrop" aria-label="Close sync health" @click="syncHealthOpen = false" />
			<section class="mail-workspace-dialog__surface">
				<header>
					<div><span>Mail operations</span><h2>Sync health</h2></div>
					<button type="button" aria-label="Close sync health" @click="syncHealthOpen = false">×</button>
				</header>
				<MailSyncHealthPanel
					:model="syncHealthModel"
					@load-more="emit('syncHealthLoadMore')"
					@refresh="emit('syncHealthRefresh')"
					@select-connection="emit('selectSyncHealthConnection', $event)"
				/>
			</section>
		</div>

		<div v-if="composeOpen" class="mail-workspace-dialog" role="dialog" aria-modal="true" aria-label="Compose mail">
			<button type="button" class="mail-workspace-dialog__backdrop" aria-label="Close compose" @click="composeOpen = false" />
			<section class="mail-workspace-dialog__surface mail-workspace-dialog__surface--compose">
				<header>
					<div><span>New message</span><h2>Compose</h2></div>
					<button type="button" aria-label="Close compose" @click="composeOpen = false">×</button>
				</header>
				<MailCompositionPanel
					:model="compositionModel"
					:can-deliver="deliveryModel.canDeliver"
					:delivery-busy="deliveryModel.busy"
					@apply-template="emit('compositionApplyTemplate')"
					@deliver="emit('deliver')"
					@new-draft="emit('compositionNewDraft')"
					@new-signature="emit('compositionNewSignature')"
					@new-template="emit('compositionNewTemplate')"
					@refresh="emit('compositionRefresh')"
					@remove-draft="emit('compositionRemoveDraft')"
					@remove-signature="emit('compositionRemoveSignature')"
					@remove-template="emit('compositionRemoveTemplate')"
					@save-draft="emit('compositionSaveDraft')"
					@save-signature="emit('compositionSaveSignature')"
					@save-template="emit('compositionSaveTemplate')"
					@select-connection="emit('compositionSelectConnection', $event)"
					@select-draft="emit('compositionSelectDraft', $event)"
					@select-signature="emit('compositionSelectSignature', $event)"
					@select-template="emit('compositionSelectTemplate', $event)"
					@update-draft="emit('compositionUpdateDraft', $event)"
					@update-signature="emit('compositionUpdateSignature', $event)"
					@update-template="emit('compositionUpdateTemplate', $event)"
					@use-signature="emit('compositionUseSignature', $event)"
				/>
				<MailDeliveryPanel
					:model="deliveryModel"
					@refresh-status="emit('refreshStatus')"
					@update-operation-id="emit('updateOperationId', $event)"
				/>
			</section>
		</div>
	</section>
</template>
