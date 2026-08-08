<script setup lang="ts">
import { DropdownMenuItem } from 'reka-ui'
import { computed } from 'vue'
import Icon from './Icon.vue'

const props = withDefaults(defineProps<{
  icon?: string
  disabled?: boolean
  class?: string
  inset?: boolean
}>(), {
  disabled: false,
  inset: false
})

const classes = computed(() => [
  'makosh-dropdown-item',
  { 'makosh-dropdown-item--inset': props.inset, 'makosh-dropdown-item--disabled': props.disabled },
  props.class
])
</script>

<template>
  <DropdownMenuItem :class="classes" :disabled="disabled">
    <Icon v-if="icon" :icon="icon" size="1rem" class="makosh-dropdown-item-icon" />
    <slot />
  </DropdownMenuItem>
</template>

