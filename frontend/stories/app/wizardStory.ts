import type { Component } from 'vue'
import { Button, Icon, Input, Steps, Switch } from '@/shared/ui'
import type { StepsItem } from '@/shared/ui'
import './wizardStory.css'

type WizardTone = 'gmail' | 'icloud' | 'telegram' | 'whatsapp' | 'ai'

type WizardField = {
	label: string
	value: string
	secret?: boolean
}

type WizardCheck = {
	label: string
	description: string
	status: 'ready' | 'pending' | 'blocked'
}

type WizardCapability = {
	label: string
	description: string
	enabled: boolean
	locked?: boolean
}

type WizardStepContent = {
	fields?: readonly WizardField[]
	checks?: readonly WizardCheck[]
	capabilities?: readonly WizardCapability[]
}

export type WizardStoryModel = {
	title: string
	subtitle: string
	providerLabel: string
	icon: string
	tone: WizardTone
	steps: readonly [StepsItem, StepsItem, StepsItem]
	content: readonly [WizardStepContent, WizardStepContent, WizardStepContent]
	primaryActionLabel: string
}

export const wizardStoryModels = {
	gmail: {
		title: 'Подключение Gmail',
		subtitle: 'Почта, контакты, файлы, фото, заметки и встречи.',
		providerLabel: 'Google Gmail',
		icon: 'tabler:brand-gmail',
		tone: 'gmail',
		primaryActionLabel: 'Открыть мастер Gmail',
		steps: [
			{ title: 'Авторизация' },
			{ title: 'Проверка доступа' },
			{ title: 'Синхронизация' }
		],
		content: [
			{
				fields: [
					{ label: 'Аккаунт Google', value: 'owner@gmail.com' },
					{ label: 'Название аккаунта', value: 'Personal Google' }
				]
			},
			{
				checks: [
					{ label: 'Почта', description: 'Готово', status: 'ready' },
					{ label: 'Контакты', description: 'Готово', status: 'ready' },
					{ label: 'Файлы, фото, заметки и встречи', description: 'Проверяется', status: 'pending' }
				]
			},
			{
				capabilities: [
					{ label: 'Почта', description: 'Письма и папки', enabled: true },
					{ label: 'Контакты', description: 'Люди и адреса', enabled: true },
					{ label: 'Google Drive', description: 'Файлы и документы', enabled: true },
					{ label: 'Google Photos', description: 'Фото и альбомы', enabled: true },
					{ label: 'Google Keep', description: 'Заметки и списки', enabled: true },
					{ label: 'Google Meet', description: 'Встречи и ссылки', enabled: true }
				]
			}
		]
	},
	icloud: {
		title: 'Подключение iCloud Mail',
		subtitle: 'Почта iCloud, папки и отправка.',
		providerLabel: 'Apple iCloud',
		icon: 'tabler:brand-apple',
		tone: 'icloud',
		primaryActionLabel: 'Открыть мастер iCloud',
		steps: [
			{ title: 'Учётные данные' },
			{ title: 'Проверка ящика' },
			{ title: 'Сервисы' }
		],
		content: [
			{
				fields: [
					{ label: 'Email', value: 'owner@icloud.com' },
					{ label: 'Пароль приложения', value: '•••• •••• •••• ••••', secret: true },
					{ label: 'Название аккаунта', value: 'Personal iCloud' }
				]
			},
			{
				checks: [
					{ label: 'Входящие', description: 'Готово', status: 'ready' },
					{ label: 'Папки', description: 'Готово', status: 'ready' },
					{ label: 'Отправка', description: 'Проверяется', status: 'pending' }
				]
			},
			{
				capabilities: [
					{ label: 'Почта', description: 'Письма и папки', enabled: true },
					{ label: 'Отправка', description: 'Исходящие письма', enabled: true },
					{ label: 'Контакты', description: 'Адреса и профили', enabled: false, locked: true }
				]
			}
		]
	},
	telegram: {
		title: 'Подключение Telegram',
		subtitle: 'Чаты, группы, контакты и медиа Telegram.',
		providerLabel: 'Telegram',
		icon: 'tabler:brand-telegram',
		tone: 'telegram',
		primaryActionLabel: 'Открыть мастер Telegram',
		steps: [
			{ title: 'Авторизация' },
			{ title: 'Проверка' },
			{ title: 'Сервисы' }
		],
		content: [
			{
				fields: [
					{ label: 'Телефон', value: '+34 ••• ••• •••' },
					{ label: 'Название аккаунта', value: 'Personal Telegram' }
				]
			},
			{
				checks: [
					{ label: 'Вход', description: 'Ожидает подтверждения', status: 'pending' },
					{ label: 'Чаты', description: 'Проверяется', status: 'pending' },
					{ label: 'Медиа', description: 'Готово', status: 'ready' }
				]
			},
			{
				capabilities: [
					{ label: 'Сообщения', description: 'Диалоги и группы', enabled: true },
					{ label: 'Контакты', description: 'Люди и профили', enabled: true },
					{ label: 'Медиа', description: 'Вложения и голосовые', enabled: false }
				]
			}
		]
	},
	whatsapp: {
		title: 'Подключение WhatsApp',
		subtitle: 'Чаты, группы и медиа WhatsApp.',
		providerLabel: 'WhatsApp',
		icon: 'tabler:brand-whatsapp',
		tone: 'whatsapp',
		primaryActionLabel: 'Открыть мастер WhatsApp',
		steps: [
			{ title: 'Авторизация' },
			{ title: 'Проверка' },
			{ title: 'Синхронизация' }
		],
		content: [
			{
				fields: [
					{ label: 'Устройство', value: 'Макошь Desktop' },
					{ label: 'Название аккаунта', value: 'Personal WhatsApp' }
				]
			},
			{
				checks: [
					{ label: 'Устройство', description: 'Ожидает QR', status: 'pending' },
					{ label: 'Чаты', description: 'Проверяется', status: 'pending' },
					{ label: 'Отправка', description: 'Готово', status: 'ready' }
				]
			},
			{
				capabilities: [
					{ label: 'Чаты и группы', description: 'Сообщения и участники', enabled: true },
					{ label: 'Медиа', description: 'Фото, документы и голосовые', enabled: false },
					{ label: 'Статусы', description: 'Обновления статусов', enabled: false, locked: true }
				]
			}
		]
	},
	ai: {
		title: 'Подключение AI провайдера',
		subtitle: 'AI-модели и действия Макошь.',
		providerLabel: 'AI провайдер',
		icon: 'tabler:sparkles',
		tone: 'ai',
		primaryActionLabel: 'Открыть мастер AI',
		steps: [
			{ title: 'Параметры API' },
			{ title: 'Проверка' },
			{ title: 'Модели Макошь' }
		],
		content: [
			{
				fields: [
					{ label: 'Название', value: 'OmniRoute' },
					{ label: 'URL API', value: 'https://ai.sh-inc.ru/v1' },
					{ label: 'API-токен', value: '••••••••••••••••', secret: true }
				]
			},
			{
				checks: [
					{ label: 'Каталог моделей', description: 'Готово', status: 'ready' },
					{ label: 'Чат', description: 'Проверяется', status: 'pending' },
					{ label: 'Приватный контекст', description: 'Нужно разрешение', status: 'pending' }
				]
			},
			{
				capabilities: [
					{ label: 'Перевод и общий чат', description: 'Быстрые ответы', enabled: true },
					{ label: 'Анализ почты', description: 'Разбор писем', enabled: true },
					{ label: 'Эмбеддинги', description: 'Поиск и похожие материалы', enabled: false }
				]
			}
		]
	}
} satisfies Record<string, WizardStoryModel>

export function createWizardStory(model: WizardStoryModel): Component {
	return {
		components: { Button, Icon, Input, Steps, Switch },
		setup() {
			return { model }
		},
		data() {
			return {
				open: true,
				step: 1
			}
		},
		template: `
			<section class="storybook-canvas storybook-canvas--wide wizard-story-canvas">
				<article class="wizard-story-launcher">
					<span :class="['wizard-story-provider-mark', 'wizard-story-provider-mark--' + model.tone]">
						<Icon :icon="model.icon" size="28" />
					</span>
					<div class="wizard-story-launcher__copy">
						<p class="wizard-story-launcher__eyebrow">Мастер подключения</p>
						<h2>{{ model.title }}</h2>
						<p>{{ model.subtitle }}</p>
					</div>
					<Button icon="tabler:route" @click="open = true">{{ model.primaryActionLabel }}</Button>
				</article>

				<Steps
					v-model:open="open"
					v-model:step="step"
					:title="model.title"
					:description="model.providerLabel"
					:step-count="3"
					:steps="model.steps"
					finish-label="Готово"
					size="lg"
					>
						<template #step-1>
							<div class="wizard-story-step">
								<div class="wizard-story-field-grid">
									<label v-for="field in model.content[0].fields" :key="field.label" class="wizard-story-field">
										<span>{{ field.label }}</span>
										<Input :model-value="field.value" :type="field.secret ? 'password' : 'text'" readonly :aria-label="field.label" />
									</label>
								</div>
							</div>
						</template>

						<template #step-2>
							<div class="wizard-story-step">
								<div class="wizard-story-check-list">
									<article
										v-for="check in model.content[1].checks"
										:key="check.label"
										:class="['wizard-story-check', 'wizard-story-check--' + check.status]"
									>
									<Icon :icon="check.status === 'ready' ? 'tabler:check' : check.status === 'pending' ? 'tabler:clock' : 'tabler:alert-triangle'" size="18" />
									<span>
										<strong>{{ check.label }}</strong>
										<small>{{ check.description }}</small>
									</span>
								</article>
								</div>
							</div>
						</template>

						<template #step-3>
							<div class="wizard-story-step">
								<div class="wizard-story-capability-list">
									<label
										v-for="capability in model.content[2].capabilities"
										:key="capability.label"
										class="wizard-story-capability"
								>
									<span>
										<strong>{{ capability.label }}</strong>
										<small>{{ capability.description }}</small>
									</span>
									<Switch :model-value="capability.enabled" :disabled="capability.locked" :aria-label="capability.label" />
								</label>
							</div>
						</div>
					</template>
				</Steps>
			</section>
		`
	}
}
