<script setup lang="ts">
import { FocusScope } from 'reka-ui'
import { computed } from 'vue'

const props = withDefaults(defineProps<{
	as?: string
	trapped?: boolean
	loop?: boolean
	class?: string
}>(), {
	as: 'div',
	trapped: true,
	loop: true
})

const emit = defineEmits<{
	mountAutoFocus: [event: Event]
	unmountAutoFocus: [event: Event]
}>()

const classes = computed(() => ['makosh-focus-trap', props.class])
</script>

<template>
	<FocusScope
		:as="as"
		:class="classes"
		:trapped="trapped"
		:loop="loop"
		@mount-auto-focus="(event) => emit('mountAutoFocus', event)"
		@unmount-auto-focus="(event) => emit('unmountAutoFocus', event)"
	>
		<slot />
	</FocusScope>
</template>
