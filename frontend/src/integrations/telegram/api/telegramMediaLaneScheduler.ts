export type TelegramMediaLanePriority = 'interactive' | 'background'

type QueuedTelegramMediaTask<T> = {
	laneKey: string
	scopeKey: string
	priority: TelegramMediaLanePriority
	isScopeActive: (scopeKey: string) => boolean
	run: () => Promise<T>
	resolve: (value: T) => void
	reject: (error: unknown) => void
}

export class TelegramMediaLaneScheduler {
	private readonly queue: Array<QueuedTelegramMediaTask<unknown>> = []
	private readonly activeByLane = new Map<string, number>()
	private activeTotal = 0

	constructor(
		private readonly maximumTotal: number,
		private readonly maximumPerLane: number,
	) {
		if (!Number.isSafeInteger(maximumTotal) || maximumTotal < 1) {
			throw new RangeError('Telegram media total concurrency must be positive')
		}
		if (!Number.isSafeInteger(maximumPerLane) || maximumPerLane < 1) {
			throw new RangeError('Telegram media lane concurrency must be positive')
		}
	}

	schedule<T>(task: {
		laneKey: string
		scopeKey: string
		priority: TelegramMediaLanePriority
		isScopeActive: (scopeKey: string) => boolean
		run: () => Promise<T>
	}): Promise<T> {
		return new Promise<T>((resolve, reject) => {
			this.queue.push({ ...task, resolve, reject } as QueuedTelegramMediaTask<unknown>)
			this.drain()
		})
	}

	notifyScopeChanged(): void {
		this.drain()
	}

	private drain(): void {
		while (this.activeTotal < this.maximumTotal) {
			const interactiveIndex = this.queue.findIndex(item =>
				item.priority === 'interactive' && this.laneHasCapacity(item.laneKey))
			const nextIndex = interactiveIndex >= 0
				? interactiveIndex
				: this.queue.findIndex(item => this.laneHasCapacity(item.laneKey))
			if (nextIndex < 0) return
			const [next] = this.queue.splice(nextIndex, 1)
			if (!next) return
			if (!next.isScopeActive(next.scopeKey)) {
				next.reject(scopeChangedError())
				continue
			}
			this.activeTotal += 1
			this.activeByLane.set(next.laneKey, (this.activeByLane.get(next.laneKey) ?? 0) + 1)
			void next.run()
				.then(next.resolve, next.reject)
				.finally(() => {
					this.activeTotal -= 1
					const remaining = (this.activeByLane.get(next.laneKey) ?? 1) - 1
					if (remaining === 0) this.activeByLane.delete(next.laneKey)
					else this.activeByLane.set(next.laneKey, remaining)
					this.drain()
				})
		}
	}

	private laneHasCapacity(laneKey: string): boolean {
		return (this.activeByLane.get(laneKey) ?? 0) < this.maximumPerLane
	}
}

function scopeChangedError(): Error {
	const error = new Error('Telegram media request was superseded')
	error.name = 'AbortError'
	return error
}
