<script setup lang="ts">
import { ToastProvider, ToastViewport, ToastRoot, ToastTitle, ToastDescription, ToastClose } from 'reka-ui'
import { ref, computed, provide, onBeforeUnmount, type Ref } from 'vue'
import Icon from './Icon.vue'
import {
  TOAST_INJECTION_KEY,
  type DefaultToastItem,
  type ToastItem,
} from './toast'

const TOAST_EXIT_ANIMATION_MS = 820

const props = withDefaults(defineProps<{
  /** Swipe direction to dismiss */
  swipeDirection?: 'right' | 'left' | 'up' | 'down'
  /** Duration in ms before auto-dismiss */
  duration?: number
  /** Accessible label for dismiss controls. */
  closeLabel?: string
  /** Initial items for Storybook and deterministic visual tests. */
  defaultToasts?: DefaultToastItem[]
  class?: string
}>(), {
  swipeDirection: 'right',
  duration: 4000,
  closeLabel: 'Dismiss notification',
  defaultToasts: () => []
})

const toasts = ref<ToastItem[]>(
  props.defaultToasts.map((toast, index) => ({
    ...toast,
    id: toast.id ?? `default-toast-${index + 1}`
  }))
) as Ref<ToastItem[]>

let toastCounter = props.defaultToasts.length
const pendingRemovalTimers = new Map<string, ReturnType<typeof setTimeout>>()

function addToast(item: Omit<ToastItem, 'id'>): string {
  const id = `toast-${++toastCounter}`
  toasts.value = [...toasts.value, { ...item, id }]
  return id
}

function removeToast(id: string): void {
  clearToastRemoval(id)
  toasts.value = toasts.value.filter((t) => t.id !== id)
}

function clearToastRemoval(id: string): void {
  const timer = pendingRemovalTimers.get(id)
  if (timer === undefined) return

  clearTimeout(timer)
  pendingRemovalTimers.delete(id)
}

function scheduleToastRemoval(id: string): void {
  clearToastRemoval(id)
  pendingRemovalTimers.set(id, setTimeout(() => {
    pendingRemovalTimers.delete(id)
    removeToast(id)
  }, TOAST_EXIT_ANIMATION_MS))
}

function handleToastOpenChange(id: string, open: boolean): void {
  if (open) {
    clearToastRemoval(id)
    return
  }

  scheduleToastRemoval(id)
}

function success(title: string, description?: string): string {
  return addToast({ title, description, variant: 'success', duration: props.duration })
}

function warning(title: string, description?: string): string {
  return addToast({ title, description, variant: 'warning', duration: props.duration })
}

function error(title: string, description?: string): string {
  return addToast({ title, description, variant: 'error', duration: props.duration })
}

provide(TOAST_INJECTION_KEY, { addToast, removeToast, success, warning, error })

const viewportClasses = computed(() => [
  'makosh-toast-viewport',
  props.class
])

const variantIcons: Record<string, string> = {
  info: 'tabler:info-circle',
  success: 'tabler:check-circle',
  warning: 'tabler:alert-triangle',
  error: 'tabler:alert-circle'
}

onBeforeUnmount(() => {
  for (const timer of pendingRemovalTimers.values()) {
    clearTimeout(timer)
  }
  pendingRemovalTimers.clear()
})
</script>

<template>
  <ToastProvider :swipe-direction="swipeDirection" :duration="duration">
    <slot />

    <ToastViewport as="div" :class="viewportClasses">
      <ToastRoot
        v-for="toast in toasts"
        :key="toast.id"
        as="div"
        :data-toast-id="toast.id"
        :class="['makosh-toast-root', `makosh-toast--${toast.variant || 'default'}`]"
        @update:open="handleToastOpenChange(toast.id, $event)"
      >
        <div class="makosh-toast-inner">
          <Icon
            v-if="toast.variant && toast.variant !== 'default'"
            :icon="variantIcons[toast.variant]"
            size="1.125rem"
            class="makosh-toast-variant-icon"
          />
          <div class="makosh-toast-content">
            <ToastTitle v-if="toast.title" class="makosh-toast-title">
              {{ toast.title }}
            </ToastTitle>
            <ToastDescription v-if="toast.description" class="makosh-toast-description">
              {{ toast.description }}
            </ToastDescription>
          </div>
          <ToastClose class="makosh-toast-close-btn" :aria-label="closeLabel">
            <Icon icon="tabler:x" size="1rem" />
          </ToastClose>
        </div>
      </ToastRoot>
    </ToastViewport>
  </ToastProvider>
</template>
