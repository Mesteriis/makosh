import type { Meta, StoryObj } from '@storybook/vue3-vite'

import WhatsAppOperationalPage from '../../src/integrations/whatsapp/presentation/WhatsAppOperationalPage.vue'
import type { WhatsAppOperationalPageModel } from '../../src/integrations/whatsapp/presentation/whatsAppOperationalPageModel'
import type { WhatsAppOperationalReadModel } from '../../src/integrations/whatsapp/presentation/whatsAppOperationalReadModel'
import type { WhatsAppOperationalReplayModel } from '../../src/integrations/whatsapp/presentation/whatsAppOperationalReplayModel'

const meta = {
	title: 'Макошь App/Communications/WhatsApp Operational',
	component: WhatsAppOperationalPage,
	parameters: { layout: 'fullscreen' },
} satisfies Meta

export default meta
type Story = StoryObj<typeof meta>

const model: WhatsAppOperationalPageModel = {
	accountId: 'whatsapp-owner-primary',
	providerChatId: '34600000000@c.us',
	draft: 'The clean-room command boundary is ready.',
	operationId: '6a80b72f-618a-4bfa-a88e-623f88d99f98',
	busy: false,
	canSend: true,
	notice: '',
	status: {
		operationId: '6a80b72f-618a-4bfa-a88e-623f88d99f98',
		accountId: 'whatsapp-owner-primary',
		state: 'completed',
		requestedAt: 'Jul 26, 2026, 10:42',
		completedAt: 'Jul 26, 2026, 10:42',
	},
}

const readModel: WhatsAppOperationalReadModel = {
	canQuery: true,
	state: 'ready',
	statusMessage: '',
	accounts: [{ id: 'whatsapp-owner-primary', label: 'whatsapp-owner-primary' }],
	selectedAccountId: 'whatsapp-owner-primary',
	selectedChatId: '34600000000@c.us',
	searchQuery: '',
	runtime: null,
	dialogs: [],
	messages: [],
	participants: [],
	events: [],
	searchResults: [],
	hasMoreDialogs: false,
	hasMoreMessages: false,
	hasMoreParticipants: false,
	hasMoreEvents: false,
	hasMoreSearchResults: false,
}

const replayModel: WhatsAppOperationalReplayModel = {
	canReplay: true,
	state: 'ready',
	statusMessage: '',
	accounts: [{ id: 'whatsapp-owner-primary', label: 'whatsapp-owner-primary' }],
	selectedAccountId: 'whatsapp-owner-primary',
	earliestSequence: '1',
	latestSequence: '12',
	nextSequence: '13',
	resetRequired: false,
	frames: [],
	hasMore: false,
}

export const Default: Story = {
	args: { model, readModel, replayModel },
}
