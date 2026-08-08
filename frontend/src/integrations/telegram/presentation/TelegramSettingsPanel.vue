<script setup lang="ts">
import { computed, ref } from 'vue'
import type { ClientModuleBootstrapV1 } from '../../../gen/makosh/gateway/v1/client_bootstrap_pb'
import {
	publicModuleSettingRows,
	publicModuleSettingsReasonCode,
	settingsApplyStateLabel,
} from '../../../platform/gateway/publicModuleSettings'
import ModuleSettingsPanel from '../../../shared/ui/settings/ModuleSettingsPanel.vue'
import type { ModuleSettingsPanelModel } from '../../../shared/ui/settings/ModuleSettingsPanelModel'
import TelegramAccountManagementPanel from './TelegramAccountManagementPanel.vue'
import TelegramAccountSetupPanel from './TelegramAccountSetupPanel.vue'

const TELEGRAM_MODULE_ID = 'makosh-telegram-runtime'
const props = defineProps<{ module: ClientModuleBootstrapV1 | null }>()
const accountRefreshRequest = ref(0)
const model = computed<ModuleSettingsPanelModel>(() => {
	const owned = props.module?.moduleId === TELEGRAM_MODULE_ID ? props.module : null
	const settings = owned?.settings
	return {
		title: 'Telegram',
		description: 'Telegram owns authorization, runtime behavior and provider-specific controls.',
		icon: 'tabler:brand-telegram',
		tone: 'telegram',
		moduleId: TELEGRAM_MODULE_ID,
		registered: Boolean(owned),
		applyState: settings ? settingsApplyStateLabel(settings.applyState) : 'No schema',
		revision: settings ? `${settings.effectiveRevision}/${settings.desiredRevision}` : '—',
		reasonCode: publicModuleSettingsReasonCode(owned),
		settings: publicModuleSettingRows(owned ? [owned] : []),
	}
})

function refreshAccounts(): void {
	accountRefreshRequest.value += 1
}
</script>

<template>
	<div class="provider-settings-stack">
		<ModuleSettingsPanel :model="model" />
		<TelegramAccountSetupPanel :module="module" @completed="refreshAccounts" />
		<TelegramAccountManagementPanel :module="module" :refresh-request="accountRefreshRequest" />
	</div>
</template>
