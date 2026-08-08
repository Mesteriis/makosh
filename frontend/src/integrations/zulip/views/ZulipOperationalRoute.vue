<script setup lang="ts">
import { watch } from 'vue'

import type { ClientModuleBootstrapV1 } from '../../../gen/makosh/gateway/v1/client_bootstrap_pb'
import ZulipOperationalPage from '../presentation/ZulipOperationalPage.vue'
import {
	zulipOperationalAccountFingerprint,
} from '../queries/zulipOperationalAccounts'
import { useZulipOperationalPage } from '../queries/useZulipOperationalPage'
import { useZulipOperationalRead } from '../queries/useZulipOperationalRead'
import { useZulipOperationalReplay } from '../queries/useZulipOperationalReplay'

const props = defineProps<{
	canCommand: boolean
	canQuery: boolean
	canReplay: boolean
	modules: readonly ClientModuleBootstrapV1[]
}>()
const surface = useZulipOperationalPage(() => props.canCommand)
const read = useZulipOperationalRead({
	canQuery: () => props.canQuery,
	modules: () => props.modules,
})
const replay = useZulipOperationalReplay({
	canReplay: () => props.canReplay,
	modules: () => props.modules,
})

watch(
	() => `${props.canQuery}:${props.canReplay}:${zulipOperationalAccountFingerprint(props.modules)}`,
	() => {
		void read.reconcile()
		void replay.reconcile()
	},
	{ immediate: true },
)
</script>

<template>
	<ZulipOperationalPage
		:model="surface.model.value"
		:read-model="read.model.value"
		:replay-model="replay.model.value"
		@load-more-conversations="read.loadMoreConversations"
		@load-more-events="read.loadMoreEvents"
		@load-more-messages="read.loadMoreMessages"
		@load-more-replay="replay.loadMore"
		@load-more-search-results="read.loadMoreSearchResults"
		@read-refresh="read.refresh"
		@refresh-status="surface.refreshStatus"
		@replay-refresh="replay.refresh"
		@search="read.search"
		@select-conversation="read.selectConversation"
		@select-read-account="read.selectAccount"
		@select-replay-account="replay.selectAccount"
		@send="surface.send"
		@update-destination="surface.updateDestination"
		@update-account-id="surface.updateAccountId"
		@update-stream="surface.updateStream"
		@update-topic="surface.updateTopic"
		@update-recipients="surface.updateRecipients"
		@update-content="surface.updateContent"
		@update-operation-id="surface.updateOperationId"
		@update-search-query="read.updateSearchQuery"
	/>
</template>
