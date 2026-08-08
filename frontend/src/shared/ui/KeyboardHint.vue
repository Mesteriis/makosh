<script setup lang="ts">
import { computed } from 'vue'
import Kbd from './Kbd.vue'

const props = withDefaults(defineProps<{
	label?: string
	keys?: string[]
	class?: string
}>(), {
	keys: () => []
})

const classes = computed(() => ['makosh-keyboard-hint', props.class])
</script>

<template>
	<span :class="classes">
		<span v-if="label" class="makosh-keyboard-hint__label">{{ label }}</span>
		<span class="makosh-keyboard-hint__keys" aria-hidden="true">
			<Kbd v-for="key in keys" :key="key">{{ key }}</Kbd>
		</span>
		<span class="makosh-sr-only">{{ [label, ...keys].filter(Boolean).join(' ') }}</span>
	</span>
</template>
