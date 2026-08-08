import { describe, expect, it } from 'vitest'
import { existsSync, readFileSync } from 'node:fs'
import { fileURLToPath } from 'node:url'
import { join } from 'node:path'

const communicationComponents = [
	'MessageBubble',
	'ChatInput',
	'TypingIndicator',
	'ReactionBadge',
	'AttachmentChip',
	'Mention',
	'QuotedMessage',
	'MessageStatus',
	'ReadReceipt',
	'DeliveryStatus',
	'ComposerToolbar'
]

describe('Макошь UI communication component contracts', () => {
	it('keeps the communication batch documented and exported through the UI kit', () => {
		const uiRoot = fileURLToPath(new URL('.', import.meta.url))
		const barrel = readFileSync(join(uiRoot, 'index.ts'), 'utf8')

		for (const componentName of communicationComponents) {
			expect(existsSync(join(uiRoot, `${componentName}.vue`))).toBe(true)
			expect(existsSync(join(uiRoot, `${componentName}.README.md`))).toBe(true)
			expect(barrel).toContain(`export { default as ${componentName} } from './${componentName}.vue'`)
		}
	})

	it('keeps communication primitives UI-only and provider-neutral', () => {
		const uiRoot = fileURLToPath(new URL('.', import.meta.url))
		const forbiddenSource = /@\/domains|@\/integrations|@\/platform|fetch\(|useQuery|defineStore|createRouter|localStorage/

		for (const componentName of communicationComponents) {
			const source = readFileSync(join(uiRoot, `${componentName}.vue`), 'utf8')
			expect(source, componentName).not.toMatch(forbiddenSource)
		}
	})

	it('keeps communication components represented in Storybook', () => {
		const storySource = readFileSync(
			fileURLToPath(new URL('../../../stories/ui/Communication.stories.ts', import.meta.url)),
			'utf8'
		)

		for (const componentName of communicationComponents) {
			expect(storySource).toContain(componentName)
		}
	})
})
