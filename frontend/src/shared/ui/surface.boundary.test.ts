import { describe, expect, it } from 'vitest'
import { existsSync, readFileSync } from 'node:fs'
import { fileURLToPath } from 'node:url'
import { join } from 'node:path'

const surfaceComponents = [
	'Surface',
	'Paper',
	'Panel',
	'Card',
	'CardHeader',
	'CardTitle',
	'CardDescription',
	'CardContent',
	'CardFooter',
	'Divider',
	'Section',
	'Accordion',
	'Callout',
	'Well',
	'Fieldset',
	'ToolbarSection',
	'StatCard',
	'ActionCard'
]

describe('Макошь UI surface component contracts', () => {
	it('keeps the surface pack documented and exported through the UI kit', () => {
		const uiRoot = fileURLToPath(new URL('.', import.meta.url))
		const barrel = readFileSync(join(uiRoot, 'index.ts'), 'utf8')
		const missingContracts = surfaceComponents.flatMap((componentName) => {
			const violations = []
			const componentPath = surfaceComponentPath(componentName)
			const barrelPath = `./${componentPath}`

			if (!existsSync(join(uiRoot, componentPath))) {
				violations.push(`${componentName}.vue`)
			}
			if (!existsSync(join(uiRoot, `${componentName}.README.md`))) {
				violations.push(`${componentName}.README.md`)
			}
			if (!barrel.includes(`export { default as ${componentName} } from '${barrelPath}'`)) {
				violations.push(`${componentName} barrel export`)
			}

			return violations
		})

		expect(missingContracts).toEqual([])
	})

	it('keeps surface components presentation-only and independent from app data boundaries', () => {
		const uiRoot = fileURLToPath(new URL('.', import.meta.url))
		const violations = surfaceComponents
			.filter((componentName) => existsSync(join(uiRoot, surfaceComponentPath(componentName))))
			.flatMap((componentName) => {
				const source = readFileSync(join(uiRoot, surfaceComponentPath(componentName)), 'utf8')
				return forbiddenUiBoundaryMatches(source).map((match) => `${componentName}: ${match}`)
			})

		expect(violations).toEqual([])
	})

	it('keeps all surface components represented in the General Surface Storybook story', () => {
		const storyPath = fileURLToPath(new URL('../../../stories/ui/GeneralSurface.stories.ts', import.meta.url))

		expect(existsSync(storyPath)).toBe(true)

		const storySource = existsSync(storyPath) ? readFileSync(storyPath, 'utf8') : ''
		const storyImports = storybookUiImports(storySource)

		expect(surfaceComponents.filter((componentName) => !storyImports.includes(componentName))).toEqual([])
	})

	it('keeps surface clipping explicit in the dedicated surface stylesheet', () => {
		const stylesPath = fileURLToPath(new URL('./styles/surfaces.css', import.meta.url))

		expect(existsSync(stylesPath)).toBe(true)

		const cssSource = existsSync(stylesPath) ? readFileSync(stylesPath, 'utf8') : ''

		expect(cssSource).toContain('.makosh-surface--clip')
		expect(cssSource).toContain('.makosh-card--clip')
		expect(surfaceSelectorsWithImplicitOverflowHidden(cssSource, 'makosh-surface')).toEqual([])
		expect(surfaceSelectorsWithImplicitOverflowHidden(cssSource, 'makosh-card')).toEqual([])
	})

	it('keeps Card signal state presentation-only and motion-safe', () => {
		const uiRoot = fileURLToPath(new URL('.', import.meta.url))
		const cardSource = readFileSync(join(uiRoot, 'primitives/Card.vue'), 'utf8')
		const cssSource = readFileSync(join(uiRoot, 'styles/surfaces.css'), 'utf8')
		const storySource = readFileSync(join(uiRoot, '../../../stories/ui/GeneralSurface.stories.ts'), 'utf8')
		const signalCss = cssSource.slice(cssSource.indexOf('.makosh-card--signal'), cssSource.indexOf('.makosh-card-header'))

		expect(cardSource).toContain('signal?: boolean')
		expect(cardSource).toContain('signalTone?: CardSignalTone')
		expect(cardSource).toContain('signalPulse?: boolean')
		expect(cssSource).toContain('.makosh-card--signal')
		expect(cssSource).toContain('@keyframes makosh-card-signal-border')
		expect(cssSource).toContain('@keyframes makosh-card-signal-inner')
		expect(signalCss).not.toContain('conic-gradient')
		expect(signalCss).not.toContain('mask-composite')
		expect(signalCss).not.toContain('rotate(')
		expect(signalCss).not.toContain('makosh-card-signal-edge')
		expect(cssSource).toContain('@media (prefers-reduced-motion: reduce)')
		expect(storySource).toContain(':signal="signalCard.active"')
		expect(storySource).toContain(':signal-tone="signalCard.tone"')
	})
})

function surfaceComponentPath(componentName: string): string {
	return componentName === 'Card' ? 'primitives/Card.vue' : `${componentName}.vue`
}

function storybookUiImports(source: string): string[] {
	return Array.from(source.matchAll(/import \{([^}]+)\} from '@\/shared\/ui'/g))
		.flatMap((match) => match[1].split(','))
		.map((name) => name.trim())
		.filter(Boolean)
		.sort()
}

function forbiddenUiBoundaryMatches(source: string): string[] {
	const forbiddenPatterns = [
		{
			name: 'state, router, or network API reference',
			pattern:
				/\b(?:fetch|XMLHttpRequest|WebSocket|EventSource|\$fetch|useQuery|useInfiniteQuery|useMutation|useQueryClient|QueryClient|defineStore|useStore|createRouter|useRouter|useRoute|axios|ky|http)\b/g
		}
	]

	const forbiddenPathReferences = stringLiteralValues(source)
		.filter(isForbiddenAppBoundaryPath)
		.map((path) => `app boundary path reference (${path})`)
	const forbiddenPackageReferences = stringLiteralValues(source)
		.filter(isForbiddenPackageReference)
		.map((path) => `router, query, or network package reference (${path})`)
	const forbiddenPatternReferences = forbiddenPatterns.flatMap(({ name, pattern }) => {
		return Array.from(source.matchAll(pattern)).map((match) => `${name} (${match[0]})`)
	})

	return [...forbiddenPathReferences, ...forbiddenPackageReferences, ...forbiddenPatternReferences]
}

function stringLiteralValues(source: string): string[] {
	return Array.from(source.matchAll(/(['"`])([^'"`]+)\1/g)).map((match) => match[2])
}

function isForbiddenAppBoundaryPath(path: string): boolean {
	const normalizedPath = appRelativePath(path)
	const forbiddenRoots = [
		'domains',
		'integrations',
		'platform',
		'stores',
		'router',
		'routers',
		'app/router',
		'app/routers',
		'shared/router',
		'shared/routers',
		'shared/stores'
	]

	return normalizedPath !== null && forbiddenRoots.some((root) => normalizedPath === root || normalizedPath.startsWith(`${root}/`))
}

function isForbiddenPackageReference(path: string): boolean {
	const forbiddenPackageRoots = [
		'vue-router',
		'axios',
		'ky',
		'ofetch',
		'@tanstack/vue-query',
		'@tauri-apps/plugin-http'
	]

	return forbiddenPackageRoots.some((root) => path === root || path.startsWith(`${root}/`))
}

function appRelativePath(path: string): string | null {
	if (path.startsWith('@/')) {
		return path.slice(2)
	}
	const relativePath = path.match(/^(?:\.\.\/)+(.+)$/)
	return relativePath?.[1] ?? null
}

function surfaceSelectorsWithImplicitOverflowHidden(source: string, className: string): string[] {
	return Array.from(source.matchAll(/([^{}]+)\{([^{}]+)\}/g))
		.filter((match) => selectorListIncludesSurfaceClassWithoutClip(match[1], className))
		.filter((match) => /\boverflow\s*:\s*hidden\b/.test(match[2]))
		.map((match) => match[1].trim())
}

function selectorListIncludesSurfaceClassWithoutClip(selectorList: string, className: string): boolean {
	return selectorList
		.split(',')
		.map((selector) => selector.trim())
		.some((selector) => selectorTargetsSurfaceClass(selector, className) && !selectorIncludesPositiveClipToken(selector, className))
}

function selectorTargetsSurfaceClass(selector: string, className: string): boolean {
	return Array.from(selector.matchAll(/\.([_a-zA-Z][\w-]*)/g))
		.map((match) => match[1])
		.some((classToken) => classToken === className || classToken.startsWith(`${className}--`))
}

function selectorIncludesPositiveClipToken(selector: string, className: string): boolean {
	const selectorWithoutNegations = selector.replace(/:not\([^)]*\)/g, '')
	return Array.from(selectorWithoutNegations.matchAll(/\.([_a-zA-Z][\w-]*)/g))
		.map((match) => match[1])
		.includes(`${className}--clip`)
}
