<script setup lang="ts">
import { computed } from 'vue'

const props = withDefaults(defineProps<{
	author?: string
	body?: string
	meta?: string
	class?: string
}>(), {})

const classes = computed(() => ['makosh-quoted-message', props.class])
</script>

<template>
	<blockquote :class="classes">
		<header v-if="author || meta" class="makosh-quoted-message__header">
			<strong v-if="author">{{ author }}</strong>
			<span v-if="meta">{{ meta }}</span>
		</header>
		<p v-if="body" class="makosh-quoted-message__body">{{ body }}</p>
		<div v-else class="makosh-quoted-message__body">
			<slot />
		</div>
	</blockquote>
</template>
