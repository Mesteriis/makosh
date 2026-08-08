import { computed, ref, shallowRef } from 'vue'
import type {
	MailTemplatePreviewV1,
	MailTemplateV1,
} from '../../../gen/makosh/mail/composition/v1/client_pb'
import {
	deleteMailTemplate,
	listMailTemplates,
	previewMailTemplate,
	upsertMailTemplate,
} from '../api/mailCompositionGateway'
import {
	buildTemplateOptions,
	parseTemplateValues,
	splitEditorLines,
	type MailTemplateEditorModel,
	type MailTemplateEditorPatch,
} from '../presentation/mailCompositionModel'

type TemplateState = Omit<MailTemplateEditorModel, 'revision'> & { revision?: bigint }

export function useMailTemplates(input: {
	canMutate: () => boolean
	connectionId: () => string
}) {
	const records = shallowRef<readonly MailTemplateV1[]>([])
	const editor = ref<TemplateState>(emptyTemplate())
	const busy = ref(false)
	const previewing = ref(false)
	const notice = ref('')

	const options = computed(() => buildTemplateOptions(records.value))
	const model = computed<MailTemplateEditorModel>(() => ({
		...editor.value,
		revision: editor.value.revision?.toString() ?? '',
	}))

	async function load(): Promise<void> {
		const page = await listMailTemplates(input.connectionId())
		records.value = page.item
		const selected = records.value.find((entry) => entry.templateId === editor.value.templateId)
		if (selected) editor.value = fromRecord(selected)
	}

	function select(templateId: string): void {
		const selected = records.value.find((candidate) => candidate.templateId === templateId)
		if (selected) editor.value = fromRecord(selected)
	}

	async function save(): Promise<void> {
		if (!mutationReady()) return
		await run(false, async () => {
			const templateId = editor.value.templateId || crypto.randomUUID()
			await upsertMailTemplate({
				connectionId: input.connectionId(),
				templateId,
				expectedRevision: editor.value.revision,
				name: editor.value.name,
				subjectTemplate: editor.value.subjectTemplate,
				textBodyTemplate: editor.value.textBodyTemplate,
				variables: splitEditorLines(editor.value.variables),
				locale: editor.value.locale,
			})
			editor.value.templateId = templateId
			notice.value = 'Mail template saved.'
			await load()
			select(templateId)
		})
	}

	async function remove(): Promise<void> {
		if (!mutationReady() || !editor.value.templateId || editor.value.revision === undefined) return
		await run(false, async () => {
			await deleteMailTemplate(
				input.connectionId(),
				editor.value.templateId,
				editor.value.revision!,
			)
			notice.value = 'Mail template deleted.'
			editor.value = emptyTemplate()
			await load()
		})
	}

	async function preview(): Promise<MailTemplatePreviewV1 | undefined> {
		if (!input.connectionId() || !editor.value.templateId) return undefined
		let result: MailTemplatePreviewV1 | undefined
		await run(true, async () => {
			result = await previewMailTemplate({
				connectionId: input.connectionId(),
				templateId: editor.value.templateId,
				values: parseTemplateValues(editor.value.previewValues),
			})
			editor.value.previewSummary = result.ready
				? 'Template is ready and applied to the draft.'
				: previewProblems(result)
		})
		return result
	}

	function update(patch: MailTemplateEditorPatch): void {
		editor.value = { ...editor.value, ...patch }
	}

	function clear(): void {
		records.value = []
		editor.value = emptyTemplate()
		notice.value = ''
	}

	function mutationReady(): boolean {
		if (!input.canMutate()) {
			notice.value = 'Mail composition command capability is not admitted.'
			return false
		}
		return Boolean(input.connectionId())
	}

	async function run(isPreview: boolean, work: () => Promise<void>): Promise<void> {
		busy.value = true
		previewing.value = isPreview
		notice.value = ''
		try {
			await work()
		} catch (error) {
			notice.value = error instanceof Error ? error.message : 'Mail template operation failed.'
		} finally {
			busy.value = false
			previewing.value = false
		}
	}

	return {
		records,
		options,
		model,
		busy,
		previewing,
		notice,
		load,
		select,
		save,
		remove,
		preview,
		update,
		clear,
		startNew: () => { editor.value = emptyTemplate() },
	}
}

function emptyTemplate(): TemplateState {
	return {
		templateId: '',
		name: '',
		subjectTemplate: '',
		textBodyTemplate: '',
		variables: '',
		locale: '',
		previewValues: '',
		previewSummary: '',
	}
}

function fromRecord(record: MailTemplateV1): TemplateState {
	return {
		templateId: record.templateId,
		revision: record.revision,
		name: record.name,
		subjectTemplate: record.subjectTemplate,
		textBodyTemplate: record.textBodyTemplate,
		variables: record.variable.join('\n'),
		locale: record.locale ?? '',
		previewValues: record.variable.map((name) => `${name}=`).join('\n'),
		previewSummary: '',
	}
}

function previewProblems(preview: MailTemplatePreviewV1): string {
	return [
		preview.missingVariable.length ? `missing: ${preview.missingVariable.join(', ')}` : '',
		preview.unresolvedVariable.length
			? `unresolved: ${preview.unresolvedVariable.join(', ')}`
			: '',
		preview.malformedPlaceholder.length
			? `malformed: ${preview.malformedPlaceholder.join(', ')}`
			: '',
	].filter(Boolean).join(' · ')
}
