<script setup lang="ts">
import { computed } from 'vue'
import Icon from './Icon.vue'
import type { NavigationItem } from './Navigation.types'

const props = withDefaults(defineProps<{
	items?: NavigationItem[]
	modelValue?: string
	label?: string
	class?: string
}>(), {
	items: () => [],
	modelValue: '',
	label: 'Menu'
})

const emit = defineEmits<{
	'update:modelValue': [value: string]
	select: [item: NavigationItem]
}>()

const classes = computed(() => ['makosh-menu', props.class])

function selectItem(item: NavigationItem): void {
	if (item.disabled) {
		return
	}
	emit('update:modelValue', item.id)
	emit('select', item)
}
</script>

<template>
	<nav :class="classes" :aria-label="label">
		<button
			v-for="item in items"
			:key="item.id"
			class="makosh-menu__item"
			type="button"
			:aria-current="item.id === modelValue || item.current ? 'page' : undefined"
			:disabled="item.disabled"
			@click="selectItem(item)"
		>
			<Icon v-if="item.icon" :icon="item.icon" size="1rem" class="makosh-menu__icon" aria-hidden="true" />
			<span>{{ item.label }}</span>
		</button>
	</nav>
</template>
