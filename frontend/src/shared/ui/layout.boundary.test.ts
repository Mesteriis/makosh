import { describe, expect, it } from 'vitest'
import { existsSync, readFileSync } from 'node:fs'
import { fileURLToPath } from 'node:url'
import { join } from 'node:path'

const layoutComponents = [
	'Stack',
	'HStack',
	'VStack',
	'Grid',
	'Flex',
	'Split',
	'Resizable',
	'Dock',
	'Spacer',
	'Toolbar',
	'ToolbarGroup',
	'ToolbarSeparator',
	'ActionBar',
	'TopBar',
	'BottomBar',
	'SidePanel',
	'FloatingPanel',
	'InspectorPanel',
	'StatusBar',
	'ScrollArea',
	'VirtualScrollArea'
]

describe('Макошь UI layout component contracts', () => {
	it('keeps the layout batch documented and exported through the UI kit', () => {
		const uiRoot = fileURLToPath(new URL('.', import.meta.url))
		const barrel = readFileSync(join(uiRoot, 'index.ts'), 'utf8')

		for (const componentName of layoutComponents) {
			const componentPath = layoutComponentPath(componentName)
			expect(existsSync(join(uiRoot, componentPath))).toBe(true)
			expect(existsSync(join(uiRoot, `${componentName}.README.md`))).toBe(true)
			expect(barrel).toContain(`export { default as ${componentName} } from './${componentPath}'`)
		}
	})

	it('keeps layout components UI-only', () => {
		const uiRoot = fileURLToPath(new URL('.', import.meta.url))
		const forbiddenSource = /@\/domains|@\/integrations|@\/platform|fetch\(|useQuery|defineStore|createRouter/

		for (const componentName of layoutComponents) {
			const source = readFileSync(join(uiRoot, layoutComponentPath(componentName)), 'utf8')
			expect(source, componentName).not.toMatch(forbiddenSource)
		}
	})

	it('keeps layout components represented in Storybook', () => {
		const storySource = readFileSync(
			fileURLToPath(new URL('../../../stories/ui/Layout.stories.ts', import.meta.url)),
			'utf8'
		)

		for (const componentName of layoutComponents) {
			expect(storySource).toContain(componentName)
		}
	})
})

function layoutComponentPath(componentName: string): string {
	return componentName === 'StatusBar' ? 'patterns/StatusBar.vue' : `${componentName}.vue`
}
