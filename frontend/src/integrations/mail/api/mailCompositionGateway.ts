import { create } from '@bufbuild/protobuf'

import {
	DeleteMailDraftCommandV1Schema,
	DeleteMailSignatureCommandV1Schema,
	DeleteMailTemplateCommandV1Schema,
	GetMailDraftQueryV1Schema,
	ListMailDraftsQueryV1Schema,
	ListMailSignaturesQueryV1Schema,
	ListMailTemplatesQueryV1Schema,
	MailCompositionCommandV1Schema,
	MailCompositionModeV1,
	MailCompositionQueryV1Schema,
	MailDraftInputV1Schema,
	MailSignatureInputV1Schema,
	MailTemplateInputV1Schema,
	MailTemplateVariableValueV1Schema,
	PreviewMailTemplateQueryV1Schema,
	UpsertMailDraftCommandV1Schema,
	UpsertMailSignatureCommandV1Schema,
	UpsertMailTemplateCommandV1Schema,
	type MailCompositionMutationReceiptV1,
	type MailCompositionCommandV1,
	type MailCompositionQueryV1,
	type MailDraftPageV1,
	type MailDraftV1,
	type MailSignaturePageV1,
	type MailTemplatePageV1,
	type MailTemplatePreviewV1,
} from '../../../gen/makosh/mail/composition/v1/client_pb'
import { getMailCompositionCommandConnectClient } from './mailCompositionCommandClient'
import { getMailCompositionQueryConnectClient } from './mailCompositionQueryClient'

const DEFAULT_PAGE_LIMIT = 100
const MAX_IDENTIFIER_BYTES = 512
const textEncoder = new TextEncoder()

export type MailDraftMutationInput = {
	connectionId: string
	draftId: string
	expectedRevision?: bigint
	mode: MailCompositionModeV1
	providerConversationId?: string
	inReplyToProviderMessageId?: string
	toRecipients: readonly string[]
	ccRecipients: readonly string[]
	bccRecipients: readonly string[]
	subject: string
	textBody: string
	templateId?: string
	signatureId?: string
}

export type MailTemplateMutationInput = {
	connectionId: string
	templateId: string
	expectedRevision?: bigint
	name: string
	subjectTemplate: string
	textBodyTemplate: string
	variables: readonly string[]
	locale?: string
}

export type MailSignatureMutationInput = {
	connectionId: string
	signatureId: string
	expectedRevision?: bigint
	name: string
	textBody: string
	isDefault: boolean
}

export async function listMailDrafts(
	connectionId: string,
	cursor?: string,
): Promise<MailDraftPageV1> {
	const response = await query({
		case: 'listDrafts',
		value: create(ListMailDraftsQueryV1Schema, {
			connectionId: identifier('connection ID', connectionId),
			cursor: optionalIdentifier('cursor', cursor),
			limit: DEFAULT_PAGE_LIMIT,
		}),
	})
	if (response.response.case !== 'drafts') throw new Error('Mail drafts response is unavailable')
	return response.response.value
}

export async function getMailDraft(
	connectionId: string,
	draftId: string,
): Promise<MailDraftV1 | null> {
	const response = await query({
		case: 'getDraft',
		value: create(GetMailDraftQueryV1Schema, {
			connectionId: identifier('connection ID', connectionId),
			draftId: identifier('draft ID', draftId),
		}),
	})
	if (response.response.case === 'notFound') return null
	if (response.response.case !== 'draft') throw new Error('Mail draft response is unavailable')
	return response.response.value
}

export async function listMailTemplates(
	connectionId: string,
	cursor?: string,
): Promise<MailTemplatePageV1> {
	const response = await query({
		case: 'listTemplates',
		value: create(ListMailTemplatesQueryV1Schema, {
			connectionId: identifier('connection ID', connectionId),
			cursor: optionalIdentifier('cursor', cursor),
			limit: DEFAULT_PAGE_LIMIT,
		}),
	})
	if (response.response.case !== 'templates') {
		throw new Error('Mail templates response is unavailable')
	}
	return response.response.value
}

export async function listMailSignatures(
	connectionId: string,
	cursor?: string,
): Promise<MailSignaturePageV1> {
	const response = await query({
		case: 'listSignatures',
		value: create(ListMailSignaturesQueryV1Schema, {
			connectionId: identifier('connection ID', connectionId),
			cursor: optionalIdentifier('cursor', cursor),
			limit: DEFAULT_PAGE_LIMIT,
		}),
	})
	if (response.response.case !== 'signatures') {
		throw new Error('Mail signatures response is unavailable')
	}
	return response.response.value
}

export async function previewMailTemplate(input: {
	connectionId: string
	templateId: string
	values: Readonly<Record<string, string>>
}): Promise<MailTemplatePreviewV1> {
	const values = Object.entries(input.values).map(([name, value]) =>
		create(MailTemplateVariableValueV1Schema, {
			name: identifier('template variable', name),
			value,
		}))
	const response = await query({
		case: 'previewTemplate',
		value: create(PreviewMailTemplateQueryV1Schema, {
			connectionId: identifier('connection ID', input.connectionId),
			templateId: identifier('template ID', input.templateId),
			value: values,
		}),
	})
	if (response.response.case !== 'templatePreview') {
		throw new Error('Mail template preview response is unavailable')
	}
	return response.response.value
}

export async function upsertMailDraft(
	input: MailDraftMutationInput,
): Promise<MailCompositionMutationReceiptV1> {
	const draft = create(MailDraftInputV1Schema, {
		connectionId: identifier('connection ID', input.connectionId),
		draftId: identifier('draft ID', input.draftId),
		mode: validMode(input.mode),
		providerConversationId: optionalIdentifier(
			'provider conversation ID',
			input.providerConversationId,
		),
		inReplyToProviderMessageId: optionalIdentifier(
			'in-reply-to provider message ID',
			input.inReplyToProviderMessageId,
		),
		toRecipient: recipients(input.toRecipients),
		ccRecipient: recipients(input.ccRecipients),
		bccRecipient: recipients(input.bccRecipients),
		subject: input.subject,
		textBody: input.textBody,
		templateId: optionalIdentifier('template ID', input.templateId),
		signatureId: optionalIdentifier('signature ID', input.signatureId),
	})
	return mutate({
		case: 'upsertDraft',
		value: create(UpsertMailDraftCommandV1Schema, {
			operationId: crypto.randomUUID(),
			draft,
			expectedRevision: input.expectedRevision,
		}),
	})
}

export async function deleteMailDraft(
	connectionId: string,
	draftId: string,
	expectedRevision: bigint,
): Promise<MailCompositionMutationReceiptV1> {
	return mutate({
		case: 'deleteDraft',
		value: create(DeleteMailDraftCommandV1Schema, {
			operationId: crypto.randomUUID(),
			connectionId: identifier('connection ID', connectionId),
			draftId: identifier('draft ID', draftId),
			expectedRevision: revision(expectedRevision),
		}),
	})
}

export async function upsertMailTemplate(
	input: MailTemplateMutationInput,
): Promise<MailCompositionMutationReceiptV1> {
	const template = create(MailTemplateInputV1Schema, {
		connectionId: identifier('connection ID', input.connectionId),
		templateId: identifier('template ID', input.templateId),
		name: requiredText('template name', input.name),
		subjectTemplate: input.subjectTemplate,
		textBodyTemplate: input.textBodyTemplate,
		variable: identifiers('template variable', input.variables),
		locale: optionalIdentifier('locale', input.locale),
	})
	return mutate({
		case: 'upsertTemplate',
		value: create(UpsertMailTemplateCommandV1Schema, {
			operationId: crypto.randomUUID(),
			template,
			expectedRevision: input.expectedRevision,
		}),
	})
}

export async function deleteMailTemplate(
	connectionId: string,
	templateId: string,
	expectedRevision: bigint,
): Promise<MailCompositionMutationReceiptV1> {
	return mutate({
		case: 'deleteTemplate',
		value: create(DeleteMailTemplateCommandV1Schema, {
			operationId: crypto.randomUUID(),
			connectionId: identifier('connection ID', connectionId),
			templateId: identifier('template ID', templateId),
			expectedRevision: revision(expectedRevision),
		}),
	})
}

export async function upsertMailSignature(
	input: MailSignatureMutationInput,
): Promise<MailCompositionMutationReceiptV1> {
	const signature = create(MailSignatureInputV1Schema, {
		connectionId: identifier('connection ID', input.connectionId),
		signatureId: identifier('signature ID', input.signatureId),
		name: requiredText('signature name', input.name),
		textBody: requiredText('signature body', input.textBody),
		isDefault: input.isDefault,
	})
	return mutate({
		case: 'upsertSignature',
		value: create(UpsertMailSignatureCommandV1Schema, {
			operationId: crypto.randomUUID(),
			signature,
			expectedRevision: input.expectedRevision,
		}),
	})
}

export async function deleteMailSignature(
	connectionId: string,
	signatureId: string,
	expectedRevision: bigint,
): Promise<MailCompositionMutationReceiptV1> {
	return mutate({
		case: 'deleteSignature',
		value: create(DeleteMailSignatureCommandV1Schema, {
			operationId: crypto.randomUUID(),
			connectionId: identifier('connection ID', connectionId),
			signatureId: identifier('signature ID', signatureId),
			expectedRevision: revision(expectedRevision),
		}),
	})
}

function query(queryInput: MailCompositionQueryV1['query']) {
	return getMailCompositionQueryConnectClient().query(
		create(MailCompositionQueryV1Schema, { query: queryInput }),
	)
}

function mutate(command: MailCompositionCommandV1['command']) {
	return getMailCompositionCommandConnectClient().mutate(
		create(MailCompositionCommandV1Schema, { command }),
	)
}

function identifier(label: string, value: string): string {
	const normalized = value.trim()
	if (
		!normalized
		|| textEncoder.encode(normalized).length > MAX_IDENTIFIER_BYTES
		|| hasControlCharacter(normalized)
	) throw new RangeError(`Mail ${label} is invalid`)
	return normalized
}

function optionalIdentifier(label: string, value?: string): string | undefined {
	const normalized = value?.trim()
	return normalized ? identifier(label, normalized) : undefined
}

function identifiers(label: string, values: readonly string[]): string[] {
	return values.map((value) => identifier(label, value))
}

function recipients(values: readonly string[]): string[] {
	return values.map((value) => value.trim()).filter(Boolean)
}

function requiredText(label: string, value: string): string {
	if (!value.trim()) throw new RangeError(`Mail ${label} is required`)
	return value
}

function revision(value: bigint): bigint {
	if (value <= 0n) throw new RangeError('Mail revision must be positive')
	return value
}

function validMode(value: MailCompositionModeV1): MailCompositionModeV1 {
	if (
		value < MailCompositionModeV1.MAIL_COMPOSITION_MODE_NEW
		|| value > MailCompositionModeV1.MAIL_COMPOSITION_MODE_REDIRECT
	) throw new RangeError('Mail composition mode is invalid')
	return value
}

function hasControlCharacter(value: string): boolean {
	for (let index = 0; index < value.length; index += 1) {
		const code = value.charCodeAt(index)
		if (code <= 0x1f || code === 0x7f) return true
	}
	return false
}
