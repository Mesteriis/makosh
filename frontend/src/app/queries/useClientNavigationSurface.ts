import { computed, onBeforeUnmount, onMounted, ref } from 'vue'

import {
	ClientSystemComponentIdV1,
	ClientSystemComponentStateV1,
	type ClientSystemComponentStatusV1,
} from '../../gen/makosh/gateway/v1/client_bootstrap_pb'

import {
	clientSurfaceCatalog,
	type ClientSurfaceIconTone,
	type ClientSurfaceRouteId,
	unavailableClientSurface,
} from '../../platform/client-runtime/clientSurfaces'
import { hasCompiledClientSurfaceAdapter } from '../client-surfaces/compiledClientSurfaceAdapters'
import {
	fetchClientBootstrap,
	recoveryClientBootstrap,
	type ClientBootstrapSnapshot,
} from '../../platform/gateway/clientBootstrap'
import {
	type BrowserGatewayRealtimeSubscription,
} from '../../platform/gateway/browserGatewayRealtime'
import { getBrowserGatewayRealtimeHub } from '../../platform/gateway/browserGatewayRealtimeHub'
import { applyClientSystemStatusEvent } from '../../platform/gateway/browserGatewaySystemStatus'
import {
	defaultUiThemeName,
	isUiThemeFamily,
	isUiThemeMode,
	themeNameToSelection,
	themeSelectionToName,
	type UiThemeFamily,
	type UiThemeMode,
	type UiThemeName,
} from '../../shared/ui/foundation/theme'

type NavigationItem = {
	id: string
	label: string
	icon?: string
	iconTone?: ClientSurfaceIconTone
	disabled?: boolean
	disabledReason?: string
}

type NavigationNode = NavigationItem & { children?: readonly NavigationNode[] }

export type ClientNavigationHealthCheck = {
	id: string
	label: string
	status: 'healthy' | 'degraded' | 'unhealthy' | 'unavailable'
	detail: string
	depth?: number
}

export function useClientNavigationSurface() {
	const selectedRouteId = ref<ClientSurfaceRouteId>('settings')
	const initialTheme = themeNameToSelection(defaultUiThemeName)
	const currentThemeFamily = ref<UiThemeFamily>(initialTheme.family)
	const currentThemeMode = ref<UiThemeMode>(initialTheme.mode)
	const currentLanguage = ref<'ru' | 'en'>(readInterfaceLanguage())
	const bootstrap = ref<ClientBootstrapSnapshot>(recoveryClientBootstrap())
	const bootstrapError = ref('bootstrap_unavailable')
	const routeDowngradeReason = ref('')
	const gatewayRoundTripMs = ref<number | null>(null)
	let realtimeSubscription: BrowserGatewayRealtimeSubscription | undefined
	let realtimeRecoveryActive = false

	const currentTheme = computed<UiThemeName>(() => themeSelectionToName(
		currentThemeFamily.value,
		currentThemeMode.value,
	))
	const routeTree = computed<readonly NavigationNode[]>(() => buildClientRouteTree(bootstrap.value))
	const selectedRoutePath = computed(() => findRoutePath(routeTree.value, selectedRouteId.value)
		?? findRoutePath(routeTree.value, 'settings')
		?? [])
	const selectedTopLevelRouteId = computed(() => selectedRoutePath.value[0]?.id ?? 'settings')
	const breadcrumbs = computed(() => selectedRoutePath.value.map(({ id, label }) => ({ id, label })))
	const navigationLevels = computed(() => selectedRoutePath.value.map((node, index, path) => {
		const siblings = index === 0 ? routeTree.value : path[index - 1]?.children ?? []
		return {
			id: `navigation-level-${index}`,
			label: index === 0 ? 'Main menu' : 'Sub menu',
			currentItem: node,
			items: siblings,
		}
	}))
	const healthChecks = computed<readonly ClientNavigationHealthCheck[]>(() => buildHealthChecks(
		bootstrap.value.systemStatus,
		bootstrapError.value,
		gatewayRoundTripMs.value,
	))

	function selectNavigationItem(itemId: string): void {
		const target = resolveClientNavigationTarget(routeTree.value, itemId)
		if (target) selectedRouteId.value = target
	}

	async function refreshBootstrap(preserveSnapshotOnFailure = false): Promise<void> {
		const startedAt = Date.now()
		try {
			bootstrap.value = await fetchClientBootstrap()
			gatewayRoundTripMs.value = Math.max(0, Date.now() - startedAt)
			bootstrapError.value = ''
		} catch {
			if (!preserveSnapshotOnFailure) bootstrap.value = recoveryClientBootstrap()
			gatewayRoundTripMs.value = null
			bootstrapError.value = 'bootstrap_unavailable'
		}
		if (selectedRouteId.value !== 'settings' && !isRouteAvailable(selectedRouteId.value)) {
			routeDowngradeReason.value = bootstrap.value.get(selectedRouteId.value)?.reasonCode
				|| 'surface_unavailable'
			selectedRouteId.value = 'settings'
		}
	}

	function isRouteAvailable(routeId: ClientSurfaceRouteId): boolean {
		return routeId === 'settings' || bootstrap.value.get(routeId)?.available === true
	}

	function selectThemeFamily(value: string): void {
		if (isUiThemeFamily(value)) currentThemeFamily.value = value
	}

	function selectThemeMode(value: string): void {
		if (isUiThemeMode(value)) currentThemeMode.value = value
	}

	function selectLanguage(value: string): void {
		if (value !== 'ru' && value !== 'en') return
		currentLanguage.value = value
		document.documentElement.lang = value
		try { window.localStorage.setItem('makosh.interface-language', value) } catch { /* preference remains memory-only */ }
	}

	function openRealtime(): void {
		realtimeSubscription?.close()
		realtimeSubscription = getBrowserGatewayRealtimeHub().subscribe({
			onEvent: (event) => {
				try {
					const updated = applyClientSystemStatusEvent(bootstrap.value, event)
					if (updated) bootstrap.value = updated
				} catch {
					void recoverRealtime()
				}
			},
			onStreamState: () => undefined,
			onReplayGap: () => void recoverRealtime(),
			onProtocolError: () => void recoverRealtime(),
		})
	}

	async function recoverRealtime(): Promise<void> {
		if (realtimeRecoveryActive) return
		realtimeRecoveryActive = true
		realtimeSubscription?.close()
		realtimeSubscription = undefined
		try {
			await refreshBootstrap(true)
			openRealtime()
		} finally {
			realtimeRecoveryActive = false
		}
	}

	onMounted(() => {
		document.documentElement.lang = currentLanguage.value
		void refreshBootstrap().finally(openRealtime)
	})
	onBeforeUnmount(() => {
		realtimeSubscription?.close()
	})

	return {
		bootstrap,
		bootstrapError,
		routeDowngradeReason,
		breadcrumbs,
		currentLanguage,
		currentTheme,
		currentThemeFamily,
		currentThemeMode,
		healthChecks,
		healthStatusLabelVisibleMs: 5000,
		languageOptions: [{ value: 'ru', label: 'Русский' }, { value: 'en', label: 'English' }],
		navigationLevels,
		notifications: computed(() => []),
		notificationToasts: computed(() => []),
		notificationToastVisibleMs: 5000,
		notificationsCount: computed(() => 0),
		selectedRouteId,
		selectedTopLevelRouteId,
		themeFamilyOptions: [{ value: 'base' as const, label: 'Base' }, { value: 'makosh' as const, label: 'Макошь' }],
		themeModeOptions: [{ value: 'light' as const, label: 'Light' }, { value: 'dark' as const, label: 'Dark' }],
		clearNotifications: () => undefined,
		dismissNotification: () => undefined,
		selectNavigationItem,
		refreshBootstrap,
		selectNotification: () => undefined,
		selectLanguage,
		selectThemeFamily,
		selectThemeMode,
	}
}

const systemComponentCatalog: readonly { id: ClientSystemComponentIdV1; label: string }[] = [
	{ id: ClientSystemComponentIdV1.KERNEL, label: 'Kernel' },
	{ id: ClientSystemComponentIdV1.CONTROL_STORE, label: 'Control Store' },
	{ id: ClientSystemComponentIdV1.MODULE_CONTROL_PLANE, label: 'Module Control Plane' },
	{ id: ClientSystemComponentIdV1.GATEWAY, label: 'Gateway' },
	{ id: ClientSystemComponentIdV1.VAULT, label: 'Vault' },
	{ id: ClientSystemComponentIdV1.STORAGE_CONTROL, label: 'Storage Control' },
	{ id: ClientSystemComponentIdV1.POSTGRESQL, label: 'PostgreSQL' },
	{ id: ClientSystemComponentIdV1.PGBOUNCER, label: 'PgBouncer' },
	{ id: ClientSystemComponentIdV1.NATS, label: 'NATS' },
	{ id: ClientSystemComponentIdV1.EVENT_HUB, label: 'Event Hub' },
	{ id: ClientSystemComponentIdV1.SCHEDULER, label: 'Scheduler' },
	{ id: ClientSystemComponentIdV1.CLOCK, label: 'Clock' },
	{ id: ClientSystemComponentIdV1.BLOB, label: 'Blob' },
	{ id: ClientSystemComponentIdV1.TELEMETRY, label: 'Telemetry' },
	{ id: ClientSystemComponentIdV1.SSE, label: 'SSE' },
]

function buildHealthChecks(
	statuses: readonly ClientSystemComponentStatusV1[],
	bootstrapError: string,
	roundTripMs: number | null,
): readonly ClientNavigationHealthCheck[] {
	const byId = new Map(statuses.map((status) => [status.componentId, status]))
	const children: ClientNavigationHealthCheck[] = [networkHealth(roundTripMs, bootstrapError)]
	for (const component of systemComponentCatalog) {
		const status = byId.get(component.id)
		children.push({
			id: `backend-${component.id}`,
			label: component.label,
		status: status ? healthTone(status.state) : 'unavailable',
			detail: status?.sanitizedReasonCode || (status ? systemStateLabel(status.state) : 'status_unavailable'),
			depth: 1,
		})
	}
	return [{
		id: 'backend',
		label: 'Backend',
		status: aggregateHealth(children),
		detail: bootstrapError || `${children.length} checks`,
	}, ...children]
}

function networkHealth(roundTripMs: number | null, error: string): ClientNavigationHealthCheck {
	if (roundTripMs === null) return { id: 'network', label: 'Connection', status: 'unhealthy', detail: error || 'unavailable', depth: 1 }
	return {
		id: 'network',
		label: 'Connection',
		status: roundTripMs >= 1000 ? 'unhealthy' : roundTripMs >= 250 ? 'degraded' : 'healthy',
		detail: `${roundTripMs} ms round-trip`,
		depth: 1,
	}
}

function healthTone(state: ClientSystemComponentStateV1): ClientNavigationHealthCheck['status'] {
	if (state === ClientSystemComponentStateV1.HEALTHY) return 'healthy'
	if (state === ClientSystemComponentStateV1.DEGRADED) return 'degraded'
	if (state === ClientSystemComponentStateV1.NOT_ADMITTED || state === ClientSystemComponentStateV1.UNAVAILABLE) return 'unavailable'
	return 'unhealthy'
}

function systemStateLabel(state: ClientSystemComponentStateV1): string {
	if (state === ClientSystemComponentStateV1.HEALTHY) return 'healthy'
	if (state === ClientSystemComponentStateV1.DEGRADED) return 'degraded'
	if (state === ClientSystemComponentStateV1.NOT_ADMITTED) return 'not_admitted'
	return 'unavailable'
}

function aggregateHealth(checks: readonly ClientNavigationHealthCheck[]): ClientNavigationHealthCheck['status'] {
	if (checks.some((check) => check.status === 'unhealthy')) return 'unhealthy'
	if (checks.some((check) => check.status === 'degraded')) return 'degraded'
	return 'healthy'
}

function readInterfaceLanguage(): 'ru' | 'en' {
	try {
		const value = window.localStorage.getItem('makosh.interface-language')
		if (value === 'ru' || value === 'en') return value
	} catch { /* use the product default */ }
	return 'ru'
}

export function buildClientRouteTree(bootstrap: ClientBootstrapSnapshot): readonly NavigationNode[] {
	const communicationsChildren = [
		toNavigationNode('communications-all', bootstrap),
		toNavigationNode('communications-mail', bootstrap),
		toNavigationNode('communications-telegram', bootstrap),
		toNavigationNode('communications-whatsapp', bootstrap),
		toNavigationNode('communications-zulip', bootstrap),
	]
	const allCommunicationsChildrenDisabled = communicationsChildren.every((item) => item.disabled)

	return [
		toNavigationNode('dashboard', bootstrap),
		{
			id: 'communications',
			label: 'Communications',
			icon: 'tabler:messages',
			iconTone: 'communication',
			disabled: allCommunicationsChildrenDisabled,
			disabledReason: allCommunicationsChildrenDisabled ? 'client_route_adapter_unavailable' : '',
			children: communicationsChildren,
		},
		toNavigationNode('review', bootstrap),
		toNavigationNode('personas', bootstrap),
		toNavigationNode('knowledge', bootstrap),
		toNavigationNode('tasks', bootstrap),
		toNavigationNode('calendar', bootstrap),
		toNavigationNode('organizations', bootstrap),
		toNavigationNode('projects', bootstrap),
		toNavigationNode('documents', bootstrap),
		toNavigationNode('settings', bootstrap),
	]
}

export function resolveClientNavigationTarget(
	nodes: readonly NavigationNode[],
	itemId: string,
): ClientSurfaceRouteId | undefined {
	const target = findRoutePath(nodes, itemId)?.at(-1)
	if (!target || target.disabled) return undefined
	const defaultChild = target.children?.find((child) => !child.disabled)
	return (defaultChild?.id ?? target.id) as ClientSurfaceRouteId
}

function toNavigationNode(routeId: ClientSurfaceRouteId, bootstrap: ClientBootstrapSnapshot): NavigationNode {
	const surface = clientSurfaceCatalog.find((item) => item.routeId === routeId)
	if (!surface) throw new Error(`Unknown compiled client surface: ${routeId}`)
	const availability = routeId === 'settings'
		? { available: true, reasonCode: '' }
		: bootstrap.get(routeId) ?? unavailableClientSurface()
	const compiledAdapterReady = hasCompiledClientSurfaceAdapter(surface)
	return {
		id: surface.routeId,
		label: surface.label,
		icon: surface.icon,
		iconTone: surface.iconTone,
		disabled: !availability.available || !compiledAdapterReady,
		disabledReason: availability.reasonCode || (compiledAdapterReady ? '' : 'client_route_adapter_unavailable'),
	}
}

function findRoutePath(nodes: readonly NavigationNode[], itemId: string, ancestors: readonly NavigationNode[] = []): NavigationNode[] | undefined {
	for (const node of nodes) {
		const path = [...ancestors, node]
		if (node.id === itemId) return path
		const nested = node.children ? findRoutePath(node.children, itemId, path) : undefined
		if (nested) return nested
	}
	return undefined
}
