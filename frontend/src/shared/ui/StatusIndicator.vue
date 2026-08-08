<script setup lang="ts">
import { computed } from 'vue'

type StatusTone = 'neutral' | 'info' | 'success' | 'warning' | 'danger'

const props = withDefaults(defineProps<{
  tone?: StatusTone
  label?: string
  size?: 'sm' | 'md' | 'lg'
  pulse?: boolean
  class?: string
}>(), {
  tone: 'neutral',
  size: 'md',
  pulse: false
})

const classes = computed(() => [
  'makosh-status-indicator',
  `makosh-status-indicator--${props.tone}`,
  `makosh-status-indicator--${props.size}`,
  { 'makosh-status-indicator--pulse': props.pulse },
  props.class
])
</script>

<template>
  <span :class="classes">
    <span class="makosh-status-indicator-dot" aria-hidden="true" />
    <span v-if="label" class="makosh-status-indicator-label">{{ label }}</span>
  </span>
</template>
