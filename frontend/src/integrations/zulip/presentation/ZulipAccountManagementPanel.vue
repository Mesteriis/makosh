<script setup lang="ts">
import { onMounted } from 'vue'
import type { ClientModuleBootstrapV1 } from '../../../gen/makosh/gateway/v1/client_bootstrap_pb'
import Icon from '../../../shared/ui/Icon.vue'
import IntegrationAccountLifecycleCard from '../../../shared/ui/settings/IntegrationAccountLifecycleCard.vue'
import { useZulipAccountManagement } from '../account-management/useZulipAccountManagement'

const props = defineProps<{ module: ClientModuleBootstrapV1 | null }>()
const management = useZulipAccountManagement(() => props.module)

onMounted(() => void management.refresh())

function retire(): void {
	const accountId = management.status.value?.accountId
	if (accountId && window.confirm(`Retire Zulip account ${accountId}? Provider access will be fenced.`)) {
		void management.retire()
	}
}
</script>

<template>
	<IntegrationAccountLifecycleCard
		eyebrow="Account lifecycle"
		title="Manage Zulip account"
		description="Projection health and API-key custody stay inside the Zulip integration."
		tone="zulip"
		icon="tabler:brand-zulip"
		:account-state="management.stateLabel.value"
		:busy="management.busy.value"
		:message="management.message.value || (!management.secureHostAvailable ? 'API-key rotation requires the desktop shell or root make dev.' : '')"
		:message-tone="management.messageTone.value"
	>
		<template #summary>
			<div><small>Account ID</small><strong>{{ management.accountId.value || 'Not configured' }}</strong></div>
			<div><small>Projection</small><strong>{{ management.status.value?.projectionReady ? 'Ready' : 'Not ready' }}</strong></div>
			<div><small>Event sequence</small><strong>{{ management.status.value?.latestEventSequence ?? '—' }}</strong></div>
			<div><small>Binding revision</small><strong>{{ management.status.value?.bindingRevision ?? '—' }}</strong></div>
		</template>

		<label v-if="management.canRotate.value">
			<span>New API key</span>
			<input v-model="management.apiKey.value" type="password" autocomplete="new-password">
		</label>

		<template #actions>
			<button type="button" :disabled="management.busy.value || !management.canQuery.value" @click="management.refresh">
				<Icon icon="tabler:refresh" /> Refresh
			</button>
			<button
				v-if="management.canRotate.value"
				class="primary"
				type="button"
				:disabled="management.busy.value || !management.apiKey.value"
				@click="management.rotateApiKey"
			>
				<Icon icon="tabler:key" /> Rotate API key
			</button>
			<button
				v-if="management.canRetire.value"
				class="danger"
				type="button"
				:disabled="management.busy.value"
				@click="retire"
			>
				<Icon icon="tabler:archive" /> Retire
			</button>
		</template>
	</IntegrationAccountLifecycleCard>
</template>
