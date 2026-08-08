<script setup lang="ts">
import { computed } from 'vue'
import type { DataDisplayTone } from './DataDisplay.types'

const props = withDefaults(defineProps<{
  label: string
  value: string | number
  description?: string
  tone?: DataDisplayTone
  class?: string
}>(), {
  tone: 'neutral'
})

const classes = computed(() => [
  'makosh-key-value',
  `makosh-key-value--${props.tone}`,
  props.class
])
</script>

<template>
  <div :class="classes">
    <dt class="makosh-key-value-label">{{ label }}</dt>
    <dd class="makosh-key-value-value">{{ value }}</dd>
    <dd v-if="description" class="makosh-key-value-description">{{ description }}</dd>
  </div>
</template>
