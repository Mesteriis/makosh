import type { Meta, StoryObj } from '@storybook/vue3-vite'
import { expect, within } from 'storybook/test'
import {
	CandlestickChart,
	DonutChart,
	RadarChart,
	ScoreGauge,
	Sparkline
} from '@/shared/ui'
import { storybookLocaleFromGlobals } from './storybook-i18n'

function graphicsText(locale: string) {
	if (locale === 'en') {
		return {
			title: 'Graphics',
			description: 'Compact semantic SVG graphics for scores, distributions, trends and market-like intervals.',
			scoreLabel: 'Dialogue confidence',
			scoreTitle: 'Score and confidence',
			distributionLabel: 'Evidence distribution',
			distributionTitle: 'Categorical distribution',
			radarLabel: 'Signal profile',
			radarTitle: 'Spider chart',
			trendLabel: 'Activity trend',
			trendTitle: 'Micro trend',
			candlesLabel: 'Risk interval candles',
			candlesTitle: 'Interval candles'
		}
	}

	if (locale === 'es') {
		return {
			title: 'Graficos',
			description: 'Graficos SVG compactos y semanticos para puntuaciones, distribuciones, tendencias e intervalos.',
			scoreLabel: 'Confianza del dialogo',
			scoreTitle: 'Puntuacion y confianza',
			distributionLabel: 'Distribucion de evidencia',
			distributionTitle: 'Distribucion categorica',
			radarLabel: 'Perfil de senales',
			radarTitle: 'Grafico de arana',
			trendLabel: 'Tendencia de actividad',
			trendTitle: 'Micro tendencia',
			candlesLabel: 'Velas de intervalo de riesgo',
			candlesTitle: 'Velas de intervalo'
		}
	}

	return {
		title: 'Графика',
		description: 'Компактные SVG-графики для оценок, распределений, трендов и интервальных значений.',
		scoreLabel: 'Достоверность диалога',
		scoreTitle: 'Оценка и уверенность',
		distributionLabel: 'Распределение доказательств',
		distributionTitle: 'Категориальное распределение',
		radarLabel: 'Профиль сигналов',
		radarTitle: 'Паутинка',
		trendLabel: 'Тренд активности',
		trendTitle: 'Микро-тренд',
		candlesLabel: 'Свечи интервала риска',
		candlesTitle: 'Свечи'
	}
}

const meta = {
	title: 'Макошь UI/General/Graphics',
	render: (_args, context) => ({
		components: {
			CandlestickChart,
			DonutChart,
			RadarChart,
			ScoreGauge,
			Sparkline
		},
		data() {
			const text = graphicsText(storybookLocaleFromGlobals(context.globals))
			return {
				text,
				distribution: [
					{ id: 'source', label: 'Source', value: 46, tone: 'success' },
					{ id: 'ai', label: 'AI', value: 32, tone: 'accent' },
					{ id: 'review', label: 'Review', value: 22, tone: 'warning' }
				],
				radarMetrics: [
					{ id: 'source', label: 'src', value: 92 },
					{ id: 'risk', label: 'risk', value: 42 },
					{ id: 'memory', label: 'mem', value: 74 },
					{ id: 'review', label: 'rev', value: 58 },
					{ id: 'owner', label: 'own', value: 84 }
				],
				trend: [18, 21, 20, 36, 32, 44, 42, 57, 51, 68, 64, 72],
				candles: [
					{ id: 'd1', label: 'Mon', open: 44, high: 58, low: 38, close: 54 },
					{ id: 'd2', label: 'Tue', open: 54, high: 61, low: 47, close: 50 },
					{ id: 'd3', label: 'Wed', open: 50, high: 66, low: 49, close: 63 },
					{ id: 'd4', label: 'Thu', open: 63, high: 70, low: 55, close: 58 },
					{ id: 'd5', label: 'Fri', open: 58, high: 76, low: 56, close: 72 }
				]
			}
		},
		template: `
			<section class="storybook-canvas">
				<div class="storybook-section">
					<h2>{{ text.title }}</h2>
					<p>{{ text.description }}</p>
				</div>

				<div class="storybook-grid">
					<div class="storybook-section">
						<h3>{{ text.scoreTitle }}</h3>
						<ScoreGauge :value="88" :label="text.scoreLabel" unit="/100" tone="success" size="lg" />
					</div>

					<div class="storybook-section">
						<h3>{{ text.distributionTitle }}</h3>
						<DonutChart :segments="distribution" :label="text.distributionLabel" size="md" />
					</div>

					<div class="storybook-section">
						<h3>{{ text.radarTitle }}</h3>
						<RadarChart :metrics="radarMetrics" :label="text.radarLabel" tone="accent" size="md" />
					</div>

					<div class="storybook-section">
						<h3>{{ text.trendTitle }}</h3>
						<Sparkline :values="trend" :label="text.trendLabel" tone="success" size="lg" />
					</div>

					<div class="storybook-section">
						<h3>{{ text.candlesTitle }}</h3>
						<CandlestickChart :candles="candles" :label="text.candlesLabel" size="lg" />
					</div>
				</div>
			</section>
		`
	}),
	play: async ({ canvasElement, globals }) => {
		const text = graphicsText(storybookLocaleFromGlobals(globals))
		const canvas = within(canvasElement)

		await expect(canvas.getByText(text.title)).toBeVisible()
		await expect(canvas.getByRole('meter', { name: text.scoreLabel })).toBeVisible()
		await expect(canvas.getByRole('img', { name: text.distributionLabel })).toBeVisible()
		await expect(canvas.getByRole('img', { name: text.radarLabel })).toBeVisible()
		await expect(canvas.getByRole('img', { name: text.trendLabel })).toBeVisible()
		await expect(canvas.getByRole('img', { name: text.candlesLabel })).toBeVisible()
	}
} satisfies Meta

export default meta
type Story = StoryObj<typeof meta>

export const Default: Story = {}
