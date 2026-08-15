<script setup lang="ts">
import { computed, ref } from 'vue'
import type { ClientModuleBootstrapV1 } from '../../../gen/makosh/gateway/v1/client_bootstrap_pb'
import Icon from '../../../shared/ui/Icon.vue'
import Steps from '../../../shared/ui/Steps.vue'
import '../../../shared/ui/settings/integrationAccountSetupCard.css'
import '../../../shared/ui/settings/providerAccountWizard.css'
import { useTelegramAccountSetup } from '../setup/useTelegramAccountSetup'
import TelegramQrPairingPanel from './TelegramQrPairingPanel.vue'
import './telegramAccountSetupPanel.css'

const props = defineProps<{ module: ClientModuleBootstrapV1 | null }>()
const emit = defineEmits<{ completed: [] }>()
const setup = useTelegramAccountSetup(() => props.module)
const open = ref(false)
const step = ref(1)
const qrStartRequest = ref(0)
const authorizationState = ref('unknown')
const steps = [
	{
		title: 'Telegram application',
		description: 'The installed application identity is sealed before provider authorization starts.',
	},
	{
		title: 'Scan the Telegram QR',
		description: 'Use Telegram → Settings → Devices → Link Desktop Device.',
	},
	{
		title: 'Account connected',
		description: 'Telegram owns its provider session; Макошь receives the admitted operational surface.',
	},
]
const canAdvance = computed(() => {
	if (step.value === 1) return setup.configured.value || setup.canSubmit.value
	if (step.value === 2) return authorizationState.value === 'ready'
	return true
})

async function openWizard(): Promise<void> {
	open.value = true
	if (setup.configured.value) {
		step.value = 2
		qrStartRequest.value += 1
		return
	}
	step.value = 1
	await setup.prepareDevelopmentCredentials()
}

async function handleNext(nextStep: number): Promise<void> {
	if (nextStep !== 2) return
	if (!setup.configured.value && !await setup.submit()) {
		step.value = 1
		return
	}
	qrStartRequest.value += 1
	emit('completed')
}

function finish(): void {
	open.value = false
}

function handleAuthorizationState(state: string): void {
	authorizationState.value = state
	if (state === 'ready' && step.value === 2) step.value = 3
}
</script>

<template>
	<section class="integration-account-setup" data-provider-tone="telegram" data-auth-method="qr">
		<header class="integration-account-setup__header">
			<span class="integration-account-setup__icon"><Icon icon="tabler:brand-telegram" /></span>
			<div>
				<small>Provider account</small>
				<h3>Connect Telegram user</h3>
				<p>Application credentials are sealed into Vault as installation prerequisites; account authorization itself starts only from the provider QR.</p>
			</div>
			<strong>User account</strong>
		</header>

		<div class="provider-account-wizard__launcher">
			<div>
				<strong>Telegram QR authorization</strong>
				<p>Provider QR is the primary account authorization surface. No phone/SMS or bot login.</p>
			</div>
			<button type="button" :disabled="!module?.settings" @click="openWizard">
				<Icon icon="tabler:qrcode" />
				Connect with QR
			</button>
		</div>
	</section>

	<Steps
		v-model:open="open"
		v-model:step="step"
		:step-count="3"
		:steps="steps"
		title="Connect Telegram"
		description="Telegram user authorization is QR-only. Application credentials never authorize the user account."
		finish-label="Done"
		:can-advance="canAdvance"
		:busy="setup.busy.value"
		size="lg"
		content-class="telegram-account-wizard"
		@next="handleNext"
		@finish="finish"
	>
		<template #step-1>
			<div v-if="setup.configured.value" class="provider-account-wizard__status provider-account-wizard__status--success">
				<h4>Telegram application is configured</h4>
				<p>The API ID is effective and both credential purposes are sealed by Vault.</p>
			</div>
			<form v-else class="provider-account-wizard__form" @submit.prevent>
				<label>
					<span>Local account ID</span>
					<input v-model="setup.accountId.value" required maxlength="128" placeholder="personal-telegram">
				</label>
				<label>
					<span>Display name</span>
					<input v-model="setup.displayName.value" required maxlength="128" placeholder="Personal Telegram">
				</label>
				<label class="wide">
					<span>Telegram API ID</span>
					<input
						v-model="setup.apiId.value"
						required
						inputmode="numeric"
						pattern="[0-9]+"
						placeholder="123456"
						:readonly="setup.developmentCredentialsAvailable.value"
					>
				</label>
				<label v-if="!setup.developmentCredentialsAvailable.value" class="wide">
					<span>Telegram API hash</span>
					<input v-model="setup.apiHash.value" required type="password" autocomplete="new-password">
				</label>
				<p class="provider-account-wizard__notice wide">
					<template v-if="setup.developmentCredentialsAvailable.value">
						Development credentials were loaded by the native host. The API hash never enters browser JavaScript and is sealed directly into Vault.
					</template>
					<template v-else>
						API ID/hash identify the Telegram application, not the user login. The API hash
						is sealed by Vault; account authorization remains QR-only. Bot tokens are intentionally not part of this contract.
					</template>
				</p>
				<p
					v-if="setup.message.value"
					class="provider-account-wizard__notice wide"
					:class="`provider-account-wizard__status--${setup.messageTone.value}`"
					aria-live="polite"
				>
					{{ setup.message.value }}
				</p>
			</form>
		</template>

		<template #step-2>
			<div data-testid="telegram-qr-primary">
				<TelegramQrPairingPanel
					:module="module"
					:start-request="qrStartRequest"
					embedded
					@state-change="handleAuthorizationState"
				/>
			</div>
		</template>

		<template #step-3>
			<div class="provider-account-wizard__status provider-account-wizard__status--success">
				<h4>Telegram is connected</h4>
				<p>The provider session is ready. Chats, contacts and media remain owned by the Telegram integration.</p>
				<p>Close the wizard to continue in Макошь.</p>
			</div>
		</template>
	</Steps>
</template>
