// Historical pre-clean-room transport specification. Not part of the active test suite.
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { ApiClient } from '../../../platform/api/ApiClient'
import {
  analyzeMessage,
  addMessageLabel,
  bulkMessageAction,
  createDraft,
  createSavedSearch,
  detectMessageLanguage,
  deleteMessageFromProvider,
  deleteDraft,
  deleteSavedSearch,
  deleteRichTemplate,
  extractMessageNotes,
  extractMessageTasks,
  fetchCommunicationMessages,
  fetchDrafts,
  fetchCommunicationBlockers,
  fetchPersonas,
  fetchMessageAuth,
  fetchMessageExplain,
  fetchMailboxHealth,
  fetchMessageSignature,
  fetchMessageSmartCc,
  fetchMessageStateCounts,
  generateAiReply,
  generateAiReplyVariants,
  fetchSubscriptions,
  fetchTopSenders,
  fetchRichTemplates,
  fetchOutboxItems,
  runWorkflowAction,
  searchEmails,
  sendEmail,
  exportMessage,
  fetchThreadMessages,
  fetchThreads,
  previewRichTemplateMailMerge,
  renderRichTemplate,
  restoreMessage,
  snoozeMessage,
  markMessageRead,
  toggleMessageImportant,
  toggleMessageMute,
  toggleMessagePin,
  translateMessage,
  saveRichTemplate,
  fetchSavedSearches,
  transitionMessageWorkflowState,
  trashMessage,
  undoOutboxItem,
  updateSavedSearch
} from './communications'

describe('communications API', () => {
  beforeEach(() => {
    ApiClient.resetForTests()
    ApiClient.init('http://127.0.0.1:8080', 'test-secret')
  })

  afterEach(() => {
    vi.unstubAllGlobals()
    ApiClient.resetForTests()
  })

  it('passes cursor pagination parameters to the mail messages endpoint', async () => {
    const fetchMock = vi.fn().mockResolvedValue(
      new Response(JSON.stringify({
        items: [],
        nextCursor: 'next-cursor',
        hasMore: true
      }), {
        status: 200,
        headers: { 'Content-Type': 'application/json' }
      })
    )
    vi.stubGlobal('fetch', fetchMock)

    const response = await fetchCommunicationMessages(
      'account-1',
      'new',
      'email',
      'quarterly update',
      'active',
      50,
      'cursor:value'
    )

    expect(response.next_cursor).toBe('next-cursor')
    expect(response.has_more).toBe(true)
    expect(fetchMock).toHaveBeenCalledOnce()
    const [url] = fetchMock.mock.calls[0]
    expect(url).toBe('http://127.0.0.1:8080/makosh.communications.v1.CommunicationsService/ListMessages')
    expect(JSON.parse(decodeBody(fetchMock.mock.calls[0][1].body))).toMatchObject({
      accountId: 'account-1',
      workflowState: 'new',
      channelKind: 'email',
      query: 'quarterly update',
      localState: 'active',
      limit: 50,
      cursor: 'cursor:value'
    })
  })

  it('routes workflow transition, state counts and search through CommunicationsService', async () => {
    const fetchMock = vi
      .fn()
      .mockResolvedValueOnce(
        new Response(JSON.stringify({
          messageId: 'msg-1',
          workflowState: 'reviewed',
          previousState: 'new'
        }), {
          status: 200,
          headers: { 'Content-Type': 'application/json' }
        })
      )
      .mockResolvedValueOnce(
        new Response(JSON.stringify({
          counts: [{ state: 'reviewed', count: 3 }]
        }), {
          status: 200,
          headers: { 'Content-Type': 'application/json' }
        })
      )
      .mockResolvedValueOnce(
        new Response(JSON.stringify({
          results: [{ objectId: 'msg-1', objectKind: 'communication_message', title: 'Result' }]
        }), {
          status: 200,
          headers: { 'Content-Type': 'application/json' }
        })
      )
    vi.stubGlobal('fetch', fetchMock)

    const transition = await transitionMessageWorkflowState('msg-1', 'reviewed')
    const counts = await fetchMessageStateCounts('account-1', 'active')
    const search = await searchEmails('invoice', 12)

    expect(transition.previous_state).toBe('new')
    expect(counts.counts[0]).toEqual({ state: 'reviewed', count: 3 })
    expect(search.results[0].object_kind).toBe('communication_message')
    expect(fetchMock.mock.calls[0][0]).toBe(
      'http://127.0.0.1:8080/makosh.communications.v1.CommunicationsService/TransitionMessageWorkflowState'
    )
    expect(fetchMock.mock.calls[1][0]).toBe(
      'http://127.0.0.1:8080/makosh.communications.v1.CommunicationsService/ListMessageWorkflowStateCounts'
    )
    expect(fetchMock.mock.calls[2][0]).toBe(
      'http://127.0.0.1:8080/makosh.communications.v1.CommunicationsService/SearchMessages'
    )
  })

  it('routes message language, translation and extraction through CommunicationsService', async () => {
    const fetchMock = vi
      .fn()
      .mockResolvedValueOnce(
        new Response(JSON.stringify({
          language: 'es',
          confidence: 0.88,
          script: 'latin'
        }), {
          status: 200,
          headers: { 'Content-Type': 'application/json' }
        })
      )
      .mockResolvedValueOnce(
        new Response(JSON.stringify({
          translated: false,
          target: 'en',
          reason: 'no LLM configured'
        }), {
          status: 200,
          headers: { 'Content-Type': 'application/json' }
        })
      )
      .mockResolvedValueOnce(
        new Response(JSON.stringify({
          tasks: [{
            title: 'Reply by Friday',
            dueDate: 'Friday',
            assignee: null,
            priority: 'normal',
            source: 'heuristic'
          }]
        }), {
          status: 200,
          headers: { 'Content-Type': 'application/json' }
        })
      )
      .mockResolvedValueOnce(
        new Response(JSON.stringify({
          notes: [{
            title: 'Contract update',
            content: 'Contains legal review items',
            tags: ['legal'],
            source: 'heuristic'
          }]
        }), {
          status: 200,
          headers: { 'Content-Type': 'application/json' }
        })
      )
    vi.stubGlobal('fetch', fetchMock)

    const language = await detectMessageLanguage('msg-1')
    const translation = await translateMessage('msg-1', 'en')
    const tasks = await extractMessageTasks('msg-1')
    const notes = await extractMessageNotes('msg-1')

    expect(language.language).toBe('es')
    expect(translation.reason).toBe('no LLM configured')
    expect(tasks.tasks[0].title).toBe('Reply by Friday')
    expect(notes.notes[0].tags).toEqual(['legal'])
    expect(fetchMock.mock.calls[0][0]).toBe(
      'http://127.0.0.1:8080/makosh.communications.v1.CommunicationsService/DetectMessageLanguage'
    )
    expect(fetchMock.mock.calls[1][0]).toBe(
      'http://127.0.0.1:8080/makosh.communications.v1.CommunicationsService/TranslateMessage'
    )
    expect(fetchMock.mock.calls[2][0]).toBe(
      'http://127.0.0.1:8080/makosh.communications.v1.CommunicationsService/ExtractMessageTasks'
    )
    expect(fetchMock.mock.calls[3][0]).toBe(
      'http://127.0.0.1:8080/makosh.communications.v1.CommunicationsService/ExtractMessageNotes'
    )
  })

  it('routes analyze, explain and smart cc through CommunicationsService', async () => {
    const fetchMock = vi
      .fn()
      .mockResolvedValueOnce(
        new Response(JSON.stringify({
          messageId: 'msg-1',
          analyzed: true,
          category: 'follow_up',
          summary: 'Needs a reply',
          summaryContract: {
            keyPoints: ['Asks for updated contract'],
            actionItems: ['Reply this week'],
            risks: [],
            deadlines: ['2026-06-30'],
            eventCandidates: [],
            personaCandidates: [{ title: 'Alice', evidence: 'from Alice <alice@example.com>' }],
            organizationCandidates: [],
            documentCandidates: [],
            agreementCandidates: []
          },
          importanceScore: 81,
          workflowState: 'needs_action',
          source: 'local_heuristic',
          evidence: ['Contains request']
        }), {
          status: 200,
          headers: { 'Content-Type': 'application/json' }
        })
      )
      .mockResolvedValueOnce(
        new Response(JSON.stringify({
          reasons: ['Contains request']
        }), {
          status: 200,
          headers: { 'Content-Type': 'application/json' }
        })
      )
      .mockResolvedValueOnce(
        new Response(JSON.stringify({
          suggestions: ['sales@example.com']
        }), {
          status: 200,
          headers: { 'Content-Type': 'application/json' }
        })
      )
    vi.stubGlobal('fetch', fetchMock)

    const analyzed = await analyzeMessage('msg-1')
    const explained = await fetchMessageExplain('msg-1')
    const smartCc = await fetchMessageSmartCc('msg-1')

    expect(analyzed.summary_contract.action_items).toEqual(['Reply this week'])
    expect(analyzed.summary_contract.persona_candidates[0]).toEqual({
      title: 'Alice',
      evidence: 'from Alice <alice@example.com>'
    })
    expect(explained.reasons).toEqual(['Contains request'])
    expect(smartCc.suggestions).toEqual(['sales@example.com'])
    expect(fetchMock.mock.calls[0][0]).toBe(
      'http://127.0.0.1:8080/makosh.communications.v1.CommunicationsService/AnalyzeMessage'
    )
    expect(fetchMock.mock.calls[1][0]).toBe(
      'http://127.0.0.1:8080/makosh.communications.v1.CommunicationsService/GetMessageExplain'
    )
    expect(fetchMock.mock.calls[2][0]).toBe(
      'http://127.0.0.1:8080/makosh.communications.v1.CommunicationsService/GetMessageSmartCc'
    )
  })

  it('routes export, auth and signature through CommunicationsService', async () => {
    const fetchMock = vi
      .fn()
      .mockResolvedValueOnce(
        new Response(JSON.stringify({
          contentType: 'application/json',
          content: '{"message_id":"msg-1"}',
          filename: 'message_msg-1.json'
        }), {
          status: 200,
          headers: { 'Content-Type': 'application/json' }
        })
      )
      .mockResolvedValueOnce(
        new Response(JSON.stringify({
          auth: {
            spf: { result: 'pass', domain: 'alice@example.com' },
            rawHeaders: ['Received-SPF: pass']
          },
          risk: {
            hasSpf: true,
            spfPass: true,
            hasDkim: false,
            dkimPass: false,
            hasDmarc: false,
            dmarcPass: false,
            isSpoofed: false,
            riskSummary: 'Authentication checks passed'
          }
        }), {
          status: 200,
          headers: { 'Content-Type': 'application/json' }
        })
      )
      .mockResolvedValueOnce(
        new Response(JSON.stringify({
          hasSignature: false
        }), {
          status: 200,
          headers: { 'Content-Type': 'application/json' }
        })
      )
    vi.stubGlobal('fetch', fetchMock)

    const exported = await exportMessage('msg-1', 'json')
    const auth = await fetchMessageAuth('msg-1')
    const signature = await fetchMessageSignature('msg-1')

    expect(exported.filename).toBe('message_msg-1.json')
    expect(auth.auth.spf?.domain).toBe('alice@example.com')
    expect(signature.has_signature).toBe(false)
    expect(fetchMock.mock.calls[0][0]).toBe(
      'http://127.0.0.1:8080/makosh.communications.v1.CommunicationsService/GetMessageExport'
    )
    expect(fetchMock.mock.calls[1][0]).toBe(
      'http://127.0.0.1:8080/makosh.communications.v1.CommunicationsService/GetMessageAuth'
    )
    expect(fetchMock.mock.calls[2][0]).toBe(
      'http://127.0.0.1:8080/makosh.communications.v1.CommunicationsService/GetMessageSignature'
    )
  })

  it('routes ai reply and ai reply variants through CommunicationsService', async () => {
    const fetchMock = vi
      .fn()
      .mockResolvedValueOnce(
        new Response(JSON.stringify({
          subject: 'Re: Quarterly update',
          body: 'Generated reply',
          tone: 'business',
          language: 'en',
          generated: true
        }), {
          status: 200,
          headers: { 'Content-Type': 'application/json' }
        })
      )
      .mockResolvedValueOnce(
        new Response(JSON.stringify({
          variants: [
            {
              subject: 'Re: Quarterly update',
              body: 'Variant one',
              tone: 'professional',
              language: 'en',
              generated: true
            }
          ]
        }), {
          status: 200,
          headers: { 'Content-Type': 'application/json' }
        })
      )
    vi.stubGlobal('fetch', fetchMock)

    const reply = await generateAiReply('msg-1', { tone: 'business', language: 'en' })
    const variants = await generateAiReplyVariants('msg-1', {
      languages: ['en'],
      tones: ['professional']
    })

    expect(reply.body).toBe('Generated reply')
    expect(variants.variants[0].tone).toBe('professional')
    expect(fetchMock.mock.calls[0][0]).toBe(
      'http://127.0.0.1:8080/makosh.communications.v1.CommunicationsService/GenerateAiReply'
    )
    expect(fetchMock.mock.calls[1][0]).toBe(
      'http://127.0.0.1:8080/makosh.communications.v1.CommunicationsService/GenerateAiReplyVariants'
    )
  })

  it('routes local state and bulk message actions through CommunicationsService', async () => {
    const fetchMock = vi
      .fn()
      .mockResolvedValueOnce(
        new Response(JSON.stringify({
          messageId: 'msg-1',
          localState: 'trash',
          providerDeleted: false
        }), {
          status: 200,
          headers: { 'Content-Type': 'application/json' }
        })
      )
      .mockResolvedValueOnce(
        new Response(JSON.stringify({
          messageId: 'msg-1',
          localState: 'active'
        }), {
          status: 200,
          headers: { 'Content-Type': 'application/json' }
        })
      )
      .mockResolvedValueOnce(
        new Response(JSON.stringify({
          action: 'trash',
          requestedCount: 2,
          matchedCount: 2,
          updatedCount: 2,
          notFound: []
        }), {
          status: 200,
          headers: { 'Content-Type': 'application/json' }
        })
      )
    vi.stubGlobal('fetch', fetchMock)

    const trashed = await trashMessage('msg-1')
    const restored = await restoreMessage('msg-1')
    const bulk = await bulkMessageAction({
      action: 'trash',
      message_ids: ['msg-1', 'msg-2']
    })

    expect(trashed.local_state).toBe('trash')
    expect(restored.local_state).toBe('active')
    expect(bulk.updated_count).toBe(2)
    expect(fetchMock.mock.calls[0][0]).toBe(
      'http://127.0.0.1:8080/makosh.communications.v1.CommunicationsService/TrashMessage'
    )
    expect(fetchMock.mock.calls[1][0]).toBe(
      'http://127.0.0.1:8080/makosh.communications.v1.CommunicationsService/RestoreMessage'
    )
    expect(fetchMock.mock.calls[2][0]).toBe(
      'http://127.0.0.1:8080/makosh.communications.v1.CommunicationsService/BulkMessageAction'
    )
  })

  it('routes mark-read and provider-delete alias through CommunicationsService', async () => {
    const fetchMock = vi
      .fn()
      .mockResolvedValueOnce(
        new Response(JSON.stringify({
          messageId: 'msg-1',
          markedRead: true,
          workflowState: 'reviewed'
        }), {
          status: 200,
          headers: { 'Content-Type': 'application/json' }
        })
      )
      .mockResolvedValueOnce(
        new Response(JSON.stringify({
          messageId: 'msg-1',
          deleted: true,
          localState: 'trash',
          providerDeleted: false
        }), {
          status: 200,
          headers: { 'Content-Type': 'application/json' }
        })
      )
    vi.stubGlobal('fetch', fetchMock)

    const markedRead = await markMessageRead('msg-1')
    const deleted = await deleteMessageFromProvider('msg-1')

    expect(markedRead).toEqual({
      message_id: 'msg-1',
      marked_read: true,
      workflow_state: 'reviewed'
    })
    expect(deleted).toEqual({
      message_id: 'msg-1',
      local_state: 'trash',
      provider_deleted: false
    })
    expect(fetchMock.mock.calls[0][0]).toBe(
      'http://127.0.0.1:8080/makosh.communications.v1.CommunicationsService/MarkMessageRead'
    )
    expect(fetchMock.mock.calls[1][0]).toBe(
      'http://127.0.0.1:8080/makosh.communications.v1.CommunicationsService/DeleteMessageFromProvider'
    )
  })

  it('routes pin, important, mute, snooze and label commands through CommunicationsService', async () => {
    const fetchMock = vi
      .fn()
      .mockResolvedValueOnce(
        new Response(JSON.stringify({ messageId: 'msg-1', pinned: true }), {
          status: 200,
          headers: { 'Content-Type': 'application/json' }
        })
      )
      .mockResolvedValueOnce(
        new Response(JSON.stringify({ messageId: 'msg-1', important: true }), {
          status: 200,
          headers: { 'Content-Type': 'application/json' }
        })
      )
      .mockResolvedValueOnce(
        new Response(JSON.stringify({ messageId: 'msg-1', muted: true }), {
          status: 200,
          headers: { 'Content-Type': 'application/json' }
        })
      )
      .mockResolvedValueOnce(
        new Response(JSON.stringify({ snoozed: true }), {
          status: 200,
          headers: { 'Content-Type': 'application/json' }
        })
      )
      .mockResolvedValueOnce(
        new Response(JSON.stringify({ labeled: true }), {
          status: 200,
          headers: { 'Content-Type': 'application/json' }
        })
      )
    vi.stubGlobal('fetch', fetchMock)

    const pinned = await toggleMessagePin('msg-1')
    const important = await toggleMessageImportant('msg-1')
    const muted = await toggleMessageMute('msg-1')
    const snoozed = await snoozeMessage('msg-1', '2026-06-30T10:00:00Z')
    const labeled = await addMessageLabel('msg-1', 'follow-up')

    expect(pinned.pinned).toBe(true)
    expect(important.important).toBe(true)
    expect(muted.pinned).toBe(true)
    expect(snoozed).toEqual({ snoozed: true })
    expect(labeled).toEqual({ labeled: true })
    expect(fetchMock.mock.calls[0][0]).toBe(
      'http://127.0.0.1:8080/makosh.communications.v1.CommunicationsService/ToggleMessagePin'
    )
    expect(fetchMock.mock.calls[1][0]).toBe(
      'http://127.0.0.1:8080/makosh.communications.v1.CommunicationsService/ToggleMessageImportant'
    )
    expect(fetchMock.mock.calls[2][0]).toBe(
      'http://127.0.0.1:8080/makosh.communications.v1.CommunicationsService/ToggleMessageMute'
    )
    expect(fetchMock.mock.calls[3][0]).toBe(
      'http://127.0.0.1:8080/makosh.communications.v1.CommunicationsService/SnoozeMessage'
    )
    expect(fetchMock.mock.calls[4][0]).toBe(
      'http://127.0.0.1:8080/makosh.communications.v1.CommunicationsService/AddMessageLabel'
    )
  })

  it('fetches outbox items with account, status and cursor filters', async () => {
    const fetchMock = vi.fn().mockResolvedValue(
      new Response(JSON.stringify({
        items: [],
        page: { nextCursor: 'next-cursor', hasMore: true }
      }), {
        status: 200,
        headers: { 'Content-Type': 'application/json' }
      })
    )
    vi.stubGlobal('fetch', fetchMock)

    const response = await fetchOutboxItems('account-1', 'scheduled', 25, 'cursor:value')

    expect(response.next_cursor).toBe('next-cursor')
    expect(response.has_more).toBe(true)
    expect(fetchMock).toHaveBeenCalledOnce()
    const [url] = fetchMock.mock.calls[0]
    expect(url).toBe('http://127.0.0.1:8080/makosh.communications.v1.CommunicationsService/ListOutbox')
    expect(JSON.parse(decodeBody(fetchMock.mock.calls[0][1].body))).toMatchObject({
      accountId: 'account-1',
      status: 'scheduled',
      page: {
        limit: 25,
        cursor: 'cursor:value'
      }
    })
  })

  it('fetches drafts with account, status and cursor filters', async () => {
    const fetchMock = vi.fn().mockResolvedValue(
      new Response(JSON.stringify({
        items: [],
        page: { nextCursor: 'next-draft-cursor', hasMore: true }
      }), {
        status: 200,
        headers: { 'Content-Type': 'application/json' }
      })
    )
    vi.stubGlobal('fetch', fetchMock)

    const response = await fetchDrafts('account-1', 'draft', 25, 'cursor:value')

    expect(response.next_cursor).toBe('next-draft-cursor')
    expect(response.has_more).toBe(true)
    expect(fetchMock).toHaveBeenCalledOnce()
    const [url] = fetchMock.mock.calls[0]
    expect(url).toBe('http://127.0.0.1:8080/makosh.communications.v1.CommunicationsService/ListDrafts')
    expect(JSON.parse(decodeBody(fetchMock.mock.calls[0][1].body))).toMatchObject({
      accountId: 'account-1',
      status: 'draft',
      page: {
        limit: 25,
        cursor: 'cursor:value'
      }
    })
  })

  it('fetches subscriptions with account and cursor filters', async () => {
    const fetchMock = vi.fn().mockResolvedValue(
      new Response(JSON.stringify({ items: [], nextCursor: 'next-subscription-cursor', hasMore: true }), {
        status: 200,
        headers: { 'Content-Type': 'application/json' }
      })
    )
    vi.stubGlobal('fetch', fetchMock)

    const response = await fetchSubscriptions('account-1', 25, 'cursor:value')

    expect(response.next_cursor).toBe('next-subscription-cursor')
    expect(response.has_more).toBe(true)
    expect(fetchMock).toHaveBeenCalledOnce()
    const [url] = fetchMock.mock.calls[0]
    expect(url).toBe('http://127.0.0.1:8080/makosh.communications.v1.CommunicationsService/ListSubscriptions')
    expect(JSON.parse(decodeBody(fetchMock.mock.calls[0][1].body))).toMatchObject({
      accountId: 'account-1',
      limit: 25,
      cursor: 'cursor:value'
    })
  })

  it('fetches top senders with account and cursor filters', async () => {
    const fetchMock = vi.fn().mockResolvedValue(
      new Response(JSON.stringify({ items: [], nextCursor: 'next-sender-cursor', hasMore: true }), {
        status: 200,
        headers: { 'Content-Type': 'application/json' }
      })
    )
    vi.stubGlobal('fetch', fetchMock)

    const response = await fetchTopSenders('account-1', 25, 'cursor:value')

    expect(response.next_cursor).toBe('next-sender-cursor')
    expect(response.has_more).toBe(true)
    expect(fetchMock).toHaveBeenCalledOnce()
    const [url] = fetchMock.mock.calls[0]
    expect(url).toBe('http://127.0.0.1:8080/makosh.communications.v1.CommunicationsService/ListTopSenders')
    expect(JSON.parse(decodeBody(fetchMock.mock.calls[0][1].body))).toMatchObject({
      accountId: 'account-1',
      limit: 25,
      cursor: 'cursor:value'
    })
  })

  it('fetches mailbox health and blockers through CommunicationsService', async () => {
    const fetchMock = vi
      .fn()
      .mockResolvedValueOnce(
        new Response(JSON.stringify({
          item: {
            totalMessages: 15,
            unread: 2,
            needsAction: 1,
            waiting: 0,
            done: 8,
            archived: 3,
            spam: 1,
            important: 4,
            withAttachments: 5,
            averageImportance: 55.5,
            oldestMessageDays: 9
          }
        }), {
          status: 200,
          headers: { 'Content-Type': 'application/json' }
        })
      )
      .mockResolvedValueOnce(
        new Response(JSON.stringify({
          items: [{
            section: '§8',
            feature: 'Безопасность вложений',
            reason: 'Требует внешнего scanner backend',
            resolution: 'Интегрировать ClamAV'
          }]
        }), {
          status: 200,
          headers: { 'Content-Type': 'application/json' }
        })
      )
    vi.stubGlobal('fetch', fetchMock)

    const health = await fetchMailboxHealth('account-1')
    const blockers = await fetchCommunicationBlockers()

    expect(health.total_messages).toBe(15)
    expect(blockers[0].feature).toBe('Безопасность вложений')
    expect(fetchMock.mock.calls[0][0]).toBe(
      'http://127.0.0.1:8080/makosh.communications.v1.CommunicationsService/GetMailboxHealth'
    )
    expect(fetchMock.mock.calls[1][0]).toBe(
      'http://127.0.0.1:8080/makosh.communications.v1.CommunicationsService/ListCommunicationBlockers'
    )
  })

  it('fetches personas through CommunicationsService', async () => {
    const fetchMock = vi.fn().mockResolvedValue(
      new Response(JSON.stringify({
        items: [{
          personaId: 'persona-1',
          accountId: 'account-1',
          name: 'Owner',
          displayName: 'Owner Persona',
          signature: 'Regards',
          defaultLanguage: 'en',
          defaultTone: 'warm',
          isDefault: true,
          metadataJson: '{"role":"owner"}',
          createdAt: '2026-06-15T10:00:00Z',
          updatedAt: '2026-06-15T10:00:00Z'
        }]
      }), {
        status: 200,
        headers: { 'Content-Type': 'application/json' }
      })
    )
    vi.stubGlobal('fetch', fetchMock)

    const response = await fetchPersonas()

    expect(response.items[0].persona_id).toBe('persona-1')
    expect(response.items[0].metadata).toEqual({ role: 'owner' })
    expect(fetchMock.mock.calls[0][0]).toBe(
      'http://127.0.0.1:8080/makosh.communications.v1.CommunicationsService/ListCommunicationPersonas'
    )
  })

  it('runs workflow actions through CommunicationsService', async () => {
    const fetchMock = vi.fn().mockResolvedValue(
      new Response(JSON.stringify({
        commandId: 'cmd-1',
        eventId: 'workflow_action:cmd-1',
        action: 'archive',
        status: 'archived',
        target: {
          kind: 'message',
          id: 'msg-1'
        },
        provenance: {
          sourceKind: 'communication_message',
          sourceId: 'msg-1',
          confidence: null,
          evidence: ['explicit action']
        }
      }), {
        status: 200,
        headers: { 'Content-Type': 'application/json' }
      })
    )
    vi.stubGlobal('fetch', fetchMock)

    const response = await runWorkflowAction({
      command_id: 'cmd-1',
      action: 'archive',
      source: { kind: 'communication_message', id: 'msg-1' }
    })

    expect(response.status).toBe('archived')
    expect(fetchMock.mock.calls[0][0]).toBe(
      'http://127.0.0.1:8080/makosh.communications.v1.CommunicationsService/RunWorkflowAction'
    )
    expect(JSON.parse(decodeBody(fetchMock.mock.calls[0][1].body))).toEqual({
      commandId: 'cmd-1',
      action: 'archive',
      source: { kind: 'communication_message', id: 'msg-1' }
    })
  })

  it('fetches threads with account and cursor filters', async () => {
    const fetchMock = vi.fn().mockResolvedValue(
      new Response(JSON.stringify({ items: [], next_cursor: 'next-thread-cursor', has_more: true }), {
        status: 200,
        headers: { 'Content-Type': 'application/json' }
      })
    )
    vi.stubGlobal('fetch', fetchMock)

    const response = await fetchThreads('account-1', 25, 'cursor:value')

    expect(response.next_cursor).toBe('next-thread-cursor')
    expect(response.has_more).toBe(true)
    expect(fetchMock).toHaveBeenCalledOnce()
    const [url] = fetchMock.mock.calls[0]
    expect(url).toBe('http://127.0.0.1:8080/makosh.communications.v1.CommunicationsService/ListThreads')
    expect(JSON.parse(decodeBody(fetchMock.mock.calls[0][1].body))).toMatchObject({
      accountId: 'account-1',
      limit: 25,
      cursor: 'cursor:value'
    })
  })

  it('preserves provider record ids in thread message responses for inline replies', async () => {
    const fetchMock = vi.fn().mockResolvedValue(
      new Response(JSON.stringify({
        items: [{
          message_id: 'msg-1',
          provider_record_id: 'provider-msg-1',
          account_id: 'account-1',
          subject: 'Quarterly update',
          sender: 'Ada <ada@example.com>',
          sender_display_name: 'Ada',
          body_text: 'Thread body',
          occurred_at: null,
          projected_at: '2026-06-15T10:00:00Z',
          workflow_state: 'new',
          importance_score: null,
          ai_category: null,
          ai_summary: null,
          delivery_state: 'received',
          attachment_count: 0
        }]
      }), {
        status: 200,
        headers: { 'Content-Type': 'application/json' }
      })
    )
    vi.stubGlobal('fetch', fetchMock)

    const response = await fetchThreadMessages('account-1', 'Quarterly update', 25)

    expect(response.items[0].provider_record_id).toBe('provider-msg-1')
    expect(fetchMock).toHaveBeenCalledOnce()
    const [url] = fetchMock.mock.calls[0]
    expect(url).toBe('http://127.0.0.1:8080/makosh.communications.v1.CommunicationsService/ListThreadMessages')
    expect(JSON.parse(decodeBody(fetchMock.mock.calls[0][1].body))).toMatchObject({
      accountId: 'account-1',
      subject: 'Quarterly update',
      limit: 25
    })
  })

  it('undoes an outbox item through the protected communications API', async () => {
    const fetchMock = vi.fn().mockResolvedValue(
      new Response(JSON.stringify({
        item: {
          outboxId: 'outbox-1',
          accountId: 'account-1',
          toRecipients: ['bob@example.com'],
          ccRecipients: [],
          bccRecipients: [],
          subject: 'Queued',
          bodyText: 'Queued body',
          status: 'canceled',
          sendAttempts: 1,
          metadataJson: '{}',
          createdAt: '2026-06-23T10:00:00Z',
          updatedAt: '2026-06-23T10:05:00Z'
        }
      }), {
        status: 200,
        headers: { 'Content-Type': 'application/json' }
      })
    )
    vi.stubGlobal('fetch', fetchMock)

    const response = await undoOutboxItem('outbox-1')

    expect(response.outbox_id).toBe('outbox-1')
    expect(response.status).toBe('canceled')
    expect(fetchMock).toHaveBeenCalledOnce()
    const [url, init] = fetchMock.mock.calls[0]
    expect(url).toBe('http://127.0.0.1:8080/makosh.communications.v1.CommunicationsService/UndoOutboxItem')
    expect(init.method).toBe('POST')
    expect(JSON.parse(decodeBody(init.body))).toMatchObject({
      outboxId: 'outbox-1'
    })
  })

  it('saves and deletes drafts through the provider-neutral CommunicationsService endpoint', async () => {
    const fetchMock = vi
      .fn()
      .mockResolvedValueOnce(
        new Response(JSON.stringify({
          item: {
            draftId: 'draft-1',
            accountId: 'account-1',
            toRecipients: ['bob@example.com'],
            ccRecipients: [],
            bccRecipients: [],
            subject: 'Draft',
            bodyText: 'Draft body',
            attachmentIds: ['attachment-1'],
            attachments: [{
              attachmentId: 'attachment-1',
              filename: 'report.pdf',
              contentType: 'application/pdf',
              sizeBytes: '1024',
              scanStatus: 'clean',
              scanEngine: 'clamav_clamd',
              scanCheckedAt: '2026-06-23T09:59:00Z',
              scanSummary: 'ClamAV found no known malware'
            }],
            status: 'draft',
            sendAttempts: 0,
            metadataJson: '{"compose_mode":"compose"}',
            createdAt: '2026-06-23T10:00:00Z',
            updatedAt: '2026-06-23T10:00:00Z'
          }
        }), {
          status: 200,
          headers: { 'Content-Type': 'application/json' }
        })
      )
      .mockResolvedValueOnce(
        new Response(JSON.stringify({ deleted: true }), {
          status: 200,
          headers: { 'Content-Type': 'application/json' }
        })
      )
    vi.stubGlobal('fetch', fetchMock)

    const draft = await createDraft({
      draft_id: 'draft-1',
      account_id: 'account-1',
      to_recipients: ['bob@example.com'],
      subject: 'Draft',
      body_text: 'Draft body',
      attachment_ids: ['attachment-1'],
      metadata: { compose_mode: 'compose' }
    })
    const deletion = await deleteDraft('draft-1')

    expect(draft.draft_id).toBe('draft-1')
    expect(draft.attachment_ids).toEqual(['attachment-1'])
    expect(draft.attachments).toEqual([{
      attachment_id: 'attachment-1',
      filename: 'report.pdf',
      content_type: 'application/pdf',
      size_bytes: 1024,
      scan_status: 'clean',
      scan_engine: 'clamav_clamd',
      scan_checked_at: '2026-06-23T09:59:00Z',
      scan_summary: 'ClamAV found no known malware'
    }])
    expect(deletion.deleted).toBe(true)
    expect(fetchMock.mock.calls[0][0]).toBe(
      'http://127.0.0.1:8080/makosh.communications.v1.CommunicationsService/CreateDraft'
    )
    expect(JSON.parse(decodeBody(fetchMock.mock.calls[0][1].body))).toMatchObject({
      draftId: 'draft-1',
      accountId: 'account-1',
      toRecipients: ['bob@example.com'],
      subject: 'Draft',
      bodyText: 'Draft body',
      attachmentIds: ['attachment-1'],
      replaceAttachments: true,
      metadataJson: '{"compose_mode":"compose"}'
    })
    expect(fetchMock.mock.calls[1][0]).toBe(
      'http://127.0.0.1:8080/makosh.communications.v1.CommunicationsService/DeleteDraft'
    )
    expect(JSON.parse(decodeBody(fetchMock.mock.calls[1][1].body))).toMatchObject({
      draftId: 'draft-1'
    })
  })

  it('sends mail through the provider-neutral CommunicationsService ConnectRPC endpoint', async () => {
    const fetchMock = vi.fn().mockResolvedValue(
      new Response(JSON.stringify({
        item: {
          outboxId: 'outbox-1',
          accountId: 'account-1',
          toRecipients: ['bob@example.com'],
          ccRecipients: [],
          bccRecipients: [],
          subject: 'Connect send',
          bodyText: 'Queued body',
          status: 'queued',
          sendAttempts: 0,
          metadataJson: '{}',
          createdAt: '2026-06-23T10:00:00Z',
          updatedAt: '2026-06-23T10:00:00Z'
        },
        messageId: 'outbox-1',
        outboxId: 'outbox-1',
        accepted: ['bob@example.com'],
        acceptedRecipients: ['bob@example.com'],
        transport: 'outbox',
        status: 'queued',
        scheduledSendAt: null,
        undoDeadlineAt: null,
        failureReason: null
      }), {
        status: 200,
        headers: { 'Content-Type': 'application/json' }
      })
    )
    vi.stubGlobal('fetch', fetchMock)

    const response = await sendEmail({
      account_id: 'account-1',
      to: ['bob@example.com'],
      subject: 'Connect send',
      body_text: 'Queued body',
      draft_id: 'draft-1',
      undo_send_seconds: 300,
      confirmed_provider_write: true
    })

    expect(response.outbox_id).toBe('outbox-1')
    expect(response.transport).toBe('outbox')
    const [url, init] = fetchMock.mock.calls[0]
    expect(url).toBe('http://127.0.0.1:8080/makosh.communications.v1.CommunicationsService/SendMessage')
    expect(init.method).toBe('POST')
    expect(JSON.parse(decodeBody(init.body))).toMatchObject({
      accountId: 'account-1',
      toRecipients: ['bob@example.com'],
      subject: 'Connect send',
      bodyText: 'Queued body',
      draftId: 'draft-1',
      undoSendSeconds: '300',
      confirmedProviderWrite: true
    })
  })

  it('saves, lists, and renders durable rich templates through the communications API', async () => {
    const fetchMock = vi
      .fn()
      .mockResolvedValueOnce(
        new Response(JSON.stringify({
          saved: true,
          template: {
            templateId: 'tpl-1',
            name: 'Intro',
            subjectTemplate: 'Hello {{name}}',
            bodyTemplate: '<p>Welcome {{name}}</p>',
            variables: ['name'],
            placeholderVariables: ['name'],
            undeclaredVariables: [],
            unusedVariables: [],
            malformedPlaceholders: [],
            language: null,
            createdAt: '2026-06-15T10:00:00Z',
            updatedAt: '2026-06-15T10:00:00Z'
          }
        }), {
          status: 200,
          headers: { 'Content-Type': 'application/json' }
        })
      )
      .mockResolvedValueOnce(
        new Response(JSON.stringify({
          saved: true,
          template: {
            templateId: 'tpl-1',
            name: 'Intro',
            subjectTemplate: 'Hello {{name}}',
            bodyTemplate: '<p>Welcome {{name}}</p>',
            variables: ['name'],
            placeholderVariables: ['name'],
            undeclaredVariables: [],
            unusedVariables: [],
            malformedPlaceholders: [],
            language: null,
            createdAt: '2026-06-15T10:00:00Z',
            updatedAt: '2026-06-15T10:00:00Z'
          }
        }), {
          status: 200,
          headers: { 'Content-Type': 'application/json' }
        })
      )
      .mockResolvedValueOnce(
        new Response(JSON.stringify({
          templates: [{
            templateId: 'tpl-1',
            name: 'Intro',
            subjectTemplate: 'Hello {{name}}',
            bodyTemplate: '<p>Welcome {{name}}</p>',
            variables: ['name'],
            placeholderVariables: ['name'],
            undeclaredVariables: [],
            unusedVariables: [],
            malformedPlaceholders: [],
            language: null,
            createdAt: '2026-06-15T10:00:00Z',
            updatedAt: '2026-06-15T10:00:00Z'
          }]
        }), {
          status: 200,
          headers: { 'Content-Type': 'application/json' }
        })
      )
      .mockResolvedValueOnce(
        new Response(JSON.stringify({
          templateId: 'tpl-1',
          variables: { name: 'Alex' },
          rendered: {
            subject: 'Hello Alex',
            body: '<p>Welcome</p>',
            missingVariables: [],
            unresolvedVariables: [],
            malformedPlaceholders: []
          }
        }), {
          status: 200,
          headers: { 'Content-Type': 'application/json' }
        })
      )
      .mockResolvedValueOnce(
        new Response(JSON.stringify({
          templateId: 'tpl-1',
          rowCount: 2,
          readyCount: 1,
          blockedCount: 1,
          items: [
            {
              rowId: 'r1',
              ready: true,
              rendered: {
                subject: 'Hello Alex',
                body: '<p>Welcome</p>',
                missingVariables: [],
                unresolvedVariables: [],
                malformedPlaceholders: []
              }
            },
            {
              rowId: 'r2',
              ready: false,
              rendered: {
                subject: 'Hello {{ name }}',
                body: '<p>{{ name }}</p>',
                missingVariables: ['name'],
                unresolvedVariables: ['name'],
                malformedPlaceholders: []
              }
            }
          ]
        }), {
          status: 200,
          headers: { 'Content-Type': 'application/json' }
        })
      )
      .mockResolvedValueOnce(
        new Response(JSON.stringify({ templateId: 'tpl-1', deleted: true }), {
          status: 200,
          headers: { 'Content-Type': 'application/json' }
        })
      )
    vi.stubGlobal('fetch', fetchMock)

    const saved = await saveRichTemplate({
      name: 'Intro',
      subject_template: 'Hello {{name}}',
      body_template: '<p>Welcome {{name}}</p>',
      variables: ['name'],
      language: null
    })
    await saveRichTemplate({
      template_id: 'tpl-1',
      name: 'Intro',
      subject_template: 'Updated {{name}}',
      body_template: '<p>Updated {{name}}</p>',
      variables: ['name'],
      language: null
    })
    const templates = await fetchRichTemplates()
    const rendered = await renderRichTemplate({
      template_id: 'tpl-1',
      variables: { name: 'Alex' }
    })
    const preview = await previewRichTemplateMailMerge({
      template_id: 'tpl-1',
      rows: [
        { row_id: 'r1', variables: { name: 'Alex' } },
        { row_id: 'r2', variables: {} }
      ]
    })
    await deleteRichTemplate('tpl-1')

    expect(saved.template.placeholder_variables).toEqual(['name'])
    expect(saved.template.undeclared_variables).toEqual([])
    expect(templates.templates[0].malformed_placeholders).toEqual([])
    expect(rendered.rendered.subject).toBe('Hello Alex')
    expect(rendered.rendered.missing_variables).toEqual([])
    expect(rendered.rendered.unresolved_variables).toEqual([])
    expect(rendered.rendered.malformed_placeholders).toEqual([])
    expect(preview.ready_count).toBe(1)
    expect(preview.blocked_count).toBe(1)
    expect(preview.items[1].rendered.missing_variables).toEqual(['name'])
    expect(fetchMock).toHaveBeenCalledTimes(6)
    expect(fetchMock.mock.calls[0][0]).toBe(
      'http://127.0.0.1:8080/makosh.communications.v1.CommunicationsService/UpsertRichTemplate'
    )
    expect(fetchMock.mock.calls[0][1].method).toBe('POST')
    expect(JSON.parse(decodeBody(fetchMock.mock.calls[0][1].body))).toEqual({
      name: 'Intro',
      subjectTemplate: 'Hello {{name}}',
      bodyTemplate: '<p>Welcome {{name}}</p>',
      variables: ['name'],
      language: ''
    })
    expect(JSON.parse(decodeBody(fetchMock.mock.calls[1][1].body))).toEqual({
      templateId: 'tpl-1',
      name: 'Intro',
      subjectTemplate: 'Updated {{name}}',
      bodyTemplate: '<p>Updated {{name}}</p>',
      variables: ['name'],
      language: ''
    })
    expect(fetchMock.mock.calls[2][0]).toBe(
      'http://127.0.0.1:8080/makosh.communications.v1.CommunicationsService/ListRichTemplates'
    )
    const [url, init] = fetchMock.mock.calls[3]
    expect(url).toBe(
      'http://127.0.0.1:8080/makosh.communications.v1.CommunicationsService/RenderRichTemplate'
    )
    expect(init.method).toBe('POST')
    expect(JSON.parse(decodeBody(init.body))).toEqual({
      templateId: 'tpl-1',
      variables: { name: 'Alex' }
    })
    const [previewUrl, previewInit] = fetchMock.mock.calls[4]
    expect(previewUrl).toBe(
      'http://127.0.0.1:8080/makosh.communications.v1.CommunicationsService/PreviewRichTemplateMailMerge'
    )
    expect(previewInit.method).toBe('POST')
    expect(JSON.parse(decodeBody(previewInit.body))).toEqual({
      templateId: 'tpl-1',
      rows: [
        { rowId: 'r1', variables: { name: 'Alex' } },
        { rowId: 'r2', variables: {} }
      ]
    })
    expect(fetchMock.mock.calls[5][0]).toBe(
      'http://127.0.0.1:8080/makosh.communications.v1.CommunicationsService/DeleteRichTemplate'
    )
    expect(fetchMock.mock.calls[5][1].method).toBe('POST')
  })

  it('posts bounded message bulk actions to the communications API', async () => {
    const fetchMock = vi.fn().mockResolvedValue(
      new Response(JSON.stringify({
        action: 'trash',
        requestedCount: 2,
        matched_count: 2,
        updated_count: 2,
        not_found: []
      }), {
        status: 200,
        headers: { 'Content-Type': 'application/json' }
      })
    )
    vi.stubGlobal('fetch', fetchMock)

    await bulkMessageAction({
      action: 'trash',
      message_ids: ['msg-1', 'msg-2']
    })

    expect(fetchMock).toHaveBeenCalledOnce()
    const [url, init] = fetchMock.mock.calls[0]
    expect(url).toBe(
      'http://127.0.0.1:8080/makosh.communications.v1.CommunicationsService/BulkMessageAction'
    )
    expect(init.method).toBe('POST')
    expect(JSON.parse(decodeBody(init.body))).toEqual({
      action: 'trash',
      messageIds: ['msg-1', 'msg-2']
    })
  })

  it('manages saved searches through the communications API', async () => {
    const fetchMock = vi
      .fn()
      .mockResolvedValueOnce(
        new Response(JSON.stringify({ items: [], page: { nextCursor: 'next-saved-search-cursor', hasMore: true } }), {
          status: 200,
          headers: { 'Content-Type': 'application/json' }
        })
      )
      .mockResolvedValueOnce(
        new Response(JSON.stringify({ item: { savedSearchId: 'mail_saved_search:1', name: 'Invoices', query: 'invoice', localState: 'active', isSmartFolder: true, sortOrder: 0, messageCount: 0, createdAt: '2026-06-23T10:00:00Z', updatedAt: '2026-06-23T10:00:00Z' } }), {
          status: 200,
          headers: { 'Content-Type': 'application/json' }
        })
      )
      .mockResolvedValueOnce(
        new Response(JSON.stringify({ item: { savedSearchId: 'mail_saved_search:1', name: 'Waiting invoices', query: 'invoice', localState: 'active', isSmartFolder: true, sortOrder: 0, messageCount: 0, createdAt: '2026-06-23T10:00:00Z', updatedAt: '2026-06-23T10:05:00Z' } }), {
          status: 200,
          headers: { 'Content-Type': 'application/json' }
        })
      )
      .mockResolvedValueOnce(
        new Response(JSON.stringify({ deleted: true }), {
          status: 200,
          headers: { 'Content-Type': 'application/json' }
        })
      )
    vi.stubGlobal('fetch', fetchMock)

    const savedSearches = await fetchSavedSearches(true, 'account-1', 25, 'cursor:value')
    await createSavedSearch({
      name: 'Invoices',
      query: 'invoice',
      workflow_state: 'needs_action',
      local_state: 'active',
      is_smart_folder: true
    })
    await updateSavedSearch('mail_saved_search:1', {
      name: 'Waiting invoices',
      workflow_state: 'waiting'
    })
    await deleteSavedSearch('mail_saved_search:1')

    expect(fetchMock).toHaveBeenCalledTimes(4)
    expect(savedSearches.next_cursor).toBe('next-saved-search-cursor')
    expect(savedSearches.has_more).toBe(true)
    expect(fetchMock.mock.calls[0][0]).toBe(
      'http://127.0.0.1:8080/makosh.communications.v1.CommunicationsService/ListSavedSearches'
    )
    expect(JSON.parse(decodeBody(fetchMock.mock.calls[0][1].body))).toMatchObject({
      accountId: 'account-1',
      smartFolder: true,
      page: {
        limit: 25,
        cursor: 'cursor:value'
      }
    })
    expect(fetchMock.mock.calls[1][0]).toBe(
      'http://127.0.0.1:8080/makosh.communications.v1.CommunicationsService/CreateSavedSearch'
    )
    expect(fetchMock.mock.calls[1][1].method).toBe('POST')
    expect(JSON.parse(decodeBody(fetchMock.mock.calls[1][1].body))).toEqual({
      name: 'Invoices',
      query: 'invoice',
      workflowState: 'needs_action',
      localState: 'active',
      isSmartFolder: true
    })
    expect(fetchMock.mock.calls[2][0]).toBe(
      'http://127.0.0.1:8080/makosh.communications.v1.CommunicationsService/UpdateSavedSearch'
    )
    expect(fetchMock.mock.calls[2][1].method).toBe('POST')
    expect(JSON.parse(decodeBody(fetchMock.mock.calls[2][1].body))).toMatchObject({
      savedSearchId: 'mail_saved_search:1',
      name: 'Waiting invoices',
      workflowState: 'waiting'
    })
    expect(fetchMock.mock.calls[3][0]).toBe(
      'http://127.0.0.1:8080/makosh.communications.v1.CommunicationsService/DeleteSavedSearch'
    )
    expect(fetchMock.mock.calls[3][1].method).toBe('POST')
    expect(JSON.parse(decodeBody(fetchMock.mock.calls[3][1].body))).toMatchObject({
      savedSearchId: 'mail_saved_search:1'
    })
  })

})

function decodeBody(body: BodyInit | null | undefined): string {
  if (typeof body === 'string') {
    return body
  }
  if (body instanceof Uint8Array) {
    return new TextDecoder().decode(body)
  }
  return ''
}
