<script setup lang="ts">
import { computed } from 'vue'
import Icon from './Icon.vue'

const props = withDefaults(defineProps<{
	modelValue?: number
	pageCount?: number
	label?: string
	previousLabel?: string
	nextLabel?: string
	class?: string
}>(), {
	modelValue: 1,
	pageCount: 1,
	label: 'Pagination',
	previousLabel: 'Previous page',
	nextLabel: 'Next page'
})

const emit = defineEmits<{
	'update:modelValue': [value: number]
	change: [value: number]
}>()

const classes = computed(() => ['makosh-pagination', props.class])
const pages = computed(() => {
	const pageCount = Math.max(props.pageCount, 1)
	const current = clampPage(props.modelValue)
	const start = Math.max(1, Math.min(current - 2, pageCount - 4))
	const end = Math.min(pageCount, start + 4)
	return Array.from({ length: end - start + 1 }, (_value, index) => start + index)
})

function clampPage(page: number): number {
	return Math.min(Math.max(page, 1), Math.max(props.pageCount, 1))
}

function selectPage(page: number): void {
	const nextPage = clampPage(page)
	emit('update:modelValue', nextPage)
	emit('change', nextPage)
}
</script>

<template>
	<nav :class="classes" :aria-label="label">
		<button
			class="makosh-pagination__button makosh-pagination__button--icon"
			type="button"
			:aria-label="previousLabel"
			:disabled="modelValue <= 1"
			@click="selectPage(modelValue - 1)"
		>
			<Icon icon="tabler:chevron-left" size="1rem" aria-hidden="true" />
		</button>
		<button
			v-for="page in pages"
			:key="page"
			class="makosh-pagination__button"
			type="button"
			:aria-current="page === modelValue ? 'page' : undefined"
			@click="selectPage(page)"
		>
			{{ page }}
		</button>
		<button
			class="makosh-pagination__button makosh-pagination__button--icon"
			type="button"
			:aria-label="nextLabel"
			:disabled="modelValue >= pageCount"
			@click="selectPage(modelValue + 1)"
		>
			<Icon icon="tabler:chevron-right" size="1rem" aria-hidden="true" />
		</button>
	</nav>
</template>
