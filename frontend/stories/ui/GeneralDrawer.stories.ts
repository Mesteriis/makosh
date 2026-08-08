import type { Meta, StoryObj } from '@storybook/vue3-vite'
import { Button, Drawer } from '@/shared/ui'
import { storybookLocaleFromGlobals } from './storybook-i18n'
import { generalStoryCopy } from './general-story-copy'

const meta = {
	title: 'Макошь UI/General/Drawer',
	component: Drawer,
	render: (_args, context) => ({
		components: { Button, Drawer },
		data() {
			return { copy: generalStoryCopy(storybookLocaleFromGlobals(context.globals)), open: true }
		},
		template: `
			<section class="storybook-canvas">
				<div class="storybook-section">
					<h2>{{ copy.controls.drawer }}</h2>
					<Button variant="outline" @click="open = true">{{ copy.controls.drawer }}</Button>
					<Drawer v-model:open="open" side="right" :title="copy.overlay.title" :description="copy.overlay.description" :close-label="copy.actions.close">
						<p>{{ copy.overlay.body }}</p>
						<template #footer>
							<Button @click="open = false">{{ copy.actions.close }}</Button>
						</template>
					</Drawer>
				</div>
			</section>
		`
	})
} satisfies Meta<typeof Drawer>

export default meta
type Story = StoryObj<typeof meta>

export const Default: Story = {}
