import { computed, ref, shallowRef } from 'vue'
import type { ClientModuleBootstrapV1 } from '../../../gen/makosh/gateway/v1/client_bootstrap_pb'
import { hasOwnerVaultProvisioningHostV1 } from '../../../platform/vault'
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
	const gmailClientId = ref('')
	const gmailRedirectUri = ref('')
	const returnedState = ref('')
	const authorizationCode = ref('')
	const gmailState = shallowRef<MailGmailSetupStateV1>()
	const busy = ref(false)
	const message = ref('')
	const messageTone = ref<'neutral' | 'success' | 'error'>('neutral')
	const secureHostAvailable = hasOwnerVaultProvisioningHostV1()
	const configured = computed(() => (module()?.settings?.effectiveRevision ?? 0n) > 0n)
	const canSubmit = computed(() => {
		if (!module()?.settings || !connectionId.value.trim()) return false
		if (kind.value === 'gmail') {
			return gmailState.value
				? Boolean(returnedState.value.trim() && authorizationCode.value)
				: Boolean(email.value.trim() && gmailClientId.value.trim() && gmailRedirectUri.value.trim())
		}
		return Boolean(
			email.value.trim()
			&& imapHost.value.trim()
			&& imapPassword.value
			&& (!smtpEnabled.value || smtpHost.value.trim()),
		)
	})
	const submitLabel = computed(() => {
		if (kind.value === 'gmail') {
			return gmailState.value ? 'Complete Gmail OAuth' : 'Start Gmail OAuth'
		}
		return 'Connect IMAP account'
	})

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
		if (!gmailState.value) {
			gmailState.value = await workflow.startGmail({
				registrationId: current.registrationId,
				expectedDesiredRevision: current.settings!.desiredRevision,
				connectionId: connectionId.value,
				email: email.value,
				clientId: gmailClientId.value,
				redirectUri: gmailRedirectUri.value,
			})
			message.value = 'Gmail configuration is active. Open Google authorization and return the state and code.'
			messageTone.value = 'neutral'
			return true
		}
		await workflow.completeGmail(gmailState.value, {
			returnedState: returnedState.value,
			authorizationCode: authorizationCode.value,
		})
		authorizationCode.value = ''
		message.value = 'Gmail OAuth completion accepted. Readiness will update after reconciliation.'
		messageTone.value = 'success'
		return true
	}

	function clearSecrets(): void {
		imapPassword.value = ''
		smtpPassword.value = ''
		authorizationCode.value = ''
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
		gmailRedirectUri,
		returnedState,
		authorizationCode,
		gmailState,
		busy,
		message,
		messageTone,
		secureHostAvailable,
		configured,
		canSubmit,
		submitLabel,
		submit,
	}
}
