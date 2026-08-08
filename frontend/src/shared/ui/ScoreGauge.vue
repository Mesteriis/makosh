<script setup lang="ts">
import { computed } from 'vue'
import type { GraphicTone } from './Graphics.types'

const props = withDefaults(defineProps<{
  value: number
  max?: number
  label: string
  tone?: GraphicTone
  size?: 'sm' | 'md' | 'lg'
  unit?: string
  showValue?: boolean
  class?: string
}>(), {
  max: 100,
  tone: 'accent',
  size: 'md',
  unit: '',
  showValue: true
})

const radius = 19
const circumference = 2 * Math.PI * radius

const boundedMax = computed(() => Math.max(props.max, 1))
const normalizedValue = computed(() => Math.min(Math.max(props.value, 0), boundedMax.value))
const percentage = computed(() => normalizedValue.value / boundedMax.value)
const dashOffset = computed(() => circumference - percentage.value * circumference)
const roundedValue = computed(() => Math.round(normalizedValue.value))
const valueText = computed(() => `${roundedValue.value}${props.unit}`)

const classes = computed(() => [
  'makosh-score-gauge',
  `makosh-score-gauge--${props.size}`,
  `makosh-score-gauge--${props.tone}`,
  props.class
])
</script>

<template>
  <div
    :class="classes"
    role="meter"
    :aria-label="label"
    :aria-valuemin="0"
    :aria-valuemax="boundedMax"
    :aria-valuenow="normalizedValue"
    :aria-valuetext="valueText"
  >
    <svg class="makosh-score-gauge__svg" viewBox="0 0 56 56" aria-hidden="true">
      <circle class="makosh-score-gauge__track" cx="28" cy="28" :r="radius" />
      <circle
        class="makosh-score-gauge__value"
        cx="28"
        cy="28"
        :r="radius"
        :stroke-dasharray="circumference"
        :stroke-dashoffset="dashOffset"
      />
    </svg>
    <span v-if="showValue" class="makosh-score-gauge__label" aria-hidden="true">
      <strong>{{ roundedValue }}</strong>
      <small v-if="unit">{{ unit }}</small>
    </span>
  </div>
</template>
