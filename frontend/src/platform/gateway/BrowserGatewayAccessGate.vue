<script setup lang="ts">
import { computed, ref } from 'vue'
import Icon from '../../shared/ui/Icon.vue'
import { readBrowserGatewayCredentialId, storeBrowserGatewayCredentialId } from './browserGatewayCredential'
import {
  authenticateBrowserGateway,
  browserGatewayAccessError,
  enrollBrowserGateway,
} from './browserGatewayAccess'

const emit = defineEmits<{ authenticated: [] }>()
const pairingId = ref('')
const busy = ref(false)
const error = ref('')
const credentialId = ref(readBrowserGatewayCredentialId())
const canAuthenticate = computed(() => Boolean(credentialId.value))

async function enroll(): Promise<void> {
	busy.value = true; error.value = ''
	try {
		const value = await enrollBrowserGateway(pairingId.value)
		storeBrowserGatewayCredentialId(value)
		credentialId.value = value
		await authenticate()
	} catch (reason) { error.value = browserGatewayAccessError(reason) } finally { busy.value = false }
}

async function authenticate(): Promise<void> {
	if (!credentialId.value) return
	busy.value = true; error.value = ''
	try {
		await authenticateBrowserGateway(credentialId.value)
		emit('authenticated')
	} catch (reason) { error.value = browserGatewayAccessError(reason) } finally { busy.value = false }
}
</script>

<template>
	<main class="browser-access-gate" data-ui-theme="base-light">
		<section class="browser-access-gate__card">
			<Icon icon="tabler:shield-lock" size="32" /><h1>Макошь</h1>
			<p>Each browser device requires approval in the Макошь CLI. No password, token or session is stored in this client.</p>
			<form v-if="!canAuthenticate" @submit.prevent="enroll"><label for="pairing-id">Pairing code</label><input id="pairing-id" v-model="pairingId" autocomplete="off" inputmode="text" maxlength="64" :disabled="busy"><button class="primary-button" :disabled="busy">{{ busy ? 'Connecting…' : 'Connect browser' }}</button></form>
			<button v-else class="primary-button" :disabled="busy" @click="authenticate">{{ busy ? 'Waiting for device key…' : 'Sign in with device key' }}</button>
			<p v-if="error" class="inline-error" role="alert">{{ error }}</p>
		</section>
	</main>
</template>
