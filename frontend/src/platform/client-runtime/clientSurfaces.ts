import {
	ClientSurfaceAvailabilityStateV1,
	ClientSurfaceIdV1,
} from '../../gen/makosh/gateway/v1/client_bootstrap_pb'

export const CLIENT_SURFACE_CONTRACT_MAJOR = 1

export type ClientSurfaceRouteId =
	| 'dashboard'
	| 'communications-all'
	| 'communications-mail'
	| 'communications-telegram'
	| 'communications-whatsapp'
	| 'communications-zulip'
	| 'review'
	| 'personas'
	| 'knowledge'
	| 'tasks'
	| 'calendar'
	| 'documents'
	| 'settings'

export type ClientSurfaceMetadata = {
	routeId: ClientSurfaceRouteId
	surfaceId: ClientSurfaceIdV1
	label: string
	icon: string
	iconTone: ClientSurfaceIconTone
	adapterId: ClientSurfaceAdapterId
	parentRouteId?: 'communications'
}

export type ClientSurfaceAdapterId =
	| 'communications-owner'
	| 'mail-integration'
	| 'planned-owner'
	| 'system-control'
	| 'telegram-integration'
	| 'whatsapp-integration'
	| 'zulip-integration'

export type ClientSurfaceIconTone =
	| 'calendar'
	| 'communication'
	| 'dashboard'
	| 'documents'
	| 'knowledge'
	| 'mail'
	| 'review'
	| 'settings'
	| 'tasks'
	| 'telegram'
	| 'whatsapp'
	| 'zulip'

export type ClientSurfaceAvailability = {
	state: ClientSurfaceAvailabilityStateV1
	reasonCode: string
	available: boolean
}

export const clientSurfaceCatalog: readonly ClientSurfaceMetadata[] = [
	{ routeId: 'dashboard', surfaceId: ClientSurfaceIdV1.DASHBOARD, label: 'Dashboard', icon: 'tabler:layout-dashboard', iconTone: 'dashboard', adapterId: 'planned-owner' },
	{ routeId: 'communications-all', surfaceId: ClientSurfaceIdV1.COMMUNICATIONS, label: 'All communications', icon: 'tabler:messages', iconTone: 'communication', adapterId: 'communications-owner', parentRouteId: 'communications' },
	{ routeId: 'communications-mail', surfaceId: ClientSurfaceIdV1.MAIL, label: 'Mail', icon: 'tabler:mail', iconTone: 'mail', adapterId: 'mail-integration', parentRouteId: 'communications' },
	{ routeId: 'communications-telegram', surfaceId: ClientSurfaceIdV1.TELEGRAM, label: 'Telegram', icon: 'tabler:brand-telegram', iconTone: 'telegram', adapterId: 'telegram-integration', parentRouteId: 'communications' },
	{ routeId: 'communications-whatsapp', surfaceId: ClientSurfaceIdV1.WHATSAPP, label: 'WhatsApp', icon: 'tabler:brand-whatsapp', iconTone: 'whatsapp', adapterId: 'whatsapp-integration', parentRouteId: 'communications' },
	{ routeId: 'communications-zulip', surfaceId: ClientSurfaceIdV1.ZULIP, label: 'Zulip', icon: 'tabler:brand-zulip', iconTone: 'zulip', adapterId: 'zulip-integration', parentRouteId: 'communications' },
	{ routeId: 'review', surfaceId: ClientSurfaceIdV1.REVIEW, label: 'Review', icon: 'tabler:clipboard-check', iconTone: 'review', adapterId: 'planned-owner' },
	{ routeId: 'personas', surfaceId: ClientSurfaceIdV1.PERSONAS, label: 'Personas', icon: 'tabler:user-circle', iconTone: 'knowledge', adapterId: 'planned-owner' },
	{ routeId: 'knowledge', surfaceId: ClientSurfaceIdV1.KNOWLEDGE, label: 'Knowledge', icon: 'tabler:share', iconTone: 'knowledge', adapterId: 'planned-owner' },
	{ routeId: 'tasks', surfaceId: ClientSurfaceIdV1.TASKS, label: 'Tasks', icon: 'tabler:checkbox', iconTone: 'tasks', adapterId: 'planned-owner' },
	{ routeId: 'calendar', surfaceId: ClientSurfaceIdV1.CALENDAR, label: 'Calendar', icon: 'tabler:calendar', iconTone: 'calendar', adapterId: 'planned-owner' },
	{ routeId: 'documents', surfaceId: ClientSurfaceIdV1.DOCUMENTS, label: 'Documents', icon: 'tabler:file-text', iconTone: 'documents', adapterId: 'planned-owner' },
	{ routeId: 'settings', surfaceId: ClientSurfaceIdV1.SETTINGS, label: 'Settings', icon: 'tabler:settings', iconTone: 'settings', adapterId: 'system-control' },
]

export function clientSurfacesByWireId(surfaceId: ClientSurfaceIdV1): readonly ClientSurfaceMetadata[] {
	return clientSurfaceCatalog.filter((surface) => surface.surfaceId === surfaceId)
}

export function unavailableClientSurface(reasonCode = 'bootstrap_unavailable'): ClientSurfaceAvailability {
	return {
		state: ClientSurfaceAvailabilityStateV1.UNAVAILABLE,
		reasonCode,
		available: false,
	}
}
