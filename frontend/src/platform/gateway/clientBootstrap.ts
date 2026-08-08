import { createClient, type Client } from '@connectrpc/connect'
import { create } from '@bufbuild/protobuf'

import {
	ClientBootstrapRequestV1Schema,
	ClientBootstrapService,
	ClientSystemComponentIdV1,
	ClientSystemComponentStateV1,
	ClientSurfaceAvailabilityStateV1,
	type ClientBootstrapResponseV1,
	type ClientModuleBootstrapV1,
	type ClientSystemComponentStatusV1,
} from '../../gen/makosh/gateway/v1/client_bootstrap_pb'
import {
	CLIENT_SURFACE_CONTRACT_MAJOR,
	clientSurfaceCatalog,
	clientSurfacesByWireId,
	type ClientSurfaceAvailability,
	type ClientSurfaceRouteId,
	unavailableClientSurface,
} from '../client-runtime/clientSurfaces'
import { createBrowserGatewayConnectTransport } from './browserGatewayConnect'

export type ClientBootstrapSnapshot = ReadonlyMap<ClientSurfaceRouteId, ClientSurfaceAvailability> & {
	readonly modules: readonly ClientModuleBootstrapV1[]
	readonly systemStatus: readonly ClientSystemComponentStatusV1[]
}

export async function fetchClientBootstrap(
	client: Client<typeof ClientBootstrapService> = createClient(
		ClientBootstrapService,
		createBrowserGatewayConnectTransport(),
	),
): Promise<ClientBootstrapSnapshot> {
	const response = await client.getBootstrap(create(ClientBootstrapRequestV1Schema))
	return validateClientBootstrap(response)
}

export function recoveryClientBootstrap(): ClientBootstrapSnapshot {
	return bootstrapSnapshot(new Map(
		clientSurfaceIds().map((routeId) => [routeId, unavailableClientSurface()]),
	), [], [])
}

export function withClientSystemStatus(
	snapshot: ClientBootstrapSnapshot,
	statuses: readonly ClientSystemComponentStatusV1[],
): ClientBootstrapSnapshot {
	return bootstrapSnapshot(
		new Map(snapshot),
		snapshot.modules,
		validateSystemStatus(statuses),
	)
}

export function validateClientBootstrap(response: ClientBootstrapResponseV1): ClientBootstrapSnapshot {
	if (response.major !== CLIENT_SURFACE_CONTRACT_MAJOR) {
		throw new Error('Unsupported client bootstrap contract major')
	}

	const availability = new Map(recoveryClientBootstrap())
	const seen = new Set<ClientSurfaceRouteId>()
	for (const surface of response.surfaces) {
		const metadata = clientSurfacesByWireId(surface.surfaceId)
		if (metadata.length === 0 || metadata.some((route) => seen.has(route.routeId))) {
			throw new Error('Invalid client surface catalog')
		}
		if (surface.supportedClientContractMajor !== CLIENT_SURFACE_CONTRACT_MAJOR) {
			throw new Error('Unsupported client surface contract major')
		}
		if (!isKnownSurfaceState(surface.state)) {
			throw new Error('Invalid client surface availability state')
		}

		for (const route of metadata) {
			seen.add(route.routeId)
			availability.set(route.routeId, {
				state: surface.state,
				reasonCode: surface.sanitizedReasonCode,
				available: surface.state === ClientSurfaceAvailabilityStateV1.AVAILABLE,
			})
		}
	}

	if (seen.size !== clientSurfaceCatalog.length) {
		throw new Error('Incomplete client surface catalog')
	}

	return bootstrapSnapshot(availability, response.modules, validateSystemStatus(response.systemStatus))
}

function bootstrapSnapshot(
	availability: Map<ClientSurfaceRouteId, ClientSurfaceAvailability>,
	modules: readonly ClientModuleBootstrapV1[],
	systemStatus: readonly ClientSystemComponentStatusV1[],
): ClientBootstrapSnapshot {
	if (modules.length > 128) throw new Error('Client bootstrap module inventory is oversized')
	for (const module of modules) {
		if (!validClientBootstrapModule(module)) {
			throw new Error('Invalid client bootstrap module')
		}
	}
	return Object.assign(availability, { modules: modules.slice(), systemStatus: systemStatus.slice() })
}

function validateSystemStatus(statuses: readonly ClientSystemComponentStatusV1[]): readonly ClientSystemComponentStatusV1[] {
	if (statuses.length !== 15) throw new Error('Incomplete client system status')
	const seen = new Set<ClientSystemComponentIdV1>()
	for (const status of statuses) {
		if (!isKnownSystemComponent(status.componentId)
			|| !isKnownSystemState(status.state)
			|| seen.has(status.componentId)) throw new Error('Invalid client system status')
		seen.add(status.componentId)
	}
	return statuses
}

function isKnownSystemComponent(value: ClientSystemComponentIdV1): boolean {
	return Number.isInteger(value)
		&& value >= ClientSystemComponentIdV1.KERNEL
		&& value <= ClientSystemComponentIdV1.SSE
}

function isKnownSystemState(value: ClientSystemComponentStateV1): boolean {
	return value === ClientSystemComponentStateV1.HEALTHY
		|| value === ClientSystemComponentStateV1.DEGRADED
		|| value === ClientSystemComponentStateV1.UNAVAILABLE
		|| value === ClientSystemComponentStateV1.NOT_ADMITTED
}

function validClientBootstrapModule(module: ClientModuleBootstrapV1): boolean {
	return module.registrationId.length > 0
		&& module.registrationId.length <= 128
		&& module.moduleId.length > 0
		&& module.moduleId.length <= 128
		&& module.capabilityIds.length <= 256
		&& module.capabilityIds.every((capabilityId) => capabilityId.length > 0 && capabilityId.length <= 128)
}

function clientSurfaceIds(): ClientSurfaceRouteId[] {
	return clientSurfaceCatalog.map((surface) => surface.routeId)
}

function isKnownSurfaceState(value: ClientSurfaceAvailabilityStateV1): boolean {
	return value === ClientSurfaceAvailabilityStateV1.AVAILABLE
		|| value === ClientSurfaceAvailabilityStateV1.NOT_ADMITTED
		|| value === ClientSurfaceAvailabilityStateV1.STARTING
		|| value === ClientSurfaceAvailabilityStateV1.BLOCKED
		|| value === ClientSurfaceAvailabilityStateV1.UNAVAILABLE
}
