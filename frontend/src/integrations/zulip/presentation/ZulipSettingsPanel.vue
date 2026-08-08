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
import ZulipAccountManagementPanel from './ZulipAccountManagementPanel.vue'
import ZulipAccountSetupPanel from './ZulipAccountSetupPanel.vue'

const ZULIP_MODULE_ID = 'makosh-zulip-runtime'
const props = defineProps<{ module: ClientModuleBootstrapV1 | null }>()
const model = computed<ModuleSettingsPanelModel>(() => {
	const owned = props.module?.moduleId === ZULIP_MODULE_ID ? props.module : null
	const settings = owned?.settings
	return {
		title: 'Zulip',
		description: 'Zulip owns account identity, provider delivery and attachment operation settings.',
		icon: 'tabler:brand-zulip',
		tone: 'zulip',
		moduleId: ZULIP_MODULE_ID,
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
		<ZulipAccountSetupPanel :module="module" />
		<ZulipAccountManagementPanel :module="module" />
	</div>
</template>
