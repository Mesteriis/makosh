<script setup lang="ts">
import { computed } from 'vue'
import type { ClientModuleBootstrapV1 } from '../../../gen/makosh/gateway/v1/client_bootstrap_pb'
import {
	publicModuleSettingRows,
	publicModuleSettingsReasonCode,
	settingsApplyStateLabel,
} from '../../../platform/gateway/publicModuleSettings'
import ModuleSettingsPanel from '../../../shared/ui/settings/ModuleSettingsPanel.vue'
import type { ModuleSettingsPanelModel } from '../../../shared/ui/settings/ModuleSettingsPanelModel'
import WhatsAppAccountSetupPanel from './WhatsAppAccountSetupPanel.vue'
import WhatsAppPairingPanel from './WhatsAppPairingPanel.vue'

const WHATSAPP_MODULE_ID = 'makosh-whatsapp-runtime'
const props = defineProps<{ module: ClientModuleBootstrapV1 | null }>()
const model = computed<ModuleSettingsPanelModel>(() => {
	const owned = props.module?.moduleId === WHATSAPP_MODULE_ID ? props.module : null
	const settings = owned?.settings
	return {
		title: 'WhatsApp',
		description: 'WhatsApp owns its isolated host profile and provider command behavior.',
		icon: 'tabler:brand-whatsapp',
		tone: 'whatsapp',
		moduleId: WHATSAPP_MODULE_ID,
		registered: Boolean(owned),
		applyState: settings ? settingsApplyStateLabel(settings.applyState) : 'No schema',
		revision: settings ? `${settings.effectiveRevision}/${settings.desiredRevision}` : '—',
		reasonCode: publicModuleSettingsReasonCode(owned),
		settings: publicModuleSettingRows(owned ? [owned] : []),
	}
})
</script>

<template>
	<div class="provider-settings-stack">
		<ModuleSettingsPanel :model="model" />
		<WhatsAppAccountSetupPanel :module="module" />
		<WhatsAppPairingPanel :module="module" />
	</div>
</template>
