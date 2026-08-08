import type { Meta, StoryObj } from '@storybook/vue3-vite'
import { Button, ButtonGroup, IconButton } from '@/shared/ui'
import { storybookLocaleFromGlobals } from './storybook-i18n'
import { generalStoryCopy } from './general-story-copy'

const meta = {
	title: 'Макошь UI/General/Button Group',
	component: ButtonGroup,
	render: (_args, context) => ({
		components: { Button, ButtonGroup, IconButton },
		setup() {
			return { copy: generalStoryCopy(storybookLocaleFromGlobals(context.globals)) }
		},
		template: `
			<section class="storybook-canvas">
				<div class="storybook-section">
					<h2>{{ copy.controls.buttonGroup }}</h2>
					<ButtonGroup :aria-label="copy.controls.buttonGroup">
						<Button variant="secondary" icon="tabler:copy">{{ copy.actions.copy }}</Button>
						<Button variant="secondary" icon="tabler:external-link">{{ copy.actions.open }}</Button>
						<IconButton icon="tabler:dots" :label="copy.actions.more" variant="secondary" />
					</ButtonGroup>
				</div>
			</section>
		`
	})
} satisfies Meta<typeof ButtonGroup>

export default meta
type Story = StoryObj<typeof meta>

export const Default: Story = {}
