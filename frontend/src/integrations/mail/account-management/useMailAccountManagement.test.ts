import { create } from '@bufbuild/protobuf'
import { describe, expect, it, vi } from 'vitest'
import {
	ClientModuleBootstrapV1Schema,
	ClientModuleSettingsBootstrapV1Schema,
	ClientSettingValueEntryV1Schema,
	ClientSettingValueV1Schema,
} from '../../../gen/makosh/gateway/v1/client_bootstrap_pb'
import {
	MailAccountReadinessV1,
	MailAccountStatusV1Schema,
} from '../../../gen/makosh/mail/account/v1/client_pb'
import { useMailAccountManagement } from './useMailAccountManagement'

describe('useMailAccountManagement', () => {
	it('loads the account catalog and applies lifecycle mutations through Mail contracts', async () => {
		const current = create(MailAccountStatusV1Schema, {
			connectionId: 'personal-mail',
			readiness: MailAccountReadinessV1.MAIL_ACCOUNT_READINESS_READY,
			lifecycleRevision: 4n,
		})
		const workflow = {
			catalog: vi.fn().mockResolvedValue({ accounts: [current] }),
			status: vi.fn(),
			retire: vi.fn().mockResolvedValue({
				operationId: 'retire-mail-1',
			}),
			delete: vi.fn(),
			retry: vi.fn(),
			refreshLifecycle: vi.fn(),
			rotatePassword: vi.fn(),
		}
		const controller = useMailAccountManagement(
			() => mailModule(),
			workflow as never,
		)

		await controller.refresh()
		expect(workflow.catalog).toHaveBeenCalledOnce()
		expect(controller.connectionId.value).toBe('personal-mail')
		expect(controller.stateLabel.value).toBe('Ready')

		await controller.retire()
		expect(workflow.retire).toHaveBeenCalledWith(current)
		expect(controller.stateLabel.value).toBe('Ready')
		expect(controller.message.value).toContain('retire-mail-1')
		expect(controller.message.value).toContain('accepted')
	})

	it('fails closed before transport when the account catalog is not admitted', async () => {
		const workflow = {
			catalog: vi.fn(),
			status: vi.fn(),
			retire: vi.fn(),
			delete: vi.fn(),
			retry: vi.fn(),
			refreshLifecycle: vi.fn(),
			rotatePassword: vi.fn(),
		}
		const controller = useMailAccountManagement(
			() => create(ClientModuleBootstrapV1Schema, {
				registrationId: 'mail.local',
				moduleId: 'makosh-mail-runtime',
				capabilityIds: ['mail.account.query.v1'],
			}),
			workflow as never,
		)

		await controller.refresh()
		expect(workflow.catalog).not.toHaveBeenCalled()
		expect(workflow.status).not.toHaveBeenCalled()
		expect(controller.message.value).toContain('catalog capability is not admitted')
	})
})

function mailModule() {
	return create(ClientModuleBootstrapV1Schema, {
		registrationId: 'mail.local',
		moduleId: 'makosh-mail-runtime',
		capabilityIds: [
			'mail.account.catalog.query.v1',
			'mail.account.query.v1',
			'mail.account.retire.v1',
		],
		settings: create(ClientModuleSettingsBootstrapV1Schema, {
			desiredRevision: 3n,
			effectiveRevision: 3n,
			values: [create(ClientSettingValueEntryV1Schema, {
				settingId: 'mail.connection_id',
				value: create(ClientSettingValueV1Schema, {
					value: { case: 'stringValue', value: 'personal-mail' },
				}),
			})],
		}),
	})
}
