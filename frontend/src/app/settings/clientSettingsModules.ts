import type { ClientModuleBootstrapV1 } from '../../gen/makosh/gateway/v1/client_bootstrap_pb'

export type ProviderSettingsOwnerId = 'mail' | 'telegram' | 'whatsapp' | 'zulip'
export type SettingsOwnerId = 'system' | 'recovery' | 'maintenance' | 'ai' | 'calendar' | 'signalHub' | ProviderSettingsOwnerId

export const providerModuleIds = {
	mail: 'makosh-mail-runtime',
	telegram: 'makosh-telegram-runtime',
	whatsapp: 'makosh-whatsapp-runtime',
	zulip: 'makosh-zulip-runtime',
} as const

export function clientSettingsModule(
	modules: readonly ClientModuleBootstrapV1[],
	owner: ProviderSettingsOwnerId,
): ClientModuleBootstrapV1 | null {
	const moduleId = providerModuleIds[owner]
	return modules.find((module) => module.moduleId === moduleId) ?? null
}
