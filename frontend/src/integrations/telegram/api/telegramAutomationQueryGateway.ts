import type {
	AutomationPolicyV1,
	AutomationPreviewReceiptV1,
	AutomationTemplateV1,
} from '../../../gen/makosh/telegram/automation/v1/automation_pb'
import { getTelegramAutomationQueryClient } from './telegramAutomationQueryClient'
import { telegramAutomationFailure } from './telegramAutomationFailure'

const PAGE_LIMIT = 50

export type TelegramAutomationTemplatePage = {
	items: readonly AutomationTemplateV1[]
	nextAfterTemplateId: string
}

export type TelegramAutomationPolicyPage = {
	items: readonly AutomationPolicyV1[]
	nextAfterPolicyId: string
}

export async function listTelegramAutomationTemplates(
	afterTemplateId = '',
): Promise<TelegramAutomationTemplatePage> {
	const response = await getTelegramAutomationQueryClient().query({
		request: {
			case: 'listTemplates',
			value: { limit: PAGE_LIMIT, afterTemplateId: optionalIdentifier(afterTemplateId) },
		},
	})
	if (response.response.case === 'failure') {
		throw telegramAutomationFailure(response.response.value)
	}
	if (response.response.case !== 'templates') {
		throw new Error('Telegram automation template list is unavailable')
	}
	return response.response.value
}

export async function getTelegramAutomationTemplate(
	templateId: string,
): Promise<AutomationTemplateV1> {
	const response = await getTelegramAutomationQueryClient().query({
		request: {
			case: 'getTemplate',
			value: { templateId: requireIdentifier('template ID', templateId) },
		},
	})
	if (response.response.case === 'failure') {
		throw telegramAutomationFailure(response.response.value)
	}
	if (response.response.case !== 'template') {
		throw new Error('Telegram automation template is unavailable')
	}
	return response.response.value
}

export async function listTelegramAutomationPolicies(
	afterPolicyId = '',
): Promise<TelegramAutomationPolicyPage> {
	const response = await getTelegramAutomationQueryClient().query({
		request: {
			case: 'listPolicies',
			value: { limit: PAGE_LIMIT, afterPolicyId: optionalIdentifier(afterPolicyId) },
		},
	})
	if (response.response.case === 'failure') {
		throw telegramAutomationFailure(response.response.value)
	}
	if (response.response.case !== 'policies') {
		throw new Error('Telegram automation policy list is unavailable')
	}
	return response.response.value
}

export async function getTelegramAutomationPolicy(
	policyId: string,
): Promise<AutomationPolicyV1> {
	const response = await getTelegramAutomationQueryClient().query({
		request: {
			case: 'getPolicy',
			value: { policyId: requireIdentifier('policy ID', policyId) },
		},
	})
	if (response.response.case === 'failure') {
		throw telegramAutomationFailure(response.response.value)
	}
	if (response.response.case !== 'policy') {
		throw new Error('Telegram automation policy is unavailable')
	}
	return response.response.value
}

export async function getTelegramAutomationPreviewReceipt(
	previewId: string,
): Promise<AutomationPreviewReceiptV1> {
	const response = await getTelegramAutomationQueryClient().query({
		request: {
			case: 'getPreviewReceipt',
			value: { previewId: requireIdentifier('preview ID', previewId) },
		},
	})
	if (response.response.case === 'failure') {
		throw telegramAutomationFailure(response.response.value)
	}
	if (response.response.case !== 'previewReceipt') {
		throw new Error('Telegram automation preview receipt is unavailable')
	}
	return response.response.value
}

function optionalIdentifier(value: string): string {
	const normalized = value.trim()
	if (normalized) {
		requireIdentifier('page cursor', normalized)
	}
	return normalized
}

function requireIdentifier(label: string, value: string): string {
	const normalized = value.trim()
	if (!normalized || normalized.length > 256 || !/^[A-Za-z0-9._:@-]+$/.test(normalized)) {
		throw new RangeError(`Telegram automation ${label} is invalid`)
	}
	return normalized
}
