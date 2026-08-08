<script setup lang="ts">
const props = withDefaults(defineProps<{
  direction?: 'up' | 'down' | 'left' | 'right'
  duration?: number
  distance?: string
  mode?: 'in-out' | 'out-in'
  appear?: boolean
}>(), {
  direction: 'up',
  duration: 200,
  distance: '1rem',
  mode: 'out-in',
  appear: false
})
</script>

<template>
  <Transition
    :name="`makosh-slide-${props.direction}`"
    :mode="mode"
    :appear="appear"
  >
    <slot />
  </Transition>
</template>
