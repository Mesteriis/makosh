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
	| 'organizations'
	| 'projects'
	| 'obligations'
	| 'decisions'
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
	| 'calendar-owner'
	| 'knowledge-owner'
	| 'organizations-owner'
	| 'projects-owner'
	| 'obligations-owner'
	| 'decisions-owner'
	| 'documents-owner'
	| 'mail-integration'
	| 'persons-owner'
	| 'review-owner'
	| 'relationships-owner'
	| 'tasks-owner'
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
	{ routeId: 'review', surfaceId: ClientSurfaceIdV1.REVIEW, label: 'Review', icon: 'tabler:clipboard-check', iconTone: 'review', adapterId: 'review-owner' },
	{ routeId: 'personas', surfaceId: ClientSurfaceIdV1.PERSONAS, label: 'Personas', icon: 'tabler:user-circle', iconTone: 'knowledge', adapterId: 'persons-owner' },
	{ routeId: 'knowledge', surfaceId: ClientSurfaceIdV1.KNOWLEDGE, label: 'Knowledge', icon: 'tabler:notebook', iconTone: 'knowledge', adapterId: 'knowledge-owner' },
	{ routeId: 'tasks', surfaceId: ClientSurfaceIdV1.TASKS, label: 'Tasks', icon: 'tabler:checkbox', iconTone: 'tasks', adapterId: 'tasks-owner' },
	{ routeId: 'calendar', surfaceId: ClientSurfaceIdV1.CALENDAR, label: 'Calendar', icon: 'tabler:calendar', iconTone: 'calendar', adapterId: 'calendar-owner' },
	{ routeId: 'organizations', surfaceId: ClientSurfaceIdV1.ORGANIZATIONS, label: 'Organizations', icon: 'tabler:building-community', iconTone: 'knowledge', adapterId: 'organizations-owner' },
	{ routeId: 'projects', surfaceId: ClientSurfaceIdV1.PROJECTS, label: 'Projects', icon: 'tabler:briefcase', iconTone: 'knowledge', adapterId: 'projects-owner' },
	{ routeId: 'obligations', surfaceId: ClientSurfaceIdV1.OBLIGATIONS, label: 'Obligations', icon: 'tabler:contract', iconTone: 'tasks', adapterId: 'obligations-owner' },
	{ routeId: 'decisions', surfaceId: ClientSurfaceIdV1.DECISIONS, label: 'Decisions', icon: 'tabler:git-branch', iconTone: 'knowledge', adapterId: 'decisions-owner' },
	{ routeId: 'documents', surfaceId: ClientSurfaceIdV1.DOCUMENTS, label: 'Documents', icon: 'tabler:file-text', iconTone: 'documents', adapterId: 'documents-owner' },
	{ routeId: 'settings', surfaceId: ClientSurfaceIdV1.SETTINGS, label: 'Settings', icon: 'tabler:settings', iconTone: 'settings', adapterId: 'system-control' },
]

// Personas composes its canonical Person directory with the separately
// admitted Relationships owner without adding a second top-level route.
export const personasSupplementalClientSurfaceAdapters = [
	{ surfaceId: ClientSurfaceIdV1.PERSONAS, adapterId: 'relationships-owner' },
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
