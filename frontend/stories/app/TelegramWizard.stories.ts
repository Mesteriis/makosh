import type { Meta, StoryObj } from '@storybook/vue3-vite'
import { expect, within } from 'storybook/test'

import { telegramQrDataUrl } from '../../src/integrations/telegram/linking/telegramQrArtifact'
import TelegramQrPairingView from '../../src/integrations/telegram/presentation/TelegramQrPairingView.vue'

const qrDataUrl = await telegramQrDataUrl('tg://login?token=makosh-visual-fixture')

const meta = {
	title: 'Макошь App/Wizard/Telegram',
	component: TelegramQrPairingView,
	parameters: { layout: 'centered' },
} satisfies Meta<typeof TelegramQrPairingView>

export default meta
type Story = StoryObj<typeof meta>

export const Default: Story = {
	args: {
		state: 'waiting_qr_scan',
		qrDataUrl,
		message: 'Scan this QR code from Telegram → Settings → Devices → Link Desktop Device.',
		messageTone: 'neutral',
		admitted: true,
		configured: true,
		canRefresh: true,
	},
	play: async ({ canvasElement }) => {
		const canvas = within(canvasElement)
		await expect(canvas.getByRole('img', { name: 'Telegram authorization QR code' })).toBeVisible()
		await expect(canvas.queryByRole('textbox')).not.toBeInTheDocument()
	},
}

export const WaitingForCloudPassword: Story = {
	args: {
		state: 'waiting_password',
		password: '',
		passwordHint: 'family name',
		message: 'Telegram requires the account 2FA password to finish linking.',
		messageTone: 'neutral',
		admitted: true,
		configured: true,
		canRefresh: false,
	},
	play: async ({ canvasElement }) => {
		const password = within(canvasElement).getByLabelText('Password')
		await expect(password).toHaveAttribute('type', 'password')
	},
}
