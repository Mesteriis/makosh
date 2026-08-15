import { computed, ref, shallowRef } from 'vue'
import type { ClientModuleBootstrapV1 } from '../../../gen/makosh/gateway/v1/client_bootstrap_pb'
import { runGmailOAuthBrowserFlowV1 } from '../oauth/gmailOAuthBrowserFlow'
import {
	MailAccountPortabilityWorkflowV1,
	type MailAccountImportStateV1,
} from './mailAccountPortabilityWorkflow'

export function useMailAccountPortability(
	module: () => ClientModuleBootstrapV1 | null,
	workflow = new MailAccountPortabilityWorkflowV1(),
) {
	const exportJson = ref('')
	const importJson = ref('')
	const imapPassword = ref('')
	const smtpPassword = ref('')
	const importState = shallowRef<MailAccountImportStateV1>()
	const busy = ref(false)
	const localErrorCode = ref('')

	const connectionId = computed(() => {
		const entry = module()?.settings?.values.find(
			(value) => value.settingId === 'mail.connection_id',
		)
		return entry?.value?.value.case === 'stringValue'
			? entry.value.value.value
			: ''
	})
	const canExport = computed(() => {
		const current = module()
		return Boolean(
			current?.sectionsEnabled
			&& current.settings?.effectiveRevision
			&& connectionId.value,
		)
	})
	const steps = computed(() => {
		const current = importState.value
		if (!current) return []
		return [
			{ label: 'Typed export validated', complete: true },
			{ label: 'Desired Settings receipt', complete: Boolean(current.settingsUpdateReceipt) },
			{ label: 'Configuration successor', complete: Boolean(current.configurationApplyReceipt) },
			{
				label: 'IMAP credential binding',
				complete: !current.imap || Boolean(current.imap.bindingReceipt),
			},
			{
				label: 'SMTP credential binding',
				complete: !current.smtp || Boolean(current.smtp.bindingReceipt),
			},
			{
				label: current.exported.configuration?.inbound.case === 'gmail'
					? 'Gmail OAuth completion'
					: 'Credential successor',
				complete: current.exported.configuration?.inbound.case === 'gmail'
					? Boolean(current.gmailOAuthAccepted)
					: Boolean(current.activationApplyReceipt),
			},
			{ label: 'Mail readiness', complete: current.phase === 'ready' },
		]
	})
	const errorCode = computed(
		() => localErrorCode.value || importState.value?.lastErrorCode || '',
	)

	async function prepareExport(): Promise<void> {
		const current = module()
		if (!current?.settings || !canExport.value) return
		await run(async () => {
			const result = await workflow.exportAccount({
				registrationId: current.registrationId,
				expectedEffectiveRevision: current.settings!.effectiveRevision,
				connectionId: connectionId.value,
			})
			exportJson.value = result.json
		}, 'mail_export_failed')
	}

	function downloadExport(): void {
		if (!exportJson.value) return
		const blobUrl = URL.createObjectURL(new Blob(
			[exportJson.value],
			{ type: 'application/json' },
		))
		const anchor = document.createElement('a')
		anchor.href = blobUrl
		anchor.download = `makosh-mail-${connectionId.value || 'account'}.json`
		anchor.click()
		URL.revokeObjectURL(blobUrl)
	}

	async function startImport(): Promise<void> {
		const current = module()
		if (!current?.settings || !importJson.value.trim()) {
			localErrorCode.value = 'mail_import_input_required'
			return
		}
		await run(async () => {
			let state = workflow.initializeImport({
				json: importJson.value,
				targetRegistrationId: current.registrationId,
				expectedDesiredRevision: current.settings!.desiredRevision,
			})
			state = await workflow.updateSettings(state)
			importState.value = state
			if (state.lastErrorCode) return
			state = await workflow.applyConfiguration(state)
			importState.value = state
		}, 'mail_import_prepare_failed')
	}

	async function continueImport(): Promise<void> {
		const current = importState.value
		if (!current) return
		await run(async () => {
			let state = current
			if (state.exported.configuration?.inbound.case === 'gmail') {
				state = await workflow.startGmailOAuth(state)
				importState.value = state
				return
			}
			if (state.imap && !state.imap.bindingReceipt) {
				if (!imapPassword.value) {
					localErrorCode.value = 'mail_import_imap_secret_required'
					return
				}
				state = await workflow.provisionCredential(
					state,
					'imap',
					new TextEncoder().encode(imapPassword.value),
				)
				imapPassword.value = ''
				importState.value = state
				if (state.lastErrorCode) return
			}
			if (state.smtp && !state.smtp.bindingReceipt) {
				if (!smtpPassword.value) {
					localErrorCode.value = 'mail_import_smtp_secret_required'
					return
				}
				state = await workflow.provisionCredential(
					state,
					'smtp',
					new TextEncoder().encode(smtpPassword.value),
				)
				smtpPassword.value = ''
				importState.value = state
				if (state.lastErrorCode) return
			}
			state = await workflow.activateCredentials(state)
			importState.value = state
		}, 'mail_import_continue_failed')
	}

	async function completeGmail(): Promise<void> {
		const current = importState.value
		if (!current?.gmailOAuthStarted) {
			localErrorCode.value = 'mail_import_gmail_completion_required'
			return
		}
		await run(async () => {
			const callback = await runGmailOAuthBrowserFlowV1(
				current.gmailOAuthStarted!.authorizationUrl,
			)
			importState.value = await workflow.completeGmailOAuth(current, {
				state: callback.returnedState,
				authorizationCode: callback.authorizationCode,
			})
		}, 'mail_import_gmail_completion_failed')
	}

	async function reconcile(): Promise<void> {
		if (!importState.value) return
		await run(async () => {
			importState.value = await workflow.reconcile(importState.value!)
		}, 'mail_import_reconciliation_failed')
	}

	async function run(action: () => Promise<void>, fallback: string): Promise<void> {
		busy.value = true
		localErrorCode.value = ''
		try {
			await action()
		} catch {
			localErrorCode.value = fallback
		} finally {
			busy.value = false
		}
	}

	return {
		exportJson,
		importJson,
		imapPassword,
		smtpPassword,
		importState,
		busy,
		canExport,
		steps,
		errorCode,
		prepareExport,
		downloadExport,
		startImport,
		continueImport,
		completeGmail,
		reconcile,
	}
}
