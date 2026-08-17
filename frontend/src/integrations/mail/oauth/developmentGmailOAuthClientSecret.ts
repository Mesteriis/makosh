import {
	DevelopmentOwnerVaultProvisioningHostV1,
	OwnerVaultActionV1,
	OwnerVaultProvisioningClientV1,
	OwnerVaultSecretClassV1,
} from '../../../platform/vault'

const GMAIL_OAUTH_CLIENT_SECRET_PROVISIONING_CAPABILITY_ID =
	'mail.gmail.oauth-client-secret.credential-provisioning.v1'

const provisionedConfigurationInstances = new Set<string>()
const DEVELOPMENT_MARKER_PREFIX = 'makosh.dev.gmail-oauth-client-secret.v1:'

export async function provisionDevelopmentGmailOAuthClientSecretV1(
	registrationId: string,
	configurationInstanceId: string,
): Promise<void> {
	if (developmentProvisioningMarker(configurationInstanceId)) return
	const vault = new OwnerVaultProvisioningClientV1()
	const host = new DevelopmentOwnerVaultProvisioningHostV1()
	await vault.provisionCustodied({
		targetRegistrationId: registrationId,
		capabilityId: GMAIL_OAUTH_CLIENT_SECRET_PROVISIONING_CAPABILITY_ID,
		configurationInstanceId,
		purposeId: 'mail_gmail_oauth_client_secret',
		secretClass: OwnerVaultSecretClassV1.PROVIDER_CREDENTIAL,
		action: OwnerVaultActionV1.CREATE,
		secretRevision: 1n,
	}, (input) => host.sealGmailOAuthClientSecret(input))
	provisionedConfigurationInstances.add(configurationInstanceId)
	try {
		globalThis.localStorage?.setItem(`${DEVELOPMENT_MARKER_PREFIX}${configurationInstanceId}`, '1')
	} catch {
		// The in-memory marker still prevents duplicate ceremonies in this page lifetime.
	}
}

function developmentProvisioningMarker(configurationInstanceId: string): boolean {
	if (provisionedConfigurationInstances.has(configurationInstanceId)) return true
	try {
		return globalThis.localStorage?.getItem(
			`${DEVELOPMENT_MARKER_PREFIX}${configurationInstanceId}`,
		) === '1'
	} catch {
		return false
	}
}
