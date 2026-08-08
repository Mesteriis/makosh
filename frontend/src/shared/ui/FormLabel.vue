<script setup lang="ts">
import { computed } from 'vue'

const props = withDefaults(defineProps<{
	for?: string
	required?: boolean
	optional?: boolean
	optionalLabel?: string
	class?: string
}>(), {
	required: false,
	optional: false,
	optionalLabel: 'Optional'
})

const classes = computed(() => ['makosh-form-label', props.class])
</script>

<template>
	<label :for="props.for" :class="classes">
		<span><slot /></span>
		<span v-if="required" class="makosh-form-label__marker" aria-hidden="true">*</span>
		<span v-else-if="optional" class="makosh-form-hint">{{ optionalLabel }}</span>
	</label>
</template>
