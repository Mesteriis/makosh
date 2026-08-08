<script setup lang="ts">
import { computed } from 'vue'
import Icon from '../Icon.vue'

type AlertTone = 'info' | 'success' | 'warning' | 'danger'

const props = withDefaults(defineProps<{
  title?: string
  description?: string
  tone?: AlertTone
  icon?: string
  class?: string
}>(), {
  tone: 'info'
})

const toneIcons: Record<AlertTone, string> = {
  info: 'tabler:info-circle',
  success: 'tabler:check-circle',
  warning: 'tabler:alert-triangle',
  danger: 'tabler:alert-circle'
}

const classes = computed(() => [
  'makosh-feedback',
  'makosh-alert',
  `makosh-feedback--${props.tone}`,
  props.class
])

const role = computed(() => props.tone === 'danger' ? 'alert' : 'status')
const resolvedIcon = computed(() => props.icon ?? toneIcons[props.tone])
</script>

<template>
  <section :class="classes" :role="role">
    <Icon :icon="resolvedIcon" size="1.125rem" class="makosh-feedback-icon" />
    <div class="makosh-feedback-body">
      <strong v-if="title" class="makosh-feedback-title">{{ title }}</strong>
      <p v-if="description" class="makosh-feedback-description">{{ description }}</p>
      <slot />
    </div>
  </section>
</template>
