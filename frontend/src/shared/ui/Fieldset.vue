<script setup lang="ts">
import { computed, useId } from 'vue'

const props = withDefaults(defineProps<{
	disabled?: boolean
	class?: string
}>(), {
	disabled: false
})

const classes = computed(() => [
	'makosh-fieldset',
	props.disabled && 'makosh-fieldset--disabled',
	props.class
])

const descriptionId = `makosh-fieldset-description-${useId()}`
</script>

<template>
	<fieldset
		:class="classes"
		:disabled="disabled"
		:aria-describedby="$slots.description ? descriptionId : undefined"
	>
		<legend v-if="$slots.legend" class="makosh-fieldset-legend">
			<slot name="legend" />
		</legend>
		<p
			v-if="$slots.description"
			:id="descriptionId"
			class="makosh-fieldset-description"
		>
			<slot name="description" />
		</p>
		<div class="makosh-fieldset-content">
			<slot />
		</div>
	</fieldset>
</template>
