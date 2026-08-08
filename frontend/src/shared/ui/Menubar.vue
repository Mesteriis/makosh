<script setup lang="ts">
import { computed } from 'vue'
import Icon from './Icon.vue'
import type { NavigationItem } from './Navigation.types'

const props = withDefaults(defineProps<{
	items?: NavigationItem[]
	label?: string
	class?: string
}>(), {
	items: () => [],
	label: 'Menubar'
})

const emit = defineEmits<{
	select: [item: NavigationItem]
}>()

const classes = computed(() => ['makosh-menubar', props.class])

function selectItem(item: NavigationItem): void {
	if (!item.disabled) {
		emit('select', item)
	}
}
</script>

<template>
	<div :class="classes" role="menubar" :aria-label="label">
		<div v-for="item in items" :key="item.id" class="makosh-menubar__group">
			<button
				class="makosh-menubar__item"
				role="menuitem"
				type="button"
				:aria-haspopup="item.children?.length ? 'menu' : undefined"
				:disabled="item.disabled"
				@click="selectItem(item)"
			>
				<Icon v-if="item.icon" :icon="item.icon" size="1rem" class="makosh-menubar__icon" aria-hidden="true" />
				<span>{{ item.label }}</span>
				<Icon v-if="item.children?.length" icon="tabler:chevron-down" size="0.875rem" aria-hidden="true" />
			</button>
			<div v-if="item.children?.length" class="makosh-menubar__menu" role="menu">
				<button
					v-for="child in item.children"
					:key="child.id"
					class="makosh-menubar__menu-item"
					role="menuitem"
					type="button"
					:disabled="child.disabled"
					@click="selectItem(child)"
				>
					{{ child.label }}
				</button>
			</div>
		</div>
	</div>
</template>
