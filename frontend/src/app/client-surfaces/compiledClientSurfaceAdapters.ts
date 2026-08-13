import type {
	ClientSurfaceAdapterId,
	ClientSurfaceMetadata,
} from '../../platform/client-runtime/clientSurfaces'

export const compiledClientSurfaceAdapterIds: readonly ClientSurfaceAdapterId[] = [
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
]

export function hasCompiledClientSurfaceAdapter(surface: ClientSurfaceMetadata): boolean {
	return compiledClientSurfaceAdapterIds.includes(surface.adapterId)
}
