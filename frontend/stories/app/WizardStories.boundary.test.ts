import { describe, expect, it } from 'vitest'
import { existsSync, readFileSync } from 'node:fs'

type WizardStoryExpectation = {
	fileName: string
	storyTitle: string
	modelKey: string
}

const wizardStories: readonly WizardStoryExpectation[] = [
	{
		fileName: 'GmailWizard.stories.ts',
		storyTitle: 'Макошь App/Wizard/Gmail',
		modelKey: 'gmail'
	},
	{
		fileName: 'ICloudMailWizard.stories.ts',
		storyTitle: 'Макошь App/Wizard/iCloud Mail',
		modelKey: 'icloud'
	},
	{
		fileName: 'TelegramWizard.stories.ts',
		storyTitle: 'Макошь App/Wizard/Telegram',
		modelKey: 'telegram'
	},
	{
		fileName: 'WhatsAppWizard.stories.ts',
		storyTitle: 'Макошь App/Wizard/WhatsApp',
		modelKey: 'whatsapp'
	},
	{
		fileName: 'AIProviderWizard.stories.ts',
		storyTitle: 'Макошь App/Wizard/AI Provider',
		modelKey: 'ai'
	}
]

describe('Макошь App wizard Storybook coverage', () => {
	it('keeps provider and AI wizards in the Макошь App/Wizard group', () => {
		for (const story of wizardStories) {
			const storyUrl = new URL(`./${story.fileName}`, import.meta.url)
			expect(existsSync(storyUrl)).toBe(true)

			const source = readFileSync(storyUrl, 'utf8')
			expect(source).toContain(`title: '${story.storyTitle}'`)
			expect(source).toContain(`wizardStoryModels.${story.modelKey}`)
			expect(source).toContain('createWizardStory')
		}
	})

	it('keeps wizard stories as Storybook fixtures without domain runtime imports', () => {
		const storySources = [
			readFileSync(new URL('./wizardStory.ts', import.meta.url), 'utf8'),
			...wizardStories.map((story) => readFileSync(new URL(`./${story.fileName}`, import.meta.url), 'utf8'))
		].join('\n')

		expect(storySources).toContain('Steps')
		expect(storySources).toContain('Мастер подключения')
		expect(storySources).not.toContain('Макошь App Wizard')
		expect(storySources).not.toContain('Callback URL')
		expect(storySources).not.toContain('Vault binding')
		expect(storySources).not.toContain('secret_ref')
		expect(storySources).not.toContain('Runtime route')
		expect(storySources).not.toContain('IMAP host')
		expect(storySources).not.toContain('Access profile')
		expect(storySources).not.toContain("{ label: 'Services'")
		expect(storySources).not.toContain('OAuth')
		expect(storySources).not.toContain('scopes')
		expect(storySources).not.toContain('backend')
		expect(storySources).not.toContain('runtime')
		expect(storySources).not.toContain('provider-command')
		expect(storySources).not.toContain('OpenAI-compatible')
		expect(storySources).not.toContain('Google consent')
		expect(storySources).not.toContain('Владелец видит')
		expect(storySources).toContain('Google Drive')
		expect(storySources).toContain('Google Photos')
		expect(storySources).toContain('Google Keep')
		expect(storySources).toContain('Google Meet')
		expect(storySources).not.toMatch(/use[A-Z][A-Za-z]+Surface/)
		expect(storySources).not.toContain('/queries/')
		expect(storySources).not.toContain('@/domains/')
		expect(storySources).not.toContain('@/integrations/')
	})
})
