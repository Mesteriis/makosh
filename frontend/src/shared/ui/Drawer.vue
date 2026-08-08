<script setup lang="ts">
import {
	DialogClose,
	DialogContent,
	DialogDescription,
	DialogOverlay,
	DialogPortal,
	DialogRoot,
	DialogTitle,
	DialogTrigger
} from 'reka-ui'
import { computed } from 'vue'
import Icon from './Icon.vue'

const props = withDefaults(defineProps<{
	open?: boolean
	title?: string
	description?: string
	closeLabel?: string
	side?: 'left' | 'right' | 'bottom'
	size?: 'compact' | 'default' | 'wide'
	class?: string
	contentClass?: string
}>(), {
	open: false,
	closeLabel: 'Close drawer',
	side: 'bottom',
	size: 'default'
})

const emit = defineEmits<{
	'update:open': [value: boolean]
}>()

const contentClasses = computed(() => [
	'makosh-drawer-content',
	`makosh-drawer--${props.side}`,
	`makosh-drawer--${props.size}`,
	props.contentClass
])
</script>

<template>
	<DialogRoot :open="open" @update:open="(value) => emit('update:open', value)">
		<DialogTrigger v-if="$slots.trigger" as-child>
			<slot name="trigger" />
		</DialogTrigger>
		<DialogPortal>
			<DialogOverlay class="makosh-drawer-overlay">
				<DialogContent :class="contentClasses">
					<div class="makosh-drawer-handle" aria-hidden="true" />
					<header class="makosh-drawer-header">
						<DialogTitle v-if="title" class="makosh-drawer-title">{{ title }}</DialogTitle>
						<DialogDescription v-if="description" class="makosh-drawer-description">
							{{ description }}
						</DialogDescription>
						<slot name="header" />
					</header>
					<div class="makosh-drawer-body">
						<slot />
					</div>
					<footer v-if="$slots.footer" class="makosh-drawer-footer">
						<slot name="footer" />
					</footer>
					<DialogClose class="makosh-drawer-close" as-child>
						<button class="makosh-drawer-close-btn" type="button" :aria-label="closeLabel">
							<Icon icon="tabler:x" size="1.125rem" />
						</button>
					</DialogClose>
				</DialogContent>
			</DialogOverlay>
		</DialogPortal>
	</DialogRoot>
</template>
