<script setup lang="ts">
import { computed } from 'vue'
import Icon from './Icon.vue'
import type { DataListItem } from './DataDisplay.types'

const props = withDefaults(defineProps<{
  items: DataListItem[]
  label?: string
  visibleStart?: number
  visibleCount?: number
  emptyText?: string
  class?: string
}>(), {
  visibleStart: 0,
  visibleCount: 6,
  emptyText: 'No items'
})

const safeStart = computed(() => Math.max(0, Math.min(props.visibleStart, props.items.length)))
const safeEnd = computed(() => Math.min(props.items.length, safeStart.value + Math.max(0, props.visibleCount)))
const visibleItems = computed(() => props.items.slice(safeStart.value, safeEnd.value))
const rangeLabel = computed(() => {
  if (props.items.length === 0) return props.emptyText
  return `${safeStart.value + 1}-${safeEnd.value} / ${props.items.length}`
})
</script>

<template>
  <section :class="['makosh-virtual-list', props.class]" :aria-label="label">
    <div class="makosh-virtual-meta">{{ rangeLabel }}</div>
    <ul class="makosh-list makosh-list--compact" role="list" :aria-label="label">
      <li v-if="visibleItems.length === 0" class="makosh-list-empty">{{ emptyText }}</li>
      <template v-else>
        <li
          v-for="(item, index) in visibleItems"
          :key="item.id"
          :aria-posinset="safeStart + index + 1"
          :aria-setsize="items.length"
          :class="['makosh-list-item', `makosh-list-item--${item.tone ?? 'neutral'}`]"
        >
          <Icon v-if="item.icon" :icon="item.icon" size="1rem" class="makosh-list-icon" />
          <div class="makosh-list-copy">
            <strong class="makosh-list-label">{{ item.label }}</strong>
            <span v-if="item.description" class="makosh-list-description">{{ item.description }}</span>
          </div>
          <span v-if="item.meta" class="makosh-list-meta">{{ item.meta }}</span>
        </li>
      </template>
    </ul>
  </section>
</template>
