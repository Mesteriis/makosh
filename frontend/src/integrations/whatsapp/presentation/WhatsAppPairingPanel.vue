<script setup lang="ts">
import type { ClientModuleBootstrapV1 } from '../../../gen/makosh/gateway/v1/client_bootstrap_pb'
import Icon from '../../../shared/ui/Icon.vue'
import { useWhatsAppPairing } from '../linking/useWhatsAppPairing'
import './whatsAppPairingPanel.css'

const props = defineProps<{ module: ClientModuleBootstrapV1 | null }>()
const pairing = useWhatsAppPairing(() => props.module)
</script>

<template>
	<section class="whatsapp-pairing">
		<div class="whatsapp-pairing__icon">
			<Icon icon="tabler:qrcode" />
		</div>
		<div class="whatsapp-pairing__copy">
			<small>Provider authorization</small>
			<h3>WhatsApp QR pairing</h3>
			<p>
				The QR code is rendered by WhatsApp Web inside an owner-visible desktop window.
				Макошь does not copy it into Settings or persist session material.
			</p>
			<p
				v-if="pairing.message.value || !pairing.nativeHostAvailable"
				:class="`whatsapp-pairing__message--${pairing.messageTone.value}`"
				aria-live="polite"
			>
				{{ pairing.message.value || 'Open the desktop shell to display the real WhatsApp QR code.' }}
			</p>
		</div>
		<button
			type="button"
			:disabled="!pairing.canOpen.value || pairing.busy.value"
			@click="pairing.open"
		>
			<Icon :icon="pairing.busy.value ? 'tabler:loader-2' : 'tabler:external-link'" />
			{{ pairing.busy.value ? 'Opening…' : 'Open WhatsApp QR' }}
		</button>
	</section>
</template>
