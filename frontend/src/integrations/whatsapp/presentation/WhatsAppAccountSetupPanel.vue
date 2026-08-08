<script setup lang="ts">
import type { ClientModuleBootstrapV1 } from '../../../gen/makosh/gateway/v1/client_bootstrap_pb'
import IntegrationAccountSetupCard from '../../../shared/ui/settings/IntegrationAccountSetupCard.vue'
import { useWhatsAppAccountSetup } from '../setup/useWhatsAppAccountSetup'

const props = defineProps<{ module: ClientModuleBootstrapV1 | null }>()
const setup = useWhatsAppAccountSetup(() => props.module)
</script>

<template>
	<IntegrationAccountSetupCard
		eyebrow="Provider account"
		title="Link WhatsApp"
		description="Creates an isolated account profile and starts only the approved hidden host bridge."
		tone="whatsapp"
		icon="tabler:brand-whatsapp"
		:account-state="setup.configured.value ? 'Configured' : 'No account'"
		submit-label="Create linking profile"
		:busy="setup.busy.value"
		:disabled="!setup.canSubmit.value"
		:message="setup.message.value"
		:message-tone="setup.messageTone.value"
		:expanded="!setup.configured.value"
		@submit="setup.submit"
	>
		<label class="wide">
			<span>Local account ID</span>
			<input
				v-model="setup.accountId.value"
				required
				maxlength="128"
				autocomplete="off"
				placeholder="personal-whatsapp"
			>
		</label>
	</IntegrationAccountSetupCard>
</template>
