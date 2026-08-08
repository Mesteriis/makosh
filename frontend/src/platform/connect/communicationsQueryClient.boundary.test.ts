import { readFileSync } from 'node:fs'

import { describe, expect, it } from 'vitest'

describe('Communications clean-room query client boundary', () => {
	it('uses only the generated owner query contract and shared Gateway transport', () => {
		const source = readFileSync(new URL('./communicationsQueryClient.ts', import.meta.url), 'utf8')

		expect(source).toContain("../../gen/makosh/communications/query/v1/query_pb")
		expect(source).toContain('CommunicationsQueryService')
		expect(source).toContain('createBrowserGatewayConnectTransport')
		expect(source).not.toContain("./communicationsClient")
		expect(source).not.toContain('CommunicationsService')
	})
})
