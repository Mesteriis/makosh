import { nextTick, ref } from 'vue'
import { beforeEach, describe, expect, it, vi } from 'vitest'

const {
	getTelegramAuthorizationStatus,
	submitTelegramAuthorizationPassword,
	telegramQrDataUrl,
	openTelegramAuthorizationRealtime,
} = vi.hoisted(() => ({
	getTelegramAuthorizationStatus: vi.fn(),
	submitTelegramAuthorizationPassword: vi.fn(),
	telegramQrDataUrl: vi.fn(),
	openTelegramAuthorizationRealtime: vi.fn(),
}))

vi.mock('../api/telegramAuthorizationGateway', () => ({
	getTelegramAuthorizationStatus,
	submitTelegramAuthorizationPassword,
}))
vi.mock('./telegramQrArtifact', () => ({ telegramQrDataUrl }))
vi.mock('../api/telegramAuthorizationRealtime', () => ({ openTelegramAuthorizationRealtime }))

import { useTelegramQrPairing } from './useTelegramQrPairing'

describe('Telegram QR pairing', () => {
	beforeEach(() => {
		getTelegramAuthorizationStatus.mockReset()
		submitTelegramAuthorizationPassword.mockReset()
		telegramQrDataUrl.mockReset()
		openTelegramAuthorizationRealtime.mockReset()
		openTelegramAuthorizationRealtime.mockReturnValue({ close: vi.fn() })
	})

	it('waits for effective Settings and then requests the real TDLib QR automatically', async () => {
		const module = ref({
			capabilityIds: [
				'telegram.authorization.realtime.v1',
				'telegram.authorization.v1',
			],
			settings: { effectiveRevision: 0n },
		})
		const startRequest = ref(0)
		getTelegramAuthorizationStatus.mockResolvedValue({
			state: 'waiting_qr_scan',
			qrLink: 'tg://login?token=provider-token',
		})
		telegramQrDataUrl.mockResolvedValue('data:image/png;base64,provider-qr')
		const pairing = useTelegramQrPairing(
			() => module.value as never,
			() => startRequest.value,
		)

		startRequest.value = 1
		await nextTick()
		expect(getTelegramAuthorizationStatus).not.toHaveBeenCalled()
		expect(pairing.message.value).toContain('Waiting for managed Settings')

		module.value.settings.effectiveRevision = 1n
		await nextTick()
		await vi.waitFor(() => expect(getTelegramAuthorizationStatus).toHaveBeenCalledOnce())
		await vi.waitFor(() => expect(pairing.qrDataUrl.value).toBe(
			'data:image/png;base64,provider-qr',
		))
		expect(telegramQrDataUrl).toHaveBeenCalledWith('tg://login?token=provider-token')
		expect(openTelegramAuthorizationRealtime).toHaveBeenCalledOnce()
		expect(String(useTelegramQrPairing)).not.toMatch(/setTimeout|setInterval|poll/i)
	})

	it('requests the real TDLib QR after the setup workflow completes before bootstrap refreshes', async () => {
		const module = ref({
			capabilityIds: [
				'telegram.authorization.realtime.v1',
				'telegram.authorization.v1',
			],
			settings: { effectiveRevision: 0n },
		})
		const startRequest = ref(0)
		const configuredLocally = ref(false)
		getTelegramAuthorizationStatus.mockResolvedValue({
			state: 'waiting_qr_scan',
			qrLink: 'tg://login?token=provider-token',
		})
		telegramQrDataUrl.mockResolvedValue('data:image/png;base64,provider-qr')
		const pairing = useTelegramQrPairing(
			() => module.value as never,
			() => startRequest.value,
			() => configuredLocally.value,
		)

		configuredLocally.value = true
		startRequest.value = 1
		await nextTick()

		await vi.waitFor(() => expect(getTelegramAuthorizationStatus).toHaveBeenCalledOnce())
		await vi.waitFor(() => expect(pairing.qrDataUrl.value).toBe(
			'data:image/png;base64,provider-qr',
		))
		expect(pairing.configured.value).toBe(true)
	})

	it('preserves the realtime password state while the runtime finishes publishing it', async () => {
		const module = ref({
			capabilityIds: [
				'telegram.authorization.realtime.v1',
				'telegram.authorization.v1',
			],
			settings: { effectiveRevision: 1n },
		})
		const startRequest = ref(0)
		let onStatusChanged: ((state: string) => void) | undefined
		openTelegramAuthorizationRealtime.mockImplementation((callback) => {
			onStatusChanged = callback
			return { close: vi.fn() }
		})
		getTelegramAuthorizationStatus
			.mockResolvedValueOnce({
				state: 'waiting_qr_scan',
				qrLink: 'tg://login?token=provider-token',
			})
			.mockRejectedValueOnce(new Error('runtime busy'))
		telegramQrDataUrl.mockResolvedValue('data:image/png;base64,provider-qr')
		const pairing = useTelegramQrPairing(
			() => module.value as never,
			() => startRequest.value,
		)

		startRequest.value = 1
		await nextTick()
		await vi.waitFor(() => expect(pairing.state.value).toBe('waiting_qr_scan'))

		onStatusChanged?.('waiting_password')

		await vi.waitFor(() => expect(getTelegramAuthorizationStatus).toHaveBeenCalledTimes(2))
		expect(pairing.state.value).toBe('waiting_password')
		expect(pairing.message.value).toContain('2FA password')
		expect(pairing.message.value).not.toContain('unavailable')
	})
})
