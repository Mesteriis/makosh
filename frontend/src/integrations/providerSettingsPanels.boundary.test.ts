import { readFileSync } from 'node:fs'

import { describe, expect, it } from 'vitest'

const panels = [
	['mail', 'makosh-mail-runtime'],
	['telegram', 'makosh-telegram-runtime'],
	['whatsapp', 'makosh-whatsapp-runtime'],
	['zulip', 'makosh-zulip-runtime'],
] as const

describe('provider settings panel boundaries', () => {
	it.each(panels)('%s settings read only their exact bootstrap projection', (provider, moduleId) => {
		const componentName = provider === 'whatsapp'
			? 'WhatsAppSettingsPanel.vue'
			: `${provider[0].toUpperCase()}${provider.slice(1)}SettingsPanel.vue`
		const source = readFileSync(
			new URL(`./${provider}/presentation/${componentName}`, import.meta.url),
			'utf8',
		)

		expect(source).toContain(moduleId)
		expect(source).toContain('publicModuleSettingRows')
		expect(source).toContain('ModuleSettingsPanel')
		expect(source).not.toContain('domains/settings')
		expect(source).not.toContain('domains/communications')
		expect(source).not.toContain('/api/v1/')
		expect(source).not.toMatch(/fetch\(|ApiClient|useMutation/)
		for (const [foreignProvider] of panels) {
			if (foreignProvider !== provider) {
				expect(source).not.toContain(`integrations/${foreignProvider}`)
			}
		}
	})
})
