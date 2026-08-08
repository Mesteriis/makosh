import type { Meta, StoryObj } from '@storybook/vue3-vite'
import { FormField, FormLabel, Input } from '@/shared/ui'
import { storybookLocaleFromGlobals } from './storybook-i18n'
import { generalStoryCopy } from './general-story-copy'

const meta = {
	title: 'Макошь UI/General/Input',
	component: Input,
	render: (_args, context) => ({
		components: { FormField, FormLabel, Input },
		data() {
			const copy = generalStoryCopy(storybookLocaleFromGlobals(context.globals))
			return { copy, value: copy.form.contextTitle }
		},
		template: `
			<section class="storybook-canvas">
				<div class="storybook-section storybook-narrow">
					<h2>{{ copy.controls.input }}</h2>
					<FormField>
						<FormLabel for="general-input">{{ copy.form.contextTitle }}</FormLabel>
						<Input id="general-input" v-model="value" :aria-label="copy.form.contextTitle" :placeholder="copy.form.contextPlaceholder" />
					</FormField>
				</div>
			</section>
		`
	})
} satisfies Meta<typeof Input>

export default meta
type Story = StoryObj<typeof meta>

export const Default: Story = {}
