<script setup lang="ts">
import { computed, ref } from 'vue'
import type { ClientSurfaceAdapterId } from '../../platform/client-runtime/clientSurfaces'
import type { ClientBootstrapSnapshot } from '../../platform/gateway/clientBootstrap'
import SystemControlPage from '../../platform/system-control/SystemControlPage.vue'
import Icon from '../../shared/ui/Icon.vue'
import MailSettingsComposition from './MailSettingsComposition.vue'
import PlatformMaintenancePanel from './PlatformMaintenancePanel.vue'
import SettingsSurfacePlaceholderPanel from './SettingsSurfacePlaceholderPanel.vue'
import TelegramSettingsPanel from '../../integrations/telegram/presentation/TelegramSettingsPanel.vue'
import WhatsAppSettingsPanel from '../../integrations/whatsapp/presentation/WhatsAppSettingsPanel.vue'
import ZulipSettingsPanel from '../../integrations/zulip/presentation/ZulipSettingsPanel.vue'
import LegacyProviderRecoveryPanel from './recovery/LegacyProviderRecoveryPanel.vue'
import {
	clientSettingsModule,
	type SettingsOwnerId,
} from './clientSettingsModules'
import './appSettingsPage.css'

const props = defineProps<{
	bootstrap: ClientBootstrapSnapshot
	routeDowngradeReason?: string
	developmentProfile: 'disabled' | 'private-lan' | 'loopback-full-stack'
	currentLanguage: string
	languageOptions: readonly { value: string; label: string }[]
	compiledAdapterIds: readonly ClientSurfaceAdapterId[]
	initialOwner?: SettingsOwnerId
}>()
const emit = defineEmits<{
	languageChange: [value: string]
	refreshRequest: []
}>()
const selectedOwner = ref<SettingsOwnerId>(props.initialOwner ?? 'system')

const mailModule = computed(() => clientSettingsModule(props.bootstrap.modules, 'mail'))
const mailContactsSyncModule = computed(() => props.bootstrap.modules.find(
	(module) => module.moduleId === 'makosh-mail-contacts-sync-runtime',
) ?? null)
const telegramModule = computed(() => clientSettingsModule(props.bootstrap.modules, 'telegram'))
const whatsAppModule = computed(() => clientSettingsModule(props.bootstrap.modules, 'whatsapp'))
const zulipModule = computed(() => clientSettingsModule(props.bootstrap.modules, 'zulip'))

const providerNavigation = [
	{ id: 'mail', label: 'Mail', icon: 'tabler:mail' },
	{ id: 'telegram', label: 'Telegram', icon: 'tabler:brand-telegram' },
	{ id: 'whatsapp', label: 'WhatsApp', icon: 'tabler:brand-whatsapp' },
	{ id: 'zulip', label: 'Zulip', icon: 'tabler:brand-zulip' },
] as const

const compositionNavigation = [
	{ id: 'ai', label: 'AI', icon: 'tabler:brain' },
	{ id: 'calendar', label: 'Calendar', icon: 'tabler:calendar-bolt' },
	{ id: 'signalHub', label: 'Signal Hub', icon: 'tabler:signal-3' },
] as const
</script>

<template>
	<section class="app-settings-page">
		<div class="app-settings-workbench">
			<nav class="app-settings-navigation" aria-label="Settings owners">
				<header class="app-settings-navigation__header">
					<span>Settings</span>
					<strong>Owner workbench</strong>
				</header>
				<section class="app-settings-navigation__group">
					<h2>Platform</h2>
					<button
						type="button"
						:class="{ active: selectedOwner === 'system' }"
						@click="selectedOwner = 'system'"
					>
						<Icon class="tree-icon" icon="tabler:heart-rate-monitor" />
						<span class="app-settings-navigation__copy">
							<strong>System Control</strong>
							<small>Kernel recovery and admission</small>
						</span>
					</button>
					<button
						type="button"
						:class="{ active: selectedOwner === 'recovery' }"
						@click="selectedOwner = 'recovery'"
					>
						<Icon class="tree-icon" icon="tabler:database-import" />
						<span class="app-settings-navigation__copy">
							<strong>Account recovery</strong>
							<small>Owner-authorized legacy migration</small>
						</span>
					</button>
					<button
						type="button"
						:class="{ active: selectedOwner === 'maintenance' }"
						@click="selectedOwner = 'maintenance'"
					>
						<Icon class="tree-icon" icon="tabler:wrench" />
						<span class="app-settings-navigation__copy">
							<strong>Maintenance</strong>
							<small>Owner-neutral maintenance composition</small>
						</span>
					</button>
				</section>
				<section class="app-settings-navigation__group">
					<h2>Integrations</h2>
						<button
							v-for="owner in providerNavigation"
							:key="owner.id"
							type="button"
							:class="{ active: selectedOwner === owner.id }"
							@click="selectedOwner = owner.id"
						>
							<Icon class="tree-icon" :icon="owner.icon" />
							<span class="app-settings-navigation__copy">
								<strong>{{ owner.label }}</strong>
								<small>Provider-owned settings</small>
							</span>
						</button>
					</section>
					<section class="app-settings-navigation__group">
						<h2>Compositions</h2>
						<button
							v-for="item in compositionNavigation"
							:key="item.id"
							type="button"
							:class="{ active: selectedOwner === item.id }"
							@click="selectedOwner = item.id as SettingsOwnerId"
						>
							<Icon class="tree-icon" :icon="item.icon" />
							<span class="app-settings-navigation__copy">
								<strong>{{ item.label }}</strong>
								<small>Owner-composed settings surface</small>
							</span>
						</button>
					</section>
				</nav>

			<main class="app-settings-content">
				<SystemControlPage
					v-if="selectedOwner === 'system'"
					:bootstrap="bootstrap"
					:route-downgrade-reason="routeDowngradeReason"
					:development-profile="developmentProfile"
					:current-language="currentLanguage"
					:language-options="languageOptions"
					:compiled-adapter-ids="compiledAdapterIds"
					@language-change="emit('languageChange', $event)"
				/>
				<LegacyProviderRecoveryPanel
					v-else-if="selectedOwner === 'recovery'"
					:mail-module="mailModule"
					:telegram-module="telegramModule"
				/>
				<PlatformMaintenancePanel
					v-else-if="selectedOwner === 'maintenance'"
					:modules="bootstrap.modules"
				/>
				<SettingsSurfacePlaceholderPanel
					v-else-if="selectedOwner === 'ai'"
					title="AI Control Center"
					description="AI settings are composed on the app surface from AI-owned capabilities."
					:capability-ids="['ai_inference_v1', 'communication_reply_suggestion_v1', 'communication_summary_v1', 'communication_translation_v1', 'communication_explanation_v1']"
				/>
				<SettingsSurfacePlaceholderPanel
					v-else-if="selectedOwner === 'calendar'"
					title="Calendar account settings"
					description="Calendar owners remain composable through their own admitted integrations."
					:capability-ids="['calendar_account_settings_composition_v1']"
				/>
				<SettingsSurfacePlaceholderPanel
					v-else-if="selectedOwner === 'signalHub'"
					title="Signal Hub"
					description="Signals stay provider-neutral and compose Review attention with platform telemetry."
					:capability-ids="['review_communications_attention_v1', 'telemetry_diagnostics_surface_v1']"
				/>
				<MailSettingsComposition
					v-else-if="selectedOwner === 'mail'"
					:modules="bootstrap.modules"
					:mail-module="mailModule"
					:sync-module="mailContactsSyncModule"
					@changed="emit('refreshRequest')"
				/>
				<TelegramSettingsPanel v-else-if="selectedOwner === 'telegram'" :module="telegramModule" />
				<WhatsAppSettingsPanel v-else-if="selectedOwner === 'whatsapp'" :module="whatsAppModule" />
				<ZulipSettingsPanel v-else :module="zulipModule" />
			</main>
		</div>
	</section>
</template>
