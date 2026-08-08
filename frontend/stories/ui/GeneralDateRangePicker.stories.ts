import type { Meta, StoryObj } from '@storybook/vue3-vite'
import { DateRangePicker, FormField, FormLabel } from '@/shared/ui'
import { storybookLocaleFromGlobals } from './storybook-i18n'
import { generalStoryCopy } from './general-story-copy'

const meta = {
	title: 'Макошь UI/General/Date Range Picker',
	component: DateRangePicker,
	render: (_args, context) => ({
		components: { DateRangePicker, FormField, FormLabel },
		data() {
			return {
				copy: generalStoryCopy(storybookLocaleFromGlobals(context.globals)),
				value: { start: '2026-07-02', end: '2026-07-09' }
			}
		},
		template: `
			<section class="storybook-canvas">
				<div class="storybook-section storybook-narrow">
					<h2>{{ copy.controls.dateRangePicker }}</h2>
					<FormField>
						<FormLabel>{{ copy.controls.dateRangePicker }}</FormLabel>
						<DateRangePicker
							v-model="value"
							:aria-label="copy.controls.dateRangePicker"
							:start-label="copy.form.dateStart"
							:end-label="copy.form.dateEnd"
						/>
					</FormField>
				</div>
			</section>
		`
	})
} satisfies Meta<typeof DateRangePicker>

export default meta
type Story = StoryObj<typeof meta>

export const Default: Story = {}
