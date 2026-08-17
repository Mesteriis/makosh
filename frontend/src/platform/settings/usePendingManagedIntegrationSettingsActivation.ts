import { computed, ref } from 'vue'

import {
	ClientSettingsApplyStateV1,
	type ClientModuleBootstrapV1,
} from '../../gen/makosh/gateway/v1/client_bootstrap_pb'
import { OwnerModuleSettingsClientV1 } from './ownerModuleSettingsClient'

type ManagedSettingsActivationPortV1 = Pick<
	OwnerModuleSettingsClientV1,
	'applyManagedIntegration'
>

export type PendingManagedIntegrationSettingsActivationOptionsV1 = {
	moduleId: string
	storageCapabilityId: string
	includeTarget: (
		target: PendingManagedIntegrationSettingsTargetV1,
		module: ClientModuleBootstrapV1,
	) => boolean
	requestHostBridge?: boolean
	targetLabel: string
}

export type PendingManagedIntegrationSettingsTargetV1 = {
	configurationInstanceId: string
	desiredRevision: bigint
	effectiveRevision: bigint
	applyState: ClientSettingsApplyStateV1
	sanitizedReasonCode: string
}

export function usePendingManagedIntegrationSettingsActivation(
	module: () => ClientModuleBootstrapV1 | null,
	options: PendingManagedIntegrationSettingsActivationOptionsV1,
	activation: ManagedSettingsActivationPortV1 = new OwnerModuleSettingsClientV1(),
) {
	const busy = ref(false)
	const message = ref('')
	const messageTone = ref<'neutral' | 'success' | 'error'>('neutral')
	const activatedTargetRevisions = ref<ReadonlySet<string>>(new Set())
	const ownedModule = computed(() => module()?.moduleId === options.moduleId ? module() : null)
	const pendingTargets = computed(() => {
		const current = ownedModule.value
		if (!current) return []
		const targets = new Map<string, PendingManagedIntegrationSettingsTargetV1>()
		if (current.settings) {
			targets.set(current.registrationId, {
				configurationInstanceId: current.registrationId,
				desiredRevision: current.settings.desiredRevision,
				effectiveRevision: current.settings.effectiveRevision,
				applyState: current.settings.applyState,
				sanitizedReasonCode: current.settings.sanitizedReasonCode,
			})
		}
		for (const target of current.settingsTargets) {
			targets.set(target.configurationInstanceId, target)
		}
		return [...targets.values()].filter((target) =>
			options.includeTarget(target, current)
			&& !activatedTargetRevisions.value.has(targetRevisionKey(target))
			&& (target.applyState === ClientSettingsApplyStateV1.PENDING_VALIDATION
				|| (target.applyState === ClientSettingsApplyStateV1.BLOCKED_CONFIG
					&& target.sanitizedReasonCode === 'managed_readiness_failed')))
	})
	const canActivate = computed(() => Boolean(
		pendingTargets.value.length > 0
		&& ownedModule.value?.capabilityIds.includes(options.storageCapabilityId),
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
					storageCapabilityId: options.storageCapabilityId,
					configurationInstanceId: target.configurationInstanceId,
					expectedDesiredRevision: target.desiredRevision,
					requestHostBridge: options.requestHostBridge ?? false,
				})
			}
			activatedTargetRevisions.value = new Set([
				...activatedTargetRevisions.value,
				...targets.map(targetRevisionKey),
			])
			message.value = `${targets.length} recovered ${options.targetLabel}${targets.length === 1 ? '' : 's'} activated.`
			messageTone.value = 'success'
			return true
		} catch {
			message.value = `Recovered ${options.targetLabel} activation did not reach confirmed readiness.`
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

function targetRevisionKey(target: PendingManagedIntegrationSettingsTargetV1): string {
	return `${target.configurationInstanceId}:${target.desiredRevision}`
}
