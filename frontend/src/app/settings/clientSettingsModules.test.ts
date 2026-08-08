import { describe, expect, it } from 'vitest'

import { clientSettingsModule, providerModuleIds } from './clientSettingsModules'

describe('client settings module ownership', () => {
	it('selects only the exact provider module', () => {
		const modules = [
			{ moduleId: providerModuleIds.mail },
			{ moduleId: providerModuleIds.telegram },
		] as never

		expect(clientSettingsModule(modules, 'mail')).toMatchObject({
			moduleId: 'makosh-mail-runtime',
		})
		expect(clientSettingsModule(modules, 'zulip')).toBeNull()
	})
})
