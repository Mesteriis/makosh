import { readFileSync } from 'node:fs'

import { describe, expect, it } from 'vitest'

describe('Zulip operational active route boundary', () => {
	it('keeps provider commands in the Zulip integration', () => {
		const route = read('../views/ZulipOperationalRoute.vue')
		const controller = read('../queries/useZulipOperationalPage.ts')
		const readController = read('../queries/useZulipOperationalRead.ts')
		const replayController = read('../queries/useZulipOperationalReplay.ts')
		const accounts = read('../queries/zulipOperationalAccounts.ts')
		const gateway = read('../api/zulipOperationalGateway.ts')
		const readClient = read('../api/zulipOperationalReadClient.ts')
		const readGateway = read('../api/zulipOperationalReadGateway.ts')
		const realtimeClient = read('../api/zulipOperationalRealtimeClient.ts')
		const replayGateway = read('../api/zulipOperationalReplayGateway.ts')
		const presentation = read('../presentation/ZulipOperationalPage.vue')
		const readPresentation = read('../presentation/ZulipOperationalReadPanel.vue')
		const messagePresentation = read('../presentation/ZulipMessageRow.vue')
		const replayPresentation = read('../presentation/ZulipOperationalReplayPanel.vue')
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
			messagePresentation,
			replayPresentation,
		]) {
			expect(source).not.toMatch(/\/api\/v1\//)
			expect(source).not.toMatch(/domains\/communications/)
			expect(source).not.toMatch(/integrations\/(mail|telegram|whatsapp)/)
			expect(source).not.toMatch(/invoke\(|@tauri-apps/)
		}
		expect(gateway).toContain('getZulipCommandConnectClient')
		expect(gateway).toContain('getZulipQueryConnectClient')
		expect(readClient).toContain('ZulipOperationalQueryService')
		expect(readGateway).toContain('ZulipOperationalQueryV1Schema')
		expect(realtimeClient).toContain('ZulipOperationalRealtimeService')
		expect(replayGateway).toContain('ZulipOperationalReplayRequestV1Schema')
		expect(accounts).toContain("'zulip.operational.query.v1'")
		expect(accounts).toContain("'zulip.operational.realtime.v1'")
		expect(presentation).not.toMatch(/queries\/|api\/|connect\/|fetch\(/)
		expect(readPresentation).not.toMatch(/queries\/|api\/|connect\/|fetch\(/)
		expect(messagePresentation).not.toMatch(/queries\/|api\/|connect\/|fetch\(/)
		expect(replayPresentation).not.toMatch(/queries\/|api\/|connect\/|fetch\(/)
		expect(appLayout).toContain('ZulipOperationalRoute')
		expect(appLayout).toContain("'zulip.command.v1'")
		expect(appLayout).toContain("'zulip.operational.query.v1'")
		expect(appLayout).toContain("'zulip.operational.realtime.v1'")
		expect(appLayout).toContain("'zulip.operational.realtime.shared.v1'")
		expect(compiledAdapters).toContain("'zulip-integration'")
	})
})

function read(relativePath: string): string {
	return readFileSync(new URL(relativePath, import.meta.url), 'utf8')
}
