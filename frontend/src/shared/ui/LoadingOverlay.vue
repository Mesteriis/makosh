<script setup lang="ts">
import { computed } from 'vue'
import Spinner from './Spinner.vue'

const props = withDefaults(defineProps<{
  visible?: boolean
  label?: string
  description?: string
  class?: string
}>(), {
  visible: true,
  label: 'Loading'
})

const classes = computed(() => [
  'makosh-loading-overlay',
  props.class
])
</script>

<template>
  <div v-if="visible" :class="classes" role="status" aria-live="polite">
    <Spinner decorative size="lg" />
    <div class="makosh-loading-overlay-copy">
      <strong class="makosh-loading-overlay-label">{{ label }}</strong>
      <p v-if="description" class="makosh-loading-overlay-description">{{ description }}</p>
      <slot />
    </div>
  </div>
</template>
