import type { ClientModuleBootstrapV1 } from '../../../gen/makosh/gateway/v1/client_bootstrap_pb'
import { OwnerModuleSettingsClientV1 } from '../../../platform/settings/ownerModuleSettingsClient'
import { usePendingManagedIntegrationSettingsActivation } from '../../../platform/settings/usePendingManagedIntegrationSettingsActivation'

const TELEGRAM_MODULE_ID = 'makosh-telegram-runtime'
const TELEGRAM_STORAGE_CAPABILITY_ID = 'telegram.storage.v1'

type TelegramSettingsActivationPortV1 = Pick<
	OwnerModuleSettingsClientV1,
	'applyManagedIntegration'
>

export function useTelegramPendingSettingsActivation(
	module: () => ClientModuleBootstrapV1 | null,
	activation: TelegramSettingsActivationPortV1 = new OwnerModuleSettingsClientV1(),
) {
	return usePendingManagedIntegrationSettingsActivation(module, {
		moduleId: TELEGRAM_MODULE_ID,
		storageCapabilityId: TELEGRAM_STORAGE_CAPABILITY_ID,
		includeTarget: (target, current) =>
			target.configurationInstanceId === current.registrationId,
		targetLabel: 'Telegram account',
	}, activation)
}
