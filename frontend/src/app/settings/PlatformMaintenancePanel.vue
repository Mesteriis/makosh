<script setup lang="ts">
import { computed } from 'vue'

import type { ClientModuleBootstrapV1 } from '../../gen/makosh/gateway/v1/client_bootstrap_pb'
import Icon from '../../shared/ui/Icon.vue'
import './platformMaintenancePanel.css'

const props = defineProps<{
	modules: readonly ClientModuleBootstrapV1[]
}>()

const maintenanceModules = computed(() => {
	const modules: ClientModuleBootstrapV1[] = []
	for (const module of props.modules) {
		if (module.capabilityIds.includes('whole_instance_backup_v1')) {
			modules.push(module)
		}
	}
	return modules
})
</script>

<template>
	<section class="platform-maintenance">
		<header class="platform-maintenance__header">
			<strong>Platform maintenance</strong>
			<p>Owner-neutral maintenance composition from platform owners and offline recovery surfaces.</p>
		</header>
		<p v-if="maintenanceModules.length === 0" class="platform-maintenance__placeholder">
			Maintenance operations are exposed as maintenance surfaces and offline commands. No active platform maintenance modules were admitted in this bootstrap.
		</p>
		<div v-else class="platform-maintenance__list">
			<article v-for="module in maintenanceModules" :key="module.registrationId" class="platform-maintenance__row">
				<Icon icon="tabler:wrench" />
				<div>
					<strong>{{ module.moduleId }}</strong>
					<small>{{ module.registrationId }} · grants {{ module.capabilityIds.length }}</small>
				</div>
				<span>{{ module.sectionsEnabled ? 'Composed' : 'Blocked' }}</span>
			</article>
		</div>
		<div class="platform-maintenance__skeleton">
			<strong>Action surface</strong>
			<p>Command entrypoints are intentionally not yet available in this app surface. Maintenance operations remain available via offline recovery surfaces and explicit maintenance sessions.</p>
		</div>
	</section>
</template>
