import { readFileSync } from 'node:fs'
import { describe, expect, it } from 'vitest'

describe('Telegram media layout stability', () => {
	it('keeps avatar identity and geometry stable while changing chats', () => {
		const chatList = readFileSync(new URL('./TelegramWorkspaceChatList.vue', import.meta.url), 'utf8')
		const thread = readFileSync(new URL('./TelegramWorkspaceThread.vue', import.meta.url), 'utf8')
		const styles = readFileSync(new URL('./telegramOperationalPage.css', import.meta.url), 'utf8')

		expect(chatList).toContain(':key="chat.avatarProviderFileId"')
		expect(chatList).toContain('cache-class="avatar"')
		expect(thread).toContain(':key="model.selectedChatAvatarProviderFileId"')
		expect(thread).toContain('cache-class="avatar"')
		expect(thread).toContain('{{ selectedChatDetail }}')
		expect(thread).not.toContain('<p>{{ model.selectedChatId }}</p>')
		expect(thread).toContain('initialTelegramHistoryScrollTop')
		expect(styles).toMatch(
			/\.telegram-workspace-chat\s*\{[^}]*grid-template-columns:\s*2\.4rem minmax\(0, 1fr\)[^}]*min-height:\s*4rem/s,
		)
		expect(styles).toMatch(
			/\.telegram-thread-message__provider-media\s*\{[^}]*aspect-ratio:\s*16 \/ 10/s,
		)
	})

	it('keeps an explicit loading state while full provider media materializes', () => {
		const media = readFileSync(new URL('./TelegramProviderMedia.vue', import.meta.url), 'utf8')
		const query = readFileSync(new URL('../queries/useTelegramProviderMedia.ts', import.meta.url), 'utf8')

		expect(query).toContain('loading.value = true')
		expect(query).toContain('if (active) loading.value = false')
		expect(query).toContain('loadStarted = false')
		expect(media).toContain(':disabled="loading"')
		expect(media).toContain('Loading Telegram ${kind}')
		expect(media).toContain('Retry Telegram ${kind}')
		expect(media).toContain('telegram-provider-media__spinner')
	})
})
