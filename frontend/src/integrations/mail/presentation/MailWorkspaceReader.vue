<script setup lang="ts">
import { computed } from 'vue'
import { HtmlPreview, Icon } from '@/shared/ui'
import { rewriteRemoteImageSources, sanitizeEmailHtml } from '@/shared/sanitize/emailHtml'
import type { MailMessageDetailCard } from './mailOperationalReadModel'

const props = defineProps<{
	detail: MailMessageDetailCard | null
	inspectorVisible: boolean
	bodyContentStatus: 'idle' | 'loading' | 'ready' | 'unavailable'
	bodyContentStatusMessage: string
	bodyText: string
	bodyFormat: 'text' | 'html'
}>()

const renderedBody = computed(() => props.bodyFormat === 'html'
	? rewriteRemoteImageSources(sanitizeEmailHtml(props.bodyText), () => null)
	: props.bodyText)

const emit = defineEmits<{
	toggleInspector: []
}>()
</script>

<template>
	<main class="mail-workspace-reader" aria-label="Open message">
		<article v-if="detail" class="mail-message">
			<nav class="mail-message-actions" aria-label="Message actions">
				<button type="button" title="Reply" disabled><Icon icon="tabler:corner-up-left" size="1rem" /></button>
				<button type="button" title="Reply all" disabled><Icon icon="tabler:arrow-back-up-double" size="1rem" /></button>
				<button type="button" title="Forward" disabled><Icon icon="tabler:arrow-forward-up" size="1rem" /></button>
				<span />
				<button
					type="button"
					:title="inspectorVisible ? 'Hide details' : 'Show details'"
					:aria-pressed="inspectorVisible"
					@click="emit('toggleInspector')"
				>
					<Icon icon="tabler:layout-sidebar-right" size="1rem" />
				</button>
			</nav>

			<section class="mail-message-paper">
				<header class="mail-message-envelope">
					<h2>{{ detail.subject }}</h2>
					<dl>
						<div><dt>From</dt><dd>{{ detail.sender }}</dd></div>
						<div><dt>To</dt><dd>{{ detail.recipients }}</dd></div>
					</dl>
					<time>{{ detail.meta }}</time>
				</header>

				<div class="mail-message-context">
					<span><Icon icon="tabler:mail-opened" size="0.9rem" /> {{ detail.flags }}</span>
					<span><Icon icon="tabler:folder" size="0.9rem" /> {{ detail.folders }}</span>
					<span><Icon icon="tabler:link" size="0.9rem" /> {{ detail.evidenceState }}</span>
				</div>

				<div class="mail-message-body">
					<div v-if="bodyContentStatus === 'loading'" class="mail-message-body__skeleton" aria-label="Loading message body" />
					<HtmlPreview
						v-else-if="bodyContentStatus === 'ready'"
						class="mail-message-body__content"
						:content="renderedBody"
						:format="bodyFormat"
						:sanitized="bodyFormat === 'html'"
						:isolated="bodyFormat === 'html'"
						empty-label="No message body"
					/>
					<p v-else>{{ bodyContentStatusMessage || detail.snippet }}</p>
					<aside v-if="bodyContentStatus !== 'ready'">
						<Icon icon="tabler:info-circle" size="1rem" />
						<span>{{ detail.contentState }}</span>
					</aside>
				</div>
			</section>
		</article>

		<section v-else class="mail-workspace-reader__empty">
			<Icon icon="tabler:mail-opened" size="2rem" />
			<h2>No message selected</h2>
			<p>Select a message from the list to open it.</p>
		</section>
	</main>
</template>
