import { describe, expect, it, vi } from 'vitest'

import type { ZulipAccountStatusV1 } from '../../../gen/makosh/zulip/operational/v1/client_pb'
import { ZulipCredentialBindingStateV1 } from '../../../gen/makosh/zulip/account/v1/client_pb'
import { ZulipAccountManagementWorkflowV1 } from './zulipAccountManagementWorkflow'

describe('ZulipAccountManagementWorkflowV1', () => {
	it('rotates the API key with Vault CAS, owner binding CAS and successor activation', async () => {
		const order: string[] = []
		const provision = vi.fn().mockImplementation(async () => {
			order.push('vault')
			return { secretRevision: 5n }
		})
		const bind = vi.fn().mockImplementation(async () => {
			order.push('bind')
			return { bindingRevision: 4n }
		})
		const applyManagedIntegration = vi.fn().mockImplementation(async () => {
			order.push('activate')
			return { effectiveRevision: 2n }
		})
		const status = vi.fn().mockImplementation(async () => {
			order.push('status')
			return accountStatus()
		})
		const workflow = new ZulipAccountManagementWorkflowV1({
			status,
			bind,
			vault: { provision },
			activation: { applyManagedIntegration },
		} as never)

		await workflow.rotateApiKey({
			registrationId: 'zulip-registration',
			accountId: 'work-zulip',
			expectedDesiredRevision: 2n,
			status: accountStatus(),
			secretPayload: new TextEncoder().encode('replacement'),
		})

		expect(order).toEqual(['vault', 'bind', 'activate', 'status'])
		expect(provision).toHaveBeenCalledWith(expect.objectContaining({
			action: 2,
			secretRevision: 5n,
		}))
		expect(bind).toHaveBeenCalledWith({
			accountId: 'work-zulip',
			expectedBindingRevision: 3n,
			credentialRevision: 5n,
		})
	})

	it('retires with the current owner-local binding revision', async () => {
		const retire = vi.fn().mockResolvedValue({ bindingRevision: 4n })
		const workflow = new ZulipAccountManagementWorkflowV1({ retire } as never)
		await workflow.retire(accountStatus())
		expect(retire).toHaveBeenCalledWith({
			accountId: 'work-zulip',
			expectedBindingRevision: 3n,
		})
	})
})

function accountStatus(): ZulipAccountStatusV1 {
	return {
		accountId: 'work-zulip',
		projectionReady: true,
		historyState: 3,
		latestEventSequence: 12n,
		credentialState: ZulipCredentialBindingStateV1.ZULIP_CREDENTIAL_BINDING_STATE_ACTIVE,
		credentialRevision: 4n,
		bindingRevision: 3n,
		appliedRuntimeGeneration: 9n,
	} as ZulipAccountStatusV1
}
