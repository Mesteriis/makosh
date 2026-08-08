<script setup lang="ts">
import { computed } from 'vue'
import Icon from './Icon.vue'
import type { ComposerToolbarAction } from './Communication.types'

const props = withDefaults(defineProps<{
	actions?: ComposerToolbarAction[]
	label?: string
	disabled?: boolean
	class?: string
}>(), {
	actions: () => [],
	label: 'Composer tools',
	disabled: false
})

const emit = defineEmits<{
	select: [action: ComposerToolbarAction]
}>()

const classes = computed(() => ['makosh-composer-toolbar', props.class])

function selectAction(action: ComposerToolbarAction): void {
	if (!props.disabled && !action.disabled) {
		emit('select', action)
	}
}
</script>

<template>
	<div :class="classes" role="toolbar" :aria-label="label">
		<button
			v-for="action in actions"
			:key="action.id"
			class="makosh-composer-toolbar__action"
			:class="{
				'makosh-composer-toolbar__action--active': action.active,
				[`makosh-composer-toolbar__action--${action.tone}`]: action.tone
			}"
			type="button"
			:aria-pressed="action.active"
			:disabled="disabled || action.disabled"
			@click="selectAction(action)"
		>
			<Icon v-if="action.icon" :icon="action.icon" size="1rem" />
			<span>{{ action.label }}</span>
		</button>
		<slot />
	</div>
</template>
