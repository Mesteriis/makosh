import { computed, ref } from 'vue'

import {
	ClientSettingsApplyStateV1,
	type ClientModuleBootstrapV1,
} from '../../../gen/makosh/gateway/v1/client_bootstrap_pb'
import { OwnerModuleSettingsClientV1 } from '../../../platform/settings/ownerModuleSettingsClient'

const MAIL_MODULE_ID = 'makosh-mail-runtime'
const MAIL_STORAGE_CAPABILITY_ID = 'mail.storage.v1'

type MailSettingsActivationPortV1 = Pick<OwnerModuleSettingsClientV1, 'applyManagedIntegration'>

export function useMailPendingSettingsActivation(
	module: () => ClientModuleBootstrapV1 | null,
	activation: MailSettingsActivationPortV1 = new OwnerModuleSettingsClientV1(),
) {
	const busy = ref(false)
	const message = ref('')
	const messageTone = ref<'neutral' | 'success' | 'error'>('neutral')
	const activatedTargetIds = ref<ReadonlySet<string>>(new Set())
	const ownedModule = computed(() => module()?.moduleId === MAIL_MODULE_ID ? module() : null)
	const pendingTargets = computed(() => {
		const current = ownedModule.value
		if (!current) return []
		return current.settingsTargets.filter((target) =>
			target.configurationInstanceId !== current.registrationId
			&& !activatedTargetIds.value.has(target.configurationInstanceId)
			&& target.applyState === ClientSettingsApplyStateV1.PENDING_VALIDATION)
	})
	const canActivate = computed(() => Boolean(
		pendingTargets.value.length > 0
		&& ownedModule.value?.capabilityIds.includes(MAIL_STORAGE_CAPABILITY_ID),
	))

	async function activate(): Promise<boolean> {
		const current = ownedModule.value
		const targets = [...pendingTargets.value]
		if (!current || !canActivate.value || busy.value) return false
		busy.value = true
		message.value = ''
		try {
			for (const target of targets) {
				await activation.applyManagedIntegration({
					registrationId: current.registrationId,
					storageCapabilityId: MAIL_STORAGE_CAPABILITY_ID,
					configurationInstanceId: target.configurationInstanceId,
					expectedDesiredRevision: target.desiredRevision,
					requestHostBridge: false,
				})
			}
			activatedTargetIds.value = new Set([
				...activatedTargetIds.value,
				...targets.map((target) => target.configurationInstanceId),
			])
			message.value = `${targets.length} recovered Mail account${targets.length === 1 ? '' : 's'} activated.`
			messageTone.value = 'success'
			return true
		} catch {
			message.value = 'Recovered Mail account activation did not reach confirmed readiness.'
			messageTone.value = 'error'
			return false
		} finally {
			busy.value = false
		}
	}

	return {
		busy,
		message,
		messageTone,
		pendingCount: computed(() => pendingTargets.value.length),
		canActivate,
		activate,
	}
}
