<script setup lang="ts">
import { TabsContent } from 'reka-ui'
import { computed } from 'vue'

const props = withDefaults(defineProps<{
  value: string
  class?: string
  forceMount?: boolean
}>(), {
  forceMount: false
})

const classes = computed(() => ['makosh-tab-content', props.class])
</script>

<template>
  <TabsContent
    :class="classes"
    :value="value"
    :force-mount="forceMount"
  >
    <slot />
  </TabsContent>
</template>

