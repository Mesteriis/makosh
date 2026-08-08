<script setup lang="ts">
import { AttachmentPreviewContentTypeV1 } from '../../../gen/makosh/attachment_preview/v1/preview_pb'
import type { AttachmentPreviewPanelModel } from './attachmentPreviewPanelModel'
import './attachmentPreviewPanel.css'

defineProps<{ model: AttachmentPreviewPanelModel }>()

const emit = defineEmits<{ retry: [] }>()
</script>

<template>
	<section
		v-if="model.visible"
		class="attachment-preview"
		:aria-busy="model.busy"
		aria-labelledby="attachment-preview-title"
	>
		<header>
			<div>
				<span>Bounded workflow</span>
				<h2 id="attachment-preview-title">Attachment preview</h2>
				<p>Derived from current safe evidence; source bytes stay behind owner custody.</p>
			</div>
			<strong>{{ model.status }}</strong>
		</header>

		<p
			v-if="model.statusMessage"
			class="attachment-preview__status"
			:role="['error', 'rejected'].includes(model.status) ? 'alert' : 'status'"
		>{{ model.statusMessage }}</p>

		<div v-if="model.busy" class="attachment-preview__skeleton" aria-hidden="true">
			<span />
			<span />
			<span />
		</div>

		<pre
			v-else-if="model.status === 'ready' && model.contentType === AttachmentPreviewContentTypeV1.TEXT_UTF8"
			class="attachment-preview__text"
		>{{ model.artifactText }}</pre>
		<img
			v-else-if="model.status === 'ready' && model.contentType === AttachmentPreviewContentTypeV1.PNG"
			class="attachment-preview__image"
			:src="model.artifactUrl"
			alt="Derived attachment preview"
		>
		<audio
			v-else-if="model.status === 'ready' && model.contentType === AttachmentPreviewContentTypeV1.MPEG_AUDIO"
			class="attachment-preview__media"
			:src="model.artifactUrl"
			controls
			preload="metadata"
		/>
		<video
			v-else-if="model.status === 'ready' && model.contentType === AttachmentPreviewContentTypeV1.MP4_VIDEO"
			class="attachment-preview__video"
			:src="model.artifactUrl"
			controls
			preload="metadata"
		/>

		<button v-if="model.canRetry" type="button" @click="emit('retry')">Retry preview</button>
	</section>
</template>
