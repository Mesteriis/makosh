export type TelegramMediaCacheClass = 'avatar' | 'media'

export type TelegramMediaCacheArtifact = {
	url: string
	sizeBytes: number
}

type TelegramMediaCacheLimit = {
	maxEntries: number
	maxBytes: number
}

type TelegramMediaCacheBucket = {
	entries: Map<string, TelegramMediaCacheArtifact>
	sizeBytes: number
	limit: TelegramMediaCacheLimit
}

export class TelegramMediaMemoryCache {
	private readonly buckets: Record<TelegramMediaCacheClass, TelegramMediaCacheBucket>

	constructor(
		limits: Record<TelegramMediaCacheClass, TelegramMediaCacheLimit>,
		private readonly revoke: (url: string) => void,
	) {
		this.buckets = {
			avatar: { entries: new Map(), sizeBytes: 0, limit: limits.avatar },
			media: { entries: new Map(), sizeBytes: 0, limit: limits.media },
		}
	}

	get(cacheClass: TelegramMediaCacheClass, key: string): TelegramMediaCacheArtifact | undefined {
		const bucket = this.buckets[cacheClass]
		const artifact = bucket.entries.get(key)
		if (!artifact) return undefined
		bucket.entries.delete(key)
		bucket.entries.set(key, artifact)
		return artifact
	}

	set(cacheClass: TelegramMediaCacheClass, key: string, artifact: TelegramMediaCacheArtifact): void {
		const bucket = this.buckets[cacheClass]
		if (artifact.sizeBytes < 1 || artifact.sizeBytes > bucket.limit.maxBytes) return
		const previous = bucket.entries.get(key)
		if (previous) {
			bucket.entries.delete(key)
			bucket.sizeBytes -= previous.sizeBytes
			if (previous.url !== artifact.url) this.revoke(previous.url)
		}
		bucket.entries.set(key, artifact)
		bucket.sizeBytes += artifact.sizeBytes
		this.trim(bucket)
	}

	private trim(bucket: TelegramMediaCacheBucket): void {
		while (bucket.entries.size > bucket.limit.maxEntries || bucket.sizeBytes > bucket.limit.maxBytes) {
			const oldest = bucket.entries.entries().next().value as [string, TelegramMediaCacheArtifact] | undefined
			if (!oldest) return
			bucket.entries.delete(oldest[0])
			bucket.sizeBytes -= oldest[1].sizeBytes
			this.revoke(oldest[1].url)
		}
	}
}
