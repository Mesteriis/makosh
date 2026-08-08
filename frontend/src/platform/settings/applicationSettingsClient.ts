import { formatISO } from 'date-fns'

export type SettingValueKind = 'boolean' | 'integer' | 'string' | 'json'

export type ApplicationSettingValue = boolean | number | string | Record<string, unknown> | unknown[]

export interface ApplicationSetting {
	setting_key: string
	category: string
	value_kind: SettingValueKind
	value: ApplicationSettingValue
	label: string
	description: string
	metadata: Record<string, unknown>
	is_editable: boolean
	updated_by_actor_id: string | null
	created_at: string
	updated_at: string
}

export interface ApplicationSettingsResponse {
	items: ApplicationSetting[]
}

export const FRONTEND_LAYOUT_SETTING_KEY = 'frontend.layout'
export const FRONTEND_SIDEBAR_SETTING_KEY = 'frontend.sidebar'
export const FRONTEND_LOCALE_SETTING_KEY = 'frontend.locale'
export const FRONTEND_THEME_SETTING_KEY = 'frontend.theme'
export const FRONTEND_UI_STATE_SETTING_KEY = 'frontend.ui_state'

const LOCAL_APPLICATION_SETTINGS_STORAGE_KEY = 'makosh-application-settings-v1'

const KNOWN_KEYS = new Set(
	[
		FRONTEND_LAYOUT_SETTING_KEY,
		FRONTEND_SIDEBAR_SETTING_KEY,
		FRONTEND_LOCALE_SETTING_KEY,
		FRONTEND_THEME_SETTING_KEY,
		FRONTEND_UI_STATE_SETTING_KEY
	]
)

interface StoredApplicationSettings {
	[key: string]: ApplicationSetting['value']
}

function emptyState(): StoredApplicationSettings {
	return {}
}

function loadStoredSettings(): StoredApplicationSettings {
	try {
		const raw = localStorage.getItem(LOCAL_APPLICATION_SETTINGS_STORAGE_KEY)
		if (!raw) {
			return emptyState()
		}
		const parsed = JSON.parse(raw) as Record<string, unknown>
		if (typeof parsed !== 'object' || parsed === null || Array.isArray(parsed)) {
			return emptyState()
		}
		const values: StoredApplicationSettings = {}
		for (const [key, value] of Object.entries(parsed)) {
			values[key] = value as ApplicationSetting['value']
		}
		return values
	} catch {
		return emptyState()
	}
}

function persistStoredSettings(values: StoredApplicationSettings): void {
	try {
		localStorage.setItem(
			LOCAL_APPLICATION_SETTINGS_STORAGE_KEY,
			JSON.stringify(values)
		)
	} catch {
		// localStorage may be unavailable in restricted environments.
	}
}

function classifyValueKind(value: ApplicationSetting['value']): SettingValueKind {
	switch (typeof value) {
		case 'boolean':
			return 'boolean'
		case 'number':
			return 'integer'
		case 'string':
			return 'string'
		default:
			return 'json'
	}
}

function makeItem(
	key: string,
	value: ApplicationSetting['value'],
	nowIso: string
): ApplicationSetting {
	return {
		setting_key: key,
		category: KNOWN_KEYS.has(key) ? 'application' : 'application_unknown',
		value_kind: classifyValueKind(value),
		value,
		label: key,
		description: 'first-party application preference',
		metadata: {},
		is_editable: true,
		updated_by_actor_id: null,
		created_at: nowIso,
		updated_at: nowIso
	}
}

export async function fetchApplicationSettings(): Promise<ApplicationSettingsResponse> {
	const values = loadStoredSettings()
	const now = formatISO(new Date())
	return {
		items: Object.entries(values).map(([settingKey, value]) => makeItem(settingKey, value, now))
	}
}

export async function saveApplicationSetting(
	settingKey: string,
	value: ApplicationSetting['value']
): Promise<ApplicationSetting> {
	const values = loadStoredSettings()
	const now = formatISO(new Date())
	values[settingKey] = value
	persistStoredSettings(values)
	const response = makeItem(settingKey, value, now)
	return response
}
