<script setup lang="ts">
import {
	AlertDialogAction,
	AlertDialogCancel,
	AlertDialogContent,
	AlertDialogDescription,
	AlertDialogOverlay,
	AlertDialogPortal,
	AlertDialogRoot,
	AlertDialogTitle,
	AlertDialogTrigger
} from 'reka-ui'
import { computed } from 'vue'

const props = withDefaults(defineProps<{
	open?: boolean
	title?: string
	description?: string
	cancelLabel?: string
	actionLabel?: string
	tone?: 'default' | 'danger'
	class?: string
	contentClass?: string
}>(), {
	open: false,
	tone: 'danger'
})

const emit = defineEmits<{
	'update:open': [value: boolean]
	action: []
	cancel: []
}>()

const contentClasses = computed(() => [
	'makosh-alert-dialog-content',
	`makosh-alert-dialog-content--${props.tone}`,
	props.contentClass
])
</script>

<template>
	<AlertDialogRoot :open="open" @update:open="(value) => emit('update:open', value)">
		<AlertDialogTrigger v-if="$slots.trigger" as-child>
			<slot name="trigger" />
		</AlertDialogTrigger>
		<AlertDialogPortal>
			<AlertDialogOverlay class="makosh-alert-dialog-overlay">
				<AlertDialogContent :class="contentClasses">
					<div class="makosh-alert-dialog-header">
						<AlertDialogTitle v-if="title" class="makosh-alert-dialog-title">{{ title }}</AlertDialogTitle>
						<AlertDialogDescription v-if="description" class="makosh-alert-dialog-description">
							{{ description }}
						</AlertDialogDescription>
						<slot name="header" />
					</div>
					<div v-if="$slots.default" class="makosh-alert-dialog-body">
						<slot />
					</div>
					<div class="makosh-alert-dialog-footer">
						<AlertDialogCancel as-child @click="emit('cancel')">
							<button class="makosh-alert-dialog-cancel">
								<slot name="cancel">{{ cancelLabel }}</slot>
							</button>
						</AlertDialogCancel>
						<AlertDialogAction as-child @click="emit('action')">
							<button :class="['makosh-alert-dialog-action', `makosh-alert-dialog-action--${tone}`]">
								<slot name="action">{{ actionLabel }}</slot>
							</button>
						</AlertDialogAction>
					</div>
				</AlertDialogContent>
			</AlertDialogOverlay>
		</AlertDialogPortal>
	</AlertDialogRoot>
</template>
