import { create } from '@bufbuild/protobuf'
import { describe, expect, it, vi } from 'vitest'
import {
	ClientModuleBootstrapV1Schema,
	ClientModuleSettingsBootstrapV1Schema,
	ClientSettingValueEntryV1Schema,
	ClientSettingValueV1Schema,
} from '../../../gen/makosh/gateway/v1/client_bootstrap_pb'
import { TelegramAccountResponseSchema } from '../../../gen/makosh/telegram/v1/client_pb'
import { useTelegramAccountManagement } from './useTelegramAccountManagement'

describe('useTelegramAccountManagement', () => {
	it('selects from the provider catalog and uses the selected runtime epoch for restart', async () => {
		const configured = create(TelegramAccountResponseSchema, {
			accountId: 'personal-telegram',
			displayName: 'Personal Telegram',
			state: 'active',
			runtimeState: 'running',
			runtimeEpoch: 7n,
		})
		const ports = {
			list: vi.fn().mockResolvedValue([
				configured,
				create(TelegramAccountResponseSchema, { accountId: 'another-account' }),
			]),
			restart: vi.fn().mockResolvedValue({
				reconfigurationId: 'reconfigure-1',
				state: 'accepted',
			}),
			replay: vi.fn(),
			retire: vi.fn(),
		}
		const controller = useTelegramAccountManagement(
			() => telegramModule(),
			ports,
		)

		await controller.refresh()
		expect(controller.accounts.value).toHaveLength(2)
		expect(controller.account.value?.accountId).toBe('personal-telegram')
		expect(controller.stateLabel.value).toBe('running')

		await controller.restart()
		expect(ports.restart).toHaveBeenCalledWith('personal-telegram', 7n)
		expect(controller.message.value).toContain('reconfigure-1')
	})

	it('lists the provider catalog without deriving account identity from module settings', async () => {
		const ports = {
			list: vi.fn().mockResolvedValue([]),
			restart: vi.fn(),
			replay: vi.fn(),
			retire: vi.fn(),
		}
		const controller = useTelegramAccountManagement(
			() => create(ClientModuleBootstrapV1Schema, {
				registrationId: 'telegram.local',
				moduleId: 'makosh-telegram-runtime',
				capabilityIds: ['telegram.lifecycle.v1'],
			}),
			ports,
		)

		await controller.refresh()
		expect(ports.list).toHaveBeenCalledOnce()
		expect(controller.message.value).toContain('No Telegram accounts')
	})
})

function telegramModule() {
	return create(ClientModuleBootstrapV1Schema, {
		registrationId: 'telegram.local',
		moduleId: 'makosh-telegram-runtime',
		capabilityIds: [
			'telegram.lifecycle.v1',
			'telegram.reconfiguration.v1',
		],
		settings: create(ClientModuleSettingsBootstrapV1Schema, {
			desiredRevision: 2n,
			effectiveRevision: 2n,
			values: [create(ClientSettingValueEntryV1Schema, {
				settingId: 'telegram.account_id',
				value: create(ClientSettingValueV1Schema, {
					value: { case: 'stringValue', value: 'personal-telegram' },
				}),
			})],
		}),
	})
}
