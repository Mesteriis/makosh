import type { Meta, StoryObj } from '@storybook/vue3-vite'
import { List } from '@/shared/ui'
import { storybookLocaleFromGlobals } from './storybook-i18n'
import { generalStoryCopy } from './general-story-copy'

const meta = {
	title: 'Макошь UI/General/List',
	component: List,
	render: (_args, context) => ({
		components: { List },
		setup() {
			return { copy: generalStoryCopy(storybookLocaleFromGlobals(context.globals)) }
		},
		template: `
			<section class="storybook-canvas">
				<div class="storybook-section storybook-narrow">
					<h2>{{ copy.controls.list }}</h2>
					<List :items="copy.data.listItems" :label="copy.data.listLabel" />
				</div>
			</section>
		`
	})
} satisfies Meta<typeof List>

export default meta
type Story = StoryObj<typeof meta>

export const Default: Story = {}
