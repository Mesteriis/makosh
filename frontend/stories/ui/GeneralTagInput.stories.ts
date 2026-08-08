import type { Meta, StoryObj } from '@storybook/vue3-vite'
import { FormField, FormLabel, TagInput } from '@/shared/ui'
import { storybookLocaleFromGlobals } from './storybook-i18n'
import { generalStoryCopy } from './general-story-copy'

const meta = {
	title: 'Макошь UI/General/Tag Input',
	component: TagInput,
	render: (_args, context) => ({
		components: { FormField, FormLabel, TagInput },
		data() {
			const copy = generalStoryCopy(storybookLocaleFromGlobals(context.globals))
			return { copy, value: copy.tags }
		},
		template: `
			<section class="storybook-canvas">
				<div class="storybook-section storybook-narrow">
					<h2>{{ copy.controls.tagInput }}</h2>
					<FormField>
						<FormLabel>{{ copy.controls.tagInput }}</FormLabel>
						<TagInput
							v-model="value"
							:aria-label="copy.controls.tagInput"
							:placeholder="copy.form.tagsPlaceholder"
							:remove-label="copy.form.remove"
							:suggestions="copy.tagSuggestions"
						/>
					</FormField>
				</div>
			</section>
		`
	})
} satisfies Meta<typeof TagInput>

export default meta
type Story = StoryObj<typeof meta>

export const Default: Story = {}
