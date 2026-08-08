<script setup lang="ts">
import { computed } from 'vue'

const props = withDefaults(defineProps<{
  size?: 'sm' | 'md' | 'lg'
  label?: string
  decorative?: boolean
  class?: string
}>(), {
  size: 'md',
  label: 'Loading',
  decorative: false
})

const classes = computed(() => [
  'makosh-spinner',
  `makosh-spinner--${props.size}`,
  props.class
])

const role = computed(() => props.decorative ? undefined : 'status')
const ariaLabel = computed(() => props.decorative ? undefined : props.label)
</script>

<template>
  <span :class="classes" :role="role" :aria-label="ariaLabel">
    <span class="makosh-spinner-mark" aria-hidden="true" />
    <span v-if="!decorative && label" class="makosh-sr-only">{{ label }}</span>
  </span>
</template>
