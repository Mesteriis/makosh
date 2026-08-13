import { readFileSync } from 'node:fs'
import { describe, expect, it } from 'vitest'
import { ClientSurfaceAvailabilityStateV1 } from '../../gen/makosh/gateway/v1/client_bootstrap_pb'
import { clientSurfaceCatalog } from '../../platform/client-runtime/clientSurfaces'
import { recoveryClientBootstrap } from '../../platform/gateway/clientBootstrap'
import { compiledClientSurfaceAdapterIds } from '../client-surfaces/compiledClientSurfaceAdapters'
import {
	buildClientRouteTree,
	resolveClientNavigationTarget,
} from './useClientNavigationSurface'

describe('compiled client navigation', () => {
	it('keeps every product route disabled in the recovery shell', () => {
		const tree = buildClientRouteTree(recoveryClientBootstrap())
		const productRoutes = flattenNavigationTree(tree).filter((item) => item.id !== 'settings')

		expect(productRoutes.every((item) => item.disabled)).toBe(true)
		expect(tree.find((item) => item.id === 'settings')?.disabled).toBe(false)
		expect(compiledClientSurfaceAdapterIds).toEqual([
			'calendar-owner',
			'communications-owner',
			'documents-owner',
			'decisions-owner',
			'knowledge-owner',
			'organizations-owner',
			'obligations-owner',
			'persons-owner',
			'projects-owner',
			'mail-integration',
			'review-owner',
			'relationships-owner',
			'tasks-owner',
			'telegram-integration',
			'whatsapp-integration',
			'zulip-integration',
			'system-control',
		])
	})

	it('enables only the admitted Communications children', () => {
		const bootstrap = Object.assign(new Map(recoveryClientBootstrap()), {
			modules: [] as const,
			systemStatus: [] as const,
		})
		bootstrap.set('communications-all', {
			state: ClientSurfaceAvailabilityStateV1.AVAILABLE,
			reasonCode: '',
			available: true,
		})
		bootstrap.set('communications-mail', {
			state: ClientSurfaceAvailabilityStateV1.AVAILABLE,
			reasonCode: '',
			available: true,
		})
		bootstrap.set('communications-telegram', {
			state: ClientSurfaceAvailabilityStateV1.AVAILABLE,
			reasonCode: '',
			available: true,
		})
		bootstrap.set('communications-whatsapp', {
			state: ClientSurfaceAvailabilityStateV1.AVAILABLE,
			reasonCode: '',
			available: true,
		})
		bootstrap.set('communications-zulip', {
			state: ClientSurfaceAvailabilityStateV1.AVAILABLE,
			reasonCode: '',
			available: true,
		})

		const communications = buildClientRouteTree(bootstrap).find(
			(item) => item.id === 'communications',
		)

		expect(communications?.disabled).toBe(false)
		expect(communications?.children?.find((item) => item.id === 'communications-all')).toMatchObject({
			disabled: false,
			disabledReason: '',
		})
		expect(communications?.children?.find((item) => item.id === 'communications-mail')).toMatchObject({
			disabled: false,
			disabledReason: '',
		})
		expect(communications?.children?.find((item) => item.id === 'communications-telegram')).toMatchObject({
			disabled: false,
			disabledReason: '',
		})
		expect(communications?.children?.find((item) => item.id === 'communications-whatsapp')).toMatchObject({
			disabled: false,
			disabledReason: '',
		})
		expect(communications?.children?.find((item) => item.id === 'communications-zulip')).toMatchObject({
			disabled: false,
			disabledReason: '',
		})
		expect(communications?.children?.filter(
			(item) =>
				item.id !== 'communications-all'
				&& item.id !== 'communications-mail'
				&& item.id !== 'communications-telegram'
				&& item.id !== 'communications-whatsapp'
				&& item.id !== 'communications-zulip',
		)
			.every((item) => item.disabled)).toBe(true)
		expect(resolveClientNavigationTarget(
			buildClientRouteTree(bootstrap),
			'communications',
		)).toBe('communications-all')
		expect(resolveClientNavigationTarget(
			buildClientRouteTree(bootstrap),
			'communications-mail',
		)).toBe('communications-mail')
	})

	it('does not route through a parent whose children are all unavailable', () => {
		expect(resolveClientNavigationTarget(
			buildClientRouteTree(recoveryClientBootstrap()),
			'communications',
		)).toBeUndefined()
	})

	it('enables only routes with exact compiled adapters when Gateway marks every route available', () => {
		const bootstrap = Object.assign(new Map(recoveryClientBootstrap()), { modules: [] as const, systemStatus: [] as const })
		for (const surface of clientSurfaceCatalog) {
			if (surface.routeId === 'settings') continue
			bootstrap.set(surface.routeId, {
				state: ClientSurfaceAvailabilityStateV1.AVAILABLE,
				reasonCode: '',
				available: true,
			})
		}

		const tree = buildClientRouteTree(bootstrap)
		const productRoutes = flattenNavigationTree(tree).filter(
			(item) => item.id !== 'settings' && item.id !== 'communications',
		)
		const compiledRoutes = productRoutes.filter(
			(item) =>
				item.id === 'communications-all'
				|| item.id === 'communications-mail'
				|| item.id === 'communications-telegram'
				|| item.id === 'communications-whatsapp'
					|| item.id === 'communications-zulip'
					|| item.id === 'review'
					|| item.id === 'knowledge'
					|| item.id === 'tasks'
					|| item.id === 'calendar'
					|| item.id === 'organizations'
					|| item.id === 'documents'
					|| item.id === 'personas'
					|| item.id === 'projects',
		)
		const uncompiledRoutes = productRoutes.filter(
			(item) =>
				item.id !== 'communications-all'
				&& item.id !== 'communications-mail'
				&& item.id !== 'communications-telegram'
				&& item.id !== 'communications-whatsapp'
					&& item.id !== 'communications-zulip'
					&& item.id !== 'review'
					&& item.id !== 'knowledge'
					&& item.id !== 'tasks'
					&& item.id !== 'calendar'
					&& item.id !== 'organizations'
					&& item.id !== 'documents'
					&& item.id !== 'personas'
					&& item.id !== 'projects',
		)

		expect(compiledRoutes).toHaveLength(13)
		expect(compiledRoutes.every((item) => !item.disabled && item.disabledReason === '')).toBe(true)
		expect(uncompiledRoutes.every((item) => item.disabled)).toBe(true)
		expect(uncompiledRoutes.every(
			(item) => item.disabledReason === 'client_route_adapter_unavailable',
		)).toBe(true)
	})

	it('does not retain the legacy navbar or Communications facade as an active fallback', () => {
		const appLayoutSource = readFileSync(new URL('../layout/AppLayoutRoot.vue', import.meta.url), 'utf8')
		const navigationSource = readFileSync(new URL('./useClientNavigationSurface.ts', import.meta.url), 'utf8')

		expect(appLayoutSource).not.toContain('useAppLayoutNavbarSurface')
		expect(appLayoutSource).not.toContain('useCommunicationsViewSurface')
		expect(navigationSource).not.toContain('useAppLayoutNavbarSurface')
		expect(navigationSource).not.toContain('useCommunicationsWorkspaceSurface')
		expect(navigationSource).not.toContain('useCommunicationsPageSurface')
	})

	it('keeps the last confirmed bootstrap while the SSE recovery request is unavailable', () => {
		const navigationSource = readFileSync(new URL('./useClientNavigationSurface.ts', import.meta.url), 'utf8')

		expect(navigationSource).toContain('refreshBootstrap(preserveSnapshotOnFailure = false)')
		expect(navigationSource).toContain('if (!preserveSnapshotOnFailure) bootstrap.value = recoveryClientBootstrap()')
		expect(navigationSource).toContain('await refreshBootstrap(true)')
		expect(navigationSource).toContain('getBrowserGatewayRealtimeHub().subscribe')
		expect(navigationSource).not.toContain('new BrowserGatewayRealtime()')
		expect(navigationSource).not.toContain('setInterval(')
	})
})

type NavigationTreeItem = {
	id: string
	disabled?: boolean
	disabledReason?: string
	children?: readonly NavigationTreeItem[]
}

function flattenNavigationTree(nodes: readonly NavigationTreeItem[]): NavigationTreeItem[] {
	return nodes.flatMap((node) => [node, ...(node.children ? flattenNavigationTree(node.children) : [])])
}
