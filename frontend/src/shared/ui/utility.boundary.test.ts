import { describe, expect, it } from 'vitest'
import { existsSync, readFileSync } from 'node:fs'
import { fileURLToPath } from 'node:url'
import { join } from 'node:path'
import {
	isUiThemeName,
	themeNameToSelection,
	themeSelectionToName,
	uiThemeFamilyOptions,
	uiThemeModeOptions,
	uiThemeNames
} from './foundation/theme'

const utilityComponents = [
	'CopyButton',
	'ThemeSwitcher',
	'LocaleSwitcher',
	'KeyboardHint',
	'Shortcut',
	'ProviderIcon',
	'StatusIcon',
	'EntityIcon',
	'FileIcon'
]

describe('Макошь UI utility component contracts', () => {
	it('keeps the utility batch documented and exported through the UI kit', () => {
		const uiRoot = fileURLToPath(new URL('.', import.meta.url))
		const barrel = readFileSync(join(uiRoot, 'index.ts'), 'utf8')

		for (const componentName of utilityComponents) {
			expect(existsSync(join(uiRoot, `${componentName}.vue`))).toBe(true)
			expect(existsSync(join(uiRoot, `${componentName}.README.md`))).toBe(true)
			expect(barrel).toContain(`export { default as ${componentName} } from './${componentName}.vue'`)
		}
	})

	it('keeps utility primitives UI-only', () => {
		const uiRoot = fileURLToPath(new URL('.', import.meta.url))
		const forbiddenSource = /@\/domains|@\/integrations|@\/platform|fetch\(|useQuery|defineStore|createRouter|localStorage/

		for (const componentName of utilityComponents) {
			const source = readFileSync(join(uiRoot, `${componentName}.vue`), 'utf8')
			expect(source, componentName).not.toMatch(forbiddenSource)
		}
	})

	it('keeps clipboard behavior explicit and local to CopyButton', () => {
		const uiRoot = fileURLToPath(new URL('.', import.meta.url))
		const copySource = readFileSync(join(uiRoot, 'CopyButton.vue'), 'utf8')

		expect(copySource).toContain('navigator.clipboard.writeText')
		expect(copySource).toContain("emit('copied'")
		expect(copySource).toContain("emit('error'")
	})

	it('keeps utility components represented in Storybook', () => {
		const storySource = readFileSync(
			fileURLToPath(new URL('../../../stories/ui/Utility.stories.ts', import.meta.url)),
			'utf8'
		)

		for (const componentName of utilityComponents) {
			expect(storySource).toContain(componentName)
		}
	})

	it('keeps theme ids explicit across family and mode axes', () => {
		expect(uiThemeNames).toEqual(['base-light', 'base-dark', 'makosh-light', 'makosh-dark'])
		expect(uiThemeFamilyOptions.map((option) => option.value)).toEqual(['base', 'makosh'])
		expect(uiThemeModeOptions.map((option) => option.value)).toEqual(['light', 'dark'])
		expect(themeSelectionToName('base', 'light')).toBe('base-light')
		expect(themeSelectionToName('base', 'dark')).toBe('base-dark')
		expect(themeSelectionToName('makosh', 'light')).toBe('makosh-light')
		expect(themeSelectionToName('makosh', 'dark')).toBe('makosh-dark')
		expect(themeNameToSelection('makosh-light')).toEqual({ family: 'makosh', mode: 'light' })
		expect(isUiThemeName('light')).toBe(false)
		expect(isUiThemeName('dark')).toBe(false)
		expect(isUiThemeName('makosh')).toBe(false)
	})
})
