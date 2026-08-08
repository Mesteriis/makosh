<script setup lang="ts">
import { computed } from 'vue'
import Table from './Table.vue'
import type { DataTableColumn, DataTableRow } from './DataDisplay.types'

const props = withDefaults(defineProps<{
  columns: DataTableColumn[]
  rows: DataTableRow[]
  caption?: string
  visibleStart?: number
  visibleCount?: number
  emptyText?: string
  class?: string
}>(), {
  visibleStart: 0,
  visibleCount: 8,
  emptyText: 'No rows'
})

const safeStart = computed(() => Math.max(0, Math.min(props.visibleStart, props.rows.length)))
const safeEnd = computed(() => Math.min(props.rows.length, safeStart.value + Math.max(0, props.visibleCount)))
const visibleRows = computed(() => props.rows.slice(safeStart.value, safeEnd.value))
const rangeLabel = computed(() => {
  if (props.rows.length === 0) return props.emptyText
  return `${safeStart.value + 1}-${safeEnd.value} / ${props.rows.length}`
})
</script>

<template>
  <section :class="['makosh-virtual-table', props.class]" :aria-label="caption">
    <div class="makosh-virtual-meta">{{ rangeLabel }}</div>
    <Table :columns="columns" :rows="visibleRows" :caption="caption" :empty-text="emptyText" density="compact" />
  </section>
</template>
