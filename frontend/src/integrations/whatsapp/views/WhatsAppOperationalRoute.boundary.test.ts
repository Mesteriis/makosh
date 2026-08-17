import { readFileSync } from 'node:fs'

import { describe, expect, it } from 'vitest'

describe('WhatsApp operational active route boundary', () => {
	it('keeps browser operations integration-owned and host execution isolated', () => {
		const route = read('../views/WhatsAppOperationalRoute.vue')
		const controller = read('../queries/useWhatsAppOperationalPage.ts')
		const readController = read('../queries/useWhatsAppOperationalRead.ts')
		const replayController = read('../queries/useWhatsAppOperationalReplay.ts')
		const accounts = read('../queries/whatsAppOperationalAccounts.ts')
		const gateway = read('../api/whatsappOperationalGateway.ts')
		const readClient = read('../api/whatsAppOperationalReadClient.ts')
		const readGateway = read('../api/whatsAppOperationalReadGateway.ts')
		const realtimeClient = read('../api/whatsAppOperationalRealtimeClient.ts')
		const replayGateway = read('../api/whatsAppOperationalReplayGateway.ts')
		const presentation = read('../presentation/WhatsAppOperationalPage.vue')
		const readPresentation = read('../presentation/WhatsAppOperationalReadPanel.vue')
		const replayPresentation = read('../presentation/WhatsAppOperationalReplayPanel.vue')
		const appLayout = read('../../../app/layout/AppLayoutRoot.vue')
		const compiledAdapters = read('../../../app/client-surfaces/compiledClientSurfaceAdapters.ts')

		for (const source of [
			route,
			controller,
			readController,
			replayController,
			accounts,
			gateway,
			readClient,
			readGateway,
			realtimeClient,
			replayGateway,
			presentation,
			readPresentation,
			replayPresentation,
		]) {
			expect(source).not.toMatch(/\/api\/v1\//)
			expect(source).not.toMatch(/domains\/communications/)
			expect(source).not.toMatch(/integrations\/(mail|telegram|zulip)/)
			expect(source).not.toMatch(/invoke\(|@tauri-apps/)
		}
		expect(gateway).toContain('getWhatsAppCommandConnectClient')
		expect(gateway).toContain('getWhatsAppQueryConnectClient')
		expect(readClient).toContain('WhatsAppOperationalQueryService')
		expect(readGateway).toContain('WhatsAppOperationalQueryV1Schema')
		expect(realtimeClient).toContain('WhatsAppOperationalRealtimeService')
		expect(replayGateway).toContain('WhatsAppOperationalReplayRequestV1Schema')
		expect(accounts).toContain("'whatsapp.operational.query.v1'")
		expect(accounts).toContain("'whatsapp.operational.realtime.v1'")
		expect(presentation).not.toMatch(/queries\/|api\/|connect\/|fetch\(/)
		expect(readPresentation).not.toMatch(/queries\/|api\/|connect\/|fetch\(/)
		expect(replayPresentation).not.toMatch(/queries\/|api\/|connect\/|fetch\(/)
		expect(appLayout).toContain('WhatsAppOperationalRoute')
		expect(appLayout).toContain("'whatsapp.command.v1'")
		expect(appLayout).toContain("'whatsapp.operational.query.v1'")
		expect(appLayout).toContain("'whatsapp.operational.realtime.v1'")
		expect(appLayout).toContain("'whatsapp.operational.realtime.shared.v1'")
		expect(compiledAdapters).toContain("'whatsapp-integration'")
	})
})

function read(relativePath: string): string {
	return readFileSync(new URL(relativePath, import.meta.url), 'utf8')
}
