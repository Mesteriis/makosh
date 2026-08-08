<script setup lang="ts">
import { computed } from 'vue'

const props = withDefaults(defineProps<{
	value?: string
	max?: number
	class?: string
}>(), {
	value: '',
	max: 0
})

const count = computed(() => props.value.length)
const overLimit = computed(() => props.max > 0 && count.value > props.max)
const classes = computed(() => ['makosh-character-counter', { 'makosh-character-counter--over': overLimit.value }, props.class])
</script>

<template>
	<span :class="classes" aria-live="polite">
		{{ count }}<template v-if="max > 0">/{{ max }}</template>
	</span>
</template>
