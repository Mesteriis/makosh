<script setup lang="ts">
import type { ClientModuleBootstrapV1 } from '../../../gen/makosh/gateway/v1/client_bootstrap_pb'
import Icon from '../../../shared/ui/Icon.vue'
import { useTelegramQrPairing } from '../linking/useTelegramQrPairing'
import './telegramQrPairingPanel.css'

const props = defineProps<{
	module: ClientModuleBootstrapV1 | null
	startRequest?: number
	embedded?: boolean
}>()
const pairing = useTelegramQrPairing(
	() => props.module,
	() => props.startRequest ?? 0,
)
</script>

<template>
	<section
		class="telegram-qr-pairing"
		:class="{ 'telegram-qr-pairing--embedded': embedded }"
	>
		<header>
			<div>
				<small>Provider authorization</small>
				<h3>Telegram user QR login</h3>
				<p>The QR is generated locally from the short-lived TDLib login link.</p>
			</div>
			<strong>{{ pairing.state.value }}</strong>
		</header>

		<div v-if="pairing.qrDataUrl.value" class="telegram-qr-pairing__artifact">
			<img
				:src="pairing.qrDataUrl.value"
				alt="Telegram authorization QR code"
				width="280"
				height="280"
			>
		</div>

		<form
			v-if="pairing.state.value === 'waiting_password'"
			class="telegram-qr-pairing__password"
			@submit.prevent="pairing.submitPassword"
		>
			<label for="telegram-settings-authorization-password">
				2FA password
				<small v-if="pairing.passwordHint.value">{{ pairing.passwordHint.value }}</small>
			</label>
			<input
				id="telegram-settings-authorization-password"
				v-model="pairing.password.value"
				type="password"
				autocomplete="current-password"
				required
			>
			<button type="submit" :disabled="pairing.busy.value || !pairing.password.value.trim()">
				Continue
			</button>
		</form>

		<footer>
			<p
				:class="`telegram-qr-pairing__message--${pairing.messageTone.value}`"
				aria-live="polite"
			>
				{{ pairing.message.value || (!pairing.admitted.value ? 'Telegram authorization capability is not admitted.' : !pairing.configured.value ? 'Configure the Telegram account before starting QR authorization.' : 'Refresh to start or resume QR authorization.') }}
			</p>
			<button
				type="button"
				:disabled="!pairing.canRefresh.value || pairing.busy.value"
				@click="pairing.refresh"
			>
				<Icon :icon="pairing.busy.value ? 'tabler:loader-2' : 'tabler:qrcode'" />
				{{ pairing.busy.value ? 'Refreshing…' : 'Refresh Telegram QR' }}
			</button>
		</footer>
	</section>
</template>
