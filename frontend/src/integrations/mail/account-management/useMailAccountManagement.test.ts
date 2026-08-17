import { create } from '@bufbuild/protobuf'
import { describe, expect, it, vi } from 'vitest'
import {
	ClientModuleBootstrapV1Schema,
	ClientModuleSettingsBootstrapV1Schema,
	ClientSettingValueEntryV1Schema,
	ClientSettingValueV1Schema,
} from '../../../gen/makosh/gateway/v1/client_bootstrap_pb'
import {
	MailAccountReadinessV1,
	MailConnectorProfileV1,
	MailAccountStatusV1Schema,
} from '../../../gen/makosh/mail/account/v1/client_pb'
import { GmailOAuthStartedV1Schema } from '../../../gen/makosh/mail/v1/client_pb'
import { useMailAccountManagement } from './useMailAccountManagement'

describe('useMailAccountManagement', () => {
	it('loads the account catalog and applies lifecycle mutations through Mail contracts', async () => {
		const current = create(MailAccountStatusV1Schema, {
			connectionId: 'personal-mail',
			readiness: MailAccountReadinessV1.MAIL_ACCOUNT_READINESS_READY,
			lifecycleRevision: 4n,
		})
		const workflow = {
			catalog: vi.fn().mockResolvedValue({ accounts: [current] }),
			status: vi.fn(),
			retire: vi.fn().mockResolvedValue({
				operationId: 'retire-mail-1',
			}),
			delete: vi.fn(),
			retry: vi.fn(),
			refreshLifecycle: vi.fn(),
			rotatePassword: vi.fn(),
		}
		const controller = useMailAccountManagement(
			() => mailModule(),
			workflow as never,
		)

		await controller.refresh()
		expect(workflow.catalog).toHaveBeenCalledOnce()
		expect(controller.connectionId.value).toBe('personal-mail')
		expect(controller.stateLabel.value).toBe('Ready')

		await controller.retire()
		expect(workflow.retire).toHaveBeenCalledWith(current)
		expect(controller.stateLabel.value).toBe('Ready')
		expect(controller.message.value).toContain('retire-mail-1')
		expect(controller.message.value).toContain('accepted')
	})

	it('fails closed before transport when the account catalog is not admitted', async () => {
		const workflow = {
			catalog: vi.fn(),
			status: vi.fn(),
			retire: vi.fn(),
			delete: vi.fn(),
			retry: vi.fn(),
			refreshLifecycle: vi.fn(),
			rotatePassword: vi.fn(),
		}
		const controller = useMailAccountManagement(
			() => create(ClientModuleBootstrapV1Schema, {
				registrationId: 'mail.local',
				moduleId: 'makosh-mail-runtime',
				capabilityIds: ['mail.account.query.v1'],
			}),
			workflow as never,
		)

		await controller.refresh()
		expect(workflow.catalog).not.toHaveBeenCalled()
		expect(workflow.status).not.toHaveBeenCalled()
		expect(controller.message.value).toContain('catalog capability is not admitted')
	})

	it('resumes Google OAuth for an admitted configuration-only Gmail account', async () => {
		const current = create(MailAccountStatusV1Schema, {
			connectionId: 'personal-gmail',
			connectorProfile: MailConnectorProfileV1.MAIL_CONNECTOR_PROFILE_GMAIL,
			readiness: MailAccountReadinessV1.MAIL_ACCOUNT_READINESS_CONFIGURATION_ONLY,
		})
		const workflow = {
			catalog: vi.fn().mockResolvedValue({ accounts: [current] }),
			status: vi.fn(),
			retire: vi.fn(),
			delete: vi.fn(),
			retry: vi.fn(),
			refreshLifecycle: vi.fn(),
			rotatePassword: vi.fn(),
		}
		const gmailOAuth = {
			start: vi.fn().mockResolvedValue(create(GmailOAuthStartedV1Schema, {
				setupId: 'setup-1',
				authorizationUrl: 'https://accounts.google.com/authorize',
			})),
			complete: vi.fn().mockResolvedValue({}),
		}
		const browserFlow = vi.fn().mockResolvedValue({
			returnedState: 'provider-state',
			authorizationCode: 'provider-code',
		})
		const controller = useMailAccountManagement(
			() => mailModule([
				'mail.account.catalog.query.v1',
				'mail.account.query.v1',
				'mail.oauth.start.v1',
				'mail.oauth.complete.v1',
			]),
			workflow as never,
			gmailOAuth,
			browserFlow,
			vi.fn(),
			() => ({ clientId: '', origin: 'http://localhost:3000' }),
			'popup',
		)

		await controller.refresh()
		expect(controller.canAuthorizeGmail.value).toBe(true)
		await controller.authorizeGmail()
		expect(gmailOAuth.start).toHaveBeenCalledWith(expect.any(String), 'personal-gmail')
		expect(controller.gmailAuthorizationLabel.value).toBe('Continue with Google')

		await controller.authorizeGmail()
		expect(browserFlow).toHaveBeenCalledWith('https://accounts.google.com/authorize')
		expect(gmailOAuth.complete).toHaveBeenCalledWith(expect.objectContaining({
			connectionId: 'personal-gmail',
			setupId: 'setup-1',
			state: 'provider-state',
			authorizationCode: 'provider-code',
		}))
		expect(controller.gmailAuthorizationLabel.value).toBe('OAuth submitted')
	})

	it('redirects the current tab when the embedded browser blocks OAuth popups', async () => {
		const current = create(MailAccountStatusV1Schema, {
			connectionId: 'personal-gmail',
			connectorProfile: MailConnectorProfileV1.MAIL_CONNECTOR_PROFILE_GMAIL,
			readiness: MailAccountReadinessV1.MAIL_ACCOUNT_READINESS_CONFIGURATION_ONLY,
		})
		const workflow = {
			catalog: vi.fn().mockResolvedValue({ accounts: [current] }),
			status: vi.fn(), retire: vi.fn(), delete: vi.fn(), retry: vi.fn(),
			refreshLifecycle: vi.fn(), rotatePassword: vi.fn(),
		}
		const gmailOAuth = {
			start: vi.fn().mockResolvedValue(create(GmailOAuthStartedV1Schema, {
				setupId: 'setup-1',
				authorizationUrl: 'https://accounts.google.com/authorize',
			})),
			complete: vi.fn(),
		}
		const redirect = vi.fn()
		const controller = useMailAccountManagement(
			() => mailModule([
				'mail.account.catalog.query.v1',
				'mail.account.query.v1',
				'mail.oauth.start.v1',
				'mail.oauth.complete.v1',
			]),
			workflow as never,
			gmailOAuth,
			vi.fn().mockRejectedValue(new Error('gmail_oauth_popup_blocked')),
			redirect,
			() => ({ clientId: '', origin: 'http://localhost:3000' }),
			'popup',
		)

		await controller.refresh()
		await controller.authorizeGmail()
		await controller.authorizeGmail()

		expect(redirect).toHaveBeenCalledWith(
			'https://accounts.google.com/authorize',
			expect.objectContaining({
				connectionId: 'personal-gmail',
				setupId: 'setup-1',
			}),
		)
		expect(gmailOAuth.complete).not.toHaveBeenCalled()
		expect(controller.message.value).toContain('Redirecting to Google')
	})

	it('repairs a legacy Gmail callback through the account Settings target', async () => {
		const current = create(MailAccountStatusV1Schema, {
			connectionId: 'personal-gmail',
			configurationInstanceId: 'gmail-target-1',
			settingsRevision: 6n,
			connectorProfile: MailConnectorProfileV1.MAIL_CONNECTOR_PROFILE_GMAIL,
			readiness: MailAccountReadinessV1.MAIL_ACCOUNT_READINESS_CONFIGURATION_ONLY,
		})
		const workflow = {
			catalog: vi.fn().mockResolvedValue({ accounts: [current] }),
			status: vi.fn(), retire: vi.fn(), delete: vi.fn(), retry: vi.fn(),
			refreshLifecycle: vi.fn(), rotatePassword: vi.fn(),
			normalizeGmailOAuthRedirect: vi.fn().mockResolvedValue(undefined),
		}
		const gmailOAuth = {
			start: vi.fn().mockResolvedValue(create(GmailOAuthStartedV1Schema, {
				setupId: 'setup-1',
				authorizationUrl: 'https://accounts.google.com/authorize',
			})),
			complete: vi.fn(),
		}
		const controller = useMailAccountManagement(
			() => mailModule([
				'mail.account.catalog.query.v1', 'mail.account.query.v1',
				'mail.oauth.start.v1', 'mail.oauth.complete.v1',
			]),
			workflow as never,
			gmailOAuth,
			vi.fn().mockRejectedValue(new Error('gmail_oauth_redirect_uri_mismatch')),
			vi.fn(),
			() => ({
				clientId: 'installed-public-client-id',
				origin: 'http://localhost:3000',
			}),
			'popup',
		)

		await controller.refresh()
		await controller.authorizeGmail()
		await controller.authorizeGmail()

		expect(workflow.normalizeGmailOAuthRedirect).toHaveBeenCalledWith({
			registrationId: 'mail.local',
			configurationInstanceId: 'gmail-target-1',
			expectedDesiredRevision: 6n,
			connectionId: 'personal-gmail',
			clientId: 'installed-public-client-id',
			redirectUri: 'http://localhost:3000/oauth/google/callback',
		})
		expect(controller.gmailAuthorizationLabel.value).toBe('Prepare Google OAuth')
		expect(controller.message.value).toContain('callback configuration was updated')
	})
})

function mailModule(capabilityIds = [
	'mail.account.catalog.query.v1',
	'mail.account.query.v1',
	'mail.account.retire.v1',
]) {
	return create(ClientModuleBootstrapV1Schema, {
		registrationId: 'mail.local',
		moduleId: 'makosh-mail-runtime',
		capabilityIds,
		settings: create(ClientModuleSettingsBootstrapV1Schema, {
			desiredRevision: 3n,
			effectiveRevision: 3n,
			values: [create(ClientSettingValueEntryV1Schema, {
				settingId: 'mail.connection_id',
				value: create(ClientSettingValueV1Schema, {
					value: { case: 'stringValue', value: 'personal-mail' },
				}),
			})],
		}),
	})
}
