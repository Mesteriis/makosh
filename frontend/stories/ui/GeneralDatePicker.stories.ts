import type { Meta, StoryObj } from '@storybook/vue3-vite'
import { DatePicker, FormField, FormLabel } from '@/shared/ui'
import { storybookLocaleFromGlobals } from './storybook-i18n'
import { generalStoryCopy } from './general-story-copy'

const meta = {
	title: 'Макошь UI/General/Date Picker',
	component: DatePicker,
	render: (_args, context) => ({
		components: { DatePicker, FormField, FormLabel },
		data() {
			return { copy: generalStoryCopy(storybookLocaleFromGlobals(context.globals)), value: '2026-07-02' }
		},
		template: `
			<section class="storybook-canvas">
				<div class="storybook-section storybook-narrow">
					<h2>{{ copy.controls.datePicker }}</h2>
					<FormField>
						<FormLabel>{{ copy.controls.datePicker }}</FormLabel>
						<DatePicker v-model="value" :aria-label="copy.controls.datePicker" />
					</FormField>
				</div>
			</section>
		`
	})
} satisfies Meta<typeof DatePicker>

export default meta
type Story = StoryObj<typeof meta>

export const Default: Story = {}
