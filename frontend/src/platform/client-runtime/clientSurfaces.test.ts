import { describe, expect, it } from 'vitest'

import { ClientSurfaceIdV1 } from '../../gen/makosh/gateway/v1/client_bootstrap_pb'
import { clientSurfaceCatalog, personasSupplementalClientSurfaceAdapters } from './clientSurfaces'

describe('compiled client surface catalog', () => {
	it('binds Communications and every provider route to its own wire surface', () => {
		const communications = clientSurfaceCatalog.filter(
			(surface) => surface.parentRouteId === 'communications',
		)

		expect(communications.map((surface) => surface.surfaceId)).toEqual([
			ClientSurfaceIdV1.COMMUNICATIONS,
			ClientSurfaceIdV1.MAIL,
			ClientSurfaceIdV1.TELEGRAM,
			ClientSurfaceIdV1.WHATSAPP,
			ClientSurfaceIdV1.ZULIP,
		])
		expect(new Set(communications.map((surface) => surface.surfaceId)).size).toBe(
			communications.length,
		)
		expect(communications.map((surface) => surface.adapterId)).toEqual([
			'communications-owner',
			'mail-integration',
			'telegram-integration',
			'whatsapp-integration',
			'zulip-integration',
		])
	})

	it('composes Relationships into Personas without adding a top-level route', () => {
		expect(personasSupplementalClientSurfaceAdapters).toEqual([{
			surfaceId: ClientSurfaceIdV1.PERSONAS,
			adapterId: 'relationships-owner',
		}])
		expect(clientSurfaceCatalog.filter((surface) => surface.routeId === 'personas')).toHaveLength(1)
	})
})
