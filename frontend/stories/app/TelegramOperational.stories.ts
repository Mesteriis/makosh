import type { Meta, StoryObj } from '@storybook/vue3-vite'

import TelegramOperationalPage from '../../src/integrations/telegram/presentation/TelegramOperationalPage.vue'
import type { TelegramAccountAccessModel } from '../../src/integrations/telegram/presentation/telegramAccountAccessModel'
import type { TelegramDiscoveryModel } from '../../src/integrations/telegram/presentation/telegramDiscoveryModel'
import type { TelegramOperationalPageModel } from '../../src/integrations/telegram/presentation/telegramOperationalPageModel'

const meta = {
	title: 'Макошь App/Communications/Telegram Operational',
	component: TelegramOperationalPage,
	parameters: { layout: 'fullscreen' },
} satisfies Meta

export default meta
type Story = StoryObj<typeof meta>

const model: TelegramOperationalPageModel = {
	accountId: 'telegram-owner-primary',
	status: 'ready',
	statusMessage: '',
	chats: [
		{ id: 'chat-architecture', title: 'Макошь Architecture', detail: '@makosh_arch · supergroup', selected: true },
		{ id: 'chat-operations', title: 'Operations', detail: 'private group', selected: false },
	],
	messages: [
		{ id: 'message-1', sender: 'Alex', body: 'Provider boundary is admitted independently.', meta: 'Jul 26, 2026, 10:42 · received', outgoing: false },
		{ id: 'message-2', sender: 'You', body: 'Great. The domain only sees neutral evidence.', meta: 'Jul 26, 2026, 10:44 · delivered', outgoing: true },
	],
	selectedChatId: 'chat-architecture',
	selectedChatTitle: 'Макошь Architecture',
	draft: 'Ship the clean-room provider surface.',
	sendPending: false,
	sendMessage: 'Accepted means queued; provider completion remains asynchronous.',
	canSend: true,
}

const accountAccess: TelegramAccountAccessModel = {
	accounts: [{
		id: 'telegram-owner-primary',
		title: 'Макошь owner',
		detail: 'Authorized user account',
		selected: true,
	}],
	selectedAccountId: 'telegram-owner-primary',
	authorizationState: 'ready',
	authorizationQrDataUrl: '',
	authorizationPasswordHint: '',
	password: '',
	provisionAccountId: '',
	provisionDisplayName: '',
	provisionExternalAccountId: '',
	statusMessage: '',
	pending: false,
	canAuthorize: true,
	canManageLifecycle: true,
	canReconfigure: true,
}

const discovery: TelegramDiscoveryModel = {
	query: '',
	statusMessage: '',
	pending: false,
	canQuery: true,
	results: [],
	history: [],
	participants: [],
	topics: [],
	folders: [],
	operations: [],
	chatState: [],
}

export const Default: Story = {
	args: { accountAccess, discovery, model },
}
