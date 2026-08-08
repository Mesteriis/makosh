<script setup lang="ts">
import { computed, ref } from 'vue'
import type { ClientModuleBootstrapV1 } from '../../../gen/makosh/gateway/v1/client_bootstrap_pb'
import TelegramQrPairingPanel from '../../../integrations/telegram/presentation/TelegramQrPairingPanel.vue'
import Icon from '../../../shared/ui/Icon.vue'
import {
	legacyProviderRecoveryCompletedV1,
	legacyProviderRecoveryFingerprintV1,
	legacyProviderRecoveryRowsV1,
} from './legacyProviderRecoveryPresentation'
import { useLegacyProviderRecovery } from './useLegacyProviderRecovery'
import './legacyProviderRecoveryPanel.css'

const props = defineProps<{
	mailModule: ClientModuleBootstrapV1 | null
	telegramModule: ClientModuleBootstrapV1 | null
}>()
const emit = defineEmits<{ completed: [] }>()
const recovery = useLegacyProviderRecovery(
	() => props.mailModule,
	() => props.telegramModule,
)
const qrStartRequest = ref(0)
const fingerprint = computed(() => legacyProviderRecoveryFingerprintV1(
	recovery.plan.value?.bundleFingerprintSha256,
))
const rows = computed(() => legacyProviderRecoveryRowsV1(
	recovery.plan.value?.candidates,
	recovery.progress.value,
))

async function recoverAll(): Promise<void> {
	await recovery.recoverAll()
	if (recovery.telegramResult.value) qrStartRequest.value += 1
	if (legacyProviderRecoveryCompletedV1(recovery.progress.value)) {
		emit('completed')
	}
}
</script>

<template>
	<section class="legacy-provider-recovery">
		<header class="legacy-provider-recovery__header">
			<span><Icon icon="tabler:database-import" /></span>
			<div>
				<small>Owner maintenance</small>
				<h2>Recover legacy provider accounts</h2>
				<p>Restore exact provider configuration through Mail and Telegram contracts. Legacy secrets remain inside the native host.</p>
			</div>
			<strong>{{ recovery.available ? 'Available' : 'Unavailable' }}</strong>
		</header>

		<div v-if="!recovery.available" class="legacy-provider-recovery__notice">
			Start the root development ensemble with an explicit private recovery bundle.
		</div>

		<div class="legacy-provider-recovery__actions">
			<button
				type="button"
				:disabled="!recovery.canInspect.value"
				@click="recovery.inspect"
			>
				<Icon icon="tabler:shield-search" />
				Verify recovery bundle
			</button>
			<button
				class="primary"
				type="button"
				:disabled="!recovery.canRecover.value"
				@click="recoverAll"
			>
				<Icon icon="tabler:database-import" />
				{{ recovery.retryOutcomeUnknown.value
					? 'Retry uncertain step'
					: 'Recover 2 Mail + 1 Telegram' }}
			</button>
		</div>

		<div v-if="recovery.plan.value" class="legacy-provider-recovery__summary">
			<div><small>Bundle fingerprint</small><strong>{{ fingerprint }}</strong></div>
			<div><small>Active accounts</small><strong>3</strong></div>
			<div><small>Mail</small><strong>2</strong></div>
			<div><small>Telegram users</small><strong>1</strong></div>
			<div><small>Deleted records excluded</small><strong>2</strong></div>
		</div>

		<ol v-if="rows.length" class="legacy-provider-recovery__candidates">
			<li v-for="row in rows" :key="row.key">
				<span>{{ row.position }}</span>
				<div>
					<strong>{{ row.label }}</strong>
					<small>No account identity or secret is rendered here.</small>
				</div>
				<em :data-state="row.state">{{ row.state.replaceAll('_', ' ') }}</em>
			</li>
		</ol>

		<div
			v-if="recovery.message.value"
			class="legacy-provider-recovery__notice"
			:data-tone="recovery.messageTone.value"
			aria-live="polite"
		>
			{{ recovery.message.value }}
		</div>

		<section
			v-if="recovery.gmailResult.value?.kind === 'gmail'"
			class="legacy-provider-recovery__continuation"
		>
			<header>
				<div>
					<small>Mail continuation</small>
					<h3>Authorize recovered Gmail account</h3>
				</div>
				<strong>{{ recovery.oauthAccepted.value ? 'Accepted' : 'Required' }}</strong>
			</header>
			<p>The old OAuth token was not imported. Complete a current Google authorization for the recovered target.</p>
			<a
				:href="recovery.gmailResult.value.oauth.started.authorizationUrl"
				target="_blank"
				rel="noreferrer"
			>
				<Icon icon="tabler:external-link" />
				Open Google authorization
			</a>
			<form v-if="!recovery.oauthAccepted.value" @submit.prevent="recovery.completeGmail">
				<label>
					<span>Returned state</span>
					<input v-model="recovery.returnedState.value" required autocomplete="off">
				</label>
				<label>
					<span>One-time authorization code</span>
					<input v-model="recovery.authorizationCode.value" required type="password" autocomplete="one-time-code">
				</label>
				<button type="submit" :disabled="recovery.busy.value">
					Complete Gmail OAuth
				</button>
			</form>
		</section>

		<TelegramQrPairingPanel
			v-if="recovery.telegramResult.value"
			:module="telegramModule"
			:start-request="qrStartRequest"
		/>
	</section>
</template>
