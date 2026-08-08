<script setup lang="ts">
import { TabsTrigger } from 'reka-ui'
import { computed } from 'vue'

const props = withDefaults(defineProps<{
  value: string
  disabled?: boolean
  class?: string
}>(), {
  disabled: false
})

const classes = computed(() => ['makosh-tab-trigger', { 'makosh-tab-trigger--disabled': props.disabled }, props.class])
</script>

<template>
  <TabsTrigger
    :class="classes"
    :value="value"
    :disabled="disabled"
  >
    <slot />
  </TabsTrigger>
</template>

