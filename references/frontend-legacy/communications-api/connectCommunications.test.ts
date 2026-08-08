// Historical pre-clean-room transport specification. Not part of the active test suite.
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { ApiClient } from '../../../platform/api/ApiClient'
import { resetCommunicationsConnectClientForTests } from '../../../platform/connect/communicationsClient'
import {
  analyzeMessageConnect,
  addMessageLabelConnect,
  bulkMessageActionConnect,
  copyMessageToFolderConnect,
  createCommunicationDraftConnect,
  createCommunicationFolderConnect,
  createCommunicationSavedSearchConnect,
  deleteMessageFromProviderConnect,
  detectMessageLanguageConnect,
  deleteCommunicationFolderConnect,
  deleteCommunicationSavedSearchConnect,
  deleteCommunicationDraftConnect,
  extractMessageNotesConnect,
  extractMessageTasksConnect,
  fetchCommunicationDraftsConnect,
  fetchCommunicationBlockersConnect,
  fetchCommunicationPersonasConnect,
  fetchCommunicationFoldersConnect,
  fetchCommunicationMessageConnect,
  fetchCommunicationMessagesConnect,
  fetchCommunicationOutboxConnect,
  fetchCommunicationSavedSearchesConnect,
  fetchMessageAuthConnect,
  fetchMessageExplainConnect,
  fetchMailboxHealthConnect,
  fetchMessageSignatureConnect,
  generateAiReplyConnect,
  generateAiReplyVariantsConnect,
  markMessageReadConnect,
  fetchMessageSmartCcConnect,
  fetchMessageStateCountsConnect,
  fetchSubscriptionsConnect,
  fetchFolderMessagesConnect,
  fetchCommunicationThreadMessagesConnect,
  fetchCommunicationThreadsConnect,
  fetchRichTemplatesConnect,
  fetchTopSendersConnect,
  moveMessageToFolderConnect,
  redirectMessageConnect,
  removeMessageLabelConnect,
  restoreMessageConnect,
  searchMessagesConnect,
  saveRichTemplateConnect,
  sendCommunicationConnect,
  snoozeMessageConnect,
  trashMessageConnect,
  toggleMessageImportantConnect,
  toggleMessageMuteConnect,
  toggleMessagePinConnect,
  translateMessageConnect,
  transitionMessageWorkflowStateConnect,
  translateCommunicationThreadConnect,
  updateCommunicationFolderConnect,
  updateCommunicationSavedSearchConnect,
  undoCommunicationOutboxItemConnect,
  exportMessageConnect,
  deleteRichTemplateConnect,
  previewRichTemplateMailMergeConnect,
  runWorkflowActionConnect,
  renderRichTemplateConnect
} from './connectCommunications'

describe('communications ConnectRPC API', () => {
  beforeEach(() => {
    ApiClient.resetForTests()
    ApiClient.init('http://127.0.0.1:8080', 'test-secret')
    resetCommunicationsConnectClientForTests()
  })

  afterEach(() => {
    vi.unstubAllGlobals()
    resetCommunicationsConnectClientForTests()
    ApiClient.resetForTests()
  })

  it('lists communications messages through the protected ConnectRPC client', async () => {
    const fetchMock = vi.fn().mockResolvedValue(
      new Response(JSON.stringify({
        items: [{
          messageId: 'msg-1',
          rawRecordId: 'raw-1',
          observationId: 'obs-1',
          accountId: 'account-1',
          providerRecordId: 'provider-1',
          subject: 'Quarterly update',
          sender: 'Ada <ada@example.com>',
          recipients: ['bob@example.com'],
          bodyText: 'Long body text for preview',
          occurredAt: '2026-06-23T10:00:00Z',
          projectedAt: '2026-06-23T10:01:00Z',
          channelKind: 'email',
          conversationId: 'thread-1',
          senderDisplayName: 'Ada',
          deliveryState: 'received',
          messageMetadataJson: '{"tag":"important"}',
          workflowState: 'needs_action',
          importanceScore: 91,
          aiCategory: 'follow_up',
          aiSummary: 'Needs a reply',
          aiSummaryGeneratedAt: '2026-06-23T10:02:00Z',
          localState: 'active',
          localStateChangedAt: '2026-06-23T10:03:00Z',
          attachmentCount: 2
        }],
        nextCursor: 'next-cursor',
        hasMore: true
      }), {
        status: 200,
        headers: { 'Content-Type': 'application/json' }
      })
    )
    vi.stubGlobal('fetch', fetchMock)

    const response = await fetchCommunicationMessagesConnect({
      account_id: 'account-1',
      workflow_state: 'needs_action',
      limit: 10
    })

    expect(response.items[0]).toMatchObject({
      message_id: 'msg-1',
      raw_record_id: 'raw-1',
      observation_id: 'obs-1',
      body_text_preview: 'Long body text for preview',
      workflow_state: 'needs_action',
      attachment_count: 2
    })
    expect(response.next_cursor).toBe('next-cursor')
    expect(response.has_more).toBe(true)
    expect(fetchMock).toHaveBeenCalledOnce()
    const [url, options] = fetchMock.mock.calls[0]
    expect(url).toBe('http://127.0.0.1:8080/makosh.communications.v1.CommunicationsService/ListMessages')
    expect(options.method).toBe('POST')
    expect(new Headers(options.headers).get('X-Макошь-Secret')).toBe('test-secret')
    expect(JSON.parse(decodeBody(options.body))).toMatchObject({
      accountId: 'account-1',
      workflowState: 'needs_action',
      limit: 10
    })
  })

  it('maps workflow transitions, state counts and search through CommunicationsService', async () => {
    const fetchMock = vi
      .fn()
      .mockResolvedValueOnce(
        new Response(JSON.stringify({
          messageId: 'msg-1',
          workflowState: 'done',
          previousState: 'needs_action'
        }), {
          status: 200,
          headers: { 'Content-Type': 'application/json' }
        })
      )
      .mockResolvedValueOnce(
        new Response(JSON.stringify({
          counts: [
            { state: 'new', count: 2 },
            { state: 'done', count: 5 }
          ]
        }), {
          status: 200,
          headers: { 'Content-Type': 'application/json' }
        })
      )
      .mockResolvedValueOnce(
        new Response(JSON.stringify({
          results: [
            {
              objectId: 'msg-1',
              objectKind: 'communication_message',
              title: '[Ada] Quarterly update'
            }
          ]
        }), {
          status: 200,
          headers: { 'Content-Type': 'application/json' }
        })
      )
    vi.stubGlobal('fetch', fetchMock)

    const transition = await transitionMessageWorkflowStateConnect('msg-1', 'done')
    const counts = await fetchMessageStateCountsConnect('account-1', 'active')
    const search = await searchMessagesConnect('quarterly', 15)

    expect(transition).toEqual({
      message_id: 'msg-1',
      workflow_state: 'done',
      previous_state: 'needs_action'
    })
    expect(counts.counts).toEqual([
      { state: 'new', count: 2 },
      { state: 'done', count: 5 }
    ])
    expect(search.results[0]).toEqual({
      object_id: 'msg-1',
      object_kind: 'communication_message',
      title: '[Ada] Quarterly update'
    })
    expect(fetchMock.mock.calls[0][0]).toBe(
      'http://127.0.0.1:8080/makosh.communications.v1.CommunicationsService/TransitionMessageWorkflowState'
    )
    expect(fetchMock.mock.calls[1][0]).toBe(
      'http://127.0.0.1:8080/makosh.communications.v1.CommunicationsService/ListMessageWorkflowStateCounts'
    )
    expect(fetchMock.mock.calls[2][0]).toBe(
      'http://127.0.0.1:8080/makosh.communications.v1.CommunicationsService/SearchMessages'
    )
    expect(JSON.parse(decodeBody(fetchMock.mock.calls[0][1].body))).toMatchObject({
      messageId: 'msg-1',
      workflowState: 'done'
    })
    expect(JSON.parse(decodeBody(fetchMock.mock.calls[1][1].body))).toMatchObject({
      accountId: 'account-1',
      localState: 'active'
    })
    expect(JSON.parse(decodeBody(fetchMock.mock.calls[2][1].body))).toMatchObject({
      query: 'quarterly',
      limit: 15
    })
  })

  it('maps subscriptions, mailbox health, top senders and blockers through CommunicationsService', async () => {
    const fetchMock = vi
      .fn()
      .mockResolvedValueOnce(
        new Response(JSON.stringify({
          items: [{
            sender: 'news@example.com',
            messageId: 'ignored',
            messageCount: 7,
            firstSeen: '2026-06-01T00:00:00Z',
            lastSeen: '2026-06-23T00:00:00Z',
            isNewsletter: true,
            hasUnsubscribe: true
          }],
          nextCursor: 'sub-cursor',
          hasMore: true
        }), {
          status: 200,
          headers: { 'Content-Type': 'application/json' }
        })
      )
      .mockResolvedValueOnce(
        new Response(JSON.stringify({
          item: {
            totalMessages: 42,
            unread: 5,
            needsAction: 3,
            waiting: 2,
            done: 20,
            archived: 10,
            spam: 2,
            important: 6,
            withAttachments: 9,
            averageImportance: 51.5,
            oldestMessageDays: 14.25
          }
        }), {
          status: 200,
          headers: { 'Content-Type': 'application/json' }
        })
      )
      .mockResolvedValueOnce(
        new Response(JSON.stringify({
          items: [{
            sender: 'ada@example.com',
            messageCount: 11,
            avgImportance: 77.5,
            lastMessageDays: 1.5
          }],
          nextCursor: 'sender-cursor',
          hasMore: false
        }), {
          status: 200,
          headers: { 'Content-Type': 'application/json' }
        })
      )
      .mockResolvedValueOnce(
        new Response(JSON.stringify({
          items: [{
            section: '§16-17',
            feature: 'Outbox tracking',
            reason: 'Needs provider callback wiring',
            resolution: 'Connect callback/runtime ingestion'
          }]
        }), {
          status: 200,
          headers: { 'Content-Type': 'application/json' }
        })
      )
    vi.stubGlobal('fetch', fetchMock)

    const subscriptions = await fetchSubscriptionsConnect('account-1', 25, 'cursor:value')
    const health = await fetchMailboxHealthConnect('account-1')
    const senders = await fetchTopSendersConnect('account-1', 10, 'sender:cursor')
    const blockers = await fetchCommunicationBlockersConnect()

    expect(subscriptions).toEqual({
      items: [{
        sender: 'news@example.com',
        message_count: 7,
        first_seen: '2026-06-01T00:00:00Z',
        last_seen: '2026-06-23T00:00:00Z',
        is_newsletter: true,
        has_unsubscribe: true
      }],
      next_cursor: 'sub-cursor',
      has_more: true
    })
    expect(health.total_messages).toBe(42)
    expect(health.oldest_message_days).toBe(14.25)
    expect(senders.items[0]).toEqual({
      sender: 'ada@example.com',
      message_count: 11,
      avg_importance: 77.5,
      last_message_days: 1.5
    })
    expect(blockers).toEqual([{
      section: '§16-17',
      feature: 'Outbox tracking',
      reason: 'Needs provider callback wiring',
      resolution: 'Connect callback/runtime ingestion'
    }])
    expect(fetchMock.mock.calls[0][0]).toBe(
      'http://127.0.0.1:8080/makosh.communications.v1.CommunicationsService/ListSubscriptions'
    )
    expect(fetchMock.mock.calls[1][0]).toBe(
      'http://127.0.0.1:8080/makosh.communications.v1.CommunicationsService/GetMailboxHealth'
    )
    expect(fetchMock.mock.calls[2][0]).toBe(
      'http://127.0.0.1:8080/makosh.communications.v1.CommunicationsService/ListTopSenders'
    )
    expect(fetchMock.mock.calls[3][0]).toBe(
      'http://127.0.0.1:8080/makosh.communications.v1.CommunicationsService/ListCommunicationBlockers'
    )
  })

  it('maps personas and rich templates through CommunicationsService', async () => {
    const fetchMock = vi
      .fn()
      .mockResolvedValueOnce(
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
            createdAt: '2026-06-23T00:00:00Z',
            updatedAt: '2026-06-23T00:00:00Z'
          }]
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
            language: 'en',
            createdAt: '2026-06-23T00:00:00Z',
            updatedAt: '2026-06-23T00:00:00Z'
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
            language: 'en',
            createdAt: '2026-06-23T00:00:00Z',
            updatedAt: '2026-06-23T00:00:00Z'
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
            body: '<p>Welcome Alex</p>',
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
          items: [{
            rowId: 'r1',
            ready: true,
            rendered: {
              subject: 'Hello Alex',
              body: '<p>Welcome Alex</p>',
              missingVariables: [],
              unresolvedVariables: [],
              malformedPlaceholders: []
            }
          }]
        }), {
          status: 200,
          headers: { 'Content-Type': 'application/json' }
        })
      )
      .mockResolvedValueOnce(
        new Response(JSON.stringify({
          templateId: 'tpl-1',
          deleted: true
        }), {
          status: 200,
          headers: { 'Content-Type': 'application/json' }
        })
      )
    vi.stubGlobal('fetch', fetchMock)

    const personas = await fetchCommunicationPersonasConnect()
    const saved = await saveRichTemplateConnect({
      name: 'Intro',
      subject_template: 'Hello {{name}}',
      body_template: '<p>Welcome {{name}}</p>',
      variables: ['name'],
      language: 'en'
    })
    const templates = await fetchRichTemplatesConnect()
    const rendered = await renderRichTemplateConnect({
      template_id: 'tpl-1',
      variables: { name: 'Alex' }
    })
    const preview = await previewRichTemplateMailMergeConnect({
      template_id: 'tpl-1',
      rows: [{ row_id: 'r1', variables: { name: 'Alex' } }, { row_id: 'r2', variables: {} }]
    })
    const deleted = await deleteRichTemplateConnect('tpl-1')

    expect(personas.items[0].metadata).toEqual({ role: 'owner' })
    expect(saved.template.placeholder_variables).toEqual(['name'])
    expect(templates.templates[0].template_id).toBe('tpl-1')
    expect(rendered.rendered.subject).toBe('Hello Alex')
    expect(preview.ready_count).toBe(1)
    expect(deleted.deleted).toBe(true)
  })

  it('maps workflow actions through CommunicationsService', async () => {
    const fetchMock = vi.fn().mockResolvedValue(
      new Response(JSON.stringify({
        commandId: 'cmd-1',
        eventId: 'workflow_action:cmd-1',
        action: 'create_task',
        status: 'created',
        target: {
          kind: 'task',
          id: 'task-1'
        },
        provenance: {
          sourceKind: 'communication_message',
          sourceId: 'msg-1',
          confidence: null,
          evidence: ['derived from message subject']
        }
      }), {
        status: 200,
        headers: { 'Content-Type': 'application/json' }
      })
    )
    vi.stubGlobal('fetch', fetchMock)

    const response = await runWorkflowActionConnect({
      command_id: 'cmd-1',
      action: 'create_task',
      source: { kind: 'communication_message', id: 'msg-1' },
      input: { title: 'Follow up' }
    })

    expect(response).toEqual({
      command_id: 'cmd-1',
      event_id: 'workflow_action:cmd-1',
      action: 'create_task',
      status: 'created',
      target: { kind: 'task', id: 'task-1' },
      provenance: {
        source_kind: 'communication_message',
        source_id: 'msg-1',
        confidence: null,
        evidence: ['derived from message subject']
      }
    })
    expect(fetchMock.mock.calls[0][0]).toBe(
      'http://127.0.0.1:8080/makosh.communications.v1.CommunicationsService/RunWorkflowAction'
    )
  })

  it('maps local state and bulk message actions through CommunicationsService', async () => {
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

    const trashed = await trashMessageConnect('msg-1')
    const restored = await restoreMessageConnect('msg-1')
    const bulk = await bulkMessageActionConnect({
      action: 'trash',
      message_ids: ['msg-1', 'msg-2']
    })

    expect(trashed).toEqual({
      message_id: 'msg-1',
      local_state: 'trash',
      provider_deleted: false
    })
    expect(restored).toEqual({
      message_id: 'msg-1',
      local_state: 'active',
      provider_deleted: undefined
    })
    expect(bulk).toEqual({
      action: 'trash',
      requested_count: 2,
      matched_count: 2,
      updated_count: 2,
      not_found: []
    })
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

  it('maps mark-read and provider-delete alias calls through CommunicationsService', async () => {
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

    const markedRead = await markMessageReadConnect('msg-1')
    const deleted = await deleteMessageFromProviderConnect('msg-1')

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

  it('maps analyze, explain and smart cc calls through CommunicationsService', async () => {
    const fetchMock = vi
      .fn()
      .mockResolvedValueOnce(
        new Response(JSON.stringify({
          messageId: 'msg-1',
          analyzed: true,
          category: 'follow_up',
          summary: 'Needs a reply this week',
          summaryContract: {
            keyPoints: ['Client asks for updated quote'],
            actionItems: ['Reply with pricing'],
            risks: ['Renewal may slip'],
            deadlines: ['2026-06-30'],
            eventCandidates: [{ title: 'Renewal review', evidence: 'next Tuesday at 10' }],
            personaCandidates: [{ title: 'Alice Example', evidence: 'from Alice <alice@example.com>' }],
            organizationCandidates: [],
            documentCandidates: [],
            agreementCandidates: []
          },
          importanceScore: 88,
          workflowState: 'needs_action',
          source: 'local_heuristic',
          evidence: ['Contains a deadline']
        }), {
          status: 200,
          headers: { 'Content-Type': 'application/json' }
        })
      )
      .mockResolvedValueOnce(
        new Response(JSON.stringify({
          reasons: ['Contains a direct request', 'Mentions a deadline']
        }), {
          status: 200,
          headers: { 'Content-Type': 'application/json' }
        })
      )
      .mockResolvedValueOnce(
        new Response(JSON.stringify({
          suggestions: ['finance@example.com', 'owner@example.com']
        }), {
          status: 200,
          headers: { 'Content-Type': 'application/json' }
        })
      )
    vi.stubGlobal('fetch', fetchMock)

    const analyzed = await analyzeMessageConnect('msg-1')
    const explained = await fetchMessageExplainConnect('msg-1')
    const smartCc = await fetchMessageSmartCcConnect('msg-1')

    expect(analyzed).toEqual({
      message_id: 'msg-1',
      analyzed: true,
      category: 'follow_up',
      summary: 'Needs a reply this week',
      summary_contract: {
        key_points: ['Client asks for updated quote'],
        action_items: ['Reply with pricing'],
        risks: ['Renewal may slip'],
        deadlines: ['2026-06-30'],
        event_candidates: [{ title: 'Renewal review', evidence: 'next Tuesday at 10' }],
        persona_candidates: [{ title: 'Alice Example', evidence: 'from Alice <alice@example.com>' }],
        organization_candidates: [],
        document_candidates: [],
        agreement_candidates: []
      },
      importance_score: 88,
      workflow_state: 'needs_action',
      source: 'local_heuristic',
      confidence: null,
      evidence: ['Contains a deadline']
    })
    expect(explained).toEqual({
      reasons: ['Contains a direct request', 'Mentions a deadline']
    })
    expect(smartCc).toEqual({
      suggestions: ['finance@example.com', 'owner@example.com']
    })
    expect(fetchMock.mock.calls[0][0]).toBe(
      'http://127.0.0.1:8080/makosh.communications.v1.CommunicationsService/AnalyzeMessage'
    )
    expect(fetchMock.mock.calls[1][0]).toBe(
      'http://127.0.0.1:8080/makosh.communications.v1.CommunicationsService/GetMessageExplain'
    )
    expect(fetchMock.mock.calls[2][0]).toBe(
      'http://127.0.0.1:8080/makosh.communications.v1.CommunicationsService/GetMessageSmartCc'
    )
    expect(JSON.parse(decodeBody(fetchMock.mock.calls[0][1].body))).toMatchObject({
      messageId: 'msg-1'
    })
  })

  it('maps export, auth and signature calls through CommunicationsService', async () => {
    const fetchMock = vi
      .fn()
      .mockResolvedValueOnce(
        new Response(JSON.stringify({
          contentType: 'text/markdown',
          content: '# Subject',
          filename: 'message_msg-1.md'
        }), {
          status: 200,
          headers: { 'Content-Type': 'application/json' }
        })
      )
      .mockResolvedValueOnce(
        new Response(JSON.stringify({
          auth: {
            spf: { result: 'pass', domain: 'alice@example.com' },
            dkim: { result: 'pass', domain: 'example.com', selector: 'mail' },
            rawHeaders: ['Authentication-Results: spf=pass dkim=pass']
          },
          risk: {
            hasSpf: true,
            spfPass: true,
            hasDkim: true,
            dkimPass: true,
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
          hasSignature: true,
          signatureType: 'pgp',
          signerInfo: null,
          isValid: null,
          certExpiryWarning: null
        }), {
          status: 200,
          headers: { 'Content-Type': 'application/json' }
        })
      )
    vi.stubGlobal('fetch', fetchMock)

    const exported = await exportMessageConnect('msg-1', 'md')
    const auth = await fetchMessageAuthConnect('msg-1')
    const signature = await fetchMessageSignatureConnect('msg-1')

    expect(exported).toEqual({
      content_type: 'text/markdown',
      content: '# Subject',
      filename: 'message_msg-1.md'
    })
    expect(auth.auth.spf?.result).toBe('pass')
    expect(auth.risk.risk_summary).toBe('Authentication checks passed')
    expect(signature).toEqual({
      has_signature: true,
      signature_type: 'pgp',
      signer_info: null,
      is_valid: null,
      cert_expiry_warning: null
    })
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

  it('maps flag, snooze and label commands through CommunicationsService', async () => {
    const fetchMock = vi
      .fn()
      .mockResolvedValueOnce(
        new Response(JSON.stringify({
          messageId: 'msg-1',
          pinned: true
        }), {
          status: 200,
          headers: { 'Content-Type': 'application/json' }
        })
      )
      .mockResolvedValueOnce(
        new Response(JSON.stringify({
          messageId: 'msg-1',
          important: true
        }), {
          status: 200,
          headers: { 'Content-Type': 'application/json' }
        })
      )
      .mockResolvedValueOnce(
        new Response(JSON.stringify({
          messageId: 'msg-1',
          muted: true
        }), {
          status: 200,
          headers: { 'Content-Type': 'application/json' }
        })
      )
      .mockResolvedValueOnce(
        new Response(JSON.stringify({
          snoozed: true
        }), {
          status: 200,
          headers: { 'Content-Type': 'application/json' }
        })
      )
      .mockResolvedValueOnce(
        new Response(JSON.stringify({
          labeled: true
        }), {
          status: 200,
          headers: { 'Content-Type': 'application/json' }
        })
      )
      .mockResolvedValueOnce(
        new Response(JSON.stringify({
          removed: true
        }), {
          status: 200,
          headers: { 'Content-Type': 'application/json' }
        })
      )
    vi.stubGlobal('fetch', fetchMock)

    const pinned = await toggleMessagePinConnect('msg-1')
    const important = await toggleMessageImportantConnect('msg-1')
    const muted = await toggleMessageMuteConnect('msg-1')
    const snoozed = await snoozeMessageConnect('msg-1', '2026-06-30T10:00:00Z')
    const labeled = await addMessageLabelConnect('msg-1', 'follow-up')
    const removed = await removeMessageLabelConnect('msg-1', 'follow-up')

    expect(pinned).toEqual({ message_id: 'msg-1', pinned: true })
    expect(important).toEqual({ message_id: 'msg-1', important: true })
    expect(muted).toEqual({ message_id: 'msg-1', pinned: true })
    expect(snoozed).toEqual({ snoozed: true })
    expect(labeled).toEqual({ labeled: true })
    expect(removed).toEqual({ removed: true })
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
    expect(fetchMock.mock.calls[5][0]).toBe(
      'http://127.0.0.1:8080/makosh.communications.v1.CommunicationsService/RemoveMessageLabel'
    )
  })

  it('maps message language, translation and extraction helpers through CommunicationsService', async () => {
    const fetchMock = vi
      .fn()
      .mockResolvedValueOnce(
        new Response(JSON.stringify({
          language: 'es',
          confidence: 0.91,
          script: 'latin'
        }), {
          status: 200,
          headers: { 'Content-Type': 'application/json' }
        })
      )
      .mockResolvedValueOnce(
        new Response(JSON.stringify({
          translated: true,
          text: 'Hello team',
          target: 'en',
          model: 'qwen3:4b'
        }), {
          status: 200,
          headers: { 'Content-Type': 'application/json' }
        })
      )
      .mockResolvedValueOnce(
        new Response(JSON.stringify({
          tasks: [{
            title: 'Review contract by Friday',
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
            title: 'Quarterly update',
            content: 'Important note',
            tags: ['finance'],
            source: 'heuristic'
          }]
        }), {
          status: 200,
          headers: { 'Content-Type': 'application/json' }
        })
      )
    vi.stubGlobal('fetch', fetchMock)

    const language = await detectMessageLanguageConnect('msg-1')
    const translation = await translateMessageConnect('msg-1', 'en')
    const tasks = await extractMessageTasksConnect('msg-1')
    const notes = await extractMessageNotesConnect('msg-1')

    expect(language).toEqual({
      language: 'es',
      confidence: 0.91,
      script: 'latin'
    })
    expect(translation).toEqual({
      translated: true,
      text: 'Hello team',
      target: 'en',
      model: 'qwen3:4b',
      reason: undefined
    })
    expect(tasks.tasks[0]).toEqual({
      title: 'Review contract by Friday',
      due_date: 'Friday',
      assignee: null,
      priority: 'normal',
      source: 'heuristic'
    })
    expect(notes.notes[0]).toEqual({
      title: 'Quarterly update',
      content: 'Important note',
      tags: ['finance'],
      source: 'heuristic'
    })
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

  it('maps message, thread, draft and outbox queries from CommunicationsService', async () => {
    const fetchMock = vi
      .fn()
      .mockResolvedValueOnce(
        new Response(JSON.stringify({
          item: {
            messageId: 'msg-1',
            rawRecordId: 'raw-1',
            observationId: 'obs-1',
            accountId: 'account-1',
            providerRecordId: 'provider-1',
            subject: 'Quarterly update',
            sender: 'Ada <ada@example.com>',
            recipients: ['bob@example.com'],
            bodyText: 'Body text',
            bodyHtml: '<p><strong>Body</strong> text</p>',
            occurredAt: '2026-06-23T10:00:00Z',
            projectedAt: '2026-06-23T10:01:00Z',
            channelKind: 'email',
            conversationId: 'thread-1',
            senderDisplayName: 'Ada',
            deliveryState: 'received',
            messageMetadataJson: '{"tag":"important"}',
            workflowState: 'new',
            localState: 'active'
          },
          attachments: [{
            attachmentId: 'att-1',
            messageId: 'msg-1',
            rawRecordId: 'raw-1',
            blobId: 'blob-1',
            providerAttachmentId: 'provider-att-1',
            filename: 'notes.txt',
            contentType: 'text/plain',
            sizeBytes: 42,
            sha256: 'abc',
            disposition: 'attachment',
            scanStatus: 'clean',
            scanMetadataJson: '{"scanner":"noop"}',
            storageKind: 'local_fs',
            storagePath: '/tmp/notes.txt',
            createdAt: '2026-06-23T10:01:00Z',
            updatedAt: '2026-06-23T10:01:00Z'
          }]
        }), {
          status: 200,
          headers: { 'Content-Type': 'application/json' }
        })
      )
      .mockResolvedValueOnce(
        new Response(JSON.stringify({
          items: [{
            threadId: 'thread-1',
            accountId: 'account-1',
            subject: 'Quarterly update',
            messageCount: 3,
            participantCount: 2,
            firstMessageAt: '2026-06-23T09:00:00Z',
            lastMessageAt: '2026-06-23T10:00:00Z',
            lastActivityAt: '2026-06-23T10:00:00Z',
            hasOpenAction: true,
            hasAttachments: true,
            dominantWorkflowState: 'needs_action'
          }],
          nextCursor: 'threads-next',
          hasMore: true
        }), {
          status: 200,
          headers: { 'Content-Type': 'application/json' }
        })
      )
      .mockResolvedValueOnce(
        new Response(JSON.stringify({
          items: [{
            messageId: 'msg-1',
            providerRecordId: 'provider-1',
            accountId: 'account-1',
            subject: 'Quarterly update',
            sender: 'Ada <ada@example.com>',
            senderDisplayName: 'Ada',
            bodyText: 'Body text',
            occurredAt: '2026-06-23T10:00:00Z',
            projectedAt: '2026-06-23T10:01:00Z',
            workflowState: 'new',
            deliveryState: 'received',
            attachmentCount: 0,
            attachments: []
          }]
        }), {
          status: 200,
          headers: { 'Content-Type': 'application/json' }
        })
      )
      .mockResolvedValueOnce(
        new Response(JSON.stringify({
          items: [{
            draftId: 'draft-1',
            accountId: 'account-1',
            toRecipients: ['bob@example.com'],
            ccRecipients: [],
            bccRecipients: [],
            subject: 'Draft',
            bodyText: 'Draft body',
            status: 'draft',
            sendAttempts: 0,
            metadataJson: '{"origin":"connect"}',
            createdAt: '2026-06-23T10:00:00Z',
            updatedAt: '2026-06-23T10:00:00Z'
          }],
          page: {
            nextCursor: 'drafts-next',
            hasMore: true
          }
        }), {
          status: 200,
          headers: { 'Content-Type': 'application/json' }
        })
      )
      .mockResolvedValueOnce(
        new Response(JSON.stringify({
          items: [{
            outboxId: 'outbox-1',
            accountId: 'account-1',
            toRecipients: ['bob@example.com'],
            ccRecipients: [],
            bccRecipients: [],
            subject: 'Queued',
            bodyText: 'Queued body',
            status: 'queued',
            sendAttempts: 0,
            metadataJson: '{"source":"connect"}',
            createdAt: '2026-06-23T10:00:00Z',
            updatedAt: '2026-06-23T10:00:00Z'
          }],
          page: {
            nextCursor: 'outbox-next',
            hasMore: false
          }
        }), {
          status: 200,
          headers: { 'Content-Type': 'application/json' }
        })
      )
    vi.stubGlobal('fetch', fetchMock)

    const message = await fetchCommunicationMessageConnect('msg-1')
    const threads = await fetchCommunicationThreadsConnect('account-1', 25, 'cursor-1')
    const threadMessages = await fetchCommunicationThreadMessagesConnect('account-1', 'Quarterly update', 25)
    const drafts = await fetchCommunicationDraftsConnect('account-1', 'draft', 25, 'cursor-2')
    const outbox = await fetchCommunicationOutboxConnect('account-1', 'queued', 25, 'cursor-3')

    expect(message.message.observation_id).toBe('obs-1')
    expect(message.message.body_html).toBe('<p><strong>Body</strong> text</p>')
    expect(message.attachments[0].scan_status).toBe('clean')
    expect(threads.next_cursor).toBe('threads-next')
    expect(threadMessages.items[0].provider_record_id).toBe('provider-1')
    expect(drafts.items[0].metadata.origin).toBe('connect')
    expect(drafts.next_cursor).toBe('drafts-next')
    expect(outbox.items[0].metadata.source).toBe('connect')
    expect(outbox.next_cursor).toBe('outbox-next')
    expect(fetchMock.mock.calls[0][0]).toBe(
      'http://127.0.0.1:8080/makosh.communications.v1.CommunicationsService/GetMessage'
    )
    expect(fetchMock.mock.calls[1][0]).toBe(
      'http://127.0.0.1:8080/makosh.communications.v1.CommunicationsService/ListThreads'
    )
    expect(fetchMock.mock.calls[2][0]).toBe(
      'http://127.0.0.1:8080/makosh.communications.v1.CommunicationsService/ListThreadMessages'
    )
    expect(fetchMock.mock.calls[3][0]).toBe(
      'http://127.0.0.1:8080/makosh.communications.v1.CommunicationsService/ListDrafts'
    )
    expect(fetchMock.mock.calls[4][0]).toBe(
      'http://127.0.0.1:8080/makosh.communications.v1.CommunicationsService/ListOutbox'
    )
  })

  it('translates a thread through CommunicationsService', async () => {
    const fetchMock = vi.fn().mockResolvedValue(
      new Response(JSON.stringify({
        accountId: 'account-1',
        subject: 'Quarterly update',
        targetLanguage: 'en',
        items: [{
          messageId: 'msg-1',
          originalLanguage: 'ru',
          confidence: 0.91,
          translated: false,
          text: null,
          target: 'en',
          model: null,
          reason: 'no LLM configured'
        }]
      }), {
        status: 200,
        headers: { 'Content-Type': 'application/json' }
      })
    )
    vi.stubGlobal('fetch', fetchMock)

    const response = await translateCommunicationThreadConnect(
      'account-1',
      'Quarterly update',
      'en',
      25
    )

    expect(response.account_id).toBe('account-1')
    expect(response.items[0].original_language).toBe('ru')
    expect(fetchMock.mock.calls[0][0]).toBe(
      'http://127.0.0.1:8080/makosh.communications.v1.CommunicationsService/TranslateThread'
    )
    expect(JSON.parse(decodeBody(fetchMock.mock.calls[0][1].body))).toMatchObject({
      accountId: 'account-1',
      subject: 'Quarterly update',
      targetLanguage: 'en',
      limit: 25
    })
  })

  it('sends a communication through CommunicationsService and returns the send result', async () => {
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
          metadataJson: '{"source":"connect","priority":"high"}',
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
        undoDeadlineAt: '2026-06-23T10:05:00Z',
        failureReason: null
      }), {
        status: 200,
        headers: { 'Content-Type': 'application/json' }
      })
    )
    vi.stubGlobal('fetch', fetchMock)

    const response = await sendCommunicationConnect({
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
    expect(response.accepted_recipients).toEqual(['bob@example.com'])
    expect(response.undo_deadline_at).toBe('2026-06-23T10:05:00Z')
    expect(fetchMock).toHaveBeenCalledOnce()
    const [url, options] = fetchMock.mock.calls[0]
    expect(url).toBe('http://127.0.0.1:8080/makosh.communications.v1.CommunicationsService/SendMessage')
    expect(JSON.parse(decodeBody(options.body))).toMatchObject({
      accountId: 'account-1',
      toRecipients: ['bob@example.com'],
      subject: 'Connect send',
      bodyText: 'Queued body',
      draftId: 'draft-1',
      undoSendSeconds: '300',
      confirmedProviderWrite: true
    })
  })

  it('redirects a message through CommunicationsService and returns the enqueue result', async () => {
    const fetchMock = vi.fn().mockResolvedValue(
      new Response(JSON.stringify({
        messageId: 'outbox-redirect-1',
        outboxId: 'outbox-redirect-1',
        accepted: ['redirect@example.com'],
        acceptedRecipients: ['redirect@example.com'],
        transport: 'outbox',
        status: 'queued',
        scheduledSendAt: null,
        undoDeadlineAt: '2026-06-23T10:05:00Z',
        failureReason: null
      }), {
        status: 200,
        headers: { 'Content-Type': 'application/json' }
      })
    )
    vi.stubGlobal('fetch', fetchMock)

    const response = await redirectMessageConnect('msg-1', {
      to: ['redirect@example.com'],
      cc: ['copy@example.com'],
      confirmed_provider_write: true
    })

    expect(response.outbox_id).toBe('outbox-redirect-1')
    expect(response.transport).toBe('outbox')
    expect(fetchMock).toHaveBeenCalledOnce()
    expect(fetchMock.mock.calls[0][0]).toBe(
      'http://127.0.0.1:8080/makosh.communications.v1.CommunicationsService/RedirectMessage'
    )
    expect(JSON.parse(decodeBody(fetchMock.mock.calls[0][1].body))).toMatchObject({
      messageId: 'msg-1',
      toRecipients: ['redirect@example.com'],
      ccRecipients: ['copy@example.com'],
      confirmedProviderWrite: true
    })
  })

  it('creates, deletes and undoes provider-neutral operations through CommunicationsService', async () => {
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
            status: 'draft',
            sendAttempts: 0,
            metadataJson: '{"compose_mode":"reply"}',
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
      .mockResolvedValueOnce(
        new Response(JSON.stringify({
          item: {
            outboxId: 'outbox-1',
            accountId: 'account-1',
            draftId: 'draft-1',
            toRecipients: ['bob@example.com'],
            ccRecipients: [],
            bccRecipients: [],
            subject: 'Queued',
            bodyText: 'Queued body',
            status: 'canceled',
            sendAttempts: 1,
            metadataJson: '{"source":"connect"}',
            createdAt: '2026-06-23T10:00:00Z',
            updatedAt: '2026-06-23T10:05:00Z'
          }
        }), {
          status: 200,
          headers: { 'Content-Type': 'application/json' }
        })
      )
    vi.stubGlobal('fetch', fetchMock)

    const draft = await createCommunicationDraftConnect({
      draft_id: 'draft-1',
      account_id: 'account-1',
      to_recipients: ['bob@example.com'],
      subject: 'Draft',
      body_text: 'Draft body',
      metadata: { compose_mode: 'reply' }
    })
    const deletion = await deleteCommunicationDraftConnect('draft-1')
    const outboxItem = await undoCommunicationOutboxItemConnect('outbox-1')

    expect(draft.draft_id).toBe('draft-1')
    expect(draft.metadata.compose_mode).toBe('reply')
    expect(deletion.deleted).toBe(true)
    expect(outboxItem.outbox_id).toBe('outbox-1')
    expect(outboxItem.status).toBe('canceled')
    expect(fetchMock.mock.calls[0][0]).toBe(
      'http://127.0.0.1:8080/makosh.communications.v1.CommunicationsService/CreateDraft'
    )
    expect(JSON.parse(decodeBody(fetchMock.mock.calls[0][1].body))).toMatchObject({
      draftId: 'draft-1',
      accountId: 'account-1',
      toRecipients: ['bob@example.com'],
      subject: 'Draft',
      bodyText: 'Draft body',
      metadataJson: '{"compose_mode":"reply"}'
    })
    expect(fetchMock.mock.calls[1][0]).toBe(
      'http://127.0.0.1:8080/makosh.communications.v1.CommunicationsService/DeleteDraft'
    )
    expect(JSON.parse(decodeBody(fetchMock.mock.calls[1][1].body))).toMatchObject({
      draftId: 'draft-1'
    })
    expect(fetchMock.mock.calls[2][0]).toBe(
      'http://127.0.0.1:8080/makosh.communications.v1.CommunicationsService/UndoOutboxItem'
    )
    expect(JSON.parse(decodeBody(fetchMock.mock.calls[2][1].body))).toMatchObject({
      outboxId: 'outbox-1'
    })
  })

  it('lists and mutates saved searches through CommunicationsService', async () => {
    const fetchMock = vi
      .fn()
      .mockResolvedValueOnce(
        new Response(JSON.stringify({
          items: [{
            savedSearchId: 'mail_saved_search:1',
            name: 'Invoices',
            description: 'Needs action invoices',
            accountId: 'account-1',
            query: 'invoice',
            workflowState: 'needs_action',
            localState: 'active',
            channelKind: 'email',
            isSmartFolder: true,
            sortOrder: 10,
            messageCount: 3,
            createdAt: '2026-06-23T10:00:00Z',
            updatedAt: '2026-06-23T10:00:00Z'
          }],
          page: {
            nextCursor: 'saved-search-next',
            hasMore: true
          }
        }), {
          status: 200,
          headers: { 'Content-Type': 'application/json' }
        })
      )
      .mockResolvedValueOnce(
        new Response(JSON.stringify({
          item: {
            savedSearchId: 'mail_saved_search:1',
            name: 'Invoices',
            description: 'Needs action invoices',
            accountId: 'account-1',
            query: 'invoice',
            workflowState: 'needs_action',
            localState: 'active',
            channelKind: 'email',
            isSmartFolder: true,
            sortOrder: 10,
            messageCount: 3,
            createdAt: '2026-06-23T10:00:00Z',
            updatedAt: '2026-06-23T10:00:00Z'
          }
        }), {
          status: 200,
          headers: { 'Content-Type': 'application/json' }
        })
      )
      .mockResolvedValueOnce(
        new Response(JSON.stringify({
          item: {
            savedSearchId: 'mail_saved_search:1',
            name: 'Waiting invoices',
            description: 'Waiting invoices',
            accountId: 'account-1',
            query: 'invoice',
            workflowState: 'waiting',
            localState: 'active',
            channelKind: 'email',
            isSmartFolder: false,
            sortOrder: 20,
            messageCount: 1,
            createdAt: '2026-06-23T10:00:00Z',
            updatedAt: '2026-06-23T10:05:00Z'
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

    const list = await fetchCommunicationSavedSearchesConnect(true, 'account-1', 25, 'cursor:value')
    const created = await createCommunicationSavedSearchConnect({
      name: 'Invoices',
      description: 'Needs action invoices',
      account_id: 'account-1',
      query: 'invoice',
      workflow_state: 'needs_action',
      local_state: 'active',
      channel_kind: 'email',
      is_smart_folder: true,
      sort_order: 10
    })
    const updated = await updateCommunicationSavedSearchConnect('mail_saved_search:1', {
      name: 'Waiting invoices',
      workflow_state: 'waiting',
      is_smart_folder: false,
      sort_order: 20
    })
    const deleted = await deleteCommunicationSavedSearchConnect('mail_saved_search:1')

    expect(list.next_cursor).toBe('saved-search-next')
    expect(list.items[0].saved_search_id).toBe('mail_saved_search:1')
    expect(created.name).toBe('Invoices')
    expect(updated.workflow_state).toBe('waiting')
    expect(deleted.deleted).toBe(true)
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
    expect(JSON.parse(decodeBody(fetchMock.mock.calls[1][1].body))).toMatchObject({
      name: 'Invoices',
      description: 'Needs action invoices',
      accountId: 'account-1',
      query: 'invoice',
      workflowState: 'needs_action',
      localState: 'active',
      channelKind: 'email',
      isSmartFolder: true,
      sortOrder: 10
    })
    expect(fetchMock.mock.calls[2][0]).toBe(
      'http://127.0.0.1:8080/makosh.communications.v1.CommunicationsService/UpdateSavedSearch'
    )
    expect(JSON.parse(decodeBody(fetchMock.mock.calls[2][1].body))).toMatchObject({
      savedSearchId: 'mail_saved_search:1',
      name: 'Waiting invoices',
      workflowState: 'waiting',
      isSmartFolder: false,
      sortOrder: 20
    })
    expect(fetchMock.mock.calls[3][0]).toBe(
      'http://127.0.0.1:8080/makosh.communications.v1.CommunicationsService/DeleteSavedSearch'
    )
    expect(JSON.parse(decodeBody(fetchMock.mock.calls[3][1].body))).toMatchObject({
      savedSearchId: 'mail_saved_search:1'
    })
  })

  it('lists and mutates folders through CommunicationsService', async () => {
    const fetchMock = vi
      .fn()
      .mockResolvedValueOnce(
        new Response(JSON.stringify({
          items: [{
            folderId: 'mail_folder:1',
            accountId: 'account-1',
            name: 'Clients',
            description: 'Important clients',
            color: '#3b82f6',
            sortOrder: 10,
            messageCount: 3,
            createdAt: '2026-06-23T10:00:00Z',
            updatedAt: '2026-06-23T10:00:00Z'
          }],
          page: {
            nextCursor: 'folder-next',
            hasMore: true
          }
        }), {
          status: 200,
          headers: { 'Content-Type': 'application/json' }
        })
      )
      .mockResolvedValueOnce(
        new Response(JSON.stringify({
          item: {
            folderId: 'mail_folder:1',
            accountId: 'account-1',
            name: 'Clients',
            description: 'Important clients',
            color: '#3b82f6',
            sortOrder: 10,
            messageCount: 3,
            createdAt: '2026-06-23T10:00:00Z',
            updatedAt: '2026-06-23T10:00:00Z'
          }
        }), {
          status: 200,
          headers: { 'Content-Type': 'application/json' }
        })
      )
      .mockResolvedValueOnce(
        new Response(JSON.stringify({
          item: {
            folderId: 'mail_folder:1',
            accountId: 'account-1',
            name: 'VIP Clients',
            description: 'Important clients',
            color: '#2563eb',
            sortOrder: 20,
            messageCount: 3,
            createdAt: '2026-06-23T10:00:00Z',
            updatedAt: '2026-06-23T10:05:00Z'
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
      .mockResolvedValueOnce(
        new Response(JSON.stringify({
          items: [{
            folderId: 'mail_folder:1',
            messageId: 'msg-1',
            accountId: 'account-1',
            subject: 'Quarterly update',
            sender: 'Ada <ada@example.com>',
            occurredAt: '2026-06-23T10:00:00Z',
            projectedAt: '2026-06-23T10:01:00Z',
            workflowState: 'needs_action',
            localState: 'active',
            addedAt: '2026-06-23T10:02:00Z',
            attachmentCount: 1
          }],
          page: {
            nextCursor: 'folder-msg-next',
            hasMore: false
          }
        }), {
          status: 200,
          headers: { 'Content-Type': 'application/json' }
        })
      )
      .mockResolvedValueOnce(
        new Response(JSON.stringify({
          item: {
            operation: 'copy',
            folderId: 'mail_folder:1',
            messageId: 'msg-1',
            message: {
              folderId: 'mail_folder:1',
              messageId: 'msg-1',
              accountId: 'account-1',
              subject: 'Quarterly update',
              sender: 'Ada <ada@example.com>',
              occurredAt: '2026-06-23T10:00:00Z',
              projectedAt: '2026-06-23T10:01:00Z',
              workflowState: 'needs_action',
              localState: 'active',
              addedAt: '2026-06-23T10:02:00Z',
              attachmentCount: 1
            }
          }
        }), {
          status: 200,
          headers: { 'Content-Type': 'application/json' }
        })
      )
      .mockResolvedValueOnce(
        new Response(JSON.stringify({
          item: {
            operation: 'move',
            folderId: 'mail_folder:1',
            messageId: 'msg-1',
            message: {
              folderId: 'mail_folder:1',
              messageId: 'msg-1',
              accountId: 'account-1',
              subject: 'Quarterly update',
              sender: 'Ada <ada@example.com>',
              occurredAt: '2026-06-23T10:00:00Z',
              projectedAt: '2026-06-23T10:01:00Z',
              workflowState: 'needs_action',
              localState: 'active',
              addedAt: '2026-06-23T10:02:00Z',
              attachmentCount: 1
            }
          }
        }), {
          status: 200,
          headers: { 'Content-Type': 'application/json' }
        })
      )
    vi.stubGlobal('fetch', fetchMock)

    const folders = await fetchCommunicationFoldersConnect('account-1', 25, 'cursor-1')
    const created = await createCommunicationFolderConnect({
      account_id: 'account-1',
      name: 'Clients',
      description: 'Important clients',
      color: '#3b82f6',
      sort_order: 10
    })
    const updated = await updateCommunicationFolderConnect('mail_folder:1', {
      name: 'VIP Clients',
      color: '#2563eb',
      sort_order: 20
    })
    const deleted = await deleteCommunicationFolderConnect('mail_folder:1')
    const folderMessages = await fetchFolderMessagesConnect('mail_folder:1', 25, 'cursor-2')
    const copied = await copyMessageToFolderConnect('mail_folder:1', 'msg-1')
    const moved = await moveMessageToFolderConnect('mail_folder:1', 'msg-1')

    expect(folders.items[0].folder_id).toBe('mail_folder:1')
    expect(created.name).toBe('Clients')
    expect(updated.name).toBe('VIP Clients')
    expect(deleted.deleted).toBe(true)
    expect(folderMessages.items[0].message_id).toBe('msg-1')
    expect(copied.operation).toBe('copy')
    expect(moved.operation).toBe('move')
    expect(fetchMock.mock.calls[0][0]).toBe(
      'http://127.0.0.1:8080/makosh.communications.v1.CommunicationsService/ListFolders'
    )
    expect(JSON.parse(decodeBody(fetchMock.mock.calls[0][1].body))).toMatchObject({
      accountId: 'account-1',
      page: {
        limit: 25,
        cursor: 'cursor-1'
      }
    })
    expect(fetchMock.mock.calls[1][0]).toBe(
      'http://127.0.0.1:8080/makosh.communications.v1.CommunicationsService/CreateFolder'
    )
    expect(fetchMock.mock.calls[2][0]).toBe(
      'http://127.0.0.1:8080/makosh.communications.v1.CommunicationsService/UpdateFolder'
    )
    expect(fetchMock.mock.calls[3][0]).toBe(
      'http://127.0.0.1:8080/makosh.communications.v1.CommunicationsService/DeleteFolder'
    )
    expect(fetchMock.mock.calls[4][0]).toBe(
      'http://127.0.0.1:8080/makosh.communications.v1.CommunicationsService/ListFolderMessages'
    )
    expect(fetchMock.mock.calls[5][0]).toBe(
      'http://127.0.0.1:8080/makosh.communications.v1.CommunicationsService/CopyMessageToFolder'
    )
    expect(fetchMock.mock.calls[6][0]).toBe(
      'http://127.0.0.1:8080/makosh.communications.v1.CommunicationsService/MoveMessageToFolder'
    )
  })

  it('maps ai reply and ai reply variants through CommunicationsService', async () => {
    const fetchMock = vi
      .fn()
      .mockResolvedValueOnce(
        new Response(JSON.stringify({
          subject: 'Re: Quarterly update',
          body: 'Thanks, sending the updated numbers today.',
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
              body: 'Professional reply',
              tone: 'professional',
              language: 'en',
              generated: true
            },
            {
              subject: 'Re: Quarterly update',
              body: 'Friendly reply',
              tone: 'friendly',
              language: 'es',
              generated: true
            }
          ]
        }), {
          status: 200,
          headers: { 'Content-Type': 'application/json' }
        })
      )
    vi.stubGlobal('fetch', fetchMock)

    const reply = await generateAiReplyConnect('msg-1', { tone: 'business', language: 'en' })
    const variants = await generateAiReplyVariantsConnect('msg-1', {
      languages: ['en', 'es'],
      tones: ['professional', 'friendly']
    })

    expect(reply.generated).toBe(true)
    expect(reply.body).toBe('Thanks, sending the updated numbers today.')
    expect(variants.variants).toHaveLength(2)
    expect(variants.variants[1].tone).toBe('friendly')
    expect(fetchMock.mock.calls[0][0]).toBe(
      'http://127.0.0.1:8080/makosh.communications.v1.CommunicationsService/GenerateAiReply'
    )
    expect(fetchMock.mock.calls[1][0]).toBe(
      'http://127.0.0.1:8080/makosh.communications.v1.CommunicationsService/GenerateAiReplyVariants'
    )
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
