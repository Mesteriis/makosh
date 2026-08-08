import type { Meta, StoryObj } from '@storybook/vue3-vite'
import { Table } from '@/shared/ui'
import { storybookLocaleFromGlobals } from './storybook-i18n'
import { generalStoryCopy } from './general-story-copy'

const meta = {
	title: 'Макошь UI/General/Table',
	component: Table,
	render: (_args, context) => ({
		components: { Table },
		setup() {
			return { copy: generalStoryCopy(storybookLocaleFromGlobals(context.globals)) }
		},
		template: `
			<section class="storybook-canvas">
				<div class="storybook-section">
					<h2>{{ copy.controls.table }}</h2>
					<Table :caption="copy.data.tableCaption" :columns="copy.data.columns" :empty-text="copy.data.empty" :rows="copy.data.rows" />
				</div>
			</section>
		`
	})
} satisfies Meta<typeof Table>

export default meta
type Story = StoryObj<typeof meta>

export const Default: Story = {}
