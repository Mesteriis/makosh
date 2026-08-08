import type { Meta, StoryObj } from '@storybook/vue3-vite'
import { FormField, FormLabel, Textarea } from '@/shared/ui'
import { storybookLocaleFromGlobals } from './storybook-i18n'
import { generalStoryCopy } from './general-story-copy'

const meta = {
	title: 'Макошь UI/General/Textarea',
	component: Textarea,
	render: (_args, context) => ({
		components: { FormField, FormLabel, Textarea },
		data() {
			const copy = generalStoryCopy(storybookLocaleFromGlobals(context.globals))
			return { copy, value: copy.form.noteValue }
		},
		template: `
			<section class="storybook-canvas">
				<div class="storybook-section storybook-narrow">
					<h2>{{ copy.controls.textarea }}</h2>
					<FormField>
						<FormLabel>{{ copy.form.noteLabel }}</FormLabel>
						<Textarea v-model="value" :aria-label="copy.form.noteLabel" :rows="4" />
					</FormField>
				</div>
			</section>
		`
	})
} satisfies Meta<typeof Textarea>

export default meta
type Story = StoryObj<typeof meta>

export const Default: Story = {}
