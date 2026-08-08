import { create } from '@bufbuild/protobuf'
import { describe, expect, it } from 'vitest'

import {
	ClientModuleBootstrapV1Schema,
	ClientModuleSettingsBootstrapV1Schema,
	ClientSettingValueEntryV1Schema,
	ClientSettingValueV1Schema,
} from '../../../gen/makosh/gateway/v1/client_bootstrap_pb'
import {
	zulipOperationalAccountFingerprint,
	zulipOperationalQueryAccounts,
	zulipOperationalReplayAccounts,
} from './zulipOperationalAccounts'

describe('Zulip operational account discovery', () => {
	it('discovers only exact enabled capability/account bindings', () => {
		const modules = [
			zulipModule('query', ['zulip.operational.query.v1'], 'account-1'),
			zulipModule('replay', ['zulip.operational.realtime.v1'], 'account-2'),
			zulipModule('wrong-capability', ['zulip.command.v1'], 'account-3'),
			zulipModule('wrong-module', ['zulip.operational.query.v1'], 'account-4', 'other'),
		]

		expect(zulipOperationalQueryAccounts(modules).map(({ accountId }) => accountId))
			.toEqual(['account-1'])
		expect(zulipOperationalReplayAccounts(modules).map(({ accountId }) => accountId))
			.toEqual(['account-2'])
		expect(zulipOperationalAccountFingerprint(modules))
			.toBe('query:query:account-1|replay:replay:account-2')
	})
})

function zulipModule(
	registrationId: string,
	capabilityIds: string[],
	accountId: string,
	moduleId = 'makosh-zulip-runtime',
) {
	return create(ClientModuleBootstrapV1Schema, {
		registrationId,
		moduleId,
		sectionsEnabled: true,
		capabilityIds,
		settings: create(ClientModuleSettingsBootstrapV1Schema, {
			values: [create(ClientSettingValueEntryV1Schema, {
				settingId: 'zulip.account_id',
				value: create(ClientSettingValueV1Schema, {
					value: { case: 'stringValue', value: accountId },
				}),
			})],
		}),
	})
}
