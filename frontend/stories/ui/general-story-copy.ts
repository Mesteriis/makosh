import type { StorybookLocale } from './storybook-i18n'

const en = {
	actions: {
		run: 'Run action',
		save: 'Save',
		archive: 'Archive',
		more: 'More actions',
		copy: 'Copy reference',
		open: 'Open context',
		close: 'Close',
		confirm: 'Confirm'
	},
	controls: {
		buttonGroup: 'Button group',
		iconButton: 'Icon button',
		splitButton: 'Split button',
		toggleGroup: 'Toggle group',
		input: 'Input',
		textarea: 'Textarea',
		searchInput: 'Search input',
		tokenInput: 'Token input',
		tagInput: 'Tag input',
		checkbox: 'Checkbox',
		radio: 'Radio',
		switch: 'Switch',
		slider: 'Slider',
		datePicker: 'Date picker',
		dateRangePicker: 'Date range picker',
		timePicker: 'Time picker',
		menu: 'Menu',
		contextMenu: 'Context menu',
		tabs: 'Tabs',
		dialog: 'Dialog',
		drawer: 'Drawer',
		tooltip: 'Tooltip',
		popover: 'Popover',
		table: 'Table',
		list: 'List',
		tree: 'Tree',
		timeline: 'Timeline',
		media: 'Media',
		editor: 'Editor',
		feedback: 'Feedback',
		layout: 'Layout',
		utility: 'Utility',
		surface: 'Surface'
	},
	form: {
		contextTitle: 'Context title',
		contextPlaceholder: 'Name the local context',
		searchPlaceholder: 'Search signals',
		noteLabel: 'Review note',
		noteValue: 'Keep the candidate in review until evidence is linked.',
		tokensPlaceholder: 'Add token and press Enter',
		tagsPlaceholder: 'Add tag',
		remove: 'Remove',
		dateStart: 'Start',
		dateEnd: 'End',
		enabled: 'Enabled for review',
		sync: 'Local sync',
		confidence: 'Confidence',
		threshold: 'Evidence threshold'
	},
	toggles: [
		{ value: 'review', label: 'Review', icon: 'tabler:checks' },
		{ value: 'evidence', label: 'Evidence', icon: 'tabler:archive' },
		{ value: 'memory', label: 'Memory', icon: 'tabler:brain' }
	],
	tokens: ['source:mail', 'review', 'evidence'],
	tags: ['urgent', 'owner-review'],
	tagSuggestions: ['urgent', 'owner-review', 'evidence', 'context-pack', 'low-risk'],
	menuItems: [
		{ id: 'copy', label: 'Copy reference', icon: 'tabler:copy' },
		{ id: 'open', label: 'Open context', icon: 'tabler:external-link' },
		{ id: 'archive', label: 'Archive', icon: 'tabler:archive' }
	],
	tabs: [
		{ id: 'review', label: 'Review' },
		{ id: 'evidence', label: 'Evidence' },
		{ id: 'memory', label: 'Memory' }
	],
	tabContent: {
		review: 'Review queue stays separate from durable truth.',
		evidence: 'Evidence links explain why a candidate exists.',
		memory: 'Memory surfaces are promoted after review.'
	},
	overlay: {
		title: 'Evidence action',
		description: 'Confirm only after source context is attached.',
		body: 'This shared overlay owns presentation only; domain actions stay outside the UI kit.',
		popoverBody: 'Compact local details without leaving the current surface.',
		tooltip: 'Open source context'
	},
	data: {
		tableCaption: 'Review candidates',
		empty: 'No rows',
		listLabel: 'Review items',
		treeLabel: 'Workspace tree',
		columns: [
			{ key: 'source', label: 'Source' },
			{ key: 'signal', label: 'Signal' },
			{ key: 'confidence', label: 'Confidence', align: 'right' as const }
		],
		rows: [
			{ id: 'row-1', source: 'Mail', signal: 'Budget review', confidence: '82%' },
			{ id: 'row-2', source: 'Calendar', signal: 'Decision follow-up', confidence: '76%' }
		],
		listItems: [
			{ id: 'item-1', label: 'Evidence linked', description: 'Source context is ready.', meta: 'ready', icon: 'tabler:archive', tone: 'success' as const },
			{ id: 'item-2', label: 'Owner review', description: 'Promotion still needs confirmation.', meta: 'queued', icon: 'tabler:checks', tone: 'warning' as const }
		],
		treeItems: [
			{
				id: 'workspace',
				label: 'Workspace',
				icon: 'tabler:layout-dashboard',
				children: [
					{ id: 'review', label: 'Review', icon: 'tabler:checks' },
					{ id: 'evidence', label: 'Evidence', icon: 'tabler:archive' }
				]
			}
		],
		timelineItems: [
			{ title: 'Signal observed', description: 'Provider input arrived as evidence.', time: '09:20', icon: 'tabler:radar', tone: 'info' as const },
			{ title: 'Review queued', description: 'Owner confirmation is pending.', time: '09:26', icon: 'tabler:checks', tone: 'warning' as const }
		]
	},
	foundation: {
		tokens: 'Design tokens',
		typography: 'Typography',
		icons: 'Icons',
		spacing: 'Spacing',
		heading: 'Макошь remembers with evidence',
		paragraph: 'Shared UI keeps product surfaces readable, local-first and traceable.',
		spacingItems: ['xs', 'sm', 'md', 'lg']
	},
	surfaces: {
		overviewTitle: 'Surface system',
		overviewDescription: 'Presentation-only containers for readable local context surfaces.',
		surfaceTitle: 'Surface',
		paperTitle: 'Paper',
		panelTitle: 'Panel',
		cardsTitle: 'Cards',
		signalTitle: 'Signal cards',
		signalDescription: 'Use an edge signal when a fresh event should pull attention without reshaping the card.',
		sectionsTitle: 'Sections',
		accordionTitle: 'Accordion',
		calloutsTitle: 'Callouts and wells',
		fieldsetTitle: 'Fieldset and toolbar',
		overlayTitle: 'Overlay safety',
		labels: {
			default: 'Default',
			muted: 'Muted',
			raised: 'Raised',
			deep: 'Deep',
			compact: 'Compact',
			comfortable: 'Comfortable',
			selected: 'Selected',
			disabled: 'Disabled',
			toolbar: 'Surface tools',
			details: 'Details',
			preview: 'Preview',
			trigger: 'Open local details'
		},
		actions: {
			primary: 'Review surface',
			secondary: 'Open details',
			save: 'Save layout',
			reset: 'Reset'
		},
		accordionItems: [
			{ id: 'density', title: 'Density', description: 'Compact or comfortable rhythm.' },
			{ id: 'hierarchy', title: 'Hierarchy', description: 'Surface, paper and panel layers.' },
			{ id: 'overlays', title: 'Overlays', description: 'Floating content stays visible.' }
		],
		stats: [
			{ label: 'Readable', value: '98%', description: 'Surface contrast check', trend: '+4', tone: 'success' as const, icon: 'tabler:eye' },
			{ label: 'Review', value: 12, description: 'UI-only samples', trend: 'stable', tone: 'accent' as const, icon: 'tabler:checks' },
			{ label: 'Risk', value: 2, description: 'Needs visual inspection', trend: '-1', tone: 'warning' as const, icon: 'tabler:alert-triangle' }
		],
		actionCards: [
			{ title: 'Create surface', description: 'Start with a neutral container.', icon: 'tabler:layout' },
			{ title: 'Compare states', description: 'Check density and hierarchy.', icon: 'tabler:arrows-diff' },
			{ title: 'Inspect overlay', description: 'Verify popovers are not clipped.', icon: 'tabler:window' }
		],
		signalCards: [
			{ title: 'New evidence arrived', description: 'The edge pulses while owner attention is useful.', tone: 'info' as const, active: true },
			{ title: 'Resolved signal', description: 'The signal can stay visible without motion.', tone: 'success' as const, active: true, pulse: false }
		],
		callouts: [
			{ tone: 'info' as const, title: 'Information', body: 'Use callouts for contextual notes.' },
			{ tone: 'success' as const, title: 'Ready', body: 'The surface is presentation-only.' },
			{ tone: 'warning' as const, title: 'Review', body: 'Check nested overlay behavior visually.' }
		],
		fields: [
			{ label: 'Surface name', value: 'Review panel' },
			{ label: 'Density', value: 'Comfortable' },
			{ label: 'Tone', value: 'Muted' }
		]
	}
}

type GeneralStoryCopy = typeof en

const ru: GeneralStoryCopy = {
	actions: {
		run: 'Выполнить',
		save: 'Сохранить',
		archive: 'В архив',
		more: 'Еще действия',
		copy: 'Скопировать ссылку',
		open: 'Открыть контекст',
		close: 'Закрыть',
		confirm: 'Подтвердить'
	},
	controls: {
		buttonGroup: 'Группа кнопок',
		iconButton: 'Кнопка-иконка',
		splitButton: 'Split button',
		toggleGroup: 'Toggle group',
		input: 'Поле ввода',
		textarea: 'Многострочное поле',
		searchInput: 'Поиск',
		tokenInput: 'Token input',
		tagInput: 'Tag input',
		checkbox: 'Checkbox',
		radio: 'Radio',
		switch: 'Switch',
		slider: 'Slider',
		datePicker: 'Дата',
		dateRangePicker: 'Диапазон дат',
		timePicker: 'Время',
		menu: 'Меню',
		contextMenu: 'Контекстное меню',
		tabs: 'Вкладки',
		dialog: 'Диалог',
		drawer: 'Drawer',
		tooltip: 'Tooltip',
		popover: 'Popover',
		table: 'Таблица',
		list: 'Список',
		tree: 'Дерево',
		timeline: 'Timeline',
		media: 'Медиа',
		editor: 'Редактор',
		feedback: 'Feedback',
		layout: 'Layout',
		utility: 'Utility',
		surface: 'Поверхность'
	},
	form: {
		contextTitle: 'Название контекста',
		contextPlaceholder: 'Назовите локальный контекст',
		searchPlaceholder: 'Искать сигналы',
		noteLabel: 'Заметка ревью',
		noteValue: 'Держать кандидата в review, пока evidence не связан.',
		tokensPlaceholder: 'Добавьте token и нажмите Enter',
		tagsPlaceholder: 'Добавить tag',
		remove: 'Удалить',
		dateStart: 'Начало',
		dateEnd: 'Конец',
		enabled: 'Включено для ревью',
		sync: 'Локальная синхронизация',
		confidence: 'Уверенность',
		threshold: 'Порог evidence'
	},
	toggles: [
		{ value: 'review', label: 'Ревью', icon: 'tabler:checks' },
		{ value: 'evidence', label: 'Evidence', icon: 'tabler:archive' },
		{ value: 'memory', label: 'Память', icon: 'tabler:brain' }
	],
	tokens: ['source:mail', 'review', 'evidence'],
	tags: ['срочно', 'owner-review'],
	tagSuggestions: ['срочно', 'owner-review', 'evidence', 'context-pack', 'низкий риск'],
	menuItems: [
		{ id: 'copy', label: 'Скопировать ссылку', icon: 'tabler:copy' },
		{ id: 'open', label: 'Открыть контекст', icon: 'tabler:external-link' },
		{ id: 'archive', label: 'В архив', icon: 'tabler:archive' }
	],
	tabs: [
		{ id: 'review', label: 'Ревью' },
		{ id: 'evidence', label: 'Evidence' },
		{ id: 'memory', label: 'Память' }
	],
	tabContent: {
		review: 'Очередь review отделена от durable truth.',
		evidence: 'Evidence объясняет, почему кандидат существует.',
		memory: 'Память продвигается после ревью.'
	},
	overlay: {
		title: 'Действие с evidence',
		description: 'Подтверждать только после привязки source context.',
		body: 'Этот общий overlay владеет только презентацией; доменные действия остаются вне UI kit.',
		popoverBody: 'Компактные локальные детали без ухода с текущей поверхности.',
		tooltip: 'Открыть source context'
	},
	data: {
		tableCaption: 'Кандидаты ревью',
		empty: 'Нет строк',
		listLabel: 'Элементы ревью',
		treeLabel: 'Дерево workspace',
		columns: [
			{ key: 'source', label: 'Источник' },
			{ key: 'signal', label: 'Сигнал' },
			{ key: 'confidence', label: 'Уверенность', align: 'right' }
		],
		rows: [
			{ id: 'row-1', source: 'Mail', signal: 'Budget review', confidence: '82%' },
			{ id: 'row-2', source: 'Calendar', signal: 'Decision follow-up', confidence: '76%' }
		],
		listItems: [
			{ id: 'item-1', label: 'Evidence связан', description: 'Source context готов.', meta: 'ready', icon: 'tabler:archive', tone: 'success' },
			{ id: 'item-2', label: 'Owner review', description: 'Promotion еще требует подтверждения.', meta: 'queued', icon: 'tabler:checks', tone: 'warning' }
		],
		treeItems: [
			{
				id: 'workspace',
				label: 'Workspace',
				icon: 'tabler:layout-dashboard',
				children: [
					{ id: 'review', label: 'Ревью', icon: 'tabler:checks' },
					{ id: 'evidence', label: 'Evidence', icon: 'tabler:archive' }
				]
			}
		],
		timelineItems: [
			{ title: 'Сигнал замечен', description: 'Provider input пришел как evidence.', time: '09:20', icon: 'tabler:radar', tone: 'info' },
			{ title: 'Review поставлен', description: 'Ожидается подтверждение владельца.', time: '09:26', icon: 'tabler:checks', tone: 'warning' }
		]
	},
	foundation: {
		tokens: 'Design tokens',
		typography: 'Типографика',
		icons: 'Иконки',
		spacing: 'Отступы',
		heading: 'Макошь помнит через evidence',
		paragraph: 'Shared UI сохраняет продуктовые поверхности читаемыми, local-first и traceable.',
		spacingItems: ['xs', 'sm', 'md', 'lg']
	},
	surfaces: {
		overviewTitle: 'Система поверхностей',
		overviewDescription: 'Presentation-only контейнеры для читаемых локальных контекстов.',
		surfaceTitle: 'Surface',
		paperTitle: 'Paper',
		panelTitle: 'Panel',
		cardsTitle: 'Карточки',
		signalTitle: 'Сигнальные карточки',
		signalDescription: 'Подсветка края привлекает внимание к свежему событию без перестройки карточки.',
		sectionsTitle: 'Секции',
		accordionTitle: 'Аккордеон',
		calloutsTitle: 'Callout и well',
		fieldsetTitle: 'Fieldset и toolbar',
		overlayTitle: 'Безопасность overlay',
		labels: {
			default: 'Default',
			muted: 'Muted',
			raised: 'Raised',
			deep: 'Deep',
			compact: 'Compact',
			comfortable: 'Comfortable',
			selected: 'Selected',
			disabled: 'Disabled',
			toolbar: 'Инструменты поверхности',
			details: 'Детали',
			preview: 'Предпросмотр',
			trigger: 'Открыть локальные детали'
		},
		actions: {
			primary: 'Проверить поверхность',
			secondary: 'Открыть детали',
			save: 'Сохранить layout',
			reset: 'Сбросить'
		},
		accordionItems: [
			{ id: 'density', title: 'Плотность', description: 'Compact или comfortable ритм.' },
			{ id: 'hierarchy', title: 'Иерархия', description: 'Слои surface, paper и panel.' },
			{ id: 'overlays', title: 'Overlay', description: 'Плавающий контент остается видимым.' }
		],
		stats: [
			{ label: 'Читаемость', value: '98%', description: 'Проверка контраста поверхности', trend: '+4', tone: 'success', icon: 'tabler:eye' },
			{ label: 'Review', value: 12, description: 'UI-only примеры', trend: 'stable', tone: 'accent', icon: 'tabler:checks' },
			{ label: 'Риск', value: 2, description: 'Нужен визуальный осмотр', trend: '-1', tone: 'warning', icon: 'tabler:alert-triangle' }
		],
		actionCards: [
			{ title: 'Создать поверхность', description: 'Начните с нейтрального контейнера.', icon: 'tabler:layout' },
			{ title: 'Сравнить состояния', description: 'Проверьте плотность и иерархию.', icon: 'tabler:arrows-diff' },
			{ title: 'Проверить overlay', description: 'Убедитесь, что popover не обрезается.', icon: 'tabler:window' }
		],
		signalCards: [
			{ title: 'Пришёл новый сигнал', description: 'Край мягко пульсирует, пока нужно внимание владельца.', tone: 'info', active: true },
			{ title: 'Сигнал обработан', description: 'Подсветку можно оставить без движения.', tone: 'success', active: true, pulse: false }
		],
		callouts: [
			{ tone: 'info', title: 'Информация', body: 'Используйте callout для контекстных заметок.' },
			{ tone: 'success', title: 'Готово', body: 'Поверхность отвечает только за презентацию.' },
			{ tone: 'warning', title: 'Review', body: 'Проверьте поведение вложенного overlay визуально.' }
		],
		fields: [
			{ label: 'Название поверхности', value: 'Review panel' },
			{ label: 'Плотность', value: 'Comfortable' },
			{ label: 'Тон', value: 'Muted' }
		]
	}
}

const es: GeneralStoryCopy = {
	actions: {
		run: 'Ejecutar',
		save: 'Guardar',
		archive: 'Archivar',
		more: 'Más acciones',
		copy: 'Copiar referencia',
		open: 'Abrir contexto',
		close: 'Cerrar',
		confirm: 'Confirmar'
	},
	controls: {
		buttonGroup: 'Grupo de botones',
		iconButton: 'Botón de icono',
		splitButton: 'Botón dividido',
		toggleGroup: 'Grupo toggle',
		input: 'Campo de texto',
		textarea: 'Área de texto',
		searchInput: 'Búsqueda',
		tokenInput: 'Token input',
		tagInput: 'Tag input',
		checkbox: 'Checkbox',
		radio: 'Radio',
		switch: 'Switch',
		slider: 'Slider',
		datePicker: 'Fecha',
		dateRangePicker: 'Rango de fechas',
		timePicker: 'Hora',
		menu: 'Menú',
		contextMenu: 'Menú contextual',
		tabs: 'Pestañas',
		dialog: 'Diálogo',
		drawer: 'Drawer',
		tooltip: 'Tooltip',
		popover: 'Popover',
		table: 'Tabla',
		list: 'Lista',
		tree: 'Árbol',
		timeline: 'Timeline',
		media: 'Media',
		editor: 'Editor',
		feedback: 'Feedback',
		layout: 'Layout',
		utility: 'Utility',
		surface: 'Superficie'
	},
	form: {
		contextTitle: 'Título del contexto',
		contextPlaceholder: 'Nombra el contexto local',
		searchPlaceholder: 'Buscar señales',
		noteLabel: 'Nota de revisión',
		noteValue: 'Mantener el candidato en revisión hasta vincular evidencia.',
		tokensPlaceholder: 'Añade token y pulsa Enter',
		tagsPlaceholder: 'Añadir tag',
		remove: 'Quitar',
		dateStart: 'Inicio',
		dateEnd: 'Fin',
		enabled: 'Activado para revisión',
		sync: 'Sincronización local',
		confidence: 'Confianza',
		threshold: 'Umbral de evidencia'
	},
	toggles: [
		{ value: 'review', label: 'Revisión', icon: 'tabler:checks' },
		{ value: 'evidence', label: 'Evidencia', icon: 'tabler:archive' },
		{ value: 'memory', label: 'Memoria', icon: 'tabler:brain' }
	],
	tokens: ['source:mail', 'review', 'evidence'],
	tags: ['urgente', 'owner-review'],
	tagSuggestions: ['urgente', 'owner-review', 'evidencia', 'context-pack', 'bajo riesgo'],
	menuItems: [
		{ id: 'copy', label: 'Copiar referencia', icon: 'tabler:copy' },
		{ id: 'open', label: 'Abrir contexto', icon: 'tabler:external-link' },
		{ id: 'archive', label: 'Archivar', icon: 'tabler:archive' }
	],
	tabs: [
		{ id: 'review', label: 'Revisión' },
		{ id: 'evidence', label: 'Evidencia' },
		{ id: 'memory', label: 'Memoria' }
	],
	tabContent: {
		review: 'La cola de revisión queda separada de la verdad durable.',
		evidence: 'La evidencia explica por qué existe un candidato.',
		memory: 'La memoria se promueve después de revisión.'
	},
	overlay: {
		title: 'Acción de evidencia',
		description: 'Confirmar solo después de vincular contexto fuente.',
		body: 'Este overlay compartido solo posee presentación; las acciones de dominio quedan fuera del UI kit.',
		popoverBody: 'Detalles locales compactos sin salir de la superficie actual.',
		tooltip: 'Abrir contexto fuente'
	},
	data: {
		tableCaption: 'Candidatos de revisión',
		empty: 'Sin filas',
		listLabel: 'Elementos de revisión',
		treeLabel: 'Árbol del workspace',
		columns: [
			{ key: 'source', label: 'Fuente' },
			{ key: 'signal', label: 'Señal' },
			{ key: 'confidence', label: 'Confianza', align: 'right' }
		],
		rows: [
			{ id: 'row-1', source: 'Mail', signal: 'Budget review', confidence: '82%' },
			{ id: 'row-2', source: 'Calendar', signal: 'Decision follow-up', confidence: '76%' }
		],
		listItems: [
			{ id: 'item-1', label: 'Evidencia vinculada', description: 'El contexto fuente está listo.', meta: 'ready', icon: 'tabler:archive', tone: 'success' },
			{ id: 'item-2', label: 'Revisión del dueño', description: 'La promoción aún requiere confirmación.', meta: 'queued', icon: 'tabler:checks', tone: 'warning' }
		],
		treeItems: [
			{
				id: 'workspace',
				label: 'Workspace',
				icon: 'tabler:layout-dashboard',
				children: [
					{ id: 'review', label: 'Revisión', icon: 'tabler:checks' },
					{ id: 'evidence', label: 'Evidencia', icon: 'tabler:archive' }
				]
			}
		],
		timelineItems: [
			{ title: 'Señal observada', description: 'La entrada del proveedor llegó como evidencia.', time: '09:20', icon: 'tabler:radar', tone: 'info' },
			{ title: 'Revisión en cola', description: 'Falta confirmación del dueño.', time: '09:26', icon: 'tabler:checks', tone: 'warning' }
		]
	},
	foundation: {
		tokens: 'Design tokens',
		typography: 'Tipografía',
		icons: 'Iconos',
		spacing: 'Espaciado',
		heading: 'Макошь recuerda con evidencia',
		paragraph: 'Shared UI mantiene las superficies legibles, local-first y trazables.',
		spacingItems: ['xs', 'sm', 'md', 'lg']
	},
	surfaces: {
		overviewTitle: 'Sistema de superficies',
		overviewDescription: 'Contenedores solo de presentación para contextos locales legibles.',
		surfaceTitle: 'Surface',
		paperTitle: 'Paper',
		panelTitle: 'Panel',
		cardsTitle: 'Tarjetas',
		signalTitle: 'Tarjetas de señal',
		signalDescription: 'El borde señala un evento reciente sin cambiar la estructura de la tarjeta.',
		sectionsTitle: 'Secciones',
		accordionTitle: 'Acordeón',
		calloutsTitle: 'Callouts y wells',
		fieldsetTitle: 'Fieldset y toolbar',
		overlayTitle: 'Seguridad de overlay',
		labels: {
			default: 'Default',
			muted: 'Muted',
			raised: 'Raised',
			deep: 'Deep',
			compact: 'Compact',
			comfortable: 'Comfortable',
			selected: 'Selected',
			disabled: 'Disabled',
			toolbar: 'Herramientas de superficie',
			details: 'Detalles',
			preview: 'Vista previa',
			trigger: 'Abrir detalles locales'
		},
		actions: {
			primary: 'Revisar superficie',
			secondary: 'Abrir detalles',
			save: 'Guardar layout',
			reset: 'Restablecer'
		},
		accordionItems: [
			{ id: 'density', title: 'Densidad', description: 'Ritmo compacto o cómodo.' },
			{ id: 'hierarchy', title: 'Jerarquía', description: 'Capas surface, paper y panel.' },
			{ id: 'overlays', title: 'Overlays', description: 'El contenido flotante queda visible.' }
		],
		stats: [
			{ label: 'Legible', value: '98%', description: 'Revisión de contraste', trend: '+4', tone: 'success', icon: 'tabler:eye' },
			{ label: 'Revisión', value: 12, description: 'Ejemplos solo UI', trend: 'stable', tone: 'accent', icon: 'tabler:checks' },
			{ label: 'Riesgo', value: 2, description: 'Necesita inspección visual', trend: '-1', tone: 'warning', icon: 'tabler:alert-triangle' }
		],
		actionCards: [
			{ title: 'Crear superficie', description: 'Empieza con un contenedor neutral.', icon: 'tabler:layout' },
			{ title: 'Comparar estados', description: 'Revisa densidad y jerarquía.', icon: 'tabler:arrows-diff' },
			{ title: 'Inspeccionar overlay', description: 'Verifica que el popover no se recorte.', icon: 'tabler:window' }
		],
		signalCards: [
			{ title: 'Nueva señal recibida', description: 'El borde pulsa mientras conviene llamar la atención.', tone: 'info', active: true },
			{ title: 'Señal resuelta', description: 'La señal puede quedar visible sin movimiento.', tone: 'success', active: true, pulse: false }
		],
		callouts: [
			{ tone: 'info', title: 'Información', body: 'Usa callouts para notas contextuales.' },
			{ tone: 'success', title: 'Listo', body: 'La superficie solo maneja presentación.' },
			{ tone: 'warning', title: 'Revisión', body: 'Comprueba visualmente el overlay anidado.' }
		],
		fields: [
			{ label: 'Nombre de superficie', value: 'Review panel' },
			{ label: 'Densidad', value: 'Comfortable' },
			{ label: 'Tono', value: 'Muted' }
		]
	}
}

const copies: Record<StorybookLocale, GeneralStoryCopy> = { ru, en, es }

export function generalStoryCopy(locale: StorybookLocale): GeneralStoryCopy {
	return copies[locale]
}
