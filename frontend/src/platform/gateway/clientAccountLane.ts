export type ClientAccountLaneIdentity = {
	provider: string
	accountId: string
}

export type ClientAccountWorkClass = 'interactive' | 'realtime' | 'enrichment' | 'media'
export type ClientAccountLaneLifecycleState =
	| 'inactive'
	| 'hydrating'
	| 'live'
	| 'stale'
	| 'recovering'
	| 'closed'

export type ClientAccountLaneMeasurement = {
	laneKind: 'provider_account'
	provider: string
	workClass: ClientAccountWorkClass
	queueDepthAtEnqueue: number
	queueWaitMillis: number
	executionMillis: number
	outcome: 'completed' | 'failed' | 'cancelled'
}

export type ClientAccountLaneOptions = {
	now?: () => number
	onMeasurement?: (measurement: ClientAccountLaneMeasurement) => void
	onLifecycleChange?: (state: ClientAccountLaneLifecycleState) => void
	maxPendingPerWorkClass?: Partial<Record<ClientAccountWorkClass, number>>
}

type LaneJob = {
	enqueuedAt: number
	queueDepthAtEnqueue: number
	operation: (signal: AbortSignal) => Promise<unknown>
	resolve: (value: unknown) => void
	reject: (error: unknown) => void
}

type WorkQueue = {
	active: boolean
	jobs: LaneJob[]
}

export class ClientAccountLaneClosedError extends Error {
	constructor() {
		super('client_account_lane_closed')
		this.name = 'ClientAccountLaneClosedError'
	}
}

export class ClientAccountLaneOverflowError extends Error {
	readonly workClass: ClientAccountWorkClass

	constructor(workClass: ClientAccountWorkClass) {
		super(`client_account_lane_${workClass}_overflow`)
		this.name = 'ClientAccountLaneOverflowError'
		this.workClass = workClass
	}
}

const DEFAULT_MAX_PENDING_PER_WORK_CLASS: Readonly<Record<ClientAccountWorkClass, number>> = {
	interactive: 32,
	realtime: 8,
	enrichment: 8,
	media: 4,
}

/**
 * Account-local scheduler over the shared Gateway transport. Work classes are
 * independent, while each class remains bounded to one active operation.
 */
export class ClientAccountLane {
	private readonly identity: ClientAccountLaneIdentity
	private readonly now: () => number
	private readonly onMeasurement?: (measurement: ClientAccountLaneMeasurement) => void
	private readonly onLifecycleChange?: (state: ClientAccountLaneLifecycleState) => void
	private readonly maxPendingPerWorkClass: Readonly<Record<ClientAccountWorkClass, number>>
	private readonly queues = new Map<ClientAccountWorkClass, WorkQueue>()
	private readonly backgroundControllers = new Set<AbortController>()
	private closed = false
	private lifecycle: ClientAccountLaneLifecycleState = 'inactive'
	private latestInvalidationRevision = 0n
	private appliedInvalidationRevision = 0n
	private invalidationRefresh?: (revision: bigint, signal: AbortSignal) => Promise<void>
	private recoveryRefresh?: (signal: AbortSignal) => Promise<void>
	private recoveryQueued = false

	constructor(identity: ClientAccountLaneIdentity, options: ClientAccountLaneOptions = {}) {
		if (!identity.provider.trim() || !identity.accountId.trim()) {
			throw new Error('client_account_lane_identity_invalid')
		}
		this.identity = { provider: identity.provider.trim(), accountId: identity.accountId.trim() }
		this.now = options.now ?? (() => performance.now())
		this.onMeasurement = options.onMeasurement
		this.onLifecycleChange = options.onLifecycleChange
		this.maxPendingPerWorkClass = {
			...DEFAULT_MAX_PENDING_PER_WORK_CLASS,
			...options.maxPendingPerWorkClass,
		}
	}

	run<T>(
		workClass: ClientAccountWorkClass,
		operation: (signal: AbortSignal) => Promise<T>,
	): Promise<T> {
		if (this.closed) return Promise.reject(new ClientAccountLaneClosedError())
		return new Promise<T>((resolve, reject) => {
			const queue = this.queue(workClass)
			const maxPending = Math.max(1, this.maxPendingPerWorkClass[workClass])
			if (queue.jobs.length >= maxPending) {
				const overflow = new ClientAccountLaneOverflowError(workClass)
				if (workClass === 'enrichment' || workClass === 'media') {
					queue.jobs.shift()?.reject(overflow)
				} else {
					if (workClass === 'realtime') {
						this.transition('stale')
						this.cancelBackground()
					}
					reject(overflow)
					return
				}
			}
			queue.jobs.push({
				enqueuedAt: this.now(),
				queueDepthAtEnqueue: queue.jobs.length + (queue.active ? 1 : 0),
				operation,
				resolve: value => resolve(value as T),
				reject,
			})
			this.drain(workClass, queue)
		})
	}

	invalidate(
		revision: bigint,
		refresh: (revision: bigint, signal: AbortSignal) => Promise<void>,
	): void {
		if (this.closed || revision <= this.latestInvalidationRevision) return
		if (this.lifecycle === 'inactive') this.transition('hydrating')
		this.latestInvalidationRevision = revision
		this.invalidationRefresh = refresh
		if (this.queue('realtime').active || this.queue('realtime').jobs.length > 0) return
		void this.run('realtime', async (signal) => {
			while (!signal.aborted && this.appliedInvalidationRevision < this.latestInvalidationRevision) {
				const target = this.latestInvalidationRevision
				await this.invalidationRefresh?.(target, signal)
				this.appliedInvalidationRevision = target
			}
		}).catch(() => undefined)
	}

	recover(refresh: (signal: AbortSignal) => Promise<void>): void {
		if (this.closed) return
		this.recoveryRefresh = refresh
		this.recoveryQueued = true
		this.transition('stale')
		this.cancelBackground()
		this.enqueueRecoveryIfIdle()
	}

	state(): ClientAccountLaneLifecycleState {
		return this.lifecycle
	}

	cancelBackground(): void {
		for (const workClass of ['enrichment', 'media'] as const) {
			const queue = this.queue(workClass)
			for (const job of queue.jobs.splice(0)) job.reject(new ClientAccountLaneClosedError())
		}
		for (const controller of this.backgroundControllers) controller.abort()
	}

	close(): void {
		if (this.closed) return
		this.closed = true
		for (const queue of this.queues.values()) {
			for (const job of queue.jobs.splice(0)) job.reject(new ClientAccountLaneClosedError())
		}
		for (const controller of this.backgroundControllers) controller.abort()
		this.transition('closed')
	}

	private queue(workClass: ClientAccountWorkClass): WorkQueue {
		let queue = this.queues.get(workClass)
		if (!queue) {
			queue = { active: false, jobs: [] }
			this.queues.set(workClass, queue)
		}
		return queue
	}

	private drain(workClass: ClientAccountWorkClass, queue: WorkQueue): void {
		if (this.closed || queue.active) return
		const job = queue.jobs.shift()
		if (!job) return
		queue.active = true
		const controller = new AbortController()
		const background = workClass === 'enrichment' || workClass === 'media'
		if (background) this.backgroundControllers.add(controller)
		const startedAt = this.now()
		let outcome: ClientAccountLaneMeasurement['outcome'] = 'completed'
		void job.operation(controller.signal)
			.then(job.resolve, (error) => {
				outcome = controller.signal.aborted ? 'cancelled' : 'failed'
				job.reject(error)
			})
			.finally(() => {
				if (background) this.backgroundControllers.delete(controller)
				const finishedAt = this.now()
				const effectiveOutcome = controller.signal.aborted ? 'cancelled' : outcome
				this.onMeasurement?.({
					laneKind: 'provider_account',
					provider: this.identity.provider,
					workClass,
					queueDepthAtEnqueue: job.queueDepthAtEnqueue,
					queueWaitMillis: Math.max(0, startedAt - job.enqueuedAt),
					executionMillis: Math.max(0, finishedAt - startedAt),
					outcome: effectiveOutcome,
				})
				if (workClass === 'realtime') {
					this.transition(
						effectiveOutcome === 'completed' && !this.recoveryQueued ? 'live' : 'stale',
					)
				}
				queue.active = false
				if (workClass === 'realtime') this.enqueueRecoveryIfIdle()
				this.drain(workClass, queue)
			})
	}

	private enqueueRecoveryIfIdle(): void {
		if (this.closed || !this.recoveryQueued) return
		const queue = this.queue('realtime')
		if (queue.active || queue.jobs.length > 0) return
		const refresh = this.recoveryRefresh
		if (!refresh) return
		this.recoveryQueued = false
		this.transition('recovering')
		void this.run('realtime', refresh).catch(() => undefined)
	}

	private transition(state: ClientAccountLaneLifecycleState): void {
		if (this.lifecycle === state) return
		this.lifecycle = state
		this.onLifecycleChange?.(state)
	}
}

export class ClientAccountLaneRegistry {
	private readonly lanes = new Map<string, ClientAccountLane>()
	private readonly options: ClientAccountLaneOptions

	constructor(options: ClientAccountLaneOptions = {}) {
		this.options = options
	}

	get(identity: ClientAccountLaneIdentity): ClientAccountLane {
		const key = laneKey(identity)
		let lane = this.lanes.get(key)
		if (!lane) {
			lane = new ClientAccountLane(identity, this.options)
			this.lanes.set(key, lane)
		}
		return lane
	}

	close(identity: ClientAccountLaneIdentity): void {
		const key = laneKey(identity)
		this.lanes.get(key)?.close()
		this.lanes.delete(key)
	}
}

function laneKey(identity: ClientAccountLaneIdentity): string {
	return `${identity.provider.length}:${identity.provider}${identity.accountId.length}:${identity.accountId}`
}

let sharedRegistry: ClientAccountLaneRegistry | undefined

function recordSharedLaneMeasurement(measurement: ClientAccountLaneMeasurement): void {
	if (typeof performance !== 'undefined' && typeof performance.measure === 'function') {
		try {
			performance.measure('makosh.client_account_lane', {
				start: 0,
				duration: measurement.queueWaitMillis + measurement.executionMillis,
				detail: measurement,
			})
		} catch {
			// Diagnostics must never affect provider work.
		}
	}
	if (import.meta.env.DEV) console.debug('client_account_lane.span', measurement)
}

export function getClientAccountLaneRegistry(): ClientAccountLaneRegistry {
	sharedRegistry ??= new ClientAccountLaneRegistry({
		onMeasurement: recordSharedLaneMeasurement,
	})
	return sharedRegistry
}

export function resetClientAccountLaneRegistryForTests(): void {
	sharedRegistry = undefined
}
