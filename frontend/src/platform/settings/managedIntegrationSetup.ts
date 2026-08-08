import type {
	ApplyOwnerManagedIntegrationSettingsReceiptV1,
	CreateOwnerModuleSettingsTargetReceiptV1,
	UpdateOwnerModuleSettingsReceiptV1,
} from '../../gen/makosh/gateway/v1/owner_module_settings_pb'
import {
	OwnerModuleSettingsClientV1,
	type OwnerSettingInputV1,
} from './ownerModuleSettingsClient'

export type ManagedIntegrationSetupInputV1 = {
	registrationId: string
	expectedDesiredRevision: bigint
	storageCapabilityId: string
	configurationInstanceId: string
	requestHostBridge: boolean
	values: readonly OwnerSettingInputV1[]
	updateOperationId?: Uint8Array
	applyOperationId?: Uint8Array
}

export type ManagedIntegrationSetupReceiptV1 = {
	settings: UpdateOwnerModuleSettingsReceiptV1
	application: ApplyOwnerManagedIntegrationSettingsReceiptV1
}

type ManagedIntegrationSettingsPortV1 = Pick<
	OwnerModuleSettingsClientV1,
	'createConfigurationTarget' | 'updateDesired' | 'applyManagedIntegration'
>

export class ManagedIntegrationSetupV1 {
	constructor(
		private readonly settings: ManagedIntegrationSettingsPortV1 =
			new OwnerModuleSettingsClientV1(),
	) {}

	async createTarget(
		registrationId: string,
		operationId?: Uint8Array,
	): Promise<CreateOwnerModuleSettingsTargetReceiptV1> {
		return this.settings.createConfigurationTarget({ registrationId, operationId })
	}

	async apply(
		input: ManagedIntegrationSetupInputV1,
	): Promise<ManagedIntegrationSetupReceiptV1> {
		const settings = await this.settings.updateDesired({
			operationId: input.updateOperationId,
			registrationId: input.registrationId,
			configurationInstanceId: input.configurationInstanceId,
			expectedDesiredRevision: input.expectedDesiredRevision,
			values: [...input.values],
		})
		const application = await this.settings.applyManagedIntegration({
			operationId: input.applyOperationId,
			registrationId: input.registrationId,
			storageCapabilityId: input.storageCapabilityId,
			configurationInstanceId: input.configurationInstanceId,
			expectedDesiredRevision: settings.desiredRevision,
			requestHostBridge: input.requestHostBridge,
		})
		return { settings, application }
	}
}
