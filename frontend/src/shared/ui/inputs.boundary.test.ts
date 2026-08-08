import { describe, expect, it } from 'vitest'
import { existsSync, readFileSync } from 'node:fs'
import { fileURLToPath } from 'node:url'
import { join } from 'node:path'

const inputComponents = [
	'SearchInput',
	'TokenInput',
	'TagInput',
	'PasswordInput',
	'EmailInput',
	'NumberInput',
	'OTPInput',
	'Checkbox',
	'Radio',
	'RadioGroup',
	'Slider',
	'RangeSlider',
	'Form',
	'FormField',
	'FormLabel',
	'FormHint',
	'FormError',
	'CharacterCounter',
	'MultiSelect',
	'Combobox',
	'Autocomplete',
	'SearchableSelect',
	'SearchableMultiSelect',
	'GroupedSelect',
	'TreeSelect',
	'Cascader',
	'AsyncSelect',
	'ColorPicker',
	'DatePicker',
	'DateRangePicker',
	'TimePicker',
	'DateTimePicker',
	'FilePicker',
	'FileDropZone'
]

const selectionStoryFiles = [
	'GeneralSelect.stories.ts',
	'GeneralSearchableSelect.stories.ts',
	'GeneralMultiSelect.stories.ts',
	'GeneralSearchableMultiSelect.stories.ts',
	'GeneralGroupedSelect.stories.ts',
	'GeneralTreeSelect.stories.ts',
	'GeneralCascader.stories.ts',
	'GeneralAsyncSelect.stories.ts'
]

describe('Макошь UI input and form component contracts', () => {
	it('keeps the input batch documented and exported through the UI kit', () => {
		const uiRoot = fileURLToPath(new URL('.', import.meta.url))
		const barrel = readFileSync(join(uiRoot, 'index.ts'), 'utf8')

		for (const componentName of inputComponents) {
			expect(existsSync(join(uiRoot, `${componentName}.vue`))).toBe(true)
			expect(existsSync(join(uiRoot, `${componentName}.README.md`))).toBe(true)
			expect(barrel).toContain(`export { default as ${componentName} } from './${componentName}.vue'`)
		}
	})

	it('keeps input components UI-only', () => {
		const uiRoot = fileURLToPath(new URL('.', import.meta.url))
		const forbiddenSource = /@\/domains|@\/integrations|@\/platform|fetch\(|useQuery|defineStore|createRouter/

		for (const componentName of inputComponents) {
			const source = readFileSync(join(uiRoot, `${componentName}.vue`), 'utf8')
			expect(source, componentName).not.toMatch(forbiddenSource)
		}
	})

	it('keeps selection components represented in General Storybook entries', () => {
		const storiesRoot = fileURLToPath(new URL('../../../stories/ui/', import.meta.url))

		for (const storyFile of selectionStoryFiles) {
			expect(existsSync(join(storiesRoot, storyFile)), storyFile).toBe(true)
			const storySource = readFileSync(join(storiesRoot, storyFile), 'utf8')
			expect(storySource).toContain('Макошь UI/General/')
		}
	})

	it('keeps AsyncSelect loading state local and non-rotating', () => {
		const uiRoot = fileURLToPath(new URL('.', import.meta.url))
		const asyncSelectSource = readFileSync(join(uiRoot, 'AsyncSelect.vue'), 'utf8')
		const controlsCss = readFileSync(join(uiRoot, 'styles/controls.css'), 'utf8')

		expect(asyncSelectSource).toContain('makosh-async-select__loading-mark')
		expect(asyncSelectSource).not.toContain("from './Spinner.vue'")
		expect(controlsCss).toContain('.makosh-async-select__loading-dot')
		expect(controlsCss).toContain('.makosh-async-select__state > span:not(.makosh-async-select__loading-mark)')
		expect(controlsCss).toContain('@keyframes makosh-async-select-loading-dot')
		expect(controlsCss).toContain('@media (prefers-reduced-motion: reduce)')
		expect(controlsCss).not.toContain('makosh-async-select-loading-spin')
	})
})
