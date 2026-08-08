import { describe, expect, it } from 'vitest'

import {
	buildCanonicalAccountRows,
	buildCanonicalConversationRows,
	buildCanonicalMessageRows,
	buildCanonicalSearchRows,
	bytesKey,
} from './canonicalCommunicationsPageModel'

describe('canonical Communications presentation model', () => {
	it('maps generated metadata without provider-specific behavior', () => {
		const accountId = new Uint8Array([0xab, 0xcd])
		const accounts = buildCanonicalAccountRows([{
			$typeName: 'makosh.communications.query.v1.AccountSummaryV1',
			accountId,
			accountCursorSha256: new Uint8Array(32),
			provider: 2,
			firstObservedAtUnixSeconds: 1n,
			lastObservedAtUnixSeconds: 2n,
			lastEvidenceId: new Uint8Array([1]),
		}], bytesKey(accountId))

		expect(accounts[0]).toMatchObject({
			key: 'abcd',
			sourceLabel: 'Source 2',
			identityLabel: 'Account #abcd',
			selected: true,
		})
		expect(buildCanonicalConversationRows([], '')).toEqual([])
		expect(buildCanonicalMessageRows([], '')).toEqual([])
		expect(buildCanonicalSearchRows([], '')).toEqual([])
	})
})
