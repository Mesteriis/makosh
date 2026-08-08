import { describe, expect, it } from 'vitest'
import { existsSync, readFileSync } from 'node:fs'
import { fileURLToPath } from 'node:url'
import { join } from 'node:path'

const feedbackComponents = [
	'Notification',
	'Banner',
	'Alert',
	'InlineMessage',
	'Spinner',
	'ProgressBar',
	'CircularProgress',
	'StatusIndicator',
	'PresenceIndicator',
	'LoadingOverlay'
]

describe('Макошь UI feedback component contracts', () => {
	it('keeps the feedback batch documented and exported through the UI kit', () => {
		const uiRoot = fileURLToPath(new URL('.', import.meta.url))
		const barrel = readFileSync(join(uiRoot, 'index.ts'), 'utf8')

		for (const componentName of feedbackComponents) {
			const componentPath = feedbackComponentPath(componentName)
			expect(existsSync(join(uiRoot, componentPath))).toBe(true)
			expect(existsSync(join(uiRoot, `${componentName}.README.md`))).toBe(true)
			expect(barrel).toContain(`export { default as ${componentName} } from './${componentPath}'`)
		}
	})

	it('keeps feedback components UI-only', () => {
		const uiRoot = fileURLToPath(new URL('.', import.meta.url))
		const forbiddenSource = /@\/domains|@\/integrations|@\/platform|fetch\(|useQuery|defineStore|createRouter/

		for (const componentName of feedbackComponents) {
			const source = readFileSync(join(uiRoot, feedbackComponentPath(componentName)), 'utf8')
			expect(source, componentName).not.toMatch(forbiddenSource)
		}
	})

	it('keeps feedback components represented in Storybook', () => {
		const storySource = readFileSync(
			fileURLToPath(new URL('../../../stories/ui/Feedback.stories.ts', import.meta.url)),
			'utf8'
		)

		for (const componentName of feedbackComponents) {
			expect(storySource).toContain(componentName)
		}
	})
})

function feedbackComponentPath(componentName: string): string {
	return componentName === 'Alert' ? 'primitives/Alert.vue' : `${componentName}.vue`
}
