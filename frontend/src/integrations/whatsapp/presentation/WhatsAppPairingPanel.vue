<script setup lang="ts">
import type { ClientModuleBootstrapV1 } from '../../../gen/makosh/gateway/v1/client_bootstrap_pb'
import { useWhatsAppPairing } from '../linking/useWhatsAppPairing'
import WhatsAppPairingView from './WhatsAppPairingView.vue'

const props = defineProps<{ module: ClientModuleBootstrapV1 | null }>()
const pairing = useWhatsAppPairing(() => props.module)
</script>

<template>
	<WhatsAppPairingView
		:busy="pairing.busy.value"
		:message="pairing.message.value"
		:message-tone="pairing.messageTone.value"
		:native-host-available="pairing.nativeHostAvailable"
		:can-open="pairing.canOpen.value"
		@open="pairing.open"
	/>
</template>
