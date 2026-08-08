<script setup lang="ts">
import { computed } from 'vue'
import type { RadarChartMetric, GraphicTone } from './Graphics.types'

const props = withDefaults(defineProps<{
  metrics: RadarChartMetric[]
  label: string
  tone?: GraphicTone
  size?: 'sm' | 'md' | 'lg'
  class?: string
}>(), {
  tone: 'accent',
  size: 'md'
})

const center = 50
const radius = 34

function pointFor(index: number, valueRatio: number): { x: number; y: number } {
  const angle = (Math.PI * 2 * index) / Math.max(props.metrics.length, 1) - Math.PI / 2
  return {
    x: center + Math.cos(angle) * radius * valueRatio,
    y: center + Math.sin(angle) * radius * valueRatio
  }
}

const normalizedMetrics = computed(() => {
  return props.metrics.map((metric) => ({
    ...metric,
    ratio: Math.min(Math.max(metric.value / Math.max(metric.max ?? 100, 1), 0), 1)
  }))
})

const polygonPoints = computed(() => {
  return normalizedMetrics.value
    .map((metric, index) => pointFor(index, metric.ratio))
    .map((point) => `${point.x.toFixed(2)},${point.y.toFixed(2)}`)
    .join(' ')
})

const axes = computed(() => {
  return normalizedMetrics.value.map((metric, index) => {
    const edge = pointFor(index, 1)
    const labelPoint = pointFor(index, 1.24)
    return {
      ...metric,
      x2: edge.x,
      y2: edge.y,
      labelX: labelPoint.x,
      labelY: labelPoint.y
    }
  })
})

const gridRings = [0.33, 0.66, 1]

const classes = computed(() => [
  'makosh-radar-chart',
  `makosh-radar-chart--${props.size}`,
  `makosh-radar-chart--${props.tone}`,
  props.class
])
</script>

<template>
  <figure :class="classes" role="img" :aria-label="label">
    <svg class="makosh-radar-chart__svg" viewBox="0 0 100 100" aria-hidden="true">
      <circle
        v-for="ring in gridRings"
        :key="ring"
        class="makosh-radar-chart__ring"
        :cx="center"
        :cy="center"
        :r="radius * ring"
      />
      <line
        v-for="axis in axes"
        :key="axis.id"
        class="makosh-radar-chart__axis"
        :x1="center"
        :y1="center"
        :x2="axis.x2"
        :y2="axis.y2"
      />
      <polygon class="makosh-radar-chart__area" :points="polygonPoints" />
      <text
        v-for="axis in axes"
        :key="`${axis.id}-label`"
        class="makosh-radar-chart__label"
        :x="axis.labelX"
        :y="axis.labelY"
        text-anchor="middle"
        dominant-baseline="middle"
      >
        {{ axis.label }}
      </text>
    </svg>
  </figure>
</template>
