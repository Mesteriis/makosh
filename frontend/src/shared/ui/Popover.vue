<script setup lang="ts">
import { PopoverRoot, PopoverTrigger, PopoverPortal, PopoverContent, PopoverArrow, PopoverClose } from 'reka-ui'
import { computed, ref } from 'vue'
import Icon from './Icon.vue'
import { useMouseLeaveDismiss } from './useMouseLeaveDismiss'

const props = withDefaults(defineProps<{
  open?: boolean
  side?: 'top' | 'bottom' | 'left' | 'right'
  sideOffset?: number
  align?: 'start' | 'center' | 'end'
  closeLabel?: string
  class?: string
}>(), {
  side: 'bottom',
  sideOffset: 4,
  align: 'center',
  closeLabel: 'Close popover'
})

const emit = defineEmits<{
  'update:open': [value: boolean]
}>()

const contentClasses = computed(() => ['makosh-popover-content', props.class])
const internalOpen = ref(false)
const resolvedOpen = computed(() => props.open ?? internalOpen.value)
const contentRef = ref<HTMLElement | { $el?: Element | null } | null>(null)

const { cancelMouseLeaveDismiss, scheduleMouseLeaveDismiss } = useMouseLeaveDismiss(() => {
  setOpen(false)
}, undefined, {
  isOpen: resolvedOpen,
  getBoundaryElements: () => [contentRef.value]
})

function setOpen(value: boolean): void {
  if (value) {
    cancelMouseLeaveDismiss()
  }

  internalOpen.value = value
  emit('update:open', value)
}
</script>

<template>
  <PopoverRoot :open="resolvedOpen" @update:open="setOpen">
    <PopoverTrigger as-child>
      <slot name="trigger" />
    </PopoverTrigger>
    <PopoverPortal>
      <PopoverContent
        ref="contentRef"
        :class="contentClasses"
        :side="side"
        :side-offset="sideOffset"
        :align="align"
        @mouseenter="cancelMouseLeaveDismiss"
        @mouseleave="scheduleMouseLeaveDismiss"
      >
        <PopoverArrow class="makosh-popover-arrow" />
        <slot />
        <PopoverClose class="makosh-popover-close" as-child>
          <button class="makosh-popover-close-btn" :aria-label="closeLabel">
            <Icon icon="tabler:x" size="0.875rem" />
          </button>
        </PopoverClose>
      </PopoverContent>
    </PopoverPortal>
  </PopoverRoot>
</template>
