import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import type {
	AiStatus,
	AiAgent,
	AiRun,
	AiAnswerResponse,
	AiMeetingPrepResponse,
	AiTaskCandidateRefreshResponse,
	AiCitation,
	OwnerPersona,
	AgentCard
} from '../types/agents'
import {
	requestAiAnswer,
	requestAiMeetingPrep,
	refreshAiTaskCandidates,
	fetchAiRun,
	fetchAiRuns
} from '../api/agents'

export const useAgentsStore = defineStore('agents-ui', () => {
	const aiStatus = ref<AiStatus | null>(null)
	const aiAgents = ref<AiAgent[]>([])
	const aiRuns = ref<AiRun[]>([])
	const ownerPersona = ref<OwnerPersona | null>(null)
	const aiError = ref('')
	const isAiLoading = ref(false)
	const isAiAnswerSubmitting = ref(false)
	const isAiMeetingPrepSubmitting = ref(false)
	const isAiTaskRefreshSubmitting = ref(false)
	const selectedAgentIndex = ref(0)
	const aiQuestion = ref('What does the local memory say about Макошь V3?')
	const aiMeetingTopic = ref('Prepare a V3 implementation review brief')
	const aiTaskQuery = ref('Find open task candidates from local messages and documents')
	const aiAnswerResult = ref<AiAnswerResponse | null>(null)
	const aiMeetingPrepResult = ref<AiMeetingPrepResponse | null>(null)
	const aiTaskRefreshResult = ref<AiTaskCandidateRefreshResponse | null>(null)

	const agentCards = computed<AgentCard[]>(() =>
		aiAgents.value.map((agent) => agentCardView(agent, aiRuns.value))
	)

	const selectedAgent = computed<AgentCard | null>(() =>
		agentCards.value[selectedAgentIndex.value] ?? agentCards.value[0] ?? null
	)

	function setWorkspace(data: {
		agents: AiAgent[]
		runs: AiRun[]
		status: AiStatus | null
		ownerPersona: OwnerPersona | null
		error: string
	}) {
		aiAgents.value = data.agents
		aiRuns.value = data.runs
		aiStatus.value = data.status
		ownerPersona.value = data.ownerPersona
		aiError.value = data.error
		if (selectedAgentIndex.value >= aiAgents.value.length) {
			selectedAgentIndex.value = 0
		}
	}

	function setLoading(v: boolean) {
		isAiLoading.value = v
	}

	function selectAgent(index: number) {
		selectedAgentIndex.value = index
	}

	async function submitAiAnswer() {
		const query = aiQuestion.value.trim()
		if (!query || isAiAnswerSubmitting.value) return
		isAiAnswerSubmitting.value = true
		try {
			const result = await requestAiAnswer({
				command_id: `ai-answer-${crypto.randomUUID()}`,
				query,
				agent_id: selectedAgent.value?.agentId ?? 'MNEMOSYNE'
			})
			const run = await waitForAiRun(result.run_id)
			aiAnswerResult.value = aiAnswerResponseFromRun(run)
			aiError.value = ''
			await loadAiRunsOnly()
		} catch (error) {
			aiAnswerResult.value = null
			aiError.value = error instanceof Error ? error.message : 'Unknown AI answer error'
		} finally {
			isAiAnswerSubmitting.value = false
		}
	}

	async function prepareAiBrief() {
		const topic = aiMeetingTopic.value.trim()
		if (!topic || isAiMeetingPrepSubmitting.value) return
		isAiMeetingPrepSubmitting.value = true
		try {
			const result = await requestAiMeetingPrep({
				command_id: `ai-meeting-prep-${crypto.randomUUID()}`,
				topic
			})
			const run = await waitForAiRun(result.run_id)
			aiMeetingPrepResult.value = aiMeetingPrepResponseFromRun(run)
			aiError.value = ''
			await loadAiRunsOnly()
		} catch (error) {
			aiMeetingPrepResult.value = null
			aiError.value = error instanceof Error ? error.message : 'Unknown AI meeting prep error'
		} finally {
			isAiMeetingPrepSubmitting.value = false
		}
	}

	async function refreshTasksFromAi() {
		const query = aiTaskQuery.value.trim()
		if (!query || isAiTaskRefreshSubmitting.value) return
		isAiTaskRefreshSubmitting.value = true
		try {
			const result = await refreshAiTaskCandidates({
				command_id: `ai-task-refresh-${crypto.randomUUID()}`,
				query
			})
			const run = await waitForAiRun(result.run_id)
			aiTaskRefreshResult.value = aiTaskRefreshResponseFromRun(run)
			aiError.value = ''
			await loadAiRunsOnly()
		} catch (error) {
			aiTaskRefreshResult.value = null
			aiError.value = error instanceof Error ? error.message : 'Unknown AI task refresh error'
		} finally {
			isAiTaskRefreshSubmitting.value = false
		}
	}

	async function loadAiRunsOnly() {
		try {
			const response = await fetchAiRuns(25)
			aiRuns.value = response.items
		} catch (error) {
			aiError.value = error instanceof Error ? error.message : 'Unknown AI run history error'
		}
	}

	async function waitForAiRun(runId: string) {
		const maxAttempts = 40
		for (let attempt = 0; attempt < maxAttempts; attempt += 1) {
			const run = await fetchAiRun(runId)
			if (run.status === 'completed') return run
			if (run.status === 'failed') {
				throw new Error(run.error_summary ?? 'AI run failed')
			}
			await delay(500)
		}
		throw new Error('AI run did not complete in time')
	}

	return {
		aiStatus,
		aiAgents,
		aiRuns,
		ownerPersona,
		aiError,
		isAiLoading,
		isAiAnswerSubmitting,
		isAiMeetingPrepSubmitting,
		isAiTaskRefreshSubmitting,
		selectedAgentIndex,
		aiQuestion,
		aiMeetingTopic,
		aiTaskQuery,
		aiAnswerResult,
		aiMeetingPrepResult,
		aiTaskRefreshResult,
		agentCards,
		selectedAgent,
		setWorkspace,
		setLoading,
		selectAgent,
		submitAiAnswer,
		prepareAiBrief,
		refreshTasksFromAi,
		loadAiRunsOnly
	}
})

function isRecord(value: unknown): value is Record<string, unknown> {
	return typeof value === 'object' && value !== null && !Array.isArray(value)
}

function agentCardView(agent: AiAgent, aiRuns: AiRun[]): AgentCard {
	const visual = agentVisual(agent.agent_id)
	const runs = aiRuns.filter((run) => run.agent_id === agent.agent_id)
	const completed = runs.filter((run) => run.status === 'completed').length
	const success = runs.length > 0 ? Math.round((completed / runs.length) * 100) : 0

	return {
		agentId: agent.agent_id,
		name: agent.persona_email ?? aiAgentPersonaEmail(agent.agent_id),
		summary: agent.role,
		icon: visual.icon,
		tasks: runs.length,
		success,
		status: agent.status,
		tone: visual.tone,
		model: agent.default_model
	}
}

function aiAgentPersonaEmail(agentId: string): string {
	return `${agentId.trim().toLowerCase()}@sh-inc.ru`
}

function agentVisual(agentId: string): { icon: string; tone: string } {
	switch (agentId) {
		case 'HESTIA':
			return { icon: 'tabler:calendar-stats', tone: 'mint' }
		case 'MAKOSH':
			return { icon: 'tabler:route', tone: 'blue' }
		case 'MNEMOSYNE':
			return { icon: 'tabler:database-search', tone: 'purple' }
		case 'ATHENA':
			return { icon: 'tabler:target-arrow', tone: 'amber' }
		default:
			return { icon: 'tabler:sparkles', tone: 'cyan' }
	}
}

export function aiRuntimeSummary(aiStatus: AiStatus | null, isAiLoading: boolean): string {
	if (!aiStatus) return isAiLoading ? 'Loading' : 'Unknown'
	return aiStatus.status === 'ok' ? 'Ready' : 'Unavailable'
}

export function aiModelSummary(aiStatus: AiStatus | null): string {
	if (!aiStatus) return 'No status'
	return `${aiStatus.chat_model} / ${aiStatus.embedding_model}`
}

export function runStatusLabel(run: AiRun): string {
	if (run.status === 'completed') return 'Completed'
	if (run.status === 'failed') return 'Failed'
	return 'Requested'
}

export function formatDuration(durationMs: number | null | undefined): string {
	if (durationMs == null) return 'n/a'
	if (durationMs < 1000) return `${durationMs} ms`
	return `${(durationMs / 1000).toFixed(1)} s`
}

export function formatDateTime(date: string): string {
	const d = new Date(date)
	if (Number.isNaN(d.getTime())) return 'Invalid date'
	return new Intl.DateTimeFormat('en', {
		month: 'short',
		day: 'numeric',
		hour: '2-digit',
		minute: '2-digit'
	}).format(d)
}

export function safeCitations(value: unknown): AiCitation[] {
	if (!Array.isArray(value)) return []
	return value.filter(isAiCitation)
}

function isAiCitation(value: unknown): value is AiCitation {
	return isRecord(value) &&
		typeof value.source_kind === 'string' &&
		typeof value.source_id === 'string' &&
		typeof value.title === 'string' &&
		typeof value.excerpt === 'string'
}

function aiAnswerResponseFromRun(run: AiRun): AiAnswerResponse {
	return {
		run_id: run.run_id,
		agent_id: run.agent_id,
		status: run.status,
		answer: run.answer ?? '',
		citations: safeCitations(run.citations),
		model: run.chat_model,
		embedding_model: run.embedding_model,
		created_at: run.created_at,
		duration_ms: run.duration_ms ?? 0
	}
}

function aiMeetingPrepResponseFromRun(run: AiRun): AiMeetingPrepResponse {
	return {
		run_id: run.run_id,
		agent_id: run.agent_id,
		status: run.status,
		briefing: run.answer ?? '',
		citations: safeCitations(run.citations),
		model: run.chat_model,
		embedding_model: run.embedding_model,
		created_at: run.created_at,
		duration_ms: run.duration_ms ?? 0
	}
}

function aiTaskRefreshResponseFromRun(run: AiRun): AiTaskCandidateRefreshResponse {
	const createdCount = Number.parseInt(run.answer?.match(/\d+/)?.[0] ?? '0', 10)
	return {
		run_id: run.run_id,
		agent_id: run.agent_id,
		status: run.status,
		created_count: Number.isFinite(createdCount) ? createdCount : 0,
		citations: safeCitations(run.citations),
		model: run.chat_model,
		embedding_model: run.embedding_model,
		created_at: run.created_at,
		duration_ms: run.duration_ms ?? 0
	}
}

function delay(ms: number) {
	return new Promise((resolve) => {
		globalThis.setTimeout(resolve, ms)
	})
}
