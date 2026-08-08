import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'

import {
	previewTelegramAutomationPolicy,
	upsertTelegramAutomationTemplate,
} from '../api/telegramAutomationCommandGateway'
import {
	listTelegramAutomationPolicies,
	listTelegramAutomationTemplates,
} from '../api/telegramAutomationQueryGateway'
import { useTelegramAutomationManagement } from './useTelegramAutomationManagement'

vi.mock('../api/telegramAutomationCommandGateway', () => ({
	previewTelegramAutomationPolicy: vi.fn(),
	upsertTelegramAutomationPolicy: vi.fn(),
	upsertTelegramAutomationTemplate: vi.fn(),
}))
vi.mock('../api/telegramAutomationQueryGateway', () => ({
	listTelegramAutomationPolicies: vi.fn(),
	listTelegramAutomationTemplates: vi.fn(),
}))

describe('Telegram automation management controller', () => {
	beforeEach(() => {
		vi.stubGlobal('crypto', { randomUUID: () => 'operation-uuid' })
		vi.mocked(listTelegramAutomationTemplates).mockResolvedValue({
			items: [],
			nextAfterTemplateId: '',
		})
		vi.mocked(listTelegramAutomationPolicies).mockResolvedValue({
			items: [],
			nextAfterPolicyId: '',
		})
	})

	afterEach(() => {
		vi.unstubAllGlobals()
		vi.clearAllMocks()
	})

	it('loads query projections and sends exact template and preview intents', async () => {
		const controller = useTelegramAutomationManagement({
			canCommand: () => true,
			canQuery: () => true,
		})
		await controller.refresh()
		expect(controller.model.value.statusMessage).toContain('Loaded 0 templates')

		vi.mocked(upsertTelegramAutomationTemplate).mockResolvedValue({
			templateId: 'template-1',
			name: 'Greeting',
			bodyTemplate: 'Hello {{name}}',
			requiredVariables: ['name'],
			revision: 1n,
			createdAtUnixSeconds: 1n,
			updatedAtUnixSeconds: 1n,
			$typeName: 'makosh.telegram.automation.v1.AutomationTemplateV1',
		})
		controller.updateTemplateId('template-1')
		controller.updateTemplateName('Greeting')
		controller.updateTemplateBody('Hello {{name}}')
		controller.updateTemplateVariables('name')
		await controller.saveTemplate()
		expect(upsertTelegramAutomationTemplate).toHaveBeenCalledWith({
			mutationId: 'telegram-automation-template:operation-uuid',
			expectedRevision: 0n,
			templateId: 'template-1',
			name: 'Greeting',
			bodyTemplate: 'Hello {{name}}',
			requiredVariables: ['name'],
		})

		vi.mocked(previewTelegramAutomationPolicy).mockResolvedValue({
			previewId: 'preview-1',
			policyId: 'policy-1',
			policyRevision: 1n,
			templateId: 'template-1',
			templateRevision: 1n,
			accountId: 'account-1',
			providerChatId: 'chat-1',
			renderedText: 'Hello Ada',
			renderedSha256: new Uint8Array([1, 2]),
			createdAtUnixSeconds: 1n,
			$typeName: 'makosh.telegram.automation.v1.AutomationPreviewReceiptV1',
		})
		controller.updatePreviewPolicyId('policy-1')
		controller.updatePreviewAccountId('account-1')
		controller.updatePreviewChatId('chat-1')
		controller.updatePreviewVariables('name=Ada')
		await controller.preview()
		expect(controller.model.value.preview.renderedText).toBe('Hello Ada')
		expect(controller.model.value.preview.renderedSha256).toBe('0102')
	})

	it('fails closed when capabilities are unavailable', async () => {
		const controller = useTelegramAutomationManagement({
			canCommand: () => false,
			canQuery: () => false,
		})
		await controller.refresh()
		await controller.saveTemplate()

		expect(listTelegramAutomationTemplates).not.toHaveBeenCalled()
		expect(upsertTelegramAutomationTemplate).not.toHaveBeenCalled()
		expect(controller.model.value.statusMessage).toContain('not admitted')
	})
})
