import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { ApiClient } from '../../../platform/api/ApiClient'
import { fetchReviewAttentionCards } from './items'
import type { AttentionCard } from '../types/review'

describe('review items API', () => {
	beforeEach(() => {
		ApiClient.resetForTests()
		ApiClient.init('http://127.0.0.1:8080', 'test-secret')
	})

	afterEach(() => {
		vi.unstubAllGlobals()
		ApiClient.resetForTests()
	})

	it('fetches Attention Cards through the protected Review API', async () => {
		const card = attentionCard()
		const fetchMock = vi.fn().mockResolvedValue(
			new Response(JSON.stringify({ cards: [card] }), {
				status: 200,
				headers: { 'Content-Type': 'application/json' }
			})
		)
		vi.stubGlobal('fetch', fetchMock)

		const response = await fetchReviewAttentionCards({ status: 'active', limit: 25 })

		expect(response.cards).toEqual([card])
		expect(fetchMock).toHaveBeenCalledOnce()
		const [url, options] = fetchMock.mock.calls[0]
		expect(url).toBe('http://127.0.0.1:8080/api/v1/review/attention-cards?status=active&limit=25')
		expect(options.method).toBe('GET')
		expect(new Headers(options.headers).get('X-Макошь-Secret')).toBe('test-secret')
	})
})

function attentionCard(): AttentionCard {
	return {
		id: 'attention:review:contract-deadline',
		title: 'Contract review reminder',
		summary: 'A source-backed message suggests a deadline-backed task.',
		importance: 'high',
		confidence: 0.91,
		evidence_count: 2,
		related_entities: [{
			entity_kind: 'project',
			entity_id: 'project:contract',
			label: 'projects'
		}],
		trace_id: 'trace-1',
		review_item_ids: ['review-1', 'review-2'],
		suggested_actions: [{
			action_kind: 'approve',
			label: 'Approve',
			target_domain: null,
			target_entity_kind: null
		}],
		source_summary: 'Review item has 2 canonical observation evidence reference(s).',
		explainability: {
			why_this_matters: 'This potential task has strong confidence and source evidence.',
			evidence: [
				{ observation_id: 'obs-1', role: 'primary' },
				{ observation_id: 'obs-2', role: 'primary' }
			],
			confidence: {
				score: 0.91,
				rationale: '91% confidence from 2 evidence source(s).'
			},
			related_objects: [{
				entity_kind: 'project',
				entity_id: 'project:contract',
				label: 'projects'
			}],
			suggested_actions: [{
				action_kind: 'approve',
				label: 'Approve',
				target_domain: null,
				target_entity_kind: null
			}]
		}
	}
}
