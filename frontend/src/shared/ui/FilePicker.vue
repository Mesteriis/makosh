<script setup lang="ts">
import { computed } from 'vue'

const props = withDefaults(defineProps<{
	id?: string
	accept?: string
	ariaLabel?: string
	disabled?: boolean
	multiple?: boolean
	class?: string
}>(), {
	disabled: false,
	multiple: false
})

const emit = defineEmits<{
	change: [files: File[]]
}>()

const classes = computed(() => ['makosh-file-picker', props.class])

function handleChange(event: Event): void {
	const target = event.target as HTMLInputElement
	emit('change', Array.from(target.files ?? []))
}
</script>

<template>
	<input
		:accept="accept"
		:aria-label="ariaLabel"
		:class="classes"
		:disabled="disabled"
		:id="id"
		:multiple="multiple"
		type="file"
		@change="handleChange"
	/>
</template>
