<script setup lang="ts">
import { computed } from 'vue'
import Icon from './Icon.vue'
import type { EntityIconKind } from './Utility.types'

const props = withDefaults(defineProps<{
	entity?: EntityIconKind
	label?: string
	size?: number | string
	class?: string
}>(), {
	entity: 'generic',
	size: '1.25rem'
})

const entityIcons: Record<EntityIconKind, string> = {
	persona: 'tabler:user',
	person: 'tabler:user',
	organization: 'tabler:building',
	project: 'tabler:briefcase',
	task: 'tabler:checkbox',
	document: 'tabler:file-text',
	decision: 'tabler:git-branch',
	obligation: 'tabler:flag',
	knowledge: 'tabler:brain',
	event: 'tabler:calendar-event',
	generic: 'tabler:cube'
}

const classes = computed(() => ['makosh-entity-icon', `makosh-entity-icon--${props.entity}`, props.class])
const accessibleLabel = computed(() => props.label ?? props.entity)
</script>

<template>
	<span :class="classes" role="img" :aria-label="accessibleLabel">
		<Icon :icon="entityIcons[entity]" :size="size" />
	</span>
</template>
