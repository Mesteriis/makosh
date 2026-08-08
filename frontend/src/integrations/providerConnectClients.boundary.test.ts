import { readFileSync } from 'node:fs'

import { describe, expect, it } from 'vitest'

const clientUnits = [
	{
		file: './mail/api/mailSyncClient.ts',
		generatedContract: '../../../gen/makosh/mail/v1/client_pb',
		service: 'MailSyncService',
		foreignOwners: ['telegram', 'zulip']
	},
	{
		file: './mail/api/mailDeliveryCommandClient.ts',
		generatedContract: '../../../gen/makosh/mail/v1/client_pb',
		service: 'MailDeliveryCommandService',
		foreignOwners: ['telegram', 'zulip']
	},
	{
		file: './mail/api/mailDeliveryQueryClient.ts',
		generatedContract: '../../../gen/makosh/mail/v1/client_pb',
		service: 'MailDeliveryQueryService',
		foreignOwners: ['telegram', 'zulip']
	},
	{
		file: './telegram/api/telegramOperationalClient.ts',
		generatedContract: '../../../gen/makosh/telegram/v1/client_pb',
		service: 'TelegramOperationalService',
		foreignOwners: ['mail', 'zulip']
	},
	{
		file: './zulip/api/zulipCommandClient.ts',
		generatedContract: '../../../gen/makosh/zulip/v1/client_pb',
		service: 'ZulipCommandService',
		foreignOwners: ['mail', 'telegram']
	},
	{
		file: './zulip/api/zulipQueryClient.ts',
		generatedContract: '../../../gen/makosh/zulip/v1/client_pb',
		service: 'ZulipQueryService',
		foreignOwners: ['mail', 'telegram']
	},
	{
		file: './whatsapp/api/whatsappCommandClient.ts',
		generatedContract: '../../../gen/makosh/whatsapp/v1/client_pb',
		service: 'WhatsAppCommandService',
		foreignOwners: ['mail', 'telegram', 'zulip']
	},
	{
		file: './whatsapp/api/whatsappQueryClient.ts',
		generatedContract: '../../../gen/makosh/whatsapp/v1/client_pb',
		service: 'WhatsAppQueryService',
		foreignOwners: ['mail', 'telegram', 'zulip']
	}
] as const

describe('provider operational Connect client boundaries', () => {
	it.each(clientUnits)('$file binds only $service to the shared Gateway transport', (unit) => {
		const source = readFileSync(new URL(unit.file, import.meta.url), 'utf8')

		expect(source).toContain(unit.generatedContract)
		expect(source).toContain(unit.service)
		expect(source).toContain('../../../platform/gateway/browserGatewayConnect')
		expect(source).toContain('createBrowserGatewayConnectTransport')
		expect(source).not.toContain('/api/v1/')
		expect(source).not.toContain('CommunicationsService')
		expect(source).not.toContain('communicationsClient')
		expect(source).not.toContain('ApiClient')
		expect(source).not.toContain('fetch(')

		for (const foreignOwner of unit.foreignOwners) {
			expect(source).not.toContain(`/makosh/${foreignOwner}/`)
		}
	})

	it('generates every admitted provider contract from its owner package', () => {
		const generator = readFileSync(
			new URL('../../scripts/generate-proto.mjs', import.meta.url),
			'utf8'
		)

		expect(generator).toContain("backend', 'src', 'mail-api', 'proto")
		expect(generator).toContain("backend', 'src', 'telegram-api', 'proto")
		expect(generator).toContain("backend', 'src', 'whatsapp-api', 'proto")
		expect(generator).toContain("backend', 'src', 'zulip-api', 'proto")
		expect(generator.match(/'makosh', '(mail|telegram|whatsapp|zulip)', 'v1', 'client\.proto'/g)).toHaveLength(
			4
		)
	})
})
