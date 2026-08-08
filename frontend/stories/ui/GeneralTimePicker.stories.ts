import type { Meta, StoryObj } from '@storybook/vue3-vite'
import { FormField, FormLabel, TimePicker } from '@/shared/ui'
import { storybookLocaleFromGlobals } from './storybook-i18n'
import { generalStoryCopy } from './general-story-copy'

const meta = {
	title: 'Макошь UI/General/Time Picker',
	component: TimePicker,
	render: (_args, context) => ({
		components: { FormField, FormLabel, TimePicker },
		data() {
			return { copy: generalStoryCopy(storybookLocaleFromGlobals(context.globals)), value: '09:30' }
		},
		template: `
			<section class="storybook-canvas">
				<div class="storybook-section storybook-narrow">
					<h2>{{ copy.controls.timePicker }}</h2>
					<FormField>
						<FormLabel>{{ copy.controls.timePicker }}</FormLabel>
						<TimePicker v-model="value" :aria-label="copy.controls.timePicker" />
					</FormField>
				</div>
			</section>
		`
	})
} satisfies Meta<typeof TimePicker>

export default meta
type Story = StoryObj<typeof meta>

export const Default: Story = {}
