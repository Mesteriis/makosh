import type { Meta, StoryObj } from '@storybook/vue3-vite'
import { Button, Dialog } from '@/shared/ui'
import { storybookLocaleFromGlobals } from './storybook-i18n'
import { generalStoryCopy } from './general-story-copy'

const meta = {
	title: 'Макошь UI/General/Dialog',
	component: Dialog,
	render: (_args, context) => ({
		components: { Button, Dialog },
		data() {
			return { copy: generalStoryCopy(storybookLocaleFromGlobals(context.globals)), open: true }
		},
		template: `
			<section class="storybook-canvas">
				<div class="storybook-section">
					<h2>{{ copy.controls.dialog }}</h2>
					<Button @click="open = true">{{ copy.controls.dialog }}</Button>
					<Dialog v-model:open="open" :title="copy.overlay.title" :description="copy.overlay.description" :close-label="copy.actions.close">
						<p>{{ copy.overlay.body }}</p>
						<template #footer>
							<Button variant="secondary" @click="open = false">{{ copy.actions.close }}</Button>
							<Button @click="open = false">{{ copy.actions.confirm }}</Button>
						</template>
					</Dialog>
				</div>
			</section>
		`
	})
} satisfies Meta<typeof Dialog>

export default meta
type Story = StoryObj<typeof meta>

export const Default: Story = {}
