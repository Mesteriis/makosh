<script setup lang="ts">
import { computed, useId } from 'vue'

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
  showValue: false
})

const id = useId()

const normalizedValue = computed(() => Math.min(Math.max(props.value, 0), props.max))
const percentage = computed(() => {
  if (props.max <= 0) return 0
  return Math.round((normalizedValue.value / props.max) * 100)
})
const progressValue = computed(() => props.indeterminate ? undefined : normalizedValue.value)

const classes = computed(() => [
  'makosh-progress-bar',
  `makosh-progress-bar--${props.size}`,
  `makosh-progress-bar--${props.tone}`,
  { 'makosh-progress-bar--indeterminate': props.indeterminate },
  props.class
])
</script>

<template>
  <div :class="classes">
    <div v-if="label || showValue" class="makosh-progress-bar-header">
      <label v-if="label" class="makosh-progress-bar-label" :for="id">{{ label }}</label>
      <span v-if="showValue && !indeterminate" class="makosh-progress-bar-value">{{ percentage }}%</span>
    </div>
    <progress
      :id="id"
      class="makosh-progress-bar-native"
      :value="progressValue"
      :max="max"
    >
      {{ percentage }}%
    </progress>
  </div>
</template>
