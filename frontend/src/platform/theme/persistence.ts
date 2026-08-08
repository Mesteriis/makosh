import {
	FRONTEND_THEME_SETTING_KEY,
	fetchApplicationSettings,
	saveApplicationSetting
} from '../settings/applicationSettingsClient'
import { defaultThemeSettings, parseThemeSettings, type ThemeSettings } from './settings'

const LOCAL_STORAGE_KEY = 'makosh-theme-settings'

export type ThemePersistenceSource = 'application_settings' | 'local_storage'

export interface PersistedThemeSettings {
	settings: ThemeSettings
	source: ThemePersistenceSource
	errorMessage: string
}

const LOCAL_STORAGE_WARNING =
	'Theme settings persisted in local browser storage only.'

export async function loadPersistedThemeSettings(): Promise<PersistedThemeSettings> {
	try {
		const response = await fetchApplicationSettings()
		const setting = response.items.find((item) => item.setting_key === FRONTEND_THEME_SETTING_KEY)
		if (setting) {
			const parsed = parseThemeSettings(setting.value)
			saveLocalThemeSettings(parsed)
			return {
				settings: parsed,
				source: 'application_settings',
				errorMessage: ''
			}
		}
	} catch {
		return {
			settings: loadLocalThemeSettings(),
			source: 'local_storage',
			errorMessage: LOCAL_STORAGE_WARNING
		}
	}

	return {
		settings: loadLocalThemeSettings(),
		source: 'application_settings',
		errorMessage: ''
	}
}

export async function savePersistedThemeSettings(settings: ThemeSettings): Promise<PersistedThemeSettings> {
	try {
		const saved = await saveApplicationSetting(FRONTEND_THEME_SETTING_KEY, settings)
		const parsed = parseThemeSettings(saved.value)
		saveLocalThemeSettings(parsed)
		return {
			settings: parsed,
			source: 'application_settings',
			errorMessage: ''
		}
	} catch {
		saveLocalThemeSettings(settings)
		return {
			settings,
			source: 'local_storage',
			errorMessage: LOCAL_STORAGE_WARNING
		}
	}
}

export function loadLocalThemeSettings(): ThemeSettings {
	try {
		const raw = localStorage.getItem(LOCAL_STORAGE_KEY)
		return raw ? parseThemeSettings(JSON.parse(raw)) : defaultThemeSettings()
	} catch {
		return defaultThemeSettings()
	}
}

function saveLocalThemeSettings(settings: ThemeSettings): void {
	try {
		localStorage.setItem(LOCAL_STORAGE_KEY, JSON.stringify(settings))
	} catch {
		// localStorage may be unavailable; runtime theme still applies in memory.
	}
}
