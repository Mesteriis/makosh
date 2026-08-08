import type { Meta, StoryObj } from '@storybook/vue3-vite'
import { Badge, Icon, ScrollArea, Separator, Surface, Toast } from '@/shared/ui'
import { storybookLocaleFromGlobals, storybookText } from './storybook-i18n'

const meta = {
	title: 'Макошь UI/Foundation/Tokens',
	render: (_args, context) => ({
		components: {
			Badge,
			Icon,
			ScrollArea,
			Separator,
			Surface,
			Toast
		},
		setup() {
			return { text: storybookText(storybookLocaleFromGlobals(context.globals)) }
		},
		template: `
			<section class="storybook-canvas">
				<div class="storybook-grid">
					<Surface tone="default" class="storybook-section">
						<h2>{{ text.foundation.iconTitle }}</h2>
						<p>{{ text.foundation.iconDescription }}</p>
						<div class="storybook-row">
							<Icon icon="tabler:messages" />
							<Icon icon="tabler:radar" />
							<Icon icon="tabler:brain" />
							<Icon icon="tabler:shield-check" />
							<Badge variant="accent">{{ text.foundation.sharedPrimitive }}</Badge>
						</div>
					</Surface>

					<Surface tone="raised" class="storybook-section">
						<h2>{{ text.foundation.separatorTitle }}</h2>
						<p>{{ text.foundation.separatorDescription }}</p>
						<Separator />
						<div class="storybook-row storybook-separator-sample">
							<template v-for="(item, index) in text.foundation.separatorItems" :key="item">
								<Separator v-if="index > 0" orientation="vertical" />
								<span>{{ item }}</span>
							</template>
						</div>
					</Surface>
				</div>

				<div class="storybook-grid">
					<Surface tone="deep" class="storybook-section">
						<h2>{{ text.foundation.scrollTitle }}</h2>
						<p>{{ text.foundation.scrollDescription }}</p>
						<ScrollArea class="storybook-scroll-sample">
							<div v-for="item in text.foundation.timelineItems" :key="item" class="storybook-scroll-item">
								<Icon icon="tabler:circle-dot" size="16" />
								<span>{{ item }}</span>
							</div>
						</ScrollArea>
					</Surface>

					<Surface tone="default" class="storybook-section">
						<h2>{{ text.foundation.toastTitle }}</h2>
						<p>{{ text.foundation.toastDescription }}</p>
						<Toast :default-toasts="text.foundation.toasts" :duration="600000" />
					</Surface>
				</div>
			</section>
		`
	})
} satisfies Meta

export default meta
type Story = StoryObj<typeof meta>

export const Default: Story = {
	parameters: {
		a11y: {
			config: {
				// Reka Toast uses aria-hidden focus proxies for keyboard focus wrapping.
				rules: [{ id: 'aria-hidden-focus', enabled: false }]
			}
		}
	}
}
