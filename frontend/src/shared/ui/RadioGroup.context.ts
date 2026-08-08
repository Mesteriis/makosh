import type { ComputedRef, InjectionKey } from 'vue'

export interface МакошьRadioGroupContext {
	name: string
	modelValue: ComputedRef<string | number | null>
	disabled: ComputedRef<boolean>
	select(value: string | number): void
}

export const makoshRadioGroupKey: InjectionKey<МакошьRadioGroupContext> = Symbol('makosh-radio-group')
