import type { OwnerDeviceProofV1 } from './ownerDeviceProof'

const DEVELOPMENT_OWNER_DEVICE_PROOF_PATH =
	'/__makosh/owner-device-proof/v1/sign'

type HostFetch = typeof fetch

export class DevelopmentOwnerDeviceProofV1 implements OwnerDeviceProofV1 {
	constructor(
		private readonly fetchImpl: HostFetch =
			(input, init) => fetch(input, init),
	) {}

	async sign(challenge: Uint8Array): Promise<Uint8Array> {
		if (challenge.byteLength !== 32) {
			throw new Error('owner device challenge is invalid')
		}
		const response = await this.fetchImpl(DEVELOPMENT_OWNER_DEVICE_PROOF_PATH, {
			method: 'POST',
			headers: { 'content-type': 'application/json' },
			body: JSON.stringify({ challengeBytes: Array.from(challenge) }),
			credentials: 'same-origin',
			cache: 'no-store',
			redirect: 'error',
		})
		if (!response.ok) {
			throw new Error(`owner device proof host rejected request (${response.status})`)
		}
		const value: unknown = await response.json()
		if (!isRecord(value)) {
			throw new Error('owner device proof host response is invalid')
		}
		return exactBytes(value.signatureRaw, 64)
	}
}

function isRecord(value: unknown): value is Record<string, unknown> {
	return typeof value === 'object' && value !== null && !Array.isArray(value)
}

function exactBytes(value: unknown, length: number): Uint8Array {
	if (!Array.isArray(value)
		|| value.length !== length
		|| value.some((item) =>
			typeof item !== 'number'
			|| !Number.isInteger(item)
			|| item < 0
			|| item > 255)) {
		throw new Error('owner device proof host response is invalid')
	}
	return Uint8Array.from(value)
}
