import { describe, expect, it } from 'vitest'
import { existsSync, readFileSync } from 'node:fs'
import { fileURLToPath } from 'node:url'
import { join } from 'node:path'

const navigationComponents = [
	'Breadcrumbs',
	'Pagination',
	'Tree',
	'TreeItem',
	'Menu',
	'ContextMenu',
	'NavigationMenu',
	'Menubar',
	'CommandPalette',
	'SearchPalette'
]

describe('Макошь UI navigation component contracts', () => {
	it('keeps the navigation batch documented and exported through the UI kit', () => {
		const uiRoot = fileURLToPath(new URL('.', import.meta.url))
		const barrel = readFileSync(join(uiRoot, 'index.ts'), 'utf8')

		for (const componentName of navigationComponents) {
			expect(existsSync(join(uiRoot, `${componentName}.vue`))).toBe(true)
			expect(existsSync(join(uiRoot, `${componentName}.README.md`))).toBe(true)
			expect(barrel).toContain(`export { default as ${componentName} } from './${componentName}.vue'`)
		}
	})

	it('keeps navigation components UI-only', () => {
		const uiRoot = fileURLToPath(new URL('.', import.meta.url))
		const forbiddenSource = /@\/domains|@\/integrations|@\/platform|fetch\(|useQuery|defineStore|createRouter/

		for (const componentName of navigationComponents) {
			const source = readFileSync(join(uiRoot, `${componentName}.vue`), 'utf8')
			expect(source, componentName).not.toMatch(forbiddenSource)
		}
	})

	it('keeps navigation components represented in Storybook', () => {
		const storySource = readFileSync(
			fileURLToPath(new URL('../../../stories/ui/Navigation.stories.ts', import.meta.url)),
			'utf8'
		)

		for (const componentName of navigationComponents) {
			expect(storySource).toContain(componentName)
		}
	})
})
