<script setup lang="ts">
import { computed } from 'vue'
import Icon from '../Icon.vue'

const props = withDefaults(defineProps<{
  title?: string
  description?: string
  class?: string
}>(), {
  title: 'Something went wrong',
  description: 'The owning surface can provide a retry action.'
})

const classes = computed(() => ['makosh-state', 'makosh-state--error', props.class])
</script>

<template>
  <section :class="classes" role="alert">
    <Icon icon="tabler:alert-circle" size="2rem" class="makosh-state-icon" />
    <div class="makosh-state-copy">
      <h3 class="makosh-state-title">{{ title }}</h3>
      <p v-if="description" class="makosh-state-description">{{ description }}</p>
    </div>
    <div v-if="$slots.action" class="makosh-state-action">
      <slot name="action" />
    </div>
  </section>
</template>
