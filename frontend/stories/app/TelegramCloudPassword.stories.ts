import type { Meta, StoryObj } from '@storybook/vue3-vite'
import { expect, userEvent, within } from 'storybook/test'

import TelegramCloudPasswordForm from '../../src/integrations/telegram/presentation/TelegramCloudPasswordForm.vue'

const meta = {
	title: 'Макошь App/Telegram/Cloud Password',
	component: TelegramCloudPasswordForm,
	parameters: { layout: 'centered' },
} satisfies Meta

export default meta
type Story = StoryObj<typeof meta>

export const Default: Story = {
	args: {
		modelValue: 'development-example',
		hint: 'family name',
		message: 'Telegram requires the account cloud password to finish linking.',
		messageTone: 'neutral',
	},
	play: async ({ canvasElement }) => {
		const canvas = within(canvasElement)
		const password = canvas.getByLabelText('Password')
		await expect(password).toHaveAttribute('type', 'password')
		await userEvent.click(canvas.getByRole('button', { name: 'Show Telegram cloud password' }))
		await expect(password).toHaveAttribute('type', 'text')
		await userEvent.click(canvas.getByRole('button', { name: 'Hide Telegram cloud password' }))
		await expect(password).toHaveAttribute('type', 'password')
	},
}
