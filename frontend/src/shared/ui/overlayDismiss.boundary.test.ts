import { readFileSync } from 'node:fs'
import { fileURLToPath } from 'node:url'
import { join } from 'node:path'
import { describe, expect, it } from 'vitest'

const mouseLeaveDismissComponents = [
	'Select',
	'DropdownMenu',
	'Popover',
	'Autocomplete',
	'SearchableSelect',
	'SearchableMultiSelect',
	'TreeSelect',
	'Cascader',
	'ContextMenu',
	'SplitButton'
]

describe('Макошь UI mouse-leave overlay dismissal contract', () => {
	it('keeps mouse-leave dismissal centralized in shared UI', () => {
		const uiRoot = fileURLToPath(new URL('.', import.meta.url))
		const source = readFileSync(join(uiRoot, 'useMouseLeaveDismiss.ts'), 'utf8')
		const barrel = readFileSync(join(uiRoot, 'index.ts'), 'utf8')

		expect(source).toContain('DEFAULT_DISMISS_DELAY_MS')
		expect(source).toContain('DEFAULT_BOUNDARY_PADDING_PX = 50')
		expect(source).toContain('getBoundingClientRect()')
		expect(source).toContain('onBeforeUnmount(stopTrackingMouseMove)')
		expect(source).not.toContain('document.addEventListener')
		expect(barrel).toContain("export { useMouseLeaveDismiss } from './useMouseLeaveDismiss'")
	})

	it('applies the shared dismissal behavior to dropdown-like surfaces', () => {
		const uiRoot = fileURLToPath(new URL('.', import.meta.url))

		for (const componentName of mouseLeaveDismissComponents) {
			const componentPath = overlayComponentPath(componentName)
			const source = readFileSync(join(uiRoot, componentPath), 'utf8')
			const mouseLeaveImport = componentPath.startsWith('primitives/')
				? "import { useMouseLeaveDismiss } from '../useMouseLeaveDismiss'"
				: "import { useMouseLeaveDismiss } from './useMouseLeaveDismiss'"

			expect(source, componentName).toContain(mouseLeaveImport)
			expect(source, componentName).toContain('cancelMouseLeaveDismiss')
			expect(source, componentName).toContain('scheduleMouseLeaveDismiss')
			expect(source, componentName).toContain('@mouseleave=')
		}
	})

	it('renders split button menus through a portal so parent overflow cannot clip them', () => {
		const uiRoot = fileURLToPath(new URL('.', import.meta.url))
		const source = readFileSync(join(uiRoot, 'SplitButton.vue'), 'utf8')

		expect(source).toContain('DropdownMenuPortal')
		expect(source).toContain('DropdownMenuContent')
	})

	it('renders tree select menus through a viewport-positioned portal so parent overflow cannot clip them', () => {
		const uiRoot = fileURLToPath(new URL('.', import.meta.url))
		const source = readFileSync(join(uiRoot, 'TreeSelect.vue'), 'utf8')
		const styles = readFileSync(join(uiRoot, 'styles/controls.css'), 'utf8')

		expect(source).toContain('<Teleport to="body">')
		expect(source).toContain('updatePopoverGeometry')
		expect(source).toContain("window.addEventListener('scroll', updatePopoverGeometry, true)")
		expect(styles).toContain('.makosh-tree-select__popover {\n\tposition: fixed;')
	})
})

function overlayComponentPath(componentName: string): string {
	return componentName === 'Select' ? 'primitives/Select.vue' : `${componentName}.vue`
}
