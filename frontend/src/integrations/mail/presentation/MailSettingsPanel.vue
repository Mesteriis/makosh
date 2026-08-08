<script setup lang="ts">
import { computed, ref } from 'vue'
import type { ClientModuleBootstrapV1 } from '../../../gen/makosh/gateway/v1/client_bootstrap_pb'
import ModuleSettingsPanel from '../../../shared/ui/settings/ModuleSettingsPanel.vue'
import MailAccountManagementPanel from './MailAccountManagementPanel.vue'
import MailAccountSetupPanel from './MailAccountSetupPanel.vue'
import MailPortabilityPanel from './MailPortabilityPanel.vue'
import MailGmailPermanentDeleteAuthorizationPanel from './MailGmailPermanentDeleteAuthorizationPanel.vue'
import { mailSettingsPanelModel } from './mailSettingsPanelModel'
import { publicModuleSettingRows } from '../../../platform/gateway/publicModuleSettings'

const moduleId = 'makosh-mail-runtime'

const props = defineProps<{ module: ClientModuleBootstrapV1 | null }>()
const emit = defineEmits<{ changed: [] }>()
const accountRefreshRequest = ref(0)
const module = computed(() => props.module?.moduleId === moduleId ? props.module : null)
const moduleSettingsRows = computed(() => publicModuleSettingRows(
	module.value ? [module.value] : [],
))
const model = computed(() => {
	const sourceModel = mailSettingsPanelModel(module.value)
	return { ...sourceModel, settings: moduleSettingsRows.value }
})

function refreshAccounts(): void {
	accountRefreshRequest.value += 1
	emit('changed')
}
</script>

<template>
	<div class="mail-settings-owner">
		<ModuleSettingsPanel :model="model" />
		<MailAccountSetupPanel :module="module" @completed="refreshAccounts" />
		<MailAccountManagementPanel :module="module" :refresh-request="accountRefreshRequest" />
		<MailGmailPermanentDeleteAuthorizationPanel />
		<MailPortabilityPanel :module="module" />
	</div>
</template>
