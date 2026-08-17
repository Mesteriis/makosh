import { readFileSync } from 'node:fs'

import { describe, expect, it } from 'vitest'

describe('Telegram account Settings composition', () => {
	it('coordinates user-account setup into provider QR without a bot credential surface', () => {
		const settings = read('./TelegramSettingsPanel.vue')
		const setup = read('./TelegramAccountSetupPanel.vue')
		const pairing = read('./TelegramQrPairingPanel.vue')
		const pairingView = read('./TelegramQrPairingView.vue')
		const cloudPassword = read('./TelegramCloudPasswordForm.vue')
		const coordinator = read('../linking/useTelegramQrPairing.ts')

		expect(settings).toContain('@completed="refreshAccounts"')
		expect(settings).not.toContain('TelegramQrPairingPanel')
		expect(setup).toContain('<Steps')
		expect(setup).not.toContain('<Dialog')
		expect(setup).toContain('TelegramQrPairingPanel')
		expect(setup).toContain('data-testid="telegram-qr-primary"')
		expect(setup).toContain('await setup.prepareDevelopmentCredentials()')
		expect(setup).toContain('useTelegramPendingSettingsActivation')
		expect(setup).toContain('await pendingActivation.activate()')
		expect(setup).toContain('The API hash never enters browser JavaScript')
		expect(setup).toContain(':start-request="qrStartRequest"')
		expect(setup).toContain(':configured="setup.configured.value"')
		expect(setup).toContain('@state-change="handleAuthorizationState"')
		expect(setup).toContain('Bot tokens are intentionally not part of this contract')
		expect(pairing).toContain('<TelegramQrPairingView')
		expect(pairing).toContain(':state="pairing.state.value"')
		expect(pairingView).toContain('Telegram user QR login')
		expect(pairingView).toContain('data-testid="telegram-qr-placeholder"')
		expect(pairingView).toContain('<TelegramCloudPasswordForm')
		expect(pairingView).toContain("state === 'waiting_password'")
		expect(cloudPassword).toContain('Telegram cloud password')
		expect(cloudPassword).toContain('autocomplete="current-password"')
		expect(cloudPassword).toContain('Show Telegram cloud password')
		expect(coordinator).toContain('getTelegramAuthorizationStatus')
		expect(coordinator).toContain('telegramQrDataUrl(status.qrLink)')
		expect(setup).toContain('data-auth-method="qr"')
		expect(setup).not.toMatch(/phone number|SMS code/i)
		for (const source of [settings, setup, pairing, pairingView, cloudPassword, coordinator]) {
			expect(source).not.toMatch(/botToken|bot_token|BotFather/)
			expect(source).not.toMatch(/domains\/communications/)
		}
	})
})

function read(relativePath: string): string {
	return readFileSync(new URL(relativePath, import.meta.url), 'utf8')
}
