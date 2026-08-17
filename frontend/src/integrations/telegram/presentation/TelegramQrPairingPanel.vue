<script setup lang="ts">
import { watch } from 'vue'
import type { ClientModuleBootstrapV1 } from '../../../gen/makosh/gateway/v1/client_bootstrap_pb'
import { useTelegramQrPairing } from '../linking/useTelegramQrPairing'
import TelegramQrPairingView from './TelegramQrPairingView.vue'

const props = defineProps<{
	module: ClientModuleBootstrapV1 | null
	startRequest?: number
	configured?: boolean
	embedded?: boolean
}>()
const emit = defineEmits<{ stateChange: [state: string] }>()
const pairing = useTelegramQrPairing(
	() => props.module,
	() => props.startRequest ?? 0,
	() => props.configured ?? false,
)

watch(pairing.state, (state) => emit('stateChange', state), { immediate: true })
</script>

<template>
	<TelegramQrPairingView
		:state="pairing.state.value"
		:qr-data-url="pairing.qrDataUrl.value"
		:password="pairing.password.value"
		:password-hint="pairing.passwordHint.value"
		:busy="pairing.busy.value"
		:message="pairing.message.value"
		:message-tone="pairing.messageTone.value"
		:admitted="pairing.admitted.value"
		:configured="pairing.configured.value"
		:can-refresh="pairing.canRefresh.value"
		:embedded="embedded"
		@refresh="pairing.refresh"
		@submit-password="pairing.submitPassword"
		@update:password="pairing.password.value = $event"
	/>
</template>
