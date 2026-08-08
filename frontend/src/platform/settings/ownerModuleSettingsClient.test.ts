import { create } from '@bufbuild/protobuf'
import type { Client } from '@connectrpc/connect'
import { describe, expect, it, vi } from 'vitest'

import {
	ApplyOwnerManagedIntegrationSettingsReceiptV1Schema,
	CommitOwnerModuleSettingsResponseV1Schema,
	type CommitOwnerModuleSettingsResponseV1,
	ExportEffectiveOwnerModuleSettingsReceiptV1Schema,
	OwnerModuleSettingsService,
	OwnerSettingEntryV1Schema,
	OwnerSettingValueV1Schema,
	PrepareOwnerModuleSettingsResponseV1Schema,
	UpdateOwnerModuleSettingsReceiptV1Schema,
} from '../../gen/makosh/gateway/v1/owner_module_settings_pb'
import type { OwnerDeviceProofV1 } from '../gateway/ownerDeviceProof'
import { OwnerModuleSettingsClientV1 } from './ownerModuleSettingsClient'

describe('OwnerModuleSettingsClientV1', () => {
	it('updates typed desired settings through fresh owner proof', async () => {
		const operationId = new Uint8Array(16).fill(1)
		const gateway = gatewayClient({
			case: 'updated',
			value: create(UpdateOwnerModuleSettingsReceiptV1Schema, {
				registrationId: 'mail-registration',
				desiredRevision: 2n,
				applyState: 'pending',
			}),
		})
		const deviceProof = proof()
		const receipt = await new OwnerModuleSettingsClientV1(
			gateway.client,
			deviceProof,
		).updateDesired({
			operationId,
			registrationId: 'mail-registration',
			configurationInstanceId: 'mail-target',
			expectedDesiredRevision: 1n,
			values: [
				{
					settingId: 'mail.imap.host',
					value: { case: 'stringValue', value: 'imap.example.test' },
				},
				{
					settingId: 'mail.imap.port',
					value: { case: 'unsignedIntegerValue', value: 993n },
				},
			],
		})

		expect(receipt.desiredRevision).toBe(2n)
		expect(gateway.prepare.mock.calls[0]?.[0]).toMatchObject({
			operationId,
			operation: {
				case: 'updateDesired',
				value: {
					registrationId: 'mail-registration',
					configurationInstanceId: 'mail-target',
					expectedDesiredRevision: 1n,
					values: [
						{
							settingId: 'mail.imap.host',
							value: {
								value: {
									case: 'stringValue',
									value: 'imap.example.test',
								},
							},
						},
						{
							settingId: 'mail.imap.port',
							value: {
								value: {
									case: 'unsignedIntegerValue',
									value: 993n,
								},
							},
						},
					],
				},
			},
		})
		expect(deviceProof.sign).toHaveBeenCalledOnce()
		expect(gateway.commit).toHaveBeenCalledOnce()
	})

	it('applies a managed integration without importing provider contracts', async () => {
		const operationId = new Uint8Array(16).fill(2)
		const gateway = gatewayClient({
			case: 'applied',
			value: create(ApplyOwnerManagedIntegrationSettingsReceiptV1Schema, {
				registrationId: 'mail-registration',
				effectiveRevision: 4n,
				runtimeGeneration: 7n,
				applyState: 'current',
				hostBridgeSocketPath: '/tmp/makosh-mail.sock',
			}),
		})
		const receipt = await new OwnerModuleSettingsClientV1(
			gateway.client,
			proof(),
		).applyManagedIntegration({
			operationId,
			registrationId: 'mail-registration',
			storageCapabilityId: 'mail.storage.v1',
			configurationInstanceId: 'mail-account',
			expectedDesiredRevision: 4n,
			requestHostBridge: true,
		})

		expect(receipt.runtimeGeneration).toBe(7n)
		expect(gateway.prepare.mock.calls[0]?.[0]).toMatchObject({
			operation: {
				case: 'applyManagedIntegration',
				value: {
					registrationId: 'mail-registration',
					storageCapabilityId: 'mail.storage.v1',
					configurationInstanceId: 'mail-account',
					expectedDesiredRevision: 4n,
					requestHostBridge: true,
				},
			},
		})
	})

	it('exports only the typed effective receipt returned by the owner gateway', async () => {
		const operationId = new Uint8Array(16).fill(3)
		const gateway = gatewayClient({
			case: 'exported',
			value: create(ExportEffectiveOwnerModuleSettingsReceiptV1Schema, {
				registrationId: 'mail-registration',
				schemaMajor: 1,
				schemaRevision: 2,
				effectiveRevision: 5n,
				values: [
					create(OwnerSettingEntryV1Schema, {
						settingId: 'mail.imap.host',
						value: create(OwnerSettingValueV1Schema, {
							value: {
								case: 'stringValue',
								value: 'imap.example.test',
							},
						}),
					}),
				],
			}),
		})
		const receipt = await new OwnerModuleSettingsClientV1(
			gateway.client,
			proof(),
		).exportEffective({
			operationId,
			registrationId: 'mail-registration',
			configurationInstanceId: 'mail-target',
			expectedEffectiveRevision: 5n,
		})

		expect(receipt).toMatchObject({
			registrationId: 'mail-registration',
			schemaMajor: 1,
			schemaRevision: 2,
			effectiveRevision: 5n,
			values: [{ settingId: 'mail.imap.host' }],
		})
		expect(gateway.prepare.mock.calls[0]?.[0]).toMatchObject({
			operation: {
				case: 'exportEffective',
				value: {
					registrationId: 'mail-registration',
					configurationInstanceId: 'mail-target',
					expectedEffectiveRevision: 5n,
				},
			},
		})
	})

	it('does not sign an expired owner challenge', async () => {
		const gateway = gatewayClient({
			case: 'updated',
			value: create(UpdateOwnerModuleSettingsReceiptV1Schema),
		}, BigInt(Date.now() - 1))
		const deviceProof = proof()
		const client = new OwnerModuleSettingsClientV1(gateway.client, deviceProof)

		await expect(client.updateDesired({
			operationId: new Uint8Array(16).fill(4),
			registrationId: 'mail-registration',
			configurationInstanceId: 'mail-target',
			expectedDesiredRevision: 1n,
			values: [{
				settingId: 'mail.imap.host',
				value: { case: 'stringValue', value: 'imap.example.test' },
			}],
		})).rejects.toThrow('owner settings challenge is invalid')

		expect(deviceProof.sign).not.toHaveBeenCalled()
		expect(gateway.commit).not.toHaveBeenCalled()
	})

	it('rejects a receipt for another operation', async () => {
		const gateway = gatewayClient({
			case: 'exported',
			value: create(ExportEffectiveOwnerModuleSettingsReceiptV1Schema),
		}, undefined, new Uint8Array(16).fill(9))
		const client = new OwnerModuleSettingsClientV1(gateway.client, proof())

		await expect(client.exportEffective({
			operationId: new Uint8Array(16).fill(5),
			registrationId: 'mail-registration',
			configurationInstanceId: 'mail-target',
			expectedEffectiveRevision: 1n,
		})).rejects.toThrow('owner settings receipt is invalid')
	})
})

function proof(): OwnerDeviceProofV1 & { sign: ReturnType<typeof vi.fn> } {
	return {
		sign: vi.fn().mockResolvedValue(new Uint8Array(64).fill(8)),
	}
}

function gatewayClient(
	result: Exclude<CommitOwnerModuleSettingsResponseV1['result'], { case: undefined }>,
	expiresAtUnixMillis = BigInt(Date.now() + 60_000),
	responseOperationId?: Uint8Array,
) {
	let preparedOperationId = new Uint8Array()
	const prepare = vi.fn().mockImplementation(async (request) => {
		preparedOperationId = request.operationId.slice()
		return create(PrepareOwnerModuleSettingsResponseV1Schema, {
			major: 1,
			challengeId: 'challenge',
			challengeBytes: new Uint8Array(32).fill(7),
			expiresAtUnixMillis,
		})
	})
	const commit = vi.fn().mockImplementation(async () => create(
		CommitOwnerModuleSettingsResponseV1Schema,
		{
			major: 1,
			operationId: responseOperationId ?? preparedOperationId,
			result,
		},
	))
	return {
		prepare,
		commit,
		client: { prepare, commit } as unknown as Client<
			typeof OwnerModuleSettingsService
		>,
	}
}
