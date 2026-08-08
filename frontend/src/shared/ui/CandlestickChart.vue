<script setup lang="ts">
import { computed } from 'vue'
import type { CandlestickPoint } from './Graphics.types'

const props = withDefaults(defineProps<{
  candles: CandlestickPoint[]
  label: string
  size?: 'sm' | 'md' | 'lg'
  class?: string
}>(), {
  size: 'md'
})

const width = 132
const height = 54
const padding = 5

const bounds = computed(() => {
  const values = props.candles.flatMap((candle) => [candle.low, candle.high, candle.open, candle.close])
  const min = values.length > 0 ? Math.min(...values) : 0
  const max = values.length > 0 ? Math.max(...values) : 1
  return { min, max, range: Math.max(max - min, 1) }
})

function yFor(value: number): number {
  return height - padding - ((value - bounds.value.min) / bounds.value.range) * (height - padding * 2)
}

const renderedCandles = computed(() => {
  const count = Math.max(props.candles.length, 1)
  const step = (width - padding * 2) / count
  const bodyWidth = Math.max(Math.min(step * 0.42, 8), 3)

  return props.candles.map((candle, index) => {
    const x = padding + step * index + step / 2
    const openY = yFor(candle.open)
    const closeY = yFor(candle.close)
    const bodyY = Math.min(openY, closeY)
    return {
      ...candle,
      x,
      bodyX: x - bodyWidth / 2,
      bodyWidth,
      wickTop: yFor(candle.high),
      wickBottom: yFor(candle.low),
      bodyY,
      bodyHeight: Math.max(Math.abs(closeY - openY), 1.5),
      direction: candle.close >= candle.open ? 'up' : 'down'
    }
  })
})

const classes = computed(() => [
  'makosh-candlestick-chart',
  `makosh-candlestick-chart--${props.size}`,
  props.class
])
</script>

<template>
  <svg :class="classes" viewBox="0 0 132 54" role="img" :aria-label="label">
    <line class="makosh-candlestick-chart__baseline" x1="0" y1="49" x2="132" y2="49" aria-hidden="true" />
    <g
      v-for="candle in renderedCandles"
      :key="candle.id"
      :class="['makosh-candlestick-chart__candle', `makosh-candlestick-chart__candle--${candle.direction}`]"
      aria-hidden="true"
    >
      <line
        class="makosh-candlestick-chart__wick"
        :x1="candle.x"
        :x2="candle.x"
        :y1="candle.wickTop"
        :y2="candle.wickBottom"
      />
      <rect
        class="makosh-candlestick-chart__body"
        :x="candle.bodyX"
        :y="candle.bodyY"
        :width="candle.bodyWidth"
        :height="candle.bodyHeight"
        rx="1.5"
      />
    </g>
  </svg>
</template>
