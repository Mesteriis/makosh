import type { Meta, StoryObj } from '@storybook/vue3-vite'
import { Menu } from '@/shared/ui'
import { storybookLocaleFromGlobals } from './storybook-i18n'
import { generalStoryCopy } from './general-story-copy'

const meta = {
	title: 'Макошь UI/General/Menu',
	component: Menu,
	render: (_args, context) => ({
		components: { Menu },
		data() {
			const copy = generalStoryCopy(storybookLocaleFromGlobals(context.globals))
			return { copy, active: 'open' }
		},
		template: `
			<section class="storybook-canvas">
				<div class="storybook-section storybook-narrow">
					<h2>{{ copy.controls.menu }}</h2>
					<Menu v-model="active" :items="copy.menuItems" :label="copy.controls.menu" />
				</div>
			</section>
		`
	})
} satisfies Meta<typeof Menu>

export default meta
type Story = StoryObj<typeof meta>

export const Default: Story = {}
