import { beforeEach, describe, expect, it, vi } from 'vitest'

import {
	MailAccountReadinessV1,
	MailProviderPathReadinessV1,
} from '../../../gen/makosh/mail/account/v1/client_pb'
import { listMailAccounts } from '../api/mailAccountQueryClient'
import { useMailAccountConnections } from './useMailAccountConnections'

vi.mock('../api/mailAccountQueryClient', () => ({
	listMailAccounts: vi.fn(),
}))

const modules = [{
	moduleId: 'makosh-mail-runtime',
	registrationId: 'mail-registration',
	sectionsEnabled: true,
	capabilityIds: ['mail.account.catalog.query.v1'],
}]
const account = {
	connectionId: 'icloud-primary',
	readiness: MailAccountReadinessV1.MAIL_ACCOUNT_READINESS_READY,
	syncReadiness: MailProviderPathReadinessV1.MAIL_PROVIDER_PATH_READINESS_READY,
	deliveryReadiness: MailProviderPathReadinessV1.MAIL_PROVIDER_PATH_READINESS_READY,
}

describe('Mail account connection controller', () => {
	beforeEach(() => {
		vi.mocked(listMailAccounts).mockReset()
	})

	it('retains the last admitted catalog while a transient refresh fails', async () => {
		vi.mocked(listMailAccounts)
			.mockResolvedValueOnce({ accounts: [account] } as never)
			.mockRejectedValueOnce(new Error('runtime busy'))
		const controller = useMailAccountConnections({
			canQuery: () => true,
			modules: () => modules as never,
		})

		await controller.refresh()
		await expect(controller.refresh()).rejects.toThrow('runtime busy')

		expect(controller.connections.value.map(({ connectionId }) => connectionId))
			.toEqual(['icloud-primary'])
	})

	it('clears cached accounts when the catalog capability is no longer admitted', async () => {
		vi.mocked(listMailAccounts).mockResolvedValueOnce({ accounts: [account] } as never)
		let admitted = true
		const controller = useMailAccountConnections({
			canQuery: () => admitted,
			modules: () => modules as never,
		})

		await controller.refresh()
		admitted = false
		await controller.refresh()

		expect(controller.connections.value).toEqual([])
	})
})
