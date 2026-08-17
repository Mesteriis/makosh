<script setup lang="ts">
import '../../../shared/ui/settings/providerAccountWizard.css'

withDefaults(defineProps<{
	stage: 'configuration' | 'authorization'
	clientConfigured?: boolean
	clientId?: string
	redirectUri?: string
	message?: string
	busy?: boolean
}>(), {
	clientConfigured: true,
	clientId: '',
	redirectUri: 'http://127.0.0.1:5173/oauth/google/callback',
	message: '',
	busy: false,
})

defineEmits<{ 'update:clientId': [value: string] }>()
</script>

<template>
	<div
		v-if="stage === 'configuration'"
		class="provider-account-wizard__form"
		data-provider="gmail"
		data-auth-method="oauth"
	>
		<p class="provider-account-wizard__notice wide">
			The Google account and mailbox identity are selected during OAuth authorization.
		</p>
		<label v-if="!clientConfigured" class="wide">
			<span>Google OAuth client ID</span>
			<input
				:value="clientId"
				required
				autocomplete="off"
				@input="$emit('update:clientId', ($event.target as HTMLInputElement).value)"
			>
		</label>
		<p v-else class="provider-account-wizard__notice wide">
			The installed Google OAuth client is configured for this Макошь build.
		</p>
		<label class="wide">
			<span>OAuth redirect URI</span>
			<input :value="redirectUri" readonly type="url">
		</label>
		<p class="provider-account-wizard__notice wide">
			Google returns to a one-use loopback callback. State and authorization code
			are validated and submitted automatically; they are never copied into this form.
		</p>
	</div>
	<div
		v-else
		class="provider-account-wizard__status provider-account-wizard__status--neutral"
		data-provider="gmail"
		data-auth-method="oauth"
	>
		<h4>Continue with Google OAuth</h4>
		<p>Select “Continue with Google”. Макошь opens the provider authorization URL
			and consumes only the exact matching loopback callback.</p>
		<p>{{ message || (busy ? 'Preparing Google authorization…' : 'Gmail configuration is active.') }}</p>
	</div>
</template>
