import type { Meta, StoryObj } from '@storybook/vue3-vite'
import { FormField, FormLabel, MultiSelect } from '@/shared/ui'
import { storybookLocaleFromGlobals, storybookText } from './storybook-i18n'

const meta = {
	title: 'Макошь UI/General/Multi Select',
	render: (_args, context) => ({
		components: { FormField, FormLabel, MultiSelect },
		data() {
			const text = storybookText(storybookLocaleFromGlobals(context.globals))
			return { text, value: ['communications', 'knowledge'] }
		},
		template: `
			<section class="storybook-canvas">
				<div class="storybook-section storybook-narrow">
					<h2>{{ text.selection.multiSelect }}</h2>
					<FormField>
						<FormLabel>{{ text.selection.multiSelect }}</FormLabel>
						<MultiSelect v-model="value" :aria-label="text.selection.multiSelect" :options="text.selection.options" />
					</FormField>
				</div>
			</section>
		`
	})
} satisfies Meta

export default meta
type Story = StoryObj<typeof meta>

export const Default: Story = {}
