import type { ClientModuleBootstrapV1 } from '../../../gen/makosh/gateway/v1/client_bootstrap_pb'

const ZULIP_MODULE_ID = 'makosh-zulip-runtime'
const ZULIP_ACCOUNT_ID_SETTING = 'zulip.account_id'
const MAX_ACCOUNT_ID_BYTES = 512
const textEncoder = new TextEncoder()

export type ZulipOperationalAccount = {
	accountId: string
	registrationId: string
}

export function zulipOperationalQueryAccounts(
	modules: readonly ClientModuleBootstrapV1[],
): readonly ZulipOperationalAccount[] {
	return accountsWithCapability(modules, 'zulip.operational.query.v1')
}

export function zulipOperationalReplayAccounts(
	modules: readonly ClientModuleBootstrapV1[],
): readonly ZulipOperationalAccount[] {
	return accountsWithCapability(modules, 'zulip.operational.realtime.v1')
}

export function zulipOperationalAccountFingerprint(
	modules: readonly ClientModuleBootstrapV1[],
): string {
	return [
		...zulipOperationalQueryAccounts(modules)
			.map((account) => `query:${account.registrationId}:${account.accountId}`),
		...zulipOperationalReplayAccounts(modules)
			.map((account) => `replay:${account.registrationId}:${account.accountId}`),
	].join('|')
}

function accountsWithCapability(
	modules: readonly ClientModuleBootstrapV1[],
	capabilityId: string,
): readonly ZulipOperationalAccount[] {
	const accounts = new Map<string, ZulipOperationalAccount>()
	for (const module of modules) {
		if (
			module.moduleId !== ZULIP_MODULE_ID
			|| !module.sectionsEnabled
			|| !module.capabilityIds.includes(capabilityId)
		) continue
		const setting = module.settings?.values.find(
			(entry) => entry.settingId === ZULIP_ACCOUNT_ID_SETTING,
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
	return Boolean(value)
		&& textEncoder.encode(value).length <= MAX_ACCOUNT_ID_BYTES
		&& !value.includes('\u0000')
		&& !value.includes('\r')
		&& !value.includes('\n')
}
