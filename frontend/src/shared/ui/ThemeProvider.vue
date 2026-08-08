<script setup lang="ts">
import { computed } from 'vue'
import type { UiThemeName } from './foundation/theme'
import { normalizeUiThemeName, themeNameToSelection } from './foundation/theme'

const props = withDefaults(defineProps<{
	as?: string
	theme?: UiThemeName
	class?: string
}>(), {
	as: 'div',
	theme: 'base-light'
})

const classes = computed(() => ['makosh-theme-provider', props.class])
const resolvedTheme = computed(() => normalizeUiThemeName(props.theme))
const resolvedThemeSelection = computed(() => themeNameToSelection(resolvedTheme.value))
</script>

<template>
	<component
		:is="as"
		:data-ui-theme="resolvedTheme"
		:data-ui-theme-family="resolvedThemeSelection.family"
		:data-ui-theme-mode="resolvedThemeSelection.mode"
		:class="classes"
	>
		<slot />
	</component>
</template>
