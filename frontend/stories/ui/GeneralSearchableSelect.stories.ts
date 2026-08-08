import type { Meta, StoryObj } from '@storybook/vue3-vite'
import { FormField, FormLabel, SearchableSelect } from '@/shared/ui'
import { storybookLocaleFromGlobals, storybookText } from './storybook-i18n'

const meta = {
	title: 'Макошь UI/General/Searchable Select',
	render: (_args, context) => ({
		components: { FormField, FormLabel, SearchableSelect },
		data() {
			const text = storybookText(storybookLocaleFromGlobals(context.globals))
			return { text, value: 'knowledge' }
		},
		template: `
			<section class="storybook-canvas">
				<div class="storybook-section storybook-narrow">
					<h2>{{ text.selection.searchableSelect }}</h2>
					<FormField>
						<FormLabel>{{ text.selection.searchableSelect }}</FormLabel>
						<SearchableSelect
							v-model="value"
							:clear-label="text.selection.clear"
							:empty-label="text.selection.empty"
							:options="text.selection.options"
							:placeholder="text.selection.placeholder"
							:search-aria-label="text.selection.searchLabel"
							:search-placeholder="text.selection.searchPlaceholder"
						/>
					</FormField>
				</div>
			</section>
		`
	})
} satisfies Meta

export default meta
type Story = StoryObj<typeof meta>

export const Default: Story = {}
