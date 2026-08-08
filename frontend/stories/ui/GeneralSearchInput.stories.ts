import type { Meta, StoryObj } from '@storybook/vue3-vite'
import { FormField, FormLabel, SearchInput } from '@/shared/ui'
import { storybookLocaleFromGlobals } from './storybook-i18n'
import { generalStoryCopy } from './general-story-copy'

const meta = {
	title: 'Макошь UI/General/Search Input',
	component: SearchInput,
	render: (_args, context) => ({
		components: { FormField, FormLabel, SearchInput },
		data() {
			const copy = generalStoryCopy(storybookLocaleFromGlobals(context.globals))
			return { copy, value: 'evidence' }
		},
		template: `
			<section class="storybook-canvas">
				<div class="storybook-section storybook-narrow">
					<h2>{{ copy.controls.searchInput }}</h2>
					<FormField>
						<FormLabel>{{ copy.controls.searchInput }}</FormLabel>
						<SearchInput v-model="value" :aria-label="copy.controls.searchInput" :placeholder="copy.form.searchPlaceholder" />
					</FormField>
				</div>
			</section>
		`
	})
} satisfies Meta<typeof SearchInput>

export default meta
type Story = StoryObj<typeof meta>

export const Default: Story = {}
