import type { Meta, StoryObj } from '@storybook/vue3-vite'
import { Badge } from '@/shared/ui'
import { storybookLocaleFromGlobals } from './storybook-i18n'
import { generalStoryCopy } from './general-story-copy'

const meta = {
	title: 'Макошь UI/Foundation/Spacing',
	render: (_args, context) => ({
		components: { Badge },
		setup() {
			return { copy: generalStoryCopy(storybookLocaleFromGlobals(context.globals)) }
		},
		template: `
			<section class="storybook-canvas">
				<div class="storybook-section">
					<h2>{{ copy.foundation.spacing }}</h2>
					<div class="storybook-spacing-scale">
						<div v-for="item in copy.foundation.spacingItems" :key="item" :class="['storybook-spacing-sample', 'storybook-spacing-sample--' + item]">
							<Badge variant="neutral">{{ item }}</Badge>
						</div>
					</div>
				</div>
			</section>
		`
	})
} satisfies Meta

export default meta
type Story = StoryObj<typeof meta>

export const Default: Story = {}
