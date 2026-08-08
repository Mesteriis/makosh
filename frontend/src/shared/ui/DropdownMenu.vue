<script setup lang="ts">
import { DropdownMenuRoot, DropdownMenuTrigger, DropdownMenuPortal, DropdownMenuContent, DropdownMenuItem, DropdownMenuSeparator, DropdownMenuLabel } from 'reka-ui'
import { computed, ref } from 'vue'
import { useMouseLeaveDismiss } from './useMouseLeaveDismiss'

const props = withDefaults(defineProps<{
  class?: string
  align?: 'start' | 'center' | 'end'
  side?: 'top' | 'bottom'
  sideOffset?: number
}>(), {
  align: 'start',
  side: 'bottom',
  sideOffset: 4
})

const contentClasses = computed(() => ['makosh-dropdown-content', props.class])
const isOpen = ref(false)
const contentRef = ref<HTMLElement | { $el?: Element | null } | null>(null)

const { cancelMouseLeaveDismiss, scheduleMouseLeaveDismiss } = useMouseLeaveDismiss(() => {
	isOpen.value = false
}, undefined, {
	isOpen,
	getBoundaryElements: () => [contentRef.value]
})

function setOpen(value: boolean): void {
	if (value) {
		cancelMouseLeaveDismiss()
	}

	isOpen.value = value
}
</script>

<template>
  <DropdownMenuRoot :open="isOpen" @update:open="setOpen">
    <DropdownMenuTrigger as-child>
      <slot name="trigger" />
    </DropdownMenuTrigger>
    <DropdownMenuPortal>
      <DropdownMenuContent
        ref="contentRef"
        :class="contentClasses"
        :align="align"
        :side="side"
        :side-offset="sideOffset"
        @mouseenter="cancelMouseLeaveDismiss"
        @mouseleave="scheduleMouseLeaveDismiss"
      >
        <slot />
      </DropdownMenuContent>
    </DropdownMenuPortal>
  </DropdownMenuRoot>
</template>
