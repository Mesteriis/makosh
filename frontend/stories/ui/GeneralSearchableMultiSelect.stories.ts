import type { Meta, StoryObj } from '@storybook/vue3-vite'
import { FormField, FormLabel, SearchableMultiSelect } from '@/shared/ui'
import { storybookLocaleFromGlobals, storybookText } from './storybook-i18n'

const meta = {
	title: 'Макошь UI/General/Searchable Multi Select',
	render: (_args, context) => ({
		components: { FormField, FormLabel, SearchableMultiSelect },
		data() {
			const text = storybookText(storybookLocaleFromGlobals(context.globals))
			return { text, value: ['communications', 'projects'] }
		},
		template: `
			<section class="storybook-canvas">
				<div class="storybook-section storybook-narrow">
					<h2>{{ text.selection.searchableMultiSelect }}</h2>
					<FormField>
						<FormLabel>{{ text.selection.searchableMultiSelect }}</FormLabel>
						<SearchableMultiSelect
							v-model="value"
							:actions-aria-label="text.selection.actionsLabel"
							:clear-all-label="text.selection.clearAll"
							:empty-label="text.selection.empty"
							:listbox-aria-label="text.selection.optionsLabel"
							:options="text.selection.options"
							:placeholder="text.selection.placeholder"
							:remove-label="(option) => text.selection.remove(option.label)"
							:search-aria-label="text.selection.searchLabel"
							:search-placeholder="text.selection.searchPlaceholder"
							:select-all-label="text.selection.selectAll"
							:selected-count-label="text.selection.selectedCount"
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
