<script setup lang="ts">
import { computed } from 'vue'
import Icon from './Icon.vue'

type CalloutTone = 'neutral' | 'info' | 'success' | 'warning' | 'danger'

const props = withDefaults(defineProps<{
	tone?: CalloutTone
	icon?: string
	class?: string
}>(), {
	tone: 'neutral'
})

const toneIcons: Record<CalloutTone, string> = {
	neutral: 'tabler:info-circle',
	info: 'tabler:info-circle',
	success: 'tabler:check-circle',
	warning: 'tabler:alert-triangle',
	danger: 'tabler:alert-circle'
}

const classes = computed(() => [
	'makosh-callout',
	`makosh-callout--${props.tone}`,
	props.class
])

const resolvedIcon = computed(() => props.icon ?? toneIcons[props.tone])
</script>

<template>
	<section :class="classes">
		<Icon
			v-if="resolvedIcon"
			:icon="resolvedIcon"
			size="1.125rem"
			class="makosh-callout-icon"
		/>
		<div class="makosh-callout-body">
			<div v-if="$slots.title" class="makosh-callout-title">
				<slot name="title" />
			</div>
			<div class="makosh-callout-content">
				<slot />
			</div>
		</div>
		<div v-if="$slots.actions" class="makosh-callout-actions">
			<slot name="actions" />
		</div>
	</section>
</template>
