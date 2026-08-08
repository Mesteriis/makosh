import type {
	ApplyOwnerManagedWorkflowSettingsReceiptV1,
	CreateOwnerModuleSettingsTargetReceiptV1,
	UpdateOwnerModuleSettingsReceiptV1,
} from '../../gen/makosh/gateway/v1/owner_module_settings_pb'
import {
	OwnerModuleSettingsClientV1,
	type OwnerSettingInputV1,
} from './ownerModuleSettingsClient'

export type ManagedWorkflowSetupInputV1 = {
	registrationId: string
	expectedDesiredRevision: bigint
	storageCapabilityId: string
	configurationInstanceId: string
	values: readonly OwnerSettingInputV1[]
	updateOperationId?: Uint8Array
	applyOperationId?: Uint8Array
}

export type ManagedWorkflowSetupReceiptV1 = {
	settings: UpdateOwnerModuleSettingsReceiptV1
	application: ApplyOwnerManagedWorkflowSettingsReceiptV1
}

type ManagedWorkflowSettingsPortV1 = Pick<
	OwnerModuleSettingsClientV1,
	'createConfigurationTarget' | 'updateDesired' | 'applyManagedWorkflow'
>

export class ManagedWorkflowSetupV1 {
	constructor(
		private readonly settings: ManagedWorkflowSettingsPortV1 =
			new OwnerModuleSettingsClientV1(),
	) {}

	async createTarget(
		registrationId: string,
		operationId?: Uint8Array,
	): Promise<CreateOwnerModuleSettingsTargetReceiptV1> {
		return this.settings.createConfigurationTarget({ registrationId, operationId })
	}

	async apply(input: ManagedWorkflowSetupInputV1): Promise<ManagedWorkflowSetupReceiptV1> {
		const settings = await this.settings.updateDesired({
			operationId: input.updateOperationId,
			registrationId: input.registrationId,
			configurationInstanceId: input.configurationInstanceId,
			expectedDesiredRevision: input.expectedDesiredRevision,
			values: [...input.values],
		})
		const application = await this.settings.applyManagedWorkflow({
			operationId: input.applyOperationId,
			registrationId: input.registrationId,
			storageCapabilityId: input.storageCapabilityId,
			configurationInstanceId: input.configurationInstanceId,
			expectedDesiredRevision: settings.desiredRevision,
		})
		return { settings, application }
	}
}
