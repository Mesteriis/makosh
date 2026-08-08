<script setup lang="ts">
import { computed } from 'vue'
import type { DonutChartSegment, GraphicTone } from './Graphics.types'

const props = withDefaults(defineProps<{
  segments: DonutChartSegment[]
  label: string
  size?: 'sm' | 'md' | 'lg'
  showLegend?: boolean
  class?: string
}>(), {
  size: 'md',
  showLegend: true
})

const radius = 18
const circumference = 2 * Math.PI * radius
const fallbackTones: GraphicTone[] = ['accent', 'success', 'warning', 'info', 'danger', 'neutral']

const positiveSegments = computed(() => props.segments.filter((segment) => segment.value > 0))
const total = computed(() => positiveSegments.value.reduce((sum, segment) => sum + segment.value, 0))
const renderedSegments = computed(() => {
  let offset = 0
  return positiveSegments.value.map((segment, index) => {
    const length = total.value > 0 ? (segment.value / total.value) * circumference : 0
    const rendered = {
      ...segment,
      tone: segment.tone ?? fallbackTones[index % fallbackTones.length],
      dashArray: `${length} ${Math.max(circumference - length, 0)}`,
      dashOffset: -offset,
      percentage: total.value > 0 ? Math.round((segment.value / total.value) * 100) : 0
    }
    offset += length
    return rendered
  })
})

const classes = computed(() => [
  'makosh-donut-chart',
  `makosh-donut-chart--${props.size}`,
  props.class
])
</script>

<template>
  <figure :class="classes" role="img" :aria-label="label">
    <svg class="makosh-donut-chart__svg" viewBox="0 0 48 48" aria-hidden="true">
      <circle class="makosh-donut-chart__track" cx="24" cy="24" :r="radius" />
      <circle
        v-for="segment in renderedSegments"
        :key="segment.id"
        :class="['makosh-donut-chart__segment', `makosh-donut-chart__segment--${segment.tone}`]"
        cx="24"
        cy="24"
        :r="radius"
        :stroke-dasharray="segment.dashArray"
        :stroke-dashoffset="segment.dashOffset"
      />
    </svg>
    <figcaption v-if="$slots.default" class="makosh-donut-chart__caption">
      <slot />
    </figcaption>
    <ul v-if="showLegend" class="makosh-donut-chart__legend" aria-hidden="true">
      <li
        v-for="segment in renderedSegments"
        :key="segment.id"
        :class="['makosh-donut-chart__legend-item', `makosh-donut-chart__legend-item--${segment.tone}`]"
      >
        <span class="makosh-donut-chart__legend-dot"></span>
        <span>{{ segment.label }}</span>
        <strong>{{ segment.percentage }}%</strong>
      </li>
    </ul>
  </figure>
</template>
