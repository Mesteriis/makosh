import type { Meta, StoryObj } from '@storybook/vue3-vite'
import { Checkbox, FormField } from '@/shared/ui'
import { storybookLocaleFromGlobals } from './storybook-i18n'
import { generalStoryCopy } from './general-story-copy'

const meta = {
	title: 'Макошь UI/General/Checkbox',
	component: Checkbox,
	render: (_args, context) => ({
		components: { Checkbox, FormField },
		data() {
			return { copy: generalStoryCopy(storybookLocaleFromGlobals(context.globals)), value: true }
		},
		template: `
			<section class="storybook-canvas">
				<div class="storybook-section">
					<h2>{{ copy.controls.checkbox }}</h2>
					<FormField>
						<Checkbox v-model="value">{{ copy.form.enabled }}</Checkbox>
					</FormField>
				</div>
			</section>
		`
	})
} satisfies Meta<typeof Checkbox>

export default meta
type Story = StoryObj<typeof meta>

export const Default: Story = {}
