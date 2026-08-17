import { readFileSync } from 'node:fs'

import { describe, expect, it } from 'vitest'

describe('Telegram operational active route boundary', () => {
	it('stays integration-owned and uses only the generated Telegram contract', () => {
		const route = read('../views/TelegramOperationalRoute.vue')
		const controller = read('../queries/useTelegramOperationalPage.ts')
		const accountController = read('../queries/useTelegramAccountAccess.ts')
		const gateway = read('../api/telegramOperationalGateway.ts')
		const authorizationGateway = read('../api/telegramAuthorizationGateway.ts')
		const lifecycleGateway = read('../api/telegramLifecycleGateway.ts')
		const reconfigurationClient = read('../api/telegramReconfigurationClient.ts')
		const discoveryGateway = read('../api/telegramDiscoveryGateway.ts')
		const inspectorGateway = read('../api/telegramMessageInspectorGateway.ts')
		const commandGateways = [
			read('../api/telegramChatCommandGateway.ts'),
			read('../api/telegramMediaCommandGateway.ts'),
			read('../api/telegramMessageCommandGateway.ts'),
			read('../api/telegramTopicCommandGateway.ts'),
		]
		const commandControllers = [
			read('../queries/useTelegramChatCommands.ts'),
			read('../queries/useTelegramMediaCommands.ts'),
			read('../queries/useTelegramMessageCommands.ts'),
			read('../queries/useTelegramTopicCommands.ts'),
		]
		const presentation = read('../presentation/TelegramOperationalPage.vue')
		const accountPresentation = read('../presentation/TelegramAccountAccessPanel.vue')
		const cloudPasswordPresentation = read('../presentation/TelegramCloudPasswordForm.vue')
		const newMessagePresentation = read('../presentation/TelegramNewMessageDialog.vue')
		const toolbarPresentation = read('../presentation/TelegramWorkspaceToolbar.vue')
		const discoveryPresentation = read('../presentation/TelegramDiscoveryPanel.vue')
		const commandWorkbench = read('../presentation/TelegramCommandWorkbench.vue')
		const messageInspector = read('../presentation/TelegramMessageInspector.vue')
		const operationRetry = read('../presentation/TelegramOperationRetryPanel.vue')
		const appLayout = read('../../../app/layout/AppLayoutRoot.vue')
		const compiledAdapters = read('../../../app/client-surfaces/compiledClientSurfaceAdapters.ts')
		const capabilityComposition = read('../../../app/client-surfaces/clientModuleCapabilities.ts')

		for (const source of [
			route,
			controller,
			accountController,
			gateway,
			authorizationGateway,
			lifecycleGateway,
			reconfigurationClient,
			discoveryGateway,
			inspectorGateway,
			...commandGateways,
			...commandControllers,
			presentation,
			accountPresentation,
			cloudPasswordPresentation,
			newMessagePresentation,
			toolbarPresentation,
			discoveryPresentation,
			commandWorkbench,
			messageInspector,
			operationRetry,
		]) {
			expect(source).not.toMatch(/\/api\/v1\//)
			expect(source).not.toMatch(/domains\/communications/)
			expect(source).not.toMatch(/integrations\/(mail|whatsapp|zulip)/)
		}
		expect(gateway).toContain('getTelegramOperationalConnectClient')
		expect(controller).not.toContain('setInterval')
		expect(controller).not.toContain('replayTelegramRealtime')
		expect(authorizationGateway).toContain('getTelegramAuthorizationConnectClient')
		expect(lifecycleGateway).toContain('getTelegramLifecycleConnectClient')
		expect(lifecycleGateway).toContain('getTelegramReconfigurationConnectClient')
		expect(lifecycleGateway).toContain("case: 'begin'")
		expect(lifecycleGateway).not.toMatch(/startAccount|stopAccount|restartAccount/)
		expect(reconfigurationClient).toContain('TelegramReconfigurationService')
		expect(discoveryGateway).toContain('getTelegramOperationalConnectClient')
		expect(discoveryGateway).not.toMatch(/as never|Record<|unknown as/)
		expect(inspectorGateway).not.toMatch(/as never|Record<|unknown as|Object\.keys/)
		expect(commandGateways.join('\n')).not.toMatch(/as never|Record<|unknown as/)
		expect(commandWorkbench).not.toMatch(/emit\('action'|emit\('update'/)
		for (const commandCase of [
			'sendMedia',
			'downloadFile',
			'reply',
			'forward',
			'edit',
			'delete',
			'restoreVisibility',
			'reaction',
			'pin',
			'markUnread',
			'archive',
			'mute',
			'join',
			'leave',
			'addChatToFolder',
			'removeChatFromFolder',
			'reassignChatFolders',
			'searchMessages',
			'listParticipants',
			'listTopics',
			'createTopic',
			'setTopicClosed',
		]) {
			expect(commandGateways.join('\n')).toContain(`case: '${commandCase}'`)
		}
		expect(presentation).not.toMatch(/queries\/|api\/|connect\/|fetch\(/)
		expect(accountPresentation).not.toMatch(/queries\/|api\/|connect\/|fetch\(/)
		expect(cloudPasswordPresentation).not.toMatch(/queries\/|api\/|connect\/|fetch\(/)
		expect(newMessagePresentation).not.toMatch(/queries\/|api\/|connect\/|fetch\(/)
		expect(discoveryPresentation).not.toMatch(/queries\/|api\/|connect\/|fetch\(/)
		expect(messageInspector).not.toMatch(/queries\/|api\/|connect\/|fetch\(/)
		expect(operationRetry).not.toMatch(/queries\/|api\/|connect\/|fetch\(/)
		expect(appLayout).toContain('TelegramOperationalRoute')
		expect(appLayout).toContain("'telegram.authorization.v1'")
		expect(appLayout).toContain("'telegram.lifecycle.v1'")
		expect(appLayout).toContain("'telegram.reconfiguration.v1'")
		expect(appLayout).toContain("'telegram.query.v1'")
		expect(appLayout).toContain("'telegram.command.v1'")
		expect(appLayout).toContain("'telegram.operational.realtime.shared.v1'")
		expect(capabilityComposition).toContain('module.sectionsEnabled')
		expect(compiledAdapters).toContain("'telegram-integration'")
		expect(toolbarPresentation).toContain("@click=\"emit('compose')\"")
		expect(presentation).toContain('<TelegramNewMessageDialog')
		expect(newMessagePresentation).toContain('<ChatInput')
		expect(newMessagePresentation).toContain("emit('selectChat'")
		expect(accountPresentation).toContain("model.authorizationState === 'waiting_password'")
		const laneReconciliation = route.slice(
			route.indexOf('async function reconcileOperationalLane'),
			route.indexOf('async function refreshAccounts'),
		)
		expect(laneReconciliation.indexOf('surface.updateAccountId(accountModel.selectedAccountId)'))
			.toBeLessThan(laneReconciliation.indexOf('if (nextLaneStateKey === operationalLaneStateKey) return'))
	})
})

function read(relativePath: string): string {
	return readFileSync(new URL(relativePath, import.meta.url), 'utf8')
}
