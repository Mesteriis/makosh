import { computed, ref, shallowRef } from 'vue'
import type { ClientModuleBootstrapV1 } from '../../../gen/makosh/gateway/v1/client_bootstrap_pb'
import { hasOwnerVaultProvisioningHostV1 } from '../../../platform/vault'
import {
	gmailOAuthLoopbackRedirectUriV1,
	runGmailOAuthBrowserFlowV1,
} from '../oauth/gmailOAuthBrowserFlow'
import {
	MailAccountSetupWorkflowV1,
	type MailGmailSetupStateV1,
} from './mailAccountSetupWorkflow'

export function useMailAccountSetup(
	module: () => ClientModuleBootstrapV1 | null,
	workflow = new MailAccountSetupWorkflowV1(),
) {
	const kind = ref<'imap' | 'gmail'>('imap')
	const connectionId = ref('')
	const email = ref('')
	const imapHost = ref('')
	const imapPort = ref('993')
	const imapPassword = ref('')
	const smtpEnabled = ref(false)
	const smtpHost = ref('')
	const smtpPort = ref('465')
	const smtpPassword = ref('')
	const installedGmailClientId = import.meta.env.VITE_MAKOSH_GMAIL_OAUTH_CLIENT_ID?.trim() ?? ''
	const gmailClientId = ref(installedGmailClientId)
	const gmailClientConfigured = computed(() => Boolean(installedGmailClientId))
	const gmailRedirectUri = computed(() => {
		try {
			return gmailOAuthLoopbackRedirectUriV1(window.location.origin)
		} catch {
			return ''
		}
	})
	const gmailState = shallowRef<MailGmailSetupStateV1>()
	const gmailCompletionSubmitted = ref(false)
	const busy = ref(false)
	const message = ref('')
	const messageTone = ref<'neutral' | 'success' | 'error'>('neutral')
	const secureHostAvailable = hasOwnerVaultProvisioningHostV1()
	const configured = computed(() => (module()?.settings?.effectiveRevision ?? 0n) > 0n)
	const canSubmit = computed(() => {
		if (!module()?.settings || !connectionId.value.trim()) return false
		if (kind.value === 'gmail') {
			return !gmailState.value
				&& Boolean(email.value.trim() && gmailClientId.value.trim() && gmailRedirectUri.value)
		}
		return Boolean(
			email.value.trim()
			&& imapHost.value.trim()
			&& imapPassword.value
			&& (!smtpEnabled.value || smtpHost.value.trim()),
		)
	})
	const canAuthorize = computed(() => Boolean(
		gmailState.value
		&& !gmailCompletionSubmitted.value
		&& gmailRedirectUri.value,
	))

	async function submit(): Promise<boolean> {
		const current = module()
		if (!current?.settings || !canSubmit.value) return false
		if (kind.value === 'imap' && !secureHostAvailable) {
			message.value = 'Use the desktop shell or root make dev to seal mail credentials.'
			messageTone.value = 'neutral'
			return false
		}
		busy.value = true
		message.value = ''
		try {
			if (kind.value === 'gmail') {
				return await submitGmail(current)
			} else {
				await workflow.setupImap({
					registrationId: current.registrationId,
					expectedDesiredRevision: current.settings.desiredRevision,
					connectionId: connectionId.value,
					imapHost: imapHost.value,
					imapPort: BigInt(imapPort.value),
					username: email.value,
					imapPassword: new TextEncoder().encode(imapPassword.value),
					smtp: smtpEnabled.value
						? {
							host: smtpHost.value,
							port: BigInt(smtpPort.value),
							username: email.value,
							fromAddress: email.value,
							password: new TextEncoder().encode(smtpPassword.value || imapPassword.value),
						}
						: undefined,
				})
				clearSecrets()
				message.value = 'Mail account configured and credential bindings activated.'
				messageTone.value = 'success'
				return true
			}
		} catch {
			clearSecrets()
			message.value = 'Mail account setup failed before readiness. Secrets were not stored in Settings.'
			messageTone.value = 'error'
			return false
		} finally {
			busy.value = false
		}
	}

	async function submitGmail(current: ClientModuleBootstrapV1): Promise<boolean> {
		if (gmailState.value) return false
		gmailState.value = await workflow.startGmail({
			registrationId: current.registrationId,
			expectedDesiredRevision: current.settings!.desiredRevision,
			connectionId: connectionId.value,
			email: email.value,
			clientId: gmailClientId.value,
			redirectUri: gmailRedirectUri.value,
		})
		gmailCompletionSubmitted.value = false
		message.value = 'Gmail configuration is active. Continue with Google to grant OAuth access.'
		messageTone.value = 'neutral'
		return true
	}

	async function authorizeGmail(): Promise<boolean> {
		const current = gmailState.value
		if (!current || !canAuthorize.value || busy.value) return false
		busy.value = true
		message.value = ''
		try {
			const callback = await runGmailOAuthBrowserFlowV1(
				current.started.authorizationUrl,
			)
			gmailCompletionSubmitted.value = true
			await workflow.completeGmail(current, callback)
			message.value = 'Gmail OAuth completion accepted. Readiness will update after reconciliation.'
			messageTone.value = 'success'
			return true
		} catch {
			message.value = gmailCompletionSubmitted.value
				? 'Gmail OAuth completion outcome is unavailable. Check account status before starting a new attempt.'
				: 'Gmail OAuth was not completed. Start a new provider authorization attempt if this one expired.'
			messageTone.value = 'error'
			return false
		} finally {
			busy.value = false
		}
	}

	function resetGmailAuthorization(): void {
		gmailState.value = undefined
		gmailCompletionSubmitted.value = false
	}

	function clearSecrets(): void {
		imapPassword.value = ''
		smtpPassword.value = ''
	}

	return {
		kind,
		connectionId,
		email,
		imapHost,
		imapPort,
		imapPassword,
		smtpEnabled,
		smtpHost,
		smtpPort,
		smtpPassword,
		gmailClientId,
		gmailClientConfigured,
		gmailRedirectUri,
		gmailState,
		busy,
		message,
		messageTone,
		secureHostAvailable,
		configured,
		canSubmit,
		canAuthorize,
		submit,
		authorizeGmail,
		resetGmailAuthorization,
	}
}
