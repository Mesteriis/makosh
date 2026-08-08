import type { Meta, StoryObj } from '@storybook/vue3-vite'
import { expect, within } from 'storybook/test'
import {
	AttachmentPreview,
	AudioPlayer,
	Button,
	CodeBlock,
	HtmlPreview,
	Image,
	ImageGallery,
	ImagePreview,
	MarkdownViewer,
	PDFViewer,
	SyntaxHighlight,
	VideoPlayer
} from '@/shared/ui'
import { storybookLocaleFromGlobals, storybookText } from './storybook-i18n'

const imageSources = [
	svgDataUri('#e6f7f4', '#0f766e', 'Timeline'),
	svgDataUri('#eef2ff', '#3730a3', 'Context'),
	svgDataUri('#fff7ed', '#c2410c', 'Review')
]

const meta = {
	title: 'Макошь UI/General/Media',
	component: Image,
	args: {
		src: imageSources[0],
		alt: 'Timeline artifact',
		caption: 'Media preview',
		ratio: 'video'
	}
} satisfies Meta<typeof Image>

export default meta
type Story = StoryObj<typeof meta>

export const ImagesAndPlayback: Story = {
	render: (_args, context) => ({
		components: {
			AudioPlayer,
			Button,
			Image,
			ImageGallery,
			ImagePreview,
			VideoPlayer
		},
		data() {
			const text = storybookText(storybookLocaleFromGlobals(context.globals))
			return {
				text,
				imageSources,
				galleryItems: text.media.galleryItems.map((item, index) => ({
					...item,
					src: imageSources[index] ?? imageSources[0]
				}))
			}
		},
		template: `
			<section class="storybook-canvas">
				<div class="storybook-section">
					<h2>{{ text.media.title }}</h2>
					<p>{{ text.media.description }}</p>
				</div>

				<div class="storybook-grid">
					<div class="storybook-section">
						<h3>{{ text.media.imagesTitle }}</h3>
						<Image :src="imageSources[0]" :alt="text.media.galleryItems[0].alt" :caption="text.media.imageCaption" ratio="video" />
					</div>
					<ImagePreview
						:src="imageSources[1]"
						:alt="text.media.galleryItems[1].alt"
						:title="text.media.galleryItems[1].title"
						:description="text.media.galleryItems[1].description"
						:meta="text.media.galleryItems[1].meta"
					>
						<template #actions>
							<Button size="sm" variant="outline">{{ text.common.review }}</Button>
						</template>
					</ImagePreview>
				</div>

				<ImageGallery :items="galleryItems" :label="text.media.galleryLabel" :empty-label="text.media.emptyImage" />

				<div class="storybook-grid">
					<VideoPlayer
						:title="text.media.videoTitle"
						:description="text.media.videoDescription"
						:fallback-label="text.media.videoFallback"
					/>
					<AudioPlayer
						:title="text.media.audioTitle"
						:description="text.media.audioDescription"
						:fallback-label="text.media.audioFallback"
					/>
				</div>
			</section>
		`
	}),
	play: async ({ canvasElement, globals }) => {
		const text = storybookText(storybookLocaleFromGlobals(globals))
		const canvas = within(canvasElement)
		await expect(canvas.getByText(text.media.title)).toBeVisible()
		await expect(canvas.getByText(text.media.videoFallback)).toBeVisible()
		await expect(canvas.getByLabelText(text.media.galleryLabel)).toBeVisible()
	}
}

export const DocumentsAndCode: Story = {
	render: (_args, context) => ({
		components: {
			CodeBlock,
			HtmlPreview,
			MarkdownViewer,
			PDFViewer,
			SyntaxHighlight
		},
		data() {
			return {
				text: storybookText(storybookLocaleFromGlobals(context.globals)),
				jsonSource: '{\n  "source": "local",\n  "reviewed": true\n}'
			}
		},
		template: `
			<section class="storybook-canvas">
				<div class="storybook-section">
					<h2>{{ text.media.documentsTitle }}</h2>
					<p>{{ text.media.description }}</p>
				</div>

				<div class="storybook-grid">
					<MarkdownViewer :title="text.media.markdownTitle" :source="text.media.markdownSource" />
					<HtmlPreview
						:title="text.media.htmlTitle"
						format="html"
						sanitized
						:content="text.media.htmlSource"
						:unsafe-label="text.media.unsafeHtml"
					/>
				</div>

				<div class="storybook-grid">
					<CodeBlock :label="text.media.codeTitle" language="ts" :code="text.media.codeSource" show-line-numbers />
					<SyntaxHighlight :label="text.media.syntaxTitle" language="json" :code="jsonSource" />
				</div>

				<div class="storybook-grid">
					<HtmlPreview
						:title="text.media.htmlTitle"
						format="text"
						:content="text.media.textSource"
						:unsafe-label="text.media.unsafeHtml"
					/>
					<PDFViewer
						:title="text.media.pdfTitle"
						:description="text.media.pdfDescription"
						:fallback-label="text.media.pdfFallback"
					/>
				</div>
			</section>
		`
	}),
	play: async ({ canvasElement, globals }) => {
		const text = storybookText(storybookLocaleFromGlobals(globals))
		const canvas = within(canvasElement)
		await expect(canvas.getByText(text.media.documentsTitle)).toBeVisible()
		await expect(canvas.getByText(text.media.markdownTitle)).toBeVisible()
		await expect(canvas.getByText(text.media.pdfFallback)).toBeVisible()
	}
}

export const Attachments: Story = {
	render: (_args, context) => ({
		components: {
			AttachmentPreview,
			Button,
			HtmlPreview
		},
		data() {
			return { text: storybookText(storybookLocaleFromGlobals(context.globals)) }
		},
		template: `
			<section class="storybook-canvas">
				<div class="storybook-section">
					<h2>{{ text.media.attachmentsTitle }}</h2>
					<p>{{ text.media.description }}</p>
				</div>

				<div class="storybook-stack">
					<AttachmentPreview
						v-for="attachment in text.media.attachments"
						:key="attachment.id"
						:name="attachment.name"
						:mime-type="attachment.mimeType"
						:size="attachment.size"
						:description="attachment.description"
						:icon="attachment.icon"
						:tone="attachment.tone"
					>
						<template #action>
							<Button size="sm" variant="outline">{{ text.media.attachmentAction }}</Button>
						</template>
					</AttachmentPreview>
				</div>

				<HtmlPreview
					:title="text.media.htmlTitle"
					format="html"
					:content="text.media.htmlSource"
					:unsafe-label="text.media.unsafeHtml"
				/>
			</section>
		`
	}),
	play: async ({ canvasElement, globals }) => {
		const text = storybookText(storybookLocaleFromGlobals(globals))
		const canvas = within(canvasElement)
		await expect(canvas.getByText(text.media.attachmentsTitle)).toBeVisible()
		await expect(canvas.getByText(text.media.attachments[0].name)).toBeVisible()
		await expect(canvas.getByText(text.media.unsafeHtml)).toBeVisible()
	}
}

function svgDataUri(background: string, foreground: string, label: string): string {
	const svg = `<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 640 360" role="img" aria-label="${label}"><rect width="640" height="360" rx="28" fill="${background}"/><path d="M72 256 204 124l94 94 58-58 212 212H72Z" fill="${foreground}" opacity=".24"/><circle cx="488" cy="104" r="44" fill="${foreground}" opacity=".32"/><text x="72" y="86" fill="${foreground}" font-family="ui-sans-serif, system-ui" font-size="44" font-weight="700">${label}</text></svg>`
	return `data:image/svg+xml,${encodeURIComponent(svg)}`
}
