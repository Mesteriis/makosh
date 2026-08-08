import { describe, expect, it } from 'vitest'
import { existsSync, readFileSync } from 'node:fs'
import { fileURLToPath } from 'node:url'
import { join } from 'node:path'

const dataDisplayComponents = [
	'Table',
	'VirtualTable',
	'VirtualList',
	'List',
	'DescriptionList',
	'PropertyGrid',
	'KeyValue',
	'TimelineItem',
	'ActivityItem',
	'Statistic',
	'Metric',
	'Counter',
	'EmptyState',
	'LoadingState',
	'ErrorState',
	'NoDataState',
	'NoSearchResultsState',
	'OfflineState',
	'ComingSoonState'
]

describe('Макошь UI data display component contracts', () => {
	it('keeps the data display batch documented and exported through the UI kit', () => {
		const uiRoot = fileURLToPath(new URL('.', import.meta.url))
		const barrel = readFileSync(join(uiRoot, 'index.ts'), 'utf8')

		for (const componentName of dataDisplayComponents) {
			const componentPath = dataDisplayComponentPath(componentName)
			expect(existsSync(join(uiRoot, componentPath))).toBe(true)
			expect(existsSync(join(uiRoot, `${componentName}.README.md`))).toBe(true)
			expect(barrel).toContain(`export { default as ${componentName} } from './${componentPath}'`)
		}
	})

	it('keeps data display components UI-only', () => {
		const uiRoot = fileURLToPath(new URL('.', import.meta.url))
		const forbiddenSource = /@\/domains|@\/integrations|@\/platform|fetch\(|useQuery|defineStore|createRouter/

		for (const componentName of dataDisplayComponents) {
			const source = readFileSync(join(uiRoot, dataDisplayComponentPath(componentName)), 'utf8')
			expect(source, componentName).not.toMatch(forbiddenSource)
		}
	})

	it('keeps data display components represented in Storybook', () => {
		const storySource = readFileSync(
			fileURLToPath(new URL('../../../stories/ui/DataDisplay.stories.ts', import.meta.url)),
			'utf8'
		)

		for (const componentName of dataDisplayComponents) {
			expect(storySource).toContain(componentName)
		}
	})
})

function dataDisplayComponentPath(componentName: string): string {
	return ['EmptyState', 'LoadingState', 'ErrorState', 'OfflineState', 'ComingSoonState'].includes(componentName)
		? `patterns/${componentName}.vue`
		: `${componentName}.vue`
}
