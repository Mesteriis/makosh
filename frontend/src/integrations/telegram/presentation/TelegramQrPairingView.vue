<script setup lang="ts">
import Icon from '../../../shared/ui/Icon.vue'
import TelegramCloudPasswordForm from './TelegramCloudPasswordForm.vue'
import './telegramQrPairingPanel.css'

withDefaults(defineProps<{
	state: string
	qrDataUrl?: string
	password?: string
	passwordHint?: string
	busy?: boolean
	message?: string
	messageTone?: 'neutral' | 'success' | 'error'
	admitted?: boolean
	configured?: boolean
	canRefresh?: boolean
	embedded?: boolean
}>(), {
	qrDataUrl: '',
	password: '',
	passwordHint: '',
	busy: false,
	message: '',
	messageTone: 'neutral',
	admitted: true,
	configured: true,
	canRefresh: true,
	embedded: false,
})

defineEmits<{
	refresh: []
	submitPassword: []
	'update:password': [value: string]
}>()
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
			<strong>{{ state }}</strong>
		</header>
		<strong v-else class="telegram-qr-pairing__embedded-state">{{ state }}</strong>

		<div v-if="qrDataUrl && state !== 'waiting_password'" class="telegram-qr-pairing__artifact">
			<img
				:src="qrDataUrl"
				alt="Telegram authorization QR code"
				width="280"
				height="280"
			>
		</div>
		<div
			v-else-if="state !== 'ready' && state !== 'waiting_password'"
			class="telegram-qr-pairing__placeholder"
			data-testid="telegram-qr-placeholder"
		>
			<Icon icon="tabler:qrcode" size="5rem" />
			<strong>QR will appear here</strong>
			<small v-if="!configured">
				Save the Telegram application credentials below so TDLib can request it.
			</small>
			<small v-else>Requesting the short-lived provider QR from TDLib.</small>
		</div>

		<TelegramCloudPasswordForm
			v-if="state === 'waiting_password'"
			id="telegram-settings-cloud-password"
			:model-value="password"
			:hint="passwordHint"
			:busy="busy"
			:message="message"
			:message-tone="messageTone"
			@submit="$emit('submitPassword')"
			@update:model-value="$emit('update:password', $event)"
		/>

		<footer v-if="state !== 'waiting_password'">
			<p
				:class="`telegram-qr-pairing__message--${messageTone}`"
				aria-live="polite"
			>
				{{ message || (!admitted ? 'Telegram authorization capability is not admitted.' : !configured ? 'Telegram application credentials are required once before TDLib can issue the QR.' : 'Refresh to start or resume QR authorization.') }}
			</p>
			<button
				type="button"
				:disabled="!canRefresh || busy"
				@click="$emit('refresh')"
			>
				<Icon :icon="busy ? 'tabler:loader-2' : 'tabler:qrcode'" />
				{{ busy ? 'Refreshing…' : 'Refresh Telegram QR' }}
			</button>
		</footer>
	</section>
</template>
