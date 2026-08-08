<script setup lang="ts">
// Shared viewport shell primitive; application composition stays in src/app.
import { computed, useSlots } from 'vue'

type AppLayoutDensity = 'compact' | 'default'
type AppLayoutMode = 'workspace' | 'focus'

const props = withDefaults(defineProps<{
	as?: string
	density?: AppLayoutDensity
	mode?: AppLayoutMode
	rail?: boolean
	sidebar?: boolean
	inspector?: boolean
	footer?: boolean
	class?: string
}>(), {
	as: 'div',
	density: 'default',
	mode: 'workspace',
	rail: true,
	sidebar: true,
	inspector: true,
	footer: true
})

const slots = useSlots()

const hasRail = computed(() => props.rail && Boolean(slots.rail))
const hasSidebar = computed(() => props.sidebar && props.mode !== 'focus' && Boolean(slots.sidebar))
const hasInspector = computed(() => props.inspector && Boolean(slots.inspector))
const hasTopbar = computed(() => Boolean(slots.topbar))
const hasFooter = computed(() => props.footer && Boolean(slots.footer))
</script>

<template>
	<component
		:is="props.as"
		:class="[
			'makosh-app-layout',
			`makosh-app-layout--${props.density}`,
			`makosh-app-layout--${props.mode}`,
			{
				'makosh-app-layout--has-rail': hasRail,
				'makosh-app-layout--has-sidebar': hasSidebar,
				'makosh-app-layout--has-inspector': hasInspector,
				'makosh-app-layout--has-topbar': hasTopbar,
				'makosh-app-layout--has-footer': hasFooter
			},
			props.class
		]"
	>
		<div v-if="hasRail" class="makosh-app-layout__rail">
			<slot name="rail" />
		</div>

		<aside v-if="hasSidebar" class="makosh-app-layout__sidebar">
			<slot name="sidebar" />
		</aside>

		<section class="makosh-app-layout__workspace">
			<header v-if="hasTopbar" class="makosh-app-layout__topbar">
				<slot name="topbar" />
			</header>

			<main class="makosh-app-layout__main">
				<slot />
			</main>

			<footer v-if="hasFooter" class="makosh-app-layout__footer">
				<slot name="footer" />
			</footer>
		</section>

		<aside v-if="hasInspector" class="makosh-app-layout__inspector">
			<slot name="inspector" />
		</aside>
	</component>
</template>
