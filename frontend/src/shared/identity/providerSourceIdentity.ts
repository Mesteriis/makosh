export type ProviderSourceIdentityBytes = {
	integrationPublicId: Uint8Array
	accountPublicId: Uint8Array
	providerSourceContactPublicId: Uint8Array
}

const PUBLIC_ID_BYTES = 16

export const EMPTY_PROVIDER_SOURCE_NAMES: ReadonlyMap<string, string> = new Map()

export function providerSourceIdentityKey(
	identity: ProviderSourceIdentityBytes | undefined,
): string | undefined {
	if (
		!identity
		|| identity.integrationPublicId.length !== PUBLIC_ID_BYTES
		|| identity.accountPublicId.length !== PUBLIC_ID_BYTES
		|| identity.providerSourceContactPublicId.length !== PUBLIC_ID_BYTES
	) return undefined

	return [
		bytesHex(identity.integrationPublicId),
		bytesHex(identity.accountPublicId),
		bytesHex(identity.providerSourceContactPublicId),
	].join(':')
}

function bytesHex(value: Uint8Array): string {
	return Array.from(value, byte => byte.toString(16).padStart(2, '0')).join('')
}
