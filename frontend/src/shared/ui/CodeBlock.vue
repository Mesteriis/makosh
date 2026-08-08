<script setup lang="ts">
import { computed } from 'vue'

const props = withDefaults(defineProps<{
	code: string
	language?: string
	label?: string
	showLineNumbers?: boolean
	wrap?: boolean
	class?: string
}>(), {
	language: 'text',
	showLineNumbers: false,
	wrap: false
})

const classes = computed(() => [
	'makosh-code-block',
	{
		'makosh-code-block--wrap': props.wrap,
		'makosh-code-block--line-numbers': props.showLineNumbers
	},
	props.class
])
const lines = computed(() => props.code.split('\n'))
</script>

<template>
	<figure :class="classes">
		<figcaption v-if="label || language" class="makosh-code-block__caption">
			<span v-if="label">{{ label }}</span>
			<span class="makosh-code-block__language">{{ language }}</span>
		</figcaption>
		<pre class="makosh-code-block__pre" tabindex="0" :aria-label="label || language"><code><template v-if="showLineNumbers"><span
			v-for="(line, index) in lines"
			:key="`${index}-${line}`"
			class="makosh-code-block__line"
		><span class="makosh-code-block__line-number">{{ index + 1 }}</span><span class="makosh-code-block__line-code">{{ line }}</span>
</span></template><template v-else>{{ code }}</template></code></pre>
	</figure>
</template>
