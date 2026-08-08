import { computed, ref } from 'vue'
import type { ClientModuleBootstrapV1 } from '../../../gen/makosh/gateway/v1/client_bootstrap_pb'
import { publicModuleStringSetting } from '../../../platform/gateway/publicModuleSettings'
import {
	NativeWhatsAppCompanionHostV1,
	type WhatsAppCompanionHostV1,
} from '../host/whatsAppCompanionHost'

export function useWhatsAppPairing(
	module: () => ClientModuleBootstrapV1 | null,
	host: WhatsAppCompanionHostV1 = new NativeWhatsAppCompanionHostV1(),
) {
	const busy = ref(false)
	const message = ref('')
	const messageTone = ref<'neutral' | 'success' | 'error'>('neutral')
	const accountId = computed(
		() => publicModuleStringSetting(module(), 'whatsapp.account_id') ?? '',
	)
	const nativeHostAvailable = host.available()
	const canOpen = computed(() => Boolean(accountId.value) && nativeHostAvailable)

	async function open(): Promise<void> {
		if (!accountId.value) {
			message.value = 'Configure a WhatsApp account ID before opening QR pairing.'
			messageTone.value = 'neutral'
			return
		}
		if (!nativeHostAvailable) {
			message.value = 'The real WhatsApp QR is available only in the desktop shell.'
			messageTone.value = 'neutral'
			return
		}
		busy.value = true
		message.value = ''
		try {
			const manifest = await host.open(accountId.value)
			if (!manifest.ownerVisible) throw new Error('whatsapp_companion_not_visible')
			message.value = 'WhatsApp Web is open in an owner-visible window. Scan the provider QR code there.'
			messageTone.value = 'success'
		} catch {
			message.value = 'WhatsApp pairing window could not be opened by the desktop host.'
			messageTone.value = 'error'
		} finally {
			busy.value = false
		}
	}

	return {
		accountId,
		busy,
		message,
		messageTone,
		nativeHostAvailable,
		canOpen,
		open,
	}
}
