import { computed, onBeforeUnmount, ref, watch } from 'vue'

import {
	ClientSettingsApplyStateV1,
	type ClientModuleBootstrapV1,
	type ClientModuleSettingsTargetBootstrapV1,
} from '../../../gen/makosh/gateway/v1/client_bootstrap_pb'
import {
	MailContactsSyncDirectionV1,
	MailContactsSyncStateV1,
	type GetMailContactsSyncResponseV1,
	type MailContactsSyncStatusChangedV1,
} from '../../../gen/makosh/mail_contacts_sync/v1/sync_pb'
import { resolveOwnerOperationIdV1 } from '../../../platform/gateway/ownerOperationId'
import { ManagedWorkflowSetupV1 } from '../../../platform/settings/managedWorkflowSetup'
import type { OwnerSettingInputV1 } from '../../../platform/settings/ownerModuleSettingsClient'
import {
	getMailContactsSync,
	openMailContactsSyncRealtime,
	startMailContactsSync,
	type MailContactsSyncRealtimeBindingV1,
} from '../api/mailContactsSync'

const WORKFLOW_MODULE_ID = 'makosh-mail-contacts-sync-runtime'
const STORAGE_CAPABILITY_ID = 'mail_contacts_sync.storage.v1'
const ID_BYTES = 16
const MIN_INTERVAL_SECONDS = 300
const MAX_INTERVAL_SECONDS = 604_800

export type MailContactsSyncAccountChoiceV1 = {
	accountId: string
	syncReady: boolean
}

export function useMailContactsSyncSettings(input: {
	module: () => ClientModuleBootstrapV1 | null
	accounts: () => readonly MailContactsSyncAccountChoiceV1[]
}, setup: ManagedWorkflowSetupV1 = new ManagedWorkflowSetupV1()) {
	const accountId = ref('')
	const direction = ref(MailContactsSyncDirectionV1.MAIL_CONTACTS_SYNC_DIRECTION_PROVIDER_TO_CONTACTS)
	const intervalSeconds = ref(900)
	const busy = ref(false)
	const message = ref('')
	const status = ref<GetMailContactsSyncResponseV1 | null>(null)
	const activeTargetId = ref('')
	const locallyAppliedAccountId = ref('')
	let realtime: MailContactsSyncRealtimeBindingV1 | undefined

	const ownedModule = computed(() => input.module()?.moduleId === WORKFLOW_MODULE_ID
		? input.module()
		: null)
	const admitted = computed(() => Boolean(
		ownedModule.value?.capabilityIds.includes(STORAGE_CAPABILITY_ID),
	))
	const selectedAccount = computed(() => input.accounts().find(
		(account) => account.accountId === accountId.value,
	))
	const selectedTarget = computed(() => ownedModule.value?.settingsTargets.find(
		(target) => targetAccountId(target) === accountId.value,
	) ?? null)
	const configured = computed(() => locallyAppliedAccountId.value === accountId.value
		|| selectedTarget.value?.applyState === ClientSettingsApplyStateV1.CURRENT)
	const canStart = computed(() => admitted.value
		&& selectedAccount.value?.syncReady
		&& configured.value
		&& !busy.value)

	watch(
		input.accounts,
		accounts => {
			if (!accounts.some((account) => account.accountId === accountId.value)) {
				accountId.value = accounts[0]?.accountId ?? ''
			}
		},
		{ immediate: true },
	)
	watch(selectedTarget, target => {
		activeTargetId.value = target?.configurationInstanceId ?? ''
	}, { immediate: true })

	async function configure(): Promise<void> {
		const module = ownedModule.value
		if (!module || !accountId.value || busy.value) return
		if (!validAccountId(accountId.value)
			|| !Number.isSafeInteger(intervalSeconds.value)
			|| intervalSeconds.value < MIN_INTERVAL_SECONDS
			|| intervalSeconds.value > MAX_INTERVAL_SECONDS) {
			message.value = 'Choose a valid account and an interval from 300 to 604800 seconds.'
			return
		}
		busy.value = true
		message.value = ''
		try {
			let configurationInstanceId = selectedTarget.value?.configurationInstanceId ?? ''
			let expectedDesiredRevision = selectedTarget.value?.desiredRevision ?? 0n
			if (!configurationInstanceId) {
				const created = await setup.createTarget(module.registrationId, resolveOwnerOperationIdV1())
				configurationInstanceId = created.configurationInstanceId
				expectedDesiredRevision = created.desiredRevision
			}
			const bidirectional = direction.value
				=== MailContactsSyncDirectionV1.MAIL_CONTACTS_SYNC_DIRECTION_BIDIRECTIONAL
			const receipt = await setup.apply({
				registrationId: module.registrationId,
				configurationInstanceId,
				expectedDesiredRevision,
				storageCapabilityId: STORAGE_CAPABILITY_ID,
				updateOperationId: resolveOwnerOperationIdV1(),
				applyOperationId: resolveOwnerOperationIdV1(),
				values: settingsValues(accountId.value, bidirectional, intervalSeconds.value),
			})
			activeTargetId.value = receipt.application.configurationInstanceId
			locallyAppliedAccountId.value = accountId.value
			message.value = `Sync configuration applied at runtime generation ${receipt.application.runtimeGeneration}.`
		} catch {
			message.value = 'Mail Contacts Sync configuration was not applied.'
		} finally {
			busy.value = false
		}
	}

	async function start(): Promise<void> {
		if (!canStart.value) return
		busy.value = true
		message.value = ''
		realtime?.close()
		try {
			realtime = openMailContactsSyncRealtime({
				onStatus: value => void receiveStatus(value),
				onUnavailable: () => { message.value = 'Mail Contacts Sync realtime is unavailable.' },
			})
			await realtime.ready
			const runId = await startMailContactsSync(
				accountId.value,
				direction.value,
				crypto.getRandomValues(new Uint8Array(ID_BYTES)),
			)
			realtime.attachRun(runId)
			status.value = await getMailContactsSync(runId)
			message.value = 'Mail Contacts Sync accepted; progress is delivered through shared realtime.'
		} catch {
			realtime?.close()
			realtime = undefined
			message.value = 'Mail Contacts Sync could not be started.'
		} finally {
			busy.value = false
		}
	}

	async function receiveStatus(event: MailContactsSyncStatusChangedV1): Promise<void> {
		try {
			status.value = await getMailContactsSync(event.runId)
			if (event.state === MailContactsSyncStateV1.MAIL_CONTACTS_SYNC_STATE_COMPLETED
				|| event.state === MailContactsSyncStateV1.MAIL_CONTACTS_SYNC_STATE_REJECTED) {
				realtime?.close()
				realtime = undefined
			}
		} catch {
			message.value = 'Mail Contacts Sync status could not be read.'
		}
	}

	onBeforeUnmount(() => realtime?.close())
	return {
		accountId,
		activeTargetId,
		admitted,
		busy,
		canStart,
		configure,
		direction,
		intervalSeconds,
		message,
		start,
		status,
	}
}

function validAccountId(value: string): boolean {
	return value.length > 0
		&& value.length <= 256
		&& value.trim() === value
		&& /^[\x20-\x7e]+$/.test(value)
}

export function mailContactsSyncStateLabel(value: MailContactsSyncStateV1 | undefined): string {
	if (value === MailContactsSyncStateV1.MAIL_CONTACTS_SYNC_STATE_ACCEPTED) return 'Accepted'
	if (value === MailContactsSyncStateV1.MAIL_CONTACTS_SYNC_STATE_FETCHING_PROVIDER_PAGE) return 'Reading provider'
	if (value === MailContactsSyncStateV1.MAIL_CONTACTS_SYNC_STATE_APPLYING_CONTACTS) return 'Applying contacts'
	if (value === MailContactsSyncStateV1.MAIL_CONTACTS_SYNC_STATE_WRITING_PROVIDER) return 'Writing provider'
	if (value === MailContactsSyncStateV1.MAIL_CONTACTS_SYNC_STATE_RECONCILING_OUTCOME) return 'Reconciling'
	if (value === MailContactsSyncStateV1.MAIL_CONTACTS_SYNC_STATE_COMPLETED) return 'Completed'
	if (value === MailContactsSyncStateV1.MAIL_CONTACTS_SYNC_STATE_REJECTED) return 'Rejected'
	return 'Not started'
}

function settingsValues(
	accountId: string,
	bidirectional: boolean,
	intervalSeconds: number,
): OwnerSettingInputV1[] {
	return [
		{ settingId: 'mail_contacts_sync.account_id', value: { case: 'stringValue', value: accountId } },
		{ settingId: 'mail_contacts_sync.direction', value: { case: 'enumValue', value: bidirectional ? 'bidirectional' : 'provider_to_contacts' } },
		{ settingId: 'mail_contacts_sync.enabled', value: { case: 'booleanValue', value: true } },
		{ settingId: 'mail_contacts_sync.interval_seconds', value: { case: 'unsignedIntegerValue', value: BigInt(intervalSeconds) } },
		{ settingId: 'mail_contacts_sync.remote_write_enabled', value: { case: 'booleanValue', value: bidirectional } },
	]
}

function targetAccountId(target: ClientModuleSettingsTargetBootstrapV1): string {
	const entry = target.values.find((value) => value.settingId === 'mail_contacts_sync.account_id')
	return entry?.value?.value.case === 'stringValue' ? entry.value.value.value : ''
}
