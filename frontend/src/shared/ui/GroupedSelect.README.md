# GroupedSelect

Flat grouped select for sectioned option sets.

Use `GroupedSelect` when groups are labels, not navigable hierarchy. Use
`TreeSelect` or `Cascader` when parent-child semantics matter.

Boundary rules:

- wraps `SearchableSelect`;
- accepts local `SelectGroup[]`;
- flattens groups into local `SelectOption[]`;
- requires option `value` to be globally unique across all groups because
  `modelValue` is a single string;
- does not fetch data or own domain state.

## Props

- `modelValue?: string` - selected option value.
- `groups?: SelectGroup[]` - sectioned options from `Selection.types`; option
  values must be globally unique across all groups.
- `placeholder?: string` - trigger text when nothing is selected.
- `searchPlaceholder?: string` - search input placeholder.
- `ariaLabel?: string` - accessible trigger label.
- `disabled?: boolean` - disables interaction.
- `emptyLabel?: string` - empty state shown after filtering.
- `class?: string` - extra class forwarded with the `makosh-grouped-select`
  marker.

## Events

- `update:modelValue` emits the selected option value.
- `search` emits the local search query from `SearchableSelect`.
- `select` emits the selected flattened option.
- `clear` emits when the current selection is cleared.
