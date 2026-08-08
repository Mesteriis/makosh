<script setup lang="ts">
import { computed } from 'vue'

const props = withDefaults(defineProps<{
  value?: number
  max?: number
  label?: string
  tone?: 'accent' | 'success' | 'warning' | 'danger'
  size?: 'sm' | 'md' | 'lg'
  indeterminate?: boolean
  showValue?: boolean
  class?: string
}>(), {
  value: 0,
  max: 100,
  tone: 'accent',
  size: 'md',
  indeterminate: false,
  showValue: true
})

const radius = 18
const circumference = 2 * Math.PI * radius

const normalizedValue = computed(() => Math.min(Math.max(props.value, 0), props.max))
const percentage = computed(() => {
  if (props.max <= 0) return 0
  return Math.round((normalizedValue.value / props.max) * 100)
})
const dashOffset = computed(() => circumference - (percentage.value / 100) * circumference)

const classes = computed(() => [
  'makosh-circular-progress',
  `makosh-circular-progress--${props.size}`,
  `makosh-circular-progress--${props.tone}`,
  { 'makosh-circular-progress--indeterminate': props.indeterminate },
  props.class
])

const ariaValueNow = computed(() => props.indeterminate ? undefined : normalizedValue.value)
const ariaValueText = computed(() => props.indeterminate ? props.label ?? 'Loading' : undefined)
</script>

<template>
  <div
    :class="classes"
    role="progressbar"
    :aria-label="label"
    :aria-valuemin="0"
    :aria-valuemax="max"
    :aria-valuenow="ariaValueNow"
    :aria-valuetext="ariaValueText"
  >
    <svg class="makosh-circular-progress-svg" viewBox="0 0 48 48" aria-hidden="true">
      <circle class="makosh-circular-progress-track" cx="24" cy="24" :r="radius" />
      <circle
        class="makosh-circular-progress-value"
        cx="24"
        cy="24"
        :r="radius"
        :stroke-dasharray="circumference"
        :stroke-dashoffset="dashOffset"
      />
    </svg>
    <span v-if="showValue && !indeterminate" class="makosh-circular-progress-label">
      {{ percentage }}%
    </span>
  </div>
</template>
