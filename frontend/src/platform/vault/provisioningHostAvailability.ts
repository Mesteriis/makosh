export function hasNativeOwnerVaultProvisioningHostV1(): boolean {
	return typeof window !== 'undefined'
		&& '__TAURI_INTERNALS__' in window
}

export function hasAndroidOwnerVaultProvisioningHostV1(): boolean {
	return (
		typeof window !== 'undefined'
		&& typeof window.__MAKOSH_ANDROID_OWNER_VAULT_HOST__ === 'object'
		&& window.__MAKOSH_ANDROID_OWNER_VAULT_HOST__ !== null
		&& typeof window.__MAKOSH_ANDROID_OWNER_VAULT_HOST__.vaultProvisioningHost === 'object'
	)
}

export function hasDevelopmentOwnerVaultProvisioningHostV1(): boolean {
	return import.meta.env.VITE_MAKOSH_DEV_OWNER_VAULT_HOST === '1'
}

export function hasOwnerVaultProvisioningHostV1(): boolean {
	return hasAndroidOwnerVaultProvisioningHostV1()
		|| hasNativeOwnerVaultProvisioningHostV1()
		|| hasDevelopmentOwnerVaultProvisioningHostV1()
}
