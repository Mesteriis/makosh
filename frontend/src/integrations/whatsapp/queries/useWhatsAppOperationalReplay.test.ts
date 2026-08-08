import { create } from '@bufbuild/protobuf'
import { beforeEach, describe, expect, it, vi } from 'vitest'

import {
	ClientModuleBootstrapV1Schema,
	ClientModuleSettingsBootstrapV1Schema,
	ClientSettingValueEntryV1Schema,
	ClientSettingValueV1Schema,
} from '../../../gen/makosh/gateway/v1/client_bootstrap_pb'
import { replayWhatsAppOperationalEvents } from '../api/whatsAppOperationalReplayGateway'
import { useWhatsAppOperationalReplay } from './useWhatsAppOperationalReplay'

vi.mock('../api/whatsAppOperationalReplayGateway', () => ({
	replayWhatsAppOperationalEvents: vi.fn(),
}))

describe('WhatsApp operational replay controller', () => {
	beforeEach(() => {
		vi.clearAllMocks()
		vi.mocked(replayWhatsAppOperationalEvents).mockResolvedValue({
			accountId: 'account-1',
			earliestAvailableSequence: 1n,
			latestAvailableSequence: 3n,
			frame: [
				{ sequence: 1n },
				{ sequence: 2n },
			],
			nextSequence: 2n,
			resetRequired: false,
		} as never)
	})

	it('loads and extends the exact monotonic replay window', async () => {
		const controller = useWhatsAppOperationalReplay({
			canReplay: () => true,
			modules: () => [whatsAppModule()],
		})

		await controller.reconcile()
		vi.mocked(replayWhatsAppOperationalEvents).mockResolvedValueOnce({
			accountId: 'account-1',
			earliestAvailableSequence: 1n,
			latestAvailableSequence: 3n,
			frame: [
				{ sequence: 2n },
				{ sequence: 3n },
			],
			nextSequence: 3n,
			resetRequired: false,
		} as never)
		await controller.loadMore()

		expect(replayWhatsAppOperationalEvents).toHaveBeenNthCalledWith(1, {
			accountId: 'account-1',
		})
		expect(replayWhatsAppOperationalEvents).toHaveBeenNthCalledWith(2, {
			accountId: 'account-1',
			afterSequence: 2n,
		})
		expect(controller.model.value.frames.map(({ sequence }) => sequence))
			.toEqual(['1', '2', '3'])
		expect(controller.model.value.hasMore).toBe(false)
	})

	it('surfaces reset explicitly and fails closed without capability', async () => {
		vi.mocked(replayWhatsAppOperationalEvents).mockResolvedValueOnce({
			accountId: 'account-1',
			earliestAvailableSequence: 5n,
			latestAvailableSequence: 8n,
			frame: [],
			nextSequence: 0n,
			resetRequired: true,
		} as never)
		const reset = useWhatsAppOperationalReplay({
			canReplay: () => true,
			modules: () => [whatsAppModule()],
		})
		await reset.reconcile()
		expect(reset.model.value).toMatchObject({
			state: 'error',
			resetRequired: true,
			hasMore: false,
		})

		vi.clearAllMocks()
		const blocked = useWhatsAppOperationalReplay({
			canReplay: () => false,
			modules: () => [whatsAppModule()],
		})
		await blocked.reconcile()
		expect(blocked.model.value.state).toBe('blocked')
		expect(replayWhatsAppOperationalEvents).not.toHaveBeenCalled()
	})
})

function whatsAppModule() {
	return create(ClientModuleBootstrapV1Schema, {
		registrationId: 'whatsapp-primary',
		moduleId: 'makosh-whatsapp-runtime',
		sectionsEnabled: true,
		capabilityIds: ['whatsapp.operational.realtime.v1'],
		settings: create(ClientModuleSettingsBootstrapV1Schema, {
			values: [create(ClientSettingValueEntryV1Schema, {
				settingId: 'whatsapp.account_id',
				value: create(ClientSettingValueV1Schema, {
					value: { case: 'stringValue', value: 'account-1' },
				}),
			})],
		}),
	})
}
