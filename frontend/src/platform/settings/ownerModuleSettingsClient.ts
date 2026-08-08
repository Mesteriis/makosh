import { create } from '@bufbuild/protobuf'
import { createClient, type Client } from '@connectrpc/connect'

import {
	ApplyOwnerManagedIntegrationSettingsV1Schema,
	type ApplyOwnerManagedIntegrationSettingsReceiptV1,
	ApplyOwnerManagedWorkflowSettingsV1Schema,
	type ApplyOwnerManagedWorkflowSettingsReceiptV1,
	CommitOwnerModuleSettingsRequestV1Schema,
	type CommitOwnerModuleSettingsResponseV1,
	type CreateOwnerModuleSettingsTargetReceiptV1,
	CreateOwnerModuleSettingsTargetV1Schema,
	type ExportEffectiveOwnerModuleSettingsReceiptV1,
	ExportEffectiveOwnerModuleSettingsV1Schema,
	OwnerModuleSettingsService,
	OwnerSettingEntryV1Schema,
	type OwnerSettingValueV1,
	OwnerSettingValueV1Schema,
	PrepareOwnerModuleSettingsRequestV1Schema,
	type PrepareOwnerModuleSettingsRequestV1,
	type UpdateOwnerModuleSettingsReceiptV1,
	UpdateOwnerModuleSettingsV1Schema,
} from '../../gen/makosh/gateway/v1/owner_module_settings_pb'
import { createBrowserGatewayConnectTransport } from '../gateway/browserGatewayConnect'
import type { OwnerDeviceProofV1 } from '../gateway/ownerDeviceProof'
import { createOwnerDeviceProofV1 } from '../gateway/ownerDeviceProofFactory'
import {
	resolveOwnerOperationIdV1,
	sameOwnerOperationIdV1,
} from '../gateway/ownerOperationId'

type DefinedOwnerSettingValueV1 = Exclude<
	OwnerSettingValueV1['value'],
	{ case: undefined }
>

export type OwnerSettingInputV1 = {
	settingId: string
	value: DefinedOwnerSettingValueV1
}

export type UpdateOwnerModuleSettingsInputV1 = {
	operationId?: Uint8Array
	registrationId: string
	configurationInstanceId: string
	expectedDesiredRevision: bigint
	values: OwnerSettingInputV1[]
}

export type CreateOwnerModuleSettingsTargetInputV1 = {
	operationId?: Uint8Array
	registrationId: string
}

export type ApplyOwnerManagedIntegrationSettingsInputV1 = {
	operationId?: Uint8Array
	registrationId: string
	storageCapabilityId: string
	configurationInstanceId: string
	expectedDesiredRevision: bigint
	requestHostBridge: boolean
}

export type ApplyOwnerManagedWorkflowSettingsInputV1 = {
	operationId?: Uint8Array
	registrationId: string
	storageCapabilityId: string
	configurationInstanceId: string
	expectedDesiredRevision: bigint
}

export type ExportEffectiveOwnerModuleSettingsInputV1 = {
	operationId?: Uint8Array
	registrationId: string
	configurationInstanceId: string
	expectedEffectiveRevision: bigint
}

export class OwnerModuleSettingsClientV1 {
	constructor(
		private readonly client: Client<typeof OwnerModuleSettingsService> = createClient(
			OwnerModuleSettingsService,
			createBrowserGatewayConnectTransport(),
		),
		private readonly deviceProof: OwnerDeviceProofV1 =
			createOwnerDeviceProofV1(),
	) {}

	async createConfigurationTarget(
		input: CreateOwnerModuleSettingsTargetInputV1,
	): Promise<CreateOwnerModuleSettingsTargetReceiptV1> {
		validateRegistrationId(input.registrationId)
		const response = await this.execute(
			input.operationId,
			{
				case: 'createConfigurationTarget',
				value: create(CreateOwnerModuleSettingsTargetV1Schema, {
					registrationId: input.registrationId,
				}),
			},
		)
		if (response.result.case !== 'created') throw unexpectedResult()
		return response.result.value
	}

	async updateDesired(
		input: UpdateOwnerModuleSettingsInputV1,
	): Promise<UpdateOwnerModuleSettingsReceiptV1> {
		validateRegistrationId(input.registrationId)
		validateIdentifier(input.configurationInstanceId)
		validateRevision(input.expectedDesiredRevision)
		if (input.values.length === 0) throw invalidInput()
		const settingIds = new Set<string>()
		for (const entry of input.values) {
			validateSettingId(entry.settingId)
			if (settingIds.has(entry.settingId)) throw invalidInput()
			settingIds.add(entry.settingId)
			validateSettingValue(entry.value)
		}

		const response = await this.execute(
			input.operationId,
			{
				case: 'updateDesired',
				value: create(UpdateOwnerModuleSettingsV1Schema, {
					registrationId: input.registrationId,
					configurationInstanceId: input.configurationInstanceId,
					expectedDesiredRevision: input.expectedDesiredRevision,
					values: input.values.map((entry) => create(
						OwnerSettingEntryV1Schema,
						{
							settingId: entry.settingId,
							value: create(OwnerSettingValueV1Schema, {
								value: entry.value,
							}),
						},
					)),
				}),
			},
		)
		if (response.result.case !== 'updated') throw unexpectedResult()
		return response.result.value
	}

	async applyManagedIntegration(
		input: ApplyOwnerManagedIntegrationSettingsInputV1,
	): Promise<ApplyOwnerManagedIntegrationSettingsReceiptV1> {
		validateRegistrationId(input.registrationId)
		validateIdentifier(input.storageCapabilityId)
		validateIdentifier(input.configurationInstanceId)
		validateRevision(input.expectedDesiredRevision)

		const response = await this.execute(
			input.operationId,
			{
				case: 'applyManagedIntegration',
				value: create(ApplyOwnerManagedIntegrationSettingsV1Schema, {
					registrationId: input.registrationId,
					storageCapabilityId: input.storageCapabilityId,
					configurationInstanceId: input.configurationInstanceId,
					expectedDesiredRevision: input.expectedDesiredRevision,
					requestHostBridge: input.requestHostBridge,
				}),
			},
		)
		if (response.result.case !== 'applied') throw unexpectedResult()
		return response.result.value
	}

	async applyManagedWorkflow(
		input: ApplyOwnerManagedWorkflowSettingsInputV1,
	): Promise<ApplyOwnerManagedWorkflowSettingsReceiptV1> {
		validateRegistrationId(input.registrationId)
		validateIdentifier(input.storageCapabilityId)
		validateIdentifier(input.configurationInstanceId)
		validateRevision(input.expectedDesiredRevision)

		const response = await this.execute(
			input.operationId,
			{
				case: 'applyManagedWorkflow',
				value: create(ApplyOwnerManagedWorkflowSettingsV1Schema, {
					registrationId: input.registrationId,
					storageCapabilityId: input.storageCapabilityId,
					configurationInstanceId: input.configurationInstanceId,
					expectedDesiredRevision: input.expectedDesiredRevision,
				}),
			},
		)
		if (response.result.case !== 'workflowApplied') throw unexpectedResult()
		return response.result.value
	}

	async exportEffective(
		input: ExportEffectiveOwnerModuleSettingsInputV1,
	): Promise<ExportEffectiveOwnerModuleSettingsReceiptV1> {
		validateRegistrationId(input.registrationId)
		validateIdentifier(input.configurationInstanceId)
		validateRevision(input.expectedEffectiveRevision)

		const response = await this.execute(
			input.operationId,
			{
				case: 'exportEffective',
				value: create(ExportEffectiveOwnerModuleSettingsV1Schema, {
					registrationId: input.registrationId,
					configurationInstanceId: input.configurationInstanceId,
					expectedEffectiveRevision: input.expectedEffectiveRevision,
				}),
			},
		)
		if (response.result.case !== 'exported') throw unexpectedResult()
		return response.result.value
	}

	private async execute(
		requestedOperationId: Uint8Array | undefined,
		operation: Exclude<
			PrepareOwnerModuleSettingsRequestV1['operation'],
			{ case: undefined }
		>,
	): Promise<CommitOwnerModuleSettingsResponseV1> {
		const operationId = resolveOwnerOperationIdV1(requestedOperationId)
		const prepared = await this.client.prepare(create(
			PrepareOwnerModuleSettingsRequestV1Schema,
			{ operationId, operation },
		))
		if (prepared.major !== 1
			|| prepared.challengeId.trim().length === 0
			|| prepared.challengeBytes.byteLength !== 32
			|| prepared.expiresAtUnixMillis <= BigInt(Date.now())) {
			throw new Error('owner settings challenge is invalid')
		}

		const signature = await this.deviceProof.sign(prepared.challengeBytes)
		if (signature.byteLength !== 64) {
			throw new Error('owner device signature is invalid')
		}
		const committed = await this.client.commit(create(
			CommitOwnerModuleSettingsRequestV1Schema,
			{
				challengeId: prepared.challengeId,
				deviceSignatureRaw: signature,
			},
		))
		if (committed.major !== 1
			|| !sameOwnerOperationIdV1(committed.operationId, operationId)) {
			throw new Error('owner settings receipt is invalid')
		}
		return committed
	}
}

function validateRegistrationId(value: string): void {
	validateIdentifier(value)
}

function validateSettingId(value: string): void {
	validateIdentifier(value)
}

function validateIdentifier(value: string): void {
	if (value.trim().length === 0 || value.length > 128) throw invalidInput()
}

function validateRevision(value: bigint): void {
	if (value <= 0n) throw invalidInput()
}

function validateSettingValue(value: DefinedOwnerSettingValueV1): void {
	if ((value.case === 'decimalValue'
			|| value.case === 'stringValue'
			|| value.case === 'enumValue'
			|| value.case === 'resourceReference')
		&& value.value.length > 4096) {
		throw invalidInput()
	}
	if ((value.case === 'durationMillis' || value.case === 'unsignedIntegerValue')
		&& value.value < 0n) {
		throw invalidInput()
	}
}

function invalidInput(): Error {
	return new Error('owner module settings input is invalid')
}

function unexpectedResult(): Error {
	return new Error('owner module settings result is unexpected')
}
