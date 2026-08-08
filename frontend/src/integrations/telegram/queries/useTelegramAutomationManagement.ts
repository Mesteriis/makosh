import { computed, ref } from 'vue'

import type {
	AutomationPolicyV1,
	AutomationTemplateV1,
} from '../../../gen/makosh/telegram/automation/v1/automation_pb'
import {
	previewTelegramAutomationPolicy,
	upsertTelegramAutomationPolicy,
	upsertTelegramAutomationTemplate,
} from '../api/telegramAutomationCommandGateway'
import {
	listTelegramAutomationPolicies,
	listTelegramAutomationTemplates,
} from '../api/telegramAutomationQueryGateway'
import {
	automationDigestHex,
	parseAutomationIdentifiers,
	parseAutomationVariables,
	type TelegramAutomationModel,
} from '../presentation/telegramAutomationModel'

export function useTelegramAutomationManagement(input: {
	canCommand: () => boolean
	canQuery: () => boolean
}) {
	const templates = ref<readonly AutomationTemplateV1[]>([])
	const policies = ref<readonly AutomationPolicyV1[]>([])
	const pending = ref(false)
	const statusMessage = ref('')
	const templateId = ref('')
	const templateName = ref('')
	const templateBody = ref('')
	const templateVariables = ref('')
	const templateRevision = ref('0')
	const policyId = ref('')
	const policyTemplateId = ref('')
	const policyName = ref('')
	const policyEnabled = ref(true)
	const policyAccountId = ref('')
	const policyChatIds = ref('')
	const policyExpiresAt = ref('')
	const policyRevision = ref('0')
	const previewPolicyId = ref('')
	const previewAccountId = ref('')
	const previewChatId = ref('')
	const previewVariables = ref('')
	const previewRenderedText = ref('')
	const previewRenderedSha256 = ref('')

	const model = computed<TelegramAutomationModel>(() => ({
		canCommand: input.canCommand(),
		canQuery: input.canQuery(),
		pending: pending.value,
		statusMessage: statusMessage.value,
		templates: templates.value.map((template) => ({
			id: template.templateId,
			name: template.name,
			revision: template.revision.toString(),
		})),
		policies: policies.value.map((policy) => ({
			id: policy.policyId,
			name: policy.name,
			accountId: policy.accountId,
			enabled: policy.enabled,
			revision: policy.revision.toString(),
		})),
		template: {
			id: templateId.value,
			name: templateName.value,
			body: templateBody.value,
			requiredVariables: templateVariables.value,
			revision: templateRevision.value,
		},
		policy: {
			id: policyId.value,
			templateId: policyTemplateId.value,
			name: policyName.value,
			enabled: policyEnabled.value,
			accountId: policyAccountId.value,
			providerChatIds: policyChatIds.value,
			expiresAtUnixSeconds: policyExpiresAt.value,
			revision: policyRevision.value,
		},
		preview: {
			policyId: previewPolicyId.value,
			accountId: previewAccountId.value,
			providerChatId: previewChatId.value,
			variables: previewVariables.value,
			renderedText: previewRenderedText.value,
			renderedSha256: previewRenderedSha256.value,
		},
	}))

	async function refresh(): Promise<void> {
		if (!input.canQuery()) {
			statusMessage.value = 'Telegram automation query capability is not admitted.'
			return
		}
		await run(async () => {
			const [templatePage, policyPage] = await Promise.all([
				listTelegramAutomationTemplates(),
				listTelegramAutomationPolicies(),
			])
			templates.value = templatePage.items
			policies.value = policyPage.items
			statusMessage.value = `Loaded ${templates.value.length} templates and ${policies.value.length} policies.`
		})
	}

	async function saveTemplate(): Promise<void> {
		if (!input.canCommand()) {
			statusMessage.value = 'Telegram automation command capability is not admitted.'
			return
		}
		await run(async () => {
			const template = await upsertTelegramAutomationTemplate({
				mutationId: operationId('template'),
				expectedRevision: revision(templateRevision.value),
				templateId: templateId.value,
				name: templateName.value,
				bodyTemplate: templateBody.value,
				requiredVariables: parseAutomationIdentifiers(templateVariables.value),
			})
			replaceById(templates, template, (item) => item.templateId)
			selectTemplate(template.templateId)
			statusMessage.value = `Template ${template.templateId} saved at revision ${template.revision}.`
		})
	}

	async function savePolicy(): Promise<void> {
		if (!input.canCommand()) {
			statusMessage.value = 'Telegram automation command capability is not admitted.'
			return
		}
		await run(async () => {
			const policy = await upsertTelegramAutomationPolicy({
				mutationId: operationId('policy'),
				expectedRevision: revision(policyRevision.value),
				policyId: policyId.value,
				templateId: policyTemplateId.value,
				name: policyName.value,
				enabled: policyEnabled.value,
				accountId: policyAccountId.value,
				providerChatIds: parseAutomationIdentifiers(policyChatIds.value),
				expiresAtUnixSeconds: optionalTimestamp(policyExpiresAt.value),
			})
			replaceById(policies, policy, (item) => item.policyId)
			selectPolicy(policy.policyId)
			statusMessage.value = `Policy ${policy.policyId} saved at revision ${policy.revision}.`
		})
	}

	async function preview(): Promise<void> {
		if (!input.canCommand()) {
			statusMessage.value = 'Telegram automation command capability is not admitted.'
			return
		}
		await run(async () => {
			const receipt = await previewTelegramAutomationPolicy({
				previewId: operationId('preview'),
				policyId: previewPolicyId.value,
				accountId: previewAccountId.value,
				providerChatId: previewChatId.value,
				variables: parseAutomationVariables(previewVariables.value),
			})
			previewRenderedText.value = receipt.renderedText
			previewRenderedSha256.value = automationDigestHex(receipt.renderedSha256)
			statusMessage.value = `Preview ${receipt.previewId} persisted without provider delivery.`
		})
	}

	function selectTemplate(id: string): void {
		const template = templates.value.find((item) => item.templateId === id)
		if (!template) return
		templateId.value = template.templateId
		templateName.value = template.name
		templateBody.value = template.bodyTemplate
		templateVariables.value = template.requiredVariables.join(', ')
		templateRevision.value = template.revision.toString()
	}

	function selectPolicy(id: string): void {
		const policy = policies.value.find((item) => item.policyId === id)
		if (!policy) return
		policyId.value = policy.policyId
		policyTemplateId.value = policy.templateId
		policyName.value = policy.name
		policyEnabled.value = policy.enabled
		policyAccountId.value = policy.accountId
		policyChatIds.value = policy.providerChatIds.join(', ')
		policyExpiresAt.value = policy.expiresAtUnixSeconds?.toString() ?? ''
		policyRevision.value = policy.revision.toString()
		previewPolicyId.value = policy.policyId
		previewAccountId.value = policy.accountId
	}

	function newTemplate(): void {
		templateId.value = ''
		templateName.value = ''
		templateBody.value = ''
		templateVariables.value = ''
		templateRevision.value = '0'
	}

	function newPolicy(): void {
		policyId.value = ''
		policyTemplateId.value = templateId.value
		policyName.value = ''
		policyEnabled.value = true
		policyAccountId.value = ''
		policyChatIds.value = ''
		policyExpiresAt.value = ''
		policyRevision.value = '0'
	}

	async function run(action: () => Promise<void>): Promise<void> {
		pending.value = true
		statusMessage.value = ''
		try {
			await action()
		} catch (error) {
			statusMessage.value = error instanceof Error ? error.message : 'Telegram automation failed.'
		} finally {
			pending.value = false
		}
	}

	return {
		model,
		newPolicy,
		newTemplate,
		preview,
		refresh,
		savePolicy,
		saveTemplate,
		selectPolicy,
		selectTemplate,
		updatePolicyAccountId: (value: string) => { policyAccountId.value = value },
		updatePolicyChatIds: (value: string) => { policyChatIds.value = value },
		updatePolicyEnabled: (value: boolean) => { policyEnabled.value = value },
		updatePolicyExpiresAt: (value: string) => { policyExpiresAt.value = value },
		updatePolicyId: (value: string) => { policyId.value = value },
		updatePolicyName: (value: string) => { policyName.value = value },
		updatePolicyTemplateId: (value: string) => { policyTemplateId.value = value },
		updatePreviewAccountId: (value: string) => { previewAccountId.value = value },
		updatePreviewChatId: (value: string) => { previewChatId.value = value },
		updatePreviewPolicyId: (value: string) => { previewPolicyId.value = value },
		updatePreviewVariables: (value: string) => { previewVariables.value = value },
		updateTemplateBody: (value: string) => { templateBody.value = value },
		updateTemplateId: (value: string) => { templateId.value = value },
		updateTemplateName: (value: string) => { templateName.value = value },
		updateTemplateVariables: (value: string) => { templateVariables.value = value },
	}
}

function operationId(kind: string): string {
	if (!globalThis.crypto?.randomUUID) {
		throw new Error('Secure Telegram automation operation IDs are unavailable')
	}
	return `telegram-automation-${kind}:${globalThis.crypto.randomUUID()}`
}

function revision(value: string): bigint {
	try {
		const parsed = BigInt(value)
		if (parsed < 0n) throw new RangeError()
		return parsed
	} catch {
		throw new RangeError('Telegram automation revision is invalid')
	}
}

function optionalTimestamp(value: string): bigint | undefined {
	if (!value.trim()) return undefined
	const timestamp = revision(value)
	if (timestamp === 0n) {
		throw new RangeError('Telegram automation expiry must be positive')
	}
	return timestamp
}

function replaceById<T>(
	target: { value: readonly T[] },
	value: T,
	id: (item: T) => string,
): void {
	target.value = [...target.value.filter((item) => id(item) !== id(value)), value]
		.sort((left, right) => id(left).localeCompare(id(right)))
}
