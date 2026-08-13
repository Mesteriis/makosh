import { MarkerType, type Edge, type Node } from '@vue-flow/core'
import type { TranslationFunction } from '../../../platform/i18n/types'
import type { PersonaPanelProfile, Relationship } from '../types/persona'
import { personaInitials } from './personaWorkspaceElements'

export type RelationshipNodeData = {
  id: string
  kind: string
  kindLabel: string
  initials: string
  title: string
  subtitle: string
}

export type RelationshipEdgeData = {
  relationshipId: string
  type: string
  state: string
  revision: number
  validFrom: string
  validUntil: string | null
  sourceTitle: string
  targetTitle: string
  icon: string
  iconLabel: string
}

export type RelationshipGraphDetail = {
  eyebrow: string
  title: string
  description: string
  rows: Array<{ label: string; value: string }>
}

export function relationshipGraphEdgeData(value: unknown): RelationshipEdgeData | null {
  if (!isRecord(value)) return null
  const data = value
  if (
    typeof data.relationshipId !== 'string' ||
    typeof data.type !== 'string' ||
    typeof data.state !== 'string' ||
    typeof data.revision !== 'number' ||
    typeof data.validFrom !== 'string' ||
    !(typeof data.validUntil === 'string' || data.validUntil === null) ||
    typeof data.sourceTitle !== 'string' ||
    typeof data.targetTitle !== 'string' ||
    typeof data.icon !== 'string' ||
    typeof data.iconLabel !== 'string'
  ) return null

  return {
    relationshipId: data.relationshipId,
    type: data.type,
    state: data.state,
    revision: data.revision,
    validFrom: data.validFrom,
    validUntil: data.validUntil,
    sourceTitle: data.sourceTitle,
    targetTitle: data.targetTitle,
    icon: data.icon,
    iconLabel: data.iconLabel,
  }
}

function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === 'object' && value !== null
}

const graphRootPosition = { x: 300, y: 190 }

export function buildRelationshipGraphNodes(params: {
  selectedPersona: PersonaPanelProfile
  entityLabels: Readonly<Record<string, string>>
  relationships: readonly Relationship[]
  t: TranslationFunction
}): Node<RelationshipNodeData>[] {
  const rootNode: Node<RelationshipNodeData> = {
    id: params.selectedPersona.persona_id,
    label: params.selectedPersona.display_name,
    position: graphRootPosition,
    class: ['personas-relationship-flow-node', 'is-root'],
    data: {
      id: params.selectedPersona.persona_id,
      kind: 'persona',
      kindLabel: entityKindLabel('persona', params.t),
      initials: personaInitials(params.selectedPersona),
      title: params.selectedPersona.display_name,
      subtitle: params.t('Selected persona')
    },
    draggable: true,
    connectable: false,
    deletable: false,
    ariaLabel: params.selectedPersona.display_name
  }
  const relatedNodes = uniqueRelatedNodes(params)
  const positions = relatedNodePositions(relatedNodes.length)
  const nodes = relatedNodes.map((node, index): Node<RelationshipNodeData> => ({
    id: node.id,
    label: node.title,
    position: positions[index] ?? { x: graphRootPosition.x + 240, y: 90 + index * 90 },
    class: ['personas-relationship-flow-node', `is-${node.kind}`],
    data: node,
    draggable: true,
    connectable: false,
    deletable: false,
    ariaLabel: `${node.title} ${node.subtitle}`
  }))

  return [rootNode, ...nodes]
}

export function buildRelationshipGraphEdges(params: {
  selectedPersona: PersonaPanelProfile
  entityLabels: Readonly<Record<string, string>>
  relationships: readonly Relationship[]
  t: TranslationFunction
}): Edge<RelationshipEdgeData>[] {
  return params.relationships
    .filter(
      (relationship) =>
        relationship.source_entity_id === params.selectedPersona.persona_id ||
        relationship.target_entity_id === params.selectedPersona.persona_id
    )
    .map((relationship): Edge<RelationshipEdgeData> => {
      const sourceTitle = entityTitle(
        relationship.source_entity_kind,
        relationship.source_entity_id,
        params
      )
      const targetTitle = entityTitle(
        relationship.target_entity_kind,
        relationship.target_entity_id,
        params
      )

      return {
        id: relationship.relationship_id,
        source: relationship.source_entity_id,
        target: relationship.target_entity_id,
        type: 'relationship',
        animated: false,
        markerEnd: MarkerType.ArrowClosed,
        class: ['personas-relationship-flow-edge', `is-${relationship.state}`],
        interactionWidth: 18,
        deletable: false,
        focusable: true,
        ariaLabel: `${params.t('Click for relationship details')}: ${relationship.relationship_type}`,
        data: {
          relationshipId: relationship.relationship_id,
          type: relationship.relationship_type,
          state: relationship.state,
          revision: relationship.relationship_revision,
          validFrom: relationship.valid_from,
          validUntil: relationship.valid_until,
          sourceTitle,
          targetTitle,
          icon: relationshipIcon(relationship),
          iconLabel: `${relationshipTypeLabel(relationship.relationship_type)}. ${params.t('Open details')}`
        }
      }
    })
}

export function relationshipGraphNodeDetail(
  data: RelationshipNodeData,
  t: TranslationFunction
): RelationshipGraphDetail {
  return {
    eyebrow: data.kindLabel,
    title: data.title,
    description: data.subtitle,
    rows: [
      { label: t('Type'), value: data.kindLabel },
      { label: t('Entity id'), value: data.id }
    ]
  }
}

export function relationshipGraphEdgeDetail(
  data: RelationshipEdgeData,
  t: TranslationFunction
): RelationshipGraphDetail {
  return {
    eyebrow: t('Relationship details'),
    title: relationshipTypeLabel(data.type),
    description: `${data.sourceTitle} -> ${data.targetTitle}`,
    rows: [
      { label: t('Relationship type'), value: relationshipTypeLabel(data.type) },
      { label: t('State'), value: relationshipStateLabel(data.state, t) },
      { label: t('Revision'), value: String(data.revision) },
      { label: t('Valid from'), value: data.validFrom },
      { label: t('Valid until'), value: data.validUntil ?? t('Open ended') },
      { label: t('Source'), value: data.sourceTitle },
      { label: t('Target'), value: data.targetTitle }
    ]
  }
}

function uniqueRelatedNodes(params: {
  selectedPersona: PersonaPanelProfile
  entityLabels: Readonly<Record<string, string>>
  relationships: readonly Relationship[]
  t: TranslationFunction
}): RelationshipNodeData[] {
  const nodes = new Map<string, RelationshipNodeData>()

  for (const relationship of params.relationships) {
    const related =
      relationship.source_entity_id === params.selectedPersona.persona_id
        ? {
            id: relationship.target_entity_id,
            kind: relationship.target_entity_kind,
            relationshipType: relationship.relationship_type
          }
        : relationship.target_entity_id === params.selectedPersona.persona_id
          ? {
              id: relationship.source_entity_id,
              kind: relationship.source_entity_kind,
              relationshipType: relationship.relationship_type
            }
          : null

    if (!related || nodes.has(related.id)) continue
    const title = entityTitle(related.kind, related.id, params)
    nodes.set(related.id, {
      id: related.id,
      kind: related.kind,
      kindLabel: entityKindLabel(related.kind, params.t),
      initials: personaInitials({ display_name: title, email_address: '' }),
      title,
      subtitle: relationshipTypeLabel(related.relationshipType)
    })
  }

  return Array.from(nodes.values())
}

function relatedNodePositions(count: number): Array<{ x: number; y: number }> {
  if (count <= 0) return []

  const preferredPositions = [
    { x: 610, y: 190 },
    { x: 470, y: 46 },
    { x: 470, y: 334 },
    { x: 24, y: 190 },
    { x: 124, y: 48 },
    { x: 124, y: 332 }
  ]

  if (count <= preferredPositions.length) {
    return preferredPositions.slice(0, count)
  }

  const center = graphRootPosition
  const radiusX = 300
  const radiusY = 168
  const startAngle = -Math.PI / 2

  return Array.from({ length: count }, (_item, index) => {
    const angle = startAngle + (index / Math.max(count, 1)) * Math.PI * 2
    return {
      x: Math.round(center.x + Math.cos(angle) * radiusX),
      y: Math.round(center.y + Math.sin(angle) * radiusY)
    }
  })
}

function entityTitle(
  kind: string,
  id: string,
  params: {
    selectedPersona: PersonaPanelProfile
    entityLabels: Readonly<Record<string, string>>
  }
): string {
  const knownTitle = params.entityLabels[id]
  if (knownTitle) return knownTitle

  if (kind === 'persona' && id === params.selectedPersona.persona_id) {
    return params.selectedPersona.display_name
  }

  const raw = id.split(':').pop() || id
  return raw
    .split(/[-_]/)
    .filter(Boolean)
    .map((part) => part.slice(0, 1).toUpperCase() + part.slice(1))
    .join(' ')
}

function relationshipTypeLabel(type: string): string {
  return type
    .split(/[-_]/)
    .filter(Boolean)
    .join(' ')
}

function relationshipIcon(relationship: Relationship): string {
  const source =
    `${relationship.relationship_type} ${relationship.source_entity_kind} ${relationship.target_entity_kind}`.toLowerCase()

  if (source.includes('contract') || source.includes('agreement') || source.includes('legal')) {
    return 'tabler:file-text'
  }
  if (source.includes('telegram')) return 'tabler:brand-telegram'
  if (source.includes('whatsapp')) return 'tabler:brand-whatsapp'
  if (source.includes('messenger') || source.includes('chat')) return 'tabler:message-circle'
  if (source.includes('mail') || source.includes('email') || source.includes('newsletter')) return 'tabler:mail'
  if (source.includes('meeting') || source.includes('calendar')) return 'tabler:calendar-event'
  if (source.includes('task') || source.includes('obligation')) return 'tabler:checkup-list'
  if (source.includes('decision')) return 'tabler:git-branch'
  if (source.includes('address book')) return 'tabler:address-book'
  if (source.includes('organization') || source.includes('vendor') || source.includes('customer')) return 'tabler:building'
  if (source.includes('project')) return 'tabler:folder'
  if (source.includes('security') || source.includes('trust')) return 'tabler:shield-check'
  if (source.includes('operations') || source.includes('incident')) return 'tabler:tool'

  return 'tabler:hierarchy-2'
}

function entityKindLabel(kind: string, t: TranslationFunction): string {
  const labels: Record<string, string> = {
    persona: t('Persona'),
    organization: t('Organization'),
    project: t('Project'),
    channel: t('Channel'),
    mail: t('Mail'),
    chat: t('Chat')
  }
  return labels[kind] ?? kind
}

function relationshipStateLabel(state: string, t: TranslationFunction): string {
  const labels: Record<string, string> = {
    confirmed: t('Confirmed'),
    ended: t('Ended')
  }
  return labels[state] ?? relationshipTypeLabel(state)
}
