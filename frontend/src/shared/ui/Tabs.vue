<script setup lang="ts">
import { TabsRoot, TabsList, TabsTrigger } from 'reka-ui'
import { computed } from 'vue'

type МакошьTab = {
  id: string
  label: string
}

// Re-export with Макошь styling
const props = withDefaults(defineProps<{
  modelValue?: string
  active?: string
  defaultValue?: string
  orientation?: 'horizontal' | 'vertical'
  class?: string
  listClass?: string
  contentClass?: string
  tabs?: МакошьTab[]
}>(), {
  orientation: 'horizontal'
})

const emit = defineEmits<{
  'update:modelValue': [value: string]
  select: [value: string]
}>()

const tabs = computed(() => props.tabs ?? [])
const selectedValue = computed(() => props.modelValue ?? props.active)
const rootClasses = computed(() => ['makosh-tabs', props.class])
const listClasses = computed(() => ['makosh-tabs-list', `makosh-tabs-list--${props.orientation}`, props.listClass])

function handleUpdateModelValue(value: string | number) {
  const nextValue = String(value)
  emit('update:modelValue', nextValue)
  emit('select', nextValue)
}
</script>

<template>
  <TabsRoot
    :class="rootClasses"
    :model-value="selectedValue"
    :default-value="defaultValue"
    :orientation="orientation"
    @update:model-value="handleUpdateModelValue"
  >
    <TabsList :class="listClasses">
      <slot name="list">
        <TabsTrigger
          v-for="tab in tabs"
          :key="tab.id"
          :value="tab.id"
          class="makosh-tabs-trigger"
        >
          {{ tab.label }}
        </TabsTrigger>
      </slot>
    </TabsList>
    <slot />
  </TabsRoot>
</template>

