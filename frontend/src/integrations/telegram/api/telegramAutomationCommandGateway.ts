import type {
	AutomationPolicyV1,
	AutomationPreviewReceiptV1,
	AutomationTemplateV1,
} from '../../../gen/makosh/telegram/automation/v1/automation_pb'
import { getTelegramAutomationCommandClient } from './telegramAutomationCommandClient'
import { telegramAutomationFailure } from './telegramAutomationFailure'

export type UpsertTelegramAutomationTemplateInput = {
	mutationId: string
	expectedRevision: bigint
	templateId: string
	name: string
	bodyTemplate: string
	requiredVariables: readonly string[]
}

export type UpsertTelegramAutomationPolicyInput = {
	mutationId: string
	expectedRevision: bigint
	policyId: string
	templateId: string
	name: string
	enabled: boolean
	accountId: string
	providerChatIds: readonly string[]
	expiresAtUnixSeconds?: bigint
}

export type PreviewTelegramAutomationPolicyInput = {
	previewId: string
	policyId: string
	accountId: string
	providerChatId: string
	variables: readonly { name: string; value: string }[]
}

export async function upsertTelegramAutomationTemplate(
	input: UpsertTelegramAutomationTemplateInput,
): Promise<AutomationTemplateV1> {
	const response = await getTelegramAutomationCommandClient().execute({
		command: {
			case: 'upsertTemplate',
			value: {
				mutationId: requireIdentifier('mutation ID', input.mutationId),
				expectedRevision: input.expectedRevision,
				templateId: requireIdentifier('template ID', input.templateId),
				name: requireText('template name', input.name, 512),
				bodyTemplate: requireText('template body', input.bodyTemplate, 16 * 1024),
				requiredVariables: requireVariables(input.requiredVariables),
			},
		},
	})
	if (response.response.case === 'failure') {
		throw telegramAutomationFailure(response.response.value)
	}
	if (response.response.case !== 'template') {
		throw new Error('Telegram automation template result is unavailable')
	}
	return response.response.value
}

export async function upsertTelegramAutomationPolicy(
	input: UpsertTelegramAutomationPolicyInput,
): Promise<AutomationPolicyV1> {
	const providerChatIds = input.providerChatIds.map((chatId) =>
		requireIdentifier('chat ID', chatId),
	)
	if (
		providerChatIds.length === 0
		|| providerChatIds.length > 128
		|| new Set(providerChatIds).size !== providerChatIds.length
	) {
		throw new RangeError('Telegram automation policy requires 1-128 unique chats')
	}
	const response = await getTelegramAutomationCommandClient().execute({
		command: {
			case: 'upsertPolicy',
			value: {
				mutationId: requireIdentifier('mutation ID', input.mutationId),
				expectedRevision: input.expectedRevision,
				policyId: requireIdentifier('policy ID', input.policyId),
				templateId: requireIdentifier('template ID', input.templateId),
				name: requireText('policy name', input.name, 512),
				enabled: input.enabled,
				accountId: requireIdentifier('account ID', input.accountId),
				providerChatIds,
				expiresAtUnixSeconds: input.expiresAtUnixSeconds,
			},
		},
	})
	if (response.response.case === 'failure') {
		throw telegramAutomationFailure(response.response.value)
	}
	if (response.response.case !== 'policy') {
		throw new Error('Telegram automation policy result is unavailable')
	}
	return response.response.value
}

export async function previewTelegramAutomationPolicy(
	input: PreviewTelegramAutomationPolicyInput,
): Promise<AutomationPreviewReceiptV1> {
	const variables = input.variables.map(({ name, value }) => ({
		name: requireVariableName(name),
		value: requireText('variable value', value, 4 * 1024),
	}))
	if (variables.length > 32 || new Set(variables.map(({ name }) => name)).size !== variables.length) {
		throw new RangeError('Telegram automation preview variables must be unique')
	}
	const response = await getTelegramAutomationCommandClient().execute({
		command: {
			case: 'previewPolicy',
			value: {
				previewId: requireIdentifier('preview ID', input.previewId),
				policyId: requireIdentifier('policy ID', input.policyId),
				accountId: requireIdentifier('account ID', input.accountId),
				providerChatId: requireIdentifier('chat ID', input.providerChatId),
				variables,
			},
		},
	})
	if (response.response.case === 'failure') {
		throw telegramAutomationFailure(response.response.value)
	}
	if (response.response.case !== 'preview') {
		throw new Error('Telegram automation preview result is unavailable')
	}
	return response.response.value
}

function requireVariables(values: readonly string[]): string[] {
	const variables = values.map(requireVariableName)
	if (variables.length > 32 || new Set(variables).size !== variables.length) {
		throw new RangeError('Telegram automation variables must be unique')
	}
	return variables
}

function requireVariableName(value: string): string {
	const normalized = value.trim()
	if (!normalized || normalized.length > 64 || !/^[A-Za-z0-9_]+$/.test(normalized)) {
		throw new RangeError('Telegram automation variable name is invalid')
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

function requireText(label: string, value: string, maxLength: number): string {
	const normalized = value.trim()
	if (!normalized || normalized.length > maxLength) {
		throw new RangeError(`Telegram automation ${label} is invalid`)
	}
	return normalized
}
