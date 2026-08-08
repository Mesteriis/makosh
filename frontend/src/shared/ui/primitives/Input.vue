<!-- Shared form control primitive. -->
<script setup lang="ts">
import { computed } from 'vue'

const props = withDefaults(defineProps<{
  id?: string
  modelValue?: string
  placeholder?: string
  disabled?: boolean
  readonly?: boolean
  type?: string
  ariaLabel?: string
  error?: string
  class?: string
}>(), {
  modelValue: '',
  placeholder: '',
  disabled: false,
  readonly: false,
  type: 'text'
})

const emit = defineEmits<{
  'update:modelValue': [value: string]
  focus: [event: FocusEvent]
  blur: [event: FocusEvent]
}>()

const classes = computed(() => [
  'makosh-input',
  { 'makosh-input--error': props.error },
  props.class
])

function handleInput(event: Event): void {
  const target = event.target as HTMLInputElement
  emit('update:modelValue', target.value)
}

function handleFocus(event: FocusEvent): void {
  emit('focus', event)
}

function handleBlur(event: FocusEvent): void {
  emit('blur', event)
}
</script>

<template>
  <div class="makosh-input-wrapper">
    <input
      :class="classes"
      :id="id"
      :value="modelValue"
      :placeholder="placeholder"
      :disabled="disabled"
      :readonly="readonly"
      :type="type"
      :aria-label="ariaLabel"
      @input="handleInput"
      @focus="handleFocus"
      @blur="handleBlur"
    />
    <span v-if="error" class="makosh-input-error">{{ error }}</span>
  </div>
</template>
