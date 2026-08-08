<script setup lang="ts">
import { DialogRoot, DialogTrigger, DialogPortal, DialogOverlay, DialogContent, DialogTitle, DialogDescription, DialogClose } from 'reka-ui'
import { computed } from 'vue'
import Icon from './Icon.vue'

const props = withDefaults(defineProps<{
  open?: boolean
  title?: string
  description?: string
  closeLabel?: string
  showClose?: boolean
  closeOnInteractOutside?: boolean
  class?: string
  contentClass?: string
}>(), {
  open: false,
  closeLabel: 'Close dialog',
  closeOnInteractOutside: true,
  showClose: true
})

const emit = defineEmits<{
  'update:open': [value: boolean]
}>()

const contentClasses = computed(() => ['makosh-dialog-content', props.contentClass])

function handleOutsideCloseEvent(event: Event): void {
  if (props.closeOnInteractOutside) return
  event.preventDefault()
}
</script>

<template>
  <DialogRoot :open="open" @update:open="(val) => emit('update:open', val)">
    <DialogTrigger v-if="$slots.trigger" as-child>
      <slot name="trigger" />
    </DialogTrigger>
    <DialogPortal>
      <DialogOverlay class="makosh-dialog-overlay">
        <DialogContent
          :class="contentClasses"
          @interact-outside="handleOutsideCloseEvent"
          @pointer-down-outside="handleOutsideCloseEvent"
        >
          <div class="makosh-dialog-header">
            <DialogTitle v-if="title" class="makosh-dialog-title">{{ title }}</DialogTitle>
            <DialogDescription v-if="description" class="makosh-dialog-description">{{ description }}</DialogDescription>
            <slot name="header" />
          </div>
          <div class="makosh-dialog-body">
            <slot />
          </div>
          <div v-if="$slots.footer" class="makosh-dialog-footer">
            <slot name="footer" />
          </div>
          <slot name="chrome" />
          <DialogClose v-if="showClose" class="makosh-dialog-close" as-child>
            <button class="makosh-dialog-close-btn" type="button" :aria-label="closeLabel">
              <Icon icon="tabler:x" size="1.125rem" />
            </button>
          </DialogClose>
        </DialogContent>
      </DialogOverlay>
    </DialogPortal>
  </DialogRoot>
</template>
