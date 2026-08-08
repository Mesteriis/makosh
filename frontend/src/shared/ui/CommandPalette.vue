<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import Button from './primitives/Button.vue'
import Command from './Command.vue'
import type { CommandGroup, CommandItem } from './Command.types'

const props = withDefaults(defineProps<{
	open?: boolean
	groups?: CommandGroup[]
	triggerLabel?: string
	placeholder?: string
	emptyMessage?: string
	class?: string
}>(), {
	open: undefined,
	groups: () => [],
	triggerLabel: 'Open command palette'
})

const emit = defineEmits<{
	'update:open': [value: boolean]
	select: [item: CommandItem]
}>()

const internalOpen = ref(false)
const classes = computed(() => ['makosh-command-palette', props.class])
const isOpen = computed({
	get: () => props.open ?? internalOpen.value,
	set: (value: boolean) => {
		internalOpen.value = value
		emit('update:open', value)
	}
})

watch(() => props.open, (value) => {
	if (typeof value === 'boolean') {
		internalOpen.value = value
	}
})
</script>

<template>
	<div :class="classes">
		<slot name="trigger" :open="() => { isOpen = true }">
			<Button icon="tabler:command" variant="outline" @click="isOpen = true">{{ triggerLabel }}</Button>
		</slot>
		<Command
			v-model:open="isOpen"
			:empty-message="emptyMessage"
			:groups="groups"
			:placeholder="placeholder"
			@select="emit('select', $event)"
		/>
	</div>
</template>
