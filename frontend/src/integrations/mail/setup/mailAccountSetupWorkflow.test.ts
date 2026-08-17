import { describe, expect, it, vi } from 'vitest'

import {
	MailAccountSetupWorkflowV1,
	mailGmailPreauthorizationSettings,
} from './mailAccountSetupWorkflow'

describe('MailAccountSetupWorkflowV1', () => {
	it('applies non-secret IMAP settings before Vault binding without a stale reapply', async () => {
		const order: string[] = []
		const createTarget = vi.fn().mockImplementation(async () => {
			order.push('target')
			return { configurationInstanceId: 'mail-target', desiredRevision: 1n }
		})
		const apply = vi.fn().mockImplementation(async () => {
			order.push('settings')
			return { settings: { desiredRevision: 2n }, application: {} }
		})
		const status = vi.fn().mockImplementation(async () => {
			order.push('status')
			return { binding: [] }
		})
		const provision = vi.fn().mockImplementation(async () => {
			order.push('vault')
			return { secretRevision: 1n }
		})
		const bind = vi.fn().mockImplementation(async () => {
			order.push('binding')
			return {}
		})
		const workflow = new MailAccountSetupWorkflowV1({
			configuration: { createTarget, apply },
			vault: { provision },
			mail: { status, bind },
			oauth: {} as never,
		} as never)

		await workflow.setupImap({
			registrationId: 'mail-registration',
			expectedDesiredRevision: 1n,
			connectionId: 'personal',
			imapHost: 'imap.example.com',
			imapPort: 993n,
			username: 'me@example.com',
			imapPassword: new TextEncoder().encode('secret'),
		})

		expect(order).toEqual(['target', 'settings', 'status', 'vault', 'binding'])
		expect(provision).toHaveBeenCalledWith(expect.objectContaining({
			capabilityId: 'mail.imap.credential-provisioning.v1',
			configurationInstanceId: 'mail-target',
			purposeId: 'mail_imap_password',
		}))
	})

	it('creates Gmail pre-authorization settings without a synthetic mailbox', () => {
		const values = mailGmailPreauthorizationSettings({
			connectionId: 'gmail-account',
			clientId: 'public-client',
			redirectUri: 'http://127.0.0.1/callback',
		})
		const byId = new Map(values.map((entry) => [entry.settingId, entry.value]))

		expect(byId.get('mail.gmail.user_id')).toEqual({
			case: 'stringValue',
			value: 'me',
		})
		expect(byId.has('mail.gmail.from_address')).toBe(false)
	})

	it('starts Gmail OAuth with provider-selected identity instead of a typed mailbox', async () => {
		const apply = vi.fn().mockResolvedValue({ settings: {}, application: {} })
		const start = vi.fn().mockResolvedValue({ setupId: 'setup', authorizationUrl: 'https://accounts.google.com/o/oauth2/v2/auth' })
		const workflow = new MailAccountSetupWorkflowV1({
			configuration: {
				createTarget: vi.fn().mockResolvedValue({
					configurationInstanceId: 'gmail-target',
					desiredRevision: 1n,
				}),
				apply,
			},
			oauth: { start },
			vault: {} as never,
			mail: {} as never,
		} as never)

		await workflow.startGmail({
			registrationId: 'mail-registration',
			expectedDesiredRevision: 1n,
			connectionId: 'gmail-account',
			clientId: 'public-client',
			redirectUri: 'http://127.0.0.1:5173/oauth/google/callback',
		})

		const values = apply.mock.calls[0]?.[0]?.values as Array<{
			settingId: string
			value: { case: string; value: unknown }
		}>
		const byId = new Map(values.map((entry) => [entry.settingId, entry.value]))
		expect(byId.get('mail.gmail.user_id')).toEqual({ case: 'stringValue', value: 'me' })
		expect(byId.has('mail.gmail.from_address')).toBe(false)
		expect(start).toHaveBeenCalledWith(expect.any(String), 'gmail-account')
	})
})
