import type { Meta, StoryObj } from '@storybook/vue3-vite'
import { Button, Input, Steps, Switch } from '@/shared/ui'
import type { StepsItem } from '@/shared/ui'
import { storybookLocaleFromGlobals, storybookText } from './storybook-i18n'

const providerSteps: StepsItem[] = [
	{
		title: 'Авторизация',
		description: 'OAuth callback и владелец аккаунта',
		requirement: 'Мастер показывает один шаг за раз, а заголовок подключения остаётся неизменным.'
	},
	{
		title: 'Проверка доступа',
		description: 'Почта, профиль и каналы синхронизации',
		requirement: 'После callback backend проверяет доступ и отдаёт состояние без фиктивных данных.'
	},
	{
		title: 'Синхронизация',
		description: 'Папки, письма и фоновые задачи',
		requirement: 'Последний шаг выбирает, что будет включено в Макошь для этого аккаунта.'
	}
]

const meta = {
	title: 'Макошь UI/General/Steps',
	component: Steps,
	render: (_args, context) => ({
		components: { Button, Input, Steps, Switch },
		data() {
			const text = storybookText(storybookLocaleFromGlobals(context.globals))
			return {
				open: true,
				step: 1,
				steps: providerSteps,
				text
			}
		},
		template: `
			<section class="storybook-canvas">
				<div class="storybook-section">
					<h2>Steps</h2>
					<p>Modal carousel wizard with named slots for each step.</p>
					<Button icon="tabler:route" @click="open = true">Открыть мастер Gmail</Button>
				</div>

				<Steps
					v-model:open="open"
					v-model:step="step"
					title="Подключение Gmail"
					:step-count="3"
					:steps="steps"
					:cancel-label="text.common.cancel"
					:finish-label="text.common.done"
				>
					<template #step-1>
						<div class="storybook-stack">
							<label class="storybook-stack">
								<strong>Аккаунт</strong>
								<Input model-value="alexander@gmail.com" aria-label="Account" />
							</label>
							<label class="storybook-stack">
								<strong>Callback URL</strong>
								<Input model-value="http://127.0.0.1:8080/api/v1/oauth/google/callback" aria-label="Callback URL" />
							</label>
							<label class="storybook-stack">
								<strong>Состояние</strong>
								<Input model-value="Ожидает OAuth callback" aria-label="Status" />
							</label>
						</div>
					</template>

					<template #step-2>
						<div class="storybook-stack">
							<h3>Права доступа</h3>
							<p>Проверка scopes, refresh token и возможности читать все папки ящика.</p>
							<Switch :model-value="true" aria-label="Mail access enabled" />
						</div>
					</template>

					<template #step-3>
						<div class="storybook-stack">
							<h3>Что включить</h3>
							<label><input type="checkbox" checked /> Почта · все папки</label>
							<label><input type="checkbox" checked /> Контакты · профиль отправителей</label>
							<label><input type="checkbox" /> Календарь · события Gmail</label>
						</div>
					</template>
				</Steps>
			</section>
		`
	})
} satisfies Meta<typeof Steps>

export default meta
type Story = StoryObj<typeof meta>

export const Default: Story = {}
