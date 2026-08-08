# Задача для DeepSeek: обновить русскую Obsidian wiki

## Safety instructions / Инструкции безопасности

- Do not print, infer, summarize, or request secrets. / Не печатай, не выводи, не пересказывай и не запрашивай секреты.
- Treat `.env`, credential, token, key, certificate, and private paths as redacted even if referenced. / Считай `.env`, учетные данные, токены, ключи, сертификаты и приватные пути редактированными.
- Keep code identifiers, file paths, commands, package names, API names, and ADR titles exactly as written. / Сохраняй идентификаторы кода, пути, команды, имена пакетов, API и названия ADR без изменений.
- Write wiki prose in Russian and keep Markdown Obsidian-compatible. / Пиши текст wiki на русском и сохраняй совместимость с Obsidian Markdown.
- Do not invent source facts. If the context is insufficient, state that explicitly. / Не выдумывай факты об исходниках. Если контекста недостаточно, напиши это явно.
- Every behavioral statement in proposed wiki pages must be directly supported by the embedded source text. / Каждое утверждение о поведении в предлагаемых wiki-страницах должно напрямую подтверждаться встроенным текстом исходников.
- Do not infer semantics for profiles, flags, annotations, environment variables, or framework conventions unless this context pack explicitly defines them. / Не выводи семантику профилей, флагов, аннотаций, переменных окружения или framework-конвенций, если этот context pack явно её не определяет.
- Do not add external background knowledge about tools, frameworks, or CLIs. / Не добавляй внешние справочные знания об инструментах, framework или CLI.
- When only a command or config value is visible, document only the literal command or value. For deeper meaning, write only that it is not confirmed by this context. / Когда видна только команда или значение конфигурации, документируй только буквальную команду или значение. Для более глубокого смысла пиши только, что он не подтвержден этим контекстом.
- Do not name likely related files unless they are embedded in this context pack. / Не называй вероятные связанные файлы, если они не встроены в этот context pack.
- Use only the embedded Source Files section below. Do not call tools, read files, inspect the filesystem, or access MCP/web resources. / Используй только встроенный ниже раздел Source Files. Не вызывай tools, не читай файлы, не инспектируй файловую систему и не обращайся к MCP/web ресурсам.
- If a referenced path or wiki page is not embedded in this context pack, report insufficient context instead of trying to open it. / Если упомянутый путь или wiki-страница не встроены в этот context pack, укажи недостаток контекста вместо попытки открыть файл.

## Chunk details / Детали чанка

- Chunk ID / ID чанка: `137-other-frontend-part-010`
- Group / Группа: `frontend`
- Role / Роль: `other`
- Status / Статус: `pending`
- Repository / Репозиторий: `/Users/avm/projects/Personal/makosh`
- Wiki path / Путь wiki: `/Users/avm/projects/Personal/makosh/docs/wiki`
- Metadata path / Путь metadata: `/Users/avm/projects/Personal/makosh/docs/wiki/_meta`
- Plan generated at / План создан: `2026-06-28T19:48:55Z`
- Per-file source limit / Лимит источника на файл: `12000` characters

## Target pages / Целевые страницы

- `components/frontend.md`

## Required Output / Требуемый результат

Return one Markdown response with these sections and no extra wrapper text. / Верни один Markdown-ответ с этими разделами и без дополнительной обертки.

### Summary / Резюме

Briefly describe what should change in the Russian wiki and why. / Кратко опиши, что нужно изменить в русской wiki и почему.

### Proposed pages / Предлагаемые страницы

For each target page, provide the wiki-relative path and full proposed Obsidian-compatible Markdown content. / Для каждой целевой страницы укажи путь относительно wiki и полный предложенный Markdown, совместимый с Obsidian.

### Source coverage / Покрытие источников

List each source file and the facts from it that the proposed pages cover. / Перечисли каждый исходный файл и факты из него, покрытые предложенными страницами.

### Drift candidates / Кандидаты на drift

List possible code/docs/ADR drift found in this chunk, or state that none is visible from the provided context. / Перечисли возможные расхождения кода, документации и ADR в этом чанке либо укажи, что из данного контекста они не видны.

## Source Files / Исходные файлы

### `frontend/src/domains/settings/components/SignalHubOperationsTab.vue`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/domains/settings/components/SignalHubOperationsTab.vue`
- Size bytes / Размер в байтах: `14296`
- Included characters / Включено символов: `12000`
- Truncated / Обрезано: `yes`

```text
<script setup lang="ts">
import { computed } from 'vue'
import Icon from '../../../shared/ui/Icon.vue'
import { useI18n } from '../../../platform/i18n'
import type { SignalHubSettingsController } from './useSignalHubSettingsController'
import {
  connectionLabel,
  formatConnectionTimeline,
  formatHealthEvidence,
  formatHealthStatus,
  formatRuntimeError,
  formatRuntimeTimeline,
  formatSettingsSummary,
  healthTone,
  runtimeTone,
  sourceIconForCode,
  statusTone
} from './signalHubSettingsPresentation'

const props = defineProps<{ state: SignalHubSettingsController }>()
const { t } = useI18n()

const sources = computed(() => props.state.sources.value)
const connections = computed(() => props.state.connections.value)
const runtimeStates = computed(() => props.state.runtimeStates.value)
const healthItems = computed(() => props.state.healthItems.value)
const replayRequests = computed(() => props.state.replayRequests.value)
const connectionCapableSources = computed(() => props.state.connectionCapableSources.value)
const replayScopedConnections = computed(() => props.state.replayScopedConnections.value)
const replayTargetConsumers = computed(() => props.state.replayTargetConsumers.value)
</script>

<template>
  <div v-if="state.activeTab.value === 'connections'" class="signal-table-layout">
    <div class="policy-layout">
      <form class="policy-form" @submit.prevent="state.handleCreateConnection">
        <label>
          <span>{{ t('Source') }}</span>
          <select v-model="state.connectionSourceCode.value" class="makosh-select-control">
            <option v-for="source in connectionCapableSources" :key="source.code" :value="source.code">
              {{ source.display_name }}
            </option>
          </select>
        </label>
        <label>
          <span>{{ t('Display Name') }}</span>
          <input v-model="state.connectionDisplayName.value" class="makosh-input-control" type="text" :placeholder="t('Connection display name')" />
        </label>
        <label>
          <span>{{ t('Profile') }}</span>
          <input v-model="state.connectionProfile.value" class="makosh-input-control" type="text" :placeholder="t('Connection profile')" />
        </label>
        <button type="submit" class="makosh-btn" :disabled="state.isCreatingConnection.value">
          <Icon icon="tabler:plug-connected" />
          {{ state.isCreatingConnection.value ? t('Creating') : t('Create Connection') }}
        </button>
      </form>

      <div v-if="connections.length === 0" class="empty-panel fill">{{ t('No source connections yet.') }}</div>
      <div v-else class="signal-table">
        <div v-for="connection in connections" :key="connection.id" class="signal-table-row signal-runtime-row">
          <div class="signal-table-main">
            <Icon :icon="sourceIconForCode(sources, connection.source_code)" />
            <span>
              <strong>{{ connection.display_name }}</strong>
              <em>{{ connection.source_code }} / {{ connection.profile ?? t('No profile') }}</em>
              <small class="signal-detail-text">{{ formatSettingsSummary(t, connection) }}</small>
              <small class="signal-detail-text">{{ formatConnectionTimeline(t, connection) }}</small>
            </span>
          </div>
          <b class="signal-pill" :data-tone="statusTone(connection.status)">{{ connection.status }}</b>
          <div class="runtime-actions">
            <button type="button" class="makosh-btn makosh-btn--outline makosh-btn--compact" :disabled="state.isUpdatingConnection.value || connection.status === 'connected'" @click="state.handleSetConnectionStatus(connection.id, 'connected')">{{ t('Connect') }}</button>
            <button type="button" class="makosh-btn makosh-btn--outline makosh-btn--compact" :disabled="state.isUpdatingConnection.value || connection.status === 'paused'" @click="state.handleSetConnectionStatus(connection.id, 'paused')">{{ t('Pause') }}</button>
            <button type="button" class="makosh-btn makosh-btn--outline makosh-btn--compact" :disabled="state.isUpdatingConnection.value || connection.status === 'muted'" @click="state.handleSetConnectionStatus(connection.id, 'muted')">{{ t('Mute') }}</button>
            <button type="button" class="makosh-btn makosh-btn--outline makosh-btn--compact" :disabled="state.isUpdatingConnection.value || connection.status === 'disabled'" @click="state.handleSetConnectionStatus(connection.id, 'disabled')">{{ t('Disable') }}</button>
            <button type="button" class="makosh-btn makosh-btn--outline makosh-btn--compact" :disabled="state.isUpdatingConnection.value || connection.status === 'removed'" @click="state.handleRemoveConnection(connection.id)">{{ t('Remove') }}</button>
          </div>
        </div>
      </div>
    </div>
  </div>

  <div v-else-if="state.activeTab.value === 'runtime'" class="signal-table-layout">
    <div v-if="runtimeStates.length === 0" class="empty-panel fill">{{ t('No runtime controls yet.') }}</div>
    <div v-else class="signal-table">
      <div v-for="runtime in runtimeStates" :key="runtime.id" class="signal-table-row signal-runtime-row">
        <div class="signal-table-main">
          <Icon :icon="sourceIconForCode(sources, runtime.source_code)" />
          <span>
            <strong>{{ runtime.runtime_kind }}</strong>
            <em>{{ runtime.source_code }}</em>
            <small class="signal-detail-text">{{ formatRuntimeTimeline(t, runtime) }}</small>
            <small v-if="formatRuntimeError(t, runtime)" class="signal-detail-text signal-detail-text--bad">{{ formatRuntimeError(t, runtime) }}</small>
          </span>
        </div>
        <b class="signal-pill" :data-tone="runtimeTone(runtime.state)">{{ runtime.state }}</b>
        <div class="runtime-actions">
          <button type="button" class="makosh-btn makosh-btn--outline makosh-btn--compact" :disabled="state.isUpdatingRuntime.value || runtime.state === 'running'" @click="state.handleSetRuntimeState(runtime, 'running')">{{ t('Run') }}</button>
          <button type="button" class="makosh-btn makosh-btn--outline makosh-btn--compact" :disabled="state.isUpdatingRuntime.value || runtime.state === 'paused'" @click="state.handleSetRuntimeState(runtime, 'paused')">{{ t('Pause') }}</button>
          <button type="button" class="makosh-btn makosh-btn--outline makosh-btn--compact" :disabled="state.isUpdatingRuntime.value || runtime.state === 'muted'" @click="state.handleSetRuntimeState(runtime, 'muted')">{{ t('Mute') }}</button>
          <button type="button" class="makosh-btn makosh-btn--outline makosh-btn--compact" :disabled="state.isUpdatingRuntime.value || runtime.state === 'stopped'" @click="state.handleSetRuntimeState(runtime, 'stopped')">{{ t('Stop') }}</button>
        </div>
      </div>
    </div>
  </div>

  <div v-else-if="state.activeTab.value === 'health'" class="signal-table-layout">
    <div v-if="healthItems.length === 0" class="empty-panel fill">{{ t('No health records yet.') }}</div>
    <div v-else class="signal-table">
      <div v-for="item in healthItems" :key="item.id" class="signal-table-row">
        <div class="signal-table-main">
          <Icon :icon="sourceIconForCode(sources, item.source_code)" />
          <span>
            <strong>{{ item.summary }}</strong>
            <em>{{ item.source_code }}</em>
            <small class="signal-detail-text">{{ formatHealthStatus(t, connections, item) }}</small>
            <small v-if="formatHealthEvidence(t, item)" class="signal-detail-text">{{ formatHealthEvidence(t, item) }}</small>
          </span>
        </div>
        <b class="signal-pill" :data-tone="healthTone(item.level)">{{ item.level }}</b>
        <div class="runtime-actions">
          <small>{{ item.next_retry_at ? `${t('Retry')} ${item.next_retry_at}` : item.last_ok_at ?? item.last_failure_at ?? t('No heartbeat') }}</small>
          <button type="button" class="makosh-btn makosh-btn--outline makosh-btn--compact" :disabled="state.isRunningHealthCheck.value" @click="state.handleRunHealthCheck(item.source_code, item.connection_id)">
            {{ state.isRunningHealthCheck.value ? t('Checking') : t('Run Check') }}
          </button>
        </div>
      </div>
    </div>
  </div>

  <div v-else-if="state.activeTab.value === 'replay'" class="signal-table-layout">
    <div class="policy-layout">
      <form class="policy-form" @submit.prevent="state.handleCreateReplayRequest">
        <label>
          <span>{{ t('Source') }}</span>
          <select v-model="state.replaySourceCode.value" class="makosh-select-control">
            <option value="">{{ t('All sources') }}</option>
            <option v-for="source in sources.filter((item) => item.supports_replay)" :key="source.code" :value="source.code">
              {{ source.display_name }}
            </option>
          </select>
        </label>
        <label>
          <span>{{ t('Connection') }}</span>
          <select v-model="state.replayConnectionId.value" class="makosh-select-control">
            <option value="">{{ t('All source connections') }}</option>
            <option v-for="connection in replayScopedConnections" :key="connection.id" :value="connection.id">
              {{ connection.display_name }} / {{ connection.status }}
            </option>
          </select>
        </label>
        <label>
          <span>{{ t('Event Pattern') }}</span>
          <input v-model="state.replayEventPattern.value" class="makosh-input-control" type="text" :placeholder="t('signal.raw.telegram.*')" />
        </label>
        <label>
          <span>{{ t('Selector Mode') }}</span>
          <select v-model="state.replaySelectorMode.value" class="makosh-select-control">
            <option value="all">{{ t('Whole pattern') }}</option>
            <option value="position">{{ t('Event log position') }}</option>
            <option value="time">{{ t('Occurred time') }}</option>
          </select>
        </label>
        <div v-if="state.replaySelectorMode.value === 'position'" class="replay-selector-grid">
          <label><span>{{ t('From Position') }}</span><input v-model="state.replayFromPosition.value" class="makosh-input-control" type="text" inputmode="numeric" placeholder="10" /></label>
          <label><span>{{ t('To Position') }}</span><input v-model="state.replayToPosition.value" class="makosh-input-control" type="text" inputmode="numeric" placeholder="20" /></label>
        </div>
        <div v-else-if="state.replaySelectorMode.value === 'time'" class="replay-selector-grid">
          <label><span>{{ t('From Time') }}</span><input v-model="state.replayFromTime.value" class="makosh-input-control" type="text" placeholder="2026-06-23T00:00:00Z" /></label>
          <label><span>{{ t('To Time') }}</span><input v-model="state.replayToTime.value" class="makosh-input-control" type="text" placeholder="2026-06-23T01:00:00Z" /></label>
        </div>
        <label>
          <span>{{ t('Target Consumer') }}</span>
          <select v-model="state.replayTargetConsumer.value" class="makosh-select-control">
            <option value="">{{ t('All downstream consumers') }}</option>
            <option v-for="runtimeKind in replayTargetConsumers" :key="runtimeKind" :value="runtimeKind">{{ runtimeKind }}</option>
          </select>
        </label>
        <label>
          <span>{{ t('Target Projection') }}</span>
          <select v-model="state.replayTargetProjection.value" class="makosh-select-control">
            <option value="">{{ t('No projection rebuild') }}</option>
            <option value="communication_messages">{{ t('Communications accepted-signal projection') }}</option>
            <option value="person_derived_evidence">{{ t('Person derived-evidence projection') }}</option>
            <option value="project_link_review_effects">{{ t('Project link-review effects projection') }}</option>
            <option value="timeline_event_log">{{ t('Timeline event-log projection') }}</
```
_Source file truncated after 12000 characters. / Исходный файл обрезан после 12000 символов._

### `frontend/src/domains/settings/components/SignalHubProfilesPoliciesTab.vue`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/domains/settings/components/SignalHubProfilesPoliciesTab.vue`
- Size bytes / Размер в байтах: `13182`
- Included characters / Включено символов: `12000`
- Truncated / Обрезано: `yes`

```text
<script setup lang="ts">
import { computed } from 'vue'
import Icon from '../../../shared/ui/Icon.vue'
import { useI18n } from '../../../platform/i18n'
import type { SignalHubSettingsController } from './useSignalHubSettingsController'
import { policyTargetLabel, profilePolicyLabel } from './signalHubSettingsPresentation'

const props = defineProps<{ state: SignalHubSettingsController }>()
const { t } = useI18n()

const sources = computed(() => props.state.sources.value)
const connections = computed(() => props.state.connections.value)
const profiles = computed(() => props.state.profiles.value)
const policies = computed(() => props.state.policies.value)
const selectedProfile = computed(() => props.state.selectedProfile.value)
const connectionCapableSources = computed(() => props.state.connectionCapableSources.value)
const profileScopeConnections = computed(() => props.state.profileScopeConnections.value)
const policyScopeConnections = computed(() => props.state.policyScopeConnections.value)
</script>

<template>
  <template v-if="state.activeTab.value === 'profiles'">
    <div class="policy-layout">
      <form class="policy-form" @submit.prevent="state.handleSaveProfile">
        <label>
          <span>{{ t('Profile Code') }}</span>
          <input
            v-model="state.profileCodeInput.value"
            class="makosh-input-control"
            type="text"
            :placeholder="t('quiet_hours')"
            :disabled="Boolean(selectedProfile)"
          />
        </label>
        <label>
          <span>{{ t('Display Name') }}</span>
          <input
            v-model="state.profileDisplayNameInput.value"
            class="makosh-input-control"
            type="text"
            :placeholder="t('Quiet Hours')"
            :disabled="selectedProfile?.is_system"
          />
        </label>
        <label>
          <span>{{ t('Description') }}</span>
          <input
            v-model="state.profileDescriptionInput.value"
            class="makosh-input-control"
            type="text"
            :placeholder="t('What this profile changes')"
            :disabled="selectedProfile?.is_system"
          />
        </label>
        <div class="replay-selector-grid">
          <label>
            <span>{{ t('Scope') }}</span>
            <select v-model="state.profilePolicyScope.value" class="makosh-select-control" :disabled="selectedProfile?.is_system">
              <option value="event_pattern">{{ t('Event Pattern') }}</option>
              <option value="source">{{ t('Source') }}</option>
              <option value="connection">{{ t('Connection') }}</option>
              <option value="global">{{ t('Global') }}</option>
            </select>
          </label>
          <label>
            <span>{{ t('Mode') }}</span>
            <select v-model="state.profilePolicyMode.value" class="makosh-select-control" :disabled="selectedProfile?.is_system">
              <option value="paused">{{ t('Pause') }}</option>
              <option value="muted">{{ t('Mute') }}</option>
              <option value="disabled">{{ t('Disable') }}</option>
              <option value="enabled">{{ t('Allow') }}</option>
            </select>
          </label>
        </div>
        <label v-if="state.profilePolicyScope.value === 'source' || state.profilePolicyScope.value === 'connection'">
          <span>{{ t('Source') }}</span>
          <select v-model="state.profilePolicySourceCode.value" class="makosh-select-control" :disabled="selectedProfile?.is_system">
            <option
              v-for="source in state.profilePolicyScope.value === 'connection' ? connectionCapableSources : sources"
              :key="source.code"
              :value="source.code"
            >
              {{ source.display_name }}
            </option>
          </select>
        </label>
        <label v-if="state.profilePolicyScope.value === 'connection'">
          <span>{{ t('Connection') }}</span>
          <select v-model="state.profilePolicyConnectionId.value" class="makosh-select-control" :disabled="selectedProfile?.is_system">
            <option value="">{{ t('Select connection') }}</option>
            <option v-for="connection in profileScopeConnections" :key="connection.id" :value="connection.id">
              {{ connection.display_name }} / {{ connection.status }}
            </option>
          </select>
        </label>
        <label v-if="state.profilePolicyScope.value === 'event_pattern'">
          <span>{{ t('Pattern') }}</span>
          <input
            v-model="state.profilePolicyEventPattern.value"
            class="makosh-input-control"
            type="text"
            placeholder="signal.raw.*"
            :disabled="selectedProfile?.is_system"
          />
        </label>
        <label>
          <span>{{ t('Reason') }}</span>
          <input
            v-model="state.profilePolicyReason.value"
            class="makosh-input-control"
            type="text"
            :placeholder="t('Profile policy reason')"
            :disabled="selectedProfile?.is_system"
          />
        </label>
        <div class="runtime-actions">
          <button type="button" class="makosh-btn makosh-btn--outline makosh-btn--compact" :disabled="selectedProfile?.is_system" @click="state.addDraftProfilePolicy">
            <Icon icon="tabler:list-plus" />
            {{ t('Add Policy') }}
          </button>
        </div>
        <div class="signal-callout">
          <strong>{{ t('Profile Policies') }}</strong>
          <span v-if="state.profileDraftPolicies.value.length === 0">{{ t('No profile policies yet.') }}</span>
          <template v-else>
            <div
              v-for="(policy, index) in state.profileDraftPolicies.value"
              :key="`${policy.scope}:${policy.mode}:${index}`"
              class="policy-row"
            >
              <Icon icon="tabler:shield-cog" />
              <span>
                <strong>{{ profilePolicyLabel(t, connections, policy) }}</strong>
                <em>{{ policy.reason }}</em>
              </span>
              <button
                v-if="!selectedProfile?.is_system"
                type="button"
                class="makosh-btn makosh-btn--outline makosh-btn--compact"
                @click="state.removeDraftProfilePolicy(index)"
              >
                {{ t('Remove') }}
              </button>
            </div>
          </template>
        </div>
        <div class="runtime-actions">
          <button type="submit" class="makosh-btn" :disabled="state.isSavingProfile.value || selectedProfile?.is_system">
            <Icon icon="tabler:device-floppy" />
            {{
              selectedProfile
                ? state.isSavingProfile.value ? t('Saving') : t('Update Profile')
                : state.isSavingProfile.value ? t('Creating') : t('Create Profile')
            }}
          </button>
          <button type="button" class="makosh-btn makosh-btn--outline" @click="state.resetProfileEditor">
            {{ t('Reset') }}
          </button>
          <button
            v-if="selectedProfile && !selectedProfile.is_system"
            type="button"
            class="makosh-btn makosh-btn--outline"
            :disabled="state.isRemovingProfile.value"
            @click="state.handleRemoveProfile(selectedProfile.code)"
          >
            {{ state.isRemovingProfile.value ? t('Removing') : t('Delete Profile') }}
          </button>
        </div>
      </form>

      <div v-if="profiles.length === 0" class="empty-panel fill">{{ t('No profiles yet.') }}</div>
      <div v-else class="signal-table">
        <button
          v-for="profile in profiles"
          :key="profile.id"
          type="button"
          class="signal-table-row signal-runtime-row"
          :class="{ selected: selectedProfile?.code === profile.code }"
          @click="state.handleSelectProfile(profile)"
        >
          <div class="signal-table-main">
            <Icon icon="tabler:layout-dashboard" />
            <span>
              <strong>{{ profile.display_name }}</strong>
              <em>{{ profile.code }} / {{ profile.policy_count }} {{ t('policies') }}</em>
              <small class="signal-detail-text">{{ profile.description }}</small>
            </span>
          </div>
          <b class="signal-pill" :data-tone="profile.is_active ? 'good' : 'neutral'">
            {{ profile.is_active ? t('Active') : profile.is_system ? t('System') : t('Custom') }}
          </b>
          <div class="runtime-actions">
            <small class="profile-description">
              {{ profile.source_policies.slice(0, 2).map((policy) => profilePolicyLabel(t, connections, policy)).join(' • ') || t('No profile policies') }}
            </small>
            <button type="button" class="makosh-btn makosh-btn--outline makosh-btn--compact" :disabled="state.isApplyingProfile.value || profile.is_active" @click.stop="state.handleApplyProfile(profile.code)">
              {{ profile.is_active ? t('Applied') : t('Apply') }}
            </button>
          </div>
        </button>
      </div>
    </div>
  </template>

  <div v-else class="policy-layout">
    <form class="policy-form" @submit.prevent="state.handleCreatePolicy">
      <label>
        <span>{{ t('Scope') }}</span>
        <select v-model="state.policyScope.value" class="makosh-select-control">
          <option value="event_pattern">{{ t('Event Pattern') }}</option>
          <option value="source">{{ t('Source') }}</option>
          <option value="connection">{{ t('Connection') }}</option>
          <option value="global">{{ t('Global') }}</option>
        </select>
      </label>
      <label>
        <span>{{ t('Mode') }}</span>
        <select v-model="state.policyMode.value" class="makosh-select-control">
          <option value="paused">{{ t('Pause') }}</option>
          <option value="muted">{{ t('Mute') }}</option>
          <option value="disabled">{{ t('Disable') }}</option>
          <option value="enabled">{{ t('Allow') }}</option>
        </select>
      </label>
      <label v-if="state.policyScope.value === 'source'">
        <span>{{ t('Source') }}</span>
        <select v-model="state.policySourceCode.value" class="makosh-select-control">
          <option v-for="source in sources" :key="source.code" :value="source.code">{{ source.display_name }}</option>
        </select>
      </label>
      <label v-if="state.policyScope.value === 'connection'">
        <span>{{ t('Source') }}</span>
        <select v-model="state.policySourceCode.value" class="makosh-select-control">
          <option v-for="source in connectionCapableSources" :key="source.code" :value="source.code">{{ source.display_name }}</option>
        </select>
      </label>
      <label v-if="state.policyScope.value === 'connection'">
        <span>{{ t('Connection') }}</span>
        <select v-model="state.policyConnectionId.value" class="makosh-select-control">
          <option value="">{{ t('Select connection') }}</option>
          <option v-for="connection in policyScopeConnections" :key="connection.id" :value="connection.id">
            {{ connection.display_name }} / {{ connection.status }}
          </option>
        </select>
      </label>
      <label v-if="state.policyScope.value === 'event_pattern'">
        <span>{{ t('Pattern') }}</span>
        <input v-model="state.policyEventPattern.value" class="makosh-input-control" type="text" placeholder="signal.raw.*" />
      </label>
      <label>
        <span>{{ t('Reason') }}</span>
        <input v-model="state.policyReason.value" class="makosh-input-control" type="text" :placeholder="t('Policy reason')" />
      </label>
      <button type="submit" class="makosh-btn" :disabled="state.isCreatingPolicy.value || state.isUpdatingSignalControls.value">
        <Icon icon="tabler:shield-plus" />
        {{ state.isCreatingPolicy.value || state.isUpdatingSignalControls.value ? t('Saving') : t('Create Policy') }}

```
_Source file truncated after 12000 characters. / Исходный файл обрезан после 12000 символов._

### `frontend/src/domains/settings/components/SignalHubSettings.css`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/domains/settings/components/SignalHubSettings.css`
- Size bytes / Размер в байтах: `7501`
- Included characters / Включено символов: `7501`
- Truncated / Обрезано: `no`

```text
.signal-tabs { display: flex; gap: 4px; padding: 8px 12px; border-top: 1px solid var(--hh-border); border-bottom: 1px solid var(--hh-border); overflow-x: auto; }
.signal-tabs button { display: inline-flex; align-items: center; gap: 6px; min-height: 30px; border: 1px solid transparent; border-radius: var(--hh-radius-sm); background: transparent; color: var(--hh-text-secondary); font-size: 12px; font-weight: 650; padding: 0 10px; white-space: nowrap; cursor: pointer; }
.signal-tabs button.active { border-color: var(--hh-accent); background: color-mix(in srgb, var(--hh-accent) 10%, transparent); color: var(--hh-accent); }
.signal-summary-strip { display: grid; grid-template-columns: repeat(8, minmax(0, 1fr)); gap: 1px; border-bottom: 1px solid var(--hh-border); background: var(--hh-border); }
.signal-summary-strip div { display: grid; gap: 2px; padding: 10px 12px; background: rgba(4, 18, 20, var(--hh-panel-alpha)); }
.signal-summary-strip strong { color: var(--hh-text-primary); font-size: 16px; font-weight: 760; }
.signal-summary-strip span { color: var(--hh-text-muted); font-size: 10px; font-weight: 700; text-transform: uppercase; }
.signal-sources-layout { display: grid; grid-template-columns: minmax(320px, 1fr) minmax(260px, 340px); min-height: 0; flex: 1; overflow: hidden; }
.source-catalog, .source-inspector { min-width: 0; min-height: 0; overflow: auto; }
.source-catalog { border-right: 1px solid var(--hh-border); }
.source-toolbar { display: grid; grid-template-columns: minmax(0, 1fr) 160px; gap: 8px; padding: 12px; border-bottom: 1px solid var(--hh-border); }
.source-list, .signal-table { display: grid; gap: 2px; padding: 8px; }
.source-row { display: grid; grid-template-columns: 22px minmax(0, 1fr) auto; align-items: center; gap: 10px; min-height: 48px; border: 1px solid transparent; border-radius: var(--hh-radius-control); background: transparent; color: var(--hh-text-secondary); padding: 0 10px; text-align: left; cursor: pointer; }
.source-row:hover, .source-row.selected { border-color: var(--hh-border-accent-soft, var(--hh-accent)); background: color-mix(in srgb, var(--hh-accent) 8%, transparent); }
.source-row svg { width: 16px; height: 16px; }
.source-row span, .signal-table-main span, .policy-row span { display: grid; min-width: 0; }
.source-row strong, .signal-table-main strong { color: var(--hh-text-primary); font-size: 12px; font-weight: 720; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
.source-row em, .signal-table-main em, .signal-table-row small { color: var(--hh-text-muted); font-size: 10px; font-style: normal; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
.source-inspector { display: grid; align-content: start; gap: 14px; padding: 16px; }
.source-inspector header { display: grid; grid-template-columns: 28px minmax(0, 1fr); align-items: center; gap: 10px; }
.source-inspector header svg, .signal-placeholder svg { width: 22px; height: 22px; color: var(--hh-accent); }
.source-inspector h3, .signal-placeholder strong { margin: 0; color: var(--hh-text-primary); font-size: 13px; font-weight: 760; }
.source-inspector header span, .profile-description, .signal-placeholder span, .fixture-emit-form label span, .fixture-emit-form small, .signal-callout span { color: var(--hh-text-muted); font-size: 11px; }
.source-inspector dl { display: grid; gap: 8px; margin: 0; }
.source-inspector dl div { display: flex; justify-content: space-between; gap: 12px; border-bottom: 1px solid var(--hh-border-muted); padding-bottom: 8px; }
.source-inspector dt, .source-inspector dd { margin: 0; font-size: 11px; }
.source-inspector dt { color: var(--hh-text-muted); }
.source-inspector dd { color: var(--hh-text-primary); text-align: right; overflow-wrap: anywhere; }
.capability-grid { display: flex; flex-wrap: wrap; gap: 6px; }
.capability-grid span { border: 1px solid var(--hh-border-muted); border-radius: var(--hh-radius-sm); color: var(--hh-text-secondary); font-size: 10px; font-weight: 720; padding: 4px 7px; }
.fixture-emit-form { display: grid; gap: 10px; margin-top: 14px; padding-top: 14px; border-top: 1px solid var(--hh-border); }
.fixture-emit-form label, .policy-form label { display: grid; gap: 6px; }
.fixture-emit-form small { overflow-wrap: anywhere; }
.policy-layout { display: grid; grid-template-columns: minmax(280px, 360px) minmax(0, 1fr); gap: 16px; padding: 16px; }
.policy-form, .policy-list { border: 1px solid var(--hh-border); border-radius: var(--hh-radius-md); background: var(--hh-surface); }
.policy-form { display: grid; align-content: start; gap: 12px; padding: 14px; }
.policy-form label span { color: var(--hh-text-secondary); font-size: 12px; font-weight: 650; }
.replay-selector-grid { display: grid; grid-template-columns: repeat(2, minmax(0, 1fr)); gap: 12px; }
.signal-callout { display: grid; gap: 4px; border: 1px solid var(--hh-border-muted); border-radius: var(--hh-radius-control); background: rgba(4, 18, 20, 0.24); padding: 10px 12px; }
.signal-callout strong { color: var(--hh-text-primary); font-size: 12px; font-weight: 700; }
.policy-list { align-content: start; overflow: hidden; }
.policy-row { display: grid; grid-template-columns: 24px minmax(0, 1fr) minmax(120px, 240px); gap: 10px; align-items: center; min-height: 52px; padding: 10px 12px; border-bottom: 1px solid var(--hh-border); }
.policy-row:last-child { border-bottom: 0; }
.policy-row svg { color: var(--hh-accent); }
.policy-row strong, .policy-row em, .policy-row b { overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
.policy-row strong { color: var(--hh-text-primary); font-size: 13px; }
.policy-row em, .policy-row b { color: var(--hh-text-secondary); font-size: 12px; font-style: normal; font-weight: 500; }
.signal-runtime-row { grid-template-columns: minmax(0, 1fr) auto minmax(240px, 320px); }
.runtime-actions { display: flex; justify-content: flex-end; gap: 6px; flex-wrap: wrap; }
.signal-placeholder { display: grid; place-items: center; align-content: center; gap: 8px; min-height: 260px; color: var(--hh-text-muted); }
.signal-table-layout { min-height: 0; flex: 1; overflow: auto; }
.signal-table-row { display: grid; grid-template-columns: minmax(0, 1fr) auto minmax(180px, 220px); align-items: center; gap: 12px; min-height: 48px; border: 1px solid var(--hh-border-muted); border-radius: var(--hh-radius-control); background: rgba(4, 18, 20, 0.28); padding: 0 12px; }
.signal-table-row.selected { border-color: var(--hh-accent); }
.signal-table-main { display: grid; grid-template-columns: 18px minmax(0, 1fr); align-items: center; gap: 10px; min-width: 0; }
.signal-detail-text { line-height: 1.4; }
.signal-detail-text--bad { color: var(--makosh-danger, #f87171) !important; }
.signal-pill { border-radius: 999px; font-size: 10px; font-weight: 760; line-height: 1; padding: 6px 8px; text-transform: uppercase; }
.signal-pill[data-tone='good'] { background: rgba(34, 197, 94, 0.12); color: #4ade80; }
.signal-pill[data-tone='warn'] { background: rgba(245, 158, 11, 0.12); color: #fbbf24; }
.signal-pill[data-tone='bad'] { background: rgba(239, 68, 68, 0.12); color: #f87171; }
.signal-pill[data-tone='neutral'] { background: rgba(148, 163, 184, 0.12); color: var(--hh-text-secondary); }
@media (max-width: 1100px) {
  .signal-sources-layout { grid-template-columns: 1fr; }
  .source-catalog { border-right: none; border-bottom: 1px solid var(--hh-border); }
  .source-inspector { max-height: 260px; }
  .policy-layout { grid-template-columns: 1fr; }
}
```

### `frontend/src/domains/settings/components/SignalHubSettings.vue`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/domains/settings/components/SignalHubSettings.vue`
- Size bytes / Размер в байтах: `2961`
- Included characters / Включено символов: `2961`
- Truncated / Обрезано: `no`

```text
<script setup lang="ts">
import Icon from '../../../shared/ui/Icon.vue'
import { useI18n } from '../../../platform/i18n'
import SignalHubSourcesTab from './SignalHubSourcesTab.vue'
import SignalHubProfilesPoliciesTab from './SignalHubProfilesPoliciesTab.vue'
import SignalHubOperationsTab from './SignalHubOperationsTab.vue'
import { useSignalHubSettingsController } from './useSignalHubSettingsController'

const { t } = useI18n()
const state = useSignalHubSettingsController()
</script>

<template>
  <div class="settings-page">
    <section class="panel settings-list-panel settings-primary-pane">
      <header class="panel-title-row">
        <div>
          <h2>{{ t('Signal Hub') }}</h2>
          <p>{{ t('Source control, recovery fixture and signal runtime state.') }}</p>
        </div>
        <button
          type="button"
          class="makosh-btn makosh-btn--outline"
          :disabled="state.isRestoringFixture.value"
          @click="state.handleRestoreFixture"
        >
          <Icon icon="tabler:restore" />
          {{ state.isRestoringFixture.value ? t('Restoring') : t('Restore Fixture') }}
        </button>
      </header>

      <div class="signal-tabs" role="tablist" :aria-label="t('Signal Hub sections')">
        <button
          v-for="tab in state.tabs"
          :key="tab.id"
          type="button"
          :class="{ active: state.activeTab.value === tab.id }"
          @click="state.activeTab.value = tab.id"
        >
          <Icon :icon="tab.icon" />
          <span>{{ t(tab.label) }}</span>
        </button>
      </div>

      <div class="signal-summary-strip">
        <div><strong>{{ state.sources.value.length }}</strong><span>{{ t('Sources') }}</span></div>
        <div><strong>{{ state.enabledCount.value }}</strong><span>{{ t('Enabled') }}</span></div>
        <div>
          <strong>{{ state.activeRuntimeCount.value }}/{{ state.runtimeCount.value }}</strong>
          <span>{{ t('Runtime') }}</span>
        </div>
        <div><strong>{{ state.replayCount.value }}</strong><span>{{ t('Replay') }}</span></div>
        <div><strong>{{ state.connectedCount.value }}</strong><span>{{ t('Connected') }}</span></div>
        <div><strong>{{ state.unhealthyCount.value }}</strong><span>{{ t('Attention') }}</span></div>
        <div><strong>{{ state.replayPendingCount.value }}</strong><span>{{ t('Replay Queue') }}</span></div>
        <div>
          <strong>{{ state.activeProfile.value?.display_name ?? t('None') }}</strong>
          <span>{{ t('Active Profile') }}</span>
        </div>
      </div>

      <SignalHubSourcesTab v-if="state.activeTab.value === 'sources'" :state="state" />
      <SignalHubProfilesPoliciesTab
        v-else-if="state.activeTab.value === 'profiles' || state.activeTab.value === 'policies'"
        :state="state"
      />
      <SignalHubOperationsTab v-else :state="state" />
    </section>
  </div>
</template>

<style src="./SignalHubSettings.css"></style>
```

### `frontend/src/domains/settings/components/SignalHubSourcesTab.vue`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/domains/settings/components/SignalHubSourcesTab.vue`
- Size bytes / Размер в байтах: `6477`
- Included characters / Включено символов: `6477`
- Truncated / Обрезано: `no`

```text
<script setup lang="ts">
import { computed } from 'vue'
import Icon from '../../../shared/ui/Icon.vue'
import { useI18n } from '../../../platform/i18n'
import type { SignalHubSettingsController } from './useSignalHubSettingsController'
import {
  capabilityLabel,
  capabilityLabels,
  capabilityTone,
  sourceControlState,
  sourceIcon,
  sourceStateTone
} from './signalHubSettingsPresentation'

const props = defineProps<{ state: SignalHubSettingsController }>()
const { t } = useI18n()

const sources = computed(() => props.state.sources.value)
const policies = computed(() => props.state.policies.value)
const categories = computed(() => props.state.categories.value)
const filteredSources = computed(() => props.state.filteredSources.value)
const selectedSource = computed(() => props.state.selectedSource.value)
const selectedSourceCapabilities = computed(() => props.state.selectedSourceCapabilities.value)
const fixtureSources = computed(() => props.state.fixtureSources.value)
</script>

<template>
  <div class="signal-sources-layout">
    <div class="source-catalog">
      <div class="source-toolbar">
        <input
          v-model="state.sourceSearch"
          class="makosh-input-control"
          type="search"
          :placeholder="t('Search sources')"
        />
        <select v-model="state.sourceCategory" class="makosh-select-control">
          <option v-for="category in categories" :key="category" :value="category">
            {{ category === 'all' ? t('All') : category }}
          </option>
        </select>
      </div>

      <div v-if="state.isLoading.value && sources.length === 0" class="empty-panel fill">
        {{ t('Loading sources...') }}
      </div>
      <div v-else-if="filteredSources.length === 0" class="empty-panel fill">
        {{ t('No matching sources.') }}
      </div>
      <div v-else class="source-list">
        <button
          v-for="source in filteredSources"
          :key="source.code"
          type="button"
          class="source-row"
          :class="{ selected: selectedSource?.code === source.code }"
          @click="state.selectedSourceCode.value = source.code"
        >
          <Icon :icon="sourceIcon(source)" />
          <span>
            <strong>{{ source.display_name }}</strong>
            <em>{{ source.code }}</em>
          </span>
          <b class="signal-pill" :data-tone="sourceStateTone(sourceControlState(policies, source))">
            {{ sourceControlState(policies, source) }}
          </b>
        </button>
      </div>
    </div>

    <aside v-if="selectedSource" class="source-inspector">
      <header>
        <Icon :icon="sourceIcon(selectedSource)" />
        <div>
          <h3>{{ selectedSource.display_name }}</h3>
          <span>{{ selectedSource.category }} / {{ selectedSource.source_kind }}</span>
        </div>
      </header>
      <dl>
        <div>
          <dt>{{ t('Code') }}</dt>
          <dd>{{ selectedSource.code }}</dd>
        </div>
        <div>
          <dt>{{ t('Schema') }}</dt>
          <dd>v{{ selectedSource.capability_schema_version }}</dd>
        </div>
        <div>
          <dt>{{ t('State') }}</dt>
          <dd>{{ sourceControlState(policies, selectedSource) }}</dd>
        </div>
        <div>
          <dt>{{ t('Updated') }}</dt>
          <dd>{{ selectedSource.updated_at }}</dd>
        </div>
      </dl>
      <div class="capability-grid">
        <span v-for="capability in capabilityLabels(selectedSource)" :key="capability">
          {{ t(capability) }}
        </span>
      </div>
      <div v-if="selectedSourceCapabilities.length > 0" class="signal-table capability-table">
        <div
          v-for="capability in selectedSourceCapabilities"
          :key="capability.id"
          class="signal-table-row"
        >
          <div class="signal-table-main">
            <Icon icon="tabler:bolt" />
            <span>
              <strong>{{ capabilityLabel(capability) }}</strong>
              <em>{{ capability.reason ?? t('No capability note') }}</em>
            </span>
          </div>
          <b class="signal-pill" :data-tone="capabilityTone(capability.state)">
            {{ capability.state }}
          </b>
          <div class="runtime-actions">
            <small>{{
              capability.requires_confirmation ? t('Confirmation required') : t('No confirmation')
            }}</small>
          </div>
        </div>
      </div>
      <div class="runtime-actions">
        <button
          type="button"
          class="makosh-btn makosh-btn--outline makosh-btn--compact"
          :disabled="state.isUpdatingSignalControls.value"
          @click="state.handleEnableSource(selectedSource.code)"
        >
          <Icon icon="tabler:player-play" />
          {{ t('Enable Source') }}
        </button>
        <button
          type="button"
          class="makosh-btn makosh-btn--outline makosh-btn--compact"
          :disabled="state.isUpdatingSignalControls.value"
          @click="state.handleDisableSource(selectedSource.code)"
        >
          <Icon icon="tabler:player-stop" />
          {{ t('Disable Source') }}
        </button>
      </div>
      <form
        v-if="selectedSource.code === 'fixture'"
        class="fixture-emit-form"
        @submit.prevent="state.handleEmitFixtureSignal"
      >
        <label>
          <span>{{ t('Fixture Signal') }}</span>
          <select v-model="state.fixtureSignalId.value" class="makosh-select-control">
            <option
              v-for="fixture in fixtureSources"
              :key="fixture.fixture_id"
              :value="fixture.fixture_id"
            >
              {{ `${fixture.fixture_id} / ${fixture.summary}` }}
            </option>
          </select>
        </label>
        <button type="submit" class="makosh-btn" :disabled="state.isEmittingFixture.value">
          <Icon icon="tabler:test-pipe" />
          {{ state.isEmittingFixture.value ? t('Emitting') : t('Emit Fixture') }}
        </button>
        <small v-if="fixtureSources.length > 0">
          {{
            fixtureSources.find((fixture) => fixture.fixture_id === state.fixtureSignalId.value)?.event_type ??
            t('No fixture catalog entry')
          }}
        </small>
        <small v-if="state.emitFixture.data.value">
          {{ `${state.emitFixture.data.value.event_type} / ${state.emitFixture.data.value.raw_event_id}` }}
        </small>
      </form>
    </aside>
  </div>
</template>
```

### `frontend/src/domains/settings/components/appearance/AccentPicker.vue`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/domains/settings/components/appearance/AccentPicker.vue`
- Size bytes / Размер в байтах: `2355`
- Included characters / Включено символов: `2355`
- Truncated / Обрезано: `no`

```text
<script setup lang="ts">
import { accentColorIds, type AccentColorId } from '../../../../platform/theme/settings'

defineProps<{
	value: AccentColorId
	title: string
	description: string
}>()

defineEmits<{
	change: [value: AccentColorId]
}>()

const labels: Record<AccentColorId, string> = {
	teal: 'Teal',
	cyan: 'Cyan',
	blue: 'Blue',
	violet: 'Violet',
	amber: 'Amber',
	rose: 'Rose'
}
</script>

<template>
	<section class="appearance-section">
		<header>
			<div>
				<h3>{{ title }}</h3>
				<p>{{ description }}</p>
			</div>
		</header>
		<div class="accent-option-grid">
			<button
				v-for="id in accentColorIds"
				:key="id"
				type="button"
				class="accent-option-btn"
				:class="{ active: value === id }"
				:aria-pressed="value === id"
				@click="$emit('change', id)"
			>
				<span class="accent-swatch" :class="`accent-swatch--${id}`" />
				<span>{{ labels[id] }}</span>
			</button>
		</div>
	</section>
</template>

<style scoped>
.appearance-section {
	display: grid;
	gap: 12px;
	padding: var(--hh-space-panel);
	border-top: 1px solid var(--hh-border);
}

.appearance-section header h3 {
	margin: 0;
	font-size: 13px;
	font-weight: 680;
	color: var(--hh-text-primary);
}

.appearance-section header p {
	margin: 2px 0 0;
	font-size: 11px;
	color: var(--hh-text-muted);
}

.accent-option-grid {
	display: grid;
	grid-template-columns: repeat(auto-fill, minmax(80px, 1fr));
	gap: 8px;
}

.accent-option-btn {
	display: flex;
	align-items: center;
	gap: 6px;
	padding: 6px 8px;
	border: 1px solid var(--hh-border);
	border-radius: var(--hh-radius-sm);
	background: transparent;
	cursor: pointer;
	font-size: 11px;
	color: var(--hh-text-secondary);
	transition: border-color 150ms ease, background 150ms ease;
}

.accent-option-btn:hover,
.accent-option-btn.active {
	border-color: var(--hh-accent);
}

.accent-option-btn.active {
	background: var(--hh-hover-bg);
	color: var(--hh-accent);
}

.accent-swatch {
	display: inline-block;
	width: 14px;
	height: 14px;
	border-radius: 50%;
	border: 1px solid var(--hh-border);
	flex-shrink: 0;
}

.accent-swatch--teal { background: #14b8a6; }
.accent-swatch--cyan { background: #06b6d4; }
.accent-swatch--blue { background: #3b82f6; }
.accent-swatch--violet { background: #8b5cf6; }
.accent-swatch--amber { background: #f59e0b; }
.accent-swatch--rose { background: #f43f5e; }
</style>
```

### `frontend/src/domains/settings/components/appearance/AppearanceHeader.vue`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/domains/settings/components/appearance/AppearanceHeader.vue`
- Size bytes / Размер в байтах: `1653`
- Included characters / Включено символов: `1653`
- Truncated / Обрезано: `no`

```text
<script setup lang="ts">
defineProps<{
	title: string
	description: string
	isSaving: boolean
	saveStateLabel: string
	persistenceError: string
}>()

defineEmits<{
	reset: []
}>()
</script>

<template>
	<header class="panel-title-row">
		<div>
			<h2>{{ title }}</h2>
			<p>{{ description }}</p>
		</div>
			<div class="appearance-settings-actions">
				<button type="button" class="makosh-btn makosh-btn--outline" @click="$emit('reset')">
					Default
				</button>
				<div class="appearance-save-status">
					<span
						class="appearance-save-state"
						:class="{ 'appearance-save-state--warning': persistenceError }"
					>
						{{ isSaving ? 'Saving' : saveStateLabel }}
					</span>
					<p v-if="persistenceError" class="appearance-save-message">
						{{ persistenceError }}
					</p>
				</div>
			</div>
		</header>
	</template>

<style scoped>
.appearance-settings-actions {
	display: flex;
	flex-wrap: wrap;
	justify-content: flex-end;
	gap: 8px;
}

.appearance-save-status {
	display: flex;
	flex-direction: column;
	align-items: flex-end;
	gap: 4px;
	max-width: 320px;
}

.appearance-save-state {
	display: inline-flex;
	align-items: center;
	min-height: 34px;
	border: 1px solid rgba(111, 205, 195, 0.12);
	border-radius: var(--hh-radius-control, 6px);
	background: rgba(2, 12, 16, 0.48);
	color: var(--hh-text-muted);
	font-size: 12px;
	font-weight: 720;
	padding: 0 12px;
}

.appearance-save-state--warning {
	border-color: rgba(242, 184, 75, 0.34);
	background: rgba(92, 56, 8, 0.34);
	color: #f8d99c;
}

.appearance-save-message {
	margin: 0;
	color: #f8d99c;
	font-size: 12px;
	line-height: 1.35;
	text-align: right;
}
</style>
```

### `frontend/src/domains/settings/components/appearance/BackgroundPicker.vue`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/domains/settings/components/appearance/BackgroundPicker.vue`
- Size bytes / Размер в байтах: `3556`
- Included characters / Включено символов: `3556`
- Truncated / Обрезано: `no`

```text
<script setup lang="ts">
import { useI18n } from '../../../../platform/i18n'
import { shellBackgroundIds, type ShellBackgroundId } from '../../../../platform/theme/settings'

defineProps<{
	value: ShellBackgroundId
	title: string
	description: string
}>()

defineEmits<{
	change: [value: ShellBackgroundId]
}>()

const { t } = useI18n()

const labels: Record<ShellBackgroundId, string> = {
	none: 'No background',
	'network-mesh': 'Digital network',
	'data-stream': 'Data flow',
	'node-frame': 'Node grid',
	'eclipse-grid': 'Dark grid',
	'dna-blueprint': 'Connection blueprint',
	'forest-network': 'Green network',
	'forest-stream': 'Green flow',
	'knowledge-map': 'Knowledge map',
	'rune-gold': 'Warm accent',
	'rune-teal': 'Teal accent'
}
</script>

<template>
	<section class="appearance-section">
		<header>
			<div>
				<h3>{{ title }}</h3>
				<p>{{ description }}</p>
			</div>
		</header>
		<div class="background-option-grid">
			<button
				v-for="id in shellBackgroundIds"
				:key="id"
				type="button"
				class="background-option-btn"
				:class="{ active: value === id }"
				:aria-pressed="value === id"
				@click="$emit('change', id)"
			>
				<span class="shell-bg-preview" :class="`shell-bg-preview--${id}`" />
				<span>{{ t(labels[id]) }}</span>
			</button>
		</div>
	</section>
</template>

<style scoped>
.appearance-section {
	display: grid;
	gap: 12px;
	padding: var(--hh-space-panel);
	border-top: 1px solid var(--hh-border);
}

.appearance-section header h3 {
	margin: 0;
	font-size: 13px;
	font-weight: 680;
	color: var(--hh-text-primary);
}

.appearance-section header p {
	margin: 2px 0 0;
	font-size: 11px;
	color: var(--hh-text-muted);
}

.background-option-grid {
	display: grid;
	grid-template-columns: repeat(auto-fill, minmax(90px, 1fr));
	gap: 8px;
}

.background-option-btn {
	display: grid;
	gap: 6px;
	padding: 8px;
	border: 1px solid var(--hh-border);
	border-radius: var(--hh-radius-sm);
	background: var(--hh-surface-deep);
	cursor: pointer;
	text-align: center;
	font-size: 11px;
	color: var(--hh-text-secondary);
	transition: border-color 150ms ease, background 150ms ease;
}

.background-option-btn:hover,
.background-option-btn.active {
	border-color: var(--hh-accent);
}

.background-option-btn.active {
	background: var(--hh-hover-bg);
	color: var(--hh-accent);
}

.shell-bg-preview {
	display: block;
	width: 100%;
	height: 48px;
	border-radius: var(--hh-radius-xs);
	background: var(--hh-surface-deep);
	border: 1px solid var(--hh-border);
}

.shell-bg-preview--rune-teal { background: linear-gradient(135deg, #0f766e 0%, #042f2e 100%); }
.shell-bg-preview--rune-gold { background: linear-gradient(135deg, #b45309 0%, #451a03 100%); }
.shell-bg-preview--forest-network { background: linear-gradient(135deg, #065f46 0%, #022c22 100%); }
.shell-bg-preview--forest-stream { background: linear-gradient(135deg, #047857 0%, #064e3b 100%); }
.shell-bg-preview--knowledge-map { background: linear-gradient(135deg, #1e40af 0%, #1e1b4b 100%); }
.shell-bg-preview--eclipse-grid { background: linear-gradient(135deg, #1e293b 0%, #0f172a 100%); }
.shell-bg-preview--network-mesh { background: linear-gradient(135deg, #334155 0%, #0f172a 100%); }
.shell-bg-preview--dna-blueprint { background: linear-gradient(135deg, #1e3a5f 0%, #0c1929 100%); }
.shell-bg-preview--data-stream { background: linear-gradient(135deg, #164e63 0%, #083344 100%); }
.shell-bg-preview--node-frame { background: linear-gradient(135deg, #0f172a 0%, #111827 100%); }
.shell-bg-preview--none { background: var(--hh-surface-deep); }
</style>
```

### `frontend/src/domains/settings/components/appearance/SpacingDensityControl.vue`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/domains/settings/components/appearance/SpacingDensityControl.vue`
- Size bytes / Размер в байтах: `1790`
- Included characters / Включено символов: `1790`
- Truncated / Обрезано: `no`

```text
<script setup lang="ts">
import { spacingDensityIds, type SpacingDensity } from '../../../../platform/theme/settings'

defineProps<{
	value: SpacingDensity
	title: string
	description: string
}>()

defineEmits<{
	change: [value: SpacingDensity]
}>()

const labels: Record<SpacingDensity, string> = {
	compact: 'Compact',
	normal: 'Normal',
	comfortable: 'Comfortable'
}
</script>

<template>
	<section class="appearance-section">
		<header>
			<div>
				<h3>{{ title }}</h3>
				<p>{{ description }}</p>
			</div>
		</header>
		<div class="density-options">
			<button
				v-for="id in spacingDensityIds"
				:key="id"
				type="button"
				class="density-option-btn"
				:class="{ active: value === id }"
				:aria-pressed="value === id"
				@click="$emit('change', id)"
			>
				{{ labels[id] }}
			</button>
		</div>
	</section>
</template>

<style scoped>
.appearance-section {
	display: grid;
	gap: 12px;
	padding: var(--hh-space-panel);
	border-top: 1px solid var(--hh-border);
}

.appearance-section header h3 {
	margin: 0;
	font-size: 13px;
	font-weight: 680;
	color: var(--hh-text-primary);
}

.appearance-section header p {
	margin: 2px 0 0;
	font-size: 11px;
	color: var(--hh-text-muted);
}

.density-options {
	display: flex;
	flex-wrap: wrap;
	gap: 6px;
}

.density-option-btn {
	min-width: 96px;
	height: 34px;
	padding: 0 10px;
	border: 1px solid var(--hh-border);
	border-radius: var(--hh-radius-sm);
	background: transparent;
	color: var(--hh-text-secondary);
	font-size: 12px;
	font-weight: 620;
	cursor: pointer;
	transition: all 100ms ease;
}

.density-option-btn:hover,
.density-option-btn.active {
	border-color: var(--hh-accent);
}

.density-option-btn.active {
	background: color-mix(in srgb, var(--hh-accent) 15%, transparent);
	color: var(--hh-accent);
}
</style>
```

### `frontend/src/domains/settings/components/appearance/ThemeRangeControl.vue`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/domains/settings/components/appearance/ThemeRangeControl.vue`
- Size bytes / Размер в байтах: `1732`
- Included characters / Включено символов: `1732`
- Truncated / Обрезано: `no`

```text
<script setup lang="ts">
defineProps<{
	id: string
	label: string
	description: string
	value: number
	min: number
	max: number
	step: number
	unit: string
}>()

defineEmits<{
	preview: [value: number]
	commit: []
}>()
</script>

<template>
	<section class="appearance-section">
		<header>
			<div>
				<h3>{{ label }}</h3>
				<p>{{ description }}</p>
			</div>
			<strong>{{ value }}{{ unit }}</strong>
		</header>
		<input
			:id="id"
			type="range"
			:min="min"
			:max="max"
			:step="step"
			:value="value"
			:aria-label="label"
			@input="$emit('preview', Number(($event.target as HTMLInputElement).value))"
			@change="$emit('commit')"
		/>
	</section>
</template>

<style scoped>
.appearance-section {
	display: grid;
	gap: 12px;
	padding: var(--hh-space-panel);
	border-top: 1px solid var(--hh-border);
}

.appearance-section header {
	display: flex;
	align-items: baseline;
	justify-content: space-between;
	gap: 12px;
}

.appearance-section header h3 {
	margin: 0;
	font-size: 13px;
	font-weight: 680;
	color: var(--hh-text-primary);
}

.appearance-section header p {
	margin: 2px 0 0;
	font-size: 11px;
	color: var(--hh-text-muted);
}

.appearance-section header strong {
	font-size: 13px;
	font-weight: 720;
	color: var(--hh-accent);
	white-space: nowrap;
}

input[type="range"] {
	width: 100%;
	height: 6px;
	-webkit-appearance: none;
	appearance: none;
	background: var(--hh-hover-bg);
	border-radius: 3px;
	outline: none;
	cursor: pointer;
}

input[type="range"]::-webkit-slider-thumb {
	-webkit-appearance: none;
	width: 18px;
	height: 18px;
	border-radius: 50%;
	background: var(--hh-accent);
	border: 2px solid var(--hh-surface-panel);
	cursor: pointer;
	box-shadow: 0 1px 4px rgba(0, 0, 0, 0.3);
}
</style>
```

### `frontend/src/domains/settings/components/sidebar/SidebarGroupEditor.vue`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/domains/settings/components/sidebar/SidebarGroupEditor.vue`
- Size bytes / Размер в байтах: `4546`
- Included characters / Включено символов: `4540`
- Truncated / Обрезано: `no`

```text
<script setup lang="ts">
import type { SidebarItemId, SidebarNavGroup } from '../../../../shared/stores/sidebar'
import SidebarItemEditor from './SidebarItemEditor.vue'

defineProps<{
  group: SidebarNavGroup
  groupIndex: number
  rootIndex: number
  rootItemCount: number
  itemLabels: Record<SidebarItemId, { label: string; icon: string }>
  hiddenItemIds: SidebarItemId[]
  groupOptions: Array<{ value: string; label: string }>
  groupLabelText: string
  defaultPlaceholder: string
  groupPlaceholder: string
  visibleDomainLabel: string
  hiddenLabel: string
  noItemsLabel: string
  moveToGroupLabel: string
  dividerLabel: string
  showLabel: string
  hideLabel: string
}>()

defineEmits<{
  rename: [groupId: string, label: string]
  moveGroup: [groupId: string, direction: -1 | 1]
  removeGroup: [groupId: string]
  moveItemToGroup: [itemId: SidebarItemId, targetGroupId: string]
  moveItem: [itemId: SidebarItemId, direction: -1 | 1]
  toggleDivider: [groupId: string, itemId: SidebarItemId]
  toggleHidden: [itemId: SidebarItemId]
}>()
</script>

<template>
  <section class="sidebar-config-group">
    <header>
      <label>
        <span>{{ groupLabelText }}</span>
        <input
          :value="group.label"
          :placeholder="groupIndex === 0 ? defaultPlaceholder : groupPlaceholder"
          autocomplete="off"
          @input="$emit('rename', group.id, ($event.target as HTMLInputElement).value)"
        />
      </label>
      <div class="sidebar-config-group-actions">
        <button
          type="button"
          class="makosh-btn makosh-btn--icon"
          :disabled="rootIndex <= 0"
          @click="$emit('moveGroup', group.id, -1)"
        >
          ↑
        </button>
        <button
          type="button"
          class="makosh-btn makosh-btn--icon"
          :disabled="rootIndex === rootItemCount - 1"
          @click="$emit('moveGroup', group.id, 1)"
        >
          ↓
        </button>
        <button
          type="button"
          class="makosh-btn makosh-btn--icon makosh-btn--destructive"
          :disabled="group.id === 'communications'"
          @click="$emit('removeGroup', group.id)"
        >
          ✕
        </button>
      </div>
    </header>
    <div class="sidebar-config-items">
      <div v-if="group.itemIds.length === 0" class="empty-panel">
        {{ noItemsLabel }}
      </div>
      <template v-else>
        <SidebarItemEditor
          v-for="(itemId, itemIndex) in group.itemIds"
          :key="itemId"
          :item-id="itemId"
          :label="itemLabels[itemId]?.label ?? itemId"
          :icon="itemLabels[itemId]?.icon ?? 'tabler:circle'"
          :hidden="hiddenItemIds.includes(itemId)"
          :status-label="hiddenItemIds.includes(itemId) ? hiddenLabel : visibleDomainLabel"
          :move-target-options="groupOptions"
          :move-target-value="group.id"
          :move-target-placeholder="moveToGroupLabel"
          :show-divider-control="true"
          :divider-active="group.separatorBeforeItemIds.includes(itemId)"
          :divider-disabled="itemIndex === 0"
          :divider-label="dividerLabel"
          :show-label="showLabel"
          :hide-label="hideLabel"
          @move-to-group="(itemId, targetGroupId) => $emit('moveItemToGroup', itemId, targetGroupId)"
          @move-up="$emit('moveItem', $event, -1)"
          @move-down="$emit('moveItem', $event, 1)"
          @toggle-divider="$emit('toggleDivider', group.id, $event)"
          @toggle-hidden="$emit('toggleHidden', $event)"
        />
      </template>
    </div>
  </section>
</template>

<style scoped>
.sidebar-config-group {
  margin-bottom: 12px;
}

.sidebar-config-group > header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 6px 8px;
  border-bottom: 1px solid var(--hh-border);
}

.sidebar-config-group > header label {
  display: flex;
  align-items: center;
  gap: 6px;
  font-size: 11px;
  color: var(--hh-text-secondary);
}

.sidebar-config-group > header input {
  max-width: 180px;
  height: 28px;
  padding: 0 8px;
  background: var(--hh-surface-deep);
  border: 1px solid var(--hh-border);
  border-radius: var(--hh-radius-sm);
  color: var(--hh-text-primary);
  font-size: 12px;
  outline: none;
}

.sidebar-config-group > header input:focus-visible {
  box-shadow: 0 0 0 2px var(--hh-focus-ring);
  border-color: var(--hh-accent);
}

.sidebar-config-group-actions {
  display: flex;
  gap: 4px;
}

.sidebar-config-items {
  display: grid;
  gap: 4px;
  padding: 4px 0;
}
</style>
```

### `frontend/src/domains/settings/components/sidebar/SidebarItemEditor.vue`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/domains/settings/components/sidebar/SidebarItemEditor.vue`
- Size bytes / Размер в байтах: `3507`
- Included characters / Включено символов: `3503`
- Truncated / Обрезано: `no`

```text
<script setup lang="ts">
import { Icon } from '@iconify/vue'
import type { SidebarItemId } from '../../../../shared/stores/sidebar'

defineProps<{
  itemId: SidebarItemId
  label: string
  icon: string
  hidden: boolean
  statusLabel: string
  moveTargetOptions?: Array<{ value: string; label: string }>
  moveTargetValue?: string
  moveTargetPlaceholder?: string
  showDividerControl?: boolean
  dividerActive?: boolean
  dividerDisabled?: boolean
  dividerLabel?: string
  showLabel: string
  hideLabel: string
}>()

defineEmits<{
  moveToGroup: [itemId: SidebarItemId, targetGroupId: string]
  moveUp: [itemId: SidebarItemId]
  moveDown: [itemId: SidebarItemId]
  toggleDivider: [itemId: SidebarItemId]
  toggleHidden: [itemId: SidebarItemId]
}>()
</script>

<template>
  <div class="sidebar-config-item" :class="{ hidden }">
    <div class="sidebar-config-item-main">
      <span class="round-icon cyan">
        <Icon :icon="icon" aria-hidden="true" />
      </span>
      <div>
        <strong>{{ label }}</strong>
        <small>{{ statusLabel }}</small>
      </div>
    </div>
    <div class="sidebar-config-item-controls">
      <select
        v-if="moveTargetOptions"
        class="makosh-select-control"
        :value="moveTargetValue"
        @change="$emit('moveToGroup', itemId, ($event.target as HTMLSelectElement).value)"
      >
        <option v-if="moveTargetPlaceholder" :value="moveTargetValue" disabled>
          {{ moveTargetPlaceholder }}
        </option>
        <option v-for="option in moveTargetOptions" :key="option.value" :value="option.value">
          {{ option.label }}
        </option>
      </select>
      <button type="button" class="makosh-btn makosh-btn--icon" @click="$emit('moveUp', itemId)">
        ↑
      </button>
      <button type="button" class="makosh-btn makosh-btn--icon" @click="$emit('moveDown', itemId)">
        ↓
      </button>
      <button
        v-if="showDividerControl"
        type="button"
        class="makosh-btn makosh-btn--icon"
        :class="{ active: dividerActive }"
        :disabled="dividerDisabled"
        @click="$emit('toggleDivider', itemId)"
      >
        {{ dividerLabel }}
      </button>
      <button
        type="button"
        class="makosh-btn makosh-btn--icon"
        :class="{ active: !hidden }"
        @click="$emit('toggleHidden', itemId)"
      >
        {{ hidden ? showLabel : hideLabel }}
      </button>
    </div>
  </div>
</template>

<style scoped>
.sidebar-config-item {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 6px 8px;
  border-radius: var(--hh-radius-sm);
  transition: background 100ms ease;
}

.sidebar-config-item:hover {
  background: var(--hh-hover-bg);
}

.sidebar-config-item.hidden {
  opacity: 0.5;
}

.sidebar-config-item-main {
  display: flex;
  align-items: center;
  gap: 8px;
  min-width: 0;
}

.sidebar-config-item-main strong {
  display: block;
  font-size: 12px;
  font-weight: 620;
  color: var(--hh-text-primary);
}

.sidebar-config-item-main small {
  font-size: 10px;
  color: var(--hh-text-muted);
}

.sidebar-config-item-controls {
  display: flex;
  align-items: center;
  gap: 4px;
  flex-shrink: 0;
}

.round-icon {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 24px;
  height: 24px;
  border-radius: 50%;
  font-size: 14px;
  flex-shrink: 0;
}

.round-icon.cyan {
  background: color-mix(in srgb, var(--hh-accent) 15%, transparent);
  color: var(--hh-accent);
}
</style>
```

### `frontend/src/domains/settings/components/sidebar/SidebarNavigationList.vue`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/domains/settings/components/sidebar/SidebarNavigationList.vue`
- Size bytes / Размер в байтах: `5285`
- Included characters / Включено символов: `5278`
- Truncated / Обрезано: `no`

```text
<script setup lang="ts">
import { Icon } from '@iconify/vue'
import type {
  ResolvedSidebarRootEntry,
  SidebarItemId,
  SidebarRootItemId
} from '../../../../shared/stores/sidebar'
import SidebarItemEditor from './SidebarItemEditor.vue'

defineProps<{
  entries: ResolvedSidebarRootEntry[]
  hiddenItemIds: SidebarItemId[]
  rootItemCount: number
  groupOptions: Array<{ value: string; label: string }>
  rootLabel: string
  sidebarRootLabel: string
  expandableGroupLabel: string
  itemsLabel: string
  hiddenLabel: string
  rootDomainLabel: string
  moveToGroupLabel: string
  showLabel: string
  hideLabel: string
}>()

defineEmits<{
  moveGroup: [groupId: string, direction: -1 | 1]
  removeGroup: [groupId: string]
  moveRootItem: [rootId: SidebarRootItemId, direction: -1 | 1]
  moveItemToGroup: [itemId: SidebarItemId, targetGroupId: string]
  toggleHidden: [itemId: SidebarItemId]
}>()
</script>

<template>
  <section class="sidebar-config-group">
    <header>
      <label>
        <span>{{ rootLabel }}</span>
        <input :value="sidebarRootLabel" disabled autocomplete="off" />
      </label>
    </header>
    <div class="sidebar-config-items">
      <template v-for="(entry, rootIndex) in entries" :key="entry.rootId">
        <div v-if="entry.kind === 'group'" class="sidebar-config-item group-node">
          <div class="sidebar-config-item-main">
            <span class="round-icon green">
              <Icon :icon="entry.group.icon" aria-hidden="true" />
            </span>
            <div>
              <strong>{{ entry.group.label }}</strong>
              <small>{{ expandableGroupLabel }} · {{ entry.group.items.length }} {{ itemsLabel }}</small>
            </div>
          </div>
          <div class="sidebar-config-item-controls">
            <button
              type="button"
              class="makosh-btn makosh-btn--icon"
              :disabled="rootIndex === 0"
              @click="$emit('moveGroup', entry.group.id, -1)"
            >
              ↑
            </button>
            <button
              type="button"
              class="makosh-btn makosh-btn--icon"
              :disabled="rootIndex === rootItemCount - 1"
              @click="$emit('moveGroup', entry.group.id, 1)"
            >
              ↓
            </button>
            <button
              type="button"
              class="makosh-btn makosh-btn--icon makosh-btn--destructive"
              :disabled="entry.group.id === 'communications'"
              @click="$emit('removeGroup', entry.group.id)"
            >
              ✕
            </button>
          </div>
        </div>

        <SidebarItemEditor
          v-else
          :item-id="entry.item.itemId"
          :label="entry.item.label"
          :icon="entry.item.icon"
          :hidden="hiddenItemIds.includes(entry.item.itemId)"
          :status-label="hiddenItemIds.includes(entry.item.itemId) ? hiddenLabel : rootDomainLabel"
          :move-target-options="groupOptions"
          move-target-value="root"
          :move-target-placeholder="moveToGroupLabel"
          :show-label="showLabel"
          :hide-label="hideLabel"
          @move-to-group="(itemId, targetGroupId) => $emit('moveItemToGroup', itemId, targetGroupId)"
          @move-up="$emit('moveRootItem', entry.rootId, -1)"
          @move-down="$emit('moveRootItem', entry.rootId, 1)"
          @toggle-hidden="$emit('toggleHidden', $event)"
        />
      </template>
    </div>
  </section>
</template>

<style scoped>
.sidebar-config-group {
  margin-bottom: 12px;
}

.sidebar-config-group > header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 6px 8px;
  border-bottom: 1px solid var(--hh-border);
}

.sidebar-config-group > header label {
  display: flex;
  align-items: center;
  gap: 6px;
  font-size: 11px;
  color: var(--hh-text-secondary);
}

.sidebar-config-group > header input {
  max-width: 180px;
  height: 28px;
  padding: 0 8px;
  background: var(--hh-surface-deep);
  border: 1px solid var(--hh-border);
  border-radius: var(--hh-radius-sm);
  color: var(--hh-text-primary);
  font-size: 12px;
  outline: none;
}

.sidebar-config-items {
  display: grid;
  gap: 4px;
  padding: 4px 0;
}

.sidebar-config-item {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 6px 8px;
  border-radius: var(--hh-radius-sm);
  transition: background 100ms ease;
}

.sidebar-config-item:hover {
  background: var(--hh-hover-bg);
}

.sidebar-config-item.group-node {
  border-left: 2px solid var(--hh-accent);
}

.sidebar-config-item-main {
  display: flex;
  align-items: center;
  gap: 8px;
  min-width: 0;
}

.sidebar-config-item-main strong {
  display: block;
  font-size: 12px;
  font-weight: 620;
  color: var(--hh-text-primary);
}

.sidebar-config-item-main small {
  font-size: 10px;
  color: var(--hh-text-muted);
}

.sidebar-config-item-controls {
  display: flex;
  align-items: center;
  gap: 4px;
  flex-shrink: 0;
}

.round-icon {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 24px;
  height: 24px;
  border-radius: 50%;
  font-size: 14px;
  flex-shrink: 0;
}

.round-icon.green {
  background: color-mix(in srgb, #22c55e 15%, transparent);
  color: #22c55e;
}
</style>
```

### `frontend/src/domains/settings/components/sidebar/SidebarSettingsSummary.vue`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/domains/settings/components/sidebar/SidebarSettingsSummary.vue`
- Size bytes / Размер в байтах: `2889`
- Included characters / Включено символов: `2889`
- Truncated / Обрезано: `no`

```text
<script setup lang="ts">
import type {
  ResolvedSidebarRootEntry,
  SidebarItemId
} from '../../../../shared/stores/sidebar'

defineProps<{
  entries: ResolvedSidebarRootEntry[]
  hiddenItemIds: SidebarItemId[]
  itemLabels: Record<SidebarItemId, { label: string; icon: string }>
  previewLabel: string
  hiddenLabel: string
  rulesLabel: string
  rootDomainLabel: string
  emptyGroupLabel: string
  noHiddenLabel: string
  showLabel: string
  rules: Array<{ text: string; badge: string }>
}>()

defineEmits<{
  toggleHidden: [itemId: SidebarItemId]
}>()
</script>

<template>
  <aside class="settings-rail sidebar-settings-summary">
    <section class="panel info-card">
      <h2>{{ previewLabel }}</h2>
      <ul class="sidebar-preview-list">
        <li v-for="entry in entries" :key="entry.rootId">
          <template v-if="entry.kind === 'group'">
            <strong>{{ entry.group.label }}</strong>
            <span>{{ entry.group.items.map((item) => item.label).join(', ') || emptyGroupLabel }}</span>
          </template>
          <template v-else>
            <strong>{{ entry.item.label }}</strong>
            <span>{{ rootDomainLabel }}</span>
          </template>
        </li>
      </ul>
    </section>
    <section class="panel info-card">
      <h2>{{ hiddenLabel }}</h2>
      <p v-if="hiddenItemIds.length === 0">{{ noHiddenLabel }}</p>
      <ul v-else class="detail-list">
        <li v-for="itemId in hiddenItemIds" :key="itemId">
          {{ itemLabels[itemId]?.label ?? itemId }}
          <button
            type="button"
            class="makosh-btn makosh-btn--ghost"
            @click="$emit('toggleHidden', itemId)"
          >
            {{ showLabel }}
          </button>
        </li>
      </ul>
    </section>
    <section class="panel info-card">
      <h2>{{ rulesLabel }}</h2>
      <ul class="detail-list">
        <li v-for="rule in rules" :key="rule.text">
          {{ rule.text }}
          <em>{{ rule.badge }}</em>
        </li>
      </ul>
    </section>
  </aside>
</template>

<style scoped>
.settings-rail {
  display: grid;
  gap: 12px;
  align-content: start;
  min-width: 0;
  min-height: 0;
  max-height: 100%;
  overflow-x: hidden;
  overflow-y: auto;
}

.sidebar-preview-list,
.detail-list {
  display: grid;
  gap: 6px;
  list-style: none;
  padding: 0;
  margin: 0;
}

.sidebar-preview-list li {
  display: grid;
  gap: 2px;
}

.sidebar-preview-list li strong {
  font-size: 12px;
  font-weight: 620;
  color: var(--hh-text-primary);
}

.sidebar-preview-list li span {
  font-size: 10px;
  color: var(--hh-text-muted);
}

.detail-list li {
  display: flex;
  align-items: center;
  justify-content: space-between;
  font-size: 11px;
  color: var(--hh-text-secondary);
}

.detail-list li em {
  font-size: 9px;
  font-weight: 720;
  color: var(--hh-accent);
  font-style: normal;
  text-transform: uppercase;
}
</style>
```

### `frontend/src/domains/settings/views/SettingsPage.vue`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/domains/settings/views/SettingsPage.vue`
- Size bytes / Размер в байтах: `6561`
- Included characters / Включено символов: `6561`
- Truncated / Обрезано: `no`

```text
<script setup lang="ts">
import { useI18n } from '../../../platform/i18n'
import { useSettingsStore } from '../stores/settings'
import { useApplicationSettingsQuery } from '../queries/useSettingsQuery'
import Icon from '../../../shared/ui/Icon.vue'
import type { SettingsSection } from '../stores/settings'

import AppearanceSettings from '../components/AppearanceSettings.vue'
import LanguageSettings from '../components/LanguageSettings.vue'
import ApplicationSettings from '../components/ApplicationSettings.vue'
import SidebarSettings from '../components/SidebarSettings.vue'
import IntegrationsSettings from '../components/IntegrationsSettings.vue'
import SignalHubSettings from '../components/SignalHubSettings.vue'
import AISettingsControlCenter from '../components/AISettingsControlCenter.vue'

const { t } = useI18n()
const store = useSettingsStore()
const { data: appSettingsData } = useApplicationSettingsQuery()

const settingsTreeGroups: Array<{ label: string; items: Array<{ id: SettingsSection; label: string; icon: string }> }> = [
  {
    label: 'General',
    items: [
      { id: 'application', label: 'Application', icon: 'tabler:adjustments-horizontal' },
      { id: 'language', label: 'Language', icon: 'tabler:language' }
    ]
  },
  {
    label: 'Interface',
    items: [
      { id: 'appearance', label: 'Appearance', icon: 'tabler:palette' },
      { id: 'sidebar', label: 'Sidebar', icon: 'tabler:layout-sidebar' }
    ]
  },
  {
    label: 'Sources',
    items: [
      { id: 'integrations', label: 'Integrations', icon: 'tabler:plug-connected' },
      { id: 'signal-hub', label: 'Signal Hub', icon: 'tabler:database-import' }
    ]
  },
  {
    label: 'AI',
    items: [
      { id: 'ai', label: 'AI Control Center', icon: 'tabler:sparkles' }
    ]
  }
]

/** Number of provider accounts for the integrations badge */
const integrationCount = appSettingsData.value?.items?.length ?? 0
</script>

<template>
  <div class="settings-page">
    <!-- Action messages -->
    <div v-if="store.actionMessage" class="setup-state success">{{ store.actionMessage }}</div>
    <div v-if="store.errorMessage" class="inline-error">{{ store.errorMessage }}</div>

    <div class="settings-workbench">
      <!-- Navigation Tree -->
      <nav class="settings-tree" :aria-label="t('Settings sections')">
        <section
          v-for="group in settingsTreeGroups"
          :key="group.label"
          class="settings-tree-group"
        >
          <h2>{{ t(group.label) }}</h2>
          <button
            v-for="item in group.items"
            :key="item.id"
            type="button"
            :class="{ active: store.selectedSection === item.id }"
            @click="store.selectSection(item.id)"
          >
            <Icon class="tree-icon" :icon="item.icon" />
            <span>{{ t(item.label) }}</span>
            <em v-if="item.id === 'integrations'">{{ integrationCount }}</em>
          </button>
        </section>
      </nav>

      <!-- Content area -->
      <div class="settings-workbench-content">
        <AppearanceSettings v-if="store.selectedSection === 'appearance'" />
        <LanguageSettings v-else-if="store.selectedSection === 'language'" />
        <ApplicationSettings v-else-if="store.selectedSection === 'application'" />
        <SidebarSettings v-else-if="store.selectedSection === 'sidebar'" />
        <IntegrationsSettings v-else-if="store.selectedSection === 'integrations'" />
        <SignalHubSettings v-else-if="store.selectedSection === 'signal-hub'" />
        <AISettingsControlCenter v-else-if="store.selectedSection === 'ai'" />
      </div>
    </div>
  </div>
</template>

<style scoped>
.settings-page {
  display: flex;
  flex-direction: column;
  gap: var(--hh-layout-gap);
  height: 100%;
  min-width: 0;
  min-height: 0;
  overflow: hidden;
}

.settings-workbench {
  display: grid;
  grid-template-columns: 220px minmax(0, 1fr);
  gap: var(--hh-layout-gap);
  width: 100%;
  min-width: 0;
  min-height: 0;
  flex: 1;
  overflow: hidden;
}

/* Navigation tree */
.settings-tree {
  display: grid;
  align-content: start;
  gap: 14px;
  min-width: 0;
  min-height: 0;
  overflow-x: hidden;
  overflow-y: auto;
  border: 1px solid var(--hh-border-muted);
  border-radius: var(--hh-radius-md);
  background: rgba(4, 18, 20, var(--hh-panel-alpha));
  backdrop-filter: blur(var(--hh-panel-blur));
  box-shadow: var(--hh-shadow-panel);
  padding: 12px 8px;
}

.settings-tree-group {
  display: grid;
  gap: 5px;
}

.settings-tree-group h2 {
  margin: 0;
  color: var(--hh-text-muted);
  font-size: 10px;
  font-weight: 760;
  text-transform: uppercase;
  padding: 0 8px;
}

.settings-tree button {
  display: grid;
  grid-template-columns: 18px minmax(0, 1fr) auto;
  align-items: center;
  gap: 8px;
  min-height: 32px;
  border: 1px solid transparent;
  border-radius: var(--hh-radius-control);
  background: transparent;
  color: var(--hh-text-secondary);
  font-size: 12px;
  font-weight: 650;
  padding: 0 8px;
  text-align: left;
  cursor: pointer;
  transition: all 100ms ease;
}

.settings-tree button:hover,
.settings-tree button:focus-visible {
  border-color: var(--hh-border-accent-soft, var(--hh-accent));
  background: rgba(45, 240, 206, 0.06);
}

.settings-tree button.active {
  border-color: var(--hh-border-accent, var(--hh-accent));
  background: color-mix(in srgb, var(--hh-accent) 10%, transparent);
  color: var(--hh-accent);
}

.settings-tree button span {
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.settings-tree button em {
  color: var(--hh-text-muted);
  font-size: 10px;
  font-style: normal;
}

.tree-icon {
  width: 14px;
  height: 14px;
  color: currentColor;
}

/* Content */
.settings-workbench-content {
  min-width: 0;
  min-height: 0;
  overflow: hidden;
}

/* Messages */
.setup-state.success {
  padding: 8px 12px;
  background: color-mix(in srgb, var(--hh-status-success, #22c55e) 15%, transparent);
  border: 1px solid color-mix(in srgb, var(--hh-status-success) 30%, transparent);
  border-radius: var(--hh-radius-sm);
  color: var(--hh-status-success, #22c55e);
  font-size: 12px;
}

.inline-error {
  padding: 8px 12px;
  background: color-mix(in srgb, var(--hh-status-danger) 15%, transparent);
  border: 1px solid color-mix(in srgb, var(--hh-status-danger) 30%, transparent);
  border-radius: var(--hh-radius-sm);
  color: var(--hh-status-danger);
  font-size: 12px;
}

/* Responsive */
@media (max-width: 900px) {
  .settings-workbench {
    grid-template-columns: 180px minmax(0, 1fr);
  }
}
</style>
```

### `frontend/src/domains/tasks/components/TaskList.vue`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/domains/tasks/components/TaskList.vue`
- Size bytes / Размер в байтах: `5615`
- Included characters / Включено символов: `5613`
- Truncated / Обрезано: `no`

```text
<script setup lang="ts">
import { ref, computed } from 'vue'
import { useVirtualizer } from '@tanstack/vue-virtual'
import { useI18n } from '../../../platform/i18n'
import Icon from '../../../shared/ui/Icon.vue'
import type { Task, TaskCandidate, TaskCandidateReviewState } from '../types/task'
import { taskSourceLabel, taskConfidence, taskCreatedTime } from '../stores/tasks'

const { t } = useI18n()

const props = defineProps<{
  activeTasks: Task[]
  suggestedTaskCandidates: TaskCandidate[]
  isTasksLoading: boolean
  setTaskCandidateReview: (candidate: TaskCandidate, state: TaskCandidateReviewState) => Promise<void>
}>()

const parentRef = ref<HTMLElement | null>(null)

// Combine active tasks and suggested candidates into one virtual list
const allRows = computed<(Task | TaskCandidate)[]>(() => {
  const separator: TaskCandidate = {
    task_candidate_id: '__separator__',
    source_kind: 'message',
    source_id: '',
    project_id: null,
    title: t('Review Queue'),
    due_text: null,
    assignee_label: null,
    confidence: 0,
    review_state: 'suggested',
    evidence_excerpt: '',
    generated_at: '',
    reviewed_at: null,
    updated_at: ''
  }
  return [...props.activeTasks, separator, ...props.suggestedTaskCandidates]
})

const virtualizer = useVirtualizer(computed(() => ({
  count: allRows.value.length,
  getScrollElement: () => parentRef.value,
  estimateSize: () => 52,
  overscan: 5
})))

const virtualItems = computed(() => virtualizer.value.getVirtualItems())
const totalSize = computed(() => virtualizer.value.getTotalSize())

function isCandidate(item: Task | TaskCandidate): item is TaskCandidate {
  return 'task_candidate_id' in item && 'confidence' in item && 'review_state' in item
}

function isSeparator(item: Task | TaskCandidate): boolean {
  return 'task_candidate_id' in item && item.task_candidate_id === '__separator__'
}

function isTask(item: Task | TaskCandidate): item is Task {
  return 'task_id' in item && !('confidence' in item)
}
</script>

<template>
  <div class="widget-frame">
    <section class="panel task-table">
      <h3 class="task-group">{{ t('Active Tasks') }} <em>{{ props.activeTasks.length }}</em></h3>

      <div class="table-head task-table-head">
        <span>{{ t('Task') }}</span>
        <span>{{ t('Source') }}</span>
        <span>{{ t('Project') }}</span>
        <span>{{ t('Created') }}</span>
        <span>{{ t('Status') }}</span>
      </div>

      <!-- Loading state -->
      <div v-if="props.isTasksLoading" class="inline-copy">
        {{ t('Loading task state…') }}
      </div>

      <!-- Empty state -->
      <div v-else-if="props.activeTasks.length === 0 && props.suggestedTaskCandidates.length === 0" class="inline-copy">
        {{ t('No active tasks yet.') }}
      </div>

      <!-- Virtual list -->
      <div v-else ref="parentRef" class="virtual-list-container">
        <div class="virtual-list-inner" :style="{ height: `${totalSize}px`, width: '100%', position: 'relative' }">
          <div
            v-for="virtualRow in virtualItems"
            :key="(allRows[virtualRow.index] as any).task_id || (allRows[virtualRow.index] as any).task_candidate_id || virtualRow.index"
            :style="{
              position: 'absolute',
              top: 0,
              left: 0,
              width: '100%',
              height: `${virtualRow.size}px`,
              transform: `translateY(${virtualRow.start}px)`
            }"
          >
            <!-- Separator row -->
            <div v-if="isSeparator(allRows[virtualRow.index])" class="task-group task-review-separator">
              <h3>{{ t('Review Queue') }} <em>{{ props.suggestedTaskCandidates.length }}</em></h3>
            </div>

            <!-- Active task row -->
            <label v-else-if="isTask(allRows[virtualRow.index])" class="task-row">
              <input type="checkbox" disabled checked />
              <strong>{{ (allRows[virtualRow.index] as Task).title }}</strong>
              <span>{{ taskSourceLabel(allRows[virtualRow.index] as Task) }}</span>
              <span>{{ (allRows[virtualRow.index] as Task).project_id ?? t('Unassigned') }}</span>
              <time>{{ taskCreatedTime((allRows[virtualRow.index] as Task).created_at) }}</time>
              <em>{{ (allRows[virtualRow.index] as Task).makosh_status }}</em>
            </label>

            <!-- Candidate row -->
            <div v-else class="task-row task-row-actions">
              <strong>{{ (allRows[virtualRow.index] as TaskCandidate).title }}</strong>
              <span>{{ taskSourceLabel(allRows[virtualRow.index] as TaskCandidate) }}</span>
              <span>{{ (allRows[virtualRow.index] as TaskCandidate).project_id ?? t('Unassigned') }}</span>
              <em>{{ taskConfidence(allRows[virtualRow.index] as TaskCandidate) }}</em>
              <div class="task-actions">
                <button type="button" @click="props.setTaskCandidateReview(allRows[virtualRow.index] as TaskCandidate, 'user_confirmed')">
                  <Icon icon="tabler:check" :size="15" /> {{ t('Confirm') }}
                </button>
                <button type="button" @click="props.setTaskCandidateReview(allRows[virtualRow.index] as TaskCandidate, 'user_rejected')">
                  <Icon icon="tabler:x" :size="15" /> {{ t('Reject') }}
                </button>
              </div>
            </div>
          </div>
        </div>
      </div>
    </section>
  </div>
</template>

<style scoped>
.virtual-list-container {
  max-height: 600px;
  overflow-y: auto;
}

.task-review-separator {
  padding: 8px 12px;
}
</style>
```

### `frontend/src/domains/tasks/components/TasksDecisionObligationReview.vue`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/domains/tasks/components/TasksDecisionObligationReview.vue`
- Size bytes / Размер в байтах: `5590`
- Included characters / Включено символов: `5588`
- Truncated / Обрезано: `no`

```text
<script setup lang="ts">
import { computed } from 'vue'
import { useI18n } from '../../../platform/i18n'
import Icon from '../../../shared/ui/Icon.vue'
import type {
  Decision,
  DecisionEntityKind,
  Obligation
} from '../types/task'
import { formatDecisionTime, formatDecisionEntity, formatObligationDueTime, formatObligationEntity } from '../stores/tasks'

const { t } = useI18n()

const entityKindOptions: DecisionEntityKind[] = [
  'project', 'task', 'persona', 'communication',
  'document', 'event', 'organization', 'knowledge'
]

const props = defineProps<{
  decisions: Decision[]
  obligations: Obligation[]
  entityKind: DecisionEntityKind
  entityId: string
  isLoading: boolean
  error: string
  reviewingItemId: string | null
  onEntityKindChange: (entityKind: DecisionEntityKind) => void
  onEntityIdChange: (entityId: string) => void
  onReload: () => Promise<void>
  onReviewDecision: (decision: Decision, reviewState: 'user_confirmed' | 'user_rejected') => Promise<void>
  onReviewObligation: (obligation: Obligation, reviewState: 'user_confirmed' | 'user_rejected') => Promise<void>
}>()

const hasScope = computed(() => props.entityId.trim().length > 0)
const reviewItemCount = computed(() => props.decisions.length + props.obligations.length)

function decisionReviewId(decision: Decision): string {
  return `decision:${decision.decision_id}`
}

function obligationReviewId(obligation: Obligation): string {
  return `obligation:${obligation.obligation_id}`
}
</script>

<template>
  <section class="widget-frame task-context-review-panel" :aria-busy="props.isLoading">
    <header>
      <div>
        <span class="panel-kicker">{{ t('Context') }}</span>
        <h2>{{ t('Decision & Obligation Review') }}</h2>
      </div>
      <button type="button" :title="t('Reload review items')" @click="props.onReload" :disabled="props.isLoading">
        <Icon icon="tabler:refresh" :size="15" />
      </button>
    </header>

    <div class="task-context-review-scope">
      <select
        :aria-label="t('Entity kind')"
        :value="props.entityKind"
        @change="props.onEntityKindChange(($event.target as HTMLSelectElement).value as DecisionEntityKind)"
      >
        <option v-for="option in entityKindOptions" :key="option" :value="option">{{ t(option) }}</option>
      </select>
      <input
        :aria-label="t('Entity id')"
        :value="props.entityId"
        :placeholder="t('Entity id')"
        @input="props.onEntityIdChange(($event.target as HTMLInputElement).value)"
      />
    </div>

    <!-- Error state -->
    <div v-if="props.error" class="task-context-review-state error">
      <span>{{ props.error }}</span>
      <button type="button" @click="props.onReload" :disabled="props.isLoading">{{ t('Retry') }}</button>
    </div>

    <!-- Loading state -->
    <div v-else-if="props.isLoading" class="task-context-review-state">
      <span>{{ hasScope ? t('Loading review items') : t('Loading global review items') }}</span>
    </div>

    <!-- Empty state -->
    <div v-else-if="reviewItemCount === 0" class="task-context-review-state">
      <span>{{ t('No open decisions or obligations') }}</span>
    </div>

    <!-- Review list -->
    <div v-else class="task-context-review-list">
      <article v-for="decision in props.decisions" :key="decision.decision_id" class="task-context-review-item">
        <div>
          <span class="panel-kicker">{{ t('Decision') }}</span>
          <strong>{{ decision.title }}</strong>
          <p>{{ decision.rationale }}</p>
          <small>{{ formatDecisionEntity(decision.decided_by_entity_kind, decision.decided_by_entity_id) }} · {{ formatDecisionTime(decision.decided_at) }}</small>
        </div>
        <div class="task-context-review-actions">
          <button
            type="button"
            :disabled="props.reviewingItemId === decisionReviewId(decision)"
            @click="props.onReviewDecision(decision, 'user_confirmed')"
          >
            <Icon icon="tabler:check" :size="14" /> {{ t('Confirm') }}
          </button>
          <button
            type="button"
            :disabled="props.reviewingItemId === decisionReviewId(decision)"
            @click="props.onReviewDecision(decision, 'user_rejected')"
          >
            <Icon icon="tabler:x" :size="14" /> {{ t('Reject') }}
          </button>
        </div>
      </article>

      <article v-for="obligation in props.obligations" :key="obligation.obligation_id" class="task-context-review-item">
        <div>
          <span class="panel-kicker">{{ t('Obligation') }}</span>
          <strong>{{ obligation.statement }}</strong>
          <p>{{ formatObligationEntity(obligation.obligated_entity_kind, obligation.obligated_entity_id) }}</p>
          <small>{{ t(obligation.risk_state) }} · {{ formatObligationDueTime(obligation.due_at) }}</small>
        </div>
        <div class="task-context-review-actions">
          <button
            type="button"
            :disabled="props.reviewingItemId === obligationReviewId(obligation)"
            @click="props.onReviewObligation(obligation, 'user_confirmed')"
          >
            <Icon icon="tabler:check" :size="14" /> {{ t('Confirm') }}
          </button>
          <button
            type="button"
            :disabled="props.reviewingItemId === obligationReviewId(obligation)"
            @click="props.onReviewObligation(obligation, 'user_rejected')"
          >
            <Icon icon="tabler:x" :size="14" /> {{ t('Reject') }}
          </button>
        </div>
      </article>
    </div>
  </section>
</template>
```

### `frontend/src/domains/tasks/views/TasksPage.vue`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/domains/tasks/views/TasksPage.vue`
- Size bytes / Размер в байтах: `8175`
- Included characters / Включено символов: `8175`
- Truncated / Обрезано: `no`

```text
<script setup lang="ts">
import { computed, onMounted } from 'vue'
import { useI18n } from '../../../platform/i18n'
import Icon from '../../../shared/ui/Icon.vue'
import { useTaskCandidatesQuery, useTasksQuery } from '../queries/useTasksQuery'
import { useTasksStore } from '../stores/tasks'
import { fetchDecisions, fetchDecisionReviewItems, reviewDecision, fetchObligations, fetchObligationReviewItems, reviewObligation } from '../api/tasks'
import { reviewTaskCandidate } from '../api/tasks'
import TaskList from '../components/TaskList.vue'
import TasksDecisionObligationReview from '../components/TasksDecisionObligationReview.vue'
import type { TaskCandidate, Task, Decision, Obligation, DecisionEntityKind, TaskCandidateReviewState } from '../types/task'

const { t } = useI18n()
const store = useTasksStore()

const { data: candidatesData, isLoading: isCandidatesLoading, refetch: refetchCandidates } = useTaskCandidatesQuery()
const { data: tasksData, isLoading: isTasksLoading, refetch: refetchTasks } = useTasksQuery()

// Derived state
const taskCandidates = computed<TaskCandidate[]>(() => candidatesData.value ?? [])
const activeTasks = computed<Task[]>(() => tasksData.value ?? [])
const isTasksLoadingCombined = computed<boolean>(() => isCandidatesLoading.value || isTasksLoading.value)

const suggestedTaskCandidates = computed<TaskCandidate[]>(() =>
  taskCandidates.value.filter((item) => item.review_state === 'suggested')
)

async function loadContextReview() {
  const entityId = store.reviewEntityId.trim()
  store.setContextReviewLoading(true)

  try {
    let decisionResult: { items: Decision[] }
    let obligationResult: { items: Obligation[] }

    if (entityId) {
      ;[decisionResult, obligationResult] = await Promise.all([
        fetchDecisions({ entityKind: store.reviewEntityKind, entityId, limit: 50 }),
        fetchObligations({ entityKind: store.reviewEntityKind as any, entityId, limit: 50 })
      ])
    } else {
      ;[decisionResult, obligationResult] = await Promise.all([
        fetchDecisionReviewItems({ reviewState: 'suggested', limit: 50 }),
        fetchObligationReviewItems({ reviewState: 'suggested', limit: 50 })
      ])
    }
    store.setDecisions(decisionResult.items)
    store.setObligations(obligationResult.items)
    store.setContextReviewError('')
  } catch (e: unknown) {
    store.setDecisions([])
    store.setObligations([])
    store.setContextReviewError(e instanceof Error ? e.message : 'Unknown context review error')
  }
  store.setContextReviewLoading(false)
}

async function reviewDecisionItem(decision: Decision, reviewState: 'user_confirmed' | 'user_rejected') {
  store.setReviewingItemId(`decision:${decision.decision_id}`)
  try {
    await reviewDecision(decision.decision_id, { review_state: reviewState })
    await loadContextReview()
  } catch (e: unknown) {
    store.setContextReviewError(e instanceof Error ? e.message : 'Unknown decision review error')
  }
  store.setReviewingItemId(null)
}

async function reviewObligationItem(obligation: Obligation, reviewState: 'user_confirmed' | 'user_rejected') {
  store.setReviewingItemId(`obligation:${obligation.obligation_id}`)
  try {
    await reviewObligation(obligation.obligation_id, { review_state: reviewState })
    await loadContextReview()
  } catch (e: unknown) {
    store.setContextReviewError(e instanceof Error ? e.message : 'Unknown obligation review error')
  }
  store.setReviewingItemId(null)
}

async function loadTasks() {
  await Promise.all([refetchCandidates(), refetchTasks()])
}

async function setTaskCandidateReview(candidate: TaskCandidate, state: TaskCandidateReviewState) {
  try {
    await reviewTaskCandidate(candidate.task_candidate_id, state)
    store.clearError()
    await loadTasks()
  } catch (e: unknown) {
    store.setError(e instanceof Error ? e.message : 'Unknown task candidate review error')
  }
}

onMounted(() => {
  loadContextReview()
})
</script>

<template>
  <section class="tasks-page">
    <div class="view-header">
      <div class="view-title-with-icon">
        <span class="hero-mark small"><Icon icon="tabler:hexagon" :size="28" /></span>
        <div>
          <h1>{{ t('Tasks') }}</h1>
          <p>{{ t('All your tasks from connected trackers') }}</p>
        </div>
      </div>

      <!-- Metrics -->
      <div class="widget-frame inline-metrics">
        <div class="metric-grid inline-metrics">
          <article class="metric-card">
            <span>{{ t('Active Tasks') }}</span>
            <strong>{{ activeTasks.length }}</strong>
            <small>{{ t('Active records') }}</small>
          </article>
          <article class="metric-card">
            <span>{{ t('Suggested Candidates') }}</span>
            <strong>{{ suggestedTaskCandidates.length }}</strong>
            <small>{{ t('Ready for review') }}</small>
          </article>
          <article class="metric-card">
            <span>{{ t('Review State') }}</span>
            <strong>{{ store.tasksError ? t('Error') : t('Ready') }}</strong>
            <small>{{ store.tasksError ? t('Show message below') : t('Live API') }}</small>
          </article>
        </div>
      </div>

      <button type="button" class="primary-button" disabled>
        <Icon icon="tabler:sparkles" :size="16" />{{ t('AI refresh') }}
      </button>
    </div>

    <p v-if="store.tasksError" class="inline-error">{{ store.tasksError }}</p>

    <div class="tasks-layout">
      <TaskList
        :activeTasks="activeTasks"
        :suggestedTaskCandidates="suggestedTaskCandidates"
        :isTasksLoading="isTasksLoadingCombined"
        :setTaskCandidateReview="setTaskCandidateReview"
      />

      <aside class="stacked-rail">
        <!-- Review Stats -->
        <div class="widget-frame">
          <section class="panel chart-panel">
            <h2>{{ t('Review Stats') }}</h2>
            <div class="donut">
              <strong>{{ taskCandidates.length }}</strong>
              <span>{{ t('Suggestions') }}</span>
            </div>
            <ul>
              <li>{{ suggestedTaskCandidates.length }} {{ t('Suggested') }}</li>
              <li>{{ activeTasks.length }} {{ t('Active') }}</li>
              <li>{{ taskCandidates.length - suggestedTaskCandidates.length - activeTasks.length }} {{ t('Done') }}</li>
            </ul>
          </section>
        </div>

        <!-- Decision & Obligation Review -->
        <TasksDecisionObligationReview
          :decisions="store.decisions"
          :obligations="store.obligations"
          :entityKind="store.reviewEntityKind"
          :entityId="store.reviewEntityId"
          :isLoading="store.isContextReviewLoading"
          :error="store.contextReviewError"
          :reviewingItemId="store.reviewingContextItemId"
          :onEntityKindChange="store.setReviewEntityKind"
          :onEntityIdChange="store.setReviewEntityId"
          :onReload="loadContextReview"
          :onReviewDecision="reviewDecisionItem"
          :onReviewObligation="reviewObligationItem"
        />

        <!-- Recent Candidate Signals -->
        <div class="widget-frame">
          <section class="panel info-card">
            <h2>{{ t('Recent Candidate Signals') }}</h2>
            <template v-if="suggestedTaskCandidates.length === 0">
              <p class="muted-copy">{{ t('No pending candidate signals.') }}</p>
            </template>
            <template v-else>
              <div v-for="candidate in suggestedTaskCandidates.slice(0, 5)" :key="candidate.task_candidate_id" class="deadline">
                <span>{{ candidate.title }}</span>
                <time>{{ candidate.source_kind }}</time>
              </div>
            </template>
          </section>
        </div>

        <!-- Active Task Sources -->
        <div class="widget-frame">
          <section class="panel info-card">
            <h2>{{ t('Active Task Sources') }}</h2>
            <div v-for="source in ['message', 'document']" :key="source" class="bar-row">
              <span>{{ source }}</span>
              <div><i></i></div>
            </div>
          </section>
        </div>
      </aside>
    </div>
  </section>
</template>
```

### `frontend/src/domains/timeline/components/TimelineFilters.vue`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/domains/timeline/components/TimelineFilters.vue`
- Size bytes / Размер в байтах: `740`
- Included characters / Включено символов: `740`
- Truncated / Обрезано: `no`

```text
<script setup lang="ts">
import type { TimelineFilters as TimelineFiltersType } from '../types/timeline'

interface Props {
	filters: TimelineFiltersType
}

interface Emits {
	(e: 'toggleFilter', kind: keyof TimelineFiltersType): void
}

const props = defineProps<Props>()
const emit = defineEmits<Emits>()

const filterItems: Array<keyof TimelineFiltersType> = ['Messages', 'Documents', 'Tasks', 'Calendar', 'Notes', 'Decisions']
</script>

<template>
	<section class="panel info-card">
		<h2>Timeline Filters</h2>
		<label v-for="item in filterItems" :key="item" class="mini-check">
			<input
				type="checkbox"
				:checked="filters[item]"
				@change="emit('toggleFilter', item)"
			/>
			{{ item }}
		</label>
	</section>
</template>
```

### `frontend/src/domains/timeline/components/TimelineStream.vue`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/domains/timeline/components/TimelineStream.vue`
- Size bytes / Размер в байтах: `2507`
- Included characters / Включено символов: `2507`
- Truncated / Обрезано: `no`

```text
<script setup lang="ts">
import { ref, computed } from 'vue'
import { useVirtualizer } from '@tanstack/vue-virtual'
import Icon from '../../../shared/ui/Icon.vue'
import type { TimelineMessage } from '../types/timeline'

interface Props {
	messages: TimelineMessage[]
}

const props = defineProps<Props>()

const parentRef = ref<HTMLElement | null>(null)

const virtualizer = useVirtualizer(computed(() => ({
	count: props.messages.length,
	getScrollElement: () => parentRef.value,
	estimateSize: () => 72,
	overscan: 10
})))

const virtualItems = computed(() => virtualizer.value.getVirtualItems())
const totalSize = computed(() => virtualizer.value.getTotalSize())
</script>

<template>
	<section class="panel feed-panel large-timeline" ref="parentRef">
		<header class="panel-title-row">
			<h2>Today</h2>
			<button type="button" class="ghost-button" disabled>All Events</button>
		</header>

		<div :style="{ height: `${totalSize}px`, position: 'relative' }">
			<article
				v-for="virtualRow in virtualItems"
				:key="String(virtualRow.key)"
				class="timeline-event-row"
				:style="{
					position: 'absolute',
					top: 0,
					left: 0,
					width: '100%',
					transform: `translateY(${virtualRow.start}px)`
				}"
			>
				<span class="rail-dot"></span>
				<span class="round-icon blue"><Icon icon="tabler:message" width="20" height="20" /></span>
				<div>
					<strong>{{ messages[virtualRow.index].sender_display_name || messages[virtualRow.index].sender || 'Unknown' }}</strong>
					<p>{{ messages[virtualRow.index].subject || messages[virtualRow.index].body_text_preview }}</p>
					<time>{{ messages[virtualRow.index].occurred_at || messages[virtualRow.index].projected_at }}</time>
				</div>
			</article>
		</div>
	</section>
</template>

<style scoped>
.large-timeline {
	max-height: 100%;
	overflow-y: auto;
	padding: 0 0 12px;
}

.timeline-event-row {
	display: grid;
	grid-template-columns: 64px 18px 40px 1fr;
	gap: 10px;
	min-height: 72px;
	border-bottom: 1px solid rgba(102, 189, 180, 0.08);
	padding: 12px 18px;
}

.timeline-event-row time {
	color: var(--hh-color-accent);
	font-size: 12px;
}

.rail-dot {
	width: 8px;
	height: 8px;
	margin-top: 8px;
	border-radius: var(--hh-radius-round);
	background: var(--hh-color-accent);
	box-shadow: 0 0 12px rgba(45, 240, 206, 0.85);
}

.timeline-event-row strong {
	color: var(--hh-color-text-bright);
	font-size: 13px;
}

.timeline-event-row p {
	margin-top: 5px;
	color: var(--hh-color-text-muted);
	font-size: 12px;
}
</style>
```

### `frontend/src/domains/timeline/views/TimelinePage.vue`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/domains/timeline/views/TimelinePage.vue`
- Size bytes / Размер в байтах: `1929`
- Included characters / Включено символов: `1929`
- Truncated / Обрезано: `no`

```text
<script setup lang="ts">
import { watch } from 'vue'
import Icon from '../../../shared/ui/Icon.vue'
import { useTimelineMessagesQuery } from '../queries/useTimelineQuery'
import { useTimelineStore } from '../stores/timeline'
import TimelineStream from '../components/TimelineStream.vue'
import TimelineFilters from '../components/TimelineFilters.vue'

const store = useTimelineStore()

const { data: messagesData, isLoading } = useTimelineMessagesQuery()

watch(messagesData, (val) => {
	if (val) {
		store.setMessages(val)
		store.setLoading(false)
	}
})

watch(isLoading, (val) => {
	store.setLoading(val)
})
</script>

<template>
	<section class="timeline-page">
		<div class="view-header">
			<div class="view-title-with-icon">
				<span class="hero-mark small"><Icon icon="tabler:timeline-event" width="28" height="28" /></span>
				<div>
					<h1>Timeline</h1>
					<p>Chronological activity across connected sources.</p>
				</div>
			</div>
		</div>
		<div class="timeline-layout">
			<TimelineStream :messages="store.filteredMessages" />
			<aside class="stacked-rail">
				<TimelineFilters :filters="store.filters" @toggle-filter="store.toggleFilter($event)" />
			</aside>
		</div>
	</section>
</template>

<style scoped>
.timeline-page {
	display: grid;
	grid-template-columns: repeat(var(--hh-layout-columns), minmax(0, 1fr));
	grid-auto-flow: row;
	grid-auto-rows: min-content;
	align-content: start;
	gap: var(--hh-layout-gap);
	height: 100%;
	min-height: 0;
	overflow: hidden;
	padding-right: 0;
}

.timeline-page > * {
	grid-column: 1 / -1;
	min-width: 0;
}

.timeline-layout {
	--hh-zone-rows: 12;

	display: grid;
	grid-template-columns: repeat(var(--hh-layout-columns), minmax(0, 1fr));
	grid-auto-flow: dense;
	grid-auto-rows: min-content;
	align-content: start;
	align-items: stretch;
	gap: var(--hh-layout-gap);
	width: 100%;
	min-width: 0;
	min-height: 0;
	max-height: 100%;
	overflow: hidden;
}
</style>
```

### `frontend/src/integrations/mail/components/AccountSetupModal.vue`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/integrations/mail/components/AccountSetupModal.vue`
- Size bytes / Размер в байтах: `13384`
- Included characters / Включено символов: `12000`
- Truncated / Обрезано: `yes`

```text
<script setup lang="ts">
import { computed, ref } from 'vue'
import { useForm } from 'vee-validate'
import { loadFrontendConfig } from '../../../platform/config/env'
import Icon from '../../../shared/ui/Icon.vue'
import Button from '../../../shared/ui/Button.vue'
import Dialog from '../../../shared/ui/Dialog.vue'
import {
  useSetupImapEmailAccountMutation,
  useStartGmailOAuthSetupMutation
} from '../queries/accountSetupQueries'
import {
  accountSetupFormDefaults,
  accountSetupFormToGmailOAuthStart,
  accountSetupFormToImapRequest,
  accountSetupVeeValidationSchema,
  type AccountSetupFormValues,
  type MailAccountSetupProvider
} from '../forms/accountSetupForm'

const emit = defineEmits<{
  close: []
}>()

const step = ref(1)
const setupError = ref('')
const setupStatusMessage = ref('')
const frontendConfig = loadFrontendConfig()
const gmailOAuthSetupMutation = useStartGmailOAuthSetupMutation()
const imapEmailAccountSetupMutation = useSetupImapEmailAccountMutation()

const providerOptions: { kind: MailAccountSetupProvider; label: string; icon: string; description: string }[] = [
  { kind: 'gmail', label: 'Gmail', icon: 'tabler:brand-google', description: 'Google OAuth account setup' },
  { kind: 'icloud', label: 'iCloud', icon: 'tabler:brand-apple', description: 'iCloud Mail with app password' },
  { kind: 'imap', label: 'IMAP', icon: 'tabler:mail', description: 'Generic IMAP and SMTP account' }
]

const {
  errors,
  handleSubmit,
  isSubmitting,
  resetForm,
  setFieldValue,
  values: formValues
} = useForm<AccountSetupFormValues>({
  validationSchema: accountSetupVeeValidationSchema,
  initialValues: accountSetupFormDefaults('icloud')
})

const selectedProvider = computed(() => formValues.provider_kind)
const selectedProviderInfo = computed(() =>
  providerOptions.find(p => p.kind === selectedProvider.value)
)
const submitLabel = computed(() => {
  if (isSubmitting.value) return selectedProvider.value === 'gmail' ? 'Starting...' : 'Connecting...'
  return selectedProvider.value === 'gmail' ? 'Continue with Google' : 'Connect Account'
})

function selectProvider(kind: MailAccountSetupProvider) {
  resetForm({ values: accountSetupFormDefaults(kind) })
  setupError.value = ''
  setupStatusMessage.value = ''
  step.value = 2
}

function goBack() {
  if (step.value > 1) {
    step.value--
    setupError.value = ''
    setupStatusMessage.value = ''
  }
}

const submitAccountSetup = handleSubmit(async (values) => {
  setupError.value = ''
  setupStatusMessage.value = ''

  try {
    if (values.provider_kind === 'gmail') {
      const response = await gmailOAuthSetupMutation.mutateAsync(
        accountSetupFormToGmailOAuthStart(values, frontendConfig.apiBaseUrl)
      )
      window.open(response.authorization_url, '_blank', 'noopener,noreferrer')
      setupStatusMessage.value = 'Google authorization opened'
      return
    }

    await imapEmailAccountSetupMutation.mutateAsync(accountSetupFormToImapRequest(values))
    emit('close')
  } catch (e) {
    setupError.value = e instanceof Error ? e.message : 'Setup failed'
  }
})

function updateStringField(
  key: keyof AccountSetupFormValues,
  event: Event
) {
  setFieldValue(key, (event.target as HTMLInputElement).value)
}

function updateNumberField(
  key: keyof AccountSetupFormValues,
  event: Event
) {
  setFieldValue(key, Number((event.target as HTMLInputElement).value))
}

function updateBooleanField(
  key: keyof AccountSetupFormValues,
  event: Event
) {
  setFieldValue(key, (event.target as HTMLInputElement).checked)
}

function handleClose() {
  emit('close')
}
</script>

<template>
  <Dialog :open="true" content-class="account-setup-dialog" @update:open="(open) => { if (!open) handleClose() }">
    <template #header>
      <div class="modal-header">
        <div class="modal-header-left">
          <Button v-if="step > 1" variant="ghost" size="sm" @click="goBack">
            <Icon icon="tabler:arrow-left" />
          </Button>
          <h2 v-if="step === 1">Add Mail Account</h2>
          <h2 v-else-if="step === 2">Configure {{ selectedProviderInfo?.label }}</h2>
        </div>
      </div>
    </template>

    <div class="setup-modal">
      <!-- Step 1: Provider selection -->
      <div v-if="step === 1" class="provider-selection">
        <p class="step-desc">Select a mail provider to connect</p>
        <div class="provider-grid">
          <button
            v-for="provider in providerOptions"
            :key="provider.kind"
            class="provider-card"
            type="button"
            @click="selectProvider(provider.kind)"
          >
            <Icon :icon="provider.icon" class="provider-icon" />
            <span class="provider-label">{{ provider.label }}</span>
            <span class="provider-desc">{{ provider.description }}</span>
          </button>
        </div>
      </div>

      <!-- Step 2: Account details -->
      <div v-else-if="step === 2" class="account-details">
        <p class="step-desc">Enter your {{ selectedProviderInfo?.label }} account details</p>

        <div class="form-fields">
          <div class="field">
            <label>Account Name</label>
            <input
              type="text"
              :value="formValues.display_name"
              placeholder="e.g., Personal Gmail"
              @input="updateStringField('display_name', $event)"
            />
          </div>
          <div class="field">
            <label>Email Address</label>
            <input
              type="email"
              :value="formValues.email"
              placeholder="you@example.com"
              @input="updateStringField('email', $event)"
            />
            <span v-if="errors.email" class="field-error">{{ errors.email }}</span>
          </div>

          <template v-if="selectedProvider === 'imap'">
            <div class="field">
              <label>IMAP Host</label>
              <input
                type="text"
                :value="formValues.imap_host"
                placeholder="imap.example.com"
                @input="updateStringField('imap_host', $event)"
              />
              <span v-if="errors.imap_host" class="field-error">{{ errors.imap_host }}</span>
            </div>
            <div class="field-row">
              <div class="field">
                <label>IMAP Port</label>
                <input
                  type="number"
                  :value="formValues.imap_port"
                  @input="updateNumberField('imap_port', $event)"
                />
              </div>
              <div class="field">
                <label>Username</label>
                <input
                  type="text"
                  :value="formValues.username"
                  placeholder="user@example.com"
                  @input="updateStringField('username', $event)"
                />
              </div>
            </div>
            <div class="field">
              <label>Password</label>
              <input
                type="password"
                :value="formValues.password"
                placeholder="Mailbox password"
                @input="updateStringField('password', $event)"
              />
              <span v-if="errors.password" class="field-error">{{ errors.password }}</span>
            </div>
            <div class="field">
              <label>SMTP Host</label>
              <input
                type="text"
                :value="formValues.smtp_host"
                placeholder="smtp.example.com"
                @input="updateStringField('smtp_host', $event)"
              />
            </div>
            <div class="field-row">
              <div class="field">
                <label>SMTP Port</label>
                <input
                  type="number"
                  :value="formValues.smtp_port"
                  @input="updateNumberField('smtp_port', $event)"
                />
              </div>
              <div class="field checkbox-field">
                <label class="checkbox-label">
                  <input
                    type="checkbox"
                    :checked="formValues.imap_tls"
                    @change="updateBooleanField('imap_tls', $event)"
                  />
                  IMAP TLS
                </label>
              </div>
            </div>
            <div class="field-row">
              <div class="field checkbox-field">
                <label class="checkbox-label">
                  <input
                    type="checkbox"
                    :checked="formValues.smtp_tls"
                    @change="updateBooleanField('smtp_tls', $event)"
                  />
                  SMTP TLS
                </label>
              </div>
              <div class="field checkbox-field">
                <label class="checkbox-label">
                  <input
                    type="checkbox"
                    :checked="formValues.smtp_starttls"
                    @change="updateBooleanField('smtp_starttls', $event)"
                  />
                  SMTP STARTTLS
                </label>
              </div>
            </div>
          </template>

          <div v-if="selectedProvider === 'icloud'" class="field">
            <label>App Password</label>
            <input
              type="password"
              :value="formValues.password"
              placeholder="Your app-specific password"
              @input="updateStringField('password', $event)"
            />
            <span v-if="errors.password" class="field-error">{{ errors.password }}</span>
          </div>
        </div>

        <div v-if="setupError" class="setup-error">{{ setupError }}</div>
        <div v-if="setupStatusMessage" class="setup-status">{{ setupStatusMessage }}</div>

        <div class="form-actions">
          <Button variant="default" @click="submitAccountSetup" :loading="isSubmitting">
            {{ submitLabel }}
          </Button>
          <Button variant="ghost" @click="goBack">Back</Button>
        </div>
      </div>
    </div>
  </Dialog>
</template>

<style scoped>
:deep(.account-setup-dialog) {
  max-width: 520px;
}

.setup-modal {
  width: 100%;
  display: flex;
  flex-direction: column;
}

.modal-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding-right: 2.5rem;
  border-bottom: 1px solid var(--hh-border, #e5e7eb);
}

.modal-header-left {
  display: flex;
  align-items: center;
  gap: 0.5rem;
}

.modal-header-left h2 {
  margin: 0;
  font-size: 1rem;
  font-weight: 600;
}

.step-desc {
  margin: 0 0 1rem;
  font-size: 0.875rem;
  color: var(--hh-text-secondary, #6b7280);
}

.provider-selection,
.account-details {
  padding: 1rem;
  overflow-y: auto;
}

.provider-grid {
  display: flex;
  flex-direction: column;
  gap: 0.5rem;
}

.provider-card {
  display: flex;
  align-items: center;
  gap: 0.75rem;
  width: 100%;
  padding: 0.75rem 1rem;
  border: 1px solid var(--hh-border, #e5e7eb);
  border-radius: 0.5rem;
  background: transparent;
  color: inherit;
  cursor: pointer;
  font: inherit;
  text-align: left;
  transition: background 0.1s, border-color 0.1s;
}

.provider-card:hover {
  background: var(--hh-bg-hover, #f3f4f6);
  border-color: var(--hh-accent, #3b82f6);
}

.provider-icon {
  width: 28px;
  height: 28px;
  color: var(--hh-accent, #3b82f6);
  flex-shrink: 0;
}

.provider-label {
  font-weight: 500;
  font-size: 0.875rem;
  color: var(--hh-text-primary, #1f2937);
}

.provider-desc {
  font-size: 0.75rem;
  color: var(--hh-text-secondary, #6b7280);
  margin-left: auto;
}

.form-fields {
  display: flex;
  flex-direction: column;
  gap: 0.75rem;
}

.field {
  display: flex;
  flex-direction: column;
  gap: 0.25rem;
}

.field label {
  font-size: 0.75rem;
  font-weight: 500;
  color: var(--hh-text-secondary, #6b7280);
}

.field input[type="text"],
.field input[type="email"],
.field input[type="password"],
.field input[type="number"] {
  padding: 0.5rem 0.625rem;
  border: 1p
```
_Source file truncated after 12000 characters. / Исходный файл обрезан после 12000 символов._

### `frontend/src/integrations/mail/components/MailSyncSettingsStrip.vue`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/integrations/mail/components/MailSyncSettingsStrip.vue`
- Size bytes / Размер в байтах: `5313`
- Included characters / Включено символов: `5313`
- Truncated / Обрезано: `no`

```text
<script setup lang="ts">
import { computed, watch } from 'vue'
import { useForm } from 'vee-validate'
import Button from '../../../shared/ui/Button.vue'
import Icon from '../../../shared/ui/Icon.vue'
import {
  syncSettingsFormDefaults,
  syncSettingsFormToUpdate,
  syncSettingsVeeValidationSchema,
  type SyncSettingsFormValues
} from '../forms/syncSettingsForm'
import type {
  MailSyncSettings,
  MailSyncSettingsUpdate
} from '../../../shared/mailSync/types'

const props = defineProps<{
  settings: MailSyncSettings | null
  isLoading: boolean
  isSaving: boolean
}>()

const emit = defineEmits<{
  update: [settings: MailSyncSettingsUpdate]
}>()

const {
  errors,
  handleSubmit,
  setFieldValue,
  setValues,
  values: formValues
} = useForm<SyncSettingsFormValues>({
  validationSchema: syncSettingsVeeValidationSchema,
  initialValues: syncSettingsFormDefaults(props.settings)
})

const isDisabled = computed(() => props.isLoading || props.isSaving || !props.settings)
const syncStateLabel = computed(() => (formValues.sync_enabled ? 'Enabled' : 'Paused'))

watch(
  () => props.settings,
  (settings) => setValues(syncSettingsFormDefaults(settings)),
  { immediate: true }
)

const submitSettings = handleSubmit((values) => {
  emit('update', syncSettingsFormToUpdate(values))
})

function updateBooleanField(event: Event): void {
  const input = event.target as HTMLInputElement
  setFieldValue('sync_enabled', input.checked)
}

function updateNumberField(field: 'batch_size' | 'poll_interval_seconds', event: Event): void {
  const input = event.target as HTMLInputElement
  setFieldValue(field, Number(input.value))
}
</script>

<template>
  <section v-if="settings || isLoading" class="mail-sync-settings-strip" aria-label="Provider sync settings">
    <div class="sync-settings-heading">
      <Icon icon="tabler:refresh-dot" class="sync-settings-icon" />
      <div>
        <div class="sync-settings-title">Provider sync</div>
        <div class="sync-settings-meta">
          <span v-if="isLoading">Loading settings...</span>
          <span v-else>{{ syncStateLabel }}</span>
        </div>
      </div>
    </div>

    <form class="sync-settings-form" @submit.prevent="submitSettings">
      <label class="sync-toggle">
        <input
          :checked="formValues.sync_enabled"
          type="checkbox"
          :disabled="isDisabled"
          @change="updateBooleanField"
        />
        <span>Sync</span>
      </label>

      <label class="sync-field">
        <span>Batch</span>
        <input
          :value="formValues.batch_size"
          type="number"
          min="1"
          max="500"
          step="1"
          :disabled="isDisabled"
          @input="updateNumberField('batch_size', $event)"
        />
        <small v-if="errors.batch_size">{{ errors.batch_size }}</small>
      </label>

      <label class="sync-field">
        <span>Poll, sec</span>
        <input
          :value="formValues.poll_interval_seconds"
          type="number"
          min="60"
          max="86400"
          step="60"
          :disabled="isDisabled"
          @input="updateNumberField('poll_interval_seconds', $event)"
        />
        <small v-if="errors.poll_interval_seconds">{{ errors.poll_interval_seconds }}</small>
      </label>

      <Button variant="outline" size="sm" :disabled="isDisabled" :loading="isSaving" type="submit">
        Save
      </Button>
    </form>
  </section>
</template>

<style scoped>
.mail-sync-settings-strip {
  display: grid;
  grid-template-columns: minmax(10rem, 0.4fr) minmax(0, 1fr);
  gap: 0.75rem;
  align-items: center;
  padding: 0.5rem 0.75rem;
  border-bottom: 1px solid var(--hh-border, #e5e7eb);
  background: color-mix(in srgb, var(--hh-bg-primary, #ffffff) 82%, transparent);
  backdrop-filter: blur(var(--hh-panel-blur));
}

.sync-settings-heading {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  min-width: 0;
}

.sync-settings-icon {
  width: 16px;
  height: 16px;
  color: var(--hh-accent, #2563eb);
}

.sync-settings-title {
  color: var(--hh-text-primary, #1f2937);
  font-size: 0.75rem;
  font-weight: 700;
}

.sync-settings-meta {
  color: var(--hh-text-secondary, #6b7280);
  font-size: 0.6875rem;
}

.sync-settings-form {
  display: flex;
  align-items: flex-start;
  justify-content: flex-end;
  gap: 0.5rem;
  min-width: 0;
}

.sync-toggle,
.sync-field {
  display: grid;
  gap: 0.1875rem;
  color: var(--hh-text-secondary, #6b7280);
  font-size: 0.6875rem;
  font-weight: 600;
}

.sync-toggle {
  grid-template-columns: auto auto;
  align-items: center;
  padding-top: 1.2rem;
}

.sync-toggle input {
  accent-color: var(--hh-accent, #2563eb);
}

.sync-field input {
  width: 6.5rem;
  min-height: 1.75rem;
  border: 1px solid var(--hh-border, #d1d5db);
  border-radius: var(--hh-radius-sm, 0.375rem);
  padding: 0.25rem 0.375rem;
  background: color-mix(in srgb, var(--hh-bg-primary, #ffffff) 74%, transparent);
  color: var(--hh-text-primary, #111827);
  font-size: 0.75rem;
}

.sync-field small {
  max-width: 8rem;
  color: var(--hh-text-error, #ef4444);
  font-size: 0.625rem;
  line-height: 1.2;
}

@media (max-width: 900px) {
  .mail-sync-settings-strip {
    grid-template-columns: 1fr;
  }

  .sync-settings-form {
    justify-content: flex-start;
    flex-wrap: wrap;
  }
}
</style>
```
