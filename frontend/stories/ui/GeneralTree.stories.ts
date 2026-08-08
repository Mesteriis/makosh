import type { Meta, StoryObj } from '@storybook/vue3-vite'
import { Tree } from '@/shared/ui'
import { storybookLocaleFromGlobals } from './storybook-i18n'
import { generalStoryCopy } from './general-story-copy'

const meta = {
	title: 'Макошь UI/General/Tree',
	component: Tree,
	render: (_args, context) => ({
		components: { Tree },
		data() {
			const copy = generalStoryCopy(storybookLocaleFromGlobals(context.globals))
			return { copy, selected: 'review', expanded: ['workspace'] }
		},
		template: `
			<section class="storybook-canvas">
				<div class="storybook-section storybook-narrow">
					<h2>{{ copy.controls.tree }}</h2>
					<Tree v-model="selected" v-model:expanded="expanded" :items="copy.data.treeItems" :label="copy.data.treeLabel" />
				</div>
			</section>
		`
	})
} satisfies Meta<typeof Tree>

export default meta
type Story = StoryObj<typeof meta>

export const Default: Story = {}
