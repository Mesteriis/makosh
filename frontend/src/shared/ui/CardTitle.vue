<script setup lang="ts">
import { computed } from 'vue'

const props = withDefaults(defineProps<{
  class?: string
}>(), {})

const classes = computed(() => ['makosh-card-title', props.class])
</script>

<template>
  <h3 :class="classes">
    <slot />
  </h3>
</template>

