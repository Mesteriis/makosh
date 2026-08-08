import { describe, expect, it } from 'vitest'
import { existsSync, readFileSync } from 'node:fs'
import { fileURLToPath } from 'node:url'
import { join } from 'node:path'

const mediaComponents = [
	'Image',
	'ImagePreview',
	'ImageGallery',
	'VideoPlayer',
	'AudioPlayer',
	'MarkdownViewer',
	'CodeBlock',
	'SyntaxHighlight',
	'PDFViewer',
	'AttachmentPreview',
	'HtmlPreview'
]

describe('Макошь UI media component contracts', () => {
	it('keeps the media batch documented and exported through the UI kit', () => {
		const uiRoot = fileURLToPath(new URL('.', import.meta.url))
		const barrel = readFileSync(join(uiRoot, 'index.ts'), 'utf8')

		for (const componentName of mediaComponents) {
			expect(existsSync(join(uiRoot, `${componentName}.vue`))).toBe(true)
			expect(existsSync(join(uiRoot, `${componentName}.README.md`))).toBe(true)
			expect(barrel).toContain(`export { default as ${componentName} } from './${componentName}.vue'`)
		}
	})

	it('keeps media components UI-only', () => {
		const uiRoot = fileURLToPath(new URL('.', import.meta.url))
		const forbiddenSource = /@\/domains|@\/integrations|@\/platform|fetch\(|useQuery|defineStore|createRouter/

		for (const componentName of mediaComponents) {
			const source = readFileSync(join(uiRoot, `${componentName}.vue`), 'utf8')
			expect(source, componentName).not.toMatch(forbiddenSource)
		}
	})

	it('keeps parser and sanitizer boundaries explicit for rich text media', () => {
		const uiRoot = fileURLToPath(new URL('.', import.meta.url))
		const markdownSource = readFileSync(join(uiRoot, 'MarkdownViewer.vue'), 'utf8')
		const syntaxSource = readFileSync(join(uiRoot, 'SyntaxHighlight.vue'), 'utf8')
		const htmlSource = readFileSync(join(uiRoot, 'HtmlPreview.vue'), 'utf8')
		const renderingSource = readFileSync(join(uiRoot, 'Media.rendering.ts'), 'utf8')

		expect(renderingSource).toContain("from 'marked'")
		expect(renderingSource).toContain("from 'highlight.js'")
		expect(renderingSource).toContain("from 'dompurify'")
		expect(markdownSource).toContain('renderMarkdownToSafeHtml')
		expect(syntaxSource).toContain('highlightCodeToSafeHtml')
		expect(htmlSource).toContain('sanitizeHtml')
		expect(htmlSource).toContain('sanitized')
		expect(htmlSource).toContain('v-html')
		expect(htmlSource).toContain('isolated')
		expect(htmlSource).toContain('sandbox="allow-same-origin"')
		expect(htmlSource).toContain('srcdoc')
	})

	it('keeps media components represented in Storybook', () => {
		const storySource = readFileSync(
			fileURLToPath(new URL('../../../stories/ui/Media.stories.ts', import.meta.url)),
			'utf8'
		)

		for (const componentName of mediaComponents) {
			expect(storySource).toContain(componentName)
		}
	})
})
