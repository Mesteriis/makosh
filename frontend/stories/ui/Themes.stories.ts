import type { Meta, StoryObj } from '@storybook/vue3-vite'
import { Badge, Button, Card, CardContent, CardDescription, CardHeader, CardTitle, Input, ThemeProvider } from '@/shared/ui'
import { storybookLocaleFromGlobals, storybookText } from './storybook-i18n'

const swatches = [
	'var(--h-color-bg)',
	'var(--h-color-surface)',
	'var(--h-color-surface-raised)',
	'var(--h-color-text-strong)',
	'var(--h-color-text-muted)',
	'var(--h-color-accent)',
	'var(--h-color-danger)',
	'var(--h-color-border)'
]

const meta = {
	title: 'Макошь UI/Foundation/Themes',
	render: (_args, context) => ({
		components: { Badge, Button, Card, CardContent, CardDescription, CardHeader, CardTitle, Input, ThemeProvider },
		setup() {
			const text = storybookText(storybookLocaleFromGlobals(context.globals))
			const themeByValue = Object.fromEntries(text.themes.options.map((theme) => [theme.value, theme])) as Record<string, (typeof text.themes.options)[number]>
			const themePairs = [
				{ id: 'base', themes: [themeByValue['base-light'], themeByValue['base-dark']] },
				{ id: 'makosh', themes: [themeByValue['makosh-light'], themeByValue['makosh-dark']] }
			]

			return { swatches, text, themePairs }
		},
		template: `
			<section class="storybook-canvas storybook-canvas--wide storybook-theme-canvas">
				<div v-for="pair in themePairs" :key="pair.id" class="storybook-theme-pair">
					<ThemeProvider v-for="theme in pair.themes" :key="theme.value" :theme="theme.value" class="storybook-theme-column">
						<div class="storybook-section storybook-theme-panel">
							<h2>{{ theme.label }}</h2>
							<p>{{ theme.description }}</p>
							<div class="storybook-token-grid">
								<div v-for="(swatch, index) in swatches" :key="swatch" class="storybook-token">
									<div class="storybook-swatch" :style="{ '--swatch': swatch }"></div>
									<strong>{{ text.themes.swatches[index] }}</strong>
								</div>
							</div>
							<Card>
								<CardHeader>
									<CardTitle>{{ text.themes.cardTitle }}</CardTitle>
									<CardDescription>{{ text.themes.cardDescription }}</CardDescription>
								</CardHeader>
								<CardContent>
									<div class="storybook-row">
										<Input :model-value="text.themes.searchValue" :aria-label="text.button.searchLabel" />
										<Button>{{ text.button.primary }}</Button>
										<Button variant="outline">{{ text.button.outline }}</Button>
										<Badge variant="accent">{{ text.themes.contextBadge }}</Badge>
									</div>
								</CardContent>
							</Card>
						</div>
					</ThemeProvider>
				</div>
			</section>
		`
	})
} satisfies Meta

export default meta
type Story = StoryObj<typeof meta>

export const AllThemes: Story = {}
