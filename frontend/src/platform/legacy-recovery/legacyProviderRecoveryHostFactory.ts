import { DevelopmentLegacyProviderRecoveryHostV1 } from './developmentLegacyProviderRecoveryHost'
import type { LegacyProviderRecoveryHostV1 } from './legacyProviderRecoveryHost'

export function hasLegacyProviderRecoveryHostV1(): boolean {
	return import.meta.env.VITE_MAKOSH_LEGACY_PROVIDER_RECOVERY === '1'
}

export function createLegacyProviderRecoveryHostV1(): LegacyProviderRecoveryHostV1 {
	if (hasLegacyProviderRecoveryHostV1()) {
		return new DevelopmentLegacyProviderRecoveryHostV1()
	}
	return new UnavailableLegacyProviderRecoveryHostV1()
}

class UnavailableLegacyProviderRecoveryHostV1 implements LegacyProviderRecoveryHostV1 {
	start(): Promise<never> {
		return Promise.reject(new Error('legacy provider recovery host is unavailable'))
	}

	source(): Promise<never> {
		return Promise.reject(new Error('legacy provider recovery host is unavailable'))
	}

	sealSource(): Promise<never> {
		return Promise.reject(new Error('legacy provider recovery host is unavailable'))
	}

	beginStep(): Promise<never> {
		return Promise.reject(new Error('legacy provider recovery host is unavailable'))
	}

	completeStep(): Promise<never> {
		return Promise.reject(new Error('legacy provider recovery host is unavailable'))
	}

	finishCandidate(): Promise<never> {
		return Promise.reject(new Error('legacy provider recovery host is unavailable'))
	}

	cancel(): Promise<void> {
		return Promise.resolve()
	}
}
