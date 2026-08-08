import type { DomainScaffoldModel } from './domainScaffoldStory'

export type DomainScaffoldKey =
  | 'agents'
  | 'calendar'
  | 'documents'
  | 'eventTraces'
  | 'home'
  | 'knowledge'
  | 'notes'
  | 'organizations'
  | 'personas'
  | 'projects'
  | 'review'
  | 'tasks'
  | 'timeline'

export const domainScaffoldModels = {
  home: {
    title: 'Home',
    subtitle: 'Ежедневная сводка, сигналы и состояние памяти.',
    icon: 'tabler:home',
    actionLabel: 'Обновить сводку',
    searchPlaceholder: 'Поиск по сигналам, людям и решениям',
    navItems: [
      { label: 'Сегодня', count: 8, selected: true },
      { label: 'Ожидает', count: 3 },
      { label: 'Память', count: 12 }
    ],
    records: [
      {
        title: 'Утренний обзор',
        summary: 'Письма, встречи, задачи и открытые решения за сегодня.',
        meta: '09:00',
        icon: 'tabler:sun',
        selected: true
      },
      {
        title: 'Риск по retention clause',
        summary: 'Legal review нужен до отправки финального пакета.',
        meta: 'high',
        icon: 'tabler:alert-triangle'
      },
      {
        title: 'Контекст по Northwind',
        summary: 'Новые письма связаны с текущим security review.',
        meta: '3',
        icon: 'tabler:building'
      }
    ],
    preview: {
      title: 'Утренний обзор владельца',
      meta: 'Собрано из коммуникаций, календаря, задач и памяти',
      icon: 'tabler:home-stats',
      chips: ['8 сигналов', '3 требуют решения', '2 встречи'],
      body: [
        'Макошь собирает только те элементы, которые помогают принять решение или сохранить контекст. В этом каркасе центральная область остается спокойной и сканируемой.',
        'Следующий шаг для Home - выбрать, какие блоки становятся постоянными виджетами, а какие остаются динамическими сигналами из Review и Timeline.'
      ]
    },
    inspectorTitle: 'Контекст дня',
    inspectorSummary: 'Связи, которые объясняют почему сигналы поднялись наверх.',
    inspectorSections: [
      {
        title: 'Связи',
        items: [
          { label: 'Northwind review', value: 'Письмо, задача и документ связаны одним evidence path.', icon: 'tabler:git-branch' },
          { label: 'Legal owner', value: 'Ожидается подтверждение по формулировке retention.', icon: 'tabler:user-check' }
        ]
      }
    ]
  },
  personas: {
    title: 'Personas',
    subtitle: 'Люди, отношения, роли и memory context.',
    icon: 'tabler:user',
    actionLabel: 'Создать персону',
    searchPlaceholder: 'Поиск людей, ролей и связей',
    navItems: [
      { label: 'Все', count: 128, selected: true },
      { label: 'Активные', count: 18 },
      { label: 'Кандидаты', count: 9 }
    ],
    records: [
      {
        title: 'Maya Chen',
        summary: 'Security vendor persona, часто участвует в review.',
        meta: 'today',
        icon: 'tabler:user-circle',
        selected: true
      },
      {
        title: 'Alex Johnson',
        summary: 'Business development, связан с partnership context.',
        meta: '2d',
        icon: 'tabler:user-circle'
      },
      {
        title: 'Legal owner',
        summary: 'Нужна нормализация из communications evidence.',
        meta: 'candidate',
        icon: 'tabler:user-question'
      }
    ],
    preview: {
      title: 'Maya Chen',
      meta: 'Person memory, evidence-backed relationship and active contexts',
      icon: 'tabler:user-circle',
      chips: ['vendor', 'security', 'recent communication'],
      body: [
        'Карточка персоны должна объяснять отношение к владельцу: кто это, откуда известно, какие текущие контексты активны.',
        'Raw address book entries и AI extraction остаются кандидатами, пока Макошь не подтверждает durable Persona через evidence.'
      ]
    },
    inspectorTitle: 'Память о персоне',
    inspectorSummary: 'Сводка по отношениям, коммуникациям и открытым обязательствам.',
    inspectorSections: [
      {
        title: 'Активные связи',
        items: [
          { label: 'Northwind Security', value: 'Vendor persona in current security review.', icon: 'tabler:building' },
          { label: 'Retention decision', value: 'Участвует как источник evidence.', icon: 'tabler:shield-check' }
        ]
      }
    ]
  },
  organizations: {
    title: 'Organizations',
    subtitle: 'Компании, команды, vendors и organizational memory.',
    icon: 'tabler:building',
    actionLabel: 'Создать организацию',
    searchPlaceholder: 'Поиск организаций, доменов и отношений',
    navItems: [
      { label: 'Все', count: 46, selected: true },
      { label: 'Vendors', count: 12 },
      { label: 'Кандидаты', count: 5 }
    ],
    records: [
      {
        title: 'Northwind Security',
        summary: 'Vendor, текущий security review и redline evidence.',
        meta: 'active',
        icon: 'tabler:building-bank',
        selected: true
      },
      {
        title: 'ACME Inc.',
        summary: 'Partnership proposal and meeting context.',
        meta: 'warm',
        icon: 'tabler:building'
      },
      {
        title: 'Legal team',
        summary: 'Внутренняя группа, пока как candidate organization.',
        meta: 'candidate',
        icon: 'tabler:users-group'
      }
    ],
    preview: {
      title: 'Northwind Security',
      meta: 'Organization memory with personas, documents and decisions',
      icon: 'tabler:building-bank',
      chips: ['vendor', 'security review', 'open decision'],
      body: [
        'Organization workspace должен показывать не CRM-профиль, а контекст: почему организация важна сейчас, какие люди, документы и решения к ней привязаны.',
        'Новые организации появляются как кандидаты из evidence, а не как автоматическая durable truth из одного письма.'
      ]
    },
    inspectorTitle: 'Контекст организации',
    inspectorSummary: 'Связанные люди, проекты, evidence и риски.',
    inspectorSections: [
      {
        title: 'Связанные объекты',
        items: [
          { label: 'Maya Chen', value: 'Primary communication persona.', icon: 'tabler:user' },
          { label: 'Security answers', value: 'Document evidence linked to current review.', icon: 'tabler:file-text' }
        ]
      }
    ]
  },
  projects: {
    title: 'Projects',
    subtitle: 'Рабочие контексты, инициативы и decision history.',
    icon: 'tabler:briefcase',
    actionLabel: 'Создать проект',
    searchPlaceholder: 'Поиск проектов, решений и evidence',
    navItems: [
      { label: 'Активные', count: 7, selected: true },
      { label: 'Ожидают', count: 4 },
      { label: 'Архив', count: 18 }
    ],
    records: [
      {
        title: 'Security vendor review',
        summary: 'Открыта формулировка retention clause.',
        meta: 'risk',
        icon: 'tabler:shield',
        selected: true
      },
      {
        title: 'Partnership proposal',
        summary: 'Proposal deck and pricing overview collected.',
        meta: 'draft',
        icon: 'tabler:briefcase'
      },
      {
        title: 'Radar release',
        summary: 'Signals promoted from research workspace.',
        meta: '2d',
        icon: 'tabler:rocket'
      }
    ],
    preview: {
      title: 'Security vendor review',
      meta: 'Project context, linked communications and decisions',
      icon: 'tabler:shield',
      chips: ['open decision', '2 documents', 'legal owner'],
      body: [
        'Project workspace собирает только устойчивый рабочий контекст: цели, открытые решения, evidence, связанных людей и обязательства.',
        'Сигналы из коммуникаций не становятся проектом автоматически: Review/Radar должен подтвердить promotion.'
      ]
    },
    inspectorTitle: 'Интеллект проекта',
    inspectorSummary: 'Риски, решения и next actions вокруг проекта.',
    inspectorSections: [
      {
        title: 'Открыто',
        items: [
          { label: 'Retention clause', value: 'Owner approval required before final send.', icon: 'tabler:git-pull-request' },
          { label: 'Evidence gap', value: 'Attachment redline is present, legal note pending.', icon: 'tabler:file-alert' }
        ]
      }
    ]
  },
  tasks: {
    title: 'Tasks',
    subtitle: 'Действия, обязательства и review-backed commitments.',
    icon: 'tabler:checkbox',
    actionLabel: 'Создать задачу',
    searchPlaceholder: 'Поиск задач, owners и обязательств',
    navItems: [
      { label: 'Мои', count: 11, selected: true },
      { label: 'Ожидают', count: 6 },
      { label: 'Кандидаты', count: 8 }
    ],
    records: [
      {
        title: 'Approve retention wording',
        summary: 'Owner должен подтвердить финальную формулировку.',
        meta: 'today',
        icon: 'tabler:checkbox',
        selected: true
      },
      {
        title: 'Create follow-up summary',
        summary: 'Сводка по partnership call для Alex Johnson.',
        meta: 'draft',
        icon: 'tabler:notes'
      },
      {
        title: 'Resolve date conflict',
        summary: 'Telegram thread содержит Tuesday/Friday conflict.',
        meta: 'review',
        icon: 'tabler:calendar-question'
      }
    ],
    preview: {
      title: 'Approve retention wording',
      meta: 'Task candidate promoted from communication evidence',
      icon: 'tabler:checkbox',
      chips: ['needs owner', 'mail evidence', 'legal review'],
      body: [
        'Tasks должны отличаться от сырых “to-do” тем, что у каждой задачи есть источник, owner context и причина появления.',
        'Кандидаты задач живут в Review до подтверждения, чтобы Макошь не плодил лишние durable commitments.'
      ]
    },
    inspectorTitle: 'Контекст задачи',
    inspectorSummary: 'Evidence, dependency и возможные автоматизации.',
    inspectorSections: [
      {
        title: 'Evidence',
        items: [
          { label: 'Mail thread', value: 'Vendor security review contains the request.', icon: 'tabler:mail' },
          { label: 'Document redline', value: 'Retention clause attachment is linked.', icon: 'tabler:file-certificate' }
        ]
      }
    ]
  },
  calendar: {
    title: 'Calendar',
    subtitle: 'Встречи, availability, deadlines и календарный контекст.',
    icon: 'tabler:calendar',
    actionLabel: 'Создать событие',
    searchPlaceholder: 'Поиск встреч, сроков и участников',
    navItems: [
      { label: 'Сегодня', count: 5, selected: true },
      { label: 'Неделя', count: 18 },
      { label: 'Кандидаты', count: 4 }
    ],
    records: [
      {
        title: 'Security review sync',
        summary: 'Legal, owner, vendor. Нужно решение по retention.',
        meta: '12:20',
        icon: 'tabler:calendar-event',
        selected: true
      },
      {
        title: 'Partnership call',
        summary: 'Proposal discussion with ACME.',
        meta: '15:00',
        icon: 'tabler:video'
      },
      {
        title: 'Date conflict candidate',
        summary: 'Friday likely stronger than Tuesday.',
        meta: 'review',
        icon: 'tabler:calendar-question'
      }
    ],
    preview: {
      title: 'Security review sync',
      meta: 'Meeting context with participants, source messages and prep notes',
      icon: 'tabler:calendar-event',
      chips: ['Zoom', '4 participants', 'prep needed'],
      body: [
        'Calendar workspace должен не просто показывать события, а объяснять зачем встреча существует и какой контекст нужен владельцу.',
        'Календарные кандидаты из коммуникаций остаются candidates, пока владелец или workflow не подтвердит создание события.'
      ]
    },
    inspectorTitle: 'Подготовка встречи',
    inspectorSummary: 'Макошь собирает контекст, evidence и открытые решения до звонка.',
    inspectorSections: [
      {
        title: 'Prep',
        items: [
          { label: 'Open decision', value: 'Retention clause approval.', icon: 'tabler:git-pull-request' },
          { label: 'Source documents', value: 'Security answers and redline.', icon: 'tabler:files' }
        ]
      }
    ]
  },
  documents: {
    title: 'Documents',
    subtitle: 'Документы, вложения, evidence и reviewed artifacts.',
    icon: 'tabler:file-text',
    actionLabel: 'Импортировать документ',
    searchPlaceholder: 'Поиск документов, вложений и evidence',
    navItems: [
      { label: 'Evidence', count: 18, selected: true },
      { label: 'Reviewed', count: 9 },
      { label: 'Candidates', count: 6 }
    ],
    records: [
      {
        title: 'retention-clause-redline.docx',
        summary: 'Attachment evidence from vendor security review.',
        meta: '88 KB',
        icon: 'tabler:file-certificate',
        selected: true
      },
      {
        title: 'security-answers.pdf',
        summary: 'Answers attached to mail thread.',
        meta: '420 KB',
        icon: 'tabler:file-type-pdf'
      },
      {
        title: 'proposal-deck.pdf',
        summary: 'Partnership proposal linked to ACME.',
        meta: '2.4 MB',
        icon: 'tabler:presentation'
      }
    ],
    preview: {
      title: 'retention-clause-redline.docx',
      meta: 'Document evidence, source communication and review status',
      icon: 'tabler:file-certificate',
      chips: ['source attachment', 'legal review', 'candidate document'],
      body: [
        'Documents surface отделяет imported artifacts от reviewed documents. Вложения не становятся durable документами без provenance и review path.',
        'Основной экран должен показывать preview, source trail, extracted entities и suggested promotions.'
      ]
    },
    inspectorTitle: 'Интеллект документа',
    inspectorSummary: 'Извлечения, риски, связанные решения и evidence path.',
    inspectorSections: [
      {
        title: 'Extracted',
        items: [
          { label: 'Retention period', value: '30 days proposed after legal approval.', icon: 'tabler:quote' },
          { label: 'Linked decision', value: 'Retention clause approval.', icon: 'tabler:git-pull-request' }
        ]
      }
    ]
  },
  notes: {
    title: 'Notes',
    subtitle: 'Заметки владельца, working memory и curated context.',
    icon: 'tabler:notebook',
    actionLabel: 'Новая заметка',
    searchPlaceholder: 'Поиск заметок, тем и связей',
    navItems: [
      { label: 'Недавние', count: 14, selected: true },
      { label: 'Связанные', count: 8 },
      { label: 'Черновики', count: 3 }
    ],
    records: [
      {
        title: 'Vendor review notes',
        summary: 'Краткие мысли по security vendor process.',
        meta: 'today',
        icon: 'tabler:note',
        selected: true
      },
      {
        title: 'Radar launch memory',
        summary: 'Что важно не забыть перед release.',
        meta: '1d',
        icon: 'tabler:note'
      },
      {
        title: 'ACME partnership thoughts',
        summary: 'Промежуточная оценка partnership proposal.',
        meta: 'draft',
        icon: 'tabler:pencil'
      }
    ],
    preview: {
      title: 'Vendor review notes',
      meta: 'Owner-authored note linked to communications and documents',
      icon: 'tabler:notebook',
      chips: ['private note', 'linked evidence', 'project context'],
      body: [
        'Notes не должны превращаться в отдельный хаотичный notebook. Они работают как curated memory рядом с evidence-backed объектами.',
        'Связи с людьми, проектами и документами важнее богатого форматирования в первой версии.'
      ]
    },
    inspectorTitle: 'Связанный контекст',
    inspectorSummary: 'Макошь показывает, где заметка участвует в памяти.',
    inspectorSections: [
      {
        title: 'Links',
        items: [
          { label: 'Security vendor review', value: 'Project context.', icon: 'tabler:briefcase' },
          { label: 'Maya Chen', value: 'Person context.', icon: 'tabler:user' }
        ]
      }
    ]
  },
  knowledge: {
    title: 'Knowledge Graph',
    subtitle: 'Связи между сущностями, evidence и promoted knowledge.',
    icon: 'tabler:share',
    actionLabel: 'Открыть граф',
    searchPlaceholder: 'Поиск сущностей, связей и evidence',
    navItems: [
      { label: 'Graph', count: 220, selected: true },
      { label: 'Entities', count: 84 },
      { label: 'Relations', count: 136 }
    ],
    records: [
      {
        title: 'Retention clause approval',
        summary: 'Decision node connected to mail, document and task.',
        meta: 'decision',
        icon: 'tabler:git-pull-request',
        selected: true
      },
      {
        title: 'Northwind Security',
        summary: 'Organization node with vendor relationship.',
        meta: 'org',
        icon: 'tabler:building'
      },
      {
        title: 'Maya Chen',
        summary: 'Person node linked to current review.',
        meta: 'person',
        icon: 'tabler:user'
      }
    ],
    preview: {
      title: 'Retention clause approval',
      meta: 'Graph node with source evidence and relation confidence',
      icon: 'tabler:share',
      chips: ['decision', '3 sources', 'medium confidence'],
      body: [
        'Knowledge Graph показывает не “красивую паутину”, а проверяемые связи, которые помогают Макошь понимать контекст.',
        'Каждая связь должна иметь provenance или быть помечена как candidate, иначе граф станет красивым мусором.'
      ]
    },
    inspectorTitle: 'Evidence path',
    inspectorSummary: 'Почему эта связь существует и где её можно проверить.',
    inspectorSections: [
      {
        title: 'Sources',
        items: [
          { label: 'Mail thread', value: 'Request and reply chain.', icon: 'tabler:mail' },
          { label: 'Document redline', value: 'Clause source evidence.', icon: 'tabler:file-text' }
        ]
      }
    ]
  },
  review: {
    title: 'Review',
    subtitle: 'Кандидаты, противоречия и owner-controlled promotion.',
    icon: 'tabler:clipboard-check',
    actionLabel: 'Начать ревью',
    searchPlaceholder: 'Поиск кандидатов, рисков и противоречий',
    navItems: [
      { label: 'Inbox', count: 17, selected: true },
      { label: 'Risk', count: 5 },
      { label: 'Promoted', count: 23 }
    ],
    records: [
      {
        title: 'Retention clause approval',
        summary: 'Candidate decision from mail thread and redline.',
        meta: 'risk',
        icon: 'tabler:git-pull-request',
        selected: true
      },
      {
        title: 'Friday vs Tuesday date',
        summary: 'Contradiction from messenger thread.',
        meta: 'conflict',
        icon: 'tabler:calendar-question'
      },
      {
        title: 'Create follow-up task',
        summary: 'Suggested action after partnership call.',
        meta: 'action',
        icon: 'tabler:checkbox'
      }
    ],
    preview: {
      title: 'Retention clause approval',
      meta: 'Review item before promotion to durable decision',
      icon: 'tabler:clipboard-check',
      chips: ['candidate', 'needs owner', 'evidence-backed'],
      body: [
        'Review - это предохранитель Макошь. Сюда попадает всё, что может стать business truth, но ещё не заслужило durable state.',
        'Экран должен помогать быстро понять источник, риск, альтернативы и последствия promotion или dismiss.'
      ]
    },
    inspectorTitle: 'Review intelligence',
    inspectorSummary: 'Evidence, confidence, risks and proposed actions.',
    inspectorSections: [
      {
        title: 'Decision support',
        items: [
          { label: 'Confidence', value: 'Medium, source evidence exists.', icon: 'tabler:gauge' },
          { label: 'Risk', value: 'Legal wording can affect audit retention.', icon: 'tabler:alert-triangle' }
        ]
      }
    ]
  },
  timeline: {
    title: 'Timeline',
    subtitle: 'События, observations и reconstructed history.',
    icon: 'tabler:timeline',
    actionLabel: 'Добавить событие',
    searchPlaceholder: 'Поиск событий, источников и causation',
    navItems: [
      { label: 'Сегодня', count: 31, selected: true },
      { label: 'Signals', count: 12 },
      { label: 'Promotions', count: 5 }
    ],
    records: [
      {
        title: 'Mail recorded',
        summary: 'Vendor security review message accepted.',
        meta: '09:42',
        icon: 'tabler:mail',
        selected: true
      },
      {
        title: 'Task candidate created',
        summary: 'Макошь detected owner approval requirement.',
        meta: '09:43',
        icon: 'tabler:checkbox'
      },
      {
        title: 'Document evidence linked',
        summary: 'Redline attachment connected to decision candidate.',
        meta: '09:44',
        icon: 'tabler:file-certificate'
      }
    ],
    preview: {
      title: 'Mail recorded',
      meta: 'Timeline event with source, causation and derived projections',
      icon: 'tabler:timeline',
      chips: ['source event', 'communication', 'rebuildable projection'],
      body: [
        'Timeline показывает цепочку причин, а не просто activity feed. Это помогает понять, почему Макошь сейчас считает объект важным.',
        'Derived projections должны быть восстановимы из событий и evidence, поэтому timeline surface должен быть особенно честным к provenance.'
      ]
    },
    inspectorTitle: 'Event trace',
    inspectorSummary: 'Causation, correlation and derived consequences.',
    inspectorSections: [
      {
        title: 'Trace',
        items: [
          { label: 'Source', value: 'integration.mail.message.observed', icon: 'tabler:plug' },
          { label: 'Projection', value: 'communication.message.recorded', icon: 'tabler:database' }
        ]
      }
    ]
  },
  eventTraces: {
    title: 'Event Traces',
    subtitle: 'Debuggable traces for observations, events and workflows.',
    icon: 'tabler:route',
    actionLabel: 'Открыть trace',
    searchPlaceholder: 'Поиск по causation, correlation и source',
    navItems: [
      { label: 'Live', count: 24, selected: true },
      { label: 'Failed', count: 2 },
      { label: 'Archived', count: 91 }
    ],
    records: [
      {
        title: 'provider-command-requested',
        summary: 'Outbound mail command queued for integration execution.',
        meta: 'live',
        icon: 'tabler:send',
        selected: true
      },
      {
        title: 'candidate-promoted',
        summary: 'Review item promoted to task command.',
        meta: 'ok',
        icon: 'tabler:arrow-up-right'
      },
      {
        title: 'projection-rebuild',
        summary: 'Search projection refreshed after document import.',
        meta: '1m',
        icon: 'tabler:refresh'
      }
    ],
    preview: {
      title: 'provider-command-requested',
      meta: 'Operational trace across app, workflow and integration boundaries',
      icon: 'tabler:route',
      chips: ['correlation', 'causation', 'provider boundary'],
      body: [
        'Event Traces - это инженерный и owner-visible слой объяснимости. Он нужен, чтобы понимать, что произошло, где остановилось и можно ли повторить.',
        'UI должен показывать flow без утечки приватного содержимого сообщений или секретов.'
      ]
    },
    inspectorTitle: 'Trace inspector',
    inspectorSummary: 'Boundary, retries, sanitized evidence and lifecycle.',
    inspectorSections: [
      {
        title: 'Runtime',
        items: [
          { label: 'Boundary', value: 'communication -> integration command.', icon: 'tabler:arrows-split' },
          { label: 'Retry', value: 'Idempotent command, no remote mutation yet.', icon: 'tabler:repeat' }
        ]
      }
    ]
  },
  agents: {
    title: 'AI Agents',
    subtitle: 'Local AI runs, suggested actions and owner-controlled execution.',
    icon: 'tabler:sparkles',
    actionLabel: 'Новый запуск',
    searchPlaceholder: 'Поиск запусков, действий и кандидатов',
    navItems: [
      { label: 'Runs', count: 9, selected: true },
      { label: 'Actions', count: 6 },
      { label: 'Blocked', count: 2 }
    ],
    records: [
      {
        title: 'Draft owner reply',
        summary: 'Agent prepared bilingual reply candidate.',
        meta: 'review',
        icon: 'tabler:wand',
        selected: true
      },
      {
        title: 'Summarize call',
        summary: 'Meeting transcript summary waiting for evidence check.',
        meta: 'queued',
        icon: 'tabler:microphone'
      },
      {
        title: 'Extract obligations',
        summary: 'Document extraction created three review candidates.',
        meta: 'done',
        icon: 'tabler:list-search'
      }
    ],
    preview: {
      title: 'Draft owner reply',
      meta: 'AI run output as candidate, not durable truth',
      icon: 'tabler:sparkles',
      chips: ['local AI', 'requires review', 'source cited'],
      body: [
        'Agents workspace должен ясно отделять AI output от подтвержденного действия. Макошь может предлагать, но владелец или deterministic workflow решает.',
        'В центре важны input evidence, proposed output, confidence и кнопки review, а не “магическая” генерация без следов.'
      ]
    },
    inspectorTitle: 'Run inspector',
    inspectorSummary: 'Inputs, citations, capability limits and safety state.',
    inspectorSections: [
      {
        title: 'Controls',
        items: [
          { label: 'Source evidence', value: 'Mail thread and redline attachment cited.', icon: 'tabler:file-search' },
          { label: 'Execution', value: 'Remote send blocked until owner approval.', icon: 'tabler:lock' }
        ]
      }
    ]
  }
} satisfies Record<DomainScaffoldKey, DomainScaffoldModel>
