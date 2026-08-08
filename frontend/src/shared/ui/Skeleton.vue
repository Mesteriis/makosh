<script setup lang="ts">
import { computed } from 'vue'

const props = withDefaults(defineProps<{
  class?: string
  width?: string
  height?: string
  rounded?: boolean
}>(), {
  width: '100%',
  height: '1rem',
  rounded: false
})

const classes = computed(() => [
  'makosh-skeleton',
  { 'makosh-skeleton--rounded': props.rounded },
  props.class
])

const styles = computed(() => ({
  '--makosh-skeleton-width': props.width,
  '--makosh-skeleton-height': props.height
}))
</script>

<template>
  <div :class="classes" :style="styles" />
</template>
