import { computed, ref, shallowRef } from 'vue'
import {
	MailCompositionModeV1,
	type MailDraftV1,
	type MailTemplatePreviewV1,
} from '../../../gen/makosh/mail/composition/v1/client_pb'
import {
	deleteMailDraft,
	listMailDrafts,
	upsertMailDraft,
} from '../api/mailCompositionGateway'
import {
	buildDraftOptions,
	splitEditorLines,
	type MailDraftEditorModel,
	type MailDraftEditorPatch,
} from '../presentation/mailCompositionModel'

type DraftState = Omit<MailDraftEditorModel, 'revision'> & { revision?: bigint }

export function useMailDrafts(input: {
	canMutate: () => boolean
	connectionId: () => string
}) {
	const records = shallowRef<readonly MailDraftV1[]>([])
	const editor = ref<DraftState>(emptyDraft())
	const busy = ref(false)
	const notice = ref('')

	const options = computed(() => buildDraftOptions(records.value))
	const model = computed<MailDraftEditorModel>(() => ({
		...editor.value,
		revision: editor.value.revision?.toString() ?? '',
	}))
	const deliveryInput = computed(() => ({
		providerConversationId: editor.value.providerConversationId,
		toRecipients: splitEditorLines(editor.value.toRecipients),
		ccRecipients: splitEditorLines(editor.value.ccRecipients),
		bccRecipients: splitEditorLines(editor.value.bccRecipients),
		subject: editor.value.subject,
		textBody: editor.value.textBody,
		signatureId: editor.value.signatureId,
	}))

	async function load(): Promise<void> {
		const page = await listMailDrafts(input.connectionId())
		records.value = page.item
		const selected = records.value.find((entry) => entry.draftId === editor.value.draftId)
		if (selected) editor.value = fromRecord(selected)
	}

	function select(draftId: string): void {
		const selected = records.value.find((candidate) => candidate.draftId === draftId)
		if (selected) editor.value = fromRecord(selected)
	}

	async function save(): Promise<void> {
		if (!mutationReady()) return
		await run(async () => {
			const draftId = editor.value.draftId || crypto.randomUUID()
			await upsertMailDraft({
				connectionId: input.connectionId(),
				draftId,
				expectedRevision: editor.value.revision,
				mode: editor.value.mode,
				providerConversationId: editor.value.providerConversationId,
				inReplyToProviderMessageId: editor.value.inReplyToProviderMessageId,
				toRecipients: splitEditorLines(editor.value.toRecipients),
				ccRecipients: splitEditorLines(editor.value.ccRecipients),
				bccRecipients: splitEditorLines(editor.value.bccRecipients),
				subject: editor.value.subject,
				textBody: editor.value.textBody,
				templateId: editor.value.templateId,
				signatureId: editor.value.signatureId,
			})
			editor.value.draftId = draftId
			notice.value = 'Mail draft saved.'
			await load()
			select(draftId)
		})
	}

	async function remove(): Promise<void> {
		if (!mutationReady() || !editor.value.draftId || editor.value.revision === undefined) return
		await run(async () => {
			await deleteMailDraft(input.connectionId(), editor.value.draftId, editor.value.revision!)
			notice.value = 'Mail draft deleted.'
			editor.value = emptyDraft()
			await load()
		})
	}

	function applyTemplate(preview: MailTemplatePreviewV1): void {
		editor.value = {
			...editor.value,
			subject: preview.subject,
			textBody: preview.textBody,
			templateId: preview.templateId,
		}
	}

	function useSignature(signatureId: string): void {
		editor.value = { ...editor.value, signatureId }
	}

	function update(patch: MailDraftEditorPatch): void {
		editor.value = { ...editor.value, ...patch }
	}

	function clear(): void {
		records.value = []
		editor.value = emptyDraft()
		notice.value = ''
	}

	function mutationReady(): boolean {
		if (!input.canMutate()) {
			notice.value = 'Mail composition command capability is not admitted.'
			return false
		}
		return Boolean(input.connectionId())
	}

	async function run(work: () => Promise<void>): Promise<void> {
		busy.value = true
		notice.value = ''
		try {
			await work()
		} catch (error) {
			notice.value = error instanceof Error ? error.message : 'Mail draft mutation failed.'
		} finally {
			busy.value = false
		}
	}

	return {
		records,
		options,
		model,
		deliveryInput,
		busy,
		notice,
		load,
		select,
		save,
		remove,
		applyTemplate,
		useSignature,
		update,
		clear,
		startNew: () => { editor.value = emptyDraft() },
	}
}

function emptyDraft(): DraftState {
	return {
		draftId: '',
		mode: MailCompositionModeV1.MAIL_COMPOSITION_MODE_NEW,
		providerConversationId: '',
		inReplyToProviderMessageId: '',
		toRecipients: '',
		ccRecipients: '',
		bccRecipients: '',
		subject: '',
		textBody: '',
		templateId: '',
		signatureId: '',
	}
}

function fromRecord(record: MailDraftV1): DraftState {
	return {
		draftId: record.draftId,
		revision: record.revision,
		mode: record.mode,
		providerConversationId: record.providerConversationId ?? '',
		inReplyToProviderMessageId: record.inReplyToProviderMessageId ?? '',
		toRecipients: record.toRecipient.join('\n'),
		ccRecipients: record.ccRecipient.join('\n'),
		bccRecipients: record.bccRecipient.join('\n'),
		subject: record.subject,
		textBody: record.textBody,
		templateId: record.templateId ?? '',
		signatureId: record.signatureId ?? '',
	}
}
