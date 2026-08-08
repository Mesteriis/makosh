<script setup lang="ts">
import { watch } from 'vue'

import type { ClientModuleBootstrapV1 } from '../../../gen/makosh/gateway/v1/client_bootstrap_pb'
import WhatsAppOperationalPage from '../presentation/WhatsAppOperationalPage.vue'
import {
	whatsAppOperationalAccountFingerprint,
} from '../queries/whatsAppOperationalAccounts'
import { useWhatsAppOperationalPage } from '../queries/useWhatsAppOperationalPage'
import { useWhatsAppOperationalRead } from '../queries/useWhatsAppOperationalRead'
import { useWhatsAppOperationalReplay } from '../queries/useWhatsAppOperationalReplay'

const props = defineProps<{
	canQuery: boolean
	canReplay: boolean
	canSend: boolean
	modules: readonly ClientModuleBootstrapV1[]
}>()
const surface = useWhatsAppOperationalPage(() => props.canSend)
const read = useWhatsAppOperationalRead({
	canQuery: () => props.canQuery,
	modules: () => props.modules,
})
const replay = useWhatsAppOperationalReplay({
	canReplay: () => props.canReplay,
	modules: () => props.modules,
})

watch(
	() => `${props.canQuery}:${props.canReplay}:${whatsAppOperationalAccountFingerprint(props.modules)}`,
	() => {
		void read.reconcile()
		void replay.reconcile()
	},
	{ immediate: true },
)
</script>

<template>
	<WhatsAppOperationalPage
		:model="surface.model.value"
		:read-model="read.model.value"
		:replay-model="replay.model.value"
		@load-more-dialogs="read.loadMoreDialogs"
		@load-more-events="read.loadMoreEvents"
		@load-more-messages="read.loadMoreMessages"
		@load-more-participants="read.loadMoreParticipants"
		@load-more-replay="replay.loadMore"
		@load-more-search-results="read.loadMoreSearchResults"
		@read-refresh="read.refresh"
		@refresh-status="surface.refreshStatus"
		@replay-refresh="replay.refresh"
		@search="read.search"
		@select-read-account="read.selectAccount"
		@select-replay-account="replay.selectAccount"
		@select-dialog="read.selectDialog"
		@send="surface.send"
		@update-account-id="surface.updateAccountId"
		@update-provider-chat-id="surface.updateProviderChatId"
		@update-draft="surface.updateDraft"
		@update-operation-id="surface.updateOperationId"
		@update-search-query="read.updateSearchQuery"
	/>
</template>
