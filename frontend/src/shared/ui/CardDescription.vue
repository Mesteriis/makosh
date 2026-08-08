<script setup lang="ts">
import { computed } from 'vue'

const props = withDefaults(defineProps<{
  class?: string
}>(), {})

const classes = computed(() => ['makosh-card-description', props.class])
</script>

<template>
  <p :class="classes">
    <slot />
  </p>
</template>

