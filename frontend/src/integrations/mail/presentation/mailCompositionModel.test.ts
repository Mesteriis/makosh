import { describe, expect, it } from 'vitest'

import {
	buildDraftOptions,
	parseTemplateValues,
	splitEditorLines,
} from './mailCompositionModel'

describe('Mail composition presentation model', () => {
	it('keeps recipient parsing bounded to explicit editor separators', () => {
		expect(splitEditorLines('one@example.test;\ntwo@example.test, three@example.test')).toEqual([
			'one@example.test',
			'two@example.test',
			'three@example.test',
		])
	})

	it('parses only explicit name=value template entries', () => {
		expect(parseTemplateValues('owner=AVM\nignored\nteam=Макошь=Hub')).toEqual({
			owner: 'AVM',
			team: 'Макошь=Hub',
		})
	})

	it('builds draft rows without exposing message bodies', () => {
		const rows = buildDraftOptions([{
			draftId: 'draft-1',
			subject: 'Private subject',
			textBody: 'private body',
			toRecipient: ['owner@example.test'],
			ccRecipient: [],
			bccRecipient: [],
			revision: 2n,
		} as never])
		expect(rows).toEqual([{
			id: 'draft-1',
			label: 'Private subject',
			detail: '1 recipients · r2',
		}])
		expect(JSON.stringify(rows)).not.toContain('private body')
	})
})
