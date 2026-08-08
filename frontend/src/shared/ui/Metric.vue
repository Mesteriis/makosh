<script setup lang="ts">
import { computed } from 'vue'
import type { DataDisplayTone } from './DataDisplay.types'

const props = withDefaults(defineProps<{
  label: string
  value: string | number
  unit?: string
  delta?: string
  tone?: DataDisplayTone
  class?: string
}>(), {
  tone: 'neutral'
})

const classes = computed(() => [
  'makosh-metric',
  `makosh-metric--${props.tone}`,
  props.class
])
</script>

<template>
  <article :class="classes">
    <span class="makosh-metric-label">{{ label }}</span>
    <strong class="makosh-metric-value">
      {{ value }}<span v-if="unit" class="makosh-metric-unit">{{ unit }}</span>
    </strong>
    <span v-if="delta" class="makosh-metric-delta">{{ delta }}</span>
  </article>
</template>
