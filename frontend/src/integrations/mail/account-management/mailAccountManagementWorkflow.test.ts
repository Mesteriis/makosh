import { describe, expect, it, vi } from 'vitest'

import {
	MailAccountReadinessV1,
	MailCredentialBindingStateV1,
	MailCredentialPurposeV1,
	type MailAccountStatusV1,
} from '../../../gen/makosh/mail/account/v1/client_pb'
import { MailAccountManagementWorkflowV1 } from './mailAccountManagementWorkflow'

describe('MailAccountManagementWorkflowV1', () => {
	it('uses the account-level lifecycle revision and operation after reload', async () => {
		const retire = vi.fn().mockResolvedValue({ lifecycleRevision: 5n })
		const deleteAccount = vi.fn().mockResolvedValue({ lifecycleRevision: 5n })
		const retry = vi.fn().mockResolvedValue({ lifecycleRevision: 4n })
		const lifecycleStatus = vi.fn().mockResolvedValue({ lifecycleRevision: 4n })
		const workflow = new MailAccountManagementWorkflowV1({
			status: vi.fn(),
			retire,
			delete: deleteAccount,
			retry,
			lifecycleStatus,
		} as never)
		const status = accountStatus({
			lifecycleRevision: 4n,
			lifecycleOperationId: 'mail-operation-4',
		})

		await workflow.retire(status)
		await workflow.delete(status)
		await workflow.retry(status)
		await workflow.refreshLifecycle(status)

		expect(retire).toHaveBeenCalledWith({
			connectionId: 'personal-mail',
			expectedLifecycleRevision: 4n,
		})
		expect(deleteAccount).toHaveBeenCalledWith({
			connectionId: 'personal-mail',
			expectedLifecycleRevision: 4n,
		})
		expect(retry).toHaveBeenCalledWith({
			operationId: 'mail-operation-4',
			connectionId: 'personal-mail',
			expectedLifecycleRevision: 4n,
		})
		expect(lifecycleStatus).toHaveBeenCalledWith({
			operationId: 'mail-operation-4',
			connectionId: 'personal-mail',
		})
	})

	it('rotates an exact password revision before supervised activation', async () => {
		const order: string[] = []
		const provision = vi.fn().mockImplementation(async () => {
			order.push('vault')
			return { secretRevision: 8n }
		})
		const bind = vi.fn().mockImplementation(async () => {
			order.push('bind')
			return { bindingRevision: 3n }
		})
		const applyManagedIntegration = vi.fn().mockImplementation(async () => {
			order.push('activate')
			return { effectiveRevision: 2n }
		})
		const status = vi.fn().mockImplementation(async () => {
			order.push('status')
			return accountStatus()
		})
		const workflow = new MailAccountManagementWorkflowV1({
			status,
			vault: { provision },
			bind,
			activation: { applyManagedIntegration },
		} as never)

		await workflow.rotatePassword({
			registrationId: 'mail-registration',
			storageCapabilityId: 'mail.storage.v1',
			configurationInstanceId: 'personal-mail',
			expectedDesiredRevision: 2n,
			status: accountStatus(),
			purpose: 'imap',
			secretPayload: new TextEncoder().encode('replacement'),
		})

		expect(order).toEqual(['vault', 'bind', 'activate', 'status'])
		expect(provision).toHaveBeenCalledWith(expect.objectContaining({
			capabilityId: 'mail.imap.credential-provisioning.v1',
			secretRevision: 8n,
		}))
		expect(bind).toHaveBeenCalledWith({
			connectionId: 'personal-mail',
			purpose: MailCredentialPurposeV1.MAIL_CREDENTIAL_PURPOSE_IMAP_PASSWORD,
			expectedBindingRevision: 2n,
			credentialRevision: 8n,
		})
	})
})

function accountStatus(
	overrides: Partial<MailAccountStatusV1> = {},
): MailAccountStatusV1 {
	return {
		connectionId: 'personal-mail',
		settingsRevision: 2n,
		runtimeGeneration: 9n,
		readiness: MailAccountReadinessV1.MAIL_ACCOUNT_READINESS_READY,
		connectorProfile: 1,
		syncReadiness: 1,
		deliveryReadiness: 1,
		binding: [{
			purpose: MailCredentialPurposeV1.MAIL_CREDENTIAL_PURPOSE_IMAP_PASSWORD,
			state: MailCredentialBindingStateV1.MAIL_CREDENTIAL_BINDING_STATE_ACTIVE,
			bindingRevision: 2n,
			credentialRevision: 7n,
			appliedRuntimeGeneration: 9n,
			$unknown: [],
		}],
		lifecycleRevision: 0n,
		$unknown: [],
		...overrides,
	} as MailAccountStatusV1
}
