<script setup lang="ts">
import { computed } from 'vue'

const props = withDefaults(defineProps<{
  id?: string
  modelValue?: string
  placeholder?: string
  ariaLabel?: string
  disabled?: boolean
  rows?: number
  error?: string
  class?: string
}>(), {
  modelValue: '',
  placeholder: '',
  disabled: false,
  rows: 3
})

const emit = defineEmits<{
  'update:modelValue': [value: string]
}>()

const classes = computed(() => [
  'makosh-textarea',
  { 'makosh-textarea--error': props.error },
  props.class
])

function handleInput(event: Event): void {
  const target = event.target as HTMLTextAreaElement
  emit('update:modelValue', target.value)
}
</script>

<template>
  <div class="makosh-textarea-wrapper">
    <textarea
      :class="classes"
      :id="id"
      :value="modelValue"
      :aria-label="ariaLabel"
      :placeholder="placeholder"
      :disabled="disabled"
      :rows="rows"
      @input="handleInput"
    />
    <span v-if="error" class="makosh-textarea-error">{{ error }}</span>
  </div>
</template>
