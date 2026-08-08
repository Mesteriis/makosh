import type { Meta, StoryObj } from '@storybook/vue3-vite'
import { Cascader, FormField, FormLabel } from '@/shared/ui'
import { storybookLocaleFromGlobals, storybookText } from './storybook-i18n'

const meta = {
	title: 'Макошь UI/General/Cascader',
	render: (_args, context) => ({
		components: { Cascader, FormField, FormLabel },
		data() {
			const text = storybookText(storybookLocaleFromGlobals(context.globals))
			return { text, value: '' }
		},
		template: `
			<section class="storybook-canvas">
				<div class="storybook-section storybook-narrow">
					<h2>{{ text.selection.cascader }}</h2>
					<FormField>
						<FormLabel>{{ text.selection.cascader }}</FormLabel>
						<Cascader
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
