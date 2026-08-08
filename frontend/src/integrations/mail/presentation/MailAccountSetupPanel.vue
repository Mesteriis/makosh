<script setup lang="ts">
import { computed, ref } from 'vue'
import type { ClientModuleBootstrapV1 } from '../../../gen/makosh/gateway/v1/client_bootstrap_pb'
import Icon from '../../../shared/ui/Icon.vue'
import Steps from '../../../shared/ui/Steps.vue'
import '../../../shared/ui/settings/integrationAccountSetupCard.css'
import '../../../shared/ui/settings/providerAccountWizard.css'
import { useMailAccountSetup } from '../setup/useMailAccountSetup'
import { useMailPendingSettingsActivation } from '../setup/useMailPendingSettingsActivation'

type MailProviderChoice = 'gmail' | 'icloud' | 'imap'

const props = defineProps<{ module: ClientModuleBootstrapV1 | null }>()
const emit = defineEmits<{ completed: [] }>()
const setup = useMailAccountSetup(() => props.module)
const pendingActivation = useMailPendingSettingsActivation(() => props.module)
const open = ref(false)
const step = ref(1)
const provider = ref<MailProviderChoice>('gmail')
const steps = [
	{
		title: 'Choose mail provider',
		description: 'Each account gets its own Settings target, credential custody and runtime status.',
	},
	{
		title: 'Account credentials',
		description: 'Connection metadata stays in Mail Settings. Passwords and provider tokens are sealed by Vault.',
	},
	{
		title: 'Authorize and confirm',
		description: 'Provider readiness remains account-scoped and can be refreshed after the wizard closes.',
	},
]
const canAdvance = computed(() => {
	if (step.value === 1) return Boolean(props.module?.settings)
	if (step.value === 2) return setup.canSubmit.value && !setup.busy.value
	if (provider.value === 'gmail') return setup.canSubmit.value && !setup.busy.value
	return setup.messageTone.value === 'success' && !setup.busy.value
})

function openWizard(): void {
	step.value = 1
	open.value = true
}

function selectProvider(next: MailProviderChoice): void {
	provider.value = next
	applyProviderDefaults()
}

function applyProviderDefaults(): void {
	setup.gmailState.value = undefined
	setup.returnedState.value = ''
	setup.authorizationCode.value = ''
	if (provider.value === 'gmail') {
		setup.kind.value = 'gmail'
		return
	}
	setup.kind.value = 'imap'
	if (provider.value === 'icloud') {
		setup.imapHost.value = 'imap.mail.me.com'
		setup.imapPort.value = '993'
		setup.smtpEnabled.value = false
		setup.smtpHost.value = 'smtp.mail.me.com'
		setup.smtpPort.value = '587'
	}
}

async function handleNext(nextStep: number): Promise<void> {
	if (nextStep === 2) {
		applyProviderDefaults()
		return
	}
	if (nextStep === 3) await setup.submit()
}

async function finish(): Promise<void> {
	if (provider.value === 'gmail') {
		if (!await setup.submit() || setup.messageTone.value !== 'success') return
	}
	if (setup.messageTone.value !== 'success') return
	emit('completed')
	open.value = false
}

async function activateRecoveredAccounts(): Promise<void> {
	if (await pendingActivation.activate()) emit('completed')
}
</script>

<template>
	<section class="integration-account-setup" data-provider-tone="mail">
		<header class="integration-account-setup__header">
			<span class="integration-account-setup__icon"><Icon icon="tabler:mail-plus" /></span>
			<div>
				<small>Provider accounts</small>
				<h3>Add a mail account</h3>
				<p>Connect Gmail, iCloud Mail or a custom IMAP mailbox through an account-scoped setup flow.</p>
			</div>
			<strong>Multi-account</strong>
		</header>

		<div class="provider-account-wizard__launcher">
			<div>
				<strong>Mail account wizard</strong>
				<p>Passwords and OAuth tokens never enter the generic Settings store.</p>
			</div>
			<button type="button" :disabled="!module?.settings" @click="openWizard">
				<Icon icon="tabler:user-plus" />
				Add account
			</button>
		</div>
		<div
			v-if="pendingActivation.pendingCount.value > 0"
			class="provider-account-wizard__launcher"
			data-testid="mail-pending-settings-activation"
		>
			<div>
				<strong>Recovered account configuration</strong>
				<p>{{ pendingActivation.pendingCount.value }} account target{{ pendingActivation.pendingCount.value === 1 ? '' : 's' }} await runtime validation.</p>
				<p v-if="pendingActivation.message.value" :data-tone="pendingActivation.messageTone.value">
					{{ pendingActivation.message.value }}
				</p>
			</div>
			<button
				type="button"
				:disabled="!pendingActivation.canActivate.value || pendingActivation.busy.value"
				@click="activateRecoveredAccounts"
			>
				<Icon icon="tabler:restore" />
				{{ pendingActivation.busy.value ? 'Activating…' : 'Activate recovered accounts' }}
			</button>
		</div>
	</section>

	<Steps
		v-model:open="open"
		v-model:step="step"
		:step-count="3"
		:steps="steps"
		title="Add mail account"
		description="Mail owns provider setup; Communications receives only provider-neutral events."
		finish-label="Complete setup"
		:can-advance="canAdvance"
		:busy="setup.busy.value"
		size="lg"
		content-class="mail-account-wizard"
		@next="handleNext"
		@finish="finish"
	>
		<template #step-1>
			<div class="provider-account-wizard__provider-grid">
				<button
					type="button"
					class="provider-account-wizard__provider"
					:data-selected="provider === 'gmail'"
					@click="selectProvider('gmail')"
				>
					<Icon icon="tabler:brand-google" size="1.75rem" />
					<strong>Gmail</strong>
					<small>OAuth authorization with a provider-issued consent URL.</small>
				</button>
				<button
					type="button"
					class="provider-account-wizard__provider"
					:data-selected="provider === 'icloud'"
					@click="selectProvider('icloud')"
				>
					<Icon icon="tabler:brand-apple" size="1.75rem" />
					<strong>iCloud Mail</strong>
					<small>IMAP with an Apple app-specific password.</small>
				</button>
				<button
					type="button"
					class="provider-account-wizard__provider"
					:data-selected="provider === 'imap'"
					@click="selectProvider('imap')"
				>
					<Icon icon="tabler:server" size="1.75rem" />
					<strong>Custom IMAP</strong>
					<small>Explicit incoming and optional implicit-TLS SMTP endpoints.</small>
				</button>
			</div>
		</template>

		<template #step-2>
			<div class="provider-account-wizard__form">
				<label>
					<span>Local account ID</span>
					<input v-model="setup.connectionId.value" required maxlength="128" placeholder="personal-mail">
				</label>
				<label>
					<span>Email / username</span>
					<input v-model="setup.email.value" required type="email" autocomplete="username" placeholder="you@example.com">
				</label>

				<template v-if="provider === 'gmail'">
					<label class="wide">
						<span>Google OAuth client ID</span>
						<input v-model="setup.gmailClientId.value" required autocomplete="off">
					</label>
					<label class="wide">
						<span>OAuth redirect URI</span>
						<input v-model="setup.gmailRedirectUri.value" required type="url">
					</label>
				</template>

				<template v-else>
					<label>
						<span>IMAP host</span>
						<input v-model="setup.imapHost.value" required>
					</label>
					<label>
						<span>IMAP port</span>
						<input v-model="setup.imapPort.value" required inputmode="numeric" pattern="[0-9]+">
					</label>
					<label class="wide">
						<span>{{ provider === 'icloud' ? 'Apple app-specific password' : 'IMAP password' }}</span>
						<input v-model="setup.imapPassword.value" required type="password" autocomplete="new-password">
					</label>
					<label v-if="provider === 'imap'" class="wide">
						<span>Outbound delivery</span>
						<select v-model="setup.smtpEnabled.value">
							<option :value="false">Configure later</option>
							<option :value="true">Enable implicit-TLS SMTP now</option>
						</select>
					</label>
					<template v-if="provider === 'imap' && setup.smtpEnabled.value">
						<label>
							<span>SMTP host</span>
							<input v-model="setup.smtpHost.value" required placeholder="smtp.example.com">
						</label>
						<label>
							<span>SMTP port</span>
							<input v-model="setup.smtpPort.value" required inputmode="numeric" pattern="[0-9]+">
						</label>
						<label class="wide">
							<span>SMTP password (blank = IMAP password)</span>
							<input v-model="setup.smtpPassword.value" type="password" autocomplete="new-password">
						</label>
					</template>
					<p v-if="provider === 'icloud'" class="provider-account-wizard__notice">
						Incoming iCloud Mail is configured now. Apple SMTP uses STARTTLS on port 587,
						which is not silently downgraded to the current implicit-TLS transport.
					</p>
				</template>
			</div>
		</template>

		<template #step-3>
			<div
				class="provider-account-wizard__status"
				:class="`provider-account-wizard__status--${setup.messageTone.value}`"
			>
				<template v-if="provider === 'gmail' && setup.gmailState.value">
					<h4>Google authorization required</h4>
					<p>Open the provider URL, then return the exact state and one-time code.</p>
					<a
						:href="setup.gmailState.value.started.authorizationUrl"
						target="_blank"
						rel="noreferrer"
					>
						<Icon icon="tabler:external-link" />
						Open Google authorization
					</a>
					<div class="provider-account-wizard__form">
						<label>
							<span>Returned state</span>
							<input v-model="setup.returnedState.value" required autocomplete="off">
						</label>
						<label>
							<span>Authorization code</span>
							<input v-model="setup.authorizationCode.value" required type="password" autocomplete="one-time-code">
						</label>
					</div>
				</template>
				<template v-else>
					<h4>{{ setup.busy.value ? 'Applying account configuration…' : 'Account setup result' }}</h4>
					<p>{{ setup.message.value || 'Waiting for the integration runtime.' }}</p>
				</template>
			</div>
		</template>
	</Steps>
</template>
