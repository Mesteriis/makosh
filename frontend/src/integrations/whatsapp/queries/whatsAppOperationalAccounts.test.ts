import { create } from '@bufbuild/protobuf'
import { describe, expect, it } from 'vitest'

import {
	ClientModuleBootstrapV1Schema,
	ClientModuleSettingsBootstrapV1Schema,
	ClientSettingValueEntryV1Schema,
	ClientSettingValueV1Schema,
} from '../../../gen/makosh/gateway/v1/client_bootstrap_pb'
import {
	whatsAppOperationalAccountFingerprint,
	whatsAppOperationalQueryAccounts,
	whatsAppOperationalReplayAccounts,
} from './whatsAppOperationalAccounts'

describe('WhatsApp operational account discovery', () => {
	it('discovers only exact enabled capability/account bindings', () => {
		const modules = [
			whatsAppModule('query', ['whatsapp.operational.query.v1'], 'account-1'),
			whatsAppModule('replay', ['whatsapp.operational.realtime.v1'], 'account-2'),
			whatsAppModule('wrong-capability', ['whatsapp.command.v1'], 'account-3'),
			whatsAppModule('wrong-module', ['whatsapp.operational.query.v1'], 'account-4', 'other'),
		]

		expect(whatsAppOperationalQueryAccounts(modules).map(({ accountId }) => accountId))
			.toEqual(['account-1'])
		expect(whatsAppOperationalReplayAccounts(modules).map(({ accountId }) => accountId))
			.toEqual(['account-2'])
		expect(whatsAppOperationalAccountFingerprint(modules))
			.toBe('query:query:account-1|replay:replay:account-2')
	})
})

function whatsAppModule(
	registrationId: string,
	capabilityIds: string[],
	accountId: string,
	moduleId = 'makosh-whatsapp-runtime',
) {
	return create(ClientModuleBootstrapV1Schema, {
		registrationId,
		moduleId,
		sectionsEnabled: true,
		capabilityIds,
		settings: create(ClientModuleSettingsBootstrapV1Schema, {
			values: [create(ClientSettingValueEntryV1Schema, {
				settingId: 'whatsapp.account_id',
				value: create(ClientSettingValueV1Schema, {
					value: { case: 'stringValue', value: accountId },
				}),
			})],
		}),
	})
}
