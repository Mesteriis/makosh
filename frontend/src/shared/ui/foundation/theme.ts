// Theme tokens are a shared UI foundation, independent of application composition.
export const uiThemeFamilies = ['base', 'makosh'] as const
export type UiThemeFamily = (typeof uiThemeFamilies)[number]

export const uiThemeModes = ['light', 'dark'] as const
export type UiThemeMode = (typeof uiThemeModes)[number]

export const uiThemeNames = ['base-light', 'base-dark', 'makosh-light', 'makosh-dark'] as const
export type UiThemeName = (typeof uiThemeNames)[number]

export interface UiThemeSelection {
	family: UiThemeFamily
	mode: UiThemeMode
}

export interface UiThemeFamilyOption {
	value: UiThemeFamily
	label: string
	description: string
}

export interface UiThemeModeOption {
	value: UiThemeMode
	label: string
	description: string
}

export interface UiThemeOption extends UiThemeSelection {
	value: UiThemeName
	label: string
	description: string
}

export const defaultUiThemeName: UiThemeName = 'base-light'

export const uiThemeFamilyOptions: UiThemeFamilyOption[] = [
	{
		value: 'base',
		label: 'Base',
		description: 'Neutral Макошь UI foundation.'
	},
	{
		value: 'makosh',
		label: 'Макошь',
		description: 'Signature emerald Макошь UI foundation.'
	}
]

export const uiThemeModeOptions: UiThemeModeOption[] = [
	{
		value: 'light',
		label: 'Light',
		description: 'Light interface variant.'
	},
	{
		value: 'dark',
		label: 'Dark',
		description: 'Dark interface variant.'
	}
]

export const uiThemeOptions: UiThemeOption[] = [
	{
		value: 'base-light',
		family: 'base',
		mode: 'light',
		label: 'Base Light',
		description: 'Clean neutral light surface for daily work.'
	},
	{
		value: 'base-dark',
		family: 'base',
		mode: 'dark',
		label: 'Base Dark',
		description: 'Low-glare neutral dark surface without neon noise.'
	},
	{
		value: 'makosh-light',
		family: 'makosh',
		mode: 'light',
		label: 'Макошь Light',
		description: 'Bright Макошь surface with emerald system accents.'
	},
	{
		value: 'makosh-dark',
		family: 'makosh',
		mode: 'dark',
		label: 'Макошь Dark',
		description: 'Signature emerald intelligence theme for focused context work.'
	}
]

export function isUiThemeFamily(value: unknown): value is UiThemeFamily {
	return typeof value === 'string' && uiThemeFamilies.includes(value as UiThemeFamily)
}

export function isUiThemeMode(value: unknown): value is UiThemeMode {
	return typeof value === 'string' && uiThemeModes.includes(value as UiThemeMode)
}

export function isUiThemeName(value: unknown): value is UiThemeName {
	return typeof value === 'string' && uiThemeNames.includes(value as UiThemeName)
}

export function normalizeUiThemeName(value: unknown): UiThemeName {
	if (isUiThemeName(value)) return value

	return defaultUiThemeName
}

export function themeSelectionToName(family: UiThemeFamily, mode: UiThemeMode): UiThemeName {
	return `${family}-${mode}` as UiThemeName
}

export function themeNameToSelection(theme: UiThemeName): UiThemeSelection {
	const [family, mode] = theme.split('-')
	if (!isUiThemeFamily(family) || !isUiThemeMode(mode)) {
		return themeNameToSelection(defaultUiThemeName)
	}

	return { family, mode }
}
