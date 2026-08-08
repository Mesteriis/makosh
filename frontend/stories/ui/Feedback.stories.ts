import type { Meta, StoryObj } from '@storybook/vue3-vite'
import { expect, within } from 'storybook/test'
import {
	Alert,
	Banner,
	Button,
	CircularProgress,
	InlineMessage,
	LoadingOverlay,
	Notification,
	PresenceIndicator,
	ProgressBar,
	Spinner,
	StatusIndicator
} from '@/shared/ui'
import { storybookLocaleFromGlobals, storybookText } from './storybook-i18n'

const meta = {
	title: 'Макошь UI/General/Feedback',
	render: (_args, context) => ({
		components: {
			Alert,
			Banner,
			Button,
			CircularProgress,
			InlineMessage,
			LoadingOverlay,
			Notification,
			PresenceIndicator,
			ProgressBar,
			Spinner,
			StatusIndicator
		},
		data() {
			const text = storybookText(storybookLocaleFromGlobals(context.globals))
			return {
				text,
				statuses: text.feedback.statuses,
				presences: text.feedback.presences
			}
		},
		template: `
			<section class="storybook-canvas">
				<div class="storybook-section">
					<h2>{{ text.feedback.title }}</h2>
					<p>{{ text.feedback.description }}</p>
					<Banner
						tone="info"
						:title="text.feedback.bannerTitle"
						:description="text.feedback.bannerDescription"
					>
						<template #action>
							<Button size="sm" variant="outline">{{ text.feedback.action }}</Button>
						</template>
					</Banner>
				</div>

				<div class="storybook-grid">
					<div class="storybook-section">
						<h3>{{ text.feedback.surfacesTitle }}</h3>
						<Notification
							tone="success"
							dismissible
							:title="text.feedback.notificationTitle"
							:description="text.feedback.notificationDescription"
						/>
						<Alert
							tone="warning"
							:title="text.feedback.alertTitle"
							:description="text.feedback.alertDescription"
						/>
						<InlineMessage tone="success" :message="text.feedback.inlineSuccess" />
						<InlineMessage tone="warning" :message="text.feedback.inlineWarning" />
					</div>

					<div class="storybook-section">
						<h3>{{ text.feedback.loadingTitle }}</h3>
						<p>{{ text.feedback.loadingDescription }}</p>
						<ProgressBar
							:value="72"
							tone="success"
							show-value
							:label="text.feedback.progressLabel"
						/>
						<div class="storybook-row">
							<CircularProgress
								:value="64"
								tone="accent"
								:label="text.feedback.circularLabel"
							/>
							<Spinner :label="text.feedback.overlayLabel" />
						</div>
						<LoadingOverlay
							:label="text.feedback.overlayLabel"
							:description="text.feedback.overlayDescription"
						/>
					</div>
				</div>

				<div class="storybook-grid">
					<div class="storybook-section">
						<h3>{{ text.feedback.statusTitle }}</h3>
						<div class="storybook-row">
							<StatusIndicator
								v-for="status in statuses"
								:key="status.label"
								:tone="status.tone"
								:label="status.label"
								:pulse="status.pulse"
							/>
						</div>
					</div>

					<div class="storybook-section">
						<h3>{{ text.feedback.presenceTitle }}</h3>
						<div class="storybook-row">
							<PresenceIndicator
								v-for="presence in presences"
								:key="presence.label"
								:status="presence.status"
								:label="presence.label"
							/>
						</div>
					</div>
				</div>

			</section>
		`
	}),
	play: async ({ canvasElement, globals }) => {
		const text = storybookText(storybookLocaleFromGlobals(globals))
		const canvas = within(canvasElement)

		await expect(canvas.getByText(text.feedback.title)).toBeVisible()
		await expect(canvas.getByLabelText(text.feedback.progressLabel)).toBeVisible()
		const overlayLabels = canvas.getAllByText(text.feedback.overlayLabel)
		await expect(overlayLabels[overlayLabels.length - 1]).toBeVisible()
	}
} satisfies Meta

export default meta
type Story = StoryObj<typeof meta>

export const Default: Story = {}
