export const storybookLocales = ['ru', 'en', 'es'] as const

export type StorybookLocale = (typeof storybookLocales)[number]

export const storybookLocaleToolbarItems = [
	{ value: 'ru', title: 'Русский' },
	{ value: 'en', title: 'English' },
	{ value: 'es', title: 'Español' }
] satisfies Array<{ value: StorybookLocale; title: string }>

const en = {
	common: {
		cancel: 'Cancel',
		create: 'Create',
		done: 'Done',
		close: 'Close',
		evidence: 'Evidence',
		openContext: 'Open context',
		primary: 'Primary',
		review: 'Review',
		risk: 'Risk',
		save: 'Save',
		trusted: 'Trusted'
	},
	button: {
		runAction: 'Run action',
		title: 'Buttons',
		description: 'Strict control surface for domain components. No direct provider behavior lives here.',
		primary: 'Primary',
		secondary: 'Secondary',
		outline: 'Outline',
		ghost: 'Ghost',
		delete: 'Delete',
		small: 'Small',
		medium: 'Medium',
		large: 'Large',
		loading: 'Loading',
		searchLabel: 'Search'
	},
	primitives: {
		title: 'Primitive components',
		description: 'Base UI-only building blocks for typography, labels, actions and surfaces.',
		typographyTitle: 'Typography',
		typographyDescription: 'Readable text primitives with stable size, tone and weight contracts.',
		heading: 'Review-ready context surface',
		paragraph: 'Макошь primitives keep domain language out of shared UI while preserving predictable layout.',
		muted: 'Muted supporting text',
		strong: 'Strong text',
		accent: 'Accent text',
		chipsTitle: 'Chips and tags',
		chipsDescription: 'Compact metadata primitives for neutral, status and accent labels.',
		chips: ['Candidate', 'Reviewed', 'Needs evidence', 'Risk'],
		actionsTitle: 'Text actions',
		actionsDescription: 'Low-emphasis actions for dense surfaces where full buttons are too heavy.',
		openDocs: 'Open docs',
		quietAction: 'Quiet action',
		dangerAction: 'Danger action',
		surfacesTitle: 'Layout surfaces',
		surfacesDescription: 'Panel, Paper and Container primitives for non-domain composition.'
	},
	command: {
		title: 'Command surface',
		description: 'Command palette is an accelerator. Predictable menu navigation can remain separate.',
		open: 'Open Command',
		selected: 'Selected',
		groups: [
			{
				label: 'Navigation',
				items: [
					{ id: 'communications', label: 'Communications', description: 'Unified mail, Telegram, WhatsApp and provider channels', icon: 'tabler:messages' },
					{ id: 'radar', label: 'Radar', description: 'Signals, observations and review queue', icon: 'tabler:radar' },
					{ id: 'knowledge', label: 'Knowledge', description: 'Notes, graph and context memory', icon: 'tabler:brain' }
				]
			},
			{
				label: 'Actions',
				items: [
					{ id: 'new-note', label: 'Create note', description: 'Capture a thought without creating fake business entities', icon: 'tabler:note' },
					{ id: 'new-task', label: 'Create task', description: 'Only after review or explicit intent', icon: 'tabler:checkbox' }
				]
			}
		]
	},
	navigation: {
		title: 'Navigation primitives',
		description: 'UI-only navigation components for dense desktop surfaces.',
		breadcrumbLabel: 'Context path',
		productNavigation: 'Product navigation',
		localMenus: 'Local menus',
		contextMenu: 'Context menu',
		contextTrigger: 'Context target',
		menubar: 'Application menubar',
		tree: 'Workspace tree',
		palettes: 'Command and search palettes',
		commandPalette: 'Open commands',
		searchPalette: 'Open search',
		commandPlaceholder: 'Run a command',
		searchPlaceholder: 'Search local actions',
		pagination: 'Review pages',
		reviewContent: 'Review queue navigation content.',
		evidenceContent: 'Evidence navigation content.',
		memoryContent: 'Memory navigation content.',
		tabs: [
			{ id: 'review', label: 'Review' },
			{ id: 'evidence', label: 'Evidence' },
			{ id: 'memory', label: 'Memory' }
		],
		breadcrumbs: [
			{ id: 'home', label: 'Макошь' },
			{ id: 'workspace', label: 'Workspace' },
			{ id: 'review', label: 'Review queue', current: true }
		],
		navItems: [
			{ id: 'communications', label: 'Communications', icon: 'tabler:messages' },
			{ id: 'radar', label: 'Radar', icon: 'tabler:radar', current: true },
			{ id: 'knowledge', label: 'Knowledge', icon: 'tabler:brain' }
		],
		menuItems: [
			{ id: 'inbox', label: 'Inbox', icon: 'tabler:inbox' },
			{ id: 'review', label: 'Review', icon: 'tabler:checks', current: true },
			{ id: 'archive', label: 'Archive', icon: 'tabler:archive' }
		],
		contextItems: [
			{ id: 'copy', label: 'Copy reference', icon: 'tabler:copy' },
			{ id: 'open', label: 'Open context', icon: 'tabler:external-link' },
			{ id: 'disabled', label: 'Unavailable action', icon: 'tabler:ban', disabled: true }
		],
		menubarItems: [
			{
				id: 'file',
				label: 'File',
				children: [
					{ id: 'new-context', label: 'New context' },
					{ id: 'export', label: 'Export snapshot' }
				]
			},
			{
				id: 'view',
				label: 'View',
				children: [
					{ id: 'compact', label: 'Compact density' },
					{ id: 'comfortable', label: 'Comfortable density' }
				]
			}
		],
		treeItems: [
			{
				id: 'workspace',
				label: 'Workspace',
				icon: 'tabler:layout-dashboard',
				children: [
					{ id: 'review-queue', label: 'Review queue', icon: 'tabler:checks' },
					{ id: 'evidence', label: 'Evidence', icon: 'tabler:archive' }
				]
			},
			{
				id: 'signals',
				label: 'Signals',
				icon: 'tabler:radar',
				children: [
					{ id: 'high-risk', label: 'High risk', icon: 'tabler:alert-triangle' },
					{ id: 'needs-context', label: 'Needs context', icon: 'tabler:help-circle' }
				]
			}
		]
	},
	data: {
		title: 'Data display primitives',
		description: 'Tables, lists, metadata, timelines, metrics and local state surfaces.',
		tableTitle: 'Tables',
		tableCaption: 'Review candidates',
		virtualTableTitle: 'Windowed table',
		listTitle: 'Lists',
		virtualListTitle: 'Windowed list',
		detailsTitle: 'Details',
		propertyGridTitle: 'Property grid',
		timelineTitle: 'Timeline and activity',
		metricsTitle: 'Metrics',
		statesTitle: 'States',
		tableColumns: [
			{ key: 'source', label: 'Source' },
			{ key: 'signal', label: 'Signal' },
			{ key: 'confidence', label: 'Confidence', align: 'right' as const },
			{ key: 'state', label: 'State' }
		],
		tableRows: [
			{ id: 'row-1', source: 'Mail', signal: 'Budget review', confidence: '82%', state: 'Review' },
			{ id: 'row-2', source: 'Calendar', signal: 'Decision follow-up', confidence: '76%', state: 'Pending' },
			{ id: 'row-3', source: 'Local note', signal: 'Context pack', confidence: '91%', state: 'Ready' },
			{ id: 'row-4', source: 'Document', signal: 'Evidence link', confidence: '68%', state: 'Needs source' }
		],
		listItems: [
			{ id: 'item-1', label: 'Canonical evidence', description: 'Source-linked context is available.', meta: 'ready', icon: 'tabler:archive', tone: 'success' as const },
			{ id: 'item-2', label: 'Review candidate', description: 'Needs owner confirmation before promotion.', meta: 'queued', icon: 'tabler:checks', tone: 'warning' as const },
			{ id: 'item-3', label: 'Context pack', description: 'Derived and rebuildable summary surface.', meta: 'local', icon: 'tabler:package', tone: 'accent' as const }
		],
		properties: [
			{ id: 'owner', label: 'Owner', value: 'Local user', description: 'Personal workspace boundary.' },
			{ id: 'trust', label: 'Trust', value: 'High', description: 'Evidence-backed surface.', tone: 'success' as const },
			{ id: 'source', label: 'Source', value: 'Observed', description: 'Not provider truth by itself.', tone: 'warning' as const },
			{ id: 'scope', label: 'Scope', value: 'UI only', description: 'No domain mutation.' }
		],
		timelineItems: [
			{ id: 'observed', title: 'Signal observed', description: 'Provider data stays outside durable truth.', time: '09:20', icon: 'tabler:radar', tone: 'info' as const },
			{ id: 'linked', title: 'Evidence linked', description: 'Canonical evidence is attached to the candidate.', time: '09:26', icon: 'tabler:link', tone: 'success' as const },
			{ id: 'review', title: 'Owner review pending', description: 'Promotion is explicit and traceable.', time: '09:31', icon: 'tabler:checks', tone: 'warning' as const }
		],
		activityItems: [
			{ id: 'activity-1', title: 'Context refreshed', description: 'Local projection updated.', meta: 'now', icon: 'tabler:refresh', tone: 'accent' as const },
			{ id: 'activity-2', title: 'Snapshot checked', description: 'Visual baseline is stable.', meta: '1m', icon: 'tabler:photo-check', tone: 'success' as const }
		],
		statistic: { label: 'Candidates', value: 24, trend: '+6', description: 'Ready for review', tone: 'accent' as const },
		metric: { label: 'Evidence score', value: 88, unit: '%', delta: '+4', tone: 'success' as const },
		counter: { label: 'open', value: 7, max: 12, tone: 'warning' as const },
		states: {
			emptyTitle: 'No candidates',
			emptyDescription: 'New signals will appear here after observation.',
			loadingTitle: 'Refreshing context',
			loadingDescription: 'Local projections are being rebuilt.',
			errorTitle: 'Could not render surface',
			errorDescription: 'The owner surface can provide a retry action.',
			noDataTitle: 'No local data',
			noSearchTitle: 'No search results',
			noSearchDescription: 'Try a broader local query.',
			noSearchQuery: 'provider root cache',
			offlineTitle: 'Offline mode',
			comingSoonTitle: 'Inspector actions'
		}
	},
	layout: {
		title: 'Layout primitives',
		description: 'UI-only composition primitives for dense desktop shells, panels, bars and scroll regions.',
		compositionTitle: 'Composition',
		compositionDescription: 'Stack, Grid, Flex, Split and Resizable keep spacing and alignment tokenized.',
		shellTitle: 'Shell surfaces',
		shellDescription: 'Dock, bars, panels and inspectors compose application-like surfaces without routing or stores.',
		scrollTitle: 'Scroll and floating surfaces',
		scrollDescription: 'ScrollArea, VirtualScrollArea and FloatingPanel keep overflow and context surfaces local.',
		stackTitle: 'Stacked review surface',
		stackDescription: 'Long text stays readable while the layout remains predictable across widths.',
		gridTitle: 'Responsive grid',
		splitPrimary: 'Primary workspace',
		splitSecondary: 'Secondary context',
		resizableTitle: 'Resizable preview',
		dockLabel: 'Workspace dock',
		toolbarLabel: 'Layout tools',
		actionLabel: 'Layout actions',
		sideTitle: 'Context rail',
		inspectorTitle: 'Evidence inspector',
		inspectorDescription: 'Generic properties, not provider runtime state.',
		topTitle: 'Макошь workspace',
		topDescription: 'UI Lab layout shell',
		bottomLabel: 'Workspace footer',
		statusLabel: 'Workspace status',
		floatingTitle: 'Floating context',
		floatingDescription: 'Non-modal helper surface with tokenized elevation.',
		virtualLabel: 'Virtual scroll sample',
		cards: [
			{ id: 'card-1', title: 'Review queue', description: 'Owner-facing candidates with source evidence.', meta: '24' },
			{ id: 'card-2', title: 'Context packs', description: 'Derived summaries that can be rebuilt.', meta: '8' },
			{ id: 'card-3', title: 'Local memory', description: 'Personal knowledge remains local-first.', meta: '152' }
		],
		navItems: ['Review', 'Evidence', 'Memory', 'Settings'],
		toolbarActions: ['Filter', 'Group', 'Sort'],
		actions: ['Cancel', 'Save layout'],
		statusItems: [
			{ id: 'mode', label: 'Mode', value: 'Local', tone: 'accent' as const },
			{ id: 'sync', label: 'Sync', value: 'Ready', tone: 'success' as const },
			{ id: 'risk', label: 'Risk', value: 'Low', tone: 'info' as const }
		],
		virtualItems: [
			'Observed provider signal',
			'Canonical evidence linked',
			'Candidate queued for review',
			'Context pack rebuilt',
			'Inspector panel opened',
			'Local projection refreshed',
			'Visual baseline checked',
			'Keyboard order reviewed',
			'Reduced motion honored',
			'Owner decision pending'
		]
	},
	media: {
		title: 'Media primitives',
		description: 'Generic previews for images, playback, documents, HTML, markdown, code and attachments.',
		imagesTitle: 'Images and gallery',
		playbackTitle: 'Playback shells',
		documentsTitle: 'Documents and source',
		attachmentsTitle: 'Attachment previews',
		galleryLabel: 'Evidence image gallery',
		imageCaption: 'Local evidence preview',
		emptyImage: 'No image available',
		videoTitle: 'Meeting clip',
		videoDescription: 'Native controls render without backend media transfer.',
		videoFallback: 'Video source not attached',
		audioTitle: 'Voice note',
		audioDescription: 'Audio playback stays generic and provider neutral.',
		audioFallback: 'Audio source not attached',
		markdownTitle: 'Markdown extract',
		markdownSource: '## Source summary\n- Evidence stays linked\n- Derived text can be rebuilt\n\n[Local reference](#)',
		codeTitle: 'Code block',
		codeSource: 'const evidence = {\n  source: "local",\n  reviewed: true\n}',
		syntaxTitle: 'Syntax highlight',
		htmlTitle: 'HTML body preview',
		htmlSource: '<article><h2>Prepared HTML</h2><p>Sanitized fragments can render as semantic HTML.</p></article>',
		textSource: 'Plain text body\nkeeps line breaks\nwithout provider-specific rendering.',
		unsafeHtml: 'HTML preview requires sanitized content',
		pdfTitle: 'PDF shell',
		pdfDescription: 'The shell is ready for safe local PDF URLs.',
		pdfFallback: 'PDF source not attached',
		attachmentAction: 'Inspect',
		galleryItems: [
			{ id: 'img-1', alt: 'Timeline artifact', title: 'Timeline artifact', description: 'Source image preview with neutral chrome.', meta: 'image/png' },
			{ id: 'img-2', alt: 'Context snapshot', title: 'Context snapshot', description: 'Visual evidence remains separated from provider runtime.', meta: 'image/svg+xml' },
			{ id: 'img-3', alt: 'Review capture', title: 'Review capture', description: 'Gallery state is local and UI-only.', meta: 'image/webp' }
		],
		attachments: [
			{ id: 'att-1', name: 'context-pack.pdf', mimeType: 'application/pdf', size: '284 KB', description: 'Prepared document preview.', icon: 'tabler:file-type-pdf', tone: 'danger' as const },
			{ id: 'att-2', name: 'message-body.html', mimeType: 'text/html', size: '18 KB', description: 'Sanitized HTML fragment.', icon: 'tabler:html', tone: 'warning' as const },
			{ id: 'att-3', name: 'notes.md', mimeType: 'text/markdown', size: '9 KB', description: 'Markdown source excerpt.', icon: 'tabler:markdown', tone: 'accent' as const }
		]
	},
	communication: {
		title: 'Communication primitives',
		description: 'UI-only message, composer and delivery primitives without provider behavior.',
		threadTitle: 'Review thread',
		composerTitle: 'Composer shell',
		composerLabel: 'Context reply',
		placeholder: 'Write a provider-neutral note',
		helper: 'Ctrl+Enter or Meta+Enter submits the local draft.',
		send: 'Send',
		attach: 'Attach evidence',
		typingLabel: 'Context assistant is composing',
		quoteAuthor: 'Source excerpt',
		quoteBody: 'Evidence remains visible before promotion.',
		toolbarLabel: 'Composer tools',
		deliveryDescription: 'Provider execution is represented by parent state.',
		readLabel: 'Read by reviewers',
		reactions: [
			{ emoji: '+', count: 4, label: 'Four positive reactions' },
			{ emoji: '!', count: 1, label: 'One needs attention reaction' }
		],
		actions: [
			{ id: 'bold', label: 'Bold', icon: 'tabler:bold' },
			{ id: 'quote', label: 'Quote', icon: 'tabler:quote' },
			{ id: 'risk', label: 'Risk', icon: 'tabler:alert-triangle', tone: 'warning' as const }
		],
		messages: [
			{ id: 'msg-1', author: 'Alex', timestamp: '09:12', meta: 'source-linked', direction: 'inbound' as const, body: 'Can we preserve the original evidence before turning this into a task?' },
			{ id: 'msg-2', author: 'Макошь', timestamp: '09:13', meta: 'review candidate', direction: 'outbound' as const, body: 'Yes. The candidate stays separate until owner review.' },
			{ id: 'msg-3', author: 'System', timestamp: '09:14', meta: 'local state', direction: 'system' as const, body: 'Delivery state is visual only in shared UI.' }
		],
		attachments: [
			{ name: 'evidence.html', meta: 'text/html', icon: 'tabler:html', tone: 'warning' as const },
			{ name: 'context.md', meta: '9 KB', icon: 'tabler:markdown', tone: 'accent' as const }
		],
		receipts: [
			{ id: 'rr-1', label: 'Owner', initials: 'OW' },
			{ id: 'rr-2', label: 'Reviewer', initials: 'RV' },
			{ id: 'rr-3', label: 'Archive', initials: 'AR' },
			{ id: 'rr-4', label: 'Memory', initials: 'ME' }
		]
	},
	utility: {
		title: 'Utility primitives',
		description: 'Small UI helpers for copy actions, switches, keyboard hints and semantic icons.',
		copyTitle: 'Copy action',
		copyValue: 'makosh://local/context-pack',
		copyLabel: 'Copy reference',
		copiedLabel: 'Copied',
		errorLabel: 'Copy unavailable',
		themeTitle: 'Theme selector',
		localeTitle: 'Locale selector',
		shortcutsTitle: 'Keyboard hints',
		iconsTitle: 'Semantic icons',
		openCommand: 'Open command',
		sendDraft: 'Submit draft',
		providerTitle: 'Providers',
		statusTitle: 'Statuses',
		entityTitle: 'Entities',
		fileTitle: 'Files',
		locales: [
			{ value: 'ru', label: 'RU', description: 'Russian' },
			{ value: 'en', label: 'EN', description: 'English' },
			{ value: 'es', label: 'ES', description: 'Spanish' }
		],
		providers: ['mail', 'telegram', 'whatsapp', 'calendar', 'documents', 'generic'] as const,
		statuses: ['idle', 'active', 'success', 'warning', 'danger', 'offline', 'syncing'] as const,
		entities: ['person', 'organization', 'project', 'task', 'document', 'decision', 'obligation', 'knowledge', 'event', 'generic'] as const,
		files: [
			{ label: 'Image', mimeType: 'image/png' },
			{ label: 'PDF', mimeType: 'application/pdf' },
			{ label: 'Code', mimeType: 'application/typescript' },
			{ label: 'Archive', mimeType: 'application/zip' }
		]
	},
	editor: {
		title: 'Rich context editor',
		description: 'Compact rich text surface for notes, mail drafts and evidence snippets. It stays semantic instead of becoming a Word-like ribbon.',
		label: 'Context editor',
		helper: 'Use compact semantic actions: headings, quotes, lists, marks, code, divider and evidence links.',
		placeholder: 'Capture the decision, cite evidence, keep the source visible.',
		toolbarLabel: 'Rich text tools',
		outputLabel: 'Sanitized output',
		previewTitle: 'Sanitized HTML preview',
		previewEmpty: 'No HTML output yet',
		keyboardLabel: 'Quick submit',
		actions: [
			{ id: 'paragraph', label: 'Paragraph', icon: 'tabler:pilcrow', group: 'structure' },
			{ id: 'heading', label: 'Heading', icon: 'tabler:h-2', group: 'structure' },
			{ id: 'subheading', label: 'Subheading', icon: 'tabler:h-3', group: 'structure' },
			{ id: 'quote', label: 'Quote', icon: 'tabler:quote', group: 'structure' },
			{ id: 'bulletList', label: 'Bulleted list', icon: 'tabler:list', group: 'lists' },
			{ id: 'orderedList', label: 'Numbered list', icon: 'tabler:list-numbers', group: 'lists' },
			{ id: 'bold', label: 'Emphasis', icon: 'tabler:bold', group: 'marks' },
			{ id: 'italic', label: 'Nuance', icon: 'tabler:italic', group: 'marks' },
			{ id: 'underline', label: 'Underline', icon: 'tabler:underline', group: 'marks' },
			{ id: 'strike', label: 'Strike', icon: 'tabler:strikethrough', group: 'marks' },
			{ id: 'code', label: 'Inline code', icon: 'tabler:code', group: 'marks' },
			{ id: 'link', label: 'Evidence link', icon: 'tabler:link', group: 'insert' },
			{ id: 'codeBlock', label: 'Code block', icon: 'tabler:code-dots', group: 'insert' },
			{ id: 'horizontalRule', label: 'Divider', icon: 'tabler:separator-horizontal', group: 'insert' },
			{ id: 'clearFormatting', label: 'Clear formatting', icon: 'tabler:eraser', group: 'cleanup' }
		],
		initialHtml: '<h2>Decision context</h2><p>Keep the <code>provider_message</code> as evidence before promotion.</p><ol><li><p>Preserve source.</p></li><li><p>Review before durable truth.</p></li></ol><blockquote><p>Source remains reviewable.</p></blockquote>'
	},
	form: {
		title: 'Form controls',
		search: 'Search',
		searchValue: 'telegram runtime',
		searchPlaceholder: 'Search anything',
		email: 'Owner email',
		emailValue: 'owner@example.local',
			password: 'Vault password',
			passwordValue: 'correct horse battery staple',
		count: 'Review limit',
		countValue: 7,
		otp: 'Verification code',
		otpValue: '482913',
		otpHint: 'Paste or type a short local code.',
		domain: 'Domain',
		multiSelect: 'Related domains',
		combobox: 'Combobox value',
		autocomplete: 'Autocomplete value',
		noResults: 'No local options',
		color: 'Accent color',
		date: 'Review date',
		time: 'Review time',
		dateTime: 'Review date and time',
		file: 'File picker',
		noFiles: 'No files selected',
		dropZone: 'Drop attachments',
		dropZoneHint: 'Keyboard users can press Enter or Space',
		contextNote: 'Context note',
		noteValue: 'Keep provider details outside user-facing communications UI.',
		hint: 'UI components expose state only; validation belongs to the owning form.',
		error: 'This sample error is rendered by FormError.',
		counterMax: 96,
		checkbox: 'Require owner review',
		radioTitle: 'Signal confidence',
		radioOptions: [
			{ value: 'low', label: 'Low' },
			{ value: 'medium', label: 'Medium' },
			{ value: 'high', label: 'High' }
		],
		slider: 'Confidence threshold',
		range: 'Evidence score range',
		realtime: 'Realtime context enabled',
		saveContract: 'Save component contract',
		options: [
			{ value: 'communications', label: 'Communications' },
			{ value: 'radar', label: 'Radar' },
			{ value: 'knowledge', label: 'Knowledge' }
		]
	},
	selection: {
		title: 'Selection controls',
		select: 'Select',
		searchableSelect: 'Searchable select',
		multiSelect: 'Multi select',
		searchableMultiSelect: 'Searchable multi select',
		groupedSelect: 'Grouped select',
		treeSelect: 'Tree select',
		cascader: 'Cascader',
		asyncSelect: 'Async select',
		placeholder: 'Choose a context',
		searchPlaceholder: 'Search local options',
		empty: 'No matching options',
		clear: 'Clear selection',
		searchLabel: 'Search options',
		optionsLabel: 'Available options',
		actionsLabel: 'Selection actions',
		selectAll: 'Select all',
		clearAll: 'Clear all',
		selectedCount: (count: number) => `${count} selected`,
		remove: (label: string) => `Remove ${label}`,
		retry: 'Retry',
		loading: 'Loading options',
		error: 'Could not load options',
		options: [
			{ value: 'communications', label: 'Communications', description: 'Canonical messages and source evidence', icon: 'tabler:messages' },
			{ value: 'knowledge', label: 'Knowledge', description: 'Reviewed facts and observations', icon: 'tabler:bulb' },
			{ value: 'projects', label: 'Projects', description: 'Bounded work context', icon: 'tabler:briefcase' },
			{ value: 'documents', label: 'Documents', description: 'Versioned evidence artifacts', icon: 'tabler:file-text' }
		],
		groups: [
			{
				id: 'memory',
				label: 'Memory',
				options: [
					{ value: 'communications', label: 'Communications' },
					{ value: 'knowledge', label: 'Knowledge' }
				]
			},
			{
				id: 'work',
				label: 'Work',
				options: [
					{ value: 'projects', label: 'Projects' },
					{ value: 'documents', label: 'Documents' }
				]
			}
		],
		tree: [
			{
				value: 'memory',
				label: 'Memory',
				children: [
					{ value: 'communications', label: 'Communications' },
					{ value: 'knowledge', label: 'Knowledge' }
				]
			},
			{
				value: 'work',
				label: 'Work',
				children: [
					{ value: 'projects', label: 'Projects' },
					{ value: 'documents', label: 'Documents' }
				]
			}
		]
	},
	foundation: {
		iconTitle: 'Icon system',
		iconDescription: 'Icons render through the Макошь Icon wrapper, not through domain-owned vendor imports.',
		sharedPrimitive: 'Shared primitive',
		separatorTitle: 'Separators',
		separatorDescription: 'Dividers use tokenized borders and stable orientation contracts.',
		scrollTitle: 'Scroll area',
		scrollDescription: 'Long local lists keep scroll behavior inside the primitive.',
		toastTitle: 'Toast viewport',
		toastDescription: 'Feedback surfaces stay tokenized and deterministic for visual baselines.',
		toasts: [
			{ id: 'visual-success', title: 'Evidence saved', description: 'Context update is ready for review.', variant: 'success' as const },
			{ id: 'visual-warning', title: 'Needs review', description: 'Provider-derived signal is not durable truth yet.', variant: 'warning' as const }
		],
		timelineItems: [
			'Provider signal observed',
			'Canonical evidence linked',
			'Review candidate created',
			'Context pack refreshed',
			'Owner decision pending'
		],
		separatorItems: ['Inbox', 'Review', 'Memory']
	},
	overlay: {
		title: 'Overlay primitives',
		description: 'Dropdown, dialog, sheet, tooltip and popover behavior comes from Reka UI. Макошь owns the style.',
		menu: 'Макошь menu',
		navigation: 'Navigation',
		communications: 'Communications',
		radar: 'Radar',
		settings: 'Settings',
		openDialog: 'Open dialog',
		openSheet: 'Open sheet',
		context: 'Context',
		popoverTitle: 'Context popover',
		popoverDescription: 'Small contextual surfaces stay lightweight and disappear quickly.',
		tooltipButton: 'Tooltip',
		tooltipContent: 'No sidebar tax, only temporary surfaces.',
		dialogTitle: 'Create Radar item',
		dialogDescription: 'Capture a signal before promoting it to a domain object.',
		dialogBody: 'Dialog content is isolated UI state. Business decisions stay in TanStack queries and stores.',
		sheetTitle: 'Inspector surface',
		sheetDescription: 'Use for temporary context, not permanent columns.',
		sheetBody: 'Sheet is here for temporary work, settings and review panels.',
		modalSurfacesTitle: 'Modal surfaces',
		infrastructureTitle: 'Overlay infrastructure',
		openAlertDialog: 'Open alert dialog',
		alertDialogTitle: 'Discard candidate?',
		alertDialogDescription: 'This removes a local candidate from the review queue. Source evidence remains untouched.',
		alertDialogBody: 'Use AlertDialog only when the owner must explicitly confirm a risky UI action.',
		alertDialogAction: 'Discard candidate',
		alertDialogCancel: 'Keep candidate',
		openDrawer: 'Open drawer',
		drawerTitle: 'Drawer context',
		drawerDescription: 'Temporary dense context without becoming a permanent side panel.',
		drawerBody: 'Drawer is modal and focus-managed, but still UI-only.',
		hoverCardButton: 'Hover preview',
		hoverCardTitle: 'Evidence preview',
		hoverCardDescription: 'HoverCard is for optional context. Required decisions must be visible elsewhere.',
		overlayHostTitle: 'Overlay host',
		overlayHostDescription: 'A passive host layer for custom non-modal surfaces.',
		portalTitle: 'Portal surface',
		portalDescription: 'Portal keeps overlay placement inside Макошь UI Kit.',
		focusTrapTitle: 'Focus trap',
		focusTrapDescription: 'Tab and Shift+Tab stay inside this demo region.'
	},
	feedback: {
		title: 'Feedback primitives',
		description: 'Status, loading and notification components for UI-only product surfaces.',
		surfacesTitle: 'Feedback surfaces',
		notificationTitle: 'Evidence saved',
		notificationDescription: 'The context update is queued for owner review.',
		bannerTitle: 'Review mode is active',
		bannerDescription: 'Provider-derived signals remain candidates until promoted.',
		alertTitle: 'Evidence is incomplete',
		alertDescription: 'Attach canonical evidence before making a durable decision.',
		inlineSuccess: 'Local validation passed.',
		inlineWarning: 'Long provider labels wrap without resizing the control surface.',
		loadingTitle: 'Loading states',
		loadingDescription: 'Deterministic spinners and progress indicators for dense desktop views.',
		progressLabel: 'Context pack build',
		circularLabel: 'Review progress',
		overlayLabel: 'Refreshing evidence',
		overlayDescription: 'This overlay belongs to the local panel, not the application shell.',
		statusTitle: 'Status indicators',
		presenceTitle: 'Presence indicators',
		statuses: [
			{ tone: 'success', label: 'Synchronized', pulse: false },
			{ tone: 'warning', label: 'Needs review', pulse: true },
			{ tone: 'danger', label: 'Blocked', pulse: false },
			{ tone: 'info', label: 'Observed', pulse: false }
		],
		presences: [
			{ status: 'online', label: 'Owner online' },
			{ status: 'away', label: 'In focus mode' },
			{ status: 'busy', label: 'Do not disturb' },
			{ status: 'offline', label: 'Offline' }
		],
		toasts: [
			{ id: 'feedback-toast-success', title: 'Context ready', description: 'Review snapshot is available.', variant: 'success' as const },
			{ id: 'feedback-toast-warning', title: 'Needs evidence', description: 'One source is still missing.', variant: 'warning' as const }
		],
		action: 'Open review'
	},
	themes: {
		options: [
			{ value: 'base-light', label: 'Base Light', description: 'Primary clean neutral light theme.' },
			{ value: 'base-dark', label: 'Base Dark', description: 'Neutral dark theme with the same component contracts.' },
			{ value: 'makosh-light', label: 'Макошь Light', description: 'Bright Макошь theme with emerald system accents.' },
			{ value: 'makosh-dark', label: 'Макошь Dark', description: 'Emerald Макошь dark theme on the same tokens.' }
		],
		swatches: ['Background', 'Surface', 'Raised', 'Text', 'Muted', 'Accent', 'Danger', 'Border'],
		cardTitle: 'Макошь surface',
		cardDescription: 'Same component, different token set.',
		searchValue: 'Search Telegram, Radar, Knowledge',
		contextBadge: 'Context'
	}
}

type StorybookText = typeof en

const ru: StorybookText = {
	common: {
		cancel: 'Отмена',
		create: 'Создать',
		done: 'Готово',
		close: 'Закрыть',
		evidence: 'Доказательство',
		openContext: 'Открыть контекст',
		primary: 'Основное',
		review: 'Проверка',
		risk: 'Риск',
		save: 'Сохранить',
		trusted: 'Надежно'
	},
	button: {
		runAction: 'Выполнить действие',
		title: 'Кнопки',
		description: 'Строгая поверхность управления для доменных компонентов. Provider-поведение здесь не живет.',
		primary: 'Основная',
		secondary: 'Вторичная',
		outline: 'Контурная',
		ghost: 'Тихая',
		delete: 'Удалить',
		small: 'Малая',
		medium: 'Средняя',
		large: 'Большая',
		loading: 'Загрузка',
		searchLabel: 'Поиск'
	},
	primitives: {
		title: 'Примитивные компоненты',
		description: 'Базовые UI-only блоки для типографики, меток, действий и поверхностей.',
		typographyTitle: 'Типографика',
		typographyDescription: 'Читаемые текстовые примитивы со стабильными контрактами размера, тона и веса.',
		heading: 'Контекстная поверхность для проверки',
		paragraph: 'Примитивы Макошь не несут доменный язык в shared UI и сохраняют предсказуемую раскладку.',
		muted: 'Приглушенный вспомогательный текст',
		strong: 'Акцентированный текст',
		accent: 'Текст акцента',
		chipsTitle: 'Chips и tags',
		chipsDescription: 'Компактные metadata-примитивы для нейтральных, статусных и акцентных меток.',
		chips: ['Кандидат', 'Проверено', 'Нужно доказательство', 'Риск'],
		actionsTitle: 'Текстовые действия',
		actionsDescription: 'Действия низкого веса для плотных поверхностей, где обычные кнопки слишком тяжелые.',
		openDocs: 'Открыть docs',
		quietAction: 'Тихое действие',
		dangerAction: 'Опасное действие',
		surfacesTitle: 'Layout-поверхности',
		surfacesDescription: 'Panel, Paper и Container для композиции без доменной логики.'
	},
	command: {
		title: 'Командная поверхность',
		description: 'Командная палитра ускоряет работу. Предсказуемая навигация меню может жить отдельно.',
		open: 'Открыть команды',
		selected: 'Выбрано',
		groups: [
			{
				label: 'Навигация',
				items: [
					{ id: 'communications', label: 'Коммуникации', description: 'Почта, Telegram, WhatsApp и provider-каналы в одном месте', icon: 'tabler:messages' },
					{ id: 'radar', label: 'Радар', description: 'Сигналы, наблюдения и очередь проверки', icon: 'tabler:radar' },
					{ id: 'knowledge', label: 'Знания', description: 'Заметки, граф и контекстная память', icon: 'tabler:brain' }
				]
			},
			{
				label: 'Действия',
				items: [
					{ id: 'new-note', label: 'Создать заметку', description: 'Зафиксировать мысль без фальшивых бизнес-сущностей', icon: 'tabler:note' },
					{ id: 'new-task', label: 'Создать задачу', description: 'Только после проверки или явного намерения', icon: 'tabler:checkbox' }
				]
			}
		]
	},
	navigation: {
		title: 'Навигационные примитивы',
		description: 'UI-only навигационные компоненты для плотных desktop-поверхностей.',
		breadcrumbLabel: 'Путь контекста',
		productNavigation: 'Продуктовая навигация',
		localMenus: 'Локальные меню',
		contextMenu: 'Контекстное меню',
		contextTrigger: 'Целевая область',
		menubar: 'Меню приложения',
		tree: 'Дерево workspace',
		palettes: 'Командная и поисковая палитры',
		commandPalette: 'Открыть команды',
		searchPalette: 'Открыть поиск',
		commandPlaceholder: 'Выполнить команду',
		searchPlaceholder: 'Искать локальные действия',
		pagination: 'Страницы проверки',
		reviewContent: 'Содержимое навигации очереди проверки.',
		evidenceContent: 'Содержимое навигации доказательств.',
		memoryContent: 'Содержимое навигации памяти.',
		tabs: [
			{ id: 'review', label: 'Проверка' },
			{ id: 'evidence', label: 'Доказательства' },
			{ id: 'memory', label: 'Память' }
		],
		breadcrumbs: [
			{ id: 'home', label: 'Макошь' },
			{ id: 'workspace', label: 'Workspace' },
			{ id: 'review', label: 'Очередь проверки', current: true }
		],
		navItems: [
			{ id: 'communications', label: 'Коммуникации', icon: 'tabler:messages' },
			{ id: 'radar', label: 'Радар', icon: 'tabler:radar', current: true },
			{ id: 'knowledge', label: 'Знания', icon: 'tabler:brain' }
		],
		menuItems: [
			{ id: 'inbox', label: 'Входящие', icon: 'tabler:inbox' },
			{ id: 'review', label: 'Проверка', icon: 'tabler:checks', current: true },
			{ id: 'archive', label: 'Архив', icon: 'tabler:archive' }
		],
		contextItems: [
			{ id: 'copy', label: 'Скопировать ссылку', icon: 'tabler:copy' },
			{ id: 'open', label: 'Открыть контекст', icon: 'tabler:external-link' },
			{ id: 'disabled', label: 'Недоступное действие', icon: 'tabler:ban', disabled: true }
		],
		menubarItems: [
			{
				id: 'file',
				label: 'Файл',
				children: [
					{ id: 'new-context', label: 'Новый контекст' },
					{ id: 'export', label: 'Экспорт snapshot' }
				]
			},
			{
				id: 'view',
				label: 'Вид',
				children: [
					{ id: 'compact', label: 'Плотный режим' },
					{ id: 'comfortable', label: 'Свободный режим' }
				]
			}
		],
		treeItems: [
			{
				id: 'workspace',
				label: 'Workspace',
				icon: 'tabler:layout-dashboard',
				children: [
					{ id: 'review-queue', label: 'Очередь проверки', icon: 'tabler:checks' },
					{ id: 'evidence', label: 'Доказательства', icon: 'tabler:archive' }
				]
			},
			{
				id: 'signals',
				label: 'Сигналы',
				icon: 'tabler:radar',
				children: [
					{ id: 'high-risk', label: 'Высокий риск', icon: 'tabler:alert-triangle' },
					{ id: 'needs-context', label: 'Нужен контекст', icon: 'tabler:help-circle' }
				]
			}
		]
	},
	data: {
		title: 'Data Display примитивы',
		description: 'Таблицы, списки, metadata, timeline, metrics и локальные state-поверхности.',
		tableTitle: 'Таблицы',
		tableCaption: 'Кандидаты проверки',
		virtualTableTitle: 'Windowed table',
		listTitle: 'Списки',
		virtualListTitle: 'Windowed list',
		detailsTitle: 'Детали',
		propertyGridTitle: 'Сетка свойств',
		timelineTitle: 'Timeline и activity',
		metricsTitle: 'Metrics',
		statesTitle: 'Состояния',
		tableColumns: [
			{ key: 'source', label: 'Источник' },
			{ key: 'signal', label: 'Сигнал' },
			{ key: 'confidence', label: 'Уверенность', align: 'right' },
			{ key: 'state', label: 'Состояние' }
		],
		tableRows: [
			{ id: 'row-1', source: 'Mail', signal: 'Проверка бюджета', confidence: '82%', state: 'Проверка' },
			{ id: 'row-2', source: 'Calendar', signal: 'Follow-up решения', confidence: '76%', state: 'Ожидает' },
			{ id: 'row-3', source: 'Локальная заметка', signal: 'Context pack', confidence: '91%', state: 'Готово' },
			{ id: 'row-4', source: 'Document', signal: 'Связь доказательства', confidence: '68%', state: 'Нужен источник' }
		],
		listItems: [
			{ id: 'item-1', label: 'Каноническое доказательство', description: 'Контекст со ссылкой на источник доступен.', meta: 'готово', icon: 'tabler:archive', tone: 'success' },
			{ id: 'item-2', label: 'Кандидат проверки', description: 'Требует подтверждения владельца до promotion.', meta: 'очередь', icon: 'tabler:checks', tone: 'warning' },
			{ id: 'item-3', label: 'Context pack', description: 'Derived и rebuildable summary surface.', meta: 'local', icon: 'tabler:package', tone: 'accent' }
		],
		properties: [
			{ id: 'owner', label: 'Владелец', value: 'Local user', description: 'Граница персонального workspace.' },
			{ id: 'trust', label: 'Доверие', value: 'Высокое', description: 'Поверхность с evidence.', tone: 'success' },
			{ id: 'source', label: 'Источник', value: 'Observed', description: 'Сам по себе не provider truth.', tone: 'warning' },
			{ id: 'scope', label: 'Scope', value: 'UI only', description: 'Без domain mutation.' }
		],
		timelineItems: [
			{ id: 'observed', title: 'Сигнал замечен', description: 'Provider data не становится durable truth.', time: '09:20', icon: 'tabler:radar', tone: 'info' },
			{ id: 'linked', title: 'Доказательство связано', description: 'Каноническое доказательство прикреплено к кандидату.', time: '09:26', icon: 'tabler:link', tone: 'success' },
			{ id: 'review', title: 'Ожидается проверка владельца', description: 'Promotion явный и traceable.', time: '09:31', icon: 'tabler:checks', tone: 'warning' }
		],
		activityItems: [
			{ id: 'activity-1', title: 'Контекст обновлен', description: 'Локальная projection обновлена.', meta: 'сейчас', icon: 'tabler:refresh', tone: 'accent' },
			{ id: 'activity-2', title: 'Snapshot проверен', description: 'Visual baseline стабилен.', meta: '1м', icon: 'tabler:photo-check', tone: 'success' }
		],
		statistic: { label: 'Кандидаты', value: 24, trend: '+6', description: 'Готовы к проверке', tone: 'accent' },
		metric: { label: 'Оценка evidence', value: 88, unit: '%', delta: '+4', tone: 'success' },
		counter: { label: 'open', value: 7, max: 12, tone: 'warning' },
		states: {
			emptyTitle: 'Нет кандидатов',
			emptyDescription: 'Новые сигналы появятся здесь после observation.',
			loadingTitle: 'Обновление контекста',
			loadingDescription: 'Локальные projections перестраиваются.',
			errorTitle: 'Не удалось отрисовать поверхность',
			errorDescription: 'Поверхность-владелец может дать retry action.',
			noDataTitle: 'Нет локальных данных',
			noSearchTitle: 'Нет результатов поиска',
			noSearchDescription: 'Попробуйте более широкий локальный запрос.',
			noSearchQuery: 'provider root cache',
			offlineTitle: 'Offline режим',
			comingSoonTitle: 'Inspector actions'
		}
	},
	layout: {
		title: 'Layout-примитивы',
		description: 'UI-only композиция для плотных desktop shell, панелей, баров и scroll-регионов.',
		compositionTitle: 'Композиция',
		compositionDescription: 'Stack, Grid, Flex, Split и Resizable держат spacing и alignment на токенах.',
		shellTitle: 'Shell-поверхности',
		shellDescription: 'Dock, bars, panels и inspectors собирают app-like поверхности без routing или stores.',
		scrollTitle: 'Scroll и floating surfaces',
		scrollDescription: 'ScrollArea, VirtualScrollArea и FloatingPanel держат overflow и контекст локально.',
		stackTitle: 'Stacked review surface',
		stackDescription: 'Длинный текст остается читаемым, а layout предсказуемым на разных ширинах.',
		gridTitle: 'Responsive grid',
		splitPrimary: 'Основной workspace',
		splitSecondary: 'Вторичный контекст',
		resizableTitle: 'Resizable preview',
		dockLabel: 'Dock workspace',
		toolbarLabel: 'Layout tools',
		actionLabel: 'Layout actions',
		sideTitle: 'Context rail',
		inspectorTitle: 'Inspector доказательств',
		inspectorDescription: 'Generic properties, не provider runtime state.',
		topTitle: 'Макошь workspace',
		topDescription: 'UI Lab layout shell',
		bottomLabel: 'Footer workspace',
		statusLabel: 'Статус workspace',
		floatingTitle: 'Floating context',
		floatingDescription: 'Non-modal helper surface с tokenized elevation.',
		virtualLabel: 'Virtual scroll sample',
		cards: [
			{ id: 'card-1', title: 'Очередь проверки', description: 'Кандидаты для владельца со source evidence.', meta: '24' },
			{ id: 'card-2', title: 'Context packs', description: 'Derived summaries, которые можно перестроить.', meta: '8' },
			{ id: 'card-3', title: 'Local memory', description: 'Personal knowledge остается local-first.', meta: '152' }
		],
		navItems: ['Проверка', 'Доказательства', 'Память', 'Настройки'],
		toolbarActions: ['Фильтр', 'Группа', 'Сортировка'],
		actions: ['Отмена', 'Сохранить layout'],
		statusItems: [
			{ id: 'mode', label: 'Режим', value: 'Local', tone: 'accent' },
			{ id: 'sync', label: 'Sync', value: 'Готово', tone: 'success' },
			{ id: 'risk', label: 'Риск', value: 'Низкий', tone: 'info' }
		],
		virtualItems: [
			'Provider-сигнал замечен',
			'Каноническое доказательство связано',
			'Кандидат поставлен на проверку',
			'Context pack перестроен',
			'Inspector panel открыт',
			'Локальная projection обновлена',
			'Visual baseline проверен',
			'Keyboard order проверен',
			'Reduced motion соблюден',
			'Решение владельца ожидается'
		]
	},
	media: {
		title: 'Media-примитивы',
		description: 'Generic preview для изображений, playback, документов, HTML, markdown, code и attachments.',
		imagesTitle: 'Изображения и gallery',
		playbackTitle: 'Playback-shells',
		documentsTitle: 'Документы и source',
		attachmentsTitle: 'Attachment previews',
		galleryLabel: 'Gallery изображений доказательств',
		imageCaption: 'Локальный preview доказательства',
		emptyImage: 'Изображение недоступно',
		videoTitle: 'Фрагмент встречи',
		videoDescription: 'Native controls отрисовываются без backend media transfer.',
		videoFallback: 'Video source не приложен',
		audioTitle: 'Voice note',
		audioDescription: 'Audio playback остается generic и provider-neutral.',
		audioFallback: 'Audio source не приложен',
		markdownTitle: 'Markdown extract',
		markdownSource: '## Source summary\n- Evidence остается связанным\n- Derived text можно перестроить\n\n[Локальная ссылка](#)',
		codeTitle: 'Code block',
		codeSource: 'const evidence = {\n  source: "local",\n  reviewed: true\n}',
		syntaxTitle: 'Syntax highlight',
		htmlTitle: 'HTML body preview',
		htmlSource: '<article><h2>Prepared HTML</h2><p>Санитизированные фрагменты могут рендериться как semantic HTML.</p></article>',
		textSource: 'Plain text body\nсохраняет переносы\nбез provider-specific rendering.',
		unsafeHtml: 'HTML preview требует санитизированный content',
		pdfTitle: 'PDF shell',
		pdfDescription: 'Shell готов для safe local PDF URLs.',
		pdfFallback: 'PDF source не приложен',
		attachmentAction: 'Открыть',
		galleryItems: [
			{ id: 'img-1', alt: 'Timeline artifact', title: 'Timeline artifact', description: 'Source image preview с нейтральной рамкой.', meta: 'image/png' },
			{ id: 'img-2', alt: 'Context snapshot', title: 'Context snapshot', description: 'Visual evidence отделено от provider runtime.', meta: 'image/svg+xml' },
			{ id: 'img-3', alt: 'Review capture', title: 'Review capture', description: 'Gallery state остается локальным и UI-only.', meta: 'image/webp' }
		],
		attachments: [
			{ id: 'att-1', name: 'context-pack.pdf', mimeType: 'application/pdf', size: '284 KB', description: 'Prepared document preview.', icon: 'tabler:file-type-pdf', tone: 'danger' },
			{ id: 'att-2', name: 'message-body.html', mimeType: 'text/html', size: '18 KB', description: 'Санитизированный HTML fragment.', icon: 'tabler:html', tone: 'warning' },
			{ id: 'att-3', name: 'notes.md', mimeType: 'text/markdown', size: '9 KB', description: 'Markdown source excerpt.', icon: 'tabler:markdown', tone: 'accent' }
		]
	},
	communication: {
		title: 'Communication-примитивы',
		description: 'UI-only message, composer и delivery primitives без provider behavior.',
		threadTitle: 'Review thread',
		composerTitle: 'Composer shell',
		composerLabel: 'Ответ с контекстом',
		placeholder: 'Напишите provider-neutral заметку',
		helper: 'Ctrl+Enter или Meta+Enter отправляет локальный draft.',
		send: 'Отправить',
		attach: 'Приложить evidence',
		typingLabel: 'Context assistant печатает',
		quoteAuthor: 'Source excerpt',
		quoteBody: 'Evidence остается видимым до promotion.',
		toolbarLabel: 'Composer tools',
		deliveryDescription: 'Provider execution представлен состоянием родителя.',
		readLabel: 'Прочитано reviewers',
		reactions: [
			{ emoji: '+', count: 4, label: 'Четыре positive reactions' },
			{ emoji: '!', count: 1, label: 'Одна reaction требует внимания' }
		],
		actions: [
			{ id: 'bold', label: 'Bold', icon: 'tabler:bold' },
			{ id: 'quote', label: 'Quote', icon: 'tabler:quote' },
			{ id: 'risk', label: 'Risk', icon: 'tabler:alert-triangle', tone: 'warning' as const }
		],
		messages: [
			{ id: 'msg-1', author: 'Alex', timestamp: '09:12', meta: 'source-linked', direction: 'inbound' as const, body: 'Сохраним original evidence до превращения этого в task?' },
			{ id: 'msg-2', author: 'Макошь', timestamp: '09:13', meta: 'review candidate', direction: 'outbound' as const, body: 'Да. Candidate остается отдельно до owner review.' },
			{ id: 'msg-3', author: 'System', timestamp: '09:14', meta: 'local state', direction: 'system' as const, body: 'Delivery state в shared UI только визуальный.' }
		],
		attachments: [
			{ name: 'evidence.html', meta: 'text/html', icon: 'tabler:html', tone: 'warning' as const },
			{ name: 'context.md', meta: '9 KB', icon: 'tabler:markdown', tone: 'accent' as const }
		],
		receipts: [
			{ id: 'rr-1', label: 'Owner', initials: 'OW' },
			{ id: 'rr-2', label: 'Reviewer', initials: 'RV' },
			{ id: 'rr-3', label: 'Archive', initials: 'AR' },
			{ id: 'rr-4', label: 'Memory', initials: 'ME' }
		]
	},
	utility: {
		title: 'Utility-примитивы',
		description: 'Малые UI helpers для copy actions, switchers, keyboard hints и semantic icons.',
		copyTitle: 'Copy action',
		copyValue: 'makosh://local/context-pack',
		copyLabel: 'Скопировать reference',
		copiedLabel: 'Скопировано',
		errorLabel: 'Copy недоступен',
		themeTitle: 'Theme selector',
		localeTitle: 'Locale selector',
		shortcutsTitle: 'Keyboard hints',
		iconsTitle: 'Semantic icons',
		openCommand: 'Открыть command',
		sendDraft: 'Отправить draft',
		providerTitle: 'Providers',
		statusTitle: 'Statuses',
		entityTitle: 'Entities',
		fileTitle: 'Files',
		locales: [
			{ value: 'ru', label: 'RU', description: 'Русский' },
			{ value: 'en', label: 'EN', description: 'English' },
			{ value: 'es', label: 'ES', description: 'Español' }
		],
		providers: ['mail', 'telegram', 'whatsapp', 'calendar', 'documents', 'generic'] as const,
		statuses: ['idle', 'active', 'success', 'warning', 'danger', 'offline', 'syncing'] as const,
		entities: ['person', 'organization', 'project', 'task', 'document', 'decision', 'obligation', 'knowledge', 'event', 'generic'] as const,
		files: [
			{ label: 'Image', mimeType: 'image/png' },
			{ label: 'PDF', mimeType: 'application/pdf' },
			{ label: 'Code', mimeType: 'application/typescript' },
			{ label: 'Archive', mimeType: 'application/zip' }
		]
	},
	editor: {
		title: 'Редактор контекста',
		description: 'Компактная rich text поверхность для заметок, черновиков писем и evidence-фрагментов. Она остается семантической, а не превращается в Word-подобную ленту.',
		label: 'Редактор контекста',
		helper: 'Используйте компактные semantic actions: заголовки, цитаты, списки, marks, code, divider и evidence-links.',
		placeholder: 'Зафиксируйте решение, процитируйте evidence и оставьте источник видимым.',
		toolbarLabel: 'Инструменты rich text',
		outputLabel: 'Sanitized output',
		previewTitle: 'Sanitized HTML preview',
		previewEmpty: 'HTML output пока пуст',
		keyboardLabel: 'Быстрая отправка',
		actions: [
			{ id: 'paragraph', label: 'Абзац', icon: 'tabler:pilcrow', group: 'structure' },
			{ id: 'heading', label: 'Заголовок', icon: 'tabler:h-2', group: 'structure' },
			{ id: 'subheading', label: 'Подзаголовок', icon: 'tabler:h-3', group: 'structure' },
			{ id: 'quote', label: 'Цитата', icon: 'tabler:quote', group: 'structure' },
			{ id: 'bulletList', label: 'Маркированный список', icon: 'tabler:list', group: 'lists' },
			{ id: 'orderedList', label: 'Нумерованный список', icon: 'tabler:list-numbers', group: 'lists' },
			{ id: 'bold', label: 'Акцент', icon: 'tabler:bold', group: 'marks' },
			{ id: 'italic', label: 'Нюанс', icon: 'tabler:italic', group: 'marks' },
			{ id: 'underline', label: 'Подчеркнуть', icon: 'tabler:underline', group: 'marks' },
			{ id: 'strike', label: 'Зачеркнуть', icon: 'tabler:strikethrough', group: 'marks' },
			{ id: 'code', label: 'Inline code', icon: 'tabler:code', group: 'marks' },
			{ id: 'link', label: 'Evidence-link', icon: 'tabler:link', group: 'insert' },
			{ id: 'codeBlock', label: 'Code block', icon: 'tabler:code-dots', group: 'insert' },
			{ id: 'horizontalRule', label: 'Divider', icon: 'tabler:separator-horizontal', group: 'insert' },
			{ id: 'clearFormatting', label: 'Очистить формат', icon: 'tabler:eraser', group: 'cleanup' }
		],
		initialHtml: '<h2>Контекст решения</h2><p>Сохраняйте <code>provider_message</code> как evidence до promotion.</p><ol><li><p>Сохранить источник.</p></li><li><p>Проверить до durable truth.</p></li></ol><blockquote><p>Источник остается доступным для review.</p></blockquote>'
	},
	form: {
		title: 'Поля формы',
		search: 'Поиск',
		searchValue: 'runtime telegram',
		searchPlaceholder: 'Искать везде',
		email: 'Email владельца',
		emailValue: 'owner@example.local',
			password: 'Пароль vault',
			passwordValue: 'correct horse battery staple',
		count: 'Лимит проверки',
		countValue: 7,
		otp: 'Код проверки',
		otpValue: '482913',
		otpHint: 'Вставьте или введите короткий локальный код.',
		domain: 'Домен',
		multiSelect: 'Связанные домены',
		combobox: 'Значение combobox',
		autocomplete: 'Значение autocomplete',
		noResults: 'Нет локальных вариантов',
		color: 'Цвет акцента',
		date: 'Дата проверки',
		time: 'Время проверки',
		dateTime: 'Дата и время проверки',
		file: 'Выбор файла',
		noFiles: 'Файлы не выбраны',
		dropZone: 'Перетащите вложения',
		dropZoneHint: 'С клавиатуры нажмите Enter или Space',
		contextNote: 'Контекстная заметка',
		noteValue: 'Provider-детали остаются вне пользовательского интерфейса коммуникаций.',
		hint: 'UI-компоненты отдают только состояние; валидация принадлежит форме-владельцу.',
		error: 'Этот пример ошибки рендерит FormError.',
		counterMax: 96,
		checkbox: 'Требовать проверку владельца',
		radioTitle: 'Уверенность сигнала',
		radioOptions: [
			{ value: 'low', label: 'Низкая' },
			{ value: 'medium', label: 'Средняя' },
			{ value: 'high', label: 'Высокая' }
		],
		slider: 'Порог уверенности',
		range: 'Диапазон оценки доказательств',
		realtime: 'Контекст в реальном времени включен',
		saveContract: 'Сохранить контракт компонента',
		options: [
			{ value: 'communications', label: 'Коммуникации' },
			{ value: 'radar', label: 'Радар' },
			{ value: 'knowledge', label: 'Знания' }
		]
	},
	selection: {
		title: 'Контролы выбора',
		select: 'Выбор',
		searchableSelect: 'Выбор с поиском',
		multiSelect: 'Множественный выбор',
		searchableMultiSelect: 'Множественный выбор с поиском',
		groupedSelect: 'Группированный выбор',
		treeSelect: 'Иерархический выбор',
		cascader: 'Каскадный выбор',
		asyncSelect: 'Асинхронный выбор',
		placeholder: 'Выберите контекст',
		searchPlaceholder: 'Искать локальные варианты',
		empty: 'Нет подходящих вариантов',
		clear: 'Очистить выбор',
		searchLabel: 'Поиск вариантов',
		optionsLabel: 'Доступные варианты',
		actionsLabel: 'Действия выбора',
		selectAll: 'Выбрать все',
		clearAll: 'Очистить все',
		selectedCount: (count: number) => `Выбрано: ${count}`,
		remove: (label: string) => `Удалить ${label}`,
		retry: 'Повторить',
		loading: 'Загрузка вариантов',
		error: 'Не удалось загрузить варианты',
		options: [
			{ value: 'communications', label: 'Коммуникации', description: 'Канонические сообщения и исходные evidence', icon: 'tabler:messages' },
			{ value: 'knowledge', label: 'Знания', description: 'Проверенные факты и наблюдения', icon: 'tabler:bulb' },
			{ value: 'projects', label: 'Проекты', description: 'Ограниченный рабочий контекст', icon: 'tabler:briefcase' },
			{ value: 'documents', label: 'Документы', description: 'Версионированные evidence-артефакты', icon: 'tabler:file-text' }
		],
		groups: [
			{
				id: 'memory',
				label: 'Память',
				options: [
					{ value: 'communications', label: 'Коммуникации' },
					{ value: 'knowledge', label: 'Знания' }
				]
			},
			{
				id: 'work',
				label: 'Работа',
				options: [
					{ value: 'projects', label: 'Проекты' },
					{ value: 'documents', label: 'Документы' }
				]
			}
		],
		tree: [
			{
				value: 'memory',
				label: 'Память',
				children: [
					{ value: 'communications', label: 'Коммуникации' },
					{ value: 'knowledge', label: 'Знания' }
				]
			},
			{
				value: 'work',
				label: 'Работа',
				children: [
					{ value: 'projects', label: 'Проекты' },
					{ value: 'documents', label: 'Документы' }
				]
			}
		]
	},
	foundation: {
		iconTitle: 'Система иконок',
		iconDescription: 'Иконки проходят через Макошь Icon wrapper, а не через vendor-импорты в доменах.',
		sharedPrimitive: 'Общий примитив',
		separatorTitle: 'Разделители',
		separatorDescription: 'Разделители используют токены границ и стабильный контракт ориентации.',
		scrollTitle: 'Область прокрутки',
		scrollDescription: 'Длинные локальные списки держат прокрутку внутри примитива.',
		toastTitle: 'Toast viewport',
		toastDescription: 'Feedback-поверхности остаются токенизированными и детерминированными для baseline.',
		toasts: [
			{ id: 'visual-success', title: 'Доказательство сохранено', description: 'Обновление контекста готово к проверке.', variant: 'success' },
			{ id: 'visual-warning', title: 'Нужна проверка', description: 'Provider-сигнал еще не является долговечной истиной.', variant: 'warning' }
		],
		timelineItems: [
			'Provider-сигнал замечен',
			'Каноническое доказательство связано',
			'Кандидат проверки создан',
			'Context pack обновлен',
			'Решение владельца ожидается'
		],
		separatorItems: ['Входящие', 'Проверка', 'Память']
	},
	overlay: {
		title: 'Overlay-примитивы',
		description: 'Dropdown, dialog, sheet, tooltip и popover используют поведение Reka UI. Стилем владеет Макошь.',
		menu: 'Меню Макошь',
		navigation: 'Навигация',
		communications: 'Коммуникации',
		radar: 'Радар',
		settings: 'Настройки',
		openDialog: 'Открыть dialog',
		openSheet: 'Открыть sheet',
		context: 'Контекст',
		popoverTitle: 'Контекстный popover',
		popoverDescription: 'Малые контекстные поверхности остаются легкими и быстро исчезают.',
		tooltipButton: 'Tooltip',
		tooltipContent: 'Никакого налога боковой панели, только временные поверхности.',
		dialogTitle: 'Создать элемент радара',
		dialogDescription: 'Зафиксировать сигнал до превращения в доменный объект.',
		dialogBody: 'Содержимое dialog изолирует UI state. Бизнес-решения остаются в TanStack queries и stores.',
		sheetTitle: 'Поверхность инспектора',
		sheetDescription: 'Используется для временного контекста, а не постоянных колонок.',
		sheetBody: 'Sheet предназначен для временной работы, настроек и review-панелей.',
		modalSurfacesTitle: 'Модальные поверхности',
		infrastructureTitle: 'Overlay-инфраструктура',
		openAlertDialog: 'Открыть alert dialog',
		alertDialogTitle: 'Удалить кандидата?',
		alertDialogDescription: 'Это убирает локального кандидата из очереди проверки. Source evidence не меняется.',
		alertDialogBody: 'AlertDialog нужен только там, где владелец явно подтверждает рискованное UI-действие.',
		alertDialogAction: 'Удалить кандидата',
		alertDialogCancel: 'Оставить кандидата',
		openDrawer: 'Открыть drawer',
		drawerTitle: 'Контекст drawer',
		drawerDescription: 'Временный плотный контекст без превращения в постоянную боковую панель.',
		drawerBody: 'Drawer модальный и управляет фокусом, но остается UI-only.',
		hoverCardButton: 'Hover preview',
		hoverCardTitle: 'Preview доказательства',
		hoverCardDescription: 'HoverCard дает дополнительный контекст. Обязательные решения должны быть видны отдельно.',
		overlayHostTitle: 'Overlay host',
		overlayHostDescription: 'Пассивный слой host для кастомных non-modal поверхностей.',
		portalTitle: 'Portal surface',
		portalDescription: 'Portal держит placement overlay внутри Макошь UI Kit.',
		focusTrapTitle: 'Focus trap',
		focusTrapDescription: 'Tab и Shift+Tab остаются внутри этой demo-области.'
	},
	feedback: {
		title: 'Feedback-примитивы',
		description: 'Компоненты статуса, загрузки и уведомлений для UI-only продуктовых поверхностей.',
		surfacesTitle: 'Feedback-поверхности',
		notificationTitle: 'Доказательство сохранено',
		notificationDescription: 'Обновление контекста поставлено в очередь проверки владельцем.',
		bannerTitle: 'Режим проверки активен',
		bannerDescription: 'Provider-сигналы остаются кандидатами до promotion.',
		alertTitle: 'Доказательство неполное',
		alertDescription: 'Свяжите каноническое доказательство перед долговечным решением.',
		inlineSuccess: 'Локальная валидация прошла.',
		inlineWarning: 'Длинные provider-метки переносятся без изменения размера control surface.',
		loadingTitle: 'Состояния загрузки',
		loadingDescription: 'Детерминированные spinners и progress indicators для плотных desktop-поверхностей.',
		progressLabel: 'Сборка context pack',
		circularLabel: 'Прогресс проверки',
		overlayLabel: 'Обновление доказательств',
		overlayDescription: 'Этот overlay принадлежит локальной панели, а не оболочке приложения.',
		statusTitle: 'Индикаторы статуса',
		presenceTitle: 'Индикаторы присутствия',
		statuses: [
			{ tone: 'success', label: 'Синхронизировано', pulse: false },
			{ tone: 'warning', label: 'Нужна проверка', pulse: true },
			{ tone: 'danger', label: 'Заблокировано', pulse: false },
			{ tone: 'info', label: 'Наблюдается', pulse: false }
		],
		presences: [
			{ status: 'online', label: 'Владелец онлайн' },
			{ status: 'away', label: 'Фокус-режим' },
			{ status: 'busy', label: 'Не беспокоить' },
			{ status: 'offline', label: 'Офлайн' }
		],
		toasts: [
			{ id: 'feedback-toast-success', title: 'Контекст готов', description: 'Review snapshot доступен.', variant: 'success' },
			{ id: 'feedback-toast-warning', title: 'Нужно доказательство', description: 'Один источник еще отсутствует.', variant: 'warning' }
		],
		action: 'Открыть проверку'
	},
	themes: {
		options: [
			{ value: 'base-light', label: 'Базовая светлая', description: 'Основная чистая нейтральная тема.' },
			{ value: 'base-dark', label: 'Базовая темная', description: 'Нейтральная темная тема с теми же контрактами компонентов.' },
			{ value: 'makosh-light', label: 'Макошь светлая', description: 'Светлая тема Макошь с emerald-акцентами системы.' },
			{ value: 'makosh-dark', label: 'Макошь темная', description: 'Темная тема Макошь с emerald-акцентами на тех же токенах.' }
		],
		swatches: ['Фон', 'Поверхность', 'Raised', 'Текст', 'Muted', 'Акцент', 'Опасность', 'Граница'],
		cardTitle: 'Поверхность Макошь',
		cardDescription: 'Тот же компонент, другой набор токенов.',
		searchValue: 'Поиск Telegram, Radar, Knowledge',
		contextBadge: 'Контекст'
	}
}

const es: StorybookText = {
	common: {
		cancel: 'Cancelar',
		create: 'Crear',
		done: 'Listo',
		close: 'Cerrar',
		evidence: 'Evidencia',
		openContext: 'Abrir contexto',
		primary: 'Principal',
		review: 'Revisión',
		risk: 'Riesgo',
		save: 'Guardar',
		trusted: 'Confiable'
	},
	button: {
		runAction: 'Ejecutar acción',
		title: 'Botones',
		description: 'Superficie de control estricta para componentes de dominio. Aquí no vive lógica de proveedores.',
		primary: 'Principal',
		secondary: 'Secundario',
		outline: 'Contorno',
		ghost: 'Ligero',
		delete: 'Eliminar',
		small: 'Pequeño',
		medium: 'Mediano',
		large: 'Grande',
		loading: 'Cargando',
		searchLabel: 'Buscar'
	},
	primitives: {
		title: 'Componentes primitivos',
		description: 'Bloques UI-only base para tipografía, etiquetas, acciones y superficies.',
		typographyTitle: 'Tipografía',
		typographyDescription: 'Primitivos de texto legibles con contratos estables de tamaño, tono y peso.',
		heading: 'Superficie de contexto lista para revisión',
		paragraph: 'Los primitivos Макошь mantienen el lenguaje de dominio fuera de shared UI y preservan layout predecible.',
		muted: 'Texto auxiliar atenuado',
		strong: 'Texto fuerte',
		accent: 'Texto de acento',
		chipsTitle: 'Chips y tags',
		chipsDescription: 'Primitivos compactos de metadata para etiquetas neutrales, de estado y de acento.',
		chips: ['Candidato', 'Revisado', 'Requiere evidencia', 'Riesgo'],
		actionsTitle: 'Acciones de texto',
		actionsDescription: 'Acciones de baja prioridad para superficies densas donde los botones completos pesan demasiado.',
		openDocs: 'Abrir docs',
		quietAction: 'Acción discreta',
		dangerAction: 'Acción peligrosa',
		surfacesTitle: 'Superficies de layout',
		surfacesDescription: 'Panel, Paper y Container para composición sin lógica de dominio.'
	},
	command: {
		title: 'Superficie de comandos',
		description: 'La paleta de comandos acelera el trabajo. La navegación predecible puede vivir aparte.',
		open: 'Abrir comandos',
		selected: 'Seleccionado',
		groups: [
			{
				label: 'Navegación',
				items: [
					{ id: 'communications', label: 'Comunicaciones', description: 'Correo, Telegram, WhatsApp y canales de proveedor unificados', icon: 'tabler:messages' },
					{ id: 'radar', label: 'Radar', description: 'Señales, observaciones y cola de revisión', icon: 'tabler:radar' },
					{ id: 'knowledge', label: 'Conocimiento', description: 'Notas, grafo y memoria de contexto', icon: 'tabler:brain' }
				]
			},
			{
				label: 'Acciones',
				items: [
					{ id: 'new-note', label: 'Crear nota', description: 'Capturar una idea sin crear entidades falsas', icon: 'tabler:note' },
					{ id: 'new-task', label: 'Crear tarea', description: 'Solo después de revisión o intención explícita', icon: 'tabler:checkbox' }
				]
			}
		]
	},
	navigation: {
		title: 'Primitivos de navegación',
		description: 'Componentes de navegación UI-only para superficies desktop densas.',
		breadcrumbLabel: 'Ruta de contexto',
		productNavigation: 'Navegación de producto',
		localMenus: 'Menús locales',
		contextMenu: 'Menú contextual',
		contextTrigger: 'Objetivo contextual',
		menubar: 'Menú de aplicación',
		tree: 'Árbol de workspace',
		palettes: 'Paletas de comandos y búsqueda',
		commandPalette: 'Abrir comandos',
		searchPalette: 'Abrir búsqueda',
		commandPlaceholder: 'Ejecutar comando',
		searchPlaceholder: 'Buscar acciones locales',
		pagination: 'Páginas de revisión',
		reviewContent: 'Contenido de navegación de revisión.',
		evidenceContent: 'Contenido de navegación de evidencia.',
		memoryContent: 'Contenido de navegación de memoria.',
		tabs: [
			{ id: 'review', label: 'Revisión' },
			{ id: 'evidence', label: 'Evidencia' },
			{ id: 'memory', label: 'Memoria' }
		],
		breadcrumbs: [
			{ id: 'home', label: 'Макошь' },
			{ id: 'workspace', label: 'Workspace' },
			{ id: 'review', label: 'Cola de revisión', current: true }
		],
		navItems: [
			{ id: 'communications', label: 'Comunicaciones', icon: 'tabler:messages' },
			{ id: 'radar', label: 'Radar', icon: 'tabler:radar', current: true },
			{ id: 'knowledge', label: 'Conocimiento', icon: 'tabler:brain' }
		],
		menuItems: [
			{ id: 'inbox', label: 'Bandeja', icon: 'tabler:inbox' },
			{ id: 'review', label: 'Revisión', icon: 'tabler:checks', current: true },
			{ id: 'archive', label: 'Archivo', icon: 'tabler:archive' }
		],
		contextItems: [
			{ id: 'copy', label: 'Copiar referencia', icon: 'tabler:copy' },
			{ id: 'open', label: 'Abrir contexto', icon: 'tabler:external-link' },
			{ id: 'disabled', label: 'Acción no disponible', icon: 'tabler:ban', disabled: true }
		],
		menubarItems: [
			{
				id: 'file',
				label: 'Archivo',
				children: [
					{ id: 'new-context', label: 'Nuevo contexto' },
					{ id: 'export', label: 'Exportar snapshot' }
				]
			},
			{
				id: 'view',
				label: 'Vista',
				children: [
					{ id: 'compact', label: 'Densidad compacta' },
					{ id: 'comfortable', label: 'Densidad cómoda' }
				]
			}
		],
		treeItems: [
			{
				id: 'workspace',
				label: 'Workspace',
				icon: 'tabler:layout-dashboard',
				children: [
					{ id: 'review-queue', label: 'Cola de revisión', icon: 'tabler:checks' },
					{ id: 'evidence', label: 'Evidencia', icon: 'tabler:archive' }
				]
			},
			{
				id: 'signals',
				label: 'Señales',
				icon: 'tabler:radar',
				children: [
					{ id: 'high-risk', label: 'Alto riesgo', icon: 'tabler:alert-triangle' },
					{ id: 'needs-context', label: 'Necesita contexto', icon: 'tabler:help-circle' }
				]
			}
		]
	},
	data: {
		title: 'Primitivos de data display',
		description: 'Tablas, listas, metadata, timeline, métricas y superficies locales de estado.',
		tableTitle: 'Tablas',
		tableCaption: 'Candidatos de revisión',
		virtualTableTitle: 'Tabla con ventana',
		listTitle: 'Listas',
		virtualListTitle: 'Lista con ventana',
		detailsTitle: 'Detalles',
		propertyGridTitle: 'Grid de propiedades',
		timelineTitle: 'Timeline y actividad',
		metricsTitle: 'Métricas',
		statesTitle: 'Estados',
		tableColumns: [
			{ key: 'source', label: 'Fuente' },
			{ key: 'signal', label: 'Señal' },
			{ key: 'confidence', label: 'Confianza', align: 'right' },
			{ key: 'state', label: 'Estado' }
		],
		tableRows: [
			{ id: 'row-1', source: 'Mail', signal: 'Revisión de presupuesto', confidence: '82%', state: 'Revisión' },
			{ id: 'row-2', source: 'Calendar', signal: 'Seguimiento de decisión', confidence: '76%', state: 'Pendiente' },
			{ id: 'row-3', source: 'Nota local', signal: 'Context pack', confidence: '91%', state: 'Listo' },
			{ id: 'row-4', source: 'Document', signal: 'Enlace de evidencia', confidence: '68%', state: 'Necesita fuente' }
		],
		listItems: [
			{ id: 'item-1', label: 'Evidencia canónica', description: 'Contexto con fuente vinculada disponible.', meta: 'listo', icon: 'tabler:archive', tone: 'success' },
			{ id: 'item-2', label: 'Candidato de revisión', description: 'Requiere confirmación del dueño antes de promover.', meta: 'cola', icon: 'tabler:checks', tone: 'warning' },
			{ id: 'item-3', label: 'Context pack', description: 'Superficie derivada y reconstruible.', meta: 'local', icon: 'tabler:package', tone: 'accent' }
		],
		properties: [
			{ id: 'owner', label: 'Dueño', value: 'Local user', description: 'Límite del workspace personal.' },
			{ id: 'trust', label: 'Confianza', value: 'Alta', description: 'Superficie respaldada por evidencia.', tone: 'success' },
			{ id: 'source', label: 'Fuente', value: 'Observed', description: 'No es verdad del proveedor por sí sola.', tone: 'warning' },
			{ id: 'scope', label: 'Scope', value: 'UI only', description: 'Sin mutación de dominio.' }
		],
		timelineItems: [
			{ id: 'observed', title: 'Señal observada', description: 'Los datos del proveedor no son durable truth.', time: '09:20', icon: 'tabler:radar', tone: 'info' },
			{ id: 'linked', title: 'Evidencia vinculada', description: 'La evidencia canónica queda adjunta al candidato.', time: '09:26', icon: 'tabler:link', tone: 'success' },
			{ id: 'review', title: 'Revisión del dueño pendiente', description: 'La promoción es explícita y trazable.', time: '09:31', icon: 'tabler:checks', tone: 'warning' }
		],
		activityItems: [
			{ id: 'activity-1', title: 'Contexto actualizado', description: 'La proyección local fue actualizada.', meta: 'ahora', icon: 'tabler:refresh', tone: 'accent' },
			{ id: 'activity-2', title: 'Snapshot revisado', description: 'El baseline visual es estable.', meta: '1m', icon: 'tabler:photo-check', tone: 'success' }
		],
		statistic: { label: 'Candidatos', value: 24, trend: '+6', description: 'Listos para revisión', tone: 'accent' },
		metric: { label: 'Score de evidencia', value: 88, unit: '%', delta: '+4', tone: 'success' },
		counter: { label: 'open', value: 7, max: 12, tone: 'warning' },
		states: {
			emptyTitle: 'Sin candidatos',
			emptyDescription: 'Las nuevas señales aparecerán aquí después de observarse.',
			loadingTitle: 'Actualizando contexto',
			loadingDescription: 'Las proyecciones locales se están reconstruyendo.',
			errorTitle: 'No se pudo renderizar la superficie',
			errorDescription: 'La superficie dueña puede ofrecer una acción de reintento.',
			noDataTitle: 'Sin datos locales',
			noSearchTitle: 'Sin resultados de búsqueda',
			noSearchDescription: 'Prueba una consulta local más amplia.',
			noSearchQuery: 'provider root cache',
			offlineTitle: 'Modo offline',
			comingSoonTitle: 'Inspector actions'
		}
	},
	layout: {
		title: 'Primitivos de layout',
		description: 'Composición UI-only para shells desktop densos, paneles, barras y regiones con scroll.',
		compositionTitle: 'Composición',
		compositionDescription: 'Stack, Grid, Flex, Split y Resizable mantienen spacing y alignment tokenizados.',
		shellTitle: 'Superficies shell',
		shellDescription: 'Dock, bars, panels e inspectors componen superficies tipo app sin routing ni stores.',
		scrollTitle: 'Scroll y superficies flotantes',
		scrollDescription: 'ScrollArea, VirtualScrollArea y FloatingPanel mantienen overflow y contexto locales.',
		stackTitle: 'Superficie de revisión apilada',
		stackDescription: 'El texto largo sigue legible mientras el layout permanece predecible en varios anchos.',
		gridTitle: 'Grid responsive',
		splitPrimary: 'Workspace principal',
		splitSecondary: 'Contexto secundario',
		resizableTitle: 'Preview redimensionable',
		dockLabel: 'Dock de workspace',
		toolbarLabel: 'Herramientas de layout',
		actionLabel: 'Acciones de layout',
		sideTitle: 'Rail de contexto',
		inspectorTitle: 'Inspector de evidencia',
		inspectorDescription: 'Propiedades genéricas, no estado runtime de proveedor.',
		topTitle: 'Макошь workspace',
		topDescription: 'Shell de layout UI Lab',
		bottomLabel: 'Footer de workspace',
		statusLabel: 'Estado de workspace',
		floatingTitle: 'Contexto flotante',
		floatingDescription: 'Superficie auxiliar no modal con elevación tokenizada.',
		virtualLabel: 'Muestra de virtual scroll',
		cards: [
			{ id: 'card-1', title: 'Cola de revisión', description: 'Candidatos para el dueño con source evidence.', meta: '24' },
			{ id: 'card-2', title: 'Context packs', description: 'Summaries derivados que se pueden reconstruir.', meta: '8' },
			{ id: 'card-3', title: 'Memoria local', description: 'El conocimiento personal sigue local-first.', meta: '152' }
		],
		navItems: ['Revisión', 'Evidencia', 'Memoria', 'Ajustes'],
		toolbarActions: ['Filtrar', 'Agrupar', 'Ordenar'],
		actions: ['Cancelar', 'Guardar layout'],
		statusItems: [
			{ id: 'mode', label: 'Modo', value: 'Local', tone: 'accent' },
			{ id: 'sync', label: 'Sync', value: 'Listo', tone: 'success' },
			{ id: 'risk', label: 'Riesgo', value: 'Bajo', tone: 'info' }
		],
		virtualItems: [
			'Señal de proveedor observada',
			'Evidencia canónica vinculada',
			'Candidato en cola de revisión',
			'Context pack reconstruido',
			'Inspector panel abierto',
			'Proyección local actualizada',
			'Visual baseline revisado',
			'Keyboard order revisado',
			'Reduced motion respetado',
			'Decisión del dueño pendiente'
		]
	},
	media: {
		title: 'Primitivos media',
		description: 'Previews genéricos para imágenes, playback, documentos, HTML, markdown, code y attachments.',
		imagesTitle: 'Imágenes y gallery',
		playbackTitle: 'Playback shells',
		documentsTitle: 'Documentos y source',
		attachmentsTitle: 'Attachment previews',
		galleryLabel: 'Gallery de imágenes de evidencia',
		imageCaption: 'Preview local de evidencia',
		emptyImage: 'Imagen no disponible',
		videoTitle: 'Clip de reunión',
		videoDescription: 'Native controls renderizan sin backend media transfer.',
		videoFallback: 'Video source no adjunto',
		audioTitle: 'Voice note',
		audioDescription: 'Audio playback sigue genérico y provider-neutral.',
		audioFallback: 'Audio source no adjunto',
		markdownTitle: 'Markdown extract',
		markdownSource: '## Source summary\n- Evidence sigue vinculada\n- Derived text se puede reconstruir\n\n[Referencia local](#)',
		codeTitle: 'Code block',
		codeSource: 'const evidence = {\n  source: "local",\n  reviewed: true\n}',
		syntaxTitle: 'Syntax highlight',
		htmlTitle: 'HTML body preview',
		htmlSource: '<article><h2>Prepared HTML</h2><p>Los fragmentos sanitizados pueden renderizar HTML semántico.</p></article>',
		textSource: 'Plain text body\nconserva saltos de línea\nsin provider-specific rendering.',
		unsafeHtml: 'HTML preview requiere content sanitizado',
		pdfTitle: 'PDF shell',
		pdfDescription: 'El shell está listo para safe local PDF URLs.',
		pdfFallback: 'PDF source no adjunto',
		attachmentAction: 'Inspeccionar',
		galleryItems: [
			{ id: 'img-1', alt: 'Timeline artifact', title: 'Timeline artifact', description: 'Source image preview con chrome neutral.', meta: 'image/png' },
			{ id: 'img-2', alt: 'Context snapshot', title: 'Context snapshot', description: 'Visual evidence queda separada del provider runtime.', meta: 'image/svg+xml' },
			{ id: 'img-3', alt: 'Review capture', title: 'Review capture', description: 'Gallery state es local y UI-only.', meta: 'image/webp' }
		],
		attachments: [
			{ id: 'att-1', name: 'context-pack.pdf', mimeType: 'application/pdf', size: '284 KB', description: 'Prepared document preview.', icon: 'tabler:file-type-pdf', tone: 'danger' },
			{ id: 'att-2', name: 'message-body.html', mimeType: 'text/html', size: '18 KB', description: 'Fragmento HTML sanitizado.', icon: 'tabler:html', tone: 'warning' },
			{ id: 'att-3', name: 'notes.md', mimeType: 'text/markdown', size: '9 KB', description: 'Markdown source excerpt.', icon: 'tabler:markdown', tone: 'accent' }
		]
	},
	communication: {
		title: 'Primitivos de comunicación',
		description: 'Message, composer y delivery primitives UI-only sin provider behavior.',
		threadTitle: 'Review thread',
		composerTitle: 'Composer shell',
		composerLabel: 'Respuesta con contexto',
		placeholder: 'Escribe una nota provider-neutral',
		helper: 'Ctrl+Enter o Meta+Enter envía el draft local.',
		send: 'Enviar',
		attach: 'Adjuntar evidence',
		typingLabel: 'Context assistant está escribiendo',
		quoteAuthor: 'Source excerpt',
		quoteBody: 'Evidence permanece visible antes de promotion.',
		toolbarLabel: 'Composer tools',
		deliveryDescription: 'Provider execution queda representado por parent state.',
		readLabel: 'Leído por reviewers',
		reactions: [
			{ emoji: '+', count: 4, label: 'Cuatro positive reactions' },
			{ emoji: '!', count: 1, label: 'Una reaction requiere atención' }
		],
		actions: [
			{ id: 'bold', label: 'Bold', icon: 'tabler:bold' },
			{ id: 'quote', label: 'Quote', icon: 'tabler:quote' },
			{ id: 'risk', label: 'Risk', icon: 'tabler:alert-triangle', tone: 'warning' as const }
		],
		messages: [
			{ id: 'msg-1', author: 'Alex', timestamp: '09:12', meta: 'source-linked', direction: 'inbound' as const, body: 'Podemos preservar original evidence antes de convertir esto en task?' },
			{ id: 'msg-2', author: 'Макошь', timestamp: '09:13', meta: 'review candidate', direction: 'outbound' as const, body: 'Sí. Candidate queda separado hasta owner review.' },
			{ id: 'msg-3', author: 'System', timestamp: '09:14', meta: 'local state', direction: 'system' as const, body: 'Delivery state en shared UI es solo visual.' }
		],
		attachments: [
			{ name: 'evidence.html', meta: 'text/html', icon: 'tabler:html', tone: 'warning' as const },
			{ name: 'context.md', meta: '9 KB', icon: 'tabler:markdown', tone: 'accent' as const }
		],
		receipts: [
			{ id: 'rr-1', label: 'Owner', initials: 'OW' },
			{ id: 'rr-2', label: 'Reviewer', initials: 'RV' },
			{ id: 'rr-3', label: 'Archive', initials: 'AR' },
			{ id: 'rr-4', label: 'Memory', initials: 'ME' }
		]
	},
	utility: {
		title: 'Primitivos utility',
		description: 'Pequeños UI helpers para copy actions, switchers, keyboard hints y semantic icons.',
		copyTitle: 'Copy action',
		copyValue: 'makosh://local/context-pack',
		copyLabel: 'Copiar reference',
		copiedLabel: 'Copiado',
		errorLabel: 'Copy no disponible',
		themeTitle: 'Theme selector',
		localeTitle: 'Locale selector',
		shortcutsTitle: 'Keyboard hints',
		iconsTitle: 'Semantic icons',
		openCommand: 'Abrir command',
		sendDraft: 'Enviar draft',
		providerTitle: 'Providers',
		statusTitle: 'Statuses',
		entityTitle: 'Entities',
		fileTitle: 'Files',
		locales: [
			{ value: 'ru', label: 'RU', description: 'Русский' },
			{ value: 'en', label: 'EN', description: 'English' },
			{ value: 'es', label: 'ES', description: 'Español' }
		],
		providers: ['mail', 'telegram', 'whatsapp', 'calendar', 'documents', 'generic'] as const,
		statuses: ['idle', 'active', 'success', 'warning', 'danger', 'offline', 'syncing'] as const,
		entities: ['person', 'organization', 'project', 'task', 'document', 'decision', 'obligation', 'knowledge', 'event', 'generic'] as const,
		files: [
			{ label: 'Image', mimeType: 'image/png' },
			{ label: 'PDF', mimeType: 'application/pdf' },
			{ label: 'Code', mimeType: 'application/typescript' },
			{ label: 'Archive', mimeType: 'application/zip' }
		]
	},
	editor: {
		title: 'Editor de contexto enriquecido',
		description: 'Superficie rich text compacta para notas, borradores de correo y fragmentos de evidence. Sigue siendo semantica en vez de convertirse en una cinta tipo Word.',
		label: 'Editor de contexto',
		helper: 'Usa acciones semanticas compactas: titulos, citas, listas, marks, code, divider y enlaces de evidence.',
		placeholder: 'Captura la decision, cita evidence y deja visible la fuente.',
		toolbarLabel: 'Herramientas rich text',
		outputLabel: 'Salida sanitizada',
		previewTitle: 'Vista previa HTML sanitizada',
		previewEmpty: 'Sin salida HTML todavia',
		keyboardLabel: 'Envio rapido',
		actions: [
			{ id: 'paragraph', label: 'Parrafo', icon: 'tabler:pilcrow', group: 'structure' },
			{ id: 'heading', label: 'Titulo', icon: 'tabler:h-2', group: 'structure' },
			{ id: 'subheading', label: 'Subtitulo', icon: 'tabler:h-3', group: 'structure' },
			{ id: 'quote', label: 'Cita', icon: 'tabler:quote', group: 'structure' },
			{ id: 'bulletList', label: 'Lista con vinietas', icon: 'tabler:list', group: 'lists' },
			{ id: 'orderedList', label: 'Lista numerada', icon: 'tabler:list-numbers', group: 'lists' },
			{ id: 'bold', label: 'Enfasis', icon: 'tabler:bold', group: 'marks' },
			{ id: 'italic', label: 'Matiz', icon: 'tabler:italic', group: 'marks' },
			{ id: 'underline', label: 'Subrayar', icon: 'tabler:underline', group: 'marks' },
			{ id: 'strike', label: 'Tachar', icon: 'tabler:strikethrough', group: 'marks' },
			{ id: 'code', label: 'Inline code', icon: 'tabler:code', group: 'marks' },
			{ id: 'link', label: 'Enlace evidence', icon: 'tabler:link', group: 'insert' },
			{ id: 'codeBlock', label: 'Code block', icon: 'tabler:code-dots', group: 'insert' },
			{ id: 'horizontalRule', label: 'Divider', icon: 'tabler:separator-horizontal', group: 'insert' },
			{ id: 'clearFormatting', label: 'Limpiar formato', icon: 'tabler:eraser', group: 'cleanup' }
		],
		initialHtml: '<h2>Contexto de decision</h2><p>Conserva el <code>provider_message</code> como evidence antes de promotion.</p><ol><li><p>Preservar fuente.</p></li><li><p>Revisar antes de durable truth.</p></li></ol><blockquote><p>La fuente sigue disponible para review.</p></blockquote>'
	},
	form: {
		title: 'Controles de formulario',
		search: 'Buscar',
		searchValue: 'runtime telegram',
		searchPlaceholder: 'Buscar en todo',
		email: 'Email del dueño',
		emailValue: 'owner@example.local',
			password: 'Contraseña vault',
			passwordValue: 'correct horse battery staple',
		count: 'Límite de revisión',
		countValue: 7,
		otp: 'Código de verificación',
		otpValue: '482913',
		otpHint: 'Pega o escribe un código local corto.',
		domain: 'Dominio',
		multiSelect: 'Dominios relacionados',
		combobox: 'Valor de combobox',
		autocomplete: 'Valor de autocomplete',
		noResults: 'Sin opciones locales',
		color: 'Color de acento',
		date: 'Fecha de revisión',
		time: 'Hora de revisión',
		dateTime: 'Fecha y hora de revisión',
		file: 'Selector de archivo',
		noFiles: 'No hay archivos seleccionados',
		dropZone: 'Soltar adjuntos',
		dropZoneHint: 'Con teclado usa Enter o Space',
		contextNote: 'Nota de contexto',
		noteValue: 'Los detalles del proveedor quedan fuera de la UI de comunicaciones.',
		hint: 'Los componentes UI solo exponen estado; la validación pertenece al formulario dueño.',
		error: 'Este error de ejemplo lo renderiza FormError.',
		counterMax: 96,
		checkbox: 'Requerir revisión del dueño',
		radioTitle: 'Confianza de señal',
		radioOptions: [
			{ value: 'low', label: 'Baja' },
			{ value: 'medium', label: 'Media' },
			{ value: 'high', label: 'Alta' }
		],
		slider: 'Umbral de confianza',
		range: 'Rango de puntuación de evidencia',
		realtime: 'Contexto en tiempo real activado',
		saveContract: 'Guardar contrato del componente',
		options: [
			{ value: 'communications', label: 'Comunicaciones' },
			{ value: 'radar', label: 'Radar' },
			{ value: 'knowledge', label: 'Conocimiento' }
		]
	},
	selection: {
		title: 'Controles de selección',
		select: 'Selección',
		searchableSelect: 'Selección con búsqueda',
		multiSelect: 'Selección múltiple',
		searchableMultiSelect: 'Selección múltiple con búsqueda',
		groupedSelect: 'Selección agrupada',
		treeSelect: 'Selección jerárquica',
		cascader: 'Selección en cascada',
		asyncSelect: 'Selección asíncrona',
		placeholder: 'Elige un contexto',
		searchPlaceholder: 'Buscar opciones locales',
		empty: 'No hay opciones coincidentes',
		clear: 'Limpiar selección',
		searchLabel: 'Buscar opciones',
		optionsLabel: 'Opciones disponibles',
		actionsLabel: 'Acciones de selección',
		selectAll: 'Seleccionar todo',
		clearAll: 'Limpiar todo',
		selectedCount: (count: number) => `${count} seleccionadas`,
		remove: (label: string) => `Quitar ${label}`,
		retry: 'Reintentar',
		loading: 'Cargando opciones',
		error: 'No se pudieron cargar las opciones',
		options: [
			{ value: 'communications', label: 'Comunicaciones', description: 'Mensajes canónicos y evidencia fuente', icon: 'tabler:messages' },
			{ value: 'knowledge', label: 'Conocimiento', description: 'Hechos y observaciones revisados', icon: 'tabler:bulb' },
			{ value: 'projects', label: 'Proyectos', description: 'Contexto de trabajo acotado', icon: 'tabler:briefcase' },
			{ value: 'documents', label: 'Documentos', description: 'Artefactos de evidencia versionados', icon: 'tabler:file-text' }
		],
		groups: [
			{
				id: 'memory',
				label: 'Memoria',
				options: [
					{ value: 'communications', label: 'Comunicaciones' },
					{ value: 'knowledge', label: 'Conocimiento' }
				]
			},
			{
				id: 'work',
				label: 'Trabajo',
				options: [
					{ value: 'projects', label: 'Proyectos' },
					{ value: 'documents', label: 'Documentos' }
				]
			}
		],
		tree: [
			{
				value: 'memory',
				label: 'Memoria',
				children: [
					{ value: 'communications', label: 'Comunicaciones' },
					{ value: 'knowledge', label: 'Conocimiento' }
				]
			},
			{
				value: 'work',
				label: 'Trabajo',
				children: [
					{ value: 'projects', label: 'Proyectos' },
					{ value: 'documents', label: 'Documentos' }
				]
			}
		]
	},
	foundation: {
		iconTitle: 'Sistema de iconos',
		iconDescription: 'Los iconos pasan por el wrapper Макошь Icon, no por imports vendor en dominios.',
		sharedPrimitive: 'Primitivo compartido',
		separatorTitle: 'Separadores',
		separatorDescription: 'Los divisores usan bordes tokenizados y contratos estables de orientación.',
		scrollTitle: 'Área de desplazamiento',
		scrollDescription: 'Las listas locales largas mantienen el scroll dentro del primitivo.',
		toastTitle: 'Viewport de toast',
		toastDescription: 'Las superficies de feedback son tokenizadas y deterministas para baselines visuales.',
		toasts: [
			{ id: 'visual-success', title: 'Evidencia guardada', description: 'La actualización de contexto está lista para revisión.', variant: 'success' },
			{ id: 'visual-warning', title: 'Requiere revisión', description: 'La señal del proveedor aún no es verdad durable.', variant: 'warning' }
		],
		timelineItems: [
			'Señal del proveedor observada',
			'Evidencia canónica vinculada',
			'Candidato de revisión creado',
			'Context pack actualizado',
			'Decisión del dueño pendiente'
		],
		separatorItems: ['Bandeja', 'Revisión', 'Memoria']
	},
	overlay: {
		title: 'Primitivos overlay',
		description: 'Dropdown, dialog, sheet, tooltip y popover usan comportamiento de Reka UI. Макошь posee el estilo.',
		menu: 'Menú Макошь',
		navigation: 'Navegación',
		communications: 'Comunicaciones',
		radar: 'Radar',
		settings: 'Ajustes',
		openDialog: 'Abrir diálogo',
		openSheet: 'Abrir panel',
		context: 'Contexto',
		popoverTitle: 'Popover contextual',
		popoverDescription: 'Las superficies contextuales pequeñas son ligeras y desaparecen rápido.',
		tooltipButton: 'Tooltip',
		tooltipContent: 'Sin impuesto de sidebar, solo superficies temporales.',
		dialogTitle: 'Crear elemento de Radar',
		dialogDescription: 'Captura una señal antes de promoverla a objeto de dominio.',
		dialogBody: 'El contenido del diálogo aísla estado de UI. Las decisiones de negocio quedan en TanStack queries y stores.',
		sheetTitle: 'Superficie de inspector',
		sheetDescription: 'Usar para contexto temporal, no columnas permanentes.',
		sheetBody: 'Sheet existe para trabajo temporal, ajustes y paneles de revisión.',
		modalSurfacesTitle: 'Superficies modales',
		infrastructureTitle: 'Infraestructura overlay',
		openAlertDialog: 'Abrir alert dialog',
		alertDialogTitle: '¿Descartar candidato?',
		alertDialogDescription: 'Esto quita un candidato local de la cola de revisión. La source evidence no cambia.',
		alertDialogBody: 'Usa AlertDialog solo cuando el dueño debe confirmar una acción UI riesgosa.',
		alertDialogAction: 'Descartar candidato',
		alertDialogCancel: 'Mantener candidato',
		openDrawer: 'Abrir drawer',
		drawerTitle: 'Contexto drawer',
		drawerDescription: 'Contexto temporal denso sin convertirse en panel lateral permanente.',
		drawerBody: 'Drawer es modal y gestiona foco, pero sigue siendo UI-only.',
		hoverCardButton: 'Hover preview',
		hoverCardTitle: 'Preview de evidencia',
		hoverCardDescription: 'HoverCard ofrece contexto opcional. Las decisiones requeridas deben estar visibles en otro lugar.',
		overlayHostTitle: 'Overlay host',
		overlayHostDescription: 'Capa host pasiva para superficies custom no modales.',
		portalTitle: 'Superficie Portal',
		portalDescription: 'Portal mantiene la colocación de overlay dentro de Макошь UI Kit.',
		focusTrapTitle: 'Focus trap',
		focusTrapDescription: 'Tab y Shift+Tab se quedan dentro de esta región demo.'
	},
	feedback: {
		title: 'Primitivos de feedback',
		description: 'Componentes de estado, carga y notificación para superficies UI-only.',
		surfacesTitle: 'Superficies de feedback',
		notificationTitle: 'Evidencia guardada',
		notificationDescription: 'La actualización de contexto queda en cola para revisión del dueño.',
		bannerTitle: 'Modo de revisión activo',
		bannerDescription: 'Las señales de proveedores siguen siendo candidatas hasta su promoción.',
		alertTitle: 'La evidencia está incompleta',
		alertDescription: 'Vincula evidencia canónica antes de tomar una decisión durable.',
		inlineSuccess: 'La validación local pasó.',
		inlineWarning: 'Las etiquetas largas del proveedor se ajustan sin redimensionar la superficie.',
		loadingTitle: 'Estados de carga',
		loadingDescription: 'Spinners e indicadores de progreso deterministas para vistas desktop densas.',
		progressLabel: 'Construcción de context pack',
		circularLabel: 'Progreso de revisión',
		overlayLabel: 'Actualizando evidencia',
		overlayDescription: 'Este overlay pertenece al panel local, no al shell de la aplicación.',
		statusTitle: 'Indicadores de estado',
		presenceTitle: 'Indicadores de presencia',
		statuses: [
			{ tone: 'success', label: 'Sincronizado', pulse: false },
			{ tone: 'warning', label: 'Requiere revisión', pulse: true },
			{ tone: 'danger', label: 'Bloqueado', pulse: false },
			{ tone: 'info', label: 'Observado', pulse: false }
		],
		presences: [
			{ status: 'online', label: 'Dueño online' },
			{ status: 'away', label: 'Modo enfoque' },
			{ status: 'busy', label: 'No molestar' },
			{ status: 'offline', label: 'Offline' }
		],
		toasts: [
			{ id: 'feedback-toast-success', title: 'Contexto listo', description: 'El snapshot de revisión está disponible.', variant: 'success' },
			{ id: 'feedback-toast-warning', title: 'Requiere evidencia', description: 'Todavía falta una fuente.', variant: 'warning' }
		],
		action: 'Abrir revisión'
	},
	themes: {
		options: [
			{ value: 'base-light', label: 'Base clara', description: 'Tema claro neutral principal.' },
			{ value: 'base-dark', label: 'Base oscura', description: 'Tema oscuro neutral con los mismos contratos de componentes.' },
			{ value: 'makosh-light', label: 'Макошь claro', description: 'Tema Макошь claro con acentos esmeralda del sistema.' },
			{ value: 'makosh-dark', label: 'Макошь oscuro', description: 'Tema Макошь oscuro con acento esmeralda sobre los mismos tokens.' }
		],
		swatches: ['Fondo', 'Superficie', 'Elevada', 'Texto', 'Muted', 'Acento', 'Peligro', 'Borde'],
		cardTitle: 'Superficie Макошь',
		cardDescription: 'Mismo componente, distinto conjunto de tokens.',
		searchValue: 'Buscar Telegram, Radar, Knowledge',
		contextBadge: 'Contexto'
	}
}

const translations: Record<StorybookLocale, StorybookText> = { ru, en, es }

export function resolveStorybookLocale(value: unknown): StorybookLocale {
	return storybookLocales.includes(value as StorybookLocale) ? value as StorybookLocale : 'ru'
}

export function storybookLocaleFromGlobals(globals: Record<string, unknown>): StorybookLocale {
	return resolveStorybookLocale(globals.locale)
}

export function storybookText(locale: StorybookLocale): StorybookText {
	return translations[locale]
}
