import type { Meta, StoryObj } from '@storybook/vue3-vite'

import ZulipOperationalPage from '../../src/integrations/zulip/presentation/ZulipOperationalPage.vue'
import type { ZulipOperationalPageModel } from '../../src/integrations/zulip/presentation/zulipOperationalPageModel'
import type { ZulipOperationalReadModel } from '../../src/integrations/zulip/presentation/zulipOperationalReadModel'
import type { ZulipOperationalReplayModel } from '../../src/integrations/zulip/presentation/zulipOperationalReplayModel'

const meta = {
	title: 'Макошь App/Communications/Zulip Operational',
	component: ZulipOperationalPage,
	parameters: { layout: 'fullscreen' },
} satisfies Meta

export default meta
type Story = StoryObj<typeof meta>

const model: ZulipOperationalPageModel = {
	destination: 'stream',
	accountId: 'zulip-owner-primary',
	stream: 'engineering',
	topic: 'clean-room',
	recipients: 'owner@example.com, team@example.com',
	content: 'The provider-owned command boundary is ready.',
	operationId: '75f8f53e-2eb2-48a1-8673-13b937e691c7',
	busy: false,
	canCommand: true,
	notice: '',
	status: {
		operationId: '75f8f53e-2eb2-48a1-8673-13b937e691c7',
		accountId: 'zulip-owner-primary',
		outcome: 'completed',
		providerMessageId: '825149',
		requestedAt: 'Jul 26, 2026, 10:42',
		completedAt: 'Jul 26, 2026, 10:42',
	},
}

const readModel: ZulipOperationalReadModel = {
	canQuery: true,
	state: 'ready',
	statusMessage: '',
	accounts: [{ id: 'zulip-owner-primary', label: 'zulip-owner-primary' }],
	selectedAccountId: 'zulip-owner-primary',
	selectedConversationId: 'engineering:clean-room',
	searchQuery: '',
	accountStatus: null,
	conversations: [],
	messages: [],
	events: [],
	searchResults: [],
	hasMoreConversations: false,
	hasMoreMessages: false,
	hasMoreEvents: false,
	hasMoreSearchResults: false,
}

const replayModel: ZulipOperationalReplayModel = {
	canReplay: true,
	state: 'ready',
	statusMessage: '',
	accounts: [{ id: 'zulip-owner-primary', label: 'zulip-owner-primary' }],
	selectedAccountId: 'zulip-owner-primary',
	earliestSequence: '1',
	latestSequence: '9',
	nextSequence: '10',
	resetRequired: false,
	frames: [],
	hasMore: false,
}

export const Default: Story = {
	args: { model, readModel, replayModel },
}
