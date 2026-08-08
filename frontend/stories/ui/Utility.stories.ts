import type { Meta, StoryObj } from '@storybook/vue3-vite'
import { expect, userEvent, within } from 'storybook/test'
import {
	CopyButton,
	EntityIcon,
	FileIcon,
	KeyboardHint,
	LocaleSwitcher,
	ProviderIcon,
	Shortcut,
	StatusIcon,
	ThemeSwitcher
} from '@/shared/ui'
import { storybookLocaleFromGlobals, storybookText } from './storybook-i18n'

const meta = {
	title: 'Макошь UI/General/Utility',
	component: CopyButton,
	args: {
		value: 'makosh://local/context-pack'
	}
} satisfies Meta<typeof CopyButton>

export default meta
type Story = StoryObj<typeof meta>

export const SwitchersAndCopy: Story = {
	render: (_args, context) => ({
		components: {
			CopyButton,
			LocaleSwitcher,
			ThemeSwitcher
		},
		data() {
			const text = storybookText(storybookLocaleFromGlobals(context.globals))
			return {
				text,
				theme: 'base-light',
				locale: 'ru'
			}
		},
		template: `
			<section class="storybook-canvas">
				<div class="storybook-section">
					<h2>{{ text.utility.title }}</h2>
					<p>{{ text.utility.description }}</p>
				</div>

				<div class="storybook-grid">
					<div class="storybook-section">
						<h3>{{ text.utility.copyTitle }}</h3>
						<CopyButton
							:value="text.utility.copyValue"
							:label="text.utility.copyLabel"
							:copied-label="text.utility.copiedLabel"
							:error-label="text.utility.errorLabel"
						/>
					</div>

					<div class="storybook-section">
						<h3>{{ text.utility.themeTitle }}</h3>
						<ThemeSwitcher v-model="theme" />
					</div>

					<div class="storybook-section">
						<h3>{{ text.utility.localeTitle }}</h3>
						<LocaleSwitcher v-model="locale" :options="text.utility.locales" />
					</div>
				</div>
			</section>
		`
	}),
	play: async ({ canvasElement, globals }) => {
		const text = storybookText(storybookLocaleFromGlobals(globals))
		const canvas = within(canvasElement)
		await expect(canvas.getByText(text.utility.title)).toBeVisible()
		await userEvent.click(canvas.getByRole('radio', { name: 'Макошь' }))
		await expect(canvas.getByRole('radio', { name: 'Макошь' })).toHaveAttribute('aria-checked', 'true')
		await userEvent.click(canvas.getByRole('radio', { name: 'Dark' }))
		await expect(canvas.getByRole('radio', { name: 'Dark' })).toHaveAttribute('aria-checked', 'true')
	}
}

export const KeyboardAndIcons: Story = {
	render: (_args, context) => ({
		components: {
			EntityIcon,
			FileIcon,
			KeyboardHint,
			ProviderIcon,
			Shortcut,
			StatusIcon
		},
		data() {
			const text = storybookText(storybookLocaleFromGlobals(context.globals))
			return { text }
		},
		template: `
			<section class="storybook-canvas">
				<div class="storybook-section">
					<h2>{{ text.utility.shortcutsTitle }}</h2>
					<div class="storybook-row">
						<KeyboardHint :label="text.utility.openCommand" :keys="['Meta', 'K']" />
						<Shortcut :label="text.utility.sendDraft" :keys="['Meta', 'Enter']" />
					</div>
				</div>

				<div class="storybook-grid">
					<div class="storybook-section">
						<h3>{{ text.utility.providerTitle }}</h3>
						<div class="storybook-row">
							<ProviderIcon
								v-for="provider in text.utility.providers"
								:key="provider"
								:provider="provider"
								:label="provider"
							/>
						</div>
					</div>

					<div class="storybook-section">
						<h3>{{ text.utility.statusTitle }}</h3>
						<div class="storybook-row">
							<StatusIcon
								v-for="status in text.utility.statuses"
								:key="status"
								:status="status"
								:label="status"
							/>
						</div>
					</div>

					<div class="storybook-section">
						<h3>{{ text.utility.entityTitle }}</h3>
						<div class="storybook-row">
							<EntityIcon
								v-for="entity in text.utility.entities"
								:key="entity"
								:entity="entity"
								:label="entity"
							/>
						</div>
					</div>

					<div class="storybook-section">
						<h3>{{ text.utility.fileTitle }}</h3>
						<div class="storybook-row">
							<FileIcon
								v-for="file in text.utility.files"
								:key="file.mimeType"
								:mime-type="file.mimeType"
								:label="file.label"
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
		await expect(canvas.getByText(text.utility.shortcutsTitle)).toBeVisible()
		await expect(canvas.getByLabelText('mail')).toBeVisible()
		await expect(canvas.getByLabelText('success')).toBeVisible()
	}
}
