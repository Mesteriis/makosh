import type {
	MailCompositionModeV1,
	MailDraftV1,
	MailSignatureV1,
	MailTemplateV1,
} from '../../../gen/makosh/mail/composition/v1/client_pb'

export type MailCompositionStatus = 'blocked' | 'loading' | 'ready' | 'empty' | 'error'
export type MailCompositionBusyAction = 'refresh' | 'draft' | 'template' | 'signature' | 'preview' | null

export type MailCompositionOption = {
	id: string
	label: string
	detail: string
}

export type MailDraftEditorModel = {
	draftId: string
	revision: string
	mode: MailCompositionModeV1
	providerConversationId: string
	inReplyToProviderMessageId: string
	toRecipients: string
	ccRecipients: string
	bccRecipients: string
	subject: string
	textBody: string
	templateId: string
	signatureId: string
}

export type MailTemplateEditorModel = {
	templateId: string
	revision: string
	name: string
	subjectTemplate: string
	textBodyTemplate: string
	variables: string
	locale: string
	previewValues: string
	previewSummary: string
}

export type MailSignatureEditorModel = {
	signatureId: string
	revision: string
	name: string
	textBody: string
	isDefault: boolean
}

export type MailDraftEditorPatch = Partial<Omit<MailDraftEditorModel, 'revision'>>
export type MailTemplateEditorPatch = Partial<Omit<MailTemplateEditorModel, 'revision'>>
export type MailSignatureEditorPatch = Partial<Omit<MailSignatureEditorModel, 'revision'>>

export type MailCompositionModel = {
	canMutate: boolean
	canQuery: boolean
	status: MailCompositionStatus
	statusMessage: string
	notice: string
	busyAction: MailCompositionBusyAction
	connections: readonly MailCompositionOption[]
	selectedConnectionId: string
	drafts: readonly MailCompositionOption[]
	templates: readonly MailCompositionOption[]
	signatures: readonly MailCompositionOption[]
	draft: MailDraftEditorModel
	template: MailTemplateEditorModel
	signature: MailSignatureEditorModel
}

export function buildDraftOptions(
	drafts: readonly MailDraftV1[],
): readonly MailCompositionOption[] {
	return drafts.map((draft) => ({
		id: draft.draftId,
		label: draft.subject.trim() || '(No subject)',
		detail: `${draft.toRecipient.length + draft.ccRecipient.length + draft.bccRecipient.length} recipients · r${draft.revision}`,
	}))
}

export function buildTemplateOptions(
	templates: readonly MailTemplateV1[],
): readonly MailCompositionOption[] {
	return templates.map((template) => ({
		id: template.templateId,
		label: template.name,
		detail: `${template.variable.length} variables · r${template.revision}`,
	}))
}

export function buildSignatureOptions(
	signatures: readonly MailSignatureV1[],
): readonly MailCompositionOption[] {
	return signatures.map((signature) => ({
		id: signature.signatureId,
		label: signature.name,
		detail: `${signature.isDefault ? 'Default · ' : ''}r${signature.revision}`,
	}))
}

export function splitEditorLines(value: string): string[] {
	return value
		.split(/[\n,;]/)
		.map((entry) => entry.trim())
		.filter(Boolean)
}

export function parseTemplateValues(value: string): Readonly<Record<string, string>> {
	const result: Record<string, string> = {}
	for (const line of value.split('\n')) {
		const separator = line.indexOf('=')
		if (separator <= 0) continue
		const name = line.slice(0, separator).trim()
		if (!name) continue
		result[name] = line.slice(separator + 1)
	}
	return result
}
