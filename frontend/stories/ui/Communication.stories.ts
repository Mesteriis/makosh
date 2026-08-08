import type { Meta, StoryObj } from '@storybook/vue3-vite'
import { expect, userEvent, within } from 'storybook/test'
import {
	AttachmentChip,
	ChatInput,
	ComposerToolbar,
	DeliveryStatus,
	Mention,
	MessageBubble,
	MessageStatus,
	QuotedMessage,
	ReactionBadge,
	ReadReceipt,
	TypingIndicator
} from '@/shared/ui'
import { storybookLocaleFromGlobals, storybookText } from './storybook-i18n'

const meta = {
	title: 'Макошь UI/General/Communication',
	component: MessageBubble,
	args: {
		author: 'Макошь',
		direction: 'inbound'
	}
} satisfies Meta<typeof MessageBubble>

export default meta
type Story = StoryObj<typeof meta>

export const ThreadPrimitives: Story = {
	render: (_args, context) => ({
		components: {
			AttachmentChip,
			DeliveryStatus,
			Mention,
			MessageBubble,
			MessageStatus,
			QuotedMessage,
			ReactionBadge,
			ReadReceipt,
			TypingIndicator
		},
		data() {
			const text = storybookText(storybookLocaleFromGlobals(context.globals))
			return { text }
		},
		template: `
			<section class="storybook-canvas">
				<div class="storybook-section">
					<h2>{{ text.communication.title }}</h2>
					<p>{{ text.communication.description }}</p>
				</div>

				<div class="storybook-stack">
					<MessageBubble
						v-for="message in text.communication.messages"
						:key="message.id"
						:author="message.author"
						:direction="message.direction"
						:meta="message.meta"
						:timestamp="message.timestamp"
					>
						<p>{{ message.body }}</p>
						<QuotedMessage
							v-if="message.id === 'msg-1'"
							:author="text.communication.quoteAuthor"
							:body="text.communication.quoteBody"
						/>
						<Mention v-if="message.id === 'msg-2'" label="@Owner" icon="tabler:user" />
						<template #footer>
							<ReactionBadge
								v-for="reaction in text.communication.reactions"
								:key="reaction.label"
								:emoji="reaction.emoji"
								:count="reaction.count"
								:label="reaction.label"
								interactive
							/>
							<MessageStatus status="read" />
						</template>
					</MessageBubble>

					<div class="storybook-row">
						<AttachmentChip
							v-for="attachment in text.communication.attachments"
							:key="attachment.name"
							:name="attachment.name"
							:meta="attachment.meta"
							:icon="attachment.icon"
							:tone="attachment.tone"
							removable
						/>
					</div>

					<div class="storybook-row">
						<TypingIndicator :label="text.communication.typingLabel" />
						<ReadReceipt :items="text.communication.receipts" :label="text.communication.readLabel">
							{{ text.communication.readLabel }}
						</ReadReceipt>
					</div>

					<DeliveryStatus
						status="delivered"
						:description="text.communication.deliveryDescription"
						timestamp="09:16"
					/>
				</div>
			</section>
		`
	}),
	play: async ({ canvasElement, globals }) => {
		const text = storybookText(storybookLocaleFromGlobals(globals))
		const canvas = within(canvasElement)
		await expect(canvas.getByText(text.communication.title)).toBeVisible()
		await expect(canvas.getByText(text.communication.messages[0].body)).toBeVisible()
		await expect(canvas.getByLabelText(text.communication.typingLabel)).toBeVisible()
	}
}

export const ComposerPrimitives: Story = {
	render: (_args, context) => ({
		components: {
			AttachmentChip,
			ChatInput,
			ComposerToolbar
		},
		data() {
			const text = storybookText(storybookLocaleFromGlobals(context.globals))
			return {
				text,
				draft: text.communication.messages[1].body,
				selectedAction: ''
			}
		},
		template: `
			<section class="storybook-canvas">
				<div class="storybook-section">
					<h2>{{ text.communication.composerTitle }}</h2>
					<p>{{ text.communication.description }}</p>
				</div>

				<ChatInput
					id="storybook-chat-input"
					v-model="draft"
					:attach-label="text.communication.attach"
					:helper="text.communication.helper"
					:label="text.communication.composerLabel"
					:max-length="160"
					:placeholder="text.communication.placeholder"
					:send-label="text.communication.send"
				>
					<template #toolbar>
						<ComposerToolbar
							:actions="text.communication.actions"
							:label="text.communication.toolbarLabel"
							@select="selectedAction = $event.label"
						/>
					</template>
				</ChatInput>

				<div class="storybook-row">
					<AttachmentChip
						v-for="attachment in text.communication.attachments"
						:key="attachment.name"
						:name="attachment.name"
						:meta="attachment.meta"
						:icon="attachment.icon"
						:tone="attachment.tone"
					/>
				</div>

				<p v-if="selectedAction">{{ selectedAction }}</p>
			</section>
		`
	}),
	play: async ({ canvasElement, globals }) => {
		const text = storybookText(storybookLocaleFromGlobals(globals))
		const canvas = within(canvasElement)
		const input = canvas.getByLabelText(text.communication.composerLabel)
		await expect(input).toBeVisible()
		await userEvent.click(canvas.getByRole('button', { name: text.communication.send }))
		await expect(canvas.getByDisplayValue(text.communication.messages[1].body)).toBeVisible()
	}
}
