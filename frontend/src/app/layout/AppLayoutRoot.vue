<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import Toast from '../../shared/ui/Toast.vue'
import AppSettingsPage from '../settings/AppSettingsPage.vue'
import { useClientNavigationSurface } from '../queries/useClientNavigationSurface'
import AppLayout from '../../shared/ui/shell/AppLayout.vue'
import AppNavbar from '../../shared/ui/shell/AppNavbar.vue'
import { BrowserGatewayAccessModeV1 } from '../../gen/makosh/gateway/v1/browser_session_pb'
import { compiledClientSurfaceAdapterIds } from '../client-surfaces/compiledClientSurfaceAdapters'
import CanonicalCommunicationsRoute from '../../domains/communications/views/CanonicalCommunicationsRoute.vue'
import TelegramOperationalRoute from '../../integrations/telegram/views/TelegramOperationalRoute.vue'
import { hasClientModuleCapability } from '../client-surfaces/clientModuleCapabilities'
import WhatsAppOperationalRoute from '../../integrations/whatsapp/views/WhatsAppOperationalRoute.vue'
import MailOperationalRoute from '../../integrations/mail/views/MailOperationalRoute.vue'
import ZulipOperationalRoute from '../../integrations/zulip/views/ZulipOperationalRoute.vue'
import TasksWorkspaceView from '../../domains/tasks/views/TasksWorkspaceView.vue'
import KnowledgeWorkspaceView from '../../domains/knowledge/views/KnowledgeWorkspaceView.vue'
import CalendarWorkspaceView from '../../domains/calendar/views/CalendarWorkspaceView.vue'
import OrganizationsWorkspaceView from '../../domains/organizations/views/OrganizationsWorkspaceView.vue'
import ProjectsWorkspaceView from '../../domains/projects/views/ProjectsWorkspaceView.vue'
import ObligationsWorkspaceView from '../../domains/obligations/views/ObligationsWorkspaceView.vue'
import DecisionsWorkspaceView from '../../domains/decisions/views/DecisionsWorkspaceView.vue'
import DocumentsWorkspaceView from '../../domains/documents/views/DocumentsWorkspaceView.vue'
import CommunicationsEvidenceExportWorkflow from '../../workflows/communications-export/CommunicationsEvidenceExportWorkflow.vue'
import AttachmentPreviewWorkflow from '../../workflows/attachment-preview/AttachmentPreviewWorkflow.vue'
import CallTranscriptionWorkflow from '../../workflows/call-transcription/CallTranscriptionWorkflow.vue'
import { useMailMessageContent } from '../../workflows/mail-message-content/useMailMessageContent'
import {
	providerAccountIdFromRoute,
	providerAccountNavigationLevel,
	type ProviderAccountNavigationSnapshot,
} from '../../shared/ui/shell/providerAccountNavigation'

const props = defineProps<{ gatewayAccessMode: BrowserGatewayAccessModeV1 }>()

const navbar = useClientNavigationSurface()
const breadcrumbs = navbar.breadcrumbs
const currentTheme = navbar.currentTheme
const currentThemeFamily = navbar.currentThemeFamily
const currentThemeMode = navbar.currentThemeMode
const healthChecks = navbar.healthChecks
const mailAccountNavigation = ref<ProviderAccountNavigationSnapshot>()
const telegramAccountNavigation = ref<ProviderAccountNavigationSnapshot>()
const requestedMailAccountId = ref<string>()
const requestedTelegramAccountId = ref<string>()
const navigationLevels = computed(() => {
	const levels = [...navbar.navigationLevels.value]
	if (navbar.selectedRouteId.value === 'communications-mail') {
		levels.push(providerAccountNavigationLevel('mail', mailAccountNavigation.value))
	}
	if (navbar.selectedRouteId.value === 'communications-telegram') {
		levels.push(providerAccountNavigationLevel('telegram', telegramAccountNavigation.value))
	}
	return levels
})
const notifications = navbar.notifications
const notificationsCount = navbar.notificationsCount
const notificationToasts = navbar.notificationToasts
const selectedRouteId = navbar.selectedRouteId
const selectedTopLevelRouteId = navbar.selectedTopLevelRouteId
const bootstrap = navbar.bootstrap
const routeDowngradeReason = navbar.routeDowngradeReason
const communicationsSavedSearchAvailable = computed(() =>
	hasClientModuleCapability(bootstrap.value, 'communications.saved-search.v1'),
)
const communicationsSenderInsightsAvailable = computed(() =>
	hasClientModuleCapability(bootstrap.value, 'communications.sender-insights.v1'),
)
const communicationsExportAvailable = computed(() =>
	hasClientModuleCapability(bootstrap.value, 'communications.export.v1'),
)
const communicationsContentAvailable = computed(() =>
	hasClientModuleCapability(bootstrap.value, 'communications.query.v1')
	&& hasClientModuleCapability(bootstrap.value, 'communications.content.v1'),
)
const attachmentPreviewAvailable = computed(() =>
	hasClientModuleCapability(bootstrap.value, 'attachment_preview.client.v1'),
)
const attachmentPreviewEvidenceReplayAvailable = computed(() =>
	hasClientModuleCapability(bootstrap.value, 'attachment-preview-evidence-replay.command.v1'),
)
const callTranscriptionAvailable = computed(() =>
	hasClientModuleCapability(bootstrap.value, 'call_transcription.v1'),
)
const currentCanonicalMessageId = ref<Uint8Array>()
const currentAttachmentAnchorId = ref<Uint8Array>()
const telegramCommandAvailable = computed(() =>
	hasClientModuleCapability(bootstrap.value, 'telegram.command.v1'),
)
const telegramAuthorizationAvailable = computed(() =>
	hasClientModuleCapability(bootstrap.value, 'telegram.authorization.v1'),
)
const telegramLifecycleAvailable = computed(() =>
	hasClientModuleCapability(bootstrap.value, 'telegram.lifecycle.v1'),
)
const telegramReconfigurationAvailable = computed(() =>
	hasClientModuleCapability(bootstrap.value, 'telegram.reconfiguration.v1'),
)
const telegramQueryAvailable = computed(() =>
	hasClientModuleCapability(bootstrap.value, 'telegram.query.v1'),
)
const whatsAppCommandAvailable = computed(() =>
	hasClientModuleCapability(bootstrap.value, 'whatsapp.command.v1'),
)
const whatsAppOperationalQueryAvailable = computed(() =>
	hasClientModuleCapability(bootstrap.value, 'whatsapp.operational.query.v1'),
)
const whatsAppOperationalRealtimeAvailable = computed(() =>
	hasClientModuleCapability(bootstrap.value, 'whatsapp.operational.realtime.v1'),
)
const mailDeliveryAvailable = computed(() =>
	hasClientModuleCapability(bootstrap.value, 'mail.delivery.v1'),
)
const mailAccountCatalogAvailable = computed(() =>
	hasClientModuleCapability(bootstrap.value, 'mail.account.catalog.query.v1'),
)
const mailCompositionCommandAvailable = computed(() =>
	hasClientModuleCapability(bootstrap.value, 'mail.composition.command.v1'),
)
const mailCompositionQueryAvailable = computed(() =>
	hasClientModuleCapability(bootstrap.value, 'mail.composition.query.v1'),
)
const mailSyncAvailable = computed(() =>
	hasClientModuleCapability(bootstrap.value, 'mail.sync.v1'),
)
const mailOperationalQueryAvailable = computed(() =>
	hasClientModuleCapability(bootstrap.value, 'mail.operational.query.v1'),
)
const mailMessageFlagCommandAvailable = computed(() =>
	hasClientModuleCapability(bootstrap.value, 'mail.message-flags.command.v1'),
)
const mailMessageFlagQueryAvailable = computed(() =>
	hasClientModuleCapability(bootstrap.value, 'mail.message-flags.query.v1'),
)
const mailMessageLocationCommandAvailable = computed(() =>
	hasClientModuleCapability(bootstrap.value, 'mail.message-location.command.v1'),
)
const mailMessageLocationQueryAvailable = computed(() =>
	hasClientModuleCapability(bootstrap.value, 'mail.message-location.query.v1'),
)
const mailMessagePermanentDeleteCommandAvailable = computed(() =>
	hasClientModuleCapability(bootstrap.value, 'mail.message-permanent-delete.command.v1'),
)
const mailMessagePermanentDeleteQueryAvailable = computed(() =>
	hasClientModuleCapability(bootstrap.value, 'mail.message-permanent-delete.query.v1'),
)
const mailSyncHealthAvailable = computed(() =>
	hasClientModuleCapability(bootstrap.value, 'mail.sync.health.query.v1'),
)
const mailMessageContent = useMailMessageContent({
	canRead: () => communicationsContentAvailable.value,
})
const zulipCommandAvailable = computed(() =>
	hasClientModuleCapability(bootstrap.value, 'zulip.command.v1'),
)
const zulipOperationalQueryAvailable = computed(() =>
	hasClientModuleCapability(bootstrap.value, 'zulip.operational.query.v1'),
)
const zulipOperationalRealtimeAvailable = computed(() =>
	hasClientModuleCapability(bootstrap.value, 'zulip.operational.realtime.v1'),
)

function selectNavigationItem(itemId: string): void {
	const mailAccountId = providerAccountIdFromRoute('mail', itemId)
	if (mailAccountId !== undefined) {
		requestedMailAccountId.value = mailAccountId
		if (mailAccountNavigation.value) {
			mailAccountNavigation.value = {
				...mailAccountNavigation.value,
				selectedAccountId: mailAccountId,
			}
		}
		return
	}
	const telegramAccountId = providerAccountIdFromRoute('telegram', itemId)
	if (telegramAccountId !== undefined) {
		requestedTelegramAccountId.value = telegramAccountId
		if (telegramAccountNavigation.value) {
			telegramAccountNavigation.value = {
				...telegramAccountNavigation.value,
				selectedAccountId: telegramAccountId,
			}
		}
		return
	}
	navbar.selectNavigationItem(itemId)
}

function openMailMessageContent(evidenceId: Uint8Array | undefined): void {
	void mailMessageContent.open(evidenceId)
}

watch([currentTheme, currentThemeFamily, currentThemeMode], ([theme, family, mode]) => {
	document.documentElement.setAttribute('data-ui-theme', theme)
	document.documentElement.setAttribute('data-ui-theme-family', family)
	document.documentElement.setAttribute('data-ui-theme-mode', mode)
}, { immediate: true })

</script>

<template>
	<section
		class="app-layout-root"
		:data-ui-theme="currentTheme"
		:data-ui-theme-family="currentThemeFamily"
		:data-ui-theme-mode="currentThemeMode"
	>
		<Toast
			class="app-layout-notification-toasts"
			close-label="Закрыть уведомление"
			:default-toasts="notificationToasts"
			:duration="navbar.notificationToastVisibleMs"
		>
			<AppLayout>
				<template #topbar>
					<AppNavbar
						:breadcrumbs="breadcrumbs"
						:health-checks="healthChecks"
						:health-status-label-visible-ms="navbar.healthStatusLabelVisibleMs"
						:current-language="navbar.currentLanguage.value"
						:current-theme-family="currentThemeFamily"
						:current-theme-mode="currentThemeMode"
						:language-options="navbar.languageOptions"
						:navigation-levels="navigationLevels"
						:notifications="notifications"
						:notifications-count="notificationsCount"
						:theme-family-options="navbar.themeFamilyOptions"
						:theme-mode-options="navbar.themeModeOptions"
						@navigation-select="selectNavigationItem"
						@language-change="navbar.selectLanguage"
						@notification-dismiss="navbar.dismissNotification"
						@notification-select="navbar.selectNotification"
						@notifications-clear="navbar.clearNotifications"
						@theme-family-change="navbar.selectThemeFamily"
						@theme-mode-change="navbar.selectThemeMode"
					/>
				</template>

				<template v-if="selectedRouteId === 'communications-all'">
					<CanonicalCommunicationsRoute
						:can-manage-saved-searches="communicationsSavedSearchAvailable"
						:can-read-sender-insights="communicationsSenderInsightsAvailable"
						@canonical-attachment-selected="currentAttachmentAnchorId = $event"
						@canonical-message-selected="currentCanonicalMessageId = $event"
					/>
					<AttachmentPreviewWorkflow
						:can-preview="attachmentPreviewAvailable"
						:can-replay-evidence="attachmentPreviewEvidenceReplayAvailable"
						:candidate-attachment-anchor-id="currentAttachmentAnchorId"
					/>
					<CommunicationsEvidenceExportWorkflow
						:can-export="communicationsExportAvailable"
						:candidate-message-id="currentCanonicalMessageId"
					/>
					<CallTranscriptionWorkflow :can-transcribe="callTranscriptionAvailable" />
				</template>
				<MailOperationalRoute
					v-else-if="selectedRouteId === 'communications-mail'"
					:body-content-status="mailMessageContent.model.value.status"
					:body-content-status-message="mailMessageContent.model.value.statusMessage"
					:body-text="mailMessageContent.model.value.bodyText"
					:body-format="mailMessageContent.model.value.bodyFormat"
					:can-compose="mailCompositionCommandAvailable"
					:can-compose-query="mailCompositionQueryAvailable"
					:can-deliver="mailDeliveryAvailable"
					:can-mutate-flags="mailMessageFlagCommandAvailable"
					:can-mutate-location="mailMessageLocationCommandAvailable"
					:can-query="mailOperationalQueryAvailable"
					:can-query-accounts="mailAccountCatalogAvailable"
					:can-query-flag-status="mailMessageFlagQueryAvailable"
					:can-query-location-status="mailMessageLocationQueryAvailable"
					:can-permanently-delete="mailMessagePermanentDeleteCommandAvailable"
					:can-query-permanent-delete-status="mailMessagePermanentDeleteQueryAvailable"
					:can-sync="mailSyncAvailable"
					:can-sync-health="mailSyncHealthAvailable"
					:modules="bootstrap.modules"
					:navigation-account-id="requestedMailAccountId"
					@account-navigation-change="mailAccountNavigation = $event"
					@message-evidence-change="openMailMessageContent"
				/>
				<TelegramOperationalRoute
					v-else-if="selectedRouteId === 'communications-telegram'"
					:can-authorize="telegramAuthorizationAvailable"
					:can-manage-lifecycle="telegramLifecycleAvailable"
					:can-query="telegramQueryAvailable"
					:can-reconfigure="telegramReconfigurationAvailable"
					:can-send="telegramCommandAvailable"
					:navigation-account-id="requestedTelegramAccountId"
					@account-navigation-change="telegramAccountNavigation = $event"
				/>
				<WhatsAppOperationalRoute
					v-else-if="selectedRouteId === 'communications-whatsapp'"
					:can-query="whatsAppOperationalQueryAvailable"
					:can-replay="whatsAppOperationalRealtimeAvailable"
					:can-send="whatsAppCommandAvailable"
					:modules="bootstrap.modules"
				/>
				<ZulipOperationalRoute
					v-else-if="selectedRouteId === 'communications-zulip'"
					:can-command="zulipCommandAvailable"
					:can-query="zulipOperationalQueryAvailable"
					:can-replay="zulipOperationalRealtimeAvailable"
					:modules="bootstrap.modules"
				/>
				<TasksWorkspaceView v-else-if="selectedRouteId === 'tasks'" />
				<KnowledgeWorkspaceView v-else-if="selectedRouteId === 'knowledge'" />
				<CalendarWorkspaceView v-else-if="selectedRouteId === 'calendar'" />
				<OrganizationsWorkspaceView v-else-if="selectedRouteId === 'organizations'" />
				<ProjectsWorkspaceView v-else-if="selectedRouteId === 'projects'" />
				<ObligationsWorkspaceView v-else-if="selectedRouteId === 'obligations'" />
				<DecisionsWorkspaceView v-else-if="selectedRouteId === 'decisions'" />
				<DocumentsWorkspaceView v-else-if="selectedRouteId === 'documents'" />
				<AppSettingsPage
					v-else-if="selectedTopLevelRouteId === 'settings'"
					:bootstrap="bootstrap"
					:route-downgrade-reason="routeDowngradeReason"
					:development-profile="props.gatewayAccessMode === BrowserGatewayAccessModeV1.LAN_DEVELOPMENT
						? 'private-lan'
						: props.gatewayAccessMode === BrowserGatewayAccessModeV1.LOCAL_DEVELOPMENT
							? 'loopback-full-stack'
							: 'disabled'"
					:current-language="navbar.currentLanguage.value"
					:language-options="navbar.languageOptions"
					:compiled-adapter-ids="compiledClientSurfaceAdapterIds"
					@language-change="navbar.selectLanguage"
					@refresh-request="navbar.refreshBootstrap(true)"
				/>
			</AppLayout>
		</Toast>
	</section>
</template>
