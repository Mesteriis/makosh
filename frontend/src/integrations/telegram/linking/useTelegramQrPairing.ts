import { computed, onBeforeUnmount, ref, watch } from 'vue'
import type { ClientModuleBootstrapV1 } from '../../../gen/makosh/gateway/v1/client_bootstrap_pb'
import {
	getTelegramAuthorizationStatus,
	submitTelegramAuthorizationPassword,
} from '../api/telegramAuthorizationGateway'
import {
	openTelegramAuthorizationRealtime,
	type TelegramAuthorizationRealtimeBinding,
} from '../api/telegramAuthorizationRealtime'
import { telegramQrDataUrl } from './telegramQrArtifact'

const TELEGRAM_AUTHORIZATION_CAPABILITY_ID = 'telegram.authorization.v1'
const TELEGRAM_AUTHORIZATION_REALTIME_CAPABILITY_ID = 'telegram.authorization.realtime.v1'

export function useTelegramQrPairing(
	module: () => ClientModuleBootstrapV1 | null,
	startRequest: () => number = () => 0,
) {
	const state = ref('unknown')
	const qrDataUrl = ref('')
	const passwordHint = ref('')
	const password = ref('')
	const busy = ref(false)
	const message = ref('')
	const messageTone = ref<'neutral' | 'success' | 'error'>('neutral')
	let realtime: TelegramAuthorizationRealtimeBinding | null = null
	const admitted = computed(
		() => {
			const capabilities = module()?.capabilityIds ?? []
			return capabilities.includes(TELEGRAM_AUTHORIZATION_CAPABILITY_ID)
				&& capabilities.includes(TELEGRAM_AUTHORIZATION_REALTIME_CAPABILITY_ID)
		},
	)
	const configured = computed(() => (module()?.settings?.effectiveRevision ?? 0n) > 0n)
	const canRefresh = computed(() => admitted.value && configured.value)
	let pendingStartRequest = 0
	let handledStartRequest = 0

	async function refresh(): Promise<void> {
		if (!canRefresh.value || busy.value) return
		busy.value = true
		try {
			const status = await getTelegramAuthorizationStatus()
			state.value = status.state || 'unknown'
			passwordHint.value = status.passwordHint ?? ''
			qrDataUrl.value = status.qrLink
				? await telegramQrDataUrl(status.qrLink)
				: ''
			message.value = statusMessage(state.value)
			messageTone.value = state.value === 'ready' ? 'success' : 'neutral'
		} catch {
			clearQr()
			message.value = 'Telegram authorization status is unavailable.'
			messageTone.value = 'error'
		} finally {
			busy.value = false
		}
	}

	async function submitPassword(): Promise<void> {
		if (!password.value.trim()) return
		busy.value = true
		try {
			await submitTelegramAuthorizationPassword(password.value)
			password.value = ''
			message.value = 'Telegram 2FA password accepted. Waiting for authorization.'
			messageTone.value = 'neutral'
		} catch {
			password.value = ''
			message.value = 'Telegram rejected the 2FA continuation.'
			messageTone.value = 'error'
		} finally {
			busy.value = false
		}
	}

	function openRealtime(): void {
		if (realtime) return
		realtime = openTelegramAuthorizationRealtime(
			() => void refresh(),
			() => {
				message.value = 'Telegram authorization realtime is unavailable. Use Refresh for recovery.'
				messageTone.value = 'error'
			},
		)
	}

	function closeRealtime(): void {
		realtime?.close()
		realtime = null
	}

	function clearQr(): void {
		qrDataUrl.value = ''
	}

	watch(
		[() => startRequest(), canRefresh],
		([requested, ready]) => {
			if (!ready) closeRealtime()
			if (requested > handledStartRequest) pendingStartRequest = requested
			if (!pendingStartRequest) return
			if (!ready) {
				message.value = 'Telegram account saved. Waiting for managed Settings before requesting the provider QR.'
				messageTone.value = 'neutral'
				return
			}
			handledStartRequest = pendingStartRequest
			pendingStartRequest = 0
			openRealtime()
			void refresh()
		},
		{ immediate: true },
	)

	onBeforeUnmount(() => {
		closeRealtime()
		clearQr()
		password.value = ''
	})

	return {
		state,
		qrDataUrl,
		passwordHint,
		password,
		busy,
		message,
		messageTone,
		admitted,
		configured,
		canRefresh,
		refresh,
		submitPassword,
	}
}

function statusMessage(state: string): string {
	switch (state) {
		case 'waiting_qr_scan':
			return 'Scan this QR code from Telegram → Settings → Devices → Link Desktop Device.'
		case 'waiting_password':
			return 'Telegram requires the account 2FA password to finish linking.'
		case 'ready':
			return 'Telegram account is authorized.'
		case 'error':
		case 'closed':
			return 'Telegram authorization stopped before completion.'
		default:
			return 'Preparing a provider-issued Telegram QR code.'
	}
}
