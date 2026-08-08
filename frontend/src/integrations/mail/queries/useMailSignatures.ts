import { computed, ref, shallowRef } from 'vue'
import type { MailSignatureV1 } from '../../../gen/makosh/mail/composition/v1/client_pb'
import {
	deleteMailSignature,
	listMailSignatures,
	upsertMailSignature,
} from '../api/mailCompositionGateway'
import {
	buildSignatureOptions,
	type MailSignatureEditorModel,
	type MailSignatureEditorPatch,
} from '../presentation/mailCompositionModel'

type SignatureState = Omit<MailSignatureEditorModel, 'revision'> & { revision?: bigint }

export function useMailSignatures(input: {
	canMutate: () => boolean
	connectionId: () => string
}) {
	const records = shallowRef<readonly MailSignatureV1[]>([])
	const editor = ref<SignatureState>(emptySignature())
	const busy = ref(false)
	const notice = ref('')

	const options = computed(() => buildSignatureOptions(records.value))
	const model = computed<MailSignatureEditorModel>(() => ({
		...editor.value,
		revision: editor.value.revision?.toString() ?? '',
	}))

	async function load(): Promise<void> {
		const page = await listMailSignatures(input.connectionId())
		records.value = page.item
		const selected = records.value.find(
			(entry) => entry.signatureId === editor.value.signatureId,
		)
		if (selected) editor.value = fromRecord(selected)
	}

	function select(signatureId: string): void {
		const selected = records.value.find((candidate) => candidate.signatureId === signatureId)
		if (selected) editor.value = fromRecord(selected)
	}

	async function save(): Promise<void> {
		if (!mutationReady()) return
		await run(async () => {
			const signatureId = editor.value.signatureId || crypto.randomUUID()
			await upsertMailSignature({
				connectionId: input.connectionId(),
				signatureId,
				expectedRevision: editor.value.revision,
				name: editor.value.name,
				textBody: editor.value.textBody,
				isDefault: editor.value.isDefault,
			})
			editor.value.signatureId = signatureId
			notice.value = 'Mail signature saved.'
			await load()
			select(signatureId)
		})
	}

	async function remove(): Promise<void> {
		if (!mutationReady() || !editor.value.signatureId || editor.value.revision === undefined) return
		await run(async () => {
			await deleteMailSignature(
				input.connectionId(),
				editor.value.signatureId,
				editor.value.revision!,
			)
			notice.value = 'Mail signature deleted.'
			editor.value = emptySignature()
			await load()
		})
	}

	function update(patch: MailSignatureEditorPatch): void {
		editor.value = { ...editor.value, ...patch }
	}

	function clear(): void {
		records.value = []
		editor.value = emptySignature()
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
			notice.value = error instanceof Error ? error.message : 'Mail signature mutation failed.'
		} finally {
			busy.value = false
		}
	}

	return {
		records,
		options,
		model,
		busy,
		notice,
		load,
		select,
		save,
		remove,
		update,
		clear,
		startNew: () => { editor.value = emptySignature() },
	}
}

function emptySignature(): SignatureState {
	return {
		signatureId: '',
		name: '',
		textBody: '',
		isDefault: false,
	}
}

function fromRecord(record: MailSignatureV1): SignatureState {
	return {
		signatureId: record.signatureId,
		revision: record.revision,
		name: record.name,
		textBody: record.textBody,
		isDefault: record.isDefault,
	}
}
