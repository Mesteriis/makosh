<script setup lang="ts">
import Icon from '../../../shared/ui/Icon.vue'
import './whatsAppPairingPanel.css'

withDefaults(defineProps<{
	busy?: boolean
	message?: string
	messageTone?: 'neutral' | 'success' | 'error'
	nativeHostAvailable?: boolean
	canOpen?: boolean
}>(), {
	busy: false,
	message: '',
	messageTone: 'neutral',
	nativeHostAvailable: true,
	canOpen: true,
})

defineEmits<{ open: [] }>()
</script>

<template>
	<section class="whatsapp-pairing" data-auth-method="qr">
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
				v-if="message || !nativeHostAvailable"
				:class="`whatsapp-pairing__message--${messageTone}`"
				aria-live="polite"
			>
				{{ message || 'Open the desktop shell to display the real WhatsApp QR code.' }}
			</p>
		</div>
		<button
			type="button"
			:disabled="!canOpen || busy"
			@click="$emit('open')"
		>
			<Icon :icon="busy ? 'tabler:loader-2' : 'tabler:external-link'" />
			{{ busy ? 'Opening…' : 'Open WhatsApp QR' }}
		</button>
	</section>
</template>
