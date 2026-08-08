import {
	MailAccountReadinessV1,
	MailCredentialBindingStateV1,
} from '../../../gen/makosh/mail/account/v1/client_pb'
import { describe, expect, it, vi } from 'vitest'

import { MailLegacyRecoveryWorkflowV1 } from './mailLegacyRecoveryWorkflow'

const plan = {
	schemaRevision: 1 as const,
	recoverySessionId: 'a'.repeat(32),
	bundleFingerprintSha256: 'b'.repeat(64),
	counts: {
		gmailActive: 1 as const,
		icloudActive: 1 as const,
		telegramUserActive: 1 as const,
		gmailDeleted: 2 as const,
	},
	candidates: [],
}

describe('MailLegacyRecoveryWorkflowV1', () => {
	it('recovers iCloud with native-custodied Vault sealing and honest readiness', async () => {
		const sourceHandle = 'c'.repeat(64)
		const sealSource = vi.fn().mockResolvedValue({})
		const apply = vi.fn().mockResolvedValue({
			settings: { desiredRevision: 2n },
			application: {},
		})
		const bind = vi.fn().mockResolvedValue({})
		const status = vi.fn()
			.mockResolvedValueOnce({ binding: [] })
			.mockResolvedValueOnce({
				binding: [{ purpose: 1, credentialRevision: 1n }],
				readiness: MailAccountReadinessV1.MAIL_ACCOUNT_READINESS_READY,
			})
		const provisionCustodied = vi.fn().mockImplementation(async (_input, seal) => {
			await seal({
				hostSessionId: 'host-session',
				operationId: new Uint8Array(16).fill(1),
				action: 1,
				secretClass: 1,
				authorized: {},
			})
			return { secretRevision: 1n }
		})
		const workflow = new MailLegacyRecoveryWorkflowV1({
			source: {
				...receiptPort(),
				source: vi.fn().mockResolvedValue({
					kind: 'icloud',
					sourceHandle,
					accountId: 'mail-account',
					displayName: 'Private',
					email: 'owner@example.test',
					imapHost: 'imap.mail.me.com',
					imapPort: 993,
					username: 'owner@example.test',
				}),
				sealSource,
			},
			configuration: {
				createTarget: vi.fn().mockResolvedValue({
					configurationInstanceId: 'mail-target',
					desiredRevision: 1n,
					applyState: 'blocked_config',
				}),
				apply,
			},
			vault: { provisionCustodied },
			mail: { status, bind },
			oauth: { start: vi.fn(), complete: vi.fn() },
		} as never)

		const result = await workflow.recover({
			registrationId: 'mail-registration',
			plan,
			candidate: { sourceHandle, kind: 'icloud', state: 'ready_to_apply' },
		})

		expect(result).toEqual({ kind: 'icloud', state: 'ready' })
		expect(sealSource).toHaveBeenCalledWith(expect.objectContaining({
			recoverySessionId: plan.recoverySessionId,
			sourceHandle,
			secretPurpose: 'icloud_imap_password',
		}))
		expect(bind).toHaveBeenCalledWith(expect.objectContaining({
			connectionId: 'mail-account',
			credentialRevision: 1n,
		}))
		expect(apply.mock.calls[0]?.[0].values).toEqual(expect.arrayContaining([
			expect.objectContaining({ settingId: 'mail.inbound.kind' }),
			expect.objectContaining({ settingId: 'mail.imap.host' }),
		]))
	})

	it('creates Gmail configuration but requires current OAuth instead of importing a token', async () => {
		const sourceHandle = 'd'.repeat(64)
		const oauthStart = vi.fn().mockResolvedValue({
			setupId: 'setup',
			authorizationUrl: 'https://accounts.google.test/authorize',
		})
		const vault = { provisionCustodied: vi.fn() }
		const apply = vi.fn().mockResolvedValue({
			settings: { desiredRevision: 2n },
			application: {},
		})
		const workflow = new MailLegacyRecoveryWorkflowV1({
			source: {
				...receiptPort(),
				source: vi.fn().mockResolvedValue({
					kind: 'gmail',
					sourceHandle,
					accountId: 'gmail-account',
					displayName: 'Gmail',
					email: 'owner@example.test',
					oauthClientId: 'public-client',
					oauthRedirectUri: 'http://127.0.0.1/callback',
				}),
			},
			configuration: {
				createTarget: vi.fn().mockResolvedValue({
					configurationInstanceId: 'gmail-target',
					desiredRevision: 1n,
					applyState: 'blocked_config',
				}),
				apply,
			},
			vault,
			mail: { status: vi.fn(), bind: vi.fn() },
			oauth: { start: oauthStart, complete: vi.fn() },
		} as never)

		const result = await workflow.recover({
			registrationId: 'mail-registration',
			plan,
			candidate: { sourceHandle, kind: 'gmail', state: 'reauthorization_required' },
		})

		expect(result.kind).toBe('gmail')
		expect(result.state).toBe('reauthorization_required')
		expect(oauthStart).toHaveBeenCalledOnce()
		expect(vault.provisionCustodied).not.toHaveBeenCalled()
		const settings = apply.mock.calls[0]?.[0].values
		expect(settings).toEqual(expect.arrayContaining([
			expect.objectContaining({
				settingId: 'mail.gmail.user_id',
				value: { case: 'stringValue', value: 'me' },
			}),
		]))
		expect(settings).not.toEqual(expect.arrayContaining([
			expect.objectContaining({ settingId: 'mail.gmail.from_address' }),
		]))
	})

	it('reconciles a current Gmail target without changing Settings revision', async () => {
		const sourceHandle = 'd'.repeat(64)
		const apply = vi.fn()
		const oauthStart = vi.fn().mockResolvedValue({
			setupId: 'setup',
			authorizationUrl: 'https://accounts.google.test/authorize',
		})
		const workflow = new MailLegacyRecoveryWorkflowV1({
			source: {
				...receiptPort(),
				source: vi.fn().mockResolvedValue({
					kind: 'gmail',
					sourceHandle,
					accountId: 'gmail-account',
					displayName: 'Gmail',
					email: 'opaque-legacy-account',
					oauthClientId: 'public-client',
					oauthRedirectUri: 'http://127.0.0.1/callback',
				}),
			},
			configuration: {
				createTarget: vi.fn().mockResolvedValue({
					configurationInstanceId: 'gmail-target',
					desiredRevision: 7n,
					applyState: 'current',
				}),
				apply,
			},
			vault: { provisionCustodied: vi.fn() },
			mail: { status: vi.fn(), bind: vi.fn() },
			oauth: { start: oauthStart, complete: vi.fn() },
		} as never)

		await workflow.recover({
			registrationId: 'mail-registration',
			plan,
			candidate: { sourceHandle, kind: 'gmail', state: 'reauthorization_required' },
		})

		expect(apply).not.toHaveBeenCalled()
		expect(oauthStart).toHaveBeenCalledOnce()
	})

	it('reuses the existing Mail target for the same connection', async () => {
		const sourceHandle = 'a'.repeat(64)
		const createTarget = vi.fn()
		const oauthStart = vi.fn().mockResolvedValue({
			setupId: 'setup',
			authorizationUrl: 'https://accounts.google.test/authorize',
		})
		const workflow = new MailLegacyRecoveryWorkflowV1({
			source: {
				...receiptPort(),
				source: vi.fn().mockResolvedValue({
					kind: 'gmail',
					sourceHandle,
					accountId: 'gmail-account',
					displayName: 'Gmail',
					email: 'owner@example.test',
					oauthClientId: 'public-client',
					oauthRedirectUri: 'http://127.0.0.1/callback',
				}),
			},
			configuration: { createTarget, apply: vi.fn() },
			vault: { provisionCustodied: vi.fn() },
			mail: { status: vi.fn(), bind: vi.fn() },
			oauth: { start: oauthStart, complete: vi.fn() },
		} as never)

		await workflow.recover({
			registrationId: 'mail-registration',
			plan,
			candidate: { sourceHandle, kind: 'gmail', state: 'reauthorization_required' },
			settingsTargets: [{
				configurationInstanceId: 'existing-target',
				desiredRevision: 7n,
				effectiveRevision: 7n,
				applyState: 1,
				sanitizedReasonCode: '',
				values: [{
					settingId: 'mail.connection_id',
					displayName: 'Connection',
					value: { value: { case: 'stringValue', value: 'gmail-account' } },
				}],
			} as never],
		})

		expect(createTarget).not.toHaveBeenCalled()
		expect(oauthStart).toHaveBeenCalledOnce()
	})

	it('rebinds a persisted iCloud credential when provider binding state was lost', async () => {
		const sourceHandle = 'e'.repeat(64)
		const bind = vi.fn().mockResolvedValue({})
		const provisionCustodied = vi.fn()
		const status = vi.fn()
			.mockResolvedValueOnce({ binding: [] })
			.mockResolvedValueOnce({
				binding: [{ purpose: 1, credentialRevision: 1n }],
				readiness: MailAccountReadinessV1.MAIL_ACCOUNT_READINESS_READY,
			})
		const completedReceiptPort = {
			beginStep: vi.fn().mockImplementation(async (input) => ({
				disposition: 'completed',
				operationId: new Uint8Array(16).fill(1),
				targetConfigurationInstanceId: input.targetConfigurationInstanceId,
				publicRevision: 1n,
			})),
			completeStep: vi.fn().mockResolvedValue(undefined),
			finishCandidate: vi.fn().mockResolvedValue(undefined),
			cancel: vi.fn().mockResolvedValue(undefined),
		}
		const workflow = new MailLegacyRecoveryWorkflowV1({
			source: {
				...completedReceiptPort,
				source: vi.fn().mockResolvedValue({
					kind: 'icloud',
					sourceHandle,
					accountId: 'mail-account',
					displayName: 'Private',
					email: 'owner@example.test',
					imapHost: 'imap.mail.me.com',
					imapPort: 993,
					username: 'owner@example.test',
				}),
				sealSource: vi.fn(),
			},
			configuration: {
				createTarget: vi.fn().mockResolvedValue({
					configurationInstanceId: 'mail-target',
					desiredRevision: 7n,
					applyState: 'current',
				}),
				apply: vi.fn(),
			},
			vault: { provisionCustodied },
			mail: { status, bind },
			oauth: { start: vi.fn(), complete: vi.fn() },
		} as never)

		const result = await workflow.recover({
			registrationId: 'mail-registration',
			plan,
			candidate: {
				sourceHandle,
				kind: 'icloud',
				state: 'ready_to_apply',
				receiptTerminalState: 'completed',
			},
		})

		expect(result).toEqual({ kind: 'icloud', state: 'ready' })
		expect(provisionCustodied).not.toHaveBeenCalled()
		expect(bind).toHaveBeenCalledWith(expect.objectContaining({
			connectionId: 'mail-account',
			credentialRevision: 1n,
		}))
	})

	it('reprovisions a non-active iCloud binding before rebinding it', async () => {
		const sourceHandle = 'f'.repeat(64)
		const sealSource = vi.fn().mockResolvedValue({})
		const bind = vi.fn().mockResolvedValue({})
		const provisionCustodied = vi.fn().mockImplementation(async (_input, seal) => {
			await seal({
				hostSessionId: 'host-session',
				operationId: new Uint8Array(16).fill(1),
				action: 1,
				secretClass: 1,
				authorized: {},
			})
			return { secretRevision: 1n }
		})
		const status = vi.fn()
			.mockResolvedValueOnce({
				binding: [{
					purpose: 1,
					credentialRevision: 1n,
					bindingRevision: 4n,
					state: MailCredentialBindingStateV1
						.MAIL_CREDENTIAL_BINDING_STATE_PENDING_RESTART,
				}],
			})
			.mockResolvedValueOnce({
				binding: [{ purpose: 1, credentialRevision: 1n }],
				readiness: MailAccountReadinessV1.MAIL_ACCOUNT_READINESS_READY,
			})
		const workflow = new MailLegacyRecoveryWorkflowV1({
			source: {
				...receiptPort(),
				source: vi.fn().mockResolvedValue({
					kind: 'icloud',
					sourceHandle,
					accountId: 'mail-account',
					displayName: 'Private',
					email: 'owner@example.test',
					imapHost: 'imap.mail.me.com',
					imapPort: 993,
					username: 'owner@example.test',
				}),
				sealSource,
			},
			configuration: {
				createTarget: vi.fn().mockResolvedValue({
					configurationInstanceId: 'mail-target',
					desiredRevision: 7n,
					applyState: 'current',
				}),
				apply: vi.fn(),
			},
			vault: { provisionCustodied },
			mail: { status, bind },
			oauth: { start: vi.fn(), complete: vi.fn() },
		} as never)

		const result = await workflow.recover({
			registrationId: 'mail-registration',
			plan,
			candidate: { sourceHandle, kind: 'icloud', state: 'ready_to_apply' },
		})

		expect(result).toEqual({ kind: 'icloud', state: 'ready' })
		expect(provisionCustodied).toHaveBeenCalledWith(
			expect.objectContaining({ secretRevision: 1n }),
			expect.any(Function),
		)
		expect(sealSource).toHaveBeenCalledOnce()
		expect(bind).toHaveBeenCalledWith(expect.objectContaining({
			expectedBindingRevision: 4n,
			credentialRevision: 1n,
		}))
	})
})

function receiptPort() {
	return {
		beginStep: vi.fn().mockImplementation(async (input) => {
			const operationId = new Uint8Array(16)
			for (const [index, character] of [...input.stepIdentifier].entries()) {
				operationId[index % operationId.length] ^= character.charCodeAt(0)
			}
			return { disposition: 'execute', operationId }
		}),
		completeStep: vi.fn().mockResolvedValue(undefined),
		finishCandidate: vi.fn().mockResolvedValue(undefined),
		cancel: vi.fn().mockResolvedValue(undefined),
	}
}
