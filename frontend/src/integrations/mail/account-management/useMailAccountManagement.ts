import { computed, ref, shallowRef } from 'vue'
import {
	MailAccountReadinessV1,
	MailConnectorProfileV1,
	MailCredentialBindingStateV1,
	MailCredentialPurposeV1,
	type MailAccountStatusV1,
} from '../../../gen/makosh/mail/account/v1/client_pb'
import type { GmailOAuthStartedV1 } from '../../../gen/makosh/mail/v1/client_pb'
import type { ClientModuleBootstrapV1 } from '../../../gen/makosh/gateway/v1/client_bootstrap_pb'
import {
	hasOwnerVaultProvisioningHostV1,
} from '../../../platform/vault'
import { MailGmailOAuthClientV1 } from '../api/mailGmailOAuthClient'
import {
	gmailOAuthLoopbackRedirectUriV1,
	runGmailOAuthBrowserFlowV1,
	type GmailOAuthBrowserResultV1,
} from '../oauth/gmailOAuthBrowserFlow'
import {
	redirectGmailOAuthInSameTabV1,
	type GmailOAuthSameTabContinuationV1,
} from '../oauth/gmailOAuthRedirectFlow'
import {
	MailAccountManagementWorkflowV1,
	type MailPasswordPurposeV1,
} from './mailAccountManagementWorkflow'

const MAIL_MODULE_ID = 'makosh-mail-runtime'
const MAIL_STORAGE_CAPABILITY_ID = 'mail.storage.v1'

type MailAccountManagementWorkflow = Pick<
	MailAccountManagementWorkflowV1,
	'catalog' | 'status' | 'retire' | 'delete' | 'retry' | 'refreshLifecycle' | 'rotatePassword'
	| 'normalizeGmailOAuthRedirect'
>

type GmailOAuthPortV1 = Pick<MailGmailOAuthClientV1, 'start' | 'complete'>
type GmailOAuthBrowserFlowV1 = (authorizationUrl: string) => Promise<GmailOAuthBrowserResultV1>
type GmailOAuthSameTabRedirectV1 = (
	authorizationUrl: string,
	continuation: GmailOAuthSameTabContinuationV1,
) => void
type GmailOAuthInstalledConfigurationV1 = () => { clientId: string; origin: string }
type GmailOAuthPresentationV1 = 'same-tab' | 'popup'

export function useMailAccountManagement(
	module: () => ClientModuleBootstrapV1 | null,
	workflow: MailAccountManagementWorkflow = new MailAccountManagementWorkflowV1(),
	gmailOAuth?: GmailOAuthPortV1,
	gmailOAuthBrowserFlow: GmailOAuthBrowserFlowV1 = runGmailOAuthBrowserFlowV1,
	gmailOAuthSameTabRedirect: GmailOAuthSameTabRedirectV1 = redirectGmailOAuthInSameTabV1,
	gmailOAuthInstalledConfiguration: GmailOAuthInstalledConfigurationV1 = () => ({
		clientId: import.meta.env.VITE_MAKOSH_GMAIL_OAUTH_CLIENT_ID?.trim() ?? '',
		origin: window.location.origin,
	}),
	gmailOAuthPresentation: GmailOAuthPresentationV1 = 'same-tab',
) {
	const status = shallowRef<MailAccountStatusV1 | null>(null)
	const accounts = shallowRef<MailAccountStatusV1[]>([])
	const connectionId = ref('')
	const imapPassword = ref('')
	const smtpPassword = ref('')
	const gmailOAuthOperationId = ref('')
	const gmailOAuthStarted = shallowRef<GmailOAuthStartedV1>()
	const gmailOAuthCompletionSubmitted = ref(false)
	const busy = ref(false)
	const message = ref('')
	const messageTone = ref<'neutral' | 'success' | 'error'>('neutral')
	const secureHostAvailable = hasOwnerVaultProvisioningHostV1()
	const ownedModule = computed(() => module()?.moduleId === MAIL_MODULE_ID ? module() : null)
	const canCatalog = computed(() => hasCapability('mail.account.catalog.query.v1'))
	const canQuery = computed(() => hasCapability('mail.account.query.v1') && Boolean(connectionId.value))
	const stateLabel = computed(() => mailReadinessLabel(status.value?.readiness))
	const canRetire = computed(() => hasCapability('mail.account.retire.v1') && Boolean(status.value))
	const canDelete = computed(() => hasCapability('mail.account.delete.v1') && Boolean(status.value))
	const canRetry = computed(() => hasCapability('mail.account.lifecycle.retry.v1')
		&& Boolean(status.value?.lifecycleOperationId))
	const canRefreshLifecycle = computed(() => hasCapability('mail.account.lifecycle.query.v1')
		&& Boolean(status.value?.lifecycleOperationId))
	const canAuthorizeGmail = computed(() =>
		hasCapability('mail.oauth.start.v1')
		&& hasCapability('mail.oauth.complete.v1')
		&& status.value?.connectorProfile === MailConnectorProfileV1.MAIL_CONNECTOR_PROFILE_GMAIL
		&& status.value.readiness === MailAccountReadinessV1.MAIL_ACCOUNT_READINESS_CONFIGURATION_ONLY)
	const gmailAuthorizationLabel = computed(() => {
		if (gmailOAuthCompletionSubmitted.value) return 'OAuth submitted'
		return gmailOAuthStarted.value ? 'Continue with Google' : 'Prepare Google OAuth'
	})

	async function refresh(): Promise<void> {
		if (!canCatalog.value) {
			status.value = null
			accounts.value = []
			message.value = 'Mail account catalog capability is not admitted.'
			messageTone.value = 'neutral'
			return
		}
		await run(async () => {
			const catalog = await workflow.catalog()
			accounts.value = [...catalog.accounts]
			if (!accounts.value.some((account) => account.connectionId === connectionId.value)) {
				connectionId.value = accounts.value[0]?.connectionId ?? ''
			}
			status.value = accounts.value.find(
				(account) => account.connectionId === connectionId.value,
			) ?? null
			message.value = status.value
				? `Mail account ${status.value.connectionId} status refreshed.`
				: 'No Mail accounts are configured yet.'
		}, 'Mail account status is unavailable.')
	}

	async function selectAccount(nextConnectionId: string): Promise<void> {
		resetGmailAuthorization()
		connectionId.value = nextConnectionId
		status.value = accounts.value.find(
			(account) => account.connectionId === nextConnectionId,
		) ?? null
		if (canQuery.value) {
			await run(async () => {
				status.value = await workflow.status(nextConnectionId)
			}, 'Mail account status is unavailable.')
		}
	}

	async function authorizeGmail(): Promise<void> {
		const current = status.value
		if (!current || !canAuthorizeGmail.value || gmailOAuthCompletionSubmitted.value) return
		const oauth = gmailOAuth ?? new MailGmailOAuthClientV1()
		await run(async () => {
			if (!gmailOAuthStarted.value) {
				// OAuth reauthorization rotates provider tokens, not the installed client
				// credential. The client secret is provisioned once by account setup and
				// must be reused here; attempting CREATE again conflicts with Vault's
				// write-once revision 1 record after a browser or frontend restart.
				gmailOAuthOperationId.value = `mail-gmail-operational-auth-${crypto.randomUUID()}`
				gmailOAuthStarted.value = await oauth.start(
					gmailOAuthOperationId.value,
					current.connectionId,
				)
				message.value = 'Google OAuth request is ready. Continue with Google to authorize Mail.'
				return
			}
			const continuation = {
				operationId: gmailOAuthOperationId.value,
				connectionId: current.connectionId,
				setupId: gmailOAuthStarted.value.setupId,
			}
			if (gmailOAuthPresentation === 'same-tab') {
				gmailOAuthSameTabRedirect(
					gmailOAuthStarted.value.authorizationUrl,
					continuation,
				)
				message.value = 'Redirecting to Google. Макошь will resume authorization after the callback.'
				return
			}
			let callback: GmailOAuthBrowserResultV1
			try {
				callback = await gmailOAuthBrowserFlow(gmailOAuthStarted.value.authorizationUrl)
			} catch (error) {
				if (error instanceof Error && error.message === 'gmail_oauth_redirect_uri_mismatch') {
					const currentModule = ownedModule.value
					const installed = gmailOAuthInstalledConfiguration()
					const clientId = installed.clientId.trim()
					if (!currentModule || !clientId) throw error
					await workflow.normalizeGmailOAuthRedirect({
						registrationId: currentModule.registrationId,
						configurationInstanceId: current.configurationInstanceId,
						expectedDesiredRevision: current.settingsRevision,
						connectionId: current.connectionId,
						clientId,
						redirectUri: gmailOAuthLoopbackRedirectUriV1(installed.origin),
					})
					resetGmailAuthorization()
					message.value = 'Gmail OAuth callback configuration was updated. Prepare Google OAuth again.'
					return
				}
				if (!(error instanceof Error) || error.message !== 'gmail_oauth_popup_blocked') throw error
				gmailOAuthSameTabRedirect(gmailOAuthStarted.value.authorizationUrl, continuation)
				message.value = 'Redirecting to Google. Макошь will resume authorization after the callback.'
				return
			}
			gmailOAuthCompletionSubmitted.value = true
			await oauth.complete({
				operationId: gmailOAuthOperationId.value,
				connectionId: current.connectionId,
				setupId: gmailOAuthStarted.value.setupId,
				state: callback.returnedState,
				authorizationCode: callback.authorizationCode,
			})
			message.value = 'Gmail OAuth completion accepted. Refresh after provider exchange.'
		}, gmailOAuthCompletionSubmitted.value
			? 'Gmail OAuth outcome is unavailable. Refresh before starting another attempt.'
			: 'Gmail OAuth authorization was not completed.')
	}

	async function retire(): Promise<void> {
		await runWithStatus(async (current) => {
			const receipt = await workflow.retire(current)
			message.value = `Mail retirement ${receipt.operationId} accepted. Refresh status to observe completion.`
		}, 'Mail account retirement failed.')
	}

	async function deleteAccount(): Promise<void> {
		await runWithStatus(async (current) => {
			const receipt = await workflow.delete(current)
			message.value = `Mail deletion ${receipt.operationId} accepted. Refresh status to observe completion.`
		}, 'Mail account deletion failed.')
	}

	async function retry(): Promise<void> {
		await runWithStatus(async (current) => {
			const receipt = await workflow.retry(current)
			message.value = `Mail lifecycle retry ${receipt.operationId} accepted. Refresh status to observe completion.`
		}, 'Mail lifecycle retry failed.')
	}

	async function refreshLifecycle(): Promise<void> {
		await runWithStatus(async (current) => {
			const receipt = await workflow.refreshLifecycle(current)
			message.value = `Mail lifecycle operation ${receipt.operationId} status received.`
		}, 'Mail lifecycle operation status is unavailable.')
	}

	async function rotatePassword(purpose: MailPasswordPurposeV1): Promise<void> {
		const secret = purpose === 'imap' ? imapPassword.value : smtpPassword.value
		if (!canRotate(purpose) || !secret) return
		await runWithStatus(async (current) => {
			const currentModule = ownedModule.value
			if (!currentModule?.settings) throw new Error('mail_settings_unavailable')
			const receipt = await workflow.rotatePassword({
				registrationId: currentModule.registrationId,
				storageCapabilityId: MAIL_STORAGE_CAPABILITY_ID,
				configurationInstanceId: current.configurationInstanceId,
				expectedDesiredRevision: current.settingsRevision,
				status: current,
				purpose,
				secretPayload: new TextEncoder().encode(secret),
			})
			status.value = receipt.status
			message.value = `${purpose.toUpperCase()} password rotated and rebound.`
		}, `Mail ${purpose.toUpperCase()} password rotation did not reach confirmed readiness.`)
		clearPassword(purpose)
	}

	function canRotate(purpose: MailPasswordPurposeV1): boolean {
		const current = status.value
		const credentialPurpose = purpose === 'imap'
			? MailCredentialPurposeV1.MAIL_CREDENTIAL_PURPOSE_IMAP_PASSWORD
			: MailCredentialPurposeV1.MAIL_CREDENTIAL_PURPOSE_SMTP_PASSWORD
		return secureHostAvailable
			&& hasCapability('mail.account.credential.bind.v1')
			&& hasCapability(`mail.${purpose}.credential-provisioning.v1`)
			&& hasCapability(MAIL_STORAGE_CAPABILITY_ID)
			&& Boolean(current?.binding.some((entry) =>
				entry.purpose === credentialPurpose
				&& entry.state === MailCredentialBindingStateV1.MAIL_CREDENTIAL_BINDING_STATE_ACTIVE
				&& entry.bindingRevision
				&& entry.credentialRevision))
	}

	async function runWithStatus(
		action: (current: MailAccountStatusV1) => Promise<void>,
		failure: string,
	): Promise<void> {
		const current = status.value
		if (!current) return
		await run(() => action(current), failure)
	}

	async function run(action: () => Promise<void>, failure: string): Promise<void> {
		busy.value = true
		message.value = ''
		try {
			await action()
			messageTone.value = 'success'
		} catch (error) {
			const failureCode = safeDeveloperFailureCode(error)
			message.value = import.meta.env.DEV && failureCode
				? `${failure} (${failureCode})`
				: failure
			messageTone.value = 'error'
		} finally {
			busy.value = false
		}
	}

	function hasCapability(capabilityId: string): boolean {
		return ownedModule.value?.capabilityIds.includes(capabilityId) ?? false
	}

	function clearPassword(purpose: MailPasswordPurposeV1): void {
		if (purpose === 'imap') imapPassword.value = ''
		else smtpPassword.value = ''
	}

	function resetGmailAuthorization(): void {
		gmailOAuthOperationId.value = ''
		gmailOAuthStarted.value = undefined
		gmailOAuthCompletionSubmitted.value = false
	}

	return {
		status,
		accounts,
		connectionId,
		imapPassword,
		smtpPassword,
		busy,
		message,
		messageTone,
		secureHostAvailable,
		stateLabel,
		canQuery,
		canRetire,
		canDelete,
		canRetry,
		canRefreshLifecycle,
		canAuthorizeGmail,
		gmailAuthorizationLabel,
		gmailOAuthCompletionSubmitted,
		canRotateImap: computed(() => canRotate('imap')),
		canRotateSmtp: computed(() => canRotate('smtp')),
		refresh,
		selectAccount,
		retire,
		deleteAccount,
		retry,
		refreshLifecycle,
		authorizeGmail,
		rotatePassword,
	}
}

function safeDeveloperFailureCode(error: unknown): string | undefined {
	if (!(error instanceof Error)) return undefined
	return /^[a-z0-9_]{1,128}$/.test(error.message) ? error.message : undefined
}

function mailReadinessLabel(readiness: MailAccountReadinessV1 | undefined): string {
	if (readiness === MailAccountReadinessV1.MAIL_ACCOUNT_READINESS_CONFIGURATION_ONLY) return 'Configuration only'
	if (readiness === MailAccountReadinessV1.MAIL_ACCOUNT_READINESS_PENDING_RESTART) return 'Pending restart'
	if (readiness === MailAccountReadinessV1.MAIL_ACCOUNT_READINESS_READY) return 'Ready'
	if (readiness === MailAccountReadinessV1.MAIL_ACCOUNT_READINESS_RETIRED) return 'Retired'
	if (readiness === MailAccountReadinessV1.MAIL_ACCOUNT_READINESS_DELETED) return 'Deleted'
	if (readiness === MailAccountReadinessV1.MAIL_ACCOUNT_READINESS_DEGRADED) return 'Degraded'
	return 'No account'
}
