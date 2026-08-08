// Historical pre-clean-room navbar test. Not part of the active test suite.
import { readFileSync } from 'node:fs'
import { describe, expect, it } from 'vitest'
import { createPinia, setActivePinia } from 'pinia'
import { useAppLayoutNavbarSurface } from '../queries/useAppLayoutNavbarSurface'
import { useNotificationsStore } from '../../shared/stores/notifications'

describe('AppNavbar boundary', () => {
  it('keeps layout navigation reusable and slot-based', () => {
    const source = readFileSync(
      new URL('../../shared/ui/shell/AppNavbar.vue', import.meta.url),
      'utf8'
    )
    const layoutCss = readFileSync(
      new URL('../../shared/ui/shell/app-layout.css', import.meta.url),
      'utf8'
    )

    expect(source).toContain('<slot name="page-actions"')
    expect(source).toContain('breadcrumbs?: readonly AppNavbarBreadcrumb[]')
    expect(source).toContain(
      'navigationLevels?: readonly AppNavbarNavigationLevel[]'
    )
    expect(source).toContain('healthChecks?: readonly AppNavbarHealthCheck[]')
    expect(source).toContain('languageOptions?: AppNavbarToggleOption[]')
    expect(source).toContain('notifications?: readonly AppNavbarNotification[]')
    expect(source).toContain('targetView?: string')
    expect(source).toContain('targetId?: string')
    expect(source).toContain(
      'themeFamilyOptions?: AppNavbarThemeFamilyOption[]'
    )
    expect(source).toContain('themeModeOptions?: AppNavbarThemeModeOption[]')
    expect(source).toContain('notificationsCount?: number')
    expect(source).toContain('healthStatusLabelVisibleMs?: number')
    expect(source).toContain('function handleBrandClick(): void')
    expect(source).toContain("emit('navigationSelect', 'dashboard')")
    expect(source).toContain('function handleSettingsClick(): void')
    expect(source).toContain("emit('navigationSelect', 'settings')")
    expect(source).toContain('aria-label="Dashboard"')
    expect(source).toContain('class="app-navbar__brand"')
    expect(source).toContain('@click="handleBrandClick"')
    expect(source).toContain("import { makoshBrandAssets } from '../assets/brand'")
    expect(source).toContain(':src="makoshBrandAssets.logoMarkLight"')
    expect(source).toContain(':src="makoshBrandAssets.logoMarkDark"')
    expect(source).toContain('app-navbar__brand-logo--light')
    expect(source).toContain('app-navbar__brand-logo--dark')
    expect(source.indexOf('class="app-navbar__brand"')).toBeLessThan(
      source.indexOf('<AppNavbarRouteBreadcrumbs')
    )
    expect(layoutCss).not.toContain('.app-navbar__brand:hover')
    expect(layoutCss).toContain('.app-navbar__brand:focus-visible')
    expect(layoutCss).toContain(
      "[data-ui-theme-mode='light'] .app-navbar__brand-logo--light"
    )
    expect(layoutCss).toContain(
      "[data-ui-theme-mode='dark'] .app-navbar__brand-logo--light"
    )
    expect(layoutCss).toContain(
      "[data-ui-theme-mode='dark'] .app-navbar__brand-logo--dark"
    )
    expect(layoutCss).toContain('.app-navbar__brand-logo--dark')
    expect(layoutCss).toContain('.app-navbar__health-button--label-visible')
    expect(layoutCss).toContain('max-width: 0')
    expect(layoutCss).toContain('max-width var(--app-navbar-health-motion)')
    expect(layoutCss).toContain('.app-navbar__notifications-clear')
    expect(layoutCss).toContain('.app-navbar__notification-main')
    expect(layoutCss).toContain('.app-navbar__notification-dismiss')
    expect(layoutCss).toContain('.app-navbar__notification--clickable')
    expect(source).toContain("import Drawer from '../Drawer.vue'")
    expect(source).toContain(
      "import AppNavbarRouteBreadcrumbs from './AppNavbarRouteBreadcrumbs.vue'"
    )
    expect(source).toContain(
      "import { useMouseLeaveDismiss } from '../useMouseLeaveDismiss'"
    )
    expect(source).not.toContain('useNavigationStore')
    expect(source).not.toContain('useNotificationsStore')
    expect(source).not.toContain('useRealtimeStatusStore')
    expect(source).not.toContain('useTopbarSurface')
  })

  it('exposes the requested system and user menu controls', () => {
    const source = readFileSync(
      new URL('../../shared/ui/shell/AppNavbar.vue', import.meta.url),
      'utf8'
    )

    expect(source).toContain('System health checks')
    expect(source).toContain('<AppNavbarRouteBreadcrumbs')
    expect(source).toContain('@select="handleNavigationSelect"')
    expect(source).toContain('tabler:heartbeat')
    expect(source).toContain('tabler:bell')
    expect(source).toContain('title="Уведомления"')
    expect(source).toContain('side="right"')
    expect(source).toContain('Список уведомлений')
    expect(source).toContain('visibleNotifications')
    expect(source).toContain("notificationSelect: [notificationId: string]")
    expect(source).toContain("notificationDismiss: [notificationId: string]")
    expect(source).toContain('notificationsClear: []')
    expect(source).toContain('function isNotificationClickable')
    expect(source).toContain('app-navbar__notifications-header')
    expect(source).toContain('Очистить все')
    expect(source).toContain("emit('notificationsClear')")
    expect(source).toContain("emit('notificationSelect', notification.id)")
    expect(source).toContain("emit('notificationDismiss', notification.id)")
    expect(source).toContain('app-navbar__notification-main')
    expect(source).toContain('app-navbar__notification-dismiss')
    expect(source).toContain(
      "import ToggleGroup from '../ToggleGroup.vue'"
    )
    expect(source).toContain('makosh-toggle-group--tabs')
    expect(source).toContain('Смена языка')
    expect(source).toContain('Смена темы')
    expect(source).toContain('Семейство темы')
    expect(source).toContain('Вариант темы')
    expect(source).toContain('aria-haspopup="listbox"')
    expect(source).toContain('role="option"')
    expect(source).toContain('@mouseleave="scheduleUserMouseLeaveDismiss"')
    expect(source).toContain('@mouseleave="scheduleThemeMouseLeaveDismiss"')
    expect(source).toContain('@mouseenter="handleHealthPointerEnter"')
    expect(source).toContain('@mouseleave="handleHealthPointerLeave"')
    expect(source).toContain('watch(systemStatus')
    expect(source).toContain('revealHealthLabelTemporarily')
    expect(source).toContain('isHealthLabelVisible')
    expect(source).toContain('app-navbar__health-button--label-visible')
    expect(source).toContain('healthStatusLabelVisibleMs: 5000')
    expect(source).toContain(':aria-hidden="!isHealthLabelVisible"')
    expect(source).toContain("emit('themeFamilyChange'")
    expect(source).toContain("emit('themeModeChange'")
    expect(source).toContain('tabler:settings')
    expect(source).toContain('Настройки')
    expect(source).toContain('@click="handleSettingsClick"')
    expect(source).not.toContain(':href="settingsHref"')
    expect(source).toContain('Выход')
  })

  it('locks the app layout to the viewport and keeps scrolling inside panes', () => {
    const layoutCss = readFileSync(
      new URL('../../shared/ui/shell/app-layout.css', import.meta.url),
      'utf8'
    )

    expect(layoutCss).toContain('html,\nbody,\n#app')
    expect(layoutCss).toContain('overflow: hidden')
    expect(layoutCss).toContain('overscroll-behavior: none')
    expect(layoutCss).toContain('.app-layout-root')
    expect(layoutCss).toContain('position: relative')
    expect(layoutCss).toContain('display: block')
    expect(layoutCss).toContain('width: 100dvw')
    expect(layoutCss).toContain('height: 100dvh')
    expect(layoutCss).toContain('max-height: 100%')
    expect(layoutCss).toContain('.makosh-app-layout__main')
    expect(layoutCss).toContain('overflow: auto')
    expect(layoutCss).toContain(
      '.makosh-app-layout__rail,\n.makosh-app-layout__sidebar,\n.makosh-app-layout__inspector'
    )
  })

  it('does not reserve a rail column at tablet widths when no rail slot exists', () => {
    const layoutCss = readFileSync(
      new URL('../../shared/ui/shell/app-layout.css', import.meta.url),
      'utf8'
    )
    const tabletStyles = layoutCss.slice(
      layoutCss.indexOf('@media (max-width: 1100px)'),
      layoutCss.indexOf('@media (max-width: 700px)')
    )

    expect(tabletStyles).toContain("grid-template-areas: 'workspace'")
    expect(tabletStyles).toContain('.makosh-app-layout--has-rail')
    expect(tabletStyles).toContain("grid-template-areas: 'rail workspace'")
  })

  it('mounts the navbar in the main AppLayout root', () => {
    const root = readFileSync(
      new URL('./AppLayoutRoot.vue', import.meta.url),
      'utf8'
    )
    const app = readFileSync(new URL('../App.vue', import.meta.url), 'utf8')
    const main = readFileSync(new URL('../../main.ts', import.meta.url), 'utf8')
    const appLayoutHtml = readFileSync(
      new URL('../../../app-layout.html', import.meta.url),
      'utf8'
    )
    const surface = readFileSync(
      new URL('../queries/useAppLayoutNavbarSurface.ts', import.meta.url),
      'utf8'
    )
    const clientNavigation = readFileSync(
      new URL('../queries/useClientNavigationSurface.ts', import.meta.url),
      'utf8'
    )
    const healthSurface = readFileSync(
      new URL('../queries/appLayoutHealthChecks.ts', import.meta.url),
      'utf8'
    )
    const accountNavigation = readFileSync(
      new URL('../queries/appLayoutAccountNavigation.ts', import.meta.url),
      'utf8'
    )
    const sidebarStore = readFileSync(
      new URL('../../shared/stores/sidebar.ts', import.meta.url),
      'utf8'
    )
    const layoutEditorStore = readFileSync(
      new URL('../../shared/stores/layoutEditor.ts', import.meta.url),
      'utf8'
    )

    expect(app).toContain(
      "import AppLayoutRoot from './layout/AppLayoutRoot.vue'"
    )
    expect(app).toContain('<AppLayoutRoot v-if="authenticated" />')
    expect(app).toContain('BrowserGatewayAccessGate')
    expect(main).not.toContain("import router from './app/router'")
    expect(main).not.toContain('app.use(router)')
    expect(main).toContain("import './shared/ui/shell/app-layout.css'")
    expect(appLayoutHtml).toContain('<div id="app"></div>')
    expect(appLayoutHtml).toContain('src="/src/main.ts"')
    expect(root).toContain(
      "import { useClientNavigationSurface } from '../queries/useClientNavigationSurface'"
    )
    expect(root).toContain("import Toast from '../../shared/ui/Toast.vue'")
    expect(root).not.toContain('CommunicationsWorkspaceView')
    expect(root).not.toContain('PersonasWorkspaceView')
    expect(root).toContain("import AppNavbar from '../../shared/ui/shell/AppNavbar.vue'")
    expect(root).toContain('const navbar = useClientNavigationSurface()')
    expect(clientNavigation).toContain("currentThemeFamily = ref<UiThemeFamily>('makosh')")
    expect(clientNavigation).toContain("currentThemeMode = ref<UiThemeMode>('dark')")
    expect(root).toContain('const breadcrumbs = navbar.breadcrumbs')
    expect(root).toContain('const currentTheme = navbar.currentTheme')
    expect(root).toContain(
      'const currentThemeFamily = navbar.currentThemeFamily'
    )
    expect(root).toContain('const currentThemeMode = navbar.currentThemeMode')
    expect(root).toContain('const navigationLevels = navbar.navigationLevels')
    expect(root).toContain(
      'const notificationToasts = navbar.notificationToasts'
    )
    expect(root).toContain('const selectedRouteId = navbar.selectedRouteId')
    expect(root).toContain(
      'const selectedTopLevelRouteId = navbar.selectedTopLevelRouteId'
    )
    expect(root).toContain(':data-ui-theme="currentTheme"')
    expect(root).toContain(':data-ui-theme-family="currentThemeFamily"')
    expect(root).toContain(':data-ui-theme-mode="currentThemeMode"')
    expect(root).toContain(
      "document.documentElement.setAttribute('data-ui-theme', theme)"
    )
    expect(root).toContain(
      "document.documentElement.setAttribute('data-ui-theme-family', family)"
    )
    expect(root).toContain(
      "document.documentElement.setAttribute('data-ui-theme-mode', mode)"
    )
    expect(root).toContain('<Toast')
    expect(root).toContain('class="app-layout-notification-toasts"')
    expect(root).toContain('close-label="Закрыть уведомление"')
    expect(root).toContain(':default-toasts="notificationToasts"')
    expect(root).toContain(':duration="navbar.notificationToastVisibleMs"')
    expect(root).toContain('<template #topbar>')
    expect(root).toContain(':breadcrumbs="breadcrumbs"')
    expect(root).toContain('const healthChecks = navbar.healthChecks')
    expect(root).toContain(':health-checks="healthChecks"')
    expect(root).toContain(
      ':health-status-label-visible-ms="navbar.healthStatusLabelVisibleMs"'
    )
    expect(root).toContain(':language-options="navbar.languageOptions"')
    expect(root).toContain(':navigation-levels="navigationLevels"')
    expect(root).toContain('const notifications = navbar.notifications')
    expect(root).toContain('const notificationsCount = navbar.notificationsCount')
    expect(root).toContain(':notifications="notifications"')
    expect(root).toContain(':theme-family-options="navbar.themeFamilyOptions"')
    expect(root).toContain(':theme-mode-options="navbar.themeModeOptions"')
    expect(root).toContain('@theme-family-change="navbar.selectThemeFamily"')
    expect(root).toContain('@theme-mode-change="navbar.selectThemeMode"')
    expect(root).toContain(':notifications-count="notificationsCount"')
    expect(root).toContain('@navigation-select="navbar.selectNavigationItem"')
    expect(root).toContain('@notification-dismiss="navbar.dismissNotification"')
    expect(root).toContain('@notification-select="navbar.selectNotification"')
    expect(root).toContain('@notifications-clear="navbar.clearNotifications"')
    expect(root).not.toContain('CommunicationsWorkspaceView')
    expect(root).not.toContain('PersonasWorkspaceView')
    expect(root).toContain('<SystemControlPage')
    expect(sidebarStore).toContain("{ id: 'personas', label: 'Personas'")
    expect(sidebarStore).not.toContain("{ id: 'persons'")
    expect(sidebarStore).toContain("if (itemId === 'persons') return 'personas'")
    expect(sidebarStore).toContain("if (rootId === 'persons') return 'personas'")
    expect(layoutEditorStore).toContain("viewScope: ['personas']")
    expect(layoutEditorStore).not.toContain("viewScope: ['persons']")
    expect(layoutEditorStore).toContain("if (viewId === 'persons') return 'personas'")
    expect(root).not.toContain('@route-select="navbar.selectNavigationItem"')
    expect(surface).toContain('Main menu')
    expect(surface).toContain('Sub menu')
    expect(surface).toContain('Elem')
    expect(surface).toContain('routeTree')
    expect(surface).toContain('useCommunicationsWorkspaceSurface')
    expect(surface).toContain('./appLayoutAccountNavigation')
    expect(accountNavigation).toContain('communicationsNavbarSurfaceIds')
    expect(surface).toContain('Communications')
    expect(surface).toContain("selectedRouteId = ref('communications-mail')")
    expect(surface).toContain('selectedTopLevelRouteId')
    expect(surface).not.toContain('communications-workspace')
    expect(surface).not.toContain('Surface map')
    expect(surface).toContain('fetchProviderAccounts')
    expect(surface).toContain('fetchTelegramAccounts')
    expect(surface).toContain('fetchWhatsappAccounts')
    expect(surface).toContain('accountNavigation')
    expect(surface).toContain('communicationRouteNode')
    expect(surface).toContain('id: `communications-${channelId}`')
    expect(surface).not.toContain('telegram-all-accounts')
    expect(surface).not.toContain('mail-all-accounts')
    expect(surface).not.toContain('whatsapp-all-accounts')
    expect(surface).not.toContain('All accounts')
    expect(accountNavigation).toContain('Все ящики')
    expect(accountNavigation).toContain('Все аккаунты')
    expect(accountNavigation).toContain('isMailProviderKind(account.provider_kind)')
    expect(accountNavigation).toContain("if (channelId === 'telegram') return 'telegram'")
    expect(accountNavigation).toContain("if (channelId === 'whatsapp') return 'whatsapp'")
    expect(accountNavigation).toContain("if (channelId === 'mail') return 'mail'")
    expect(surface).toContain("iconTone: 'communication'")
    expect(surface).not.toContain('Telegram account 1')
    expect(surface).not.toContain('Mail account 1')
    expect(surface).toContain('selectNavigationItem')
    expect(surface).toContain("if (node.id === 'communications')")
    expect(surface).toContain("child.id === 'communications-mail'")
    expect(surface).toContain('isCommunicationChannelRouteId')
    expect(surface).toContain('Русский')
    expect(surface).toContain('English')
    expect(surface).toContain('currentTheme')
    expect(surface).toContain("currentThemeFamily = ref<UiThemeFamily>('base')")
    expect(surface).toContain("currentThemeMode = ref<UiThemeMode>('light')")
    expect(surface).toContain('themeSelectionToName(')
    expect(surface).toContain('currentThemeFamily.value')
    expect(surface).toContain('currentThemeMode.value')
    expect(surface).toContain('themeFamilyOptions')
    expect(surface).toContain('themeModeOptions')
    expect(surface).toContain('Light')
    expect(surface).toContain('Dark')
    expect(surface).toContain('Макошь')
    expect(surface).toContain('healthStatusLabelVisibleMs = 5000')
    expect(surface).toContain('healthStatusLabelVisibleMs')
    expect(surface).toContain('HEALTH_RECOVERY_REFRESH_MS = 5_000')
    expect(surface).toContain('scheduleHealthRefresh')
    expect(surface).toContain('scheduleAccountRefresh')
    expect(surface).toContain('fetchMailSyncStatus')
    expect(surface).toContain('useRealtimeStatusStore')
    expect(surface).toContain('getActivePinia')
    expect(healthSurface).toContain('frontendRuntimeHealthChecks')
    expect(healthSurface).toContain('mailSyncStatusHealthChecks')
    expect(healthSurface).toContain('integrationAccountHealthChecks')
    expect(healthSurface).toContain('Backend API')
    expect(healthSurface).toContain('Frontend realtime')
    expect(healthSurface).toContain('Mail sync:')
    expect(surface).toContain('notificationToasts')
    expect(surface).toContain('notificationToastVisibleMs = 5000')
    expect(surface).toContain('notificationToneToToastVariant')
    expect(surface).toContain('useNotificationsStore')
    expect(surface).toContain('navbarNotification')
    expect(surface).toContain('selectNotification')
    expect(surface).toContain('dismissNotification')
    expect(surface).toContain('clearNotifications')
    expect(surface).toContain('openNotificationTarget(notification)')
    expect(surface).toContain('targetView: notification.targetView')
    expect(surface).toContain('targetId: notification.targetId')
    expect(surface).not.toContain('Vault требует разблокировки')
    expect(surface).toContain('Review')
    expect(surface).toContain('Personas')
    expect(surface).toContain("id: 'personas'")
    expect(surface).toContain("id: 'settings'")
    expect(surface).toContain('notificationsCount')
  })

  it('models communication aggregate pages as selectable route leaves', () => {
    const surface = useAppLayoutNavbarSurface()

    expect(currentNavigationPath(surface)).toEqual(['Communications', 'Mail'])
    expect(levelOptions(surface, 'Sub menu')).toEqual([
      'Mail',
      'Telegram',
      'WhatsApp',
    ])
    expect(levelOptions(surface, 'Elem')).toEqual([])

    surface.selectNavigationItem('communications')
    expect(currentNavigationPath(surface)).toEqual(['Communications', 'Mail'])

    surface.selectNavigationItem('communications-mail')
    expect(currentNavigationPath(surface)).toEqual(['Communications', 'Mail'])
    expect(levelOptions(surface, 'Elem')).toEqual([])

    surface.selectNavigationItem('communications-whatsapp')
    expect(currentNavigationPath(surface)).toEqual([
      'Communications',
      'WhatsApp',
    ])
    expect(levelOptions(surface, 'Elem')).toEqual([])
  })

  it('accepts OAuth callback return routes for same-tab browser fallback redirects', () => {
    const surface = useAppLayoutNavbarSurface()

    surface.selectReturnRouteFromSearch(
      '?makosh_route=settings&makosh_oauth=gmail_connected'
    )

    expect(surface.selectedRouteId.value).toBe('settings')
    expect(currentNavigationPath(surface)).toEqual(['Settings'])
  })

  it('prepares app notifications for the toast host', () => {
    const surface = useAppLayoutNavbarSurface()

    expect(surface.notificationToastVisibleMs).toBe(5000)
    expect(surface.notificationsCount.value).toBe(0)
    expect(surface.notificationToasts.value).toEqual([])
  })

  it('routes clickable notifications to their target and clears notification state', () => {
    setActivePinia(createPinia())
    const surface = useAppLayoutNavbarSurface()
    const notificationsStore = useNotificationsStore()

    notificationsStore.addNotification({
      id: 'mail-translation-ready',
      title: 'Translation ready',
      body: 'Translated to ru',
      icon: 'tabler:language',
      tone: 'success',
      sourceLabel: 'Mail',
      time: new Date('2026-07-06T00:49:52.000Z'),
      targetView: 'communications-mail',
      targetId: 'message-1',
    })

    surface.selectNavigationItem('communications-telegram')
    surface.selectNotification('mail-translation-ready')

    expect(surface.selectedRouteId.value).toBe('communications-mail')
    expect(notificationsStore.pendingNotificationTarget?.targetId).toBe(
      'message-1'
    )
    expect(surface.notifications.value[0]).toMatchObject({
      targetView: 'communications-mail',
      targetId: 'message-1',
    })

    surface.dismissNotification('mail-translation-ready')
    expect(surface.notificationsCount.value).toBe(0)

    notificationsStore.addNotification({
      id: 'mail-action-completed',
      title: 'Mail action completed',
      icon: 'tabler:check',
      tone: 'success',
      sourceLabel: 'Mail',
      time: new Date('2026-07-06T00:50:52.000Z'),
    })
    surface.clearNotifications()
    expect(notificationsStore.rawNotificationItems).toEqual([])
    expect(notificationsStore.pendingNotificationTarget).toBeNull()
  })

  it('maps navbar theme family and mode to explicit runtime ids', () => {
    const surface = useAppLayoutNavbarSurface()

    expect(surface.currentTheme.value).toBe('base-light')
    expect(surface.themeFamilyOptions.map((option) => option.value)).toEqual([
      'base',
      'makosh',
    ])
    expect(surface.themeModeOptions.map((option) => option.value)).toEqual([
      'light',
      'dark',
    ])

    surface.selectThemeMode('dark')
    expect(surface.currentTheme.value).toBe('base-dark')

    surface.selectThemeFamily('makosh')
    expect(surface.currentTheme.value).toBe('makosh-dark')

    surface.selectThemeMode('light')
    expect(surface.currentTheme.value).toBe('makosh-light')

    surface.selectThemeFamily('base')
    expect(surface.currentTheme.value).toBe('base-light')
    surface.selectThemeFamily('makosh')
    surface.selectThemeMode('dark')
    surface.selectThemeFamily('light')
    surface.selectThemeMode('makosh')
    expect(surface.currentTheme.value).toBe('makosh-dark')
  })
})

type AppLayoutNavbarSurface = ReturnType<typeof useAppLayoutNavbarSurface>

function currentNavigationPath(surface: AppLayoutNavbarSurface): string[] {
  return surface.navigationLevels.value.map((level) => level.currentItem.label)
}

function levelOptions(
  surface: AppLayoutNavbarSurface,
  label: string
): string[] {
  return (
    surface.navigationLevels.value
      .find((level) => level.label === label)
      ?.items.map((item) => item.label) ?? []
  )
}
