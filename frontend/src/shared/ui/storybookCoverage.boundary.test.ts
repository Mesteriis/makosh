import { describe, expect, it } from 'vitest'
import { readdirSync, readFileSync } from 'node:fs'
import type { Dirent } from 'node:fs'
import { fileURLToPath } from 'node:url'
import { join } from 'node:path'

describe('Макошь UI Storybook visual coverage boundary', () => {
	it('exports every shared UI component through the kit barrel', () => {
		const uiDir = fileURLToPath(new URL('.', import.meta.url))
		const componentNames = ['.', 'primitives', 'patterns']
			.flatMap((relativePath) => readdirSync(join(uiDir, relativePath)))
			.filter((fileName) => fileName.endsWith('.vue'))
			.map((fileName) => fileName.replace(/\.vue$/, ''))
			.sort()
		const exportedNames = exportedComponentNames(readFileSync(new URL('./index.ts', import.meta.url), 'utf8'))

		expect(exportedNames).toEqual(componentNames)
	})

	it('keeps every exported shared UI component represented in Storybook', () => {
		const storiesDir = fileURLToPath(new URL('../../../stories/ui/', import.meta.url))
		const storySources = readdirSync(storiesDir)
			.filter((fileName) => fileName.endsWith('.stories.ts'))
			.map((fileName) => readFileSync(join(storiesDir, fileName), 'utf8'))
			.join('\n')
		const exportedNames = exportedComponentNames(readFileSync(new URL('./index.ts', import.meta.url), 'utf8'))
		const storyImports = storybookUiImports(storySources)

		expect(exportedNames.filter((componentName) => !storyImports.includes(componentName))).toEqual([])
	})

	it('keeps Storybook configured as the Макошь UI lab surface', () => {
		const frontendRoot = fileURLToPath(new URL('../../../', import.meta.url))
		const mainConfig = readFileSync(join(frontendRoot, '.storybook/main.ts'), 'utf8')
		const previewConfig = readFileSync(join(frontendRoot, '.storybook/preview.ts'), 'utf8')
		const requiredAddons = [
			'@storybook/addon-docs',
			'@storybook/addon-a11y',
			'@storybook/addon-themes',
			'@storybook/addon-vitest',
			'@storybook/addon-coverage',
			'@storybook/addon-designs',
			'msw-storybook-addon',
			'storybook-addon-pseudo-states',
			'storybook-design-token'
		]

		for (const addonName of requiredAddons) {
			expect(mainConfig).toContain(addonName)
		}
		expect(mainConfig).toContain("staticDirs: ['../public']")
		expect(mainConfig).toContain("const allowedStorybookHosts = ['localhost', '127.0.0.1']")
		expect(mainConfig).toContain('allowedHosts:')
		expect(mainConfig).toContain("designTokenGlob: 'src/shared/ui/{foundation,styles}/**/*.css'")
		expect(previewConfig).toContain('withThemeByDataAttribute')
		expect(previewConfig).toContain('initialize({')
		expect(previewConfig).toContain('loaders: [mswLoader]')
		expect(previewConfig).toContain('storybookLocaleToolbarItems')
	})

	it('keeps Storybook stories localized for Russian, English and Spanish', () => {
		const storiesDir = fileURLToPath(new URL('../../../stories/ui/', import.meta.url))
		const localeSource = readFileSync(join(storiesDir, 'storybook-i18n.ts'), 'utf8')
		const storySources = readdirSync(storiesDir)
			.filter((fileName) => fileName.endsWith('.stories.ts'))
			.map((fileName) => `${fileName}\n${readFileSync(join(storiesDir, fileName), 'utf8')}`)

		expect(localeSource).toContain("export const storybookLocales = ['ru', 'en', 'es'] as const")
		expect(localeSource).toContain("'Русский'")
		expect(localeSource).toContain("'English'")
		expect(localeSource).toContain("'Español'")
		for (const source of storySources) {
			expect(source).toContain("from './storybook-i18n'")
		}
	})

	it('keeps standard controls under the General Storybook hierarchy', () => {
		const storiesDir = fileURLToPath(new URL('../../../stories/ui/', import.meta.url))
		const storySources = readdirSync(storiesDir)
			.filter((fileName) => fileName.endsWith('.stories.ts'))
			.map((fileName) => `${fileName}\n${readFileSync(join(storiesDir, fileName), 'utf8')}`)
			.join('\n')
		const requiredGeneralTitles = [
			'Макошь UI/General/Button',
			'Макошь UI/General/Button Group',
			'Макошь UI/General/Icon Button',
			'Макошь UI/General/Split Button',
			'Макошь UI/General/Toggle Group',
			'Макошь UI/General/Select',
			'Макошь UI/General/Searchable Select',
			'Макошь UI/General/Multi Select',
			'Макошь UI/General/Searchable Multi Select',
			'Макошь UI/General/Grouped Select',
			'Макошь UI/General/Tree Select',
			'Макошь UI/General/Cascader',
			'Макошь UI/General/Async Select',
			'Макошь UI/General/Input',
			'Макошь UI/General/Textarea',
			'Макошь UI/General/Search Input',
			'Макошь UI/General/Token Input',
			'Макошь UI/General/Tag Input',
			'Макошь UI/General/Checkbox',
			'Макошь UI/General/Communication',
			'Макошь UI/General/Radio',
			'Макошь UI/General/Switch',
			'Макошь UI/General/Slider',
			'Макошь UI/General/Date Picker',
			'Макошь UI/General/Date Range Picker',
			'Макошь UI/General/Time Picker',
			'Макошь UI/General/Menu',
			'Макошь UI/General/Context Menu',
			'Макошь UI/General/Command',
			'Макошь UI/General/Tabs',
			'Макошь UI/General/Dialog',
			'Макошь UI/General/Steps',
			'Макошь UI/General/Drawer',
			'Макошь UI/General/Tooltip',
			'Макошь UI/General/Popover',
			'Макошь UI/General/Surface',
			'Макошь UI/General/Table',
			'Макошь UI/General/List',
			'Макошь UI/General/Tree',
			'Макошь UI/General/Timeline',
			'Макошь UI/General/Media',
			'Макошь UI/General/Editor',
			'Макошь UI/General/Feedback',
			'Макошь UI/General/Graphics',
			'Макошь UI/General/Layout',
			'Макошь UI/General/Utility'
		]
		const requiredFoundationTitles = [
			'Макошь UI/Foundation/Tokens',
			'Макошь UI/Foundation/Themes',
			'Макошь UI/Foundation/Typography',
			'Макошь UI/Foundation/Icons',
			'Макошь UI/Foundation/Spacing'
		]
		const forbiddenLegacyTopLevelTitles = [
			'Макошь UI/Command',
			'Макошь UI/Communication',
			'Макошь UI/Data Display',
			'Макошь UI/Editor',
			'Макошь UI/Feedback',
			'Макошь UI/Foundation',
			'Макошь UI/Layout',
			'Макошь UI/Media',
			'Макошь UI/Navigation',
			'Макошь UI/Overlays',
			'Макошь UI/Primitives',
			'Макошь UI/Themes',
			'Макошь UI/Utility'
		]

		expect(storySources).not.toContain('Макошь UI/Controls/')
		for (const title of [...requiredGeneralTitles, ...requiredFoundationTitles]) {
			expect(storySources).toContain(`title: '${title}'`)
		}
		expect(storySources).not.toContain("title: 'Макошь UI/Domain/")
		for (const title of forbiddenLegacyTopLevelTitles) {
			expect(storySources).not.toContain(`title: '${title}'`)
		}
	})

	it('screenshots product stories in the baseline theme across locales and responsive widths', () => {
		const visualSpec = readFileSync(
			new URL('../../../tests/visual/storybook.visual.spec.ts', import.meta.url),
			'utf8'
		)

		expect(visualSpec).toContain(
			"const CANONICAL_THEMES = ['base-light', 'base-dark', 'makosh-light', 'makosh-dark'] as const"
		)
		expect(visualSpec).toContain("const VISUAL_SNAPSHOT_THEMES = ['base-light'] as const")
		expect(visualSpec).toContain(
			'Cross-theme token regressions are covered by Макошь UI/Foundation/Themes'
		)
			 expect(visualSpec).toContain("const LOCALES = ['en'] as const")
		for (const width of [320, 375, 768, 1024, 1440, 1920, 5120]) {
			expect(visualSpec).toContain(`width: ${width}`)
		}
		expect(visualSpec).toContain('for (const theme of VISUAL_SNAPSHOT_THEMES)')
		expect(visualSpec).toContain("request.get('/index.json')")
		expect(visualSpec).toContain("entry.type === 'story'")
		expect(visualSpec).toContain('data-ui-locale')
		expect(visualSpec).toContain('toHaveScreenshot')
	})

	it('keeps Storybook visual regression wired into the frontend validation command as a compare-only gate', () => {
		const frontendRoot = fileURLToPath(new URL('../../../', import.meta.url))
		const playwrightConfig = readFileSync(join(frontendRoot, 'playwright.config.ts'), 'utf8')
		expect(playwrightConfig).toContain("process.env.MAKOSH_STORYBOOK_HOST ?? 'localhost'")
		expect(playwrightConfig).toContain('pnpm exec storybook build --quiet --test --output-dir storybook-static')
		expect(playwrightConfig).toContain('pnpm storybook:serve')
		expect(playwrightConfig).toContain('reuseExistingServer: false')
		const packageJson = readFileSync(join(frontendRoot, 'package.json'), 'utf8')
		expect(packageJson).toContain('"test:visual": "playwright test"')
		expect(packageJson).toContain('"test:visual:update": "playwright test --update-snapshots"')
		expect(packageJson).toContain('"validate": "pnpm check:cleanroom-tauri-bundle && pnpm lint && pnpm typecheck && pnpm test:unit && pnpm test:visual && pnpm build"')
		expect(packageJson).toContain('"storybook:serve": "node scripts/serve-storybook-static.mjs"')
		expect(packageJson).toContain('test-storybook --url http://localhost:6006')
	})

	it('keeps vendor UI primitives behind the Макошь UI kit boundary', () => {
		const frontendRoot = fileURLToPath(new URL('../../../', import.meta.url))
		const checkedRoots = ['src', 'stories', '.storybook']
		const forbiddenImports = /from ['"](reka-ui|shadcn-vue|@radix-ui\/[^'"]+|lucide(?:-[^'"]+)?)['"]/g
		const violations = checkedRoots
			.flatMap((root) => sourceFiles(join(frontendRoot, root)))
			.filter((filePath) => !filePath.includes('/src/shared/ui/'))
			.flatMap((filePath) => {
				const source = readFileSync(filePath, 'utf8')
				return Array.from(source.matchAll(forbiddenImports)).map((match) => `${filePath}: ${match[1]}`)
			})

		expect(violations).toEqual([])
	})
})

function exportedComponentNames(source: string): string[] {
	return Array.from(source.matchAll(/export \{ default as (\w+) \} from '\.\/[^']+\.vue'/g))
		.map((match) => match[1])
		.sort()
}

function storybookUiImports(source: string): string[] {
	return Array.from(source.matchAll(/import \{([^}]+)\} from '@\/shared\/ui'/g))
		.flatMap((match) => match[1].split(','))
		.map((name) => name.trim())
		.filter(Boolean)
		.sort()
}

function sourceFiles(root: string): string[] {
	return readdirSync(root, { withFileTypes: true }).flatMap((entry: Dirent) => {
		const entryPath = join(root, entry.name)
		if (entry.isDirectory()) {
			return sourceFiles(entryPath)
		}
		return /\.(ts|vue)$/.test(entry.name) ? [entryPath] : []
	})
}
