import { create } from '@bufbuild/protobuf'
import { describe, expect, it, vi } from 'vitest'
import {
	ClientModuleBootstrapV1Schema,
	ClientModuleSettingsBootstrapV1Schema,
	ClientSettingValueEntryV1Schema,
	ClientSettingValueV1Schema,
} from '../../../gen/makosh/gateway/v1/client_bootstrap_pb'
import {
	ZulipCredentialBindingStateV1,
} from '../../../gen/makosh/zulip/account/v1/client_pb'
import {
	ZulipAccountStatusV1Schema,
} from '../../../gen/makosh/zulip/operational/v1/client_pb'
import { useZulipAccountManagement } from './useZulipAccountManagement'

describe('useZulipAccountManagement', () => {
	it('loads and retires the exact configured Zulip account', async () => {
		const current = create(ZulipAccountStatusV1Schema, {
			accountId: 'work-zulip',
			projectionReady: true,
			credentialState: ZulipCredentialBindingStateV1.ZULIP_CREDENTIAL_BINDING_STATE_ACTIVE,
			bindingRevision: 3n,
		})
		const retired = create(ZulipAccountStatusV1Schema, {
			accountId: 'work-zulip',
			credentialState: ZulipCredentialBindingStateV1.ZULIP_CREDENTIAL_BINDING_STATE_RETIRED,
			bindingRevision: 4n,
		})
		const workflow = {
			status: vi.fn()
				.mockResolvedValueOnce(current)
				.mockResolvedValueOnce(retired),
			retire: vi.fn().mockResolvedValue({
				accountId: 'work-zulip',
				bindingRevision: 4n,
				state: ZulipCredentialBindingStateV1.ZULIP_CREDENTIAL_BINDING_STATE_RETIRED,
			}),
			rotateApiKey: vi.fn(),
		}
		const controller = useZulipAccountManagement(
			() => zulipModule(),
			workflow as never,
		)

		await controller.refresh()
		expect(workflow.status).toHaveBeenCalledWith('work-zulip')
		expect(controller.stateLabel.value).toBe('Active')

		await controller.retire()
		expect(workflow.retire).toHaveBeenCalledWith(current)
		expect(controller.stateLabel.value).toBe('Retired')
	})

	it('does not query arbitrary Zulip accounts when Settings has no account ID', async () => {
		const workflow = {
			status: vi.fn(),
			retire: vi.fn(),
			rotateApiKey: vi.fn(),
		}
		const controller = useZulipAccountManagement(
			() => create(ClientModuleBootstrapV1Schema, {
				registrationId: 'zulip.local',
				moduleId: 'makosh-zulip-runtime',
				capabilityIds: ['zulip.operational.query.v1'],
			}),
			workflow as never,
		)

		await controller.refresh()
		expect(workflow.status).not.toHaveBeenCalled()
		expect(controller.message.value).toContain('Configure a Zulip account')
	})
})

function zulipModule() {
	return create(ClientModuleBootstrapV1Schema, {
		registrationId: 'zulip.local',
		moduleId: 'makosh-zulip-runtime',
		capabilityIds: [
			'zulip.operational.query.v1',
			'zulip.account.lifecycle.v1',
		],
		settings: create(ClientModuleSettingsBootstrapV1Schema, {
			desiredRevision: 3n,
			effectiveRevision: 3n,
			values: [create(ClientSettingValueEntryV1Schema, {
				settingId: 'zulip.account_id',
				value: create(ClientSettingValueV1Schema, {
					value: { case: 'stringValue', value: 'work-zulip' },
				}),
			})],
		}),
	})
}
