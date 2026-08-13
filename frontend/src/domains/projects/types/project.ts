import type {
  ProjectOutcomeStateV1,
  ProjectReferenceKindV1,
  ProjectStateV1,
  TimestampV1
} from '../../../gen/makosh/projects/client/v1/projects_pb'

export interface ProjectDraft {
  name: string
  description: string
  startAt?: TimestampV1
  targetAt?: TimestampV1
}

export interface ProjectOutcomeDraft {
  title: string
  description: string
  targetAt?: TimestampV1
}

export interface ProjectReferenceDraft {
  kind: ProjectReferenceKindV1
  publicId: Uint8Array
  label: string
}

export type ProjectLifecycleState = ProjectStateV1
export type ProjectExpectedOutcomeState = ProjectOutcomeStateV1
