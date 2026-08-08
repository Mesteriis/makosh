<script setup lang="ts">
import { computed } from 'vue'
import Skeleton from './Skeleton.vue'

const props = withDefaults(defineProps<{
  title?: string
  description?: string
  rows?: number
  fill?: boolean
  class?: string
}>(), {
  title: '',
  description: '',
  rows: 5,
  fill: true
})

const classes = computed(() => [
  'makosh-skeleton-panel',
  { 'makosh-skeleton-panel--fill': props.fill },
  props.class
])
</script>

<template>
  <section :class="classes" aria-busy="true" role="status">
    <header v-if="title || description" class="makosh-skeleton-panel__header">
      <strong v-if="title">{{ title }}</strong>
      <p v-if="description">{{ description }}</p>
    </header>

    <div class="makosh-skeleton-panel__body">
      <Skeleton height="64px" />
      <div
        v-for="row in rows"
        :key="row"
        class="makosh-skeleton-panel__row"
      >
        <Skeleton width="44px" height="44px" rounded />
        <span>
          <Skeleton :width="row % 2 === 0 ? '78%' : '92%'" height="14px" />
          <Skeleton :width="row % 3 === 0 ? '52%' : '68%'" height="11px" />
        </span>
      </div>
      <Skeleton height="112px" />
    </div>
  </section>
</template>
