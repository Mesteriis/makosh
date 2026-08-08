<script setup lang="ts">
import { computed } from 'vue'

const props = withDefaults(defineProps<{
	disabled?: boolean
	novalidate?: boolean
	class?: string
}>(), {
	disabled: false,
	novalidate: true
})

const emit = defineEmits<{
	submit: [event: SubmitEvent]
}>()

const classes = computed(() => ['makosh-form', { 'makosh-form--disabled': props.disabled }, props.class])

function handleSubmit(event: SubmitEvent): void {
	emit('submit', event)
}
</script>

<template>
	<form :class="classes" :novalidate="novalidate" @submit.prevent="handleSubmit">
		<fieldset :disabled="disabled">
			<slot />
		</fieldset>
	</form>
</template>
