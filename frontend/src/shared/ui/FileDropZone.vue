<script setup lang="ts">
import { computed, ref } from 'vue'
import Icon from './Icon.vue'

const props = withDefaults(defineProps<{
	id?: string
	accept?: string
	label?: string
	hint?: string
	disabled?: boolean
	multiple?: boolean
	class?: string
}>(), {
	label: 'Drop files here',
	hint: 'or choose files',
	disabled: false,
	multiple: false
})

const emit = defineEmits<{
	change: [files: File[]]
}>()

const fileInput = ref<HTMLInputElement>()
const isDragging = ref(false)
const classes = computed(() => [
	'makosh-file-drop-zone',
	{
		'makosh-file-drop-zone--dragging': isDragging.value,
		'makosh-file-drop-zone--disabled': props.disabled
	},
	props.class
])

function emitFiles(fileList: FileList | null): void {
	emit('change', Array.from(fileList ?? []))
}

function openPicker(): void {
	if (!props.disabled) {
		fileInput.value?.click()
	}
}

function handleDragover(event: DragEvent): void {
	if (props.disabled) {
		return
	}
	event.preventDefault()
	isDragging.value = true
}

function handleDrop(event: DragEvent): void {
	if (props.disabled) {
		return
	}
	event.preventDefault()
	isDragging.value = false
	emitFiles(event.dataTransfer?.files ?? null)
}

function handleNativeChange(event: Event): void {
	const target = event.target as HTMLInputElement
	emitFiles(target.files)
}
</script>

<template>
	<input
		ref="fileInput"
		class="makosh-file-drop-zone__input"
		:accept="accept"
		:aria-label="label"
		:disabled="disabled"
		:id="id"
		:multiple="multiple"
		type="file"
		@change="handleNativeChange"
	/>
	<div
		:class="classes"
		role="button"
		tabindex="0"
		:aria-disabled="disabled"
		:aria-label="label"
		@dragenter="handleDragover"
		@dragover="handleDragover"
		@dragleave="isDragging = false"
		@drop="handleDrop"
		@click="openPicker"
		@keydown.enter.prevent="openPicker"
		@keydown.space.prevent="openPicker"
	>
		<Icon icon="tabler:upload" size="1.25rem" aria-hidden="true" />
		<span class="makosh-file-drop-zone__label">{{ label }}</span>
		<span class="makosh-file-drop-zone__hint">{{ hint }}</span>
	</div>
</template>
