<script setup lang="ts">
import { watch } from 'vue'
import type { ClientModuleBootstrapV1 } from '../../../gen/hermes/gateway/v1/client_bootstrap_pb'
import type { ProviderAccountNavigationSnapshot } from '../../../shared/ui/shell/providerAccountNavigation'
import MailOperationalPage from '../presentation/MailOperationalPage.vue'
import { useMailComposition } from '../queries/useMailComposition'
import { useMailDelivery } from '../queries/useMailDelivery'
import { useMailAccountConnections } from '../queries/useMailAccountConnections'
import { useMailOperationalRead } from '../queries/useMailOperationalRead'
import { useMailMessageFlags } from '../queries/useMailMessageFlags'
import { useMailMessageLocation } from '../queries/useMailMessageLocation'
import { useMailMessagePermanentDelete } from '../queries/useMailMessagePermanentDelete'
import { useMailSync } from '../queries/useMailSync'
import { useMailSyncHealth } from '../queries/useMailSyncHealth'
import { mailAccountNavigation } from '../presentation/mailAccountNavigation'

const props = defineProps<{
	canCompose: boolean
	canComposeQuery: boolean
	canDeliver: boolean
	canMutateFlags: boolean
	canQuery: boolean
	canQueryAccounts: boolean
	canQueryFlagStatus: boolean
	canMutateLocation: boolean
	canQueryLocationStatus: boolean
	canPermanentlyDelete: boolean
	canQueryPermanentDeleteStatus: boolean
	canSync: boolean
	canSyncHealth: boolean
	bodyContentStatus: 'idle' | 'loading' | 'ready' | 'unavailable'
	bodyContentStatusMessage: string
	bodyText: string
	bodyFormat: 'text' | 'html'
	modules: readonly ClientModuleBootstrapV1[]
	navigationAccountId?: string
}>()
const emit = defineEmits<{
	accountNavigationChange: [snapshot: ProviderAccountNavigationSnapshot]
	messageEvidenceChange: [evidenceId: Uint8Array | undefined]
}>()
let accountNavigationLoading = true

const accountConnections = useMailAccountConnections({
	canQuery: () => props.canQueryAccounts,
	modules: () => props.modules,
})
const composition = useMailComposition({
	canMutate: () => props.canCompose,
	canQuery: () => props.canComposeQuery,
	connections: () => accountConnections.connections.value,
})
const delivery = useMailDelivery({
	canDeliver: () => props.canDeliver,
	connectionId: () => accountConnections.connections.value.find(
		(connection) => connection.connectionId === composition.connectionId(),
	)?.deliveryReady
		? composition.connectionId()
		: '',
})
const read = useMailOperationalRead({
	canQuery: () => props.canQuery,
	connections: () => accountConnections.connections.value,
})
const sync = useMailSync({
	canSync: () => props.canSync,
	connectionId: () => accountConnections.connections.value.find(
		(connection) => connection.connectionId === read.model.value.selectedConnectionId,
	)?.syncReady
		? read.model.value.selectedConnectionId
		: '',
})
const messageFlags = useMailMessageFlags({
	canMutate: () => props.canMutateFlags,
	canQueryStatus: () => props.canQueryFlagStatus,
	selection: () => {
		const detail = read.model.value.detail
		const connectionId = read.model.value.selectedConnectionId
		if (!detail || !connectionId) return null
		return {
			connectionId,
			messageId: detail.id,
			isRead: detail.isRead,
			isStarred: detail.isStarred,
		}
	},
	refreshProjection: read.refresh,
})
const messageLocation = useMailMessageLocation({
	canMutate: () => props.canMutateLocation,
	canQueryStatus: () => props.canQueryLocationStatus,
	selection: () => {
		const detail = read.model.value.detail
		const connectionId = read.model.value.selectedConnectionId
		if (!detail || !connectionId) return null
		return {
			connectionId,
			messageId: detail.id,
			isTrashed: detail.isTrashed,
			folders: read.model.value.folders,
		}
	},
	refreshProjection: read.refresh,
})
const messagePermanentDelete = useMailMessagePermanentDelete({
	canDelete: () => props.canPermanentlyDelete,
	canQueryStatus: () => props.canQueryPermanentDeleteStatus,
	selection: () => {
		const detail = read.model.value.detail
		const connectionId = read.model.value.selectedConnectionId
		if (!detail || !connectionId) return null
		return {
			connectionId,
			messageId: detail.id,
			projectionRevision: BigInt(detail.projectionRevision),
			isTrashed: detail.isTrashed,
		}
	},
	refreshProjection: read.refresh,
})
const syncHealth = useMailSyncHealth({
	canQuery: () => props.canSyncHealth,
	connections: () => accountConnections.connections.value,
})

watch(
	() => [
		props.canComposeQuery,
		props.canQuery,
		props.canQueryAccounts,
		props.canSyncHealth,
		props.modules,
	] as const,
	() => { void reconcileAccountConsumers() },
	{ immediate: true },
)

async function reconcileAccountConsumers(): Promise<void> {
	accountNavigationLoading = true
	emitAccountNavigation()
	try {
		await accountConnections.refresh()
	} finally {
		try {
			await Promise.all([
				composition.reconcile(),
				read.reconcile(),
				syncHealth.reconcile(),
			])
		} finally {
			accountNavigationLoading = false
			emitAccountNavigation()
		}
	}
}

function emitAccountNavigation(): void {
	emit('accountNavigationChange', mailAccountNavigation(
		accountConnections.connections.value,
		read.model.value.selectedConnectionId,
		accountNavigationLoading,
	))
}

watch(
	() => props.navigationAccountId,
	(connectionId) => {
		if (
			connectionId === undefined
			|| connectionId === read.model.value.selectedConnectionId
			|| !connectionId
		) return
		void read.selectConnection(connectionId).finally(emitAccountNavigation)
	},
)

watch(
	() => read.model.value.selectedConnectionId,
	() => emitAccountNavigation(),
)

watch(
	() => read.model.value.detail?.observationAnchorId,
	(evidenceId) => emit('messageEvidenceChange', evidenceId),
	{ immediate: true },
)
</script>

<template>
	<MailOperationalPage
		:composition-model="composition.model.value"
		:body-content-status="bodyContentStatus"
		:body-content-status-message="bodyContentStatusMessage"
		:body-text="bodyText"
		:body-format="bodyFormat"
		:delivery-model="delivery.model.value"
		:flag-model="messageFlags.model.value"
		:location-model="messageLocation.model.value"
		:permanent-delete-model="messagePermanentDelete.model.value"
		:read-model="read.model.value"
		:sync-health-model="syncHealth.model.value"
		:sync-model="sync.model.value"
		@composition-apply-template="composition.applyTemplate"
		@composition-new-draft="composition.newDraft"
		@composition-new-signature="composition.newSignature"
		@composition-new-template="composition.newTemplate"
		@composition-refresh="composition.refresh"
		@composition-remove-draft="composition.removeDraft"
		@composition-remove-signature="composition.removeSignature"
		@composition-remove-template="composition.removeTemplate"
		@composition-save-draft="composition.saveDraft"
		@composition-save-signature="composition.saveSignature"
		@composition-save-template="composition.saveTemplate"
		@composition-select-connection="composition.selectConnection"
		@composition-select-draft="composition.selectDraft"
		@composition-select-signature="composition.selectSignature"
		@composition-select-template="composition.selectTemplate"
		@composition-update-draft="composition.updateDraft"
		@composition-update-signature="composition.updateSignature"
		@composition-update-template="composition.updateTemplate"
		@composition-use-signature="composition.useSignature"
		@deliver="delivery.deliver(composition.deliveryInput.value)"
		@load-more-folders="read.loadMoreFolders"
		@load-more-messages="read.loadMoreMessages"
		@load-more-threads="read.loadMoreThreads"
		@read-refresh="read.refresh"
		@flag-refresh-status="messageFlags.refreshStatus"
		@flag-set-read="messageFlags.setRead"
		@flag-set-starred="messageFlags.setStarred"
		@location-archive="messageLocation.archive"
		@location-move="messageLocation.move"
		@location-refresh-status="messageLocation.refreshStatus"
		@location-restore="messageLocation.restore"
		@location-select-target-folder="messageLocation.selectTargetFolder"
		@location-trash="messageLocation.trash"
		@permanent-delete="messagePermanentDelete.permanentlyDelete"
		@permanent-delete-refresh-status="messagePermanentDelete.refreshStatus"
		@permanent-delete-set-confirmed="messagePermanentDelete.setConfirmed"
		@refresh-status="delivery.refreshStatus"
		@select-folder="read.selectFolder"
		@select-message="read.selectMessage"
		@select-thread="read.selectThread"
		@sync="sync.sync"
		@sync-health-load-more="syncHealth.loadMore"
		@sync-health-refresh="syncHealth.refresh"
		@select-sync-health-connection="syncHealth.selectConnection"
		@update-operation-id="delivery.updateOperationId"
	/>
</template>
