<script setup lang="ts">
import { computed } from 'vue'
import Icon from '../Icon.vue'

const props = withDefaults(defineProps<{
  title?: string
  description?: string
  icon?: string
  class?: string
}>(), {
  title: 'Nothing here yet',
  icon: 'tabler:inbox'
})

const classes = computed(() => ['makosh-state', 'makosh-state--empty', props.class])
</script>

<template>
  <section :class="classes">
    <Icon :icon="icon" size="2rem" class="makosh-state-icon" />
    <div class="makosh-state-copy">
      <h3 class="makosh-state-title">{{ title }}</h3>
      <p v-if="description" class="makosh-state-description">{{ description }}</p>
    </div>
    <div v-if="$slots.action" class="makosh-state-action">
      <slot name="action" />
    </div>
  </section>
</template>
