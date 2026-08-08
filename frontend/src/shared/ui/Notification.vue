<script setup lang="ts">
import { computed } from 'vue'
import Icon from './Icon.vue'

type FeedbackTone = 'neutral' | 'info' | 'success' | 'warning' | 'danger'

const props = withDefaults(defineProps<{
  title?: string
  description?: string
  tone?: FeedbackTone
  icon?: string
  dismissible?: boolean
  closeLabel?: string
  class?: string
}>(), {
  tone: 'neutral',
  dismissible: false,
  closeLabel: 'Dismiss notification'
})

const emit = defineEmits<{
  close: []
}>()

const toneIcons: Record<FeedbackTone, string> = {
  neutral: 'tabler:bell',
  info: 'tabler:info-circle',
  success: 'tabler:check-circle',
  warning: 'tabler:alert-triangle',
  danger: 'tabler:alert-circle'
}

const classes = computed(() => [
  'makosh-feedback',
  'makosh-notification',
  `makosh-feedback--${props.tone}`,
  props.class
])

const role = computed(() => props.tone === 'danger' ? 'alert' : 'status')
const resolvedIcon = computed(() => props.icon ?? toneIcons[props.tone])
</script>

<template>
  <div :class="classes" :role="role">
    <Icon :icon="resolvedIcon" size="1.125rem" class="makosh-feedback-icon" />
    <div class="makosh-feedback-body">
      <strong v-if="title" class="makosh-feedback-title">{{ title }}</strong>
      <p v-if="description" class="makosh-feedback-description">{{ description }}</p>
      <slot />
    </div>
    <button
      v-if="dismissible"
      type="button"
      class="makosh-feedback-close"
      :aria-label="closeLabel"
      @click="emit('close')"
    >
      <Icon icon="tabler:x" size="1rem" />
    </button>
  </div>
</template>
