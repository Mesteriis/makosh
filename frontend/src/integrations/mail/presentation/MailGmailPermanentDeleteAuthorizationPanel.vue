<script setup lang="ts">
import { onMounted } from 'vue'
import Icon from '../../../shared/ui/Icon.vue'
import '../../../shared/ui/settings/integrationAccountSetupCard.css'
import { useMailGmailPermanentDeleteAuthorization } from '../setup/useMailGmailPermanentDeleteAuthorization'

const authorization = useMailGmailPermanentDeleteAuthorization()
onMounted(() => void authorization.refreshAccounts())
</script>

<template>
	<section class="integration-account-setup" data-provider-tone="mail" data-auth-method="oauth">
		<header class="integration-account-setup__header">
			<span class="integration-account-setup__icon"><Icon icon="tabler:shield-lock" /></span>
			<div>
				<small>Gmail authority</small>
				<h3>Permanent deletion</h3>
				<p>Optional broad Gmail permission, requested separately and only after owner action.</p>
			</div>
			<strong>Explicit opt-in</strong>
		</header>
		<details>
			<summary>
				<span><Icon icon="tabler:key" /> Authorize permanent deletion</span>
				<Icon icon="tabler:chevron-down" />
			</summary>
			<form class="integration-account-setup__form" @submit.prevent="authorization.submit">
				<div class="integration-account-setup__fields">
					<label class="wide">
						<span>Gmail account</span>
						<select
							v-model="authorization.connectionId.value"
							:disabled="Boolean(authorization.started.value)"
							required
						>
							<option v-if="authorization.accounts.value.length === 0" value="">No Gmail accounts</option>
							<option
								v-for="account in authorization.accounts.value"
								:key="account.connectionId"
								:value="account.connectionId"
							>
								{{ account.connectionId }}
							</option>
						</select>
					</label>
					<p v-if="authorization.started.value" class="wide">
						Google OAuth opens from the button below and returns through the guarded loopback callback.
					</p>
				</div>
				<footer>
					<p
						v-if="authorization.message.value"
						:class="authorization.failed.value
							? 'integration-account-setup__message--error'
							: 'integration-account-setup__message--neutral'"
						aria-live="polite"
					>
						{{ authorization.message.value }}
					</p>
					<button v-if="authorization.operationId.value" type="button" :disabled="authorization.busy.value" @click="authorization.refreshStatus">
						Refresh status
					</button>
					<button type="submit" :disabled="authorization.busy.value || authorization.completionSubmitted.value">
						<Icon :icon="authorization.busy.value ? 'tabler:loader-2' : 'tabler:shield-check'" />
						{{ authorization.busy.value ? 'Applying…' : authorization.submitLabel.value }}
					</button>
				</footer>
			</form>
		</details>
	</section>
</template>
