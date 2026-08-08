<script setup lang="ts">
import { computed } from 'vue'
import Icon from './Icon.vue'

const props = withDefaults(defineProps<{
  title?: string
  description?: string
  class?: string
}>(), {
  title: 'No data',
  description: 'There is no local data for this surface yet.'
})

const classes = computed(() => ['makosh-state', 'makosh-state--no-data', props.class])
</script>

<template>
  <section :class="classes">
    <Icon icon="tabler:database-off" size="2rem" class="makosh-state-icon" />
    <div class="makosh-state-copy">
      <h3 class="makosh-state-title">{{ title }}</h3>
      <p v-if="description" class="makosh-state-description">{{ description }}</p>
    </div>
  </section>
</template>
