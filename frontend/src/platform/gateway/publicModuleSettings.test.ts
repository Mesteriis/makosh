import { create } from '@bufbuild/protobuf'
import { describe, expect, it } from 'vitest'

import {
	ClientModuleBootstrapV1Schema,
	ClientModuleSettingsBootstrapV1Schema,
	ClientSettingValueEntryV1Schema,
	ClientSettingValueV1Schema,
	ClientSettingsApplyStateV1,
} from '../../gen/makosh/gateway/v1/client_bootstrap_pb'
import {
	publicModuleSettingRows,
	publicModuleStringSetting,
	publicModuleSettingsReasonCode,
} from './publicModuleSettings'

describe('public module settings projection', () => {
	it('projects only typed sanitized bootstrap values', () => {
		const rows = publicModuleSettingRows([create(ClientModuleBootstrapV1Schema, {
			registrationId: 'mail.local',
			moduleId: 'makosh-mail-runtime',
			settings: create(ClientModuleSettingsBootstrapV1Schema, {
				applyState: ClientSettingsApplyStateV1.CURRENT,
				values: [create(ClientSettingValueEntryV1Schema, {
					settingId: 'sync_interval',
					displayName: 'Sync interval',
					editable: true,
					value: create(ClientSettingValueV1Schema, {
						value: { case: 'durationMillis', value: 15000n },
					}),
				})],
			}),
		})])

		expect(rows).toEqual([expect.objectContaining({
			moduleId: 'makosh-mail-runtime',
			label: 'Sync interval',
			value: '15000 ms',
			editable: true,
			applyState: 'Current',
			blocked: false,
		})])
	})

	it('does not project entries whose typed value is absent', () => {
		const rows = publicModuleSettingRows([create(ClientModuleBootstrapV1Schema, {
			registrationId: 'mail.local',
			moduleId: 'makosh-mail-runtime',
			settings: create(ClientModuleSettingsBootstrapV1Schema, {
				values: [create(ClientSettingValueEntryV1Schema, {
					settingId: 'missing',
				})],
			}),
		})])

		expect(rows).toEqual([])
	})

	it('reads only an exact sanitized string setting', () => {
		const module = create(ClientModuleBootstrapV1Schema, {
			registrationId: 'mail.local',
			moduleId: 'makosh-mail-runtime',
			settings: {
				values: [
					{
						settingId: 'mail.connection_id',
						value: { value: { case: 'stringValue', value: 'personal-mail' } },
					},
					{
						settingId: 'mail.imap.port',
						value: { value: { case: 'unsignedIntegerValue', value: 993n } },
					},
				],
			},
		})

		expect(publicModuleStringSetting(module, 'mail.connection_id')).toBe('personal-mail')
		expect(publicModuleStringSetting(module, 'mail.imap.port')).toBeNull()
		expect(publicModuleStringSetting(module, 'mail.missing')).toBeNull()
	})

	it('distinguishes absent modules, absent schemas, and current schemas', () => {
		const withoutSchema = create(ClientModuleBootstrapV1Schema, {
			moduleId: 'makosh-mail-runtime',
		})
		const current = create(ClientModuleBootstrapV1Schema, {
			moduleId: 'makosh-mail-runtime',
			settings: create(ClientModuleSettingsBootstrapV1Schema),
		})
		const blocked = create(ClientModuleBootstrapV1Schema, {
			moduleId: 'makosh-mail-runtime',
			settings: create(ClientModuleSettingsBootstrapV1Schema, {
				sanitizedReasonCode: 'owner_action_required',
			}),
		})

		expect(publicModuleSettingsReasonCode(null)).toBe('module_not_registered')
		expect(publicModuleSettingsReasonCode(withoutSchema)).toBe('settings_schema_unavailable')
		expect(publicModuleSettingsReasonCode(current)).toBe('current')
		expect(publicModuleSettingsReasonCode(blocked)).toBe('owner_action_required')
	})
})
