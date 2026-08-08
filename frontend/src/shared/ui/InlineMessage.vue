<script setup lang="ts">
import { computed } from 'vue'
import Icon from './Icon.vue'

type InlineMessageTone = 'neutral' | 'info' | 'success' | 'warning' | 'danger'

const props = withDefaults(defineProps<{
  message?: string
  tone?: InlineMessageTone
  icon?: string
  class?: string
}>(), {
  tone: 'neutral'
})

const toneIcons: Record<InlineMessageTone, string> = {
  neutral: 'tabler:point',
  info: 'tabler:info-circle',
  success: 'tabler:check',
  warning: 'tabler:alert-triangle',
  danger: 'tabler:alert-circle'
}

const classes = computed(() => [
  'makosh-inline-message',
  `makosh-inline-message--${props.tone}`,
  props.class
])

const resolvedIcon = computed(() => props.icon ?? toneIcons[props.tone])
</script>

<template>
  <p :class="classes">
    <Icon :icon="resolvedIcon" size="0.95rem" class="makosh-inline-message-icon" />
    <span v-if="message">{{ message }}</span>
    <slot v-else />
  </p>
</template>
