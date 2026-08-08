<script setup lang="ts">
import { computed } from 'vue'
import type { GraphicTone } from './Graphics.types'

const props = withDefaults(defineProps<{
  values: number[]
  label: string
  tone?: GraphicTone
  size?: 'sm' | 'md' | 'lg'
  showArea?: boolean
  class?: string
}>(), {
  tone: 'accent',
  size: 'md',
  showArea: true
})

const width = 120
const height = 44
const padding = 4

const bounds = computed(() => {
  const values = props.values.length > 0 ? props.values : [0]
  const min = Math.min(...values)
  const max = Math.max(...values)
  return { min, max, range: Math.max(max - min, 1) }
})

const points = computed(() => {
  const values = props.values.length > 0 ? props.values : [0]
  const step = values.length > 1 ? (width - padding * 2) / (values.length - 1) : 0
  return values.map((value, index) => {
    const x = padding + index * step
    const y = height - padding - ((value - bounds.value.min) / bounds.value.range) * (height - padding * 2)
    return { x, y }
  })
})

const linePoints = computed(() => points.value.map((point) => `${point.x.toFixed(2)},${point.y.toFixed(2)}`).join(' '))
const areaPoints = computed(() => {
  if (points.value.length === 0) return ''
  const first = points.value[0]
  const last = points.value[points.value.length - 1]
  return `${first.x.toFixed(2)},${height - padding} ${linePoints.value} ${last.x.toFixed(2)},${height - padding}`
})

const classes = computed(() => [
  'makosh-sparkline',
  `makosh-sparkline--${props.size}`,
  `makosh-sparkline--${props.tone}`,
  props.class
])
</script>

<template>
  <svg :class="classes" viewBox="0 0 120 44" role="img" :aria-label="label">
    <polygon v-if="showArea" class="makosh-sparkline__area" :points="areaPoints" aria-hidden="true" />
    <polyline class="makosh-sparkline__line" :points="linePoints" aria-hidden="true" />
  </svg>
</template>
