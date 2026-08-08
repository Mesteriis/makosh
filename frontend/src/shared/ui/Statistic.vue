<script setup lang="ts">
import { computed } from 'vue'
import type { DataDisplayTone } from './DataDisplay.types'

const props = withDefaults(defineProps<{
  label: string
  value: string | number
  description?: string
  trend?: string
  tone?: DataDisplayTone
  class?: string
}>(), {
  tone: 'neutral'
})

const classes = computed(() => [
  'makosh-statistic',
  `makosh-statistic--${props.tone}`,
  props.class
])
</script>

<template>
  <article :class="classes">
    <span class="makosh-statistic-label">{{ label }}</span>
    <strong class="makosh-statistic-value">{{ value }}</strong>
    <span v-if="trend" class="makosh-statistic-trend">{{ trend }}</span>
    <p v-if="description" class="makosh-statistic-description">{{ description }}</p>
  </article>
</template>
