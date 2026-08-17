<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import Button from './primitives/Button.vue'
import Dialog from './Dialog.vue'
import type { StepsItem, StepsSlotProps } from './Steps.types'

interface NormalizedStepItem {
	index: number
	title: string
	description?: string
	requirement?: string
}

const props = withDefaults(defineProps<{
	open?: boolean
	step?: number
	stepCount: number
	steps?: StepsItem[]
	title?: string
	description?: string
	closeLabel?: string
	cancelLabel?: string
	previousLabel?: string
	nextLabel?: string
	finishLabel?: string
	stepsLabel?: string
	canAdvance?: boolean
	showCancel?: boolean
	busy?: boolean
	size?: 'md' | 'lg'
	contentClass?: string
}>(), {
	open: false,
	step: 1,
	closeLabel: 'Close wizard',
	cancelLabel: 'Cancel',
	previousLabel: 'Back',
	nextLabel: 'Next',
	finishLabel: 'Finish',
	stepsLabel: 'Steps',
	canAdvance: true,
	showCancel: false,
	busy: false,
	size: 'md'
})

const emit = defineEmits<{
	'update:open': [value: boolean]
	'update:step': [value: number]
	next: [value: number]
	previous: [value: number]
	finish: []
	cancel: []
}>()

const safeStepCount = computed(() => normalizeStepCount(props.stepCount))
const activeStep = computed(() => clampStep(props.step, safeStepCount.value))
const isFirst = computed(() => activeStep.value <= 1)
const isLast = computed(() => activeStep.value >= safeStepCount.value)
const transitionName = ref('makosh-steps-slide-forward')

const normalizedSteps = computed<NormalizedStepItem[]>(() => {
	const items: NormalizedStepItem[] = []
	for (let index = 1; index <= safeStepCount.value; index += 1) {
		const source = props.steps?.[index - 1]
		const title = source?.title?.trim() || `Step ${index}`
		items.push({
			index,
			title,
			description: source?.description,
			requirement: source?.requirement
		})
	}
	return items
})

const activeItem = computed(() => normalizedSteps.value[activeStep.value - 1] ?? normalizedSteps.value[0])
const activeSlotName = computed(() => `step-${activeStep.value}`)
const dialogContentClass = computed(() => [
	'makosh-steps-dialog',
	`makosh-steps-dialog--${props.size}`,
	props.contentClass
].filter(Boolean).join(' '))

const slotProps = computed<StepsSlotProps>(() => ({
	step: activeStep.value,
	stepCount: safeStepCount.value,
	isFirst: isFirst.value,
	isLast: isLast.value,
	next: nextStep,
	previous: previousStep,
	goToStep,
	close: closeDialog
}))

watch(activeStep, (value, previousValue) => {
	if (value === previousValue) return
	transitionName.value = value > previousValue ? 'makosh-steps-slide-forward' : 'makosh-steps-slide-backward'
})

function normalizeStepCount(value: number): number {
	if (!Number.isFinite(value)) return 1
	return Math.max(1, Math.trunc(value))
}

function clampStep(value: number, count: number): number {
	if (!Number.isFinite(value)) return 1
	return Math.min(Math.max(1, Math.trunc(value)), count)
}

function closeDialog(): void {
	emit('update:open', false)
}

function cancel(): void {
	emit('cancel')
	closeDialog()
}

function goToStep(value: number): void {
	const nextValue = clampStep(value, safeStepCount.value)
	if (nextValue !== activeStep.value) {
		emit('update:step', nextValue)
	}
}

function previousStep(): void {
	if (isFirst.value || props.busy) return
	const nextValue = activeStep.value - 1
	emit('previous', nextValue)
	emit('update:step', nextValue)
}

function nextStep(): void {
	if (!props.canAdvance || props.busy) return
	if (isLast.value) {
		emit('finish')
		return
	}
	const nextValue = activeStep.value + 1
	emit('next', nextValue)
	emit('update:step', nextValue)
}
</script>

<template>
	<Dialog
		:open="open"
		:title="title"
		:description="description"
		:close-label="closeLabel"
		:content-class="dialogContentClass"
		:show-close="true"
		@update:open="(value) => emit('update:open', value)"
	>
		<template v-if="$slots.trigger" #trigger>
			<slot name="trigger" />
		</template>

		<section class="makosh-steps" aria-live="polite">
			<section class="makosh-steps__content" role="region" :aria-label="activeItem.title">
				<Transition :name="transitionName" mode="out-in">
					<div :key="activeStep" class="makosh-steps__slide">
						<div class="makosh-steps__content-header">
							<h3>{{ activeItem.title }}</h3>
							<p v-if="activeItem.description">{{ activeItem.description }}</p>
							<p v-if="activeItem.requirement">{{ activeItem.requirement }}</p>
						</div>
						<div class="makosh-steps__slot">
							<slot :name="activeSlotName" v-bind="slotProps">
								<slot v-bind="slotProps">
									<p class="makosh-steps__empty">
										Provide content with the #{{ activeSlotName }} slot.
									</p>
								</slot>
							</slot>
						</div>
					</div>
				</Transition>
			</section>
		</section>

		<template #footer>
			<slot name="footer" v-bind="slotProps">
				<div class="makosh-steps__dock">
					<Button v-if="showCancel" variant="ghost" :disabled="busy" @click="cancel">
						{{ cancelLabel }}
					</Button>
					<Button
						class="makosh-steps__dock-back"
						variant="ghost"
						icon="tabler:arrow-left"
						:aria-label="previousLabel"
						:disabled="isFirst || busy"
						@click="previousStep"
					/>
					<nav class="makosh-steps__dots" :aria-label="stepsLabel">
						<button
							v-for="item in normalizedSteps"
							:key="item.index"
							class="makosh-steps__dot"
							:class="{ 'makosh-steps__dot--active': item.index === activeStep }"
							type="button"
							:aria-current="item.index === activeStep ? 'step' : undefined"
							:aria-label="item.title"
							:disabled="busy"
							@click="goToStep(item.index)"
						/>
					</nav>
					<Button
						class="makosh-steps__dock-next"
						:loading="busy"
						:disabled="!canAdvance"
						@click="nextStep"
					>
						{{ isLast ? finishLabel : nextLabel }}
					</Button>
				</div>
			</slot>
		</template>
	</Dialog>
</template>
