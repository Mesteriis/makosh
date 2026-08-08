<script setup lang="ts">
import { computed } from 'vue'
import Icon from './Icon.vue'
import type { ProviderIconKind } from './Utility.types'

const props = withDefaults(defineProps<{
	provider?: ProviderIconKind
	label?: string
	size?: number | string
	class?: string
}>(), {
	provider: 'generic',
	size: '1.25rem'
})

const providerIcons: Record<ProviderIconKind, string> = {
	mail: 'tabler:mail',
	telegram: 'tabler:brand-telegram',
	whatsapp: 'tabler:brand-whatsapp',
	calendar: 'tabler:calendar',
	documents: 'tabler:files',
	generic: 'tabler:plug-connected'
}

const classes = computed(() => ['makosh-provider-icon', `makosh-provider-icon--${props.provider}`, props.class])
const accessibleLabel = computed(() => props.label ?? props.provider)
</script>

<template>
	<span :class="classes" role="img" :aria-label="accessibleLabel">
		<Icon :icon="providerIcons[provider]" :size="size" />
	</span>
</template>
