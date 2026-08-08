<script setup lang="ts">
import { computed } from 'vue'
import KeyValue from './KeyValue.vue'
import type { KeyValueItem } from './DataDisplay.types'

const props = withDefaults(defineProps<{
  items: KeyValueItem[]
  title?: string
  class?: string
}>(), {})

const classes = computed(() => [
  'makosh-description-list',
  props.class
])
</script>

<template>
  <section :class="classes" :aria-label="title">
    <h3 v-if="title" class="makosh-display-title">{{ title }}</h3>
    <dl class="makosh-description-list-grid">
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
