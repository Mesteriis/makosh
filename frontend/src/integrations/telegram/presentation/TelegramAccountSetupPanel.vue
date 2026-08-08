<script setup lang="ts">
import { computed, ref } from 'vue'
import type { ClientModuleBootstrapV1 } from '../../../gen/makosh/gateway/v1/client_bootstrap_pb'
import Icon from '../../../shared/ui/Icon.vue'
import Steps from '../../../shared/ui/Steps.vue'
import '../../../shared/ui/settings/integrationAccountSetupCard.css'
import '../../../shared/ui/settings/providerAccountWizard.css'
import { useTelegramAccountSetup } from '../setup/useTelegramAccountSetup'
import TelegramQrPairingPanel from './TelegramQrPairingPanel.vue'

const props = defineProps<{ module: ClientModuleBootstrapV1 | null }>()
const emit = defineEmits<{ completed: [] }>()
const setup = useTelegramAccountSetup(() => props.module)
const open = ref(false)
const step = ref(1)
const qrStartRequest = ref(0)
const steps = [
	{
		title: 'Telegram user account',
		description: 'Макошь accepts a Telegram user session here. Bot tokens are intentionally not part of this contract.',
	},
	{
		title: 'Scan provider QR',
		description: 'The QR is generated locally from the short-lived authorization link returned by TDLib.',
	},
]
const canAdvance = computed(() =>
	step.value === 1
		? setup.canSubmit.value && !setup.busy.value
		: !setup.busy.value,
)

function openWizard(): void {
	step.value = 1
	open.value = true
}

function resumeQrAuthorization(): void {
	step.value = 2
	open.value = true
	qrStartRequest.value += 1
}

async function handleNext(nextStep: number): Promise<void> {
	if (nextStep !== 2) return
	if (!await setup.submit()) {
		step.value = 1
		return
	}
	qrStartRequest.value += 1
	emit('completed')
}

function finish(): void {
	open.value = false
}
</script>

<template>
	<section class="integration-account-setup" data-provider-tone="telegram">
		<header class="integration-account-setup__header">
			<span class="integration-account-setup__icon"><Icon icon="tabler:brand-telegram" /></span>
			<div>
				<small>Provider account</small>
				<h3>Connect Telegram user</h3>
				<p>Save API credentials in Vault, then authorize the real TDLib session with Telegram QR.</p>
			</div>
			<strong>User account</strong>
		</header>

		<div class="provider-account-wizard__launcher">
			<div>
				<strong>Telegram account wizard</strong>
				<p>No bot setup and no generated placeholder QR payload.</p>
			</div>
			<button type="button" :disabled="!module?.settings" @click="openWizard">
				<Icon icon="tabler:qrcode" />
				Add account
			</button>
		</div>
		<div v-if="setup.configured.value" class="provider-account-wizard__launcher">
			<div>
				<strong>Telegram QR authorization</strong>
				<p>Resume the provider-issued QR flow without replacing existing Vault credentials.</p>
			</div>
			<button type="button" @click="resumeQrAuthorization">
				<Icon icon="tabler:qrcode" />
				Continue QR authorization
			</button>
		</div>
	</section>

	<Steps
		v-model:open="open"
		v-model:step="step"
		:step-count="2"
		:steps="steps"
		title="Connect Telegram user"
		description="Telegram integration owns API credentials, TDLib authorization and provider session state."
		finish-label="Done"
		:can-advance="canAdvance"
		:busy="setup.busy.value"
		size="lg"
		content-class="telegram-account-wizard"
		@next="handleNext"
		@finish="finish"
	>
		<template #step-1>
			<div class="provider-account-wizard__form">
				<label>
					<span>Local account ID</span>
					<input v-model="setup.accountId.value" required maxlength="128" placeholder="personal-telegram">
				</label>
				<label>
					<span>Display name</span>
					<input v-model="setup.displayName.value" required maxlength="128" placeholder="Personal Telegram">
				</label>
				<label>
					<span>Telegram API ID</span>
					<input v-model="setup.apiId.value" required inputmode="numeric" pattern="[0-9]+" placeholder="123456">
				</label>
				<label>
					<span>Telegram API hash</span>
					<input v-model="setup.apiHash.value" required type="password" autocomplete="new-password">
				</label>
				<p class="provider-account-wizard__notice">
					Get the API ID and API hash from Telegram’s application portal. The API hash is
					sealed by Vault; the TDLib session store remains integration-owned.
				</p>
				<p
					v-if="setup.message.value"
					class="provider-account-wizard__notice"
					:class="`provider-account-wizard__status--${setup.messageTone.value}`"
					aria-live="polite"
				>
					{{ setup.message.value }}
				</p>
			</div>
		</template>

		<template #step-2>
			<TelegramQrPairingPanel :module="module" :start-request="qrStartRequest" embedded />
		</template>
	</Steps>
</template>
