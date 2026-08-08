<script setup lang="ts">
import { computed } from 'vue'
import Icon from './Icon.vue'
import type { DataDisplayTone } from './DataDisplay.types'

const props = withDefaults(defineProps<{
  title: string
  description?: string
  time?: string
  icon?: string
  tone?: DataDisplayTone
  class?: string
}>(), {
  tone: 'neutral'
})

const classes = computed(() => [
  'makosh-timeline-item',
  `makosh-timeline-item--${props.tone}`,
  props.class
])
</script>

<template>
  <article :class="classes">
    <div class="makosh-timeline-marker" aria-hidden="true">
      <Icon v-if="icon" :icon="icon" size="0.875rem" />
    </div>
    <div class="makosh-timeline-copy">
      <div class="makosh-timeline-heading">
        <strong class="makosh-timeline-title">{{ title }}</strong>
        <time v-if="time" class="makosh-timeline-time">{{ time }}</time>
      </div>
      <p v-if="description" class="makosh-timeline-description">{{ description }}</p>
      <slot />
    </div>
  </article>
</template>
