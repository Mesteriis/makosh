import { defineConfig } from '@playwright/test'

const storybookHost = process.env.MAKOSH_STORYBOOK_HOST ?? 'localhost'
const storybookPort = Number(process.env.MAKOSH_STORYBOOK_PORT ?? '6006')
const storybookBaseUrl = `http://${storybookHost}:${storybookPort}`
const storybookStaticCommand = [
	'pnpm exec storybook build --quiet --test --output-dir storybook-static',
	'pnpm storybook:serve'
].join(' && ')

export default defineConfig({
	testDir: './tests/visual',
	outputDir: './test-results/visual',
	forbidOnly: Boolean(process.env.CI),
	retries: 0,
	timeout: 1_200_000,
	workers: 1,
	reporter: process.env.CI
		? [
				['list'],
				['html', { outputFolder: 'playwright-report', open: 'never' }]
			]
		: [['list']],
	expect: {
		toHaveScreenshot: {
			animations: 'disabled',
			caret: 'hide',
			maxDiffPixelRatio: 0.005,
			scale: 'css',
			threshold: 0.08
		}
	},
	use: {
		baseURL: storybookBaseUrl,
		browserName: 'chromium',
		colorScheme: 'light',
		reducedMotion: 'reduce',
		trace: 'retain-on-failure',
		viewport: {
			width: 1440,
			height: 900
		}
	},
	webServer: {
		command: storybookStaticCommand,
		reuseExistingServer: false,
		timeout: 180_000,
		url: `${storybookBaseUrl}/index.json`
	}
})
