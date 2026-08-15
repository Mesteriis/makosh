<script setup lang="ts">
import { ref } from 'vue'
import './telegramCloudPasswordForm.css'

withDefaults(defineProps<{
	modelValue: string
	id?: string
	hint?: string
	busy?: boolean
	message?: string
	messageTone?: 'neutral' | 'success' | 'error'
	compact?: boolean
}>(), {
	hint: '',
	id: 'telegram-cloud-password',
	busy: false,
	message: '',
	messageTone: 'neutral',
	compact: false,
})

const emit = defineEmits<{
	'update:modelValue': [value: string]
	submit: []
}>()
const passwordVisible = ref(false)
</script>

<template>
	<form
		class="telegram-cloud-password"
		:class="{ 'telegram-cloud-password--compact': compact }"
		@submit.prevent="emit('submit')"
	>
		<div class="telegram-cloud-password__heading">
			<span aria-hidden="true">2FA</span>
			<div>
				<strong>Telegram cloud password</strong>
				<p>This account uses two-step verification. Enter the cloud password to finish authorization.</p>
			</div>
		</div>
		<label :for="id">
			Password
			<small v-if="hint">Hint: {{ hint }}</small>
		</label>
		<div class="telegram-cloud-password__control">
			<input
				:id="id"
				:type="passwordVisible ? 'text' : 'password'"
				autocomplete="current-password"
				required
				:disabled="busy"
				:value="modelValue"
				@input="emit('update:modelValue', ($event.target as HTMLInputElement).value)"
			>
			<button
				type="button"
				class="telegram-cloud-password__visibility"
				:aria-label="passwordVisible ? 'Hide Telegram cloud password' : 'Show Telegram cloud password'"
				@click="passwordVisible = !passwordVisible"
			>
				{{ passwordVisible ? 'Hide' : 'Show' }}
			</button>
		</div>
		<p v-if="message" class="telegram-cloud-password__message" :data-tone="messageTone" aria-live="polite">
			{{ message }}
		</p>
		<button class="telegram-cloud-password__submit" type="submit" :disabled="busy || !modelValue.trim()">
			{{ busy ? 'Checking…' : 'Continue' }}
		</button>
	</form>
</template>
