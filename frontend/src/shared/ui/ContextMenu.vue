<script setup lang="ts">
import { computed, ref } from 'vue'
import Icon from './Icon.vue'
import type { NavigationItem } from './Navigation.types'
import { useMouseLeaveDismiss } from './useMouseLeaveDismiss'

const props = withDefaults(defineProps<{
	items?: NavigationItem[]
	label?: string
	openLabel?: string
	defaultOpen?: boolean
	class?: string
}>(), {
	items: () => [],
	label: 'Context menu',
	openLabel: 'Open context menu',
	defaultOpen: false
})

const emit = defineEmits<{
	select: [item: NavigationItem]
}>()

const isOpen = ref(props.defaultOpen)
const rootRef = ref<HTMLElement | null>(null)
const contentRef = ref<HTMLElement | null>(null)
const classes = computed(() => ['makosh-context-menu', props.class])
const { cancelMouseLeaveDismiss, scheduleMouseLeaveDismiss } = useMouseLeaveDismiss(closeMenu, undefined, {
	isOpen,
	getBoundaryElements: () => [rootRef.value, contentRef.value]
})

function openMenu(event?: Event): void {
	event?.preventDefault()
	cancelMouseLeaveDismiss()
	isOpen.value = true
}

function closeMenu(): void {
	cancelMouseLeaveDismiss()
	isOpen.value = false
}

function selectItem(item: NavigationItem): void {
	if (item.disabled) {
		return
	}
	emit('select', item)
	closeMenu()
}
</script>

<template>
	<div
		ref="rootRef"
		:class="classes"
		@keydown.escape="closeMenu"
		@mouseenter="cancelMouseLeaveDismiss"
		@mouseleave="scheduleMouseLeaveDismiss"
	>
		<div class="makosh-context-menu__trigger" @contextmenu="openMenu">
			<slot name="trigger">
				<button class="makosh-context-menu__button" type="button" @click="openMenu">
					{{ openLabel }}
				</button>
			</slot>
		</div>
		<div v-if="isOpen" ref="contentRef" class="makosh-context-menu__content" role="menu" :aria-label="label">
			<button
				v-for="item in items"
				:key="item.id"
				class="makosh-context-menu__item"
				role="menuitem"
				type="button"
				:disabled="item.disabled"
				@click="selectItem(item)"
			>
				<Icon v-if="item.icon" :icon="item.icon" size="1rem" class="makosh-context-menu__icon" aria-hidden="true" />
				<span>{{ item.label }}</span>
			</button>
		</div>
	</div>
</template>
