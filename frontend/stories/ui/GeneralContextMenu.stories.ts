import type { Meta, StoryObj } from '@storybook/vue3-vite'
import { Button, ContextMenu } from '@/shared/ui'
import { storybookLocaleFromGlobals } from './storybook-i18n'
import { generalStoryCopy } from './general-story-copy'

const meta = {
	title: 'Макошь UI/General/Context Menu',
	component: ContextMenu,
	render: (_args, context) => ({
		components: { Button, ContextMenu },
		setup() {
			return { copy: generalStoryCopy(storybookLocaleFromGlobals(context.globals)) }
		},
		template: `
			<section class="storybook-canvas">
				<div class="storybook-section storybook-narrow">
					<h2>{{ copy.controls.contextMenu }}</h2>
					<ContextMenu :items="copy.menuItems" :label="copy.controls.contextMenu" :open-label="copy.actions.more" default-open>
						<template #trigger>
							<Button variant="outline" icon="tabler:dots">{{ copy.actions.more }}</Button>
						</template>
					</ContextMenu>
				</div>
			</section>
		`
	})
} satisfies Meta<typeof ContextMenu>

export default meta
type Story = StoryObj<typeof meta>

export const Default: Story = {}
