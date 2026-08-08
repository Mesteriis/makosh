<script setup lang="ts">
import { DialogRoot, DialogPortal, DialogOverlay, DialogContent, DialogTitle, DialogDescription } from 'reka-ui'
import { ref, computed, watch, nextTick } from 'vue'
import type { CommandGroup, CommandItem } from './Command.types'
import Icon from './Icon.vue'

const props = withDefaults(defineProps<{
  open?: boolean
  groups?: CommandGroup[]
  placeholder?: string
  emptyMessage?: string
  class?: string
  contentClass?: string
}>(), {
  placeholder: 'Поиск...',
  emptyMessage: 'Ничего не найдено'
})

const emit = defineEmits<{
  'update:open': [value: boolean]
  'select': [item: CommandItem]
}>()

const query = ref('')
const inputRef = ref<HTMLInputElement | null>(null)
const selectedIndex = ref(0)


const filteredGroups = computed(() => {
  const q = query.value.toLowerCase().trim()
  if (!q) return props.groups || []

  return (props.groups || [])
    .map((group) => ({
      ...group,
      items: group.items.filter((item) => {
        const labelMatch = item.label.toLowerCase().includes(q)
        const descMatch = item.description?.toLowerCase().includes(q)
        const keywordMatch = item.keywords?.some((k) => k.toLowerCase().includes(q))
        return labelMatch || descMatch || keywordMatch
      })
    }))
    .filter((g) => g.items.length > 0)
})

const filteredFlatItems = computed(() => {
  return filteredGroups.value.flatMap((g) => g.items)
})

watch(() => props.open, (isOpen) => {
  if (isOpen) {
    query.value = ''
    selectedIndex.value = 0
    nextTick(() => inputRef.value?.focus())
  }
})

function handleKeyDown(event: KeyboardEvent): void {
  const items = filteredFlatItems.value
  if (items.length === 0) return

  switch (event.key) {
    case 'ArrowDown':
      event.preventDefault()
      selectedIndex.value = Math.min(selectedIndex.value + 1, items.length - 1)
      break
    case 'ArrowUp':
      event.preventDefault()
      selectedIndex.value = Math.max(selectedIndex.value - 1, 0)
      break
    case 'Enter':
      event.preventDefault()
      const selected = items[selectedIndex.value]
      if (selected) {
        selected.onSelect?.()
        emit('select', selected)
        emit('update:open', false)
      }
      break
  }
}

function selectItem(item: CommandItem): void {
  item.onSelect?.()
  emit('select', item)
  emit('update:open', false)
}

const contentClasses = computed(() => [
  'makosh-command-content',
  props.contentClass
])
</script>

<template>
  <DialogRoot :open="open" @update:open="(val) => emit('update:open', val)">
    <DialogPortal>
      <DialogOverlay class="makosh-command-overlay" @pointerdown="emit('update:open', false)">
        <DialogContent :class="contentClasses" @keydown="handleKeyDown" @open-auto-focus="(e: Event) => e.preventDefault()">
          <DialogTitle class="makosh-sr-only">Command palette</DialogTitle>
          <DialogDescription class="makosh-sr-only">Search and run Макошь commands.</DialogDescription>
          <div class="makosh-command-input-wrapper">
            <Icon icon="tabler:search" size="1.125rem" class="makosh-command-search-icon" />
            <input
              ref="inputRef"
              v-model="query"
              class="makosh-command-input"
              :placeholder="placeholder"
              @keydown.stop="handleKeyDown"
            />
            <kbd class="makosh-command-kbd">ESC</kbd>
          </div>

          <div class="makosh-command-list">
            <template v-if="filteredGroups.length > 0">
              <div v-for="(group, gi) in filteredGroups" :key="gi" class="makosh-command-group">
                <div class="makosh-command-group-label">{{ group.label }}</div>
                <button
                  v-for="(item, ii) in group.items"
                  :key="item.id"
                  class="makosh-command-item"
                  :class="{ 'makosh-command-item--selected': filteredFlatItems.indexOf(item) === selectedIndex }"
                  @click="selectItem(item)"
                  @mouseenter="selectedIndex = filteredFlatItems.indexOf(item)"
                >
                  <Icon v-if="item.icon" :icon="item.icon" size="1.125rem" class="makosh-command-item-icon" />
                  <div class="makosh-command-item-text">
                    <span class="makosh-command-item-label">{{ item.label }}</span>
                    <span v-if="item.description" class="makosh-command-item-desc">{{ item.description }}</span>
                  </div>
                </button>
              </div>
            </template>
            <div v-else-if="query" class="makosh-command-empty">
              <Icon icon="tabler:search-off" size="1.5rem" />
              <span>{{ emptyMessage }}</span>
            </div>
          </div>
        </DialogContent>
      </DialogOverlay>
    </DialogPortal>
  </DialogRoot>
</template>
