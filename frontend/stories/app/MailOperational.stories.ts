import type { Meta, StoryObj } from '@storybook/vue3-vite'

import { MailCompositionModeV1 } from '../../src/gen/makosh/mail/composition/v1/client_pb'
import MailOperationalPage from '../../src/integrations/mail/presentation/MailOperationalPage.vue'
import type { MailCompositionModel } from '../../src/integrations/mail/presentation/mailCompositionModel'
import type { MailDeliveryModel } from '../../src/integrations/mail/presentation/mailDeliveryModel'
import type { MailMessageFlagModel } from '../../src/integrations/mail/presentation/mailMessageFlagModel'
import type { MailMessageLocationModel } from '../../src/integrations/mail/presentation/mailMessageLocationModel'
import type { MailMessagePermanentDeleteModel } from '../../src/integrations/mail/presentation/mailMessagePermanentDeleteModel'
import type { MailOperationalReadModel } from '../../src/integrations/mail/presentation/mailOperationalReadModel'
import type { MailSyncHealthModel } from '../../src/integrations/mail/presentation/mailSyncHealthModel'
import type { MailSyncModel } from '../../src/integrations/mail/presentation/mailSyncModel'

const meta = {
	title: 'Макошь App/Communications/Mail Operational',
	component: MailOperationalPage,
	parameters: { layout: 'fullscreen' },
} satisfies Meta

export default meta
type Story = StoryObj<typeof meta>

const deliveryModel: MailDeliveryModel = {
	operationId: '170d768c-d956-4963-9603-2a0f578a2db4',
	busy: false,
	canDeliver: true,
	notice: '',
	status: {
		operationId: '170d768c-d956-4963-9603-2a0f578a2db4',
		connectionId: 'gmail-primary',
		outcome: 'accepted',
		requestedAt: 'Jul 26, 2026, 10:42',
		completedAt: 'Jul 26, 2026, 10:42',
		responseCode: '202',
	},
}

const syncModel: MailSyncModel = {
	busy: false,
	canSync: true,
	notice: '',
	summary: '18 messages observed by sync-17.',
}

const compositionModel: MailCompositionModel = {
	canMutate: true,
	canQuery: true,
	status: 'ready',
	statusMessage: '',
	notice: '',
	busyAction: null,
	connections: [{ id: 'gmail-primary', label: 'gmail-primary', detail: 'Gmail · operational' }],
	selectedConnectionId: 'gmail-primary',
	drafts: [{ id: 'draft-1', label: 'Clean-room delivery boundary', detail: 'revision 7' }],
	templates: [{ id: 'template-1', label: 'Status update', detail: 'en' }],
	signatures: [{ id: 'signature-1', label: 'Owner', detail: 'default' }],
	draft: {
		draftId: 'draft-1',
		revision: '7',
		mode: MailCompositionModeV1.MAIL_COMPOSITION_MODE_NEW,
		providerConversationId: '',
		inReplyToProviderMessageId: '',
		toRecipients: 'owner@example.com',
		ccRecipients: '',
		bccRecipients: '',
		subject: 'Clean-room delivery boundary',
		textBody: 'Mail owns provider delivery. Communications receives durable evidence.',
		templateId: 'template-1',
		signatureId: 'signature-1',
	},
	template: {
		templateId: 'template-1',
		revision: '3',
		name: 'Status update',
		subjectTemplate: 'Макошь status: {{subject}}',
		textBodyTemplate: 'Current status: {{status}}',
		variables: 'subject\nstatus',
		locale: 'en',
		previewValues: 'subject=Clean-room delivery\nstatus=ready',
		previewSummary: 'Макошь status: Clean-room delivery',
	},
	signature: {
		signatureId: 'signature-1',
		revision: '2',
		name: 'Owner',
		textBody: 'Макошь owner',
		isDefault: true,
	},
}

const syncHealthModel: MailSyncHealthModel = {
	canQuery: true,
	state: 'ready',
	statusMessage: '',
	connections: [{ id: 'gmail-primary', label: 'gmail-primary' }],
	selectedConnectionId: 'gmail-primary',
	readiness: 'Ready',
	readinessTone: 'success',
	latestOutcome: 'Succeeded',
	latestOutcomeTone: 'success',
	lastSuccessAt: 'Jul 26, 2026, 10:42',
	consecutiveFailures: '0',
	projectionRevision: '17',
	runs: [
		{
			operationId: 'sync-17',
			trigger: 'Manual',
			outcome: 'Succeeded',
			outcomeTone: 'success',
			observedMessages: '18',
			startedAt: 'Jul 26, 2026, 10:41',
			completedAt: 'Jul 26, 2026, 10:42',
			failure: '',
			runtimeGeneration: '5',
			projectionRevision: '17',
		},
	],
	hasMoreRuns: false,
}

const flagModel: MailMessageFlagModel = {
	canMutate: true,
	canQueryStatus: true,
	hasSelection: true,
	isRead: false,
	isStarred: true,
	busy: false,
	status: 'idle',
	statusMessage: '',
	operationId: '',
}

const locationModel: MailMessageLocationModel = {
	canMutate: true,
	canQueryStatus: true,
	hasSelection: true,
	isTrashed: false,
	busy: false,
	status: 'idle',
	statusMessage: '',
	operationId: '',
	targetFolderId: 'archive',
	targetFolders: [{ id: 'archive', label: 'Archive' }],
}

const permanentDeleteModel: MailMessagePermanentDeleteModel = {
	canDelete: false,
	canQueryStatus: true,
	hasTrashSelection: false,
	confirmed: false,
	busy: false,
	status: 'idle',
	statusMessage: '',
	operationId: '',
}

const readModel: MailOperationalReadModel = {
	canQuery: true,
	status: 'ready',
	statusMessage: '',
	connections: [{ id: 'gmail-primary', label: 'gmail-primary' }],
	selectedConnectionId: 'gmail-primary',
	folders: [
		{ id: 'inbox', label: 'Inbox', meta: '4 unread · 18 total', selected: true },
		{ id: 'sent', label: 'Sent', meta: '0 unread · 9 total', selected: false },
		{ id: 'archive', label: 'Archive', meta: '0 unread · 31 total', selected: false },
	],
	threads: [
		{
			id: 'thread-1',
			subject: 'Clean-room boundary',
			snippet: 'The managed route now returns bounded provider evidence.',
			meta: 'Jul 26, 2026, 10:42 · 3 messages',
			selected: true,
			unread: true,
		},
		{
			id: 'thread-2',
			subject: 'Release evidence',
			snippet: 'Managed conformance passed on the disposable host contour.',
			meta: 'Jul 26, 2026, 09:18 · 2 messages',
			selected: false,
			unread: false,
		},
	],
	messages: [
		{
			id: 'message-1',
			subject: 'Clean-room boundary',
			sender: 'owner@example.com',
			snippet: 'The managed route now returns bounded provider evidence.',
			meta: 'Jul 26, 2026, 10:42',
			selected: true,
			unread: true,
			hasAttachments: true,
		},
		{
			id: 'message-2',
			subject: 'Re: Clean-room boundary',
			sender: 'team@example.com',
			snippet: 'Confirmed. Communications remains the canonical evidence owner.',
			meta: 'Jul 26, 2026, 10:37',
			selected: false,
			unread: false,
			hasAttachments: false,
		},
	],
	detail: {
		id: 'message-1',
		subject: 'Clean-room boundary',
		sender: 'owner@example.com',
		recipients: 'team@example.com',
		snippet: 'The managed route now returns bounded provider evidence.',
		meta: 'Jul 26, 2026, 10:42 · revision 7',
		folders: 'inbox',
		flags: 'Starred',
		evidenceState: 'Canonical evidence linked',
		contentState: 'Authorized body content is Communications-owned and is not part of this Mail projection.',
	},
	hasMoreFolders: false,
	hasMoreThreads: true,
	hasMoreMessages: false,
}

export const Default: Story = {
	args: {
		compositionModel,
		deliveryModel,
		flagModel,
		locationModel,
		permanentDeleteModel,
		readModel,
		syncHealthModel,
		syncModel,
	},
}
