<script setup lang="ts">
import { computed } from 'vue'

const props = withDefaults(defineProps<{
  class?: string
}>(), {})

const classes = computed(() => ['makosh-card-header', props.class])
</script>

<template>
  <div :class="classes">
    <slot />
  </div>
</template>

