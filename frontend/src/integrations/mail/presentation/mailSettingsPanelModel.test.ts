import { create } from '@bufbuild/protobuf'
import { describe, expect, it } from 'vitest'

import {
	ClientModuleBootstrapV1Schema,
	ClientModuleSettingsBootstrapV1Schema,
	ClientModuleSettingsTargetBootstrapV1Schema,
	ClientSettingsApplyStateV1,
} from '../../../gen/makosh/gateway/v1/client_bootstrap_pb'
import { mailSettingsPanelModel } from './mailSettingsPanelModel'

describe('mailSettingsPanelModel', () => {
	it('uses account-scoped targets instead of the legacy registration target', () => {
		const module = create(ClientModuleBootstrapV1Schema, {
			registrationId: 'mail-registration',
			moduleId: 'makosh-mail-runtime',
			settings: create(ClientModuleSettingsBootstrapV1Schema, {
				desiredRevision: 3n,
				effectiveRevision: 0n,
				applyState: ClientSettingsApplyStateV1.BLOCKED_CONFIG,
				sanitizedReasonCode: 'required_settings_missing',
			}),
			settingsTargets: [
				create(ClientModuleSettingsTargetBootstrapV1Schema, {
					configurationInstanceId: 'mail-registration',
					desiredRevision: 3n,
					applyState: ClientSettingsApplyStateV1.BLOCKED_CONFIG,
				}),
				create(ClientModuleSettingsTargetBootstrapV1Schema, {
					configurationInstanceId: 'account-a',
					desiredRevision: 6n,
					effectiveRevision: 6n,
					applyState: ClientSettingsApplyStateV1.CURRENT,
				}),
				create(ClientModuleSettingsTargetBootstrapV1Schema, {
					configurationInstanceId: 'account-b',
					desiredRevision: 12n,
					effectiveRevision: 12n,
					applyState: ClientSettingsApplyStateV1.CURRENT,
				}),
			],
		})

		expect(mailSettingsPanelModel(module)).toMatchObject({
			applyState: 'Current',
			revision: '6/6 · 12/12',
			reasonCode: 'current',
		})
	})

	it('reports the strongest non-current account state', () => {
		const module = create(ClientModuleBootstrapV1Schema, {
			registrationId: 'mail-registration',
			moduleId: 'makosh-mail-runtime',
			settingsTargets: [
				create(ClientModuleSettingsTargetBootstrapV1Schema, {
					configurationInstanceId: 'account-a',
					applyState: ClientSettingsApplyStateV1.PENDING_VALIDATION,
					sanitizedReasonCode: 'pending',
				}),
				create(ClientModuleSettingsTargetBootstrapV1Schema, {
					configurationInstanceId: 'account-b',
					applyState: ClientSettingsApplyStateV1.BLOCKED_CONFIG,
					sanitizedReasonCode: 'blocked',
				}),
			],
		})

		expect(mailSettingsPanelModel(module)).toMatchObject({
			applyState: 'Blocked configuration',
			reasonCode: 'blocked',
		})
	})
})
