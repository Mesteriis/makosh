import type { Meta, StoryObj } from '@storybook/vue3-vite'
import { FormField, FormLabel, TreeSelect } from '@/shared/ui'
import { storybookLocaleFromGlobals, storybookText } from './storybook-i18n'

const meta = {
	title: 'Макошь UI/General/Tree Select',
	render: (_args, context) => ({
		components: { FormField, FormLabel, TreeSelect },
		data() {
			const text = storybookText(storybookLocaleFromGlobals(context.globals))
			return { text, value: 'knowledge' }
		},
		template: `
			<section class="storybook-canvas">
				<div class="storybook-section storybook-narrow">
					<h2>{{ text.selection.treeSelect }}</h2>
					<FormField>
						<FormLabel>{{ text.selection.treeSelect }}</FormLabel>
						<TreeSelect
							v-model="value"
							:empty-label="text.selection.empty"
							:options="text.selection.tree"
							:placeholder="text.selection.placeholder"
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
