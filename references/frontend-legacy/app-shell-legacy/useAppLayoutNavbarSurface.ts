// Historical pre-clean-room navbar composition. Not part of the active client graph.
import { formatDistanceToNow } from 'date-fns'
import { computed, getCurrentInstance, onBeforeUnmount, onMounted, ref } from 'vue'
import { getActivePinia } from 'pinia'
import {
  isUiThemeFamily,
  isUiThemeMode,
  themeSelectionToName,
  type UiThemeFamily,
  type UiThemeMode,
  type UiThemeName,
} from '../../shared/ui/foundation/theme'
import type { CommunicationSubSurfaceId } from '../../domains/communications/queries/communicationChannelSurface'
import { useCommunicationsWorkspaceSurface } from '../../domains/communications/queries/useCommunicationsWorkspaceSurface'
import { fetchProviderAccounts } from '../../domains/settings/api/settings'
import { fetchTelegramAccounts } from '../../integrations/telegram/api/telegram'
import { fetchWhatsappAccounts } from '../../integrations/whatsapp/api/whatsapp'
import { ApiClient } from '../../platform/api/ApiClient'
import { fetchApplicationSettings } from '../../platform/settings/applicationSettingsClient'
import { fetchMailSyncStatus } from '../../shared/mailSync/syncApi'
import {
  useNotificationsStore,
  type NotificationItem,
} from '../../shared/stores/notifications'
import { useRealtimeStatusStore } from '../../shared/stores/realtimeStatus'
import {
  backendErrorHealthCheck,
  backendReadinessHealthChecks,
  frontendRuntimeHealthChecks,
  healthChecksNeedRecovery,
  integrationAccountHealthChecks,
  mailSyncErrorHealthCheck,
  mailSyncStatusHealthChecks,
  DEFAULT_CONSECUTIVE_PROVIDER_FAILURES_BEFORE_DEGRADED,
  type AppLayoutNavbarHealthCheck,
  type BackendReadinessResponse,
} from './appLayoutHealthChecks'
import {
  communicationAccountRouteNodes,
  communicationRouteIconTone,
  emptyAccountNavigation,
  isCommunicationChannelRouteId,
  isCommunicationsNavbarSurfaceId,
  isVisibleMailAccount,
  isVisibleTelegramAccount,
  isVisibleWhatsappAccount,
  type AppLayoutNavbarAccountNavigationState,
  type AppLayoutNavbarCommunicationSurfaceId,
  type AppLayoutNavbarNavigationIconTone,
  type AppLayoutNavbarRouteNode,
} from './appLayoutAccountNavigation'

export type {
  AppLayoutNavbarHealthCheck,
  AppLayoutNavbarHealthStatus,
} from './appLayoutHealthChecks'

export type AppLayoutNavbarBreadcrumb = {
  id: string
  label: string
}

export type AppLayoutNavbarNavigationItem = {
  id: string
  label: string
  icon?: string
  iconTone?: AppLayoutNavbarNavigationIconTone
}

export type AppLayoutNavbarNavigationLevel = {
  id: string
  label: string
  currentItem: AppLayoutNavbarNavigationItem
  items: readonly AppLayoutNavbarNavigationItem[]
}

export type AppLayoutNavbarToggleOption = {
  value: string
  label: string
}

export type AppLayoutNavbarThemeFamilyOption = {
  value: UiThemeFamily
  label: string
}

export type AppLayoutNavbarThemeModeOption = {
  value: UiThemeMode
  label: string
}

export type AppLayoutNavbarNotification = {
  id: string
  title: string
  body?: string
  sourceLabel: string
  timeLabel: string
  icon: string
  tone: 'info' | 'success' | 'warning' | 'danger'
  targetView?: string
  targetId?: string
}

export type AppLayoutNotificationToast = {
  id: string
  title: string
  description?: string
  variant: 'info' | 'success' | 'warning' | 'error'
}

const HEALTH_OK_REFRESH_MS = 30_000
const HEALTH_RECOVERY_REFRESH_MS = 5_000
const ACCOUNT_REFRESH_MS = 60_000
const ACCOUNT_RECOVERY_REFRESH_MS = 15_000

export function useAppLayoutNavbarSurface() {
  const currentThemeFamily = ref<UiThemeFamily>('base')
  const currentThemeMode = ref<UiThemeMode>('light')
  const selectedRouteId = ref('communications-mail')
  const communicationsSurface = useCommunicationsWorkspaceSurface()
  const accountNavigation = ref<AppLayoutNavbarAccountNavigationState>(
    emptyAccountNavigation()
  )
  const accountNavigationError = ref('')
  const backendHealthChecks = ref<readonly AppLayoutNavbarHealthCheck[]>([])
  const mailSyncHealthChecks = ref<readonly AppLayoutNavbarHealthCheck[]>([])
  const realtimeStatusStore = getActivePinia() ? useRealtimeStatusStore() : null
  const notificationsStore = getActivePinia() ? useNotificationsStore() : null
  const healthStatusLabelVisibleMs = 5000
  const notificationToastVisibleMs = 5000
  let healthRefreshTimer: number | undefined
  let accountRefreshTimer: number | undefined
  let pollingActive = false
  const healthChecks = computed<readonly AppLayoutNavbarHealthCheck[]>(() => [
    ...frontendRuntimeHealthChecks(realtimeStatusStore?.status ?? null),
    ...backendHealthChecks.value,
    ...mailSyncHealthChecks.value,
    ...integrationAccountHealthChecks(
      accountNavigation.value,
      accountNavigationError.value
    ),
  ])
  const currentTheme = computed<UiThemeName>(() => {
    return themeSelectionToName(
      currentThemeFamily.value,
      currentThemeMode.value
    )
  })

  const routeTree = computed<readonly AppLayoutNavbarRouteNode[]>(() => [
    {
      id: 'dashboard',
      label: 'Dashboard',
      icon: 'tabler:layout-dashboard',
      iconTone: 'dashboard',
    },
    {
      id: 'communications',
      label: 'Communications',
      icon: 'tabler:messages',
      iconTone: 'communication',
      children: communicationsSurface.childSurfaces.flatMap((surface) => {
        if (!isCommunicationsNavbarSurfaceId(surface.id)) return []

        return [
          communicationRouteNode(
            surface.id,
            surface.labelKey,
            accountNavigation.value
          ),
        ]
      }),
    },
    {
      id: 'review',
      label: 'Review',
      icon: 'tabler:clipboard-check',
      iconTone: 'review',
    },
    {
      id: 'personas',
      label: 'Personas',
      icon: 'tabler:user-circle',
      iconTone: 'knowledge',
    },
    {
      id: 'knowledge',
      label: 'Knowledge',
      icon: 'tabler:share',
      iconTone: 'knowledge',
    },
    { id: 'tasks', label: 'Tasks', icon: 'tabler:checkbox', iconTone: 'tasks' },
    {
      id: 'calendar',
      label: 'Calendar',
      icon: 'tabler:calendar',
      iconTone: 'calendar',
    },
    {
      id: 'documents',
      label: 'Documents',
      icon: 'tabler:file-text',
      iconTone: 'documents',
    },
    {
      id: 'settings',
      label: 'Settings',
      icon: 'tabler:settings',
      iconTone: 'settings',
    },
  ])

  const selectedRoutePath = computed(() => {
    const path = findRoutePath(routeTree.value, selectedRouteId.value)
    if (!path) return [routeTree.value[0]]

    const leaf = path.at(-1)
    if (leaf && isCommunicationChannelRouteId(leaf.id) && leaf.children?.length) {
      return [...path, leaf.children[0]]
    }

    return path
  })

  const selectedTopLevelRouteId = computed(
    () => selectedRoutePath.value[0]?.id ?? routeTree.value[0].id
  )

  const breadcrumbs = computed<readonly AppLayoutNavbarBreadcrumb[]>(() => {
    return selectedRoutePath.value.map((item) => ({
      id: item.id,
      label: item.label,
    }))
  })

  const navigationLevels = computed<readonly AppLayoutNavbarNavigationLevel[]>(
    () => {
      return selectedRoutePath.value.map((node, index, path) => {
        const parentNode = index === 0 ? undefined : path[index - 1]
        const siblings = parentNode?.children ?? routeTree.value

        return {
          id: `navigation-level-${index}`,
          label: navigationLevelLabel(index),
          currentItem: toNavigationItem(node),
          items: siblings.map(toNavigationItem),
        }
      })
    }
  )

  const languageOptions: AppLayoutNavbarToggleOption[] = [
    { value: 'ru', label: 'Русский' },
    { value: 'en', label: 'English' },
  ]

  const themeFamilyOptions: AppLayoutNavbarThemeFamilyOption[] = [
    { value: 'base', label: 'Base' },
    { value: 'makosh', label: 'Макошь' },
  ]

  const themeModeOptions: AppLayoutNavbarThemeModeOption[] = [
    { value: 'light', label: 'Light' },
    { value: 'dark', label: 'Dark' },
  ]

  const notifications = computed<readonly AppLayoutNavbarNotification[]>(() =>
    (notificationsStore?.notificationItems ?? []).map(navbarNotification)
  )

  const notificationToasts = computed<AppLayoutNotificationToast[]>(() => {
    return notifications.value.map((notification) => ({
      id: notification.id,
      title: notification.title,
      description: notification.body,
      variant: notificationToneToToastVariant(notification.tone),
    }))
  })
  const notificationsCount = computed(() => notifications.value.length)

  void refreshAccountNavigation()
  void refreshHealthChecks()
  if (getCurrentInstance()) {
    onMounted(() => {
      pollingActive = true
      scheduleHealthRefresh(nextHealthRefreshDelay())
      scheduleAccountRefresh(nextAccountRefreshDelay())
    })
    onBeforeUnmount(() => {
      pollingActive = false
      clearHealthRefresh()
      clearAccountRefresh()
    })
  }

  function selectThemeFamily(value: string): void {
    if (!isUiThemeFamily(value)) return

    currentThemeFamily.value = value
  }

  function selectThemeMode(value: string): void {
    if (!isUiThemeMode(value)) return

    currentThemeMode.value = value
  }

  function selectNavigationItem(itemId: string): void {
    const path = findRoutePath(routeTree.value, normalizeLegacyRouteId(itemId))
    const selectedNode = path?.at(-1)
    if (!selectedNode) return

    selectedRouteId.value = defaultRouteLeaf(selectedNode).id
  }

  function selectReturnRouteFromSearch(search: string): void {
    const routeId = oauthReturnRouteId(search)
    if (!routeId) return

    selectNavigationItem(routeId)
  }

  function selectNotification(notificationId: string): void {
    const notification = notificationsStore?.notificationItems.find(
      (item) => item.id === notificationId
    )
    if (!notification) return
    if (!notification.targetView && !notification.targetId) return

    if (
      notification.targetView &&
      findRoutePath(routeTree.value, normalizeLegacyRouteId(notification.targetView))
    ) {
      selectedRouteId.value = normalizeLegacyRouteId(notification.targetView)
    }

    notificationsStore?.openNotificationTarget(notification)
  }

  function dismissNotification(notificationId: string): void {
    notificationsStore?.dismissNotification(notificationId)
  }

  function clearNotifications(): void {
    notificationsStore?.clearNotifications()
  }

  return {
    accountNavigationError,
    breadcrumbs,
    healthChecks,
    navigationLevels,
    selectedRouteId,
    selectedTopLevelRouteId,
    currentTheme,
    currentThemeFamily,
    currentThemeMode,
    currentLanguage: 'ru',
    languageOptions,
    notifications,
    notificationToasts,
    notificationToastVisibleMs,
    clearNotifications,
    dismissNotification,
    selectReturnRouteFromSearch,
    selectNavigationItem,
    selectNotification,
    selectThemeFamily,
    selectThemeMode,
    themeFamilyOptions,
    themeModeOptions,
    healthStatusLabelVisibleMs,
    notificationsCount,
  }

  async function refreshAccountNavigation(): Promise<void> {
    try {
      const [mailResponse, telegramResponse, whatsappResponse] =
        await Promise.all([
          fetchProviderAccounts(),
          fetchTelegramAccounts(),
          fetchWhatsappAccounts(false),
        ])

      accountNavigation.value = {
        mail: mailResponse.items.filter(isVisibleMailAccount),
        telegram: telegramResponse.items.filter(isVisibleTelegramAccount),
        whatsapp: whatsappResponse.items.filter(isVisibleWhatsappAccount),
      }
      accountNavigationError.value = ''
    } catch (error) {
      accountNavigation.value = emptyAccountNavigation()
      accountNavigationError.value =
        error instanceof Error
          ? error.message
          : 'Navigation account load failed'
    }
  }

  async function refreshHealthChecks(): Promise<void> {
    const [readinessResult, mailSyncResult, applicationSettingsResult] = await Promise.allSettled([
      fetchBackendReadiness(),
      fetchMailSyncStatusSafely(),
      fetchApplicationSettings(),
    ])
    const consecutiveFailuresBeforeDegraded =
      applicationSettingsResult.status === 'fulfilled'
        ? mailDegradationThreshold(applicationSettingsResult.value.items)
        : DEFAULT_CONSECUTIVE_PROVIDER_FAILURES_BEFORE_DEGRADED

    backendHealthChecks.value =
      readinessResult.status === 'fulfilled'
        ? backendReadinessHealthChecks(readinessResult.value)
        : [backendErrorHealthCheck(readinessResult.reason)]
    mailSyncHealthChecks.value =
      mailSyncResult.status === 'fulfilled'
        ? mailSyncStatusHealthChecks(mailSyncResult.value.items, consecutiveFailuresBeforeDegraded)
        : [mailSyncErrorHealthCheck(mailSyncResult.reason)]
  }

  async function fetchBackendReadiness(): Promise<BackendReadinessResponse> {
    return ApiClient.instance.get<BackendReadinessResponse>(
      '/readyz',
      'Backend readiness request failed'
    )
  }

  async function fetchMailSyncStatusSafely(): Promise<
    Awaited<ReturnType<typeof fetchMailSyncStatus>>
  > {
    return fetchMailSyncStatus()
  }

  function mailDegradationThreshold(
    settings: Awaited<ReturnType<typeof fetchApplicationSettings>>['items']
  ): number {
    const value = settings.find(
      (setting) => setting.setting_key === 'communications.mail.consecutive_failures_before_degraded'
    )?.value
    return typeof value === 'number' && Number.isInteger(value) && value >= 1
      ? value
      : DEFAULT_CONSECUTIVE_PROVIDER_FAILURES_BEFORE_DEGRADED
  }

  function scheduleHealthRefresh(delayMs: number): void {
    clearHealthRefresh()
    healthRefreshTimer = window.setTimeout(() => {
      healthRefreshTimer = undefined
      void refreshHealthChecks().finally(() => {
        if (pollingActive) scheduleHealthRefresh(nextHealthRefreshDelay())
      })
    }, delayMs)
  }

  function scheduleAccountRefresh(delayMs: number): void {
    clearAccountRefresh()
    accountRefreshTimer = window.setTimeout(() => {
      accountRefreshTimer = undefined
      void refreshAccountNavigation().finally(() => {
        if (pollingActive) scheduleAccountRefresh(nextAccountRefreshDelay())
      })
    }, delayMs)
  }

  function nextHealthRefreshDelay(): number {
    return healthChecksNeedRecovery(healthChecks.value)
      ? HEALTH_RECOVERY_REFRESH_MS
      : HEALTH_OK_REFRESH_MS
  }

  function nextAccountRefreshDelay(): number {
    return accountNavigationError.value
      ? ACCOUNT_RECOVERY_REFRESH_MS
      : ACCOUNT_REFRESH_MS
  }

  function clearHealthRefresh(): void {
    if (healthRefreshTimer === undefined) return
    window.clearTimeout(healthRefreshTimer)
    healthRefreshTimer = undefined
  }

  function clearAccountRefresh(): void {
    if (accountRefreshTimer === undefined) return
    window.clearTimeout(accountRefreshTimer)
    accountRefreshTimer = undefined
  }
}

function communicationRouteIcon(channelId: CommunicationSubSurfaceId): string {
  if (channelId === 'mail') return 'tabler:mail'
  if (channelId === 'telegram') return 'tabler:brand-telegram'
  if (channelId === 'whatsapp') return 'tabler:brand-whatsapp'

  return 'tabler:message-circle'
}

function communicationRouteNode(
  channelId: AppLayoutNavbarCommunicationSurfaceId,
  label: string,
  accountNavigation: AppLayoutNavbarAccountNavigationState
): AppLayoutNavbarRouteNode {
  const children = communicationAccountRouteNodes(channelId, accountNavigation)

  return {
    id: `communications-${channelId}`,
    label,
    icon: communicationRouteIcon(channelId),
    iconTone: communicationRouteIconTone(channelId),
    ...(children.length > 0 ? { children } : {}),
  }
}

function navigationLevelLabel(index: number): string {
  if (index === 0) return 'Main menu'
  if (index === 1) return 'Sub menu'
  return 'Elem'
}

function notificationToneToToastVariant(
  tone: AppLayoutNavbarNotification['tone']
): AppLayoutNotificationToast['variant'] {
  if (tone === 'danger') return 'error'

  return tone
}

function navbarNotification(
  notification: NotificationItem
): AppLayoutNavbarNotification {
  return {
    id: notification.id,
    title: notification.title,
    body: notification.body,
    sourceLabel: notification.sourceLabel ?? 'Макошь',
    timeLabel: notificationTimeLabel(notification.time),
    icon: notification.icon,
    tone: notification.tone ?? 'info',
    targetView: notification.targetView,
    targetId: notification.targetId,
  }
}

function notificationTimeLabel(time: Date): string {
  return formatDistanceToNow(time, { addSuffix: true })
}

function oauthReturnRouteId(search: string): string | null {
  const params = new URLSearchParams(search)
  return params.get('makosh_route')?.trim() || null
}

function normalizeLegacyRouteId(routeId: string): string {
  if (routeId === 'persons') return 'personas'

  return routeId
}

function toNavigationItem(
  node: AppLayoutNavbarRouteNode
): AppLayoutNavbarNavigationItem {
  return {
    id: node.id,
    label: node.label,
    icon: node.icon,
    iconTone: node.iconTone,
  }
}

function defaultRouteLeaf(
  node: AppLayoutNavbarRouteNode
): AppLayoutNavbarRouteNode {
  if (node.id === 'communications') {
    return (
      node.children?.find((child) => child.id === 'communications-mail') ?? node
    )
  }
  if (isCommunicationChannelRouteId(node.id)) return node.children?.[0] ?? node

  const firstChild = node.children?.[0]
  if (!firstChild) return node

  return defaultRouteLeaf(firstChild)
}

function findRoutePath(
  nodes: readonly AppLayoutNavbarRouteNode[],
  itemId: string,
  ancestors: readonly AppLayoutNavbarRouteNode[] = []
): AppLayoutNavbarRouteNode[] | undefined {
  for (const node of nodes) {
    const path = [...ancestors, node]
    if (node.id === itemId) {
      return path
    }

    const childPath = node.children
      ? findRoutePath(node.children, itemId, path)
      : undefined
    if (childPath) {
      return childPath
    }
  }

  return undefined
}
