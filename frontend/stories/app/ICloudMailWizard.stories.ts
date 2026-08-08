import type { Meta, StoryObj } from '@storybook/vue3-vite'
import { createWizardStory, wizardStoryModels } from './wizardStory'

const meta = {
	title: 'Макошь App/Wizard/iCloud Mail'
} satisfies Meta

export default meta
type Story = StoryObj<typeof meta>

export const Default: Story = {
	render: () => createWizardStory(wizardStoryModels.icloud)
}
