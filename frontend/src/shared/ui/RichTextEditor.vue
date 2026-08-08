<script setup lang="ts">
import { EditorContent, useEditor } from '@tiptap/vue-3'
import { computed, onBeforeUnmount, ref, watch } from 'vue'
import IconButton from './IconButton.vue'
import Tooltip from './Tooltip.vue'
import { richTextEditorExtensions, normalizeRichTextHref } from './RichTextEditor.extensions'
import type { RichTextEditorAction, RichTextEditorToolbarAction } from './RichTextEditor.types'
import { sanitizeHtml } from './Media.rendering'

defineOptions({
	inheritAttrs: false
})

const defaultRichTextHtml = '<p></p>'
interface RichTextEditorSnapshot {
	getHTML: () => string
	getText: () => string
}

const defaultRichTextActions: RichTextEditorToolbarAction[] = [
	{ id: 'paragraph', label: 'Paragraph', icon: 'tabler:pilcrow', group: 'structure' },
	{ id: 'heading', label: 'Heading', icon: 'tabler:h-2', group: 'structure' },
	{ id: 'subheading', label: 'Subheading', icon: 'tabler:h-3', group: 'structure' },
	{ id: 'quote', label: 'Quote', icon: 'tabler:quote', group: 'structure' },
	{ id: 'bulletList', label: 'Bulleted list', icon: 'tabler:list', group: 'lists' },
	{ id: 'orderedList', label: 'Numbered list', icon: 'tabler:list-numbers', group: 'lists' },
	{ id: 'bold', label: 'Emphasis', icon: 'tabler:bold', group: 'marks' },
	{ id: 'italic', label: 'Nuance', icon: 'tabler:italic', group: 'marks' },
	{ id: 'underline', label: 'Underline', icon: 'tabler:underline', group: 'marks' },
	{ id: 'strike', label: 'Strike', icon: 'tabler:strikethrough', group: 'marks' },
	{ id: 'code', label: 'Inline code', icon: 'tabler:code', group: 'marks' },
	{ id: 'link', label: 'Evidence link', icon: 'tabler:link', group: 'insert' },
	{ id: 'codeBlock', label: 'Code block', icon: 'tabler:code-dots', group: 'insert' },
	{ id: 'horizontalRule', label: 'Divider', icon: 'tabler:separator-horizontal', group: 'insert' },
	{ id: 'clearFormatting', label: 'Clear formatting', icon: 'tabler:eraser', group: 'cleanup' }
]

let richTextEditorId = 0

const props = withDefaults(defineProps<{
	modelValue?: string
	id?: string
	label?: string
	helper?: string
	placeholder?: string
	toolbarLabel?: string
	outputLabel?: string
	disabled?: boolean
	maxLength?: number
	linkHref?: string
	actions?: RichTextEditorToolbarAction[]
	class?: string
}>(), {
	modelValue: defaultRichTextHtml,
	placeholder: 'Write with context',
	toolbarLabel: 'Rich text tools',
	outputLabel: 'Sanitized HTML',
	disabled: false,
	linkHref: 'https://example.local/evidence'
})

const emit = defineEmits<{
	'update:modelValue': [value: string]
	change: [value: string]
}>()

const generatedEditorId = `makosh-rich-text-editor-${++richTextEditorId}`
const editorRevision = ref(0)
const editorText = ref('')
const currentHtml = ref(normalizeEditorHtml(props.modelValue))

const editor = useEditor({
	content: currentHtml.value,
	editable: !props.disabled,
	extensions: richTextEditorExtensions,
	editorProps: {
		attributes: {
			id: props.id ?? generatedEditorId,
			role: 'textbox',
			'aria-label': props.label ?? props.placeholder,
			class: 'makosh-rich-text-editor__prosemirror'
		}
	},
	onCreate: ({ editor: currentEditor }) => syncEditorSnapshot(currentEditor),
	onUpdate: ({ editor: currentEditor }) => emitEditorHtml(currentEditor),
	onSelectionUpdate: ({ editor: currentEditor }) => syncEditorSnapshot(currentEditor)
})

const editorId = computed(() => props.id ?? generatedEditorId)
const toolbarActions = computed(() => props.actions?.length ? props.actions : defaultRichTextActions)
const isEditorEmpty = computed(() => editorText.value.trim().length === 0)
const remainingCharacters = computed(() => {
	if (typeof props.maxLength !== 'number') {
		return null
	}
	return props.maxLength - editorText.value.length
})
const isOverLimit = computed(() => typeof remainingCharacters.value === 'number' && remainingCharacters.value < 0)
const classes = computed(() => [
	'makosh-rich-text-editor',
	{
		'makosh-rich-text-editor--disabled': props.disabled,
		'makosh-rich-text-editor--over-limit': isOverLimit.value
	},
	props.class
])
const counterClasses = computed(() => [
	'makosh-rich-text-editor__counter',
	{ 'makosh-rich-text-editor__counter--over': isOverLimit.value }
])

watch(() => props.disabled, (disabled) => {
	editor.value?.setEditable(!disabled)
})

watch(() => props.modelValue, (nextValue) => {
	const normalizedHtml = normalizeEditorHtml(nextValue)
	if (!editor.value || normalizedHtml === currentHtml.value || normalizedHtml === editor.value.getHTML()) {
		return
	}
	editor.value.commands.setContent(normalizedHtml, { emitUpdate: false })
	syncEditorSnapshot(editor.value)
})

watch([() => props.id, () => props.label, () => props.placeholder], () => {
	editor.value?.setOptions({
		editorProps: {
			attributes: {
				id: editorId.value,
				role: 'textbox',
				'aria-label': props.label ?? props.placeholder,
				class: 'makosh-rich-text-editor__prosemirror'
			}
		}
	})
})

onBeforeUnmount(() => {
	editor.value?.destroy()
})

function emitEditorHtml(currentEditor: RichTextEditorSnapshot): void {
	syncEditorSnapshot(currentEditor)
	const sanitizedHtml = normalizeEditorHtml(currentEditor.getHTML())
	currentHtml.value = sanitizedHtml
	emit('update:modelValue', sanitizedHtml)
	emit('change', sanitizedHtml)
}

function syncEditorSnapshot(currentEditor: RichTextEditorSnapshot): void {
	currentHtml.value = normalizeEditorHtml(currentEditor.getHTML())
	editorText.value = currentEditor.getText()
	editorRevision.value += 1
}

function normalizeEditorHtml(value: string | undefined): string {
	const sanitizedHtml = sanitizeHtml(value ?? '')
	return sanitizedHtml.trim() ? sanitizedHtml : defaultRichTextHtml
}

function isActionActive(action: RichTextEditorAction): boolean {
	const currentEditor = editor.value
	if (!currentEditor) {
		return false
	}
	const currentRevision = editorRevision.value
	if (currentRevision < 0) {
		return false
	}

	switch (action) {
		case 'paragraph':
			return currentEditor.isActive('paragraph')
		case 'heading':
			return currentEditor.isActive('heading', { level: 2 })
		case 'subheading':
			return currentEditor.isActive('heading', { level: 3 })
		case 'quote':
			return currentEditor.isActive('blockquote')
		case 'bulletList':
			return currentEditor.isActive('bulletList')
		case 'orderedList':
			return currentEditor.isActive('orderedList')
		case 'bold':
			return currentEditor.isActive('bold')
		case 'italic':
			return currentEditor.isActive('italic')
		case 'underline':
			return currentEditor.isActive('underline')
		case 'strike':
			return currentEditor.isActive('strike')
		case 'code':
			return currentEditor.isActive('code')
		case 'link':
			return currentEditor.isActive('link')
		case 'codeBlock':
			return currentEditor.isActive('codeBlock')
		case 'horizontalRule':
		case 'clearFormatting':
			return false
		default:
			return false
	}
}

function actionClass(action: RichTextEditorToolbarAction, index: number): string {
	return [
		isActionActive(action.id) ? 'makosh-rich-text-editor__action--active' : '',
		isSeparatedAction(action, index) ? 'makosh-rich-text-editor__action--separated' : ''
	].filter(Boolean).join(' ')
}

function isSeparatedAction(action: RichTextEditorToolbarAction, index: number): boolean {
	if (index === 0 || !action.group) {
		return false
	}
	return toolbarActions.value[index - 1]?.group !== action.group
}

function runAction(action: RichTextEditorAction): void {
	if (!editor.value || props.disabled) {
		return
	}
	const command = editor.value.chain().focus()

	switch (action) {
		case 'paragraph':
			command.setNode('paragraph').run()
			return
		case 'heading':
			command.setNode('heading', { level: 2 }).run()
			return
		case 'subheading':
			command.setNode('heading', { level: 3 }).run()
			return
		case 'quote':
			command.toggleWrap('blockquote').run()
			return
		case 'bulletList':
			command.toggleList('bulletList', 'listItem').run()
			return
		case 'orderedList':
			command.toggleList('orderedList', 'listItem').run()
			return
		case 'bold':
			command.toggleMark('bold').run()
			return
		case 'italic':
			command.toggleMark('italic').run()
			return
		case 'underline':
			command.toggleMark('underline').run()
			return
		case 'strike':
			command.toggleMark('strike').run()
			return
		case 'code':
			command.toggleMark('code').run()
			return
		case 'link':
			toggleLink()
			return
		case 'codeBlock':
			command.setNode('codeBlock').run()
			return
		case 'horizontalRule':
			command.insertContent('<hr>').run()
			return
		case 'clearFormatting':
			command.unsetAllMarks().clearNodes().run()
			return
	}
}

function toggleLink(): void {
	const currentEditor = editor.value
	if (!currentEditor) {
		return
	}
	if (currentEditor.isActive('link')) {
		currentEditor.chain().focus().extendMarkRange('link').unsetMark('link').run()
		return
	}
	const href = normalizeRichTextHref(props.linkHref)
	if (!href) {
		return
	}
	currentEditor.chain().focus().setMark('link', { href }).run()
}
</script>

<template>
	<section :class="classes" v-bind="$attrs">
		<div v-if="label || helper || remainingCharacters !== null" class="makosh-rich-text-editor__header">
			<div class="makosh-rich-text-editor__copy">
				<label v-if="label" class="makosh-rich-text-editor__label" :for="editorId">
					{{ label }}
				</label>
				<p v-if="helper" class="makosh-rich-text-editor__helper">
					{{ helper }}
				</p>
			</div>
			<span v-if="remainingCharacters !== null" :class="counterClasses">
				{{ remainingCharacters }}
			</span>
		</div>

		<div class="makosh-rich-text-editor__toolbar" role="toolbar" :aria-label="toolbarLabel">
			<Tooltip v-for="(action, index) in toolbarActions" :key="action.id" :content="action.label">
				<template #trigger>
					<IconButton
						:class="actionClass(action, index)"
						:disabled="disabled"
						:icon="action.icon"
						:label="action.label"
						size="sm"
						variant="ghost"
						:aria-pressed="isActionActive(action.id)"
						@click="runAction(action.id)"
					/>
				</template>
			</Tooltip>
			<slot name="toolbar-end" />
		</div>

		<div class="makosh-rich-text-editor__surface">
			<span v-if="isEditorEmpty" class="makosh-rich-text-editor__placeholder">
				{{ placeholder }}
			</span>
			<EditorContent :editor="editor" class="makosh-rich-text-editor__content" />
		</div>

		<div class="makosh-rich-text-editor__footer">
			<span>{{ outputLabel }}</span>
			<code>html</code>
		</div>
	</section>
</template>
