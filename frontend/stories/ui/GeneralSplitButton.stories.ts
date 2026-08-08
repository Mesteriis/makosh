import type { Meta, StoryObj } from '@storybook/vue3-vite'
import { SplitButton } from '@/shared/ui'
import { storybookLocaleFromGlobals } from './storybook-i18n'
import { generalStoryCopy } from './general-story-copy'

const meta = {
	title: 'Макошь UI/General/Split Button',
	component: SplitButton,
	render: (_args, context) => ({
		components: { SplitButton },
		data() {
			const copy = generalStoryCopy(storybookLocaleFromGlobals(context.globals))
			return { copy, selected: 'copy' }
		},
		template: `
			<section class="storybook-canvas">
				<div class="storybook-section">
					<h2>{{ copy.controls.splitButton }}</h2>
					<SplitButton
						v-model="selected"
						icon="tabler:bolt"
						:items="copy.menuItems"
						:label="copy.actions.run"
						:menu-label="copy.actions.more"
					/>
				</div>
			</section>
		`
	})
} satisfies Meta<typeof SplitButton>

export default meta
type Story = StoryObj<typeof meta>

export const Default: Story = {}
