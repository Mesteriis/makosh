<script setup lang="ts">
import { computed } from 'vue'
import type { LayoutStatusItem } from '../Layout.types'

const props = withDefaults(defineProps<{
	as?: string
	items?: LayoutStatusItem[]
	label?: string
	class?: string
}>(), {
	as: 'footer',
	items: () => []
})

const classes = computed(() => ['makosh-status-bar', props.class])
</script>

<template>
	<component :is="as" :class="classes" :aria-label="label">
		<ul v-if="items.length > 0" class="makosh-status-bar__items" role="list">
			<li
				v-for="item in items"
				:key="item.id"
				:class="['makosh-status-bar__item', `makosh-status-bar__item--${item.tone ?? 'neutral'}`]"
			>
				<span class="makosh-status-bar__label">{{ item.label }}</span>
				<strong v-if="item.value !== undefined" class="makosh-status-bar__value">{{ item.value }}</strong>
			</li>
		</ul>
		<slot />
	</component>
</template>
