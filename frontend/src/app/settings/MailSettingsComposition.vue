<script setup lang="ts">
import type { ClientModuleBootstrapV1 } from '../../gen/makosh/gateway/v1/client_bootstrap_pb'
import MailSettingsPanel from '../../integrations/mail/presentation/MailSettingsPanel.vue'

defineProps<{
	mailModule: ClientModuleBootstrapV1 | null
}>()
const emit = defineEmits<{ changed: [] }>()

function refreshMailState(): void {
	emit('changed')
}
</script>

<template>
	<div class="mail-settings-owner">
		<MailSettingsPanel :module="mailModule" @changed="refreshMailState" />
	</div>
</template>
