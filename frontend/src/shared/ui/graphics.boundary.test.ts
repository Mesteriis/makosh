import { describe, expect, it } from 'vitest'
import { existsSync, readFileSync } from 'node:fs'
import { fileURLToPath } from 'node:url'
import { join } from 'node:path'

const graphicComponents = [
	'ScoreGauge',
	'DonutChart',
	'RadarChart',
	'Sparkline',
	'CandlestickChart'
]

describe('Макошь UI graphics component contracts', () => {
	it('keeps graphics components documented and exported through the UI kit', () => {
		const uiRoot = fileURLToPath(new URL('.', import.meta.url))
		const barrel = readFileSync(join(uiRoot, 'index.ts'), 'utf8')

		for (const componentName of graphicComponents) {
			expect(existsSync(join(uiRoot, `${componentName}.vue`))).toBe(true)
			expect(existsSync(join(uiRoot, `${componentName}.README.md`))).toBe(true)
			expect(barrel).toContain(`export { default as ${componentName} } from './${componentName}.vue'`)
		}
	})

	it('keeps graphics components UI-only and dependency-light', () => {
		const uiRoot = fileURLToPath(new URL('.', import.meta.url))
		const forbiddenSource = /@\/domains|@\/integrations|@\/platform|fetch\(|useQuery|defineStore|createRouter|echarts|chart\.js|highcharts/

		for (const componentName of graphicComponents) {
			const source = readFileSync(join(uiRoot, `${componentName}.vue`), 'utf8')
			expect(source, componentName).not.toMatch(forbiddenSource)
			expect(source, componentName).toContain('<svg')
		}
	})

	it('keeps graphics components represented in Storybook', () => {
		const storySource = readFileSync(
			fileURLToPath(new URL('../../../stories/ui/Graphics.stories.ts', import.meta.url)),
			'utf8'
		)

		for (const componentName of graphicComponents) {
			expect(storySource).toContain(componentName)
		}
	})
})
