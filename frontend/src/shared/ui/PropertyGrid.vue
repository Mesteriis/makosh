<script setup lang="ts">
import { computed } from 'vue'
import KeyValue from './KeyValue.vue'
import type { KeyValueItem } from './DataDisplay.types'

const props = withDefaults(defineProps<{
  items: KeyValueItem[]
  columns?: 'two' | 'three'
  title?: string
  class?: string
}>(), {
  columns: 'two'
})

const classes = computed(() => [
  'makosh-property-grid',
  `makosh-property-grid--${props.columns}`,
  props.class
])
</script>

<template>
  <section :class="classes" :aria-label="title">
    <h3 v-if="title" class="makosh-display-title">{{ title }}</h3>
    <dl class="makosh-property-grid-list">
      <KeyValue
        v-for="item in items"
        :key="item.id"
        :label="item.label"
        :value="item.value"
        :description="item.description"
        :tone="item.tone"
      />
    </dl>
  </section>
</template>
