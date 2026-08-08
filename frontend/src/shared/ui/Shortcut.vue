<script setup lang="ts">
import { computed } from 'vue'
import Kbd from './Kbd.vue'

const props = withDefaults(defineProps<{
	keys?: string[]
	label?: string
	joiner?: string
	class?: string
}>(), {
	keys: () => [],
	joiner: '+'
})

const classes = computed(() => ['makosh-shortcut', props.class])
const accessibleLabel = computed(() => props.label ?? props.keys.join(` ${props.joiner} `))
</script>

<template>
	<span :class="classes" :aria-label="accessibleLabel">
		<template v-for="(key, index) in keys" :key="`${key}-${index}`">
			<Kbd>{{ key }}</Kbd>
			<span v-if="index < keys.length - 1" class="makosh-shortcut__joiner" aria-hidden="true">{{ joiner }}</span>
		</template>
	</span>
</template>
