import type {
	BeginLegacyProviderRecoveryStepInputV1,
	CompleteLegacyProviderRecoveryStepInputV1,
	FinishLegacyProviderRecoveryCandidateInputV1,
	LegacyProviderRecoveryCandidateV1,
	LegacyProviderRecoveryHostV1,
	LegacyProviderRecoveryPlanV1,
	LegacyProviderRecoverySourceV1,
	LegacyProviderRecoveryStepV1,
	SealLegacyProviderRecoverySourceInputV1,
} from './legacyProviderRecoveryHost'

const DEVELOPMENT_RECOVERY_BASE_PATH = '/__makosh/legacy-provider-recovery/v1'

type HostFetch = typeof fetch

export class DevelopmentLegacyProviderRecoveryHostV1
implements LegacyProviderRecoveryHostV1 {
	constructor(
		private readonly fetchImpl: HostFetch = (input, init) => fetch(input, init),
	) {}

	async start(): Promise<LegacyProviderRecoveryPlanV1> {
		const value = await this.post('/start', {})
		const candidates = requiredArray(value.candidates).map(candidate)
		if (value.schemaRevision !== 1
			|| candidates.length !== 3
			|| value.counts === undefined
			|| !isRecord(value.counts)
			|| value.counts.gmailActive !== 1
			|| value.counts.icloudActive !== 1
			|| value.counts.telegramUserActive !== 1
			|| value.counts.gmailDeleted !== 2) {
			throw invalidResponse()
		}
		return {
			schemaRevision: 1,
			recoverySessionId: exactHex(value.recoverySessionId, 32),
			bundleFingerprintSha256: exactHex(value.bundleFingerprintSha256, 64),
			counts: {
				gmailActive: 1,
				icloudActive: 1,
				telegramUserActive: 1,
				gmailDeleted: 2,
			},
			candidates,
		}
	}

	async source(
		recoverySessionId: string,
		sourceHandle: string,
	): Promise<LegacyProviderRecoverySourceV1> {
		const value = await this.post('/source', { recoverySessionId, sourceHandle })
		switch (value.kind) {
			case 'gmail':
				return {
					kind: 'gmail',
					sourceHandle: exactHandle(value.sourceHandle),
					accountId: requiredString(value.accountId),
					displayName: requiredString(value.displayName),
					email: requiredString(value.email),
					oauthClientId: requiredString(value.oauthClientId),
					oauthRedirectUri: requiredString(value.oauthRedirectUri),
				}
			case 'icloud':
				return {
					kind: 'icloud',
					sourceHandle: exactHandle(value.sourceHandle),
					accountId: requiredString(value.accountId),
					displayName: requiredString(value.displayName),
					email: requiredString(value.email),
					imapHost: requiredString(value.imapHost),
					imapPort: requiredPort(value.imapPort),
					username: requiredString(value.username),
				}
			case 'telegram_user':
				return {
					kind: 'telegram_user',
					sourceHandle: exactHandle(value.sourceHandle),
					accountId: requiredString(value.accountId),
					displayName: requiredString(value.displayName),
					externalAccountId: optionalString(value.externalAccountId),
					apiId: BigInt(requiredPositiveInteger(value.apiId)),
				}
			default:
				throw invalidResponse()
		}
	}

	async sealSource(
		input: SealLegacyProviderRecoverySourceInputV1,
	) {
		const value = await this.post('/seal-source', {
			recoverySessionId: input.recoverySessionId,
			sourceHandle: input.sourceHandle,
			secretPurpose: input.secretPurpose,
			hostSessionId: input.hostSessionId,
			operationId: Array.from(input.operationId),
			action: input.action,
			secretClass: input.secretClass,
			authorized: {
				vaultRuntimeGeneration: input.authorized.vaultRuntimeGeneration.toString(),
				vaultHpkePublicKeyX25519: Array.from(input.authorized.vaultHpkePublicKeyX25519),
				audienceRegistrationId: input.authorized.audienceRegistrationId,
				audienceRuntimeInstanceId: input.authorized.audienceRuntimeInstanceId,
				audienceRuntimeGeneration: input.authorized.audienceRuntimeGeneration.toString(),
				audienceGrantEpoch: input.authorized.audienceGrantEpoch.toString(),
				leaseRequestId: Array.from(input.authorized.leaseRequestId),
				leaseOperationDigestSha256: Array.from(input.authorized.leaseOperationDigestSha256),
				commandRequestId: Array.from(input.authorized.commandRequestId),
				leaseResponseHpkeEncappedKey: Array.from(input.authorized.leaseResponseHpkeEncappedKey),
				leaseResponseCiphertext: Array.from(input.authorized.leaseResponseCiphertext),
				leaseResponseHpkeAuthenticationTag: Array.from(
					input.authorized.leaseResponseHpkeAuthenticationTag,
				),
			},
		})
		return {
			operationDigestSha256: bytes(value.operationDigestSha256, 32),
			hpkeEncappedKey: bytes(value.hpkeEncappedKey, 32),
			ciphertext: bytes(value.ciphertext),
			hpkeAuthenticationTag: bytes(value.hpkeAuthenticationTag, 16),
		}
	}

	async beginStep(
		input: BeginLegacyProviderRecoveryStepInputV1,
	): Promise<LegacyProviderRecoveryStepV1> {
		const value = await this.post('/begin-step', {
			recoverySessionId: input.recoverySessionId,
			sourceHandle: input.sourceHandle,
			stepIdentifier: input.stepIdentifier,
			targetConfigurationInstanceId: input.targetConfigurationInstanceId,
			explicitRetry: input.explicitRetry,
		})
		return {
			disposition: recoveryStepDisposition(value.disposition),
			operationId: bytes(value.operationId, 16),
			targetConfigurationInstanceId: optionalBoundedIdentifier(
				value.targetConfigurationInstanceId,
			),
			publicRevision: optionalUnsigned(value.publicRevision),
		}
	}

	async completeStep(input: CompleteLegacyProviderRecoveryStepInputV1): Promise<void> {
		await this.post('/complete-step', {
			recoverySessionId: input.recoverySessionId,
			sourceHandle: input.sourceHandle,
			stepIdentifier: input.stepIdentifier,
			operationId: Array.from(input.operationId),
			targetConfigurationInstanceId: input.targetConfigurationInstanceId,
			publicRevision: input.publicRevision?.toString(),
		})
	}

	async finishCandidate(
		input: FinishLegacyProviderRecoveryCandidateInputV1,
	): Promise<void> {
		await this.post('/finish-candidate', {
			recoverySessionId: input.recoverySessionId,
			sourceHandle: input.sourceHandle,
			targetConfigurationInstanceId: input.targetConfigurationInstanceId,
			terminalState: input.terminalState,
		})
	}

	async cancel(recoverySessionId: string): Promise<void> {
		await this.post('/cancel', { recoverySessionId })
	}

	private async post(path: string, body: Record<string, unknown>): Promise<Record<string, unknown>> {
		const response = await this.fetchImpl(`${DEVELOPMENT_RECOVERY_BASE_PATH}${path}`, {
			method: 'POST',
			headers: { 'content-type': 'application/json' },
			body: JSON.stringify(body),
			credentials: 'same-origin',
			cache: 'no-store',
			redirect: 'error',
		})
		if (!response.ok) {
			throw new Error(`legacy provider recovery host rejected request (${response.status})`)
		}
		const value: unknown = await response.json()
		if (!isRecord(value)) throw invalidResponse()
		return value
	}
}

function candidate(value: unknown): LegacyProviderRecoveryCandidateV1 {
	if (!isRecord(value)
		|| !isCandidateKind(value.kind)
		|| !isRecoveryState(value.state)) {
		throw invalidResponse()
	}
	return {
		sourceHandle: exactHandle(value.sourceHandle),
		kind: value.kind,
		state: value.state,
		receiptTerminalState: optionalTerminalState(value.terminalState),
	}
}

function isCandidateKind(value: unknown): value is LegacyProviderRecoveryCandidateV1['kind'] {
	return value === 'gmail' || value === 'icloud' || value === 'telegram_user'
}

function isRecoveryState(value: unknown): value is LegacyProviderRecoveryCandidateV1['state'] {
	return value === 'ready_to_apply'
		|| value === 'reauthorization_required'
		|| value === 'qr_authorization_required'
}

function optionalTerminalState(
	value: unknown,
): LegacyProviderRecoveryCandidateV1['receiptTerminalState'] {
	if (value === null || value === undefined) return undefined
	if (value === 'completed'
		|| value === 'reauthorization_required'
		|| value === 'qr_authorization_required'
		|| value === 'blocked_source'
		|| value === 'blocked_config'
		|| value === 'outcome_unknown') {
		return value
	}
	throw invalidResponse()
}

function isRecord(value: unknown): value is Record<string, unknown> {
	return typeof value === 'object' && value !== null && !Array.isArray(value)
}

function requiredArray(value: unknown): unknown[] {
	if (!Array.isArray(value)) throw invalidResponse()
	return value
}

function requiredString(value: unknown): string {
	if (typeof value !== 'string' || !value.trim() || value.length > 4096) {
		throw invalidResponse()
	}
	return value
}

function optionalString(value: unknown): string {
	if (typeof value !== 'string' || value.length > 4096) throw invalidResponse()
	return value
}

function exactHex(value: unknown, length: number): string {
	if (typeof value !== 'string'
		|| value.length !== length
		|| !/^[0-9a-f]+$/.test(value)) {
		throw invalidResponse()
	}
	return value
}

function exactHandle(value: unknown): string {
	return exactHex(value, 64)
}

function requiredPositiveInteger(value: unknown): number {
	if (typeof value !== 'number' || !Number.isSafeInteger(value) || value <= 0) {
		throw invalidResponse()
	}
	return value
}

function requiredPort(value: unknown): number {
	const port = requiredPositiveInteger(value)
	if (port > 65_535) throw invalidResponse()
	return port
}

function bytes(value: unknown, exactLength?: number): Uint8Array {
	if (!Array.isArray(value)
		|| value.some((item) => !Number.isInteger(item) || item < 0 || item > 255)
		|| (exactLength !== undefined && value.length !== exactLength)) {
		throw invalidResponse()
	}
	return Uint8Array.from(value)
}

function optionalBoundedIdentifier(value: unknown): string | undefined {
	if (value === null || value === undefined) return undefined
	if (typeof value !== 'string'
		|| !/^[A-Za-z0-9._:-]{1,256}$/.test(value)) {
		throw invalidResponse()
	}
	return value
}

function optionalUnsigned(value: unknown): bigint | undefined {
	if (value === null || value === undefined) return undefined
	if (typeof value !== 'string' || !/^(0|[1-9][0-9]*)$/.test(value)) {
		throw invalidResponse()
	}
	const revision = BigInt(value)
	if (revision > 18_446_744_073_709_551_615n) throw invalidResponse()
	return revision
}

function recoveryStepDisposition(
	value: unknown,
): LegacyProviderRecoveryStepV1['disposition'] {
	if (value === 'execute' || value === 'completed' || value === 'outcome_unknown') {
		return value
	}
	throw invalidResponse()
}

function invalidResponse(): Error {
	return new Error('legacy provider recovery host response is invalid')
}
