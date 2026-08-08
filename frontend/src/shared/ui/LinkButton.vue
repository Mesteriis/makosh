<script setup lang="ts">
import { computed } from 'vue'
import Icon from './Icon.vue'

const props = withDefaults(defineProps<{
	href: string
	size?: 'sm' | 'md' | 'lg'
	tone?: 'accent' | 'quiet' | 'danger'
	icon?: string
	external?: boolean
	disabled?: boolean
	class?: string
}>(), {
	size: 'md',
	tone: 'accent',
	external: false,
	disabled: false
})

const classes = computed(() => [
	'makosh-link-button',
	`makosh-link-button--${props.size}`,
	`makosh-link-button--${props.tone}`,
	{ 'makosh-link-button--disabled': props.disabled },
	props.class
])

const target = computed(() => props.external ? '_blank' : undefined)
const rel = computed(() => props.external ? 'noreferrer' : undefined)
</script>

<template>
	<a
		:aria-disabled="disabled || undefined"
		:class="classes"
		:href="disabled ? undefined : href"
		:rel="rel"
		:target="target"
	>
		<Icon v-if="icon" :icon="icon" size="1em" />
		<span><slot /></span>
	</a>
</template>
