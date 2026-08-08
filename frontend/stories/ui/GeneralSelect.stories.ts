import type { Meta, StoryObj } from '@storybook/vue3-vite'
import { FormField, FormLabel, Select } from '@/shared/ui'
import { storybookLocaleFromGlobals, storybookText } from './storybook-i18n'

const meta = {
	title: 'Макошь UI/General/Select',
	render: (_args, context) => ({
		components: { FormField, FormLabel, Select },
		data() {
			const text = storybookText(storybookLocaleFromGlobals(context.globals))
			return { text, value: 'communications' }
		},
		template: `
			<section class="storybook-canvas">
				<div class="storybook-section storybook-narrow">
					<h2>{{ text.selection.select }}</h2>
					<FormField>
						<FormLabel>{{ text.selection.select }}</FormLabel>
						<Select v-model="value" :aria-label="text.selection.select" :options="text.selection.options" />
					</FormField>
				</div>
			</section>
		`
	})
} satisfies Meta

export default meta
type Story = StoryObj<typeof meta>

export const Default: Story = {}
