import { computed, ref, shallowRef } from 'vue'
import { ZulipCredentialBindingStateV1 } from '../../../gen/makosh/zulip/account/v1/client_pb'
import type { ZulipAccountStatusV1 } from '../../../gen/makosh/zulip/operational/v1/client_pb'
import type { ClientModuleBootstrapV1 } from '../../../gen/makosh/gateway/v1/client_bootstrap_pb'
import { publicModuleStringSetting } from '../../../platform/gateway/publicModuleSettings'
import { hasOwnerVaultProvisioningHostV1 } from '../../../platform/vault'
import { ZulipAccountManagementWorkflowV1 } from './zulipAccountManagementWorkflow'

const ZULIP_MODULE_ID = 'makosh-zulip-runtime'

type ZulipAccountManagementWorkflow = Pick<
	ZulipAccountManagementWorkflowV1,
	'status' | 'retire' | 'rotateApiKey'
>

export function useZulipAccountManagement(
	module: () => ClientModuleBootstrapV1 | null,
	workflow: ZulipAccountManagementWorkflow = new ZulipAccountManagementWorkflowV1(),
) {
	const status = shallowRef<ZulipAccountStatusV1 | null>(null)
	const apiKey = ref('')
	const busy = ref(false)
	const message = ref('')
	const messageTone = ref<'neutral' | 'success' | 'error'>('neutral')
	const secureHostAvailable = hasOwnerVaultProvisioningHostV1()
	const ownedModule = computed(() => module()?.moduleId === ZULIP_MODULE_ID ? module() : null)
	const accountId = computed(() => publicModuleStringSetting(ownedModule.value, 'zulip.account_id') ?? '')
	const canQuery = computed(() => hasCapability('zulip.operational.query.v1') && Boolean(accountId.value))
	const canRetire = computed(() => hasCapability('zulip.account.lifecycle.v1') && Boolean(status.value))
	const canRotate = computed(() => secureHostAvailable
		&& hasCapability('zulip.account.lifecycle.v1')
		&& hasCapability('zulip.api-key.credential-provisioning.v1')
		&& hasCapability('zulip.storage.v1')
		&& status.value?.credentialState === ZulipCredentialBindingStateV1.ZULIP_CREDENTIAL_BINDING_STATE_ACTIVE
		&& Boolean(status.value.credentialRevision)
		&& status.value.bindingRevision > 0n)
	const stateLabel = computed(() => zulipCredentialStateLabel(status.value?.credentialState))

	async function refresh(): Promise<void> {
		if (!canQuery.value) {
			status.value = null
			message.value = accountId.value
				? 'Zulip account status capability is not admitted.'
				: 'Configure a Zulip account to expose its lifecycle status.'
			messageTone.value = 'neutral'
			return
		}
		await run(async () => {
			status.value = await workflow.status(accountId.value)
			message.value = `Zulip account ${status.value.accountId} status refreshed.`
		}, 'Zulip account status is unavailable.')
	}

	async function retire(): Promise<void> {
		const current = status.value
		if (!current || !canRetire.value) return
		await run(async () => {
			const receipt = await workflow.retire(current)
			status.value = {
				...current,
				credentialState: receipt.state,
				bindingRevision: receipt.bindingRevision,
			}
			message.value = `Zulip retirement receipt received for ${receipt.accountId}.`
		}, 'Zulip account retirement failed.')
	}

	async function rotateApiKey(): Promise<void> {
		const current = status.value
		const currentModule = ownedModule.value
		if (!current || !currentModule?.settings || !canRotate.value || !apiKey.value) return
		const secret = apiKey.value
		await run(async () => {
			const receipt = await workflow.rotateApiKey({
				registrationId: currentModule.registrationId,
				accountId: current.accountId,
				expectedDesiredRevision: currentModule.settings!.desiredRevision,
				status: current,
				secretPayload: new TextEncoder().encode(secret),
			})
			status.value = receipt.status
			message.value = 'Zulip API key rotated and rebound.'
		}, 'Zulip API key rotation did not reach confirmed readiness.')
		apiKey.value = ''
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

	return {
		status,
		accountId,
		apiKey,
		busy,
		message,
		messageTone,
		secureHostAvailable,
		stateLabel,
		canQuery,
		canRetire,
		canRotate,
		refresh,
		retire,
		rotateApiKey,
	}
}

function zulipCredentialStateLabel(state: ZulipCredentialBindingStateV1 | undefined): string {
	if (state === ZulipCredentialBindingStateV1.ZULIP_CREDENTIAL_BINDING_STATE_UNCONFIGURED) return 'Unconfigured'
	if (state === ZulipCredentialBindingStateV1.ZULIP_CREDENTIAL_BINDING_STATE_PENDING_RESTART) return 'Pending restart'
	if (state === ZulipCredentialBindingStateV1.ZULIP_CREDENTIAL_BINDING_STATE_ACTIVE) return 'Active'
	if (state === ZulipCredentialBindingStateV1.ZULIP_CREDENTIAL_BINDING_STATE_RETIRED) return 'Retired'
	return 'No account'
}
