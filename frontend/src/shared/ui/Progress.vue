<script setup lang="ts">
import { ProgressRoot, ProgressIndicator } from 'reka-ui'
import { computed, ref, watchEffect } from 'vue'

const props = withDefaults(defineProps<{
  modelValue?: number
  max?: number
  indeterminate?: boolean
  size?: 'sm' | 'md' | 'lg'
  class?: string
}>(), {
  modelValue: 0,
  max: 100,
  indeterminate: false,
  size: 'md'
})

const emit = defineEmits<{
  'update:modelValue': [value: number]
}>()

const percentage = computed(() => {
  if (props.max <= 0) return 0
  return Math.round((props.modelValue / props.max) * 100)
})

const rootClasses = computed(() => [
  'makosh-progress-root',
  `makosh-progress--${props.size}`,
  props.class,
  { 'makosh-progress--indeterminate': props.indeterminate }
])

const indicatorRef = ref<InstanceType<typeof ProgressIndicator> | null>(null)

watchEffect(() => {
  const element = indicatorRef.value?.$el as HTMLElement | undefined
  if (!element || props.indeterminate) return
  element.style.transform = `translateX(-${100 - percentage.value}%)`
})
</script>

<template>
  <ProgressRoot
    :model-value="modelValue"
    :max="max"
    :class="rootClasses"
    @update:model-value="(val: any) => emit('update:modelValue', Number(val))"
  >
    <ProgressIndicator ref="indicatorRef" class="makosh-progress-indicator" />
  </ProgressRoot>
</template>

