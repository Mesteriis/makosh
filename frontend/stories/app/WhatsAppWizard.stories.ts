import type { Meta, StoryObj } from '@storybook/vue3-vite'
import { expect, within } from 'storybook/test'

import WhatsAppPairingView from '../../src/integrations/whatsapp/presentation/WhatsAppPairingView.vue'

const meta = {
	title: 'Макошь App/Wizard/WhatsApp',
	component: WhatsAppPairingView,
	parameters: { layout: 'centered' },
} satisfies Meta<typeof WhatsAppPairingView>

export default meta
type Story = StoryObj<typeof meta>

export const Default: Story = {
	args: {
		nativeHostAvailable: true,
		canOpen: true,
		message: 'Open the owner-visible desktop window to scan the real provider QR.',
		messageTone: 'neutral',
	},
	play: async ({ canvasElement }) => {
		const canvas = within(canvasElement)
		await expect(canvas.getByRole('heading', { name: 'WhatsApp QR pairing' })).toBeVisible()
		await expect(canvas.getByRole('button', { name: 'Open WhatsApp QR' })).toBeEnabled()
		await expect(canvas.queryByRole('textbox')).not.toBeInTheDocument()
	},
}
