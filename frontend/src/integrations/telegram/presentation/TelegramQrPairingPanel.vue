<script setup lang="ts">
import { watch } from 'vue'
import type { ClientModuleBootstrapV1 } from '../../../gen/makosh/gateway/v1/client_bootstrap_pb'
import Icon from '../../../shared/ui/Icon.vue'
import { useTelegramQrPairing } from '../linking/useTelegramQrPairing'
import TelegramCloudPasswordForm from './TelegramCloudPasswordForm.vue'
import './telegramQrPairingPanel.css'

const props = defineProps<{
	module: ClientModuleBootstrapV1 | null
	startRequest?: number
	embedded?: boolean
}>()
const emit = defineEmits<{ stateChange: [state: string] }>()
const pairing = useTelegramQrPairing(
	() => props.module,
	() => props.startRequest ?? 0,
)

watch(pairing.state, (state) => emit('stateChange', state), { immediate: true })
</script>

<template>
	<section
		class="telegram-qr-pairing"
		:class="{ 'telegram-qr-pairing--embedded': embedded }"
	>
		<header v-if="!embedded">
			<div>
				<small>Provider authorization</small>
				<h3>Telegram user QR login</h3>
				<p>The QR is generated locally from the short-lived TDLib login link.</p>
			</div>
			<strong>{{ pairing.state.value }}</strong>
		</header>
		<strong v-else class="telegram-qr-pairing__embedded-state">{{ pairing.state.value }}</strong>

		<div v-if="pairing.qrDataUrl.value && pairing.state.value !== 'waiting_password'" class="telegram-qr-pairing__artifact">
			<img
				:src="pairing.qrDataUrl.value"
				alt="Telegram authorization QR code"
				width="280"
				height="280"
			>
		</div>
		<div
			v-else-if="pairing.state.value !== 'ready' && pairing.state.value !== 'waiting_password'"
			class="telegram-qr-pairing__placeholder"
			data-testid="telegram-qr-placeholder"
		>
			<Icon icon="tabler:qrcode" size="5rem" />
			<strong>QR will appear here</strong>
			<small v-if="!pairing.configured.value">
				Save the Telegram application credentials below so TDLib can request it.
			</small>
			<small v-else>Requesting the short-lived provider QR from TDLib.</small>
		</div>

		<TelegramCloudPasswordForm
			v-if="pairing.state.value === 'waiting_password'"
			id="telegram-settings-cloud-password"
			:model-value="pairing.password.value"
			:hint="pairing.passwordHint.value"
			:busy="pairing.busy.value"
			:message="pairing.message.value"
			:message-tone="pairing.messageTone.value"
			@submit="pairing.submitPassword"
			@update:model-value="pairing.password.value = $event"
		/>

		<footer v-if="pairing.state.value !== 'waiting_password'">
			<p
				:class="`telegram-qr-pairing__message--${pairing.messageTone.value}`"
				aria-live="polite"
			>
				{{ pairing.message.value || (!pairing.admitted.value ? 'Telegram authorization capability is not admitted.' : !pairing.configured.value ? 'Telegram application credentials are required once before TDLib can issue the QR.' : 'Refresh to start or resume QR authorization.') }}
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
