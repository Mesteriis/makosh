<!-- Shared form control primitive. -->
<script setup lang="ts">
import { SwitchRoot, SwitchThumb } from 'reka-ui'
import { computed } from 'vue'

const props = withDefaults(defineProps<{
  modelValue?: boolean
  ariaLabel?: string
  disabled?: boolean
  class?: string
}>(), {
  modelValue: false,
  disabled: false
})

const emit = defineEmits<{
  'update:modelValue': [value: boolean]
}>()

const rootClasses = computed(() => ['makosh-switch', { 'makosh-switch--disabled': props.disabled }, props.class])
const accessibleLabel = computed(() => props.ariaLabel ?? 'Toggle')
</script>

<template>
  <SwitchRoot
    :class="rootClasses"
    :aria-label="accessibleLabel"
    :checked="modelValue"
    :disabled="disabled"
    @update:checked="(val: boolean) => emit('update:modelValue', val)"
  >
    <SwitchThumb class="makosh-switch-thumb" />
  </SwitchRoot>
</template>
