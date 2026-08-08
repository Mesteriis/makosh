<script setup lang="ts">
import { computed } from 'vue'
import Icon from './Icon.vue'
import type { DataDisplayTone } from './DataDisplay.types'

const props = withDefaults(defineProps<{
  title: string
  description?: string
  meta?: string
  icon?: string
  tone?: DataDisplayTone
  class?: string
}>(), {
  tone: 'neutral',
  icon: 'tabler:activity'
})

const classes = computed(() => [
  'makosh-activity-item',
  `makosh-activity-item--${props.tone}`,
  props.class
])
</script>

<template>
  <article :class="classes">
    <span class="makosh-activity-icon" aria-hidden="true">
      <Icon :icon="icon" size="1rem" />
    </span>
    <div class="makosh-activity-copy">
      <strong class="makosh-activity-title">{{ title }}</strong>
      <span v-if="description" class="makosh-activity-description">{{ description }}</span>
    </div>
    <span v-if="meta" class="makosh-activity-meta">{{ meta }}</span>
  </article>
</template>
