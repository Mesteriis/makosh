import { create } from '@bufbuild/protobuf'
import { describe, expect, it, vi } from 'vitest'

import {
	ClientModuleBootstrapV1Schema,
	ClientModuleSettingsBootstrapV1Schema,
	ClientModuleSettingsTargetBootstrapV1Schema,
	ClientSettingsApplyStateV1,
} from '../../../gen/makosh/gateway/v1/client_bootstrap_pb'
import { useTelegramPendingSettingsActivation } from './useTelegramPendingSettingsActivation'

describe('useTelegramPendingSettingsActivation', () => {
	it('retries only the primary Telegram target after managed readiness failed', async () => {
		const applyManagedIntegration = vi.fn().mockResolvedValue({ applyState: 'current' })
		const module = create(ClientModuleBootstrapV1Schema, {
			registrationId: 'telegram-registration',
			moduleId: 'makosh-telegram-runtime',
			capabilityIds: ['telegram.storage.v1'],
			settings: create(ClientModuleSettingsBootstrapV1Schema, {
				desiredRevision: 16n,
				effectiveRevision: 15n,
				applyState: ClientSettingsApplyStateV1.BLOCKED_CONFIG,
				sanitizedReasonCode: 'managed_readiness_failed',
			}),
			settingsTargets: [
				create(ClientModuleSettingsTargetBootstrapV1Schema, {
					configurationInstanceId: 'unowned-account-target',
					desiredRevision: 2n,
					applyState: ClientSettingsApplyStateV1.PENDING_VALIDATION,
				}),
			],
		})
		const activation = useTelegramPendingSettingsActivation(
			() => module,
			{ applyManagedIntegration },
		)

		expect(activation.pendingCount.value).toBe(1)
		expect(await activation.activate()).toBe(true)
		expect(applyManagedIntegration).toHaveBeenCalledOnce()
		expect(applyManagedIntegration).toHaveBeenCalledWith({
			registrationId: 'telegram-registration',
			storageCapabilityId: 'telegram.storage.v1',
			configurationInstanceId: 'telegram-registration',
			expectedDesiredRevision: 16n,
			requestHostBridge: false,
		})
		expect(activation.pendingCount.value).toBe(0)
		expect(activation.messageTone.value).toBe('success')
	})
})
