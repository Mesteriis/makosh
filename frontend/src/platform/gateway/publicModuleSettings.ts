import {
	ClientSettingsApplyStateV1,
	type ClientModuleBootstrapV1,
	type ClientModuleSettingsTargetBootstrapV1,
	type ClientSettingValueV1,
} from '../../gen/makosh/gateway/v1/client_bootstrap_pb'

export type PublicModuleSettingRow = {
	key: string
	moduleId: string
	settingId: string
	label: string
	value: string
	editable: boolean
	applyState: string
	blocked: boolean
}

export function publicModuleSettingRows(
	modules: readonly ClientModuleBootstrapV1[],
): readonly PublicModuleSettingRow[] {
	return modules.flatMap((module) => {
		const settings = module.settings
		if (!settings) return []
		const applyState = settingsApplyStateLabel(settings.applyState)
		const blocked = settings.applyState !== ClientSettingsApplyStateV1.CURRENT
		return settings.values.flatMap((entry) => entry.value
			? [{
				key: `${module.registrationId}:${entry.settingId}`,
				moduleId: module.moduleId,
				settingId: entry.settingId,
				label: entry.displayName || entry.settingId,
				value: settingValueLabel(entry.value),
				editable: entry.editable,
				applyState,
				blocked,
			}]
			: [])
	})
}

export function publicModuleSettingsTargetRows(
	moduleId: string,
	targets: readonly ClientModuleSettingsTargetBootstrapV1[],
): readonly PublicModuleSettingRow[] {
	return targets.flatMap((target) => {
		const applyState = settingsApplyStateLabel(target.applyState)
		const blocked = target.applyState !== ClientSettingsApplyStateV1.CURRENT
		return target.values.flatMap((entry) => entry.value
			? [{
				key: `${target.configurationInstanceId}:${entry.settingId}`,
				moduleId,
				settingId: entry.settingId,
				label: entry.displayName || entry.settingId,
				value: settingValueLabel(entry.value),
				editable: entry.editable,
				applyState,
				blocked,
			}]
			: [])
	})
}

export function publicModuleStringSetting(
	module: ClientModuleBootstrapV1 | null | undefined,
	settingId: string,
): string | null {
	const value = module?.settings?.values.find((entry) => entry.settingId === settingId)?.value
	return value?.value.case === 'stringValue' ? value.value.value : null
}

export function settingsApplyStateLabel(state: ClientSettingsApplyStateV1): string {
	if (state === ClientSettingsApplyStateV1.CURRENT) return 'Current'
	if (state === ClientSettingsApplyStateV1.PENDING_VALIDATION) return 'Pending validation'
	if (state === ClientSettingsApplyStateV1.PENDING_APPLY) return 'Pending apply'
	if (state === ClientSettingsApplyStateV1.APPLYING) return 'Applying'
	if (state === ClientSettingsApplyStateV1.AWAITING_EXTERNAL_RESTART) return 'Awaiting restart'
	return 'Blocked configuration'
}

export function publicModuleSettingsReasonCode(module: ClientModuleBootstrapV1 | null): string {
	if (!module) return 'module_not_registered'
	if (!module.settings) return 'settings_schema_unavailable'
	return module.settings.sanitizedReasonCode || 'current'
}

function settingValueLabel(value: ClientSettingValueV1): string {
	if (value.value.case === 'booleanValue') return value.value.value ? 'Enabled' : 'Disabled'
	if (value.value.case === 'durationMillis') return `${value.value.value} ms`
	if (value.value.case === 'timestampUnixMillis') {
		const milliseconds = Number(value.value.value)
		return Number.isSafeInteger(milliseconds)
			? new Date(milliseconds).toLocaleString()
			: 'Invalid timestamp'
	}
	return String(value.value.value)
}
