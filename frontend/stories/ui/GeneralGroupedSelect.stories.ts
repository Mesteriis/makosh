import type { Meta, StoryObj } from '@storybook/vue3-vite'
import { FormField, FormLabel, GroupedSelect } from '@/shared/ui'
import { storybookLocaleFromGlobals, storybookText } from './storybook-i18n'

const meta = {
	title: 'Макошь UI/General/Grouped Select',
	render: (_args, context) => ({
		components: { FormField, FormLabel, GroupedSelect },
		data() {
			const text = storybookText(storybookLocaleFromGlobals(context.globals))
			return { text, value: 'documents' }
		},
		template: `
			<section class="storybook-canvas">
				<div class="storybook-section storybook-narrow">
					<h2>{{ text.selection.groupedSelect }}</h2>
					<FormField>
						<FormLabel>{{ text.selection.groupedSelect }}</FormLabel>
						<GroupedSelect
							v-model="value"
							:empty-label="text.selection.empty"
							:groups="text.selection.groups"
							:placeholder="text.selection.placeholder"
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
