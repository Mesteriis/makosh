import { create } from '@bufbuild/protobuf'
import { describe, expect, it, vi } from 'vitest'

import {
	ClientModuleBootstrapV1Schema,
	ClientModuleSettingsTargetBootstrapV1Schema,
	ClientSettingsApplyStateV1,
} from '../../../gen/makosh/gateway/v1/client_bootstrap_pb'
import { useMailPendingSettingsActivation } from './useMailPendingSettingsActivation'

describe('useMailPendingSettingsActivation', () => {
	it('applies only non-legacy pending Mail targets in revision order', async () => {
		const applyManagedIntegration = vi.fn().mockResolvedValue({ applyState: 'current' })
		const module = create(ClientModuleBootstrapV1Schema, {
			registrationId: 'mail-registration',
			moduleId: 'makosh-mail-runtime',
			capabilityIds: ['mail.storage.v1'],
			settingsTargets: [
				create(ClientModuleSettingsTargetBootstrapV1Schema, {
					configurationInstanceId: 'mail-registration',
					desiredRevision: 3n,
					applyState: ClientSettingsApplyStateV1.BLOCKED_CONFIG,
				}),
				create(ClientModuleSettingsTargetBootstrapV1Schema, {
					configurationInstanceId: 'account-a',
					desiredRevision: 6n,
					applyState: ClientSettingsApplyStateV1.PENDING_VALIDATION,
				}),
				create(ClientModuleSettingsTargetBootstrapV1Schema, {
					configurationInstanceId: 'account-b',
					desiredRevision: 12n,
					applyState: ClientSettingsApplyStateV1.BLOCKED_CONFIG,
					sanitizedReasonCode: 'managed_readiness_failed',
				}),
				create(ClientModuleSettingsTargetBootstrapV1Schema, {
					configurationInstanceId: 'invalid-account',
					desiredRevision: 7n,
					applyState: ClientSettingsApplyStateV1.BLOCKED_CONFIG,
					sanitizedReasonCode: 'settings_validation_failed',
				}),
			],
		})
		const activation = useMailPendingSettingsActivation(
			() => module,
			{ applyManagedIntegration },
		)

		expect(activation.pendingCount.value).toBe(2)
		expect(await activation.activate()).toBe(true)
		expect(applyManagedIntegration).toHaveBeenCalledTimes(2)
		expect(applyManagedIntegration).toHaveBeenNthCalledWith(1, expect.objectContaining({
			configurationInstanceId: 'account-a',
			expectedDesiredRevision: 6n,
			storageCapabilityId: 'mail.storage.v1',
		}))
		expect(applyManagedIntegration).toHaveBeenNthCalledWith(2, expect.objectContaining({
			configurationInstanceId: 'account-b',
			expectedDesiredRevision: 12n,
		}))
		expect(activation.pendingCount.value).toBe(0)
		expect(activation.canActivate.value).toBe(false)
		expect(activation.messageTone.value).toBe('success')
	})
})
