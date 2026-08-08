<script setup lang="ts">
import { TooltipProvider, TooltipRoot, TooltipTrigger, TooltipPortal, TooltipContent, TooltipArrow } from 'reka-ui'
import { computed } from 'vue'

const props = withDefaults(defineProps<{
  content?: string
  side?: 'top' | 'bottom' | 'left' | 'right'
  sideOffset?: number
  delayDuration?: number
  class?: string
}>(), {
  side: 'top',
  sideOffset: 4,
  delayDuration: 400
})

const contentClasses = computed(() => ['makosh-tooltip-content', props.class])
</script>

<template>
  <TooltipProvider :delay-duration="delayDuration">
    <TooltipRoot>
      <TooltipTrigger as-child>
        <slot name="trigger" />
      </TooltipTrigger>
      <TooltipPortal>
        <TooltipContent :class="contentClasses" :side="side" :side-offset="sideOffset">
          <slot>{{ content }}</slot>
          <TooltipArrow class="makosh-tooltip-arrow" />
        </TooltipContent>
      </TooltipPortal>
    </TooltipRoot>
  </TooltipProvider>
</template>
