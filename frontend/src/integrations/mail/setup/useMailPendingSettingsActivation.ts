import type { ClientModuleBootstrapV1 } from '../../../gen/makosh/gateway/v1/client_bootstrap_pb'
import { OwnerModuleSettingsClientV1 } from '../../../platform/settings/ownerModuleSettingsClient'
import { usePendingManagedIntegrationSettingsActivation } from '../../../platform/settings/usePendingManagedIntegrationSettingsActivation'

const MAIL_MODULE_ID = 'makosh-mail-runtime'
const MAIL_STORAGE_CAPABILITY_ID = 'mail.storage.v1'

type MailSettingsActivationPortV1 = Pick<OwnerModuleSettingsClientV1, 'applyManagedIntegration'>

export function useMailPendingSettingsActivation(
	module: () => ClientModuleBootstrapV1 | null,
	activation: MailSettingsActivationPortV1 = new OwnerModuleSettingsClientV1(),
) {
	return usePendingManagedIntegrationSettingsActivation(module, {
		moduleId: MAIL_MODULE_ID,
		storageCapabilityId: MAIL_STORAGE_CAPABILITY_ID,
		includeTarget: (target, current) =>
			target.configurationInstanceId !== current.registrationId,
		targetLabel: 'Mail account',
	}, activation)
}
