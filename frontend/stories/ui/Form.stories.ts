import type { Meta, StoryObj } from '@storybook/vue3-vite'
import {
	Autocomplete,
	Button,
	CharacterCounter,
	Checkbox,
	ColorPicker,
	Combobox,
	DatePicker,
	DateTimePicker,
	EmailInput,
	FileDropZone,
	FilePicker,
	Form,
	FormError,
	FormField,
	FormHint,
	FormLabel,
	MultiSelect,
	NumberInput,
	OTPInput,
	PasswordInput,
	Radio,
	RadioGroup,
	RangeSlider,
	SearchInput,
	Select,
	Slider,
	Switch,
	TimePicker,
	Textarea
} from '@/shared/ui'
import { storybookLocaleFromGlobals, storybookText } from './storybook-i18n'

const meta = {
	title: 'Макошь UI/General/Form',
	render: (_args, context) => ({
		components: {
			Autocomplete,
			Button,
			CharacterCounter,
			Checkbox,
			ColorPicker,
			Combobox,
			DatePicker,
			DateTimePicker,
			EmailInput,
			FileDropZone,
			FilePicker,
			Form,
			FormError,
			FormField,
			FormHint,
			FormLabel,
			MultiSelect,
			NumberInput,
			OTPInput,
			PasswordInput,
			Radio,
			RadioGroup,
			RangeSlider,
			SearchInput,
			Select,
			Slider,
			Switch,
			TimePicker,
			Textarea
		},
		data() {
			const text = storybookText(storybookLocaleFromGlobals(context.globals))
			return {
				text,
				query: text.form.searchValue,
				email: text.form.emailValue,
					password: text.form.passwordValue,
				count: text.form.countValue,
				otp: text.form.otpValue,
				note: text.form.noteValue,
				channel: 'communications',
				relatedDomains: ['communications', 'knowledge'],
				comboboxValue: 'radar',
				autocompleteValue: 'knowledge',
				accentColor: '#178f6e',
				dueDate: '2026-07-01',
				dueTime: '09:30',
				dueDateTime: '2026-07-01T09:30',
				fileSummary: text.form.noFiles,
				enabled: true,
				reviewRequired: true,
				confidence: 'medium',
				threshold: 62,
				scoreRange: { min: 24, max: 84 },
				options: text.form.options,
				describeFiles: (files: File[]) => files.length ? files.map((file) => file.name).join(', ') : text.form.noFiles
			}
		},
		template: `
			<section class="storybook-canvas">
				<div class="storybook-section">
					<h2>{{ text.form.title }}</h2>
					<Form>
						<div class="storybook-grid">
							<FormField>
								<FormLabel for="story-search" required>{{ text.form.search }}</FormLabel>
								<SearchInput id="story-search" v-model="query" :aria-label="text.form.search" :placeholder="text.form.searchPlaceholder" />
								<FormHint>{{ text.form.hint }}</FormHint>
							</FormField>
							<FormField>
								<FormLabel for="story-email">{{ text.form.email }}</FormLabel>
								<EmailInput id="story-email" v-model="email" :aria-label="text.form.email" />
							</FormField>
							<FormField>
									<FormLabel for="story-password">{{ text.form.password }}</FormLabel>
									<PasswordInput id="story-password" v-model="password" :aria-label="text.form.password" />
							</FormField>
							<FormField>
								<FormLabel for="story-count">{{ text.form.count }}</FormLabel>
								<NumberInput id="story-count" v-model="count" :aria-label="text.form.count" :min="1" :max="99" />
								<FormError>{{ text.form.error }}</FormError>
							</FormField>
							<FormField>
								<FormLabel>{{ text.form.otp }}</FormLabel>
								<OTPInput v-model="otp" :label="text.form.otp" />
								<FormHint>{{ text.form.otpHint }}</FormHint>
							</FormField>
							<FormField>
								<FormLabel>{{ text.form.domain }}</FormLabel>
								<Select v-model="channel" :options="options" :aria-label="text.form.domain" />
							</FormField>
							<FormField>
								<FormLabel>{{ text.form.multiSelect }}</FormLabel>
								<MultiSelect v-model="relatedDomains" :options="options" :aria-label="text.form.multiSelect" />
							</FormField>
							<FormField>
								<FormLabel>{{ text.form.contextNote }}</FormLabel>
								<Textarea v-model="note" :rows="4" :aria-label="text.form.contextNote" />
								<CharacterCounter :value="note" :max="text.form.counterMax" />
							</FormField>
						</div>

						<div class="storybook-grid">
							<FormField>
								<Checkbox v-model="reviewRequired">{{ text.form.checkbox }}</Checkbox>
								<div class="storybook-row">
									<Switch v-model="enabled" :aria-label="text.form.realtime" />
									<span>{{ text.form.realtime }}</span>
								</div>
							</FormField>
							<FormField>
								<RadioGroup v-model="confidence" name="signal-confidence" :label="text.form.radioTitle">
									<Radio v-for="option in text.form.radioOptions" :key="option.value" :value="option.value">
										{{ option.label }}
									</Radio>
								</RadioGroup>
							</FormField>
							<FormField>
								<Slider v-model="threshold" :label="text.form.slider" :min="0" :max="100" />
								<RangeSlider v-model="scoreRange" :label="text.form.range" :min="0" :max="100" />
							</FormField>
						</div>

						<div class="storybook-grid">
							<FormField>
								<FormLabel for="story-combobox">{{ text.form.combobox }}</FormLabel>
								<Combobox id="story-combobox" v-model="comboboxValue" :options="options" :aria-label="text.form.combobox" />
							</FormField>
							<FormField>
								<FormLabel for="story-autocomplete">{{ text.form.autocomplete }}</FormLabel>
								<Autocomplete id="story-autocomplete" v-model="autocompleteValue" :options="options" :aria-label="text.form.autocomplete" :no-results-label="text.form.noResults" />
							</FormField>
							<FormField>
								<FormLabel for="story-color">{{ text.form.color }}</FormLabel>
								<ColorPicker id="story-color" v-model="accentColor" :label="text.form.color" />
							</FormField>
							<FormField>
								<FormLabel for="story-date">{{ text.form.date }}</FormLabel>
								<DatePicker id="story-date" v-model="dueDate" :aria-label="text.form.date" />
							</FormField>
							<FormField>
								<FormLabel for="story-time">{{ text.form.time }}</FormLabel>
								<TimePicker id="story-time" v-model="dueTime" :aria-label="text.form.time" />
							</FormField>
							<FormField>
								<FormLabel for="story-datetime">{{ text.form.dateTime }}</FormLabel>
								<DateTimePicker id="story-datetime" v-model="dueDateTime" :aria-label="text.form.dateTime" />
							</FormField>
							<FormField>
								<FormLabel for="story-file">{{ text.form.file }}</FormLabel>
								<FilePicker id="story-file" :aria-label="text.form.file" multiple @change="fileSummary = describeFiles($event)" />
								<FormHint>{{ fileSummary }}</FormHint>
							</FormField>
							<FormField>
								<FormLabel>{{ text.form.dropZone }}</FormLabel>
								<FileDropZone :label="text.form.dropZone" :hint="text.form.dropZoneHint" multiple @change="fileSummary = describeFiles($event)" />
							</FormField>
						</div>

						<div class="storybook-row">
							<Button>{{ text.form.saveContract }}</Button>
						</div>
					</Form>
				</div>
			</section>
		`
	})
} satisfies Meta

export default meta
type Story = StoryObj<typeof meta>

export const Default: Story = {}
