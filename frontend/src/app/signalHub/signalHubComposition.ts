import { getConsistencyQueryClient } from '../../platform/connect/consistencyClient'
import { getGraphQueryClient } from '../../platform/connect/graphClient'
import { getMemoryQueryClient } from '../../platform/connect/memoryClient'
import { getRiskQueryClient } from '../../platform/connect/riskClient'
import { getSearchQueryClient } from '../../platform/connect/searchClient'
import { getTimelineQueryClient } from '../../platform/connect/timelineClient'

export type SignalHubReadiness = Readonly<{
  searchGeneration: bigint
  timelineGeneration: bigint
  graphGeneration: bigint
  memoryGeneration: bigint
  consistencyGeneration: bigint
  riskGeneration: bigint
}>

export async function loadSignalHubReadiness(): Promise<SignalHubReadiness> {
  const [search, timeline, graph, memory, consistency, risk] = await Promise.all([
    getSearchQueryClient().getStatus({ logicalOwnerId: '' }),
    getTimelineQueryClient().getStatus({ logicalOwnerId: '' }),
    getGraphQueryClient().getStatus({ logicalOwnerId: '' }),
    getMemoryQueryClient().getStatus({ logicalOwnerId: '' }),
    getConsistencyQueryClient().getStatus({ logicalOwnerId: '' }),
    getRiskQueryClient().getStatus({ logicalOwnerId: '' }),
  ])
  return {
    searchGeneration: search.activeGeneration,
    timelineGeneration: timeline.activeGeneration,
    graphGeneration: graph.activeGeneration,
    memoryGeneration: memory.activeGeneration,
    consistencyGeneration: consistency.activeGeneration,
    riskGeneration: risk.activeGeneration,
  }
}
