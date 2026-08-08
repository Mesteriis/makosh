<script setup lang="ts">
import { Separator as RekaSeparator } from 'reka-ui'
import { computed } from 'vue'

const props = withDefaults(defineProps<{
  orientation?: 'horizontal' | 'vertical'
  decorative?: boolean
  class?: string
}>(), {
  orientation: 'horizontal',
  decorative: true
})

const classes = computed(() => [
  'makosh-separator',
  `makosh-separator--${props.orientation}`,
  props.class
])
</script>

<template>
  <RekaSeparator
    :class="classes"
    :orientation="orientation"
    :decorative="decorative"
  />
</template>

