import type { ClientModuleBootstrapV1 } from '../../../gen/makosh/gateway/v1/client_bootstrap_pb'

const WHATSAPP_MODULE_ID = 'makosh-whatsapp-runtime'
const WHATSAPP_ACCOUNT_ID_SETTING = 'whatsapp.account_id'
const MAX_ACCOUNT_ID_BYTES = 512
const textEncoder = new TextEncoder()

export type WhatsAppOperationalAccount = {
	accountId: string
	registrationId: string
}

export function whatsAppOperationalQueryAccounts(
	modules: readonly ClientModuleBootstrapV1[],
): readonly WhatsAppOperationalAccount[] {
	return accountsWithCapability(modules, 'whatsapp.operational.query.v1')
}

export function whatsAppOperationalReplayAccounts(
	modules: readonly ClientModuleBootstrapV1[],
): readonly WhatsAppOperationalAccount[] {
	return accountsWithCapability(modules, 'whatsapp.operational.realtime.v1')
}

export function whatsAppOperationalAccountFingerprint(
	modules: readonly ClientModuleBootstrapV1[],
): string {
	return [
		...whatsAppOperationalQueryAccounts(modules)
			.map((account) => `query:${account.registrationId}:${account.accountId}`),
		...whatsAppOperationalReplayAccounts(modules)
			.map((account) => `replay:${account.registrationId}:${account.accountId}`),
	].join('|')
}

function accountsWithCapability(
	modules: readonly ClientModuleBootstrapV1[],
	capabilityId: string,
): readonly WhatsAppOperationalAccount[] {
	const accounts = new Map<string, WhatsAppOperationalAccount>()
	for (const module of modules) {
		if (
			module.moduleId !== WHATSAPP_MODULE_ID
			|| !module.sectionsEnabled
			|| !module.capabilityIds.includes(capabilityId)
		) continue
		const setting = module.settings?.values.find(
			(entry) => entry.settingId === WHATSAPP_ACCOUNT_ID_SETTING,
		)
		if (setting?.value?.value.case !== 'stringValue') continue
		const accountId = setting.value.value.value.trim()
		if (!validAccountId(accountId) || accounts.has(accountId)) continue
		accounts.set(accountId, { accountId, registrationId: module.registrationId })
	}
	return [...accounts.values()].sort(
		(left, right) => left.accountId.localeCompare(right.accountId),
	)
}

function validAccountId(value: string): boolean {
	if (!value || textEncoder.encode(value).length > MAX_ACCOUNT_ID_BYTES) return false
	for (let index = 0; index < value.length; index += 1) {
		const code = value.charCodeAt(index)
		if (code <= 0x1f || code === 0x7f) return false
	}
	return true
}
