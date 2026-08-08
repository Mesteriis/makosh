import type { Meta, StoryObj } from '@storybook/vue3-vite'
import { Button, Popover } from '@/shared/ui'
import { storybookLocaleFromGlobals } from './storybook-i18n'
import { generalStoryCopy } from './general-story-copy'

const meta = {
	title: 'Макошь UI/General/Popover',
	component: Popover,
	render: (_args, context) => ({
		components: { Button, Popover },
		data() {
			return { copy: generalStoryCopy(storybookLocaleFromGlobals(context.globals)), open: true }
		},
		template: `
			<section class="storybook-canvas">
				<div class="storybook-section">
					<h2>{{ copy.controls.popover }}</h2>
					<Popover v-model:open="open" :close-label="copy.actions.close">
						<template #trigger>
							<Button variant="outline" icon="tabler:info-circle">{{ copy.controls.popover }}</Button>
						</template>
						<p>{{ copy.overlay.popoverBody }}</p>
					</Popover>
				</div>
			</section>
		`
	})
} satisfies Meta<typeof Popover>

export default meta
type Story = StoryObj<typeof meta>

export const Default: Story = {}
