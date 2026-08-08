<script setup lang="ts">
import { SelectRoot, SelectTrigger, SelectValue, SelectContent, SelectItem, SelectItemIndicator, SelectViewport, SelectPortal } from 'reka-ui'
import { computed, ref } from 'vue'
import Icon from '../Icon.vue'
import { useMouseLeaveDismiss } from '../useMouseLeaveDismiss'

const props = withDefaults(defineProps<{
  modelValue?: string
  placeholder?: string
  ariaLabel?: string
  disabled?: boolean
  error?: string
  class?: string
  options?: Array<{ value: string; label: string }>
}>(), {
  modelValue: '',
  placeholder: 'Select…',
  disabled: false
})

const emit = defineEmits<{
  'update:modelValue': [value: string]
}>()

const triggerClasses = computed(() => [
  'makosh-select-trigger',
  { 'makosh-select--error': props.error },
  props.class
])

const accessibleLabel = computed(() => props.ariaLabel ?? props.placeholder)
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
  <div class="makosh-select-wrapper">
    <SelectRoot
      :open="isOpen"
      :model-value="modelValue || undefined"
      :disabled="disabled"
      @update:model-value="(val) => emit('update:modelValue', val || '')"
      @update:open="setOpen"
    >
      <SelectTrigger :class="triggerClasses" :aria-label="accessibleLabel">
        <SelectValue :placeholder="placeholder" class="makosh-select-value" />
        <Icon icon="tabler:chevron-down" size="1rem" class="makosh-select-chevron" />
      </SelectTrigger>
      <SelectPortal>
        <SelectContent
          ref="contentRef"
          class="makosh-select-content"
          :side-offset="4"
          @mouseenter="cancelMouseLeaveDismiss"
          @mouseleave="scheduleMouseLeaveDismiss"
        >
          <SelectViewport class="makosh-select-viewport">
            <SelectItem
              v-for="opt in options"
              :key="opt.value"
              :value="opt.value"
              class="makosh-select-item"
            >
              <SelectItemIndicator>
                <Icon icon="tabler:check" size="0.875rem" class="makosh-select-check" />
              </SelectItemIndicator>
              <span>{{ opt.label }}</span>
            </SelectItem>
          </SelectViewport>
        </SelectContent>
      </SelectPortal>
    </SelectRoot>
    <span v-if="error" class="makosh-select-error">{{ error }}</span>
  </div>
</template>
