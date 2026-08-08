<script setup lang="ts">
import { HoverCardArrow, HoverCardContent, HoverCardPortal, HoverCardRoot, HoverCardTrigger } from 'reka-ui'
import { computed } from 'vue'

const props = withDefaults(defineProps<{
	open?: boolean
	side?: 'top' | 'bottom' | 'left' | 'right'
	sideOffset?: number
	align?: 'start' | 'center' | 'end'
	openDelay?: number
	closeDelay?: number
	contentRole?: string
	ariaLabel?: string
	class?: string
}>(), {
	side: 'bottom',
	sideOffset: 8,
	align: 'center',
	openDelay: 250,
	closeDelay: 120,
	contentRole: 'region'
})

const emit = defineEmits<{
	'update:open': [value: boolean]
}>()

const contentClasses = computed(() => ['makosh-hover-card-content', props.class])
</script>

<template>
	<HoverCardRoot
		:open="open"
		:open-delay="openDelay"
		:close-delay="closeDelay"
		@update:open="(value) => emit('update:open', value)"
	>
		<HoverCardTrigger as-child>
			<slot name="trigger" />
		</HoverCardTrigger>
		<HoverCardPortal>
			<HoverCardContent
				:class="contentClasses"
				:side="side"
				:side-offset="sideOffset"
				:align="align"
				:role="contentRole"
				:aria-label="ariaLabel"
			>
				<slot />
				<HoverCardArrow class="makosh-hover-card-arrow" />
			</HoverCardContent>
		</HoverCardPortal>
	</HoverCardRoot>
</template>
