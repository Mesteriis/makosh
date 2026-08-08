<script setup lang="ts">
import { ScrollAreaRoot, ScrollAreaViewport, ScrollAreaScrollbar, ScrollAreaThumb } from 'reka-ui'
import { computed } from 'vue'

const props = withDefaults(defineProps<{
  class?: string
  size?: 'sm' | 'md' | 'lg' | 'full'
  maxHeight?: string
}>(), {
  size: 'full'
})

const classes = computed(() => [
  'makosh-scroll-area',
  `makosh-scroll-area--${props.size}`,
  { 'makosh-scroll-area--bounded': props.maxHeight },
  props.class
])
</script>

<template>
  <ScrollAreaRoot :class="classes">
    <ScrollAreaViewport class="makosh-scroll-viewport">
      <slot />
    </ScrollAreaViewport>
    <ScrollAreaScrollbar class="makosh-scrollbar" orientation="vertical">
      <ScrollAreaThumb class="makosh-scroll-thumb" />
    </ScrollAreaScrollbar>
    <ScrollAreaScrollbar class="makosh-scrollbar" orientation="horizontal">
      <ScrollAreaThumb class="makosh-scroll-thumb" />
    </ScrollAreaScrollbar>
  </ScrollAreaRoot>
</template>
