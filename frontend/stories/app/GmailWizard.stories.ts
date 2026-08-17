import type { Meta, StoryObj } from '@storybook/vue3-vite'
import { expect, within } from 'storybook/test'

import GmailOAuthSetupView from '../../src/integrations/mail/presentation/GmailOAuthSetupView.vue'

const meta = {
	title: 'Макошь App/Wizard/Gmail',
	component: GmailOAuthSetupView,
	parameters: { layout: 'centered' },
} satisfies Meta<typeof GmailOAuthSetupView>

export default meta
type Story = StoryObj<typeof meta>

export const Default: Story = {
	args: {
		stage: 'configuration',
		clientConfigured: true,
		redirectUri: 'http://127.0.0.1:5173/oauth/google/callback',
	},
	play: async ({ canvasElement }) => {
		const canvas = within(canvasElement)
		await expect(canvas.getByText(/mailbox identity are selected during OAuth/i)).toBeVisible()
		await expect(canvas.queryByLabelText('Email / username')).not.toBeInTheDocument()
		await expect(canvas.getByDisplayValue('http://127.0.0.1:5173/oauth/google/callback')).toHaveAttribute('readonly')
	},
}

export const ReadyToAuthorize: Story = {
	args: {
		stage: 'authorization',
		message: 'Gmail configuration is active. Continue with Google to grant OAuth access.',
	},
	play: async ({ canvasElement }) => {
		const canvas = within(canvasElement)
		await expect(canvas.getByRole('heading', { name: 'Continue with Google OAuth' })).toBeVisible()
		await expect(canvas.queryByRole('textbox')).not.toBeInTheDocument()
	},
}
