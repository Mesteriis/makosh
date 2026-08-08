export async function legacyRecoveryOperationIdV1(
	bundleFingerprintSha256: string,
	sourceHandle: string,
	step: string,
): Promise<Uint8Array> {
	if (!/^[0-9a-f]{64}$/.test(bundleFingerprintSha256)
		|| !/^[0-9a-f]{64}$/.test(sourceHandle)
		|| !/^[a-z][a-z0-9_]{0,63}$/.test(step)) {
		throw new Error('legacy recovery operation identity is invalid')
	}
	const digest = await crypto.subtle.digest(
		'SHA-256',
		new TextEncoder().encode(
			`makosh-legacy-provider-recovery-v1\0${bundleFingerprintSha256}\0${sourceHandle}\0${step}`,
		),
	)
	const operationId = new Uint8Array(digest).slice(0, 16)
	if (operationId.every((byte) => byte === 0)) {
		throw new Error('legacy recovery operation identity is invalid')
	}
	return operationId
}

export async function legacyRecoveryOperationKeyV1(
	bundleFingerprintSha256: string,
	sourceHandle: string,
	step: string,
): Promise<string> {
	const operationId = await legacyRecoveryOperationIdV1(
		bundleFingerprintSha256,
		sourceHandle,
		step,
	)
	return `legacy-recovery-${Array.from(operationId, (byte) =>
		byte.toString(16).padStart(2, '0')).join('')}`
}
