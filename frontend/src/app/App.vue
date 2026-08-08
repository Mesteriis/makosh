<script setup lang="ts">
import { onMounted, ref } from 'vue'
import AppLayoutRoot from './layout/AppLayoutRoot.vue'
import BrowserGatewayAccessGate from '../platform/gateway/BrowserGatewayAccessGate.vue'
import { BrowserGatewayAccessModeV1 } from '../gen/makosh/gateway/v1/browser_session_pb'
import { fetchBrowserGatewaySessionStatus } from '../platform/gateway/browserGatewaySession'

const authenticated = ref(false)
const checkingSession = ref(true)
const accessMode = ref<
	| BrowserGatewayAccessModeV1.PAIRED
	| BrowserGatewayAccessModeV1.LAN_DEVELOPMENT
	| BrowserGatewayAccessModeV1.LOCAL_DEVELOPMENT
>(BrowserGatewayAccessModeV1.PAIRED)

async function enterAuthenticatedShell(): Promise<void> {
	const status = await fetchBrowserGatewaySessionStatus()
	accessMode.value = status.accessMode
	authenticated.value = true
}

onMounted(async () => {
	try { await enterAuthenticatedShell() } catch { authenticated.value = false } finally { checkingSession.value = false }
})
</script>

<template>
	<AppLayoutRoot v-if="authenticated" :gateway-access-mode="accessMode" />
	<main v-else-if="checkingSession" class="browser-access-gate" data-ui-theme="base-light" aria-busy="true"><section class="browser-access-gate__card"><p>Checking Gateway session…</p></section></main>
	<BrowserGatewayAccessGate v-else @authenticated="enterAuthenticatedShell" />
</template>
