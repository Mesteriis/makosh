import {
	ClientSettingsApplyStateV1,
	type ClientModuleBootstrapV1,
	type ClientModuleSettingsTargetBootstrapV1,
} from '../../../gen/makosh/gateway/v1/client_bootstrap_pb'
import {
	publicModuleSettingRows,
	publicModuleSettingsReasonCode,
	publicModuleSettingsTargetRows,
	settingsApplyStateLabel,
} from '../../../platform/gateway/publicModuleSettings'
import type { ModuleSettingsPanelModel } from '../../../shared/ui/settings/ModuleSettingsPanelModel'

const MAIL_MODULE_ID = 'makosh-mail-runtime'

export function mailSettingsPanelModel(
	module: ClientModuleBootstrapV1 | null,
): ModuleSettingsPanelModel {
	const owned = module?.moduleId === MAIL_MODULE_ID ? module : null
	const accountTargets = owned?.settingsTargets.filter((target) =>
		target.configurationInstanceId !== owned.registrationId) ?? []
	if (owned && accountTargets.length > 0) return accountTargetModel(owned, accountTargets)
	const settings = owned?.settings
	return baseModel({
		registered: Boolean(owned),
		applyState: settings ? settingsApplyStateLabel(settings.applyState) : 'No schema',
		revision: settings ? `${settings.effectiveRevision}/${settings.desiredRevision}` : '—',
		reasonCode: publicModuleSettingsReasonCode(owned),
		settings: publicModuleSettingRows(owned ? [owned] : []),
	})
}

function accountTargetModel(
	module: ClientModuleBootstrapV1,
	targets: readonly ClientModuleSettingsTargetBootstrapV1[],
): ModuleSettingsPanelModel {
	const authoritativeTarget = [...targets].sort(
		(left, right) => statePriority(right.applyState) - statePriority(left.applyState),
	)[0]
	const applyState = authoritativeTarget?.applyState ?? ClientSettingsApplyStateV1.BLOCKED_CONFIG
	return baseModel({
		registered: true,
		applyState: settingsApplyStateLabel(applyState),
		revision: targets
			.map((target) => `${target.effectiveRevision}/${target.desiredRevision}`)
			.join(' · '),
		reasonCode: applyState === ClientSettingsApplyStateV1.CURRENT
			? 'current'
			: authoritativeTarget?.sanitizedReasonCode || 'account_settings_not_current',
		settings: publicModuleSettingsTargetRows(module.moduleId, targets),
	})
}

function baseModel(input: Pick<ModuleSettingsPanelModel,
	'registered' | 'applyState' | 'revision' | 'reasonCode' | 'settings'>): ModuleSettingsPanelModel {
	return {
		title: 'Mail',
		description: 'Mail owns provider accounts, synchronization and outbound delivery settings.',
		icon: 'tabler:mail',
		tone: 'mail',
		moduleId: MAIL_MODULE_ID,
		...input,
	}
}

function statePriority(state: ClientSettingsApplyStateV1): number {
	if (state === ClientSettingsApplyStateV1.BLOCKED_CONFIG
		|| state === ClientSettingsApplyStateV1.UNSPECIFIED) return 6
	if (state === ClientSettingsApplyStateV1.AWAITING_EXTERNAL_RESTART) return 5
	if (state === ClientSettingsApplyStateV1.APPLYING) return 4
	if (state === ClientSettingsApplyStateV1.PENDING_APPLY) return 3
	if (state === ClientSettingsApplyStateV1.PENDING_VALIDATION) return 2
	return 1
}
