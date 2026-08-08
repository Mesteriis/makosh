<script setup lang="ts">
withDefaults(defineProps<{
  duration?: number
  mode?: 'in-out' | 'out-in'
  appear?: boolean
}>(), {
  duration: 200,
  mode: 'out-in',
  appear: false
})
</script>

<template>
  <Transition
    :name="'makosh-fade'"
    :mode="mode"
    :appear="appear"
  >
    <slot />
  </Transition>
</template>
