import { readFileSync } from 'node:fs'

import { describe, expect, it } from 'vitest'

describe('Mail account wizard composition', () => {
	it('keeps provider setup account-scoped and credential custody outside Settings', () => {
		const wizard = read('./MailAccountSetupPanel.vue')
		const gmailOAuth = read('./GmailOAuthSetupView.vue')
		const management = read('./MailAccountManagementPanel.vue')
		const setup = read('../setup/mailAccountSetupWorkflow.ts')
		const setupController = read('../setup/useMailAccountSetup.ts')
		const catalog = read('../api/mailAccountQueryClient.ts')

		expect(wizard).toContain('<Steps')
		expect(wizard).toContain('Gmail')
		expect(wizard).toContain('iCloud Mail')
		expect(wizard).toContain('Custom IMAP')
		expect(wizard).toContain('setup.authorizeGmail')
		expect(wizard).toContain('GmailOAuthSetupView')
		expect(gmailOAuth).toContain('OAuth redirect URI')
		expect(wizard).toContain("provider !== 'gmail'")
		expect(gmailOAuth).toContain('mailbox identity are selected during OAuth authorization')
		expect(gmailOAuth).toContain(':value="redirectUri" readonly')
		expect(gmailOAuth).toContain('Continue with Google OAuth')
		expect(`${wizard}\n${gmailOAuth}`).not.toMatch(/Returned state|Authorization code/)
		expect(setupController).toContain('redirectGmailOAuthInSameTabV1')
		expect(setupController).toContain('provisionDevelopmentGmailOAuthClientSecretV1')
		expect(management).not.toContain('provisionDevelopmentGmailOAuthClientSecretV1')
		expect(setupController).toContain('current.started.authorizationUrl')
		expect(management).toContain('management.accounts.value')
		expect(management).toContain('management.selectAccount')
		expect(setup).toContain('createTarget')
		expect(setup).toContain('configurationInstanceId')
		expect(catalog).toContain('MailAccountCatalogService')
		expect(catalog).toContain('{ major: 1 }')
		for (const source of [wizard, gmailOAuth, management, setup, setupController, catalog]) {
			expect(source).not.toMatch(/domains\/communications/)
			expect(source).not.toMatch(/password.*settingId|settingId.*password/i)
		}
	})

	it('uses the guarded browser callback for every Gmail authorization surface', () => {
		const entrypoint = read('../../../main.ts')
		const surfaces = [
			read('./MailAccountSetupPanel.vue'),
			read('./MailGmailPermanentDeleteAuthorizationPanel.vue'),
			read('./MailPortabilityPanel.vue'),
			read('../../../app/settings/recovery/LegacyProviderRecoveryPanel.vue'),
		]
		const controllers = [
			read('../setup/useMailAccountSetup.ts'),
			read('../setup/useMailGmailPermanentDeleteAuthorization.ts'),
			read('../portability/useMailAccountPortability.ts'),
			read('../../../app/settings/recovery/useLegacyProviderRecovery.ts'),
		]

		for (const surface of surfaces) {
			expect(surface).not.toMatch(/Returned state|Authorization code|One-time authorization code/)
		}
		for (const controller of controllers) {
			expect(controller).toContain('redirectGmailOAuthInSameTabV1')
		}
		expect(entrypoint).toContain("import('./app/bootstrap')")
		expect(entrypoint).not.toContain("from './app/App.vue'")
	})
})

function read(relativePath: string): string {
	return readFileSync(new URL(relativePath, import.meta.url), 'utf8')
}
