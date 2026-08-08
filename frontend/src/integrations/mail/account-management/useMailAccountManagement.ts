import { computed, ref, shallowRef } from 'vue'
import {
	MailAccountReadinessV1,
	MailCredentialBindingStateV1,
	MailCredentialPurposeV1,
	type MailAccountStatusV1,
} from '../../../gen/makosh/mail/account/v1/client_pb'
import type { ClientModuleBootstrapV1 } from '../../../gen/makosh/gateway/v1/client_bootstrap_pb'
import { hasOwnerVaultProvisioningHostV1 } from '../../../platform/vault'
import {
	MailAccountManagementWorkflowV1,
	type MailPasswordPurposeV1,
} from './mailAccountManagementWorkflow'

const MAIL_MODULE_ID = 'makosh-mail-runtime'
const MAIL_STORAGE_CAPABILITY_ID = 'mail.storage.v1'

type MailAccountManagementWorkflow = Pick<
	MailAccountManagementWorkflowV1,
	'catalog' | 'status' | 'retire' | 'delete' | 'retry' | 'refreshLifecycle' | 'rotatePassword'
>

export function useMailAccountManagement(
	module: () => ClientModuleBootstrapV1 | null,
	workflow: MailAccountManagementWorkflow = new MailAccountManagementWorkflowV1(),
) {
	const status = shallowRef<MailAccountStatusV1 | null>(null)
	const accounts = shallowRef<MailAccountStatusV1[]>([])
	const connectionId = ref('')
	const imapPassword = ref('')
	const smtpPassword = ref('')
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
		} catch {
			message.value = failure
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
		canRotateImap: computed(() => canRotate('imap')),
		canRotateSmtp: computed(() => canRotate('smtp')),
		refresh,
		selectAccount,
		retire,
		deleteAccount,
		retry,
		refreshLifecycle,
		rotatePassword,
	}
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
