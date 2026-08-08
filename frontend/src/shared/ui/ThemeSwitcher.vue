<script setup lang="ts">
import { computed } from 'vue'
import type { UiThemeFamily, UiThemeFamilyOption, UiThemeMode, UiThemeModeOption, UiThemeName } from './foundation/theme'
import {
	normalizeUiThemeName,
	themeNameToSelection,
	themeSelectionToName,
	uiThemeFamilyOptions,
	uiThemeModeOptions
} from './foundation/theme'

const props = withDefaults(defineProps<{
	modelValue?: UiThemeName
	familyOptions?: UiThemeFamilyOption[]
	modeOptions?: UiThemeModeOption[]
	label?: string
	disabled?: boolean
	class?: string
}>(), {
	modelValue: 'base-light',
	label: 'Theme',
	disabled: false
})

const emit = defineEmits<{
	'update:modelValue': [value: UiThemeName]
}>()

const familyOptions = computed(() => props.familyOptions?.length ? props.familyOptions : uiThemeFamilyOptions)
const modeOptions = computed(() => props.modeOptions?.length ? props.modeOptions : uiThemeModeOptions)
const classes = computed(() => ['makosh-theme-switcher', props.class])
const resolvedTheme = computed(() => normalizeUiThemeName(props.modelValue))
const resolvedThemeSelection = computed(() => themeNameToSelection(resolvedTheme.value))

function updateFamily(family: UiThemeFamily): void {
	emit('update:modelValue', themeSelectionToName(family, resolvedThemeSelection.value.mode))
}

function updateMode(mode: UiThemeMode): void {
	emit('update:modelValue', themeSelectionToName(resolvedThemeSelection.value.family, mode))
}
</script>

<template>
	<div :class="classes" role="group" :aria-label="label">
		<div class="makosh-theme-switcher__group" role="radiogroup" :aria-label="`${label}: family`">
			<button
				v-for="option in familyOptions"
				:key="option.value"
				class="makosh-theme-switcher__option"
				type="button"
				role="radio"
				:aria-checked="resolvedThemeSelection.family === option.value"
				:disabled="disabled"
				:title="option.description"
				@click="updateFamily(option.value)"
			>
				<span
					class="makosh-theme-switcher__swatch"
					:data-theme-swatch="themeSelectionToName(option.value, resolvedThemeSelection.mode)"
					aria-hidden="true"
				/>
				<span>{{ option.label }}</span>
			</button>
		</div>

		<div class="makosh-theme-switcher__group" role="radiogroup" :aria-label="`${label}: mode`">
			<button
				v-for="option in modeOptions"
				:key="option.value"
				class="makosh-theme-switcher__option"
				type="button"
				role="radio"
				:aria-checked="resolvedThemeSelection.mode === option.value"
				:disabled="disabled"
				:title="option.description"
				@click="updateMode(option.value)"
			>
				<span>{{ option.label }}</span>
			</button>
		</div>
	</div>
</template>
