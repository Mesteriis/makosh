<script setup lang="ts">
import type {
	TelegramDiscoveryDetailRow,
	TelegramDiscoveryModel,
} from './telegramDiscoveryModel'
import './telegramDiscoveryPanel.css'

defineProps<{ model: TelegramDiscoveryModel }>()

const emit = defineEmits<{
	refreshContext: []
	search: []
	selectResult: [result: TelegramDiscoveryModel['results'][number]]
	updateQuery: [value: string]
}>()

function title(items: readonly TelegramDiscoveryDetailRow[], label: string): string {
	return `${label} · ${items.length}`
}
</script>

<template>
	<section class="telegram-discovery">
		<header>
			<div>
				<span>Operational discovery</span>
				<h2>Search & context</h2>
			</div>
			<button
				type="button"
				:disabled="!model.canQuery || model.pending"
				@click="emit('refreshContext')"
			>
				Refresh selected chat
			</button>
		</header>

		<form @submit.prevent="emit('search')">
			<label for="telegram-discovery-query">Search chats and messages</label>
			<div>
				<input
					id="telegram-discovery-query"
					type="search"
					:value="model.query"
					@input="emit('updateQuery', ($event.target as HTMLInputElement).value)"
				>
				<button type="submit" :disabled="!model.canQuery || !model.query.trim() || model.pending">
					Search
				</button>
			</div>
		</form>

		<p v-if="model.statusMessage" class="telegram-discovery__status" role="status">
			{{ model.statusMessage }}
		</p>

		<div v-if="model.chatState.length" class="telegram-discovery__badges">
			<span v-for="state in model.chatState" :key="state">{{ state }}</span>
		</div>

		<div class="telegram-discovery__columns">
			<details open>
				<summary>Search results · {{ model.results.length }}</summary>
				<button
					v-for="result in model.results"
					:key="`${result.kind}:${result.id}`"
					type="button"
					@click="emit('selectResult', result)"
				>
					<strong>{{ result.title }}</strong>
					<small>{{ result.detail }}</small>
				</button>
			</details>

			<details>
				<summary>{{ title(model.history, 'Provider history') }}</summary>
				<article v-for="item in model.history" :key="item.id">
					<strong>{{ item.title }}</strong><small>{{ item.detail }}</small>
				</article>
			</details>

			<details>
				<summary>{{ title(model.participants, 'Participants') }}</summary>
				<article v-for="item in model.participants" :key="item.id">
					<strong>{{ item.title }}</strong><small>{{ item.detail }}</small>
				</article>
			</details>

			<details>
				<summary>{{ title(model.topics, 'Topics') }}</summary>
				<article v-for="item in model.topics" :key="item.id">
					<strong>{{ item.title }}</strong><small>{{ item.detail }}</small>
				</article>
			</details>

			<details>
				<summary>{{ title(model.folders, 'Folders') }}</summary>
				<article v-for="item in model.folders" :key="item.id">
					<strong>{{ item.title }}</strong><small>{{ item.detail }}</small>
				</article>
			</details>

			<details>
				<summary>{{ title(model.operations, 'Operations') }}</summary>
				<article v-for="item in model.operations" :key="item.id">
					<strong>{{ item.title }}</strong><small>{{ item.detail }}</small>
				</article>
			</details>
		</div>
	</section>
</template>
