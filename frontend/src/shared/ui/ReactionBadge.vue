<script setup lang="ts">
import { computed } from 'vue'

const props = withDefaults(defineProps<{
	emoji: string
	count?: number
	label?: string
	active?: boolean
	interactive?: boolean
	class?: string
}>(), {
	count: 0,
	active: false,
	interactive: false
})

const emit = defineEmits<{
	click: [event: MouseEvent]
}>()

const classes = computed(() => [
	'makosh-reaction-badge',
	{
		'makosh-reaction-badge--active': props.active,
		'makosh-reaction-badge--interactive': props.interactive
	},
	props.class
])
const accessibleLabel = computed(() => props.label ?? `${props.emoji} ${props.count}`)
</script>

<template>
	<button
		v-if="interactive"
		:class="classes"
		type="button"
		:aria-pressed="active"
		:aria-label="accessibleLabel"
		@click="emit('click', $event)"
	>
		<span aria-hidden="true">{{ emoji }}</span>
		<span>{{ count }}</span>
	</button>
	<span v-else :class="classes" :aria-label="accessibleLabel">
		<span aria-hidden="true">{{ emoji }}</span>
		<span>{{ count }}</span>
	</span>
</template>
