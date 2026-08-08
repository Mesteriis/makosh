<script setup lang="ts">
import { computed } from 'vue'
import Spinner from '../Spinner.vue'

const props = withDefaults(defineProps<{
  title?: string
  description?: string
  class?: string
}>(), {
  title: 'Loading'
})

const classes = computed(() => ['makosh-state', 'makosh-state--loading', props.class])
</script>

<template>
  <section :class="classes" role="status" aria-live="polite">
    <Spinner decorative size="lg" />
    <div class="makosh-state-copy">
      <h3 class="makosh-state-title">{{ title }}</h3>
      <p v-if="description" class="makosh-state-description">{{ description }}</p>
    </div>
  </section>
</template>
