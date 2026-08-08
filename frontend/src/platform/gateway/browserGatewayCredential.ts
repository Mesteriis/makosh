const credentialStorageKey = 'makosh.browser.credential-id.v1'

/** Only the public WebAuthn credential identifier is persisted client-side. */
export function readBrowserGatewayCredentialId(): string {
	try {
		return localStorage.getItem(credentialStorageKey) ?? ''
	} catch {
		return ''
	}
}

export function storeBrowserGatewayCredentialId(credentialId: string): void {
	localStorage.setItem(credentialStorageKey, credentialId)
}
