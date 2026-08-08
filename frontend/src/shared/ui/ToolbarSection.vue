<script setup lang="ts">
import { computed, useId } from 'vue'

type ToolbarSectionOrientation = 'horizontal' | 'vertical'

const props = withDefaults(defineProps<{
	orientation?: ToolbarSectionOrientation
	class?: string
}>(), {
	orientation: 'horizontal'
})

const classes = computed(() => [
	'makosh-toolbar-section',
	`makosh-toolbar-section--${props.orientation}`,
	props.class
])

const labelId = `makosh-toolbar-section-label-${useId()}`
</script>

<template>
	<section
		:class="classes"
		role="group"
		:aria-labelledby="$slots.label ? labelId : undefined"
	>
		<span
			v-if="$slots.label"
			:id="labelId"
			class="makosh-toolbar-section-label"
		>
			<slot name="label" />
		</span>
		<div class="makosh-toolbar-section-content">
			<slot />
		</div>
	</section>
</template>
