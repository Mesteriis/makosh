import { beforeEach, describe, expect, it, vi } from 'vitest'
import {
	fetchApplicationSettings,
	saveApplicationSetting
} from '../settings/applicationSettingsClient'
import { defaultThemeSettings } from './settings'
import {
	loadPersistedThemeSettings,
	savePersistedThemeSettings
} from './persistence'

vi.mock('../settings/applicationSettingsClient', () => ({
	FRONTEND_THEME_SETTING_KEY: 'frontend.theme',
	fetchApplicationSettings: vi.fn(),
	saveApplicationSetting: vi.fn()
}))

const storage = new Map<string, string>()

const localStorageDouble = {
	getItem: vi.fn((key: string) => storage.get(key) ?? null),
	setItem: vi.fn((key: string, value: string) => {
		storage.set(key, value)
	})
}

describe('theme persistence', () => {
	beforeEach(() => {
		storage.clear()
		vi.clearAllMocks()
		vi.stubGlobal('localStorage', localStorageDouble)
	})

	it('reports backend save fallback instead of hiding it as autosave success', async () => {
		const settings = {
			...defaultThemeSettings(),
			accentColor: 'violet' as const
		}
		vi.mocked(saveApplicationSetting).mockRejectedValue(new Error('offline'))

		const result = await savePersistedThemeSettings(settings)

		expect(result.source).toBe('local_storage')
		expect(result.errorMessage).toContain('local browser storage')
		expect(JSON.parse(storage.get('makosh-theme-settings') ?? '{}')).toMatchObject({
			accentColor: 'violet'
		})
	})

	it('reports backend load fallback while keeping locally stored settings usable', async () => {
		storage.set(
			'makosh-theme-settings',
			JSON.stringify({
				...defaultThemeSettings(),
				accentColor: 'cyan'
			})
		)
		vi.mocked(fetchApplicationSettings).mockRejectedValue(new Error('offline'))

		const result = await loadPersistedThemeSettings()

		expect(result.source).toBe('local_storage')
		expect(result.errorMessage).toContain('local browser storage')
		expect(result.settings.accentColor).toBe('cyan')
	})
})
