import type { Meta, StoryObj } from '@storybook/vue3-vite'
import { IconButton } from '@/shared/ui'
import { storybookLocaleFromGlobals } from './storybook-i18n'
import { generalStoryCopy } from './general-story-copy'

const meta = {
	title: 'Макошь UI/General/Icon Button',
	component: IconButton,
	render: (_args, context) => ({
		components: { IconButton },
		setup() {
			return { copy: generalStoryCopy(storybookLocaleFromGlobals(context.globals)) }
		},
		template: `
			<section class="storybook-canvas">
				<div class="storybook-section">
					<h2>{{ copy.controls.iconButton }}</h2>
					<div class="storybook-row">
						<IconButton icon="tabler:search" :label="copy.controls.searchInput" variant="outline" />
						<IconButton icon="tabler:copy" :label="copy.actions.copy" variant="secondary" />
						<IconButton icon="tabler:archive" :label="copy.actions.archive" variant="ghost" />
					</div>
				</div>
			</section>
		`
	})
} satisfies Meta<typeof IconButton>

export default meta
type Story = StoryObj<typeof meta>

export const Default: Story = {}
