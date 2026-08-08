import { create } from '@bufbuild/protobuf'
import { describe, expect, it } from 'vitest'

import {
	ClientBootstrapResponseV1Schema,
	ClientModuleBootstrapV1Schema,
	ClientSystemComponentIdV1,
	ClientSystemComponentStateV1,
	ClientSystemComponentStatusV1Schema,
	ClientSurfaceAvailabilityStateV1,
	ClientSurfaceAvailabilityV1Schema,
	ClientSurfaceIdV1,
} from '../../gen/makosh/gateway/v1/client_bootstrap_pb'
import { clientSurfaceCatalog } from '../client-runtime/clientSurfaces'
import { validateClientBootstrap } from './clientBootstrap'

describe('validateClientBootstrap', () => {
	it('admits only explicitly available compiled surfaces', () => {
		const bootstrap = validateClientBootstrap(responseWithSurfaces())

		expect(bootstrap.get('settings')).toMatchObject({ available: true })
		expect(bootstrap.get('communications-all')).toMatchObject({
			available: false,
			reasonCode: 'surface_not_admitted',
		})
	})

	it('fails closed for an incomplete catalog', () => {
		const response = responseWithSurfaces()
		response.surfaces.pop()

		expect(() => validateClientBootstrap(response)).toThrow('Incomplete client surface catalog')
	})

	it('does not fan out the owner-neutral Communications surface to integration routes', () => {
		const response = responseWithSurfaces()
		const communications = response.surfaces.find(
			(surface) => surface.surfaceId === ClientSurfaceIdV1.COMMUNICATIONS,
		)
		communications!.state = ClientSurfaceAvailabilityStateV1.AVAILABLE
		communications!.sanitizedReasonCode = ''

		const bootstrap = validateClientBootstrap(response)

		expect(bootstrap.get('communications-all')).toMatchObject({ available: true, reasonCode: '' })
		for (const routeId of [
			'communications-mail',
			'communications-telegram',
			'communications-whatsapp',
			'communications-zulip',
		] as const) {
			expect(bootstrap.get(routeId)).toMatchObject({
				available: false,
				reasonCode: 'surface_not_admitted',
			})
		}
	})

	it('fails closed for an unknown wire enum value', () => {
		const response = responseWithSurfaces()
		response.surfaces[0]!.surfaceId = 99 as never

		expect(() => validateClientBootstrap(response)).toThrow('Invalid client surface catalog')
	})

	it('fails closed for incomplete or unknown system status', () => {
		const incomplete = responseWithSurfaces()
		incomplete.systemStatus.pop()
		expect(() => validateClientBootstrap(incomplete)).toThrow('Incomplete client system status')

		const unknown = responseWithSurfaces()
		unknown.systemStatus[0]!.componentId = 99 as never
		expect(() => validateClientBootstrap(unknown)).toThrow('Invalid client system status')
	})

	it('retains only bounded owner-scoped module composition from bootstrap', () => {
		const response = responseWithSurfaces()
		response.modules.push(create(ClientModuleBootstrapV1Schema, {
			registrationId: 'registration-visible',
			moduleId: 'module-visible',
			grantEpoch: 1n,
			capabilityIds: ['sections.read'],
			sectionsEnabled: true,
		}))

		const bootstrap = validateClientBootstrap(response)

		expect(bootstrap.modules).toHaveLength(1)
		expect(bootstrap.modules[0]).toMatchObject({ moduleId: 'module-visible', sectionsEnabled: true })
	})
})

function responseWithSurfaces() {
	return create(ClientBootstrapResponseV1Schema, {
		major: 1,
		systemStatus: Object.values(ClientSystemComponentIdV1)
			.filter((value): value is ClientSystemComponentIdV1 => typeof value === 'number' && value !== ClientSystemComponentIdV1.UNSPECIFIED)
			.map((componentId) => create(ClientSystemComponentStatusV1Schema, {
				componentId,
				state: ClientSystemComponentStateV1.HEALTHY,
			})),
		surfaces: [...new Map(clientSurfaceCatalog.map((surface) => [surface.surfaceId, surface])).values()].map((surface) => create(ClientSurfaceAvailabilityV1Schema, {
			surfaceId: surface.surfaceId,
			state: surface.routeId === 'settings'
				? ClientSurfaceAvailabilityStateV1.AVAILABLE
				: ClientSurfaceAvailabilityStateV1.NOT_ADMITTED,
			sanitizedReasonCode: surface.routeId === 'settings' ? '' : 'surface_not_admitted',
			supportedClientContractMajor: 1,
		})),
	})
}
