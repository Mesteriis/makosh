<script setup lang="ts">
import { computed } from 'vue'

const props = withDefaults(defineProps<{
  htmlFor?: string
  class?: string
}>(), {})

const classes = computed(() => ['makosh-label', props.class])
</script>

<template>
  <label :class="classes" :for="htmlFor">
    <slot />
  </label>
</template>

