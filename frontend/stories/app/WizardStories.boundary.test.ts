import { describe, expect, it } from 'vitest'
import { existsSync, readFileSync } from 'node:fs'

type WizardStoryExpectation = {
	fileName: string
	storyTitle: string
	modelKey?: string
	componentName?: string
}

const wizardStories: readonly WizardStoryExpectation[] = [
	{
		fileName: 'GmailWizard.stories.ts',
		storyTitle: 'Макошь App/Wizard/Gmail',
		componentName: 'GmailOAuthSetupView'
	},
	{
		fileName: 'ICloudMailWizard.stories.ts',
		storyTitle: 'Макошь App/Wizard/iCloud Mail',
		modelKey: 'icloud'
	},
	{
		fileName: 'TelegramWizard.stories.ts',
		storyTitle: 'Макошь App/Wizard/Telegram',
		componentName: 'TelegramQrPairingView'
	},
	{
		fileName: 'WhatsAppWizard.stories.ts',
		storyTitle: 'Макошь App/Wizard/WhatsApp',
		componentName: 'WhatsAppPairingView'
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
			if (story.modelKey) {
				expect(source).toContain(`wizardStoryModels.${story.modelKey}`)
				expect(source).toContain('createWizardStory')
			} else {
				expect(story.componentName).toBeTruthy()
				expect(source).toContain(story.componentName!)
			}
		}
	})

	it('keeps wizard stories as Storybook fixtures without domain runtime imports', () => {
		const genericWizardStories = wizardStories.filter((story) => story.modelKey)
		const storySources = [
			readFileSync(new URL('./wizardStory.ts', import.meta.url), 'utf8'),
			...genericWizardStories.map((story) => readFileSync(new URL(`./${story.fileName}`, import.meta.url), 'utf8'))
		].join('\n')
		const telegramStory = readFileSync(new URL('./TelegramWizard.stories.ts', import.meta.url), 'utf8')
		const gmailStory = readFileSync(new URL('./GmailWizard.stories.ts', import.meta.url), 'utf8')
		const whatsappStory = readFileSync(new URL('./WhatsAppWizard.stories.ts', import.meta.url), 'utf8')

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
		expect(telegramStory).toContain("../../src/integrations/telegram/presentation/TelegramQrPairingView.vue")
		expect(telegramStory).toContain('Telegram authorization QR code')
		expect(telegramStory).toContain("state: 'waiting_qr_scan'")
		expect(telegramStory).toContain("state: 'waiting_password'")
		expect(telegramStory).not.toMatch(/phone|SMS|bot token/i)
		expect(gmailStory).toContain('GmailOAuthSetupView')
		expect(gmailStory).toContain("stage: 'configuration'")
		expect(gmailStory).toContain("stage: 'authorization'")
		expect(gmailStory).toContain('Continue with Google OAuth')
		expect(whatsappStory).toContain('WhatsAppPairingView')
		expect(whatsappStory).toContain('WhatsApp QR pairing')
		expect(whatsappStory).toContain('Open WhatsApp QR')
		expect(whatsappStory).not.toMatch(/device name|account name/i)
	})
})
