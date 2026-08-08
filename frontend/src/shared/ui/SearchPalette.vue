<script setup lang="ts">
import { computed } from 'vue'
import CommandPalette from './CommandPalette.vue'
import type { CommandGroup, CommandItem } from './Command.types'

const props = withDefaults(defineProps<{
	open?: boolean
	groups?: CommandGroup[]
	triggerLabel?: string
	placeholder?: string
	emptyMessage?: string
	class?: string
}>(), {
	groups: () => [],
	triggerLabel: 'Open search',
	placeholder: 'Search...'
})

const emit = defineEmits<{
	'update:open': [value: boolean]
	select: [item: CommandItem]
}>()

const classes = computed(() => ['makosh-search-palette', props.class].filter(Boolean).join(' '))
</script>

<template>
	<CommandPalette
		:class="classes"
		:empty-message="emptyMessage"
		:groups="groups"
		:open="open"
		:placeholder="placeholder"
		:trigger-label="triggerLabel"
		@select="emit('select', $event)"
		@update:open="emit('update:open', $event)"
	/>
</template>
