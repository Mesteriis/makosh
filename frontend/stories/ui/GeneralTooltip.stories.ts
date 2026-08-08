import type { Meta, StoryObj } from '@storybook/vue3-vite'
import { Button, Tooltip } from '@/shared/ui'
import { storybookLocaleFromGlobals } from './storybook-i18n'
import { generalStoryCopy } from './general-story-copy'

const meta = {
	title: 'Макошь UI/General/Tooltip',
	component: Tooltip,
	render: (_args, context) => ({
		components: { Button, Tooltip },
		setup() {
			return { copy: generalStoryCopy(storybookLocaleFromGlobals(context.globals)) }
		},
		template: `
			<section class="storybook-canvas">
				<div class="storybook-section">
					<h2>{{ copy.controls.tooltip }}</h2>
					<Tooltip :content="copy.overlay.tooltip" :delay-duration="0">
						<template #trigger>
							<Button variant="outline" icon="tabler:info-circle">{{ copy.overlay.tooltip }}</Button>
						</template>
					</Tooltip>
				</div>
			</section>
		`
	})
} satisfies Meta<typeof Tooltip>

export default meta
type Story = StoryObj<typeof meta>

export const Default: Story = {}
