# Cascader

Column-based hierarchical selector for local `TreeSelectOption` data.

Use `Cascader` when a deep hierarchy is easier to scan progressively by columns
than as one expanded tree.

## Usage

```vue
<script setup lang="ts">
import { ref } from 'vue'
import { Cascader } from '@/shared/ui'
import type { TreeSelectOption } from '@/shared/ui'

const value = ref('')

const options: TreeSelectOption[] = [
	{
		value: 'communications',
		label: 'Communications',
		children: [
			{ value: 'communications.inbox', label: 'Inbox' },
			{ value: 'communications.outbox', label: 'Outbox' }
		]
	}
]
</script>

<template>
	<Cascader v-model="value" :options="options" placeholder="Select area" />
</template>
```

## Contract

- `modelValue` stores one enabled leaf option `value`.
- Option values must be globally unique across the whole tree. Duplicate values
  are treated as invalid and rendered as disabled so the component never chooses
  the first matching branch silently.
- Parent options are navigation-only. Clicking a parent or pressing `Enter` /
  `Space` on it opens the next column and does not emit a durable selection.
- A parent `modelValue`, missing value, disabled value, or value under a
  disabled ancestor is treated as invalid display state and renders the
  placeholder.
- Disabled options are visible in their column but cannot be selected or opened.
  Descendants under disabled ancestors are not reachable through the cascader.
- `readonly` keeps the current value visible while preventing the popover from
  opening or emitting selection changes.
- Labels, descriptions, and placeholders are owned by the caller. Pass already
  translated text from the active Макошь i18n context.
- `emptyLabel` is also caller-owned text for empty option columns.

## Events

- `update:modelValue` emits only after an enabled leaf is selected.
- `select` emits the selected leaf `TreeSelectOption`.
- `open` emits when the popover opens.
- `close` emits when the popover closes.

## Keyboard

- `Escape` closes the popover.
- `ArrowUp` / `ArrowDown` move within the active column.
- `ArrowRight` opens the next column for the active branch.
- `ArrowLeft` returns to the previous column.
- `Enter` / `Space` opens the active branch or selects the active enabled leaf.
- `Tab` is not trapped; focus leaving the component closes the popover.
