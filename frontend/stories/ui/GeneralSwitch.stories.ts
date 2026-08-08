import type { Meta, StoryObj } from '@storybook/vue3-vite'
import { Switch } from '@/shared/ui'
import { storybookLocaleFromGlobals } from './storybook-i18n'
import { generalStoryCopy } from './general-story-copy'

const meta = {
	title: 'Макошь UI/General/Switch',
	component: Switch,
	render: (_args, context) => ({
		components: { Switch },
		data() {
			return { copy: generalStoryCopy(storybookLocaleFromGlobals(context.globals)), value: true }
		},
		template: `
			<section class="storybook-canvas">
				<div class="storybook-section">
					<h2>{{ copy.controls.switch }}</h2>
					<div class="storybook-row">
						<Switch v-model="value" :aria-label="copy.form.sync" />
						<span>{{ copy.form.sync }}</span>
					</div>
				</div>
			</section>
		`
	})
} satisfies Meta<typeof Switch>

export default meta
type Story = StoryObj<typeof meta>

export const Default: Story = {}
