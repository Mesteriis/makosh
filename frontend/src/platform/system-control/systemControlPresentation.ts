import {
  ClientSettingsApplyStateV1,
  ClientSurfaceAvailabilityStateV1,
  type ClientModuleBootstrapV1,
} from '../../gen/makosh/gateway/v1/client_bootstrap_pb'
import {
  clientSurfaceCatalog,
  type ClientSurfaceAdapterId,
  type ClientSurfaceRouteId,
} from '../client-runtime/clientSurfaces'
import type { ClientBootstrapSnapshot } from '../gateway/clientBootstrap'

export type SystemControlSurfaceRow = (typeof clientSurfaceCatalog)[number] & {
  available: boolean
  reasonCode: string
  state: ClientSurfaceAvailabilityStateV1
  compiledAdapterReady: boolean
}

export type SystemControlModuleRow = {
  registrationId: string
  moduleId: string
  grantEpoch: string
  capabilityCount: number
  sectionsEnabled: boolean
  applyState: ClientSettingsApplyStateV1 | null
  reasonCode: string
}

export function systemControlAvailableSurfaceCount(bootstrap: ClientBootstrapSnapshot): number {
  return clientSurfaceCatalog.filter((surface) =>
    surface.routeId === 'settings' || bootstrap.get(surface.routeId)?.available
  ).length
}

export function systemControlSurfaceRows(
  bootstrap: ClientBootstrapSnapshot,
  compiledAdapterIds: readonly ClientSurfaceAdapterId[]
): readonly SystemControlSurfaceRow[] {
  return clientSurfaceCatalog.map((surface) => {
    const availability = surface.routeId === 'settings'
      ? { available: true, reasonCode: '', state: ClientSurfaceAvailabilityStateV1.AVAILABLE }
      : bootstrap.get(surface.routeId) ?? {
        available: false,
        reasonCode: 'bootstrap_unavailable',
        state: ClientSurfaceAvailabilityStateV1.UNAVAILABLE,
      }
    return {
      ...surface,
      ...availability,
      compiledAdapterReady: compiledAdapterIds.includes(surface.adapterId),
    }
  })
}

export function systemControlModuleRows(
  modules: readonly ClientModuleBootstrapV1[]
): readonly SystemControlModuleRow[] {
  return modules.map((module) => ({
    registrationId: module.registrationId,
    moduleId: module.moduleId,
    grantEpoch: module.grantEpoch.toString(),
    capabilityCount: module.capabilityIds.length,
    sectionsEnabled: module.sectionsEnabled,
    applyState: module.settings?.applyState ?? null,
    reasonCode: module.settings?.sanitizedReasonCode ?? '',
  }))
}

export function systemControlSurfaceStateLabel(
  routeId: ClientSurfaceRouteId,
  state: ClientSurfaceAvailabilityStateV1,
  available: boolean
): string {
  if (routeId === 'settings') return 'Recovery available'
  if (available) return 'Available'
  if (state === ClientSurfaceAvailabilityStateV1.AVAILABLE) return 'Adapter unavailable'
  if (state === ClientSurfaceAvailabilityStateV1.STARTING) return 'Starting'
  if (state === ClientSurfaceAvailabilityStateV1.BLOCKED) return 'Blocked'
  if (state === ClientSurfaceAvailabilityStateV1.UNAVAILABLE) return 'Unavailable'
  return 'Not admitted'
}
