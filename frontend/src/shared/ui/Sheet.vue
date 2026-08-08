<script setup lang="ts">
import { DialogRoot, DialogTrigger, DialogPortal, DialogOverlay, DialogContent, DialogTitle, DialogDescription, DialogClose } from 'reka-ui'
import { computed } from 'vue'
import Icon from './Icon.vue'

const props = withDefaults(defineProps<{
  open?: boolean
  title?: string
  description?: string
  side?: 'left' | 'right' | 'top' | 'bottom'
  class?: string
  contentClass?: string
}>(), {
  open: false,
  side: 'right'
})

const emit = defineEmits<{
  'update:open': [value: boolean]
}>()

const contentClasses = computed(() => [
  'makosh-sheet-content',
  `makosh-sheet--${props.side}`,
  props.contentClass
])
</script>

<template>
  <DialogRoot :open="open" @update:open="(val) => emit('update:open', val)">
    <DialogTrigger as-child>
      <slot name="trigger" />
    </DialogTrigger>
    <DialogPortal>
      <DialogOverlay class="makosh-sheet-overlay">
        <DialogContent :class="contentClasses">
          <div class="makosh-sheet-header">
            <DialogTitle v-if="title" class="makosh-sheet-title">{{ title }}</DialogTitle>
            <DialogDescription v-if="description" class="makosh-sheet-description">{{ description }}</DialogDescription>
            <slot name="header" />
          </div>
          <div class="makosh-sheet-body">
            <slot />
          </div>
          <div v-if="$slots.footer" class="makosh-sheet-footer">
            <slot name="footer" />
          </div>
          <DialogClose class="makosh-sheet-close" as-child>
            <button class="makosh-sheet-close-btn">
              <Icon icon="tabler:x" size="1.125rem" />
            </button>
          </DialogClose>
        </DialogContent>
      </DialogOverlay>
    </DialogPortal>
  </DialogRoot>
</template>

