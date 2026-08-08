import { beforeEach, describe, expect, it, vi } from 'vitest'

import {
	fetchApplicationSettings,
	saveApplicationSetting,
	FRONTEND_THEME_SETTING_KEY
} from './applicationSettingsClient'

describe('application settings client', () => {
	const storage = new Map<string, string>()
	const localStorageDouble = {
		getItem: vi.fn((key: string) => storage.get(key) ?? null),
		setItem: vi.fn((key: string, value: string) => {
			storage.set(key, value)
		})
	}

	beforeEach(() => {
		storage.clear()
		vi.clearAllMocks()
		vi.stubGlobal('localStorage', localStorageDouble)
	})

	it('saves and loads app preferences from local storage', async () => {
		const item = await saveApplicationSetting(FRONTEND_THEME_SETTING_KEY, {
			accentColor: 'violet'
		})
		const loaded = await fetchApplicationSettings()

		expect(item.setting_key).toBe(FRONTEND_THEME_SETTING_KEY)
		expect(loaded.items.find((entry) => entry.setting_key === FRONTEND_THEME_SETTING_KEY)).toEqual(item)
		expect(JSON.parse(storage.get('makosh-application-settings-v1') ?? '{}')).toEqual({
			[FRONTEND_THEME_SETTING_KEY]: { accentColor: 'violet' }
		})
	})

	it('returns empty settings when local storage is unavailable', async () => {
		vi.stubGlobal('localStorage', undefined)

		const loaded = await fetchApplicationSettings()
		expect(loaded.items).toEqual([])
	})
})
