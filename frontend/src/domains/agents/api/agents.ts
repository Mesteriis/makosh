import { ApiClient } from '../../../platform/api/ApiClient'
import type {
	AiStatus,
	AiAgentListResponse,
	AiHubRequestAcceptedResponse,
	AiRun,
	AiRunListResponse,
	AiAnswerRequest,
	AiMeetingPrepRequest,
	AiTaskCandidateRefreshRequest
} from '../types/agents'

export async function fetchAiStatus(): Promise<AiStatus> {
	return ApiClient.instance.get<AiStatus>('/api/v1/ai/status', 'AI status request failed')
}

export async function fetchAiAgents(): Promise<AiAgentListResponse> {
	return ApiClient.instance.get<AiAgentListResponse>('/api/v1/ai/agents', 'AI agents request failed')
}

export async function fetchAiRuns(limit = 25): Promise<AiRunListResponse> {
	const params = new URLSearchParams({ limit: String(Math.trunc(limit)) })
	return ApiClient.instance.get<AiRunListResponse>(
		`/api/v1/ai/runs?${params.toString()}`,
		'AI run history request failed'
	)
}

export async function fetchAiRun(runId: string): Promise<AiRun> {
	return ApiClient.instance.get<AiRun>(
		`/api/v1/ai/runs/${encodeURIComponent(runId)}`,
		'AI run request failed'
	)
}

export async function requestAiAnswer(request: AiAnswerRequest): Promise<AiHubRequestAcceptedResponse> {
	return ApiClient.instance.post<AiHubRequestAcceptedResponse>(
		'/api/v1/ai/answers',
		request,
		'AI answer request failed'
	)
}

export async function requestAiMeetingPrep(request: AiMeetingPrepRequest): Promise<AiHubRequestAcceptedResponse> {
	return ApiClient.instance.post<AiHubRequestAcceptedResponse>(
		'/api/v1/ai/meeting-prep',
		request,
		'AI meeting prep request failed'
	)
}

export async function refreshAiTaskCandidates(
	request: AiTaskCandidateRefreshRequest
): Promise<AiHubRequestAcceptedResponse> {
	return ApiClient.instance.post<AiHubRequestAcceptedResponse>(
		'/api/v1/ai/task-candidates/refresh',
		request,
		'AI task candidate refresh request failed'
	)
}
