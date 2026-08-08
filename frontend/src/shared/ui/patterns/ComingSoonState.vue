<script setup lang="ts">
import { computed } from 'vue'
import Icon from '../Icon.vue'

const props = withDefaults(defineProps<{
  title?: string
  description?: string
  class?: string
}>(), {
  title: 'Coming soon',
  description: 'This UI contract is reserved for a future product surface.'
})

const classes = computed(() => ['makosh-state', 'makosh-state--coming-soon', props.class])
</script>

<template>
  <section :class="classes">
    <Icon icon="tabler:sparkles" size="2rem" class="makosh-state-icon" />
    <div class="makosh-state-copy">
      <h3 class="makosh-state-title">{{ title }}</h3>
      <p v-if="description" class="makosh-state-description">{{ description }}</p>
    </div>
  </section>
</template>
