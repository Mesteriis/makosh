import { duplicates, list, violation } from './validation-diagnostics.mjs';

const IMPLEMENTATION_KEYS = [
  'currentSlice',
  'productionPackageMode',
  'productionPackages',
  'workspaceDependencyAllowlist',
  'thirdPartyDependencyAllowlist',
  'forbiddenDependencies',
  'forbiddenDependencyPrefixes',
  'cargoFeaturesEnabled',
  'cargoFeatureAllowlist',
  'targetPolicy',
  'developmentProfile',
  'ownerInventory',
  'kernelProfile',
  'exitGates',
];

const RECOVERY_PRODUCTION_PACKAGES = [
  { name: 'makosh-events-protocol', role: 'platform', owner: 'events', surface: 'contract' },
  { name: 'makosh-runtime-protocol', role: 'platform', owner: 'runtime_protocol', surface: 'contract' },
  { name: 'makosh-gateway-protocol', role: 'api', owner: 'gateway', surface: 'contract' },
  { name: 'makosh-kernel-control-store', role: 'core', owner: 'kernel', surface: 'contract' },
  { name: 'makosh-kernel-control-store-sqlite', role: 'core', owner: 'kernel', surface: 'persistence' },
  { name: 'makosh-kernel', role: 'core', owner: 'kernel', surface: 'runtime' },
];

const VAULT_FOUNDATION_PRODUCTION_PACKAGES = [
  ...RECOVERY_PRODUCTION_PACKAGES,
  { name: 'makosh-vault-protocol', role: 'platform', owner: 'vault', surface: 'contract' },
  { name: 'makosh-managed-vault-client', role: 'platform', owner: 'vault', surface: 'contract' },
  { name: 'makosh-vault-key-provider', role: 'platform', owner: 'vault', surface: 'contract' },
  { name: 'makosh-vault-key-provider-file', role: 'platform', owner: 'vault', surface: 'implementation' },
  { name: 'makosh-secure-file', role: 'platform', owner: 'secure_file', surface: 'contract' },
  { name: 'makosh-vault-store-sqlcipher', role: 'platform', owner: 'vault', surface: 'persistence' },
  { name: 'makosh-vault-runtime', role: 'platform', owner: 'vault', surface: 'runtime' },
];

const CLOCK_PRODUCTION_PACKAGES = [
  ...VAULT_FOUNDATION_PRODUCTION_PACKAGES,
  { name: 'makosh-clock-protocol', role: 'platform', owner: 'clock', surface: 'contract' },
  { name: 'makosh-clock-runtime', role: 'platform', owner: 'clock', surface: 'implementation' },
];

const TELEMETRY_FOUNDATION_PRODUCTION_PACKAGES = [
  ...CLOCK_PRODUCTION_PACKAGES,
  { name: 'makosh-telemetry-protocol', role: 'platform', owner: 'telemetry', surface: 'contract' },
  { name: 'makosh-telemetry-collector', role: 'platform', owner: 'telemetry', surface: 'runtime' },
];

const STORAGE_FOUNDATION_PRODUCTION_PACKAGES = [
  ...TELEMETRY_FOUNDATION_PRODUCTION_PACKAGES,
  { name: 'makosh-storage-protocol', role: 'platform', owner: 'storage', surface: 'contract' },
  { name: 'makosh-storage-control', role: 'platform', owner: 'storage', surface: 'implementation' },
  { name: 'makosh-storage-vault', role: 'platform', owner: 'storage', surface: 'contract' },
  { name: 'makosh-storage-runtime', role: 'platform', owner: 'storage', surface: 'runtime' },
  { name: 'makosh-storage-postgres', role: 'platform', owner: 'storage', surface: 'persistence' },
  { name: 'makosh-storage-pgbouncer', role: 'platform', owner: 'storage', surface: 'implementation' },
  { name: 'makosh-storage-migrations', role: 'platform', owner: 'storage', surface: 'implementation' },
];

const NATS_FOUNDATION_PRODUCTION_PACKAGES = [
  ...STORAGE_FOUNDATION_PRODUCTION_PACKAGES,
  { name: 'makosh-events-jetstream', role: 'platform', owner: 'events', surface: 'implementation' },
  { name: 'makosh-events-authority', role: 'platform', owner: 'events', surface: 'implementation' },
  { name: 'makosh-events-authority-runtime-control', role: 'platform', owner: 'events', surface: 'implementation' },
  { name: 'makosh-events-authority-runtime', role: 'platform', owner: 'events', surface: 'runtime' },
];

const RECOVERY_WORKSPACE_DEPENDENCY_ALLOWLIST = {
  'makosh-events-protocol': [],
  'makosh-runtime-protocol': [],
  'makosh-gateway-protocol': [
    { name: 'makosh-runtime-protocol', kind: 'normal' },
  ],
  'makosh-kernel-control-store': [],
  'makosh-kernel-control-store-sqlite': [
    { name: 'makosh-kernel-control-store', kind: 'normal' },
  ],
  'makosh-kernel': [
    { name: 'makosh-gateway-protocol', kind: 'normal' },
    { name: 'makosh-kernel-control-store', kind: 'normal' },
    { name: 'makosh-kernel-control-store-sqlite', kind: 'normal' },
    { name: 'makosh-runtime-protocol', kind: 'normal' },
    { name: 'makosh-secure-file', kind: 'normal' },
  ],
  'makosh-secure-file': [],
};

const VAULT_FOUNDATION_WORKSPACE_DEPENDENCY_ALLOWLIST = {
  ...RECOVERY_WORKSPACE_DEPENDENCY_ALLOWLIST,
  'makosh-vault-protocol': [],
  'makosh-managed-vault-client': [
    { name: 'makosh-runtime-protocol', kind: 'normal' },
    { name: 'makosh-vault-protocol', kind: 'normal' },
  ],
  'makosh-vault-key-provider': [],
  'makosh-vault-key-provider-file': [
    { name: 'makosh-vault-key-provider', kind: 'normal' },
    { name: 'makosh-secure-file', kind: 'normal' },
  ],
  'makosh-vault-store-sqlcipher': [
    { name: 'makosh-vault-key-provider', kind: 'normal' },
    { name: 'makosh-vault-protocol', kind: 'normal' },
  ],
  'makosh-vault-runtime': [
    { name: 'makosh-vault-key-provider', kind: 'normal' },
    { name: 'makosh-vault-key-provider-file', kind: 'normal' },
    { name: 'makosh-secure-file', kind: 'normal' },
    { name: 'makosh-vault-protocol', kind: 'normal' },
    { name: 'makosh-vault-store-sqlcipher', kind: 'normal' },
    { name: 'makosh-runtime-protocol', kind: 'normal' },
  ],
};

const CLOCK_WORKSPACE_DEPENDENCY_ALLOWLIST = {
  ...VAULT_FOUNDATION_WORKSPACE_DEPENDENCY_ALLOWLIST,
  'makosh-clock-protocol': [],
  'makosh-clock-runtime': [
    { name: 'makosh-clock-protocol', kind: 'normal' },
  ],
};

const TELEMETRY_FOUNDATION_WORKSPACE_DEPENDENCY_ALLOWLIST = {
  ...CLOCK_WORKSPACE_DEPENDENCY_ALLOWLIST,
  'makosh-telemetry-protocol': [],
  'makosh-telemetry-collector': [
    { name: 'makosh-telemetry-protocol', kind: 'normal' },
    { name: 'makosh-runtime-protocol', kind: 'normal' },
  ],
};

const STORAGE_FOUNDATION_WORKSPACE_DEPENDENCY_ALLOWLIST = {
  ...TELEMETRY_FOUNDATION_WORKSPACE_DEPENDENCY_ALLOWLIST,
  'makosh-kernel': [
    ...RECOVERY_WORKSPACE_DEPENDENCY_ALLOWLIST['makosh-kernel'],
    { name: 'makosh-storage-protocol', kind: 'normal' },
  ],
  'makosh-storage-protocol': [],
  'makosh-storage-control': [
    { name: 'makosh-storage-protocol', kind: 'normal' },
    { name: 'makosh-storage-vault', kind: 'normal' },
  ],
  'makosh-storage-vault': [
    { name: 'makosh-runtime-protocol', kind: 'normal' },
    { name: 'makosh-storage-protocol', kind: 'normal' },
    { name: 'makosh-vault-protocol', kind: 'normal' },
  ],
  'makosh-storage-runtime': [
    { name: 'makosh-storage-protocol', kind: 'normal' },
    { name: 'makosh-storage-control', kind: 'normal' },
    { name: 'makosh-storage-postgres', kind: 'normal' },
    { name: 'makosh-storage-pgbouncer', kind: 'normal' },
    { name: 'makosh-storage-migrations', kind: 'normal' },
    { name: 'makosh-runtime-protocol', kind: 'normal' },
    { name: 'makosh-storage-vault', kind: 'normal' },
    { name: 'makosh-vault-protocol', kind: 'normal' },
  ],
  'makosh-storage-postgres': [
    { name: 'makosh-storage-control', kind: 'normal' },
    { name: 'makosh-storage-migrations', kind: 'normal' },
    { name: 'makosh-storage-protocol', kind: 'normal' },
  ],
  'makosh-storage-pgbouncer': [
    { name: 'makosh-storage-control', kind: 'normal' },
    { name: 'makosh-storage-protocol', kind: 'normal' },
  ],
  'makosh-storage-migrations': [
    { name: 'makosh-storage-control', kind: 'normal' },
    { name: 'makosh-storage-protocol', kind: 'normal' },
  ],
};

const NATS_FOUNDATION_WORKSPACE_DEPENDENCY_ALLOWLIST = {
  ...STORAGE_FOUNDATION_WORKSPACE_DEPENDENCY_ALLOWLIST,
  'makosh-events-jetstream': [
    { name: 'makosh-events-protocol', kind: 'normal' },
    { name: 'makosh-runtime-protocol', kind: 'normal' },
    { name: 'makosh-scheduler-protocol', kind: 'normal' },
    { name: 'makosh-vault-protocol', kind: 'normal' },
    { name: 'makosh-vault-protocol', kind: 'normal' },
  ],
  'makosh-events-authority': [
    { name: 'makosh-events-jetstream', kind: 'normal' },
  ],
  'makosh-events-authority-runtime-control': [
    { name: 'makosh-events-authority', kind: 'normal' },
    { name: 'makosh-events-jetstream', kind: 'normal' },
    { name: 'makosh-runtime-protocol', kind: 'normal' },
  ],
  'makosh-events-authority-runtime': [
    { name: 'makosh-events-authority-runtime-control', kind: 'normal' },
    { name: 'makosh-runtime-protocol', kind: 'normal' },
  ],
};

const BLOB_FOUNDATION_PRODUCTION_PACKAGES = [
  ...NATS_FOUNDATION_PRODUCTION_PACKAGES,
  { name: 'makosh-blob-protocol', role: 'platform', owner: 'blob', surface: 'contract' },
];

const BLOB_RUNTIME_FOUNDATION_PRODUCTION_PACKAGES = [
  ...BLOB_FOUNDATION_PRODUCTION_PACKAGES,
  { name: 'makosh-blob-client-contract', role: 'platform', owner: 'blob', surface: 'contract' },
  { name: 'makosh-blob-client', role: 'platform', owner: 'blob', surface: 'contract' },
  { name: 'makosh-blob-runtime', role: 'platform', owner: 'blob', surface: 'implementation' },
  { name: 'makosh-blob-service', role: 'platform', owner: 'blob', surface: 'runtime' },
];

const SCHEDULER_PROTOCOL_FOUNDATION_PRODUCTION_PACKAGES = [
  ...BLOB_RUNTIME_FOUNDATION_PRODUCTION_PACKAGES,
  { name: 'makosh-scheduler-protocol', role: 'platform', owner: 'scheduler', surface: 'contract' },
];

const SCHEDULER_FOUNDATION_PRODUCTION_PACKAGES = [
  ...SCHEDULER_PROTOCOL_FOUNDATION_PRODUCTION_PACKAGES,
  { name: 'makosh-scheduler', role: 'platform', owner: 'scheduler', surface: 'implementation' },
];

const SCHEDULER_PERSISTENCE_FOUNDATION_PRODUCTION_PACKAGES = [
  ...SCHEDULER_FOUNDATION_PRODUCTION_PACKAGES,
  { name: 'makosh-scheduler-persistence', role: 'platform', owner: 'scheduler', surface: 'persistence' },
];

const GATEWAY_SESSION_FOUNDATION_PRODUCTION_PACKAGES = [
  ...SCHEDULER_PERSISTENCE_FOUNDATION_PRODUCTION_PACKAGES,
  { name: 'makosh-gateway-session-contract', role: 'api', owner: 'gateway', surface: 'contract' },
  { name: 'makosh-gateway-session', role: 'api', owner: 'gateway', surface: 'implementation' },
];

const SCHEDULER_RECEIPT_DELIVERY_FOUNDATION_PRODUCTION_PACKAGES = [
  ...GATEWAY_SESSION_FOUNDATION_PRODUCTION_PACKAGES,
];

const SCHEDULER_JETSTREAM_FOUNDATION_PRODUCTION_PACKAGES = [
  ...SCHEDULER_RECEIPT_DELIVERY_FOUNDATION_PRODUCTION_PACKAGES,
  { name: 'makosh-scheduler-jetstream', role: 'platform', owner: 'scheduler', surface: 'implementation' },
];

const SCHEDULER_RUNTIME_FOUNDATION_PRODUCTION_PACKAGES = [
  ...SCHEDULER_JETSTREAM_FOUNDATION_PRODUCTION_PACKAGES,
  { name: 'makosh-scheduler-runtime', role: 'platform', owner: 'scheduler', surface: 'runtime' },
];

const GATEWAY_RUNTIME_FOUNDATION_PRODUCTION_PACKAGES = [
  ...SCHEDULER_RUNTIME_FOUNDATION_PRODUCTION_PACKAGES,
  { name: 'makosh-gateway-runtime', role: 'api', owner: 'gateway', surface: 'implementation' },
];

const MAIL_COMMUNICATIONS_FOUNDATION_PRODUCTION_PACKAGES = [
  ...GATEWAY_RUNTIME_FOUNDATION_PRODUCTION_PACKAGES,
  { name: 'makosh-mail-api', role: 'integration', owner: 'mail', surface: 'contract' },
  { name: 'makosh-mail-core', role: 'integration', owner: 'mail', surface: 'implementation' },
  { name: 'makosh-mail-imap', role: 'integration', owner: 'mail', surface: 'implementation' },
  { name: 'makosh-mail-gmail', role: 'integration', owner: 'mail', surface: 'implementation' },
  { name: 'makosh-mail-smtp', role: 'integration', owner: 'mail', surface: 'implementation' },
  { name: 'makosh-mail-persistence', role: 'integration', owner: 'mail', surface: 'persistence' },
  { name: 'makosh-mail-runtime', role: 'integration', owner: 'mail', surface: 'runtime' },
  { name: 'makosh-mail-assembly', role: 'integration', owner: 'mail', surface: 'assembly' },
  { name: 'makosh-telegram-api', role: 'integration', owner: 'telegram', surface: 'contract' },
  { name: 'makosh-telegram-core', role: 'integration', owner: 'telegram', surface: 'implementation' },
  { name: 'makosh-telegram-tdlib', role: 'integration', owner: 'telegram', surface: 'implementation' },
  { name: 'makosh-telegram-persistence', role: 'integration', owner: 'telegram', surface: 'persistence' },
  { name: 'makosh-telegram-runtime', role: 'integration', owner: 'telegram', surface: 'runtime' },
  { name: 'makosh-telegram-assembly', role: 'integration', owner: 'telegram', surface: 'assembly' },
  { name: 'makosh-whatsapp-api', role: 'integration', owner: 'whatsapp', surface: 'contract' },
  { name: 'makosh-whatsapp-core', role: 'integration', owner: 'whatsapp', surface: 'implementation' },
  { name: 'makosh-whatsapp-persistence', role: 'integration', owner: 'whatsapp', surface: 'persistence' },
  { name: 'makosh-whatsapp-runtime', role: 'integration', owner: 'whatsapp', surface: 'runtime' },
  { name: 'makosh-whatsapp-assembly', role: 'integration', owner: 'whatsapp', surface: 'assembly' },
  { name: 'makosh-zulip-api', role: 'integration', owner: 'zulip', surface: 'contract' },
  { name: 'makosh-zulip-core', role: 'integration', owner: 'zulip', surface: 'implementation' },
  { name: 'makosh-zulip-http', role: 'integration', owner: 'zulip', surface: 'implementation' },
  { name: 'makosh-zulip-persistence', role: 'integration', owner: 'zulip', surface: 'persistence' },
  { name: 'makosh-zulip-runtime', role: 'integration', owner: 'zulip', surface: 'runtime' },
  { name: 'makosh-communications-ingress', role: 'domain', owner: 'communications', surface: 'contract' },
  { name: 'makosh-communications-attachment-contract', role: 'domain', owner: 'communications', surface: 'contract' },
  { name: 'makosh-communications-api', role: 'domain', owner: 'communications', surface: 'contract' },
  { name: 'makosh-communications-domain', role: 'domain', owner: 'communications', surface: 'implementation' },
  { name: 'makosh-communications-persistence', role: 'domain', owner: 'communications', surface: 'persistence' },
  { name: 'makosh-communications-runtime', role: 'domain', owner: 'communications', surface: 'runtime' },
  { name: 'makosh-communications-assembly', role: 'domain', owner: 'communications', surface: 'assembly' },
];

const FIRST_OWNER_PRODUCTION_PACKAGES = [
  ...GATEWAY_RUNTIME_FOUNDATION_PRODUCTION_PACKAGES,
  { name: 'makosh-communications-ingress', role: 'domain', owner: 'communications', surface: 'contract' },
  { name: 'makosh-communications-attachment-contract', role: 'domain', owner: 'communications', surface: 'contract' },
  { name: 'makosh-communications-api', role: 'domain', owner: 'communications', surface: 'contract' },
  { name: 'makosh-communications-domain', role: 'domain', owner: 'communications', surface: 'implementation' },
  { name: 'makosh-communications-persistence', role: 'domain', owner: 'communications', surface: 'persistence' },
  { name: 'makosh-communications-runtime', role: 'domain', owner: 'communications', surface: 'runtime' },
  { name: 'makosh-communications-assembly', role: 'domain', owner: 'communications', surface: 'assembly' },
];

const ATTACHMENT_SECURITY_ENGINE_PRODUCTION_PACKAGES = [
  ...FIRST_OWNER_PRODUCTION_PACKAGES,
  { name: 'makosh-attachment-security-contract', role: 'engine', owner: 'attachment_security', surface: 'contract' },
  { name: 'makosh-attachment-security-core', role: 'engine', owner: 'attachment_security', surface: 'implementation' },
  { name: 'makosh-attachment-security-clamav', role: 'engine', owner: 'attachment_security', surface: 'implementation' },
  { name: 'makosh-attachment-security-persistence', role: 'engine', owner: 'attachment_security', surface: 'persistence' },
  { name: 'makosh-attachment-security-runtime', role: 'engine', owner: 'attachment_security', surface: 'runtime' },
  { name: 'makosh-attachment-security-assembly', role: 'engine', owner: 'attachment_security', surface: 'assembly' },
];

const MAIL_OUTBOUND_MIME_ATTACHMENTS_PRODUCTION_PACKAGES = [
  ...ATTACHMENT_SECURITY_ENGINE_PRODUCTION_PACKAGES,
  { name: 'makosh-mail-api', role: 'integration', owner: 'mail', surface: 'contract' },
  { name: 'makosh-mail-core', role: 'integration', owner: 'mail', surface: 'implementation' },
  { name: 'makosh-mail-imap', role: 'integration', owner: 'mail', surface: 'implementation' },
  { name: 'makosh-mail-gmail', role: 'integration', owner: 'mail', surface: 'implementation' },
  { name: 'makosh-mail-smtp', role: 'integration', owner: 'mail', surface: 'implementation' },
  { name: 'makosh-mail-persistence', role: 'integration', owner: 'mail', surface: 'persistence' },
  { name: 'makosh-mail-runtime', role: 'integration', owner: 'mail', surface: 'runtime' },
  { name: 'makosh-mail-assembly', role: 'integration', owner: 'mail', surface: 'assembly' },
];

const COMMUNICATIONS_CONTENT_READ_PRODUCTION_PACKAGES = [
  ...MAIL_OUTBOUND_MIME_ATTACHMENTS_PRODUCTION_PACKAGES,
  {
    name: 'makosh-communications-content-api',
    role: 'domain',
    owner: 'communications',
    surface: 'contract',
  },
];

const COMMUNICATIONS_SAVED_SEARCH_PRODUCTION_PACKAGES = [
  ...COMMUNICATIONS_CONTENT_READ_PRODUCTION_PACKAGES,
  {
    name: 'makosh-communications-saved-query-api',
    role: 'domain',
    owner: 'communications',
    surface: 'contract',
  },
];

const COMMUNICATIONS_SENDER_INSIGHTS_PRODUCTION_PACKAGES = [
  ...COMMUNICATIONS_SAVED_SEARCH_PRODUCTION_PACKAGES,
  {
    name: 'makosh-communications-sender-insights-api',
    role: 'domain',
    owner: 'communications',
    surface: 'contract',
  },
];

const COMMUNICATIONS_EXPORT_PRODUCTION_PACKAGES = [
  ...COMMUNICATIONS_SENDER_INSIGHTS_PRODUCTION_PACKAGES,
  {
    name: 'makosh-communications-evidence-export-source-api',
    role: 'domain',
    owner: 'communications',
    surface: 'contract',
  },
  {
    name: 'makosh-communications-export-api',
    role: 'workflow',
    owner: 'communications_export',
    surface: 'contract',
  },
  {
    name: 'makosh-communications-export-core',
    role: 'workflow',
    owner: 'communications_export',
    surface: 'implementation',
  },
  {
    name: 'makosh-communications-export-persistence',
    role: 'workflow',
    owner: 'communications_export',
    surface: 'persistence',
  },
  {
    name: 'makosh-communications-export-runtime',
    role: 'workflow',
    owner: 'communications_export',
    surface: 'runtime',
  },
  {
    name: 'makosh-communications-export-assembly',
    role: 'workflow',
    owner: 'communications_export',
    surface: 'assembly',
  },
];

const COMMUNICATION_DELIVERY_INTENT_CONTRACT_CORE_PRODUCTION_PACKAGES = [
  ...COMMUNICATIONS_EXPORT_PRODUCTION_PACKAGES,
  {
    name: 'makosh-communication-delivery-intent-api',
    role: 'workflow',
    owner: 'communication_delivery_intent',
    surface: 'contract',
  },
  {
    name: 'makosh-communication-delivery-intent-core',
    role: 'workflow',
    owner: 'communication_delivery_intent',
    surface: 'implementation',
  },
];

const COMMUNICATION_DELIVERY_INTENT_PERSISTENCE_PRODUCTION_PACKAGES = [
  ...COMMUNICATION_DELIVERY_INTENT_CONTRACT_CORE_PRODUCTION_PACKAGES,
  {
    name: 'makosh-communication-delivery-intent-persistence',
    role: 'workflow',
    owner: 'communication_delivery_intent',
    surface: 'persistence',
  },
];

const COMMUNICATION_DELIVERY_INTENT_RUNTIME_PRODUCTION_PACKAGES = [
  ...COMMUNICATION_DELIVERY_INTENT_PERSISTENCE_PRODUCTION_PACKAGES,
  {
    name: 'makosh-communication-delivery-intent-runtime',
    role: 'workflow',
    owner: 'communication_delivery_intent',
    surface: 'runtime',
  },
];

const COMMUNICATION_DELIVERY_INTENT_ASSEMBLY_PRODUCTION_PACKAGES = [
  ...COMMUNICATION_DELIVERY_INTENT_RUNTIME_PRODUCTION_PACKAGES,
  {
    name: 'makosh-communication-delivery-intent-assembly',
    role: 'workflow',
    owner: 'communication_delivery_intent',
    surface: 'assembly',
  },
];

const DELIVERY_INTENT_TRANSACTIONAL_EVENT_ADAPTERS_PRODUCTION_PACKAGES = [
  ...COMMUNICATION_DELIVERY_INTENT_ASSEMBLY_PRODUCTION_PACKAGES,
  {
    name: 'makosh-mail-delivery-intent-contract',
    role: 'integration',
    owner: 'mail',
    surface: 'contract',
  },
  {
    name: 'makosh-telegram-delivery-intent-contract',
    role: 'integration',
    owner: 'telegram',
    surface: 'contract',
  },
  {
    name: 'makosh-whatsapp-delivery-intent-contract',
    role: 'integration',
    owner: 'whatsapp',
    surface: 'contract',
  },
  {
    name: 'makosh-zulip-delivery-intent-contract',
    role: 'integration',
    owner: 'zulip',
    surface: 'contract',
  },
  {
    name: 'makosh-communication-delivery-intent-event-adapters',
    role: 'workflow',
    owner: 'communication_delivery_intent',
    surface: 'implementation',
  },
];

const COMMUNICATION_BULK_ACTION_CONTRACT_CORE_PRODUCTION_PACKAGES = [
  ...DELIVERY_INTENT_TRANSACTIONAL_EVENT_ADAPTERS_PRODUCTION_PACKAGES,
  {
    name: 'makosh-communication-bulk-action-api',
    role: 'workflow',
    owner: 'communication_bulk_action',
    surface: 'contract',
  },
  {
    name: 'makosh-communication-bulk-action-core',
    role: 'workflow',
    owner: 'communication_bulk_action',
    surface: 'implementation',
  },
];

const COMMUNICATION_BULK_ACTION_PERSISTENCE_PRODUCTION_PACKAGES = [
  ...COMMUNICATION_BULK_ACTION_CONTRACT_CORE_PRODUCTION_PACKAGES,
  {
    name: 'makosh-communication-bulk-action-persistence',
    role: 'workflow',
    owner: 'communication_bulk_action',
    surface: 'persistence',
  },
];

const COMMUNICATION_BULK_ACTION_RUNTIME_CORE_PRODUCTION_PACKAGES = [
  ...COMMUNICATION_BULK_ACTION_PERSISTENCE_PRODUCTION_PACKAGES,
  {
    name: 'makosh-communication-bulk-action-runtime',
    role: 'workflow',
    owner: 'communication_bulk_action',
    surface: 'runtime',
  },
];

const COMMUNICATION_BULK_ACTION_ASSEMBLY_PRODUCTION_PACKAGES = [
  ...COMMUNICATION_BULK_ACTION_RUNTIME_CORE_PRODUCTION_PACKAGES,
  {
    name: 'makosh-communication-bulk-action-assembly',
    role: 'workflow',
    owner: 'communication_bulk_action',
    surface: 'assembly',
  },
];

const COMMUNICATION_DELAYED_DELIVERY_CONTRACT_CORE_PRODUCTION_PACKAGES = [
  ...COMMUNICATION_BULK_ACTION_ASSEMBLY_PRODUCTION_PACKAGES,
  {
    name: 'makosh-communication-delayed-delivery-api',
    role: 'workflow',
    owner: 'communication_delayed_delivery',
    surface: 'contract',
  },
  {
    name: 'makosh-communication-delayed-delivery-core',
    role: 'workflow',
    owner: 'communication_delayed_delivery',
    surface: 'implementation',
  },
];

const COMMUNICATION_DELAYED_DELIVERY_PERSISTENCE_PRODUCTION_PACKAGES = [
  ...COMMUNICATION_DELAYED_DELIVERY_CONTRACT_CORE_PRODUCTION_PACKAGES,
  {
    name: 'makosh-communication-delayed-delivery-persistence',
    role: 'workflow',
    owner: 'communication_delayed_delivery',
    surface: 'persistence',
  },
];

const COMMUNICATION_DELAYED_DELIVERY_EXECUTION_PRODUCTION_PACKAGES = [
  ...COMMUNICATION_DELAYED_DELIVERY_PERSISTENCE_PRODUCTION_PACKAGES,
  {
    name: 'makosh-communication-delayed-delivery-execution',
    role: 'workflow',
    owner: 'communication_delayed_delivery',
    surface: 'implementation',
  },
];

const COMMUNICATION_DELAYED_DELIVERY_EVENT_ADAPTERS_PRODUCTION_PACKAGES = [
  ...COMMUNICATION_DELAYED_DELIVERY_EXECUTION_PRODUCTION_PACKAGES,
  {
    name: 'makosh-communication-delayed-delivery-event-adapters',
    role: 'workflow',
    owner: 'communication_delayed_delivery',
    surface: 'implementation',
  },
];

const COMMUNICATION_DELAYED_DELIVERY_RUNTIME_ADAPTERS_PRODUCTION_PACKAGES = [
  ...COMMUNICATION_DELAYED_DELIVERY_EVENT_ADAPTERS_PRODUCTION_PACKAGES,
  {
    name: 'makosh-communication-delayed-delivery-runtime-adapters',
    role: 'workflow',
    owner: 'communication_delayed_delivery',
    surface: 'implementation',
  },
];

const COMMUNICATION_DELAYED_DELIVERY_STORE_ADAPTERS_PRODUCTION_PACKAGES = [
  ...COMMUNICATION_DELAYED_DELIVERY_RUNTIME_ADAPTERS_PRODUCTION_PACKAGES,
  {
    name: 'makosh-communication-delayed-delivery-store-adapters',
    role: 'workflow',
    owner: 'communication_delayed_delivery',
    surface: 'persistence',
  },
];

const COMMUNICATION_DELAYED_DELIVERY_MANAGED_RUNTIME_PRODUCTION_PACKAGES = [
  ...COMMUNICATION_DELAYED_DELIVERY_STORE_ADAPTERS_PRODUCTION_PACKAGES,
  {
    name: 'makosh-communication-delayed-delivery-runtime',
    role: 'workflow',
    owner: 'communication_delayed_delivery',
    surface: 'runtime',
  },
];

const COMMUNICATION_DELAYED_DELIVERY_ASSEMBLY_PRODUCTION_PACKAGES = [
  ...COMMUNICATION_DELAYED_DELIVERY_MANAGED_RUNTIME_PRODUCTION_PACKAGES,
  {
    name: 'makosh-communication-delayed-delivery-assembly',
    role: 'workflow',
    owner: 'communication_delayed_delivery',
    surface: 'assembly',
  },
];

const COMMUNICATION_CROSS_CHANNEL_FORWARD_CONTRACT_CORE_PRODUCTION_PACKAGES = [
  ...COMMUNICATION_DELAYED_DELIVERY_ASSEMBLY_PRODUCTION_PACKAGES,
  {
    name: 'makosh-communication-cross-channel-forward-api',
    role: 'workflow',
    owner: 'communication_cross_channel_forward',
    surface: 'contract',
  },
  {
    name: 'makosh-communication-cross-channel-forward-core',
    role: 'workflow',
    owner: 'communication_cross_channel_forward',
    surface: 'implementation',
  },
];

const COMMUNICATION_CROSS_CHANNEL_FORWARD_PERSISTENCE_PRODUCTION_PACKAGES = [
  ...COMMUNICATION_CROSS_CHANNEL_FORWARD_CONTRACT_CORE_PRODUCTION_PACKAGES,
  {
    name: 'makosh-communication-cross-channel-forward-persistence',
    role: 'workflow',
    owner: 'communication_cross_channel_forward',
    surface: 'persistence',
  },
];

const COMMUNICATION_CROSS_CHANNEL_FORWARD_SOURCE_CONTRACT_PRODUCTION_PACKAGES = [
  ...COMMUNICATION_CROSS_CHANNEL_FORWARD_PERSISTENCE_PRODUCTION_PACKAGES,
  {
    name: 'makosh-communications-cross-channel-forward-source-api',
    role: 'domain',
    owner: 'communications',
    surface: 'contract',
  },
];

const COMMUNICATION_DELIVERY_INTENT_INGRESS_CONTRACT_PRODUCTION_PACKAGES = [
  ...COMMUNICATION_CROSS_CHANNEL_FORWARD_SOURCE_CONTRACT_PRODUCTION_PACKAGES,
  {
    name: 'makosh-communication-delivery-intent-ingress-api',
    role: 'workflow',
    owner: 'communication_delivery_intent',
    surface: 'contract',
  },
];

const COMMUNICATION_CROSS_CHANNEL_FORWARD_EVENT_PERSISTENCE_PRODUCTION_PACKAGES =
  COMMUNICATION_DELIVERY_INTENT_INGRESS_CONTRACT_PRODUCTION_PACKAGES;

const COMMUNICATION_CROSS_CHANNEL_FORWARD_MANAGED_RUNTIME_PRODUCTION_PACKAGES = [
  ...COMMUNICATION_CROSS_CHANNEL_FORWARD_EVENT_PERSISTENCE_PRODUCTION_PACKAGES,
  {
    name: 'makosh-communication-cross-channel-forward-runtime',
    role: 'workflow',
    owner: 'communication_cross_channel_forward',
    surface: 'runtime',
  },
];

const COMMUNICATION_CROSS_CHANNEL_FORWARD_CLIENT_ASSEMBLY_PRODUCTION_PACKAGES = [
  ...COMMUNICATION_CROSS_CHANNEL_FORWARD_MANAGED_RUNTIME_PRODUCTION_PACKAGES,
  {
    name: 'makosh-communication-cross-channel-forward-assembly',
    role: 'workflow',
    owner: 'communication_cross_channel_forward',
    surface: 'assembly',
  },
];

const COMMUNICATIONS_CALL_EVIDENCE_CONTRACT_CORE_PRODUCTION_PACKAGES = [
  ...COMMUNICATION_CROSS_CHANNEL_FORWARD_CLIENT_ASSEMBLY_PRODUCTION_PACKAGES,
  {
    name: 'makosh-communications-call-evidence-ingress',
    role: 'domain',
    owner: 'communications',
    surface: 'contract',
  },
  {
    name: 'makosh-communications-call-evidence-core',
    role: 'domain',
    owner: 'communications',
    surface: 'implementation',
  },
];

const COMMUNICATIONS_CALL_EVIDENCE_PERSISTENCE_PRODUCTION_PACKAGES = [
  ...COMMUNICATIONS_CALL_EVIDENCE_CONTRACT_CORE_PRODUCTION_PACKAGES,
  {
    name: 'makosh-communications-call-evidence-persistence',
    role: 'domain',
    owner: 'communications',
    surface: 'persistence',
  },
];

const COMMUNICATIONS_CALL_EVIDENCE_QUERY_REALTIME_PRODUCTION_PACKAGES = [
  ...COMMUNICATIONS_CALL_EVIDENCE_PERSISTENCE_PRODUCTION_PACKAGES,
  {
    name: 'makosh-communications-call-evidence-api',
    role: 'domain',
    owner: 'communications',
    surface: 'contract',
  },
];

const REVIEW_COMMUNICATIONS_ATTENTION_CONTRACT_CORE_PRODUCTION_PACKAGES = [
  ...COMMUNICATIONS_CALL_EVIDENCE_QUERY_REALTIME_PRODUCTION_PACKAGES,
  {
    name: 'makosh-review-attention-api',
    role: 'domain',
    owner: 'review',
    surface: 'contract',
  },
  {
    name: 'makosh-review-attention-core',
    role: 'domain',
    owner: 'review',
    surface: 'implementation',
  },
];

const REVIEW_COMMUNICATIONS_ATTENTION_PERSISTENCE_PRODUCTION_PACKAGES = [
  ...REVIEW_COMMUNICATIONS_ATTENTION_CONTRACT_CORE_PRODUCTION_PACKAGES,
  {
    name: 'makosh-review-attention-persistence',
    role: 'domain',
    owner: 'review',
    surface: 'persistence',
  },
];

const REVIEW_COMMUNICATIONS_ATTENTION_MANAGED_RUNTIME_PRODUCTION_PACKAGES = [
  ...REVIEW_COMMUNICATIONS_ATTENTION_PERSISTENCE_PRODUCTION_PACKAGES,
  {
    name: 'makosh-review-attention-runtime',
    role: 'domain',
    owner: 'review',
    surface: 'runtime',
  },
];

const REVIEW_COMMUNICATIONS_ATTENTION_ASSEMBLY_PRODUCTION_PACKAGES = [
  ...REVIEW_COMMUNICATIONS_ATTENTION_MANAGED_RUNTIME_PRODUCTION_PACKAGES,
  {
    name: 'makosh-review-attention-assembly',
    role: 'domain',
    owner: 'review',
    surface: 'assembly',
  },
];

const COMMUNICATIONS_AI_SOURCE_CONTRACT_PRODUCTION_PACKAGES = [
  ...REVIEW_COMMUNICATIONS_ATTENTION_ASSEMBLY_PRODUCTION_PACKAGES,
  {
    name: 'makosh-communications-ai-source-api',
    role: 'domain',
    owner: 'communications',
    surface: 'contract',
  },
  {
    name: 'makosh-communication-reply-suggestion-api',
    role: 'workflow',
    owner: 'communication_reply_suggestion',
    surface: 'contract',
  },
  {
    name: 'makosh-communication-reply-suggestion-core',
    role: 'workflow',
    owner: 'communication_reply_suggestion',
    surface: 'implementation',
  },
  {
    name: 'makosh-communication-reply-suggestion-persistence',
    role: 'workflow',
    owner: 'communication_reply_suggestion',
    surface: 'persistence',
  },
  {
    name: 'makosh-communication-reply-suggestion-runtime',
    role: 'workflow',
    owner: 'communication_reply_suggestion',
    surface: 'runtime',
  },
  {
    name: 'makosh-communication-reply-suggestion-assembly',
    role: 'workflow',
    owner: 'communication_reply_suggestion',
    surface: 'assembly',
  },
  {
    name: 'makosh-ai-contracts',
    role: 'engine',
    owner: 'ai',
    surface: 'contract',
  },
  {
    name: 'makosh-ai-inference-core',
    role: 'engine',
    owner: 'ai',
    surface: 'implementation',
  },
  {
    name: 'makosh-ai-inference-persistence',
    role: 'engine',
    owner: 'ai',
    surface: 'persistence',
  },
  {
    name: 'makosh-ollama-ai-api',
    role: 'integration',
    owner: 'ollama',
    surface: 'contract',
  },
  {
    name: 'makosh-ollama-ai-assembly',
    role: 'integration',
    owner: 'ollama',
    surface: 'assembly',
  },
  {
    name: 'makosh-ollama-ai-core',
    role: 'integration',
    owner: 'ollama',
    surface: 'implementation',
  },
  {
    name: 'makosh-ollama-ai-http',
    role: 'integration',
    owner: 'ollama',
    surface: 'implementation',
  },
  {
    name: 'makosh-ollama-ai-persistence',
    role: 'integration',
    owner: 'ollama',
    surface: 'persistence',
  },
  {
    name: 'makosh-ollama-ai-runtime',
    role: 'integration',
    owner: 'ollama',
    surface: 'runtime',
  },
];

const ATTACHMENT_ARCHIVE_INSPECTION_CONTRACT_CORE_PRODUCTION_PACKAGES = [
  ...COMMUNICATIONS_AI_SOURCE_CONTRACT_PRODUCTION_PACKAGES,
  {
    name: 'makosh-attachment-archive-inspection-api',
    role: 'engine',
    owner: 'attachment_archive_inspection',
    surface: 'contract',
  },
  {
    name: 'makosh-attachment-archive-inspection-ingress',
    role: 'engine',
    owner: 'attachment_archive_inspection',
    surface: 'contract',
  },
  {
    name: 'makosh-attachment-archive-inspection-core',
    role: 'engine',
    owner: 'attachment_archive_inspection',
    surface: 'implementation',
  },
  {
    name: 'makosh-attachment-archive-inspection-zip',
    role: 'engine',
    owner: 'attachment_archive_inspection',
    surface: 'implementation',
  },
];

const ATTACHMENT_ARCHIVE_INSPECTION_PERSISTENCE_PRODUCTION_PACKAGES = [
  ...ATTACHMENT_ARCHIVE_INSPECTION_CONTRACT_CORE_PRODUCTION_PACKAGES,
  {
    name: 'makosh-attachment-archive-inspection-persistence',
    role: 'engine',
    owner: 'attachment_archive_inspection',
    surface: 'persistence',
  },
];

const ATTACHMENT_ARCHIVE_INSPECTION_RUNTIME_PRODUCTION_PACKAGES = [
  ...ATTACHMENT_ARCHIVE_INSPECTION_PERSISTENCE_PRODUCTION_PACKAGES,
  {
    name: 'makosh-attachment-archive-inspection-runtime',
    role: 'engine',
    owner: 'attachment_archive_inspection',
    surface: 'runtime',
  },
];

const ATTACHMENT_ARCHIVE_INSPECTION_ASSEMBLY_PRODUCTION_PACKAGES = [
  ...ATTACHMENT_ARCHIVE_INSPECTION_RUNTIME_PRODUCTION_PACKAGES,
  {
    name: 'makosh-attachment-archive-inspection-assembly',
    role: 'engine',
    owner: 'attachment_archive_inspection',
    surface: 'assembly',
  },
];

const COMMUNICATION_SUMMARY_BUILD_UNITS_PRODUCTION_PACKAGES = [
  ...ATTACHMENT_ARCHIVE_INSPECTION_ASSEMBLY_PRODUCTION_PACKAGES,
  { name: 'makosh-communication-summary-api', role: 'workflow', owner: 'communication_summary', surface: 'contract' },
  { name: 'makosh-communication-summary-core', role: 'workflow', owner: 'communication_summary', surface: 'implementation' },
  { name: 'makosh-communication-summary-persistence', role: 'workflow', owner: 'communication_summary', surface: 'persistence' },
  { name: 'makosh-communication-summary-runtime', role: 'workflow', owner: 'communication_summary', surface: 'runtime' },
  { name: 'makosh-communication-summary-assembly', role: 'workflow', owner: 'communication_summary', surface: 'assembly' },
];

const COMMUNICATION_TRANSLATION_CONTRACT_CORE_PRODUCTION_PACKAGES = [
  ...COMMUNICATION_SUMMARY_BUILD_UNITS_PRODUCTION_PACKAGES,
  { name: 'makosh-communication-translation-api', role: 'workflow', owner: 'communication_translation', surface: 'contract' },
  { name: 'makosh-communication-translation-core', role: 'workflow', owner: 'communication_translation', surface: 'implementation' },
];

const COMMUNICATION_TRANSLATION_PERSISTENCE_PRODUCTION_PACKAGES = [
  ...COMMUNICATION_TRANSLATION_CONTRACT_CORE_PRODUCTION_PACKAGES,
  { name: 'makosh-communication-translation-persistence', role: 'workflow', owner: 'communication_translation', surface: 'persistence' },
];

const COMMUNICATION_TRANSLATION_RUNTIME_PRODUCTION_PACKAGES = [
  ...COMMUNICATION_TRANSLATION_PERSISTENCE_PRODUCTION_PACKAGES,
  { name: 'makosh-communication-translation-runtime', role: 'workflow', owner: 'communication_translation', surface: 'runtime' },
];

const COMMUNICATION_TRANSLATION_ASSEMBLY_PRODUCTION_PACKAGES = [
  ...COMMUNICATION_TRANSLATION_RUNTIME_PRODUCTION_PACKAGES,
  { name: 'makosh-communication-translation-assembly', role: 'workflow', owner: 'communication_translation', surface: 'assembly' },
];

const COMMUNICATION_EXPLANATION_CONTRACT_CORE_PRODUCTION_PACKAGES = [
  ...COMMUNICATION_TRANSLATION_ASSEMBLY_PRODUCTION_PACKAGES,
  { name: 'makosh-communication-explanation-api', role: 'workflow', owner: 'communication_explanation', surface: 'contract' },
  { name: 'makosh-communication-explanation-core', role: 'workflow', owner: 'communication_explanation', surface: 'implementation' },
];

const COMMUNICATION_EXPLANATION_PERSISTENCE_PRODUCTION_PACKAGES = [
  ...COMMUNICATION_EXPLANATION_CONTRACT_CORE_PRODUCTION_PACKAGES,
  { name: 'makosh-communication-explanation-persistence', role: 'workflow', owner: 'communication_explanation', surface: 'persistence' },
];

const COMMUNICATION_EXPLANATION_RUNTIME_PRODUCTION_PACKAGES = [
  ...COMMUNICATION_EXPLANATION_PERSISTENCE_PRODUCTION_PACKAGES,
  { name: 'makosh-communication-explanation-runtime', role: 'workflow', owner: 'communication_explanation', surface: 'runtime' },
];

const COMMUNICATION_EXPLANATION_ASSEMBLY_PRODUCTION_PACKAGES = [
  ...COMMUNICATION_EXPLANATION_RUNTIME_PRODUCTION_PACKAGES,
  { name: 'makosh-communication-explanation-assembly', role: 'workflow', owner: 'communication_explanation', surface: 'assembly' },
];

const COMMUNICATION_RECIPIENT_SUGGESTION_CONTRACT_CORE_PRODUCTION_PACKAGES = [
  ...COMMUNICATION_EXPLANATION_ASSEMBLY_PRODUCTION_PACKAGES,
  { name: 'makosh-communication-recipient-suggestion-api', role: 'workflow', owner: 'communication_recipient_suggestion', surface: 'contract' },
  { name: 'makosh-communication-recipient-suggestion-core', role: 'workflow', owner: 'communication_recipient_suggestion', surface: 'implementation' },
];

const COMMUNICATION_RECIPIENT_SUGGESTION_SOURCE_CONTRACT_PRODUCTION_PACKAGES = [
  ...COMMUNICATION_RECIPIENT_SUGGESTION_CONTRACT_CORE_PRODUCTION_PACKAGES,
  { name: 'makosh-communications-recipient-source-api', role: 'domain', owner: 'communications', surface: 'contract' },
];

const COMMUNICATION_RECIPIENT_SUGGESTION_PERSISTENCE_PRODUCTION_PACKAGES = [
  ...COMMUNICATION_RECIPIENT_SUGGESTION_CONTRACT_CORE_PRODUCTION_PACKAGES,
  { name: 'makosh-communication-recipient-suggestion-persistence', role: 'workflow', owner: 'communication_recipient_suggestion', surface: 'persistence' },
  { name: 'makosh-communications-recipient-source-api', role: 'domain', owner: 'communications', surface: 'contract' },
];

const COMMUNICATION_RECIPIENT_SUGGESTION_RUNTIME_PRODUCTION_PACKAGES = [
  ...COMMUNICATION_RECIPIENT_SUGGESTION_CONTRACT_CORE_PRODUCTION_PACKAGES,
  { name: 'makosh-communication-recipient-suggestion-persistence', role: 'workflow', owner: 'communication_recipient_suggestion', surface: 'persistence' },
  { name: 'makosh-communication-recipient-suggestion-runtime', role: 'workflow', owner: 'communication_recipient_suggestion', surface: 'runtime' },
  { name: 'makosh-communications-recipient-source-api', role: 'domain', owner: 'communications', surface: 'contract' },
];

const COMMUNICATION_RECIPIENT_SUGGESTION_ASSEMBLY_PRODUCTION_PACKAGES = [
  ...COMMUNICATION_RECIPIENT_SUGGESTION_RUNTIME_PRODUCTION_PACKAGES,
  { name: 'makosh-communication-recipient-suggestion-assembly', role: 'workflow', owner: 'communication_recipient_suggestion', surface: 'assembly' },
];

const COMMUNICATION_TASK_CANDIDATE_CONTRACT_CORE_SOURCE_PRODUCTION_PACKAGES = [
  ...COMMUNICATION_RECIPIENT_SUGGESTION_ASSEMBLY_PRODUCTION_PACKAGES,
  { name: 'makosh-communication-task-candidate-api', role: 'workflow', owner: 'communication_task_candidate_extraction', surface: 'contract' },
  { name: 'makosh-communication-task-candidate-core', role: 'workflow', owner: 'communication_task_candidate_extraction', surface: 'implementation' },
  { name: 'makosh-communications-task-source-api', role: 'domain', owner: 'communications', surface: 'contract' },
];

const COMMUNICATION_TASK_CANDIDATE_PERSISTENCE_PRODUCTION_PACKAGES = [
  ...COMMUNICATION_TASK_CANDIDATE_CONTRACT_CORE_SOURCE_PRODUCTION_PACKAGES,
  { name: 'makosh-communication-task-candidate-persistence', role: 'workflow', owner: 'communication_task_candidate_extraction', surface: 'persistence' },
];

const COMMUNICATION_TASK_CANDIDATE_RUNTIME_PRODUCTION_PACKAGES = [
  ...COMMUNICATION_TASK_CANDIDATE_PERSISTENCE_PRODUCTION_PACKAGES,
  { name: 'makosh-communication-task-candidate-runtime', role: 'workflow', owner: 'communication_task_candidate_extraction', surface: 'runtime' },
];

const COMMUNICATION_TASK_CANDIDATE_ASSEMBLY_PRODUCTION_PACKAGES = [
  ...COMMUNICATION_TASK_CANDIDATE_RUNTIME_PRODUCTION_PACKAGES,
  { name: 'makosh-communication-task-candidate-assembly', role: 'workflow', owner: 'communication_task_candidate_extraction', surface: 'assembly' },
];

const REVIEW_TASK_CANDIDATE_CORE_PRODUCTION_PACKAGES = [
  ...COMMUNICATION_TASK_CANDIDATE_ASSEMBLY_PRODUCTION_PACKAGES,
  { name: 'makosh-review-task-candidate-api', role: 'domain', owner: 'review', surface: 'contract' },
  { name: 'makosh-review-task-candidate-core', role: 'domain', owner: 'review', surface: 'implementation' },
];

const REVIEW_TASK_CANDIDATE_PERSISTENCE_PRODUCTION_PACKAGES = [
  ...REVIEW_TASK_CANDIDATE_CORE_PRODUCTION_PACKAGES,
  { name: 'makosh-review-task-candidate-persistence', role: 'domain', owner: 'review', surface: 'persistence' },
];

const REVIEW_TASK_CANDIDATE_MANAGED_RUNTIME_PRODUCTION_PACKAGES = [
  ...REVIEW_TASK_CANDIDATE_PERSISTENCE_PRODUCTION_PACKAGES,
  { name: 'makosh-review-task-candidate-runtime', role: 'domain', owner: 'review', surface: 'runtime' },
];

const REVIEW_TASK_CANDIDATE_ASSEMBLY_PRODUCTION_PACKAGES = [
  ...REVIEW_TASK_CANDIDATE_MANAGED_RUNTIME_PRODUCTION_PACKAGES,
  { name: 'makosh-review-task-candidate-assembly', role: 'domain', owner: 'review', surface: 'assembly' },
];

const TASKS_REVIEWED_CANDIDATE_CONTRACT_CORE_PRODUCTION_PACKAGES = [
  ...REVIEW_TASK_CANDIDATE_ASSEMBLY_PRODUCTION_PACKAGES,
  { name: 'makosh-tasks-command-api', role: 'domain', owner: 'tasks', surface: 'contract' },
  { name: 'makosh-tasks-core', role: 'domain', owner: 'tasks', surface: 'implementation' },
];

const TASKS_REVIEWED_CANDIDATE_PERSISTENCE_PRODUCTION_PACKAGES = [
  ...TASKS_REVIEWED_CANDIDATE_CONTRACT_CORE_PRODUCTION_PACKAGES,
  { name: 'makosh-tasks-persistence', role: 'domain', owner: 'tasks', surface: 'persistence' },
];

const TASKS_REVIEWED_CANDIDATE_MANAGED_RUNTIME_PRODUCTION_PACKAGES = [
  ...TASKS_REVIEWED_CANDIDATE_PERSISTENCE_PRODUCTION_PACKAGES,
  { name: 'makosh-tasks-runtime', role: 'domain', owner: 'tasks', surface: 'runtime' },
];

const TASKS_REVIEWED_CANDIDATE_ASSEMBLY_PRODUCTION_PACKAGES = [
  ...TASKS_REVIEWED_CANDIDATE_MANAGED_RUNTIME_PRODUCTION_PACKAGES,
  { name: 'makosh-tasks-assembly', role: 'domain', owner: 'tasks', surface: 'assembly' },
];

const REVIEWED_TASK_CANDIDATE_PROMOTION_CONTRACT_CORE_PRODUCTION_PACKAGES = [
  ...TASKS_REVIEWED_CANDIDATE_ASSEMBLY_PRODUCTION_PACKAGES,
  { name: 'makosh-review-task-candidate-promotion-api', role: 'domain', owner: 'review', surface: 'contract' },
  { name: 'makosh-reviewed-task-candidate-promotion-core', role: 'workflow', owner: 'reviewed_task_candidate_promotion', surface: 'implementation' },
];

const REVIEWED_TASK_CANDIDATE_PROMOTION_PERSISTENCE_PRODUCTION_PACKAGES = [
  ...REVIEWED_TASK_CANDIDATE_PROMOTION_CONTRACT_CORE_PRODUCTION_PACKAGES,
  { name: 'makosh-reviewed-task-candidate-promotion-persistence', role: 'workflow', owner: 'reviewed_task_candidate_promotion', surface: 'persistence' },
];

const REVIEWED_TASK_CANDIDATE_PROMOTION_RUNTIME_PRODUCTION_PACKAGES = [
  ...REVIEWED_TASK_CANDIDATE_PROMOTION_PERSISTENCE_PRODUCTION_PACKAGES,
  { name: 'makosh-reviewed-task-candidate-promotion-runtime', role: 'workflow', owner: 'reviewed_task_candidate_promotion', surface: 'runtime' },
];

const REVIEWED_TASK_CANDIDATE_PROMOTION_ASSEMBLY_PRODUCTION_PACKAGES = [
  ...REVIEWED_TASK_CANDIDATE_PROMOTION_RUNTIME_PRODUCTION_PACKAGES,
  { name: 'makosh-reviewed-task-candidate-promotion-assembly', role: 'workflow', owner: 'reviewed_task_candidate_promotion', surface: 'assembly' },
];

const COMMUNICATION_NOTE_CANDIDATE_CONTRACT_CORE_PRODUCTION_PACKAGES = [
  ...REVIEWED_TASK_CANDIDATE_PROMOTION_ASSEMBLY_PRODUCTION_PACKAGES,
  { name: 'makosh-communication-note-candidate-api', role: 'workflow', owner: 'communication_note_candidate_extraction', surface: 'contract' },
  { name: 'makosh-communication-note-candidate-core', role: 'workflow', owner: 'communication_note_candidate_extraction', surface: 'implementation' },
  { name: 'makosh-communications-note-source-api', role: 'domain', owner: 'communications', surface: 'contract' },
];

const COMMUNICATION_NOTE_CANDIDATE_PERSISTENCE_PRODUCTION_PACKAGES = [
  ...COMMUNICATION_NOTE_CANDIDATE_CONTRACT_CORE_PRODUCTION_PACKAGES,
  { name: 'makosh-communication-note-candidate-persistence', role: 'workflow', owner: 'communication_note_candidate_extraction', surface: 'persistence' },
];

const REVIEW_NOTE_CANDIDATE_CONTRACT_CORE_PRODUCTION_PACKAGES = [
  ...COMMUNICATION_NOTE_CANDIDATE_PERSISTENCE_PRODUCTION_PACKAGES,
  { name: 'makosh-review-note-candidate-api', role: 'domain', owner: 'review', surface: 'contract' },
  { name: 'makosh-review-note-candidate-core', role: 'domain', owner: 'review', surface: 'implementation' },
];

const KNOWLEDGE_VERIFIED_NOTE_CONTRACT_CORE_PRODUCTION_PACKAGES = [
  ...REVIEW_NOTE_CANDIDATE_CONTRACT_CORE_PRODUCTION_PACKAGES,
  { name: 'makosh-knowledge-command-api', role: 'domain', owner: 'knowledge', surface: 'contract' },
  { name: 'makosh-knowledge-core', role: 'domain', owner: 'knowledge', surface: 'implementation' },
];

const KNOWLEDGE_VERIFIED_NOTE_PERSISTENCE_PRODUCTION_PACKAGES = [
  ...KNOWLEDGE_VERIFIED_NOTE_CONTRACT_CORE_PRODUCTION_PACKAGES,
  { name: 'makosh-knowledge-persistence', role: 'domain', owner: 'knowledge', surface: 'persistence' },
];

const KNOWLEDGE_VERIFIED_NOTE_MANAGED_RUNTIME_PRODUCTION_PACKAGES = [
  ...KNOWLEDGE_VERIFIED_NOTE_PERSISTENCE_PRODUCTION_PACKAGES,
  { name: 'makosh-knowledge-runtime', role: 'domain', owner: 'knowledge', surface: 'runtime' },
];

const KNOWLEDGE_VERIFIED_NOTE_ASSEMBLY_PRODUCTION_PACKAGES = [
  ...KNOWLEDGE_VERIFIED_NOTE_MANAGED_RUNTIME_PRODUCTION_PACKAGES,
  { name: 'makosh-knowledge-assembly', role: 'domain', owner: 'knowledge', surface: 'assembly' },
];

const REVIEW_NOTE_CANDIDATE_PERSISTENCE_PRODUCTION_PACKAGES = [
  ...KNOWLEDGE_VERIFIED_NOTE_ASSEMBLY_PRODUCTION_PACKAGES,
  { name: 'makosh-review-note-candidate-persistence', role: 'domain', owner: 'review', surface: 'persistence' },
];

const REVIEW_NOTE_CANDIDATE_MANAGED_RUNTIME_PRODUCTION_PACKAGES = [
  ...REVIEW_NOTE_CANDIDATE_PERSISTENCE_PRODUCTION_PACKAGES,
  { name: 'makosh-review-note-candidate-promotion-api', role: 'domain', owner: 'review', surface: 'contract' },
  { name: 'makosh-review-note-candidate-runtime', role: 'domain', owner: 'review', surface: 'runtime' },
];

const REVIEW_NOTE_CANDIDATE_ASSEMBLY_PRODUCTION_PACKAGES = [
  ...REVIEW_NOTE_CANDIDATE_MANAGED_RUNTIME_PRODUCTION_PACKAGES,
  { name: 'makosh-review-note-candidate-assembly', role: 'domain', owner: 'review', surface: 'assembly' },
];

const REVIEWED_NOTE_CANDIDATE_PROMOTION_PRODUCTION_PACKAGES = [
  ...REVIEW_NOTE_CANDIDATE_ASSEMBLY_PRODUCTION_PACKAGES,
  { name: 'makosh-reviewed-note-candidate-promotion-core', role: 'workflow', owner: 'reviewed_note_candidate_promotion', surface: 'implementation' },
  { name: 'makosh-reviewed-note-candidate-promotion-persistence', role: 'workflow', owner: 'reviewed_note_candidate_promotion', surface: 'persistence' },
  { name: 'makosh-reviewed-note-candidate-promotion-runtime', role: 'workflow', owner: 'reviewed_note_candidate_promotion', surface: 'runtime' },
  { name: 'makosh-reviewed-note-candidate-promotion-assembly', role: 'workflow', owner: 'reviewed_note_candidate_promotion', surface: 'assembly' },
];

const COMMUNICATION_NOTE_CANDIDATE_ASSEMBLY_PRODUCTION_PACKAGES = [
  ...REVIEWED_NOTE_CANDIDATE_PROMOTION_PRODUCTION_PACKAGES,
  { name: 'makosh-communication-note-candidate-runtime', role: 'workflow', owner: 'communication_note_candidate_extraction', surface: 'runtime' },
  { name: 'makosh-communication-note-candidate-assembly', role: 'workflow', owner: 'communication_note_candidate_extraction', surface: 'assembly' },
];

const ATTACHMENT_TEXT_EXTRACTION_CONTRACT_CORE_PRODUCTION_PACKAGES = [
  ...COMMUNICATION_NOTE_CANDIDATE_ASSEMBLY_PRODUCTION_PACKAGES,
  { name: 'makosh-attachment-text-extraction-api', role: 'workflow', owner: 'attachment_text_extraction', surface: 'contract' },
  { name: 'makosh-attachment-text-extraction-ingress', role: 'workflow', owner: 'attachment_text_extraction', surface: 'contract' },
  { name: 'makosh-attachment-text-extraction-core', role: 'workflow', owner: 'attachment_text_extraction', surface: 'implementation' },
];

const ATTACHMENT_TEXT_EXTRACTION_PARSER_ADAPTERS_PRODUCTION_PACKAGES = [
  ...ATTACHMENT_TEXT_EXTRACTION_CONTRACT_CORE_PRODUCTION_PACKAGES,
  { name: 'makosh-attachment-text-extraction-parser-contract', role: 'workflow', owner: 'attachment_text_extraction', surface: 'contract' },
  { name: 'makosh-attachment-text-extraction-plain', role: 'workflow', owner: 'attachment_text_extraction', surface: 'implementation' },
  { name: 'makosh-attachment-text-extraction-pdf', role: 'workflow', owner: 'attachment_text_extraction', surface: 'implementation' },
  { name: 'makosh-attachment-text-extraction-docx', role: 'workflow', owner: 'attachment_text_extraction', surface: 'implementation' },
  { name: 'makosh-attachment-text-extraction-ocr', role: 'workflow', owner: 'attachment_text_extraction', surface: 'implementation' },
];

const ATTACHMENT_TEXT_EXTRACTION_PERSISTENCE_PRODUCTION_PACKAGES = [
  ...ATTACHMENT_TEXT_EXTRACTION_PARSER_ADAPTERS_PRODUCTION_PACKAGES,
  { name: 'makosh-attachment-text-extraction-persistence', role: 'workflow', owner: 'attachment_text_extraction', surface: 'persistence' },
];

const ATTACHMENT_TEXT_EXTRACTION_RUNTIME_ASSEMBLY_PRODUCTION_PACKAGES = [
  ...ATTACHMENT_TEXT_EXTRACTION_PERSISTENCE_PRODUCTION_PACKAGES,
  { name: 'makosh-attachment-text-extraction-runtime', role: 'workflow', owner: 'attachment_text_extraction', surface: 'runtime' },
  { name: 'makosh-attachment-text-extraction-assembly', role: 'workflow', owner: 'attachment_text_extraction', surface: 'assembly' },
];

const ATTACHMENT_PREVIEW_FOUNDATION_PRODUCTION_PACKAGES = [
  ...ATTACHMENT_TEXT_EXTRACTION_RUNTIME_ASSEMBLY_PRODUCTION_PACKAGES,
  { name: 'makosh-attachment-preview-api', role: 'workflow', owner: 'attachment_preview', surface: 'contract' },
  { name: 'makosh-attachment-preview-ingress', role: 'workflow', owner: 'attachment_preview', surface: 'contract' },
  { name: 'makosh-attachment-preview-core', role: 'workflow', owner: 'attachment_preview', surface: 'implementation' },
  { name: 'makosh-attachment-preview-renderer-contract', role: 'workflow', owner: 'attachment_preview', surface: 'contract' },
];

const ATTACHMENT_PREVIEW_SAFE_ADAPTERS_PRODUCTION_PACKAGES = [
  ...ATTACHMENT_PREVIEW_FOUNDATION_PRODUCTION_PACKAGES,
  { name: 'makosh-attachment-preview-text', role: 'workflow', owner: 'attachment_preview', surface: 'implementation' },
  { name: 'makosh-attachment-preview-image', role: 'workflow', owner: 'attachment_preview', surface: 'implementation' },
  { name: 'makosh-attachment-preview-media', role: 'workflow', owner: 'attachment_preview', surface: 'implementation' },
];

const ATTACHMENT_PREVIEW_PDF_ADAPTER_PRODUCTION_PACKAGES = [
  ...ATTACHMENT_PREVIEW_SAFE_ADAPTERS_PRODUCTION_PACKAGES,
  { name: 'makosh-attachment-preview-pdf', role: 'workflow', owner: 'attachment_preview', surface: 'implementation' },
];

const ATTACHMENT_PREVIEW_DOCX_ADAPTER_PRODUCTION_PACKAGES = [
  ...ATTACHMENT_PREVIEW_PDF_ADAPTER_PRODUCTION_PACKAGES,
  { name: 'makosh-attachment-preview-docx', role: 'workflow', owner: 'attachment_preview', surface: 'implementation' },
];

const ATTACHMENT_PREVIEW_PERSISTENCE_PRODUCTION_PACKAGES = [
  ...ATTACHMENT_PREVIEW_DOCX_ADAPTER_PRODUCTION_PACKAGES,
  { name: 'makosh-attachment-preview-persistence', role: 'workflow', owner: 'attachment_preview', surface: 'persistence' },
];

const ATTACHMENT_PREVIEW_RUNTIME_PRODUCTION_PACKAGES = [
  ...ATTACHMENT_PREVIEW_PERSISTENCE_PRODUCTION_PACKAGES,
  { name: 'makosh-attachment-preview-runtime', role: 'workflow', owner: 'attachment_preview', surface: 'runtime' },
];

const ATTACHMENT_PREVIEW_ASSEMBLY_PRODUCTION_PACKAGES = [
  ...ATTACHMENT_PREVIEW_RUNTIME_PRODUCTION_PACKAGES,
  { name: 'makosh-attachment-preview-assembly', role: 'workflow', owner: 'attachment_preview', surface: 'assembly' },
];

const ATTACHMENT_PREVIEW_RETAINED_EVIDENCE_REPLAY_PRODUCTION_PACKAGES = [
  ...ATTACHMENT_PREVIEW_ASSEMBLY_PRODUCTION_PACKAGES,
  { name: 'makosh-retained-evidence-replay-protocol', role: 'workflow', owner: 'attachment_preview_evidence_replay', surface: 'contract' },
  { name: 'makosh-attachment-preview-evidence-replay-api', role: 'workflow', owner: 'attachment_preview_evidence_replay', surface: 'contract' },
  { name: 'makosh-attachment-preview-evidence-replay-core', role: 'workflow', owner: 'attachment_preview_evidence_replay', surface: 'implementation' },
  { name: 'makosh-attachment-preview-evidence-replay-persistence', role: 'workflow', owner: 'attachment_preview_evidence_replay', surface: 'persistence' },
  { name: 'makosh-attachment-preview-evidence-replay-runtime', role: 'workflow', owner: 'attachment_preview_evidence_replay', surface: 'runtime' },
  { name: 'makosh-attachment-preview-evidence-replay-assembly', role: 'workflow', owner: 'attachment_preview_evidence_replay', surface: 'assembly' },
  { name: 'makosh-communications-retained-evidence-replay-persistence', role: 'domain', owner: 'communications', surface: 'persistence' },
  { name: 'makosh-mail-retained-evidence-replay-persistence', role: 'integration', owner: 'mail', surface: 'persistence' },
  { name: 'makosh-communications-retained-evidence-replay-contract', role: 'domain', owner: 'communications', surface: 'contract' },
  { name: 'makosh-mail-retained-evidence-replay-contract', role: 'integration', owner: 'mail', surface: 'contract' },
];

const ATTACHMENT_TRANSLATION_CONTRACTS_PRODUCTION_PACKAGES = [
  ...ATTACHMENT_PREVIEW_RETAINED_EVIDENCE_REPLAY_PRODUCTION_PACKAGES,
  { name: 'makosh-attachment-translation-api', role: 'workflow', owner: 'attachment_translation', surface: 'contract' },
  { name: 'makosh-attachment-translation-ingress', role: 'workflow', owner: 'attachment_translation', surface: 'contract' },
  { name: 'makosh-attachment-translation-core', role: 'workflow', owner: 'attachment_translation', surface: 'implementation' },
];

const ATTACHMENT_TRANSLATION_PERSISTENCE_PRODUCTION_PACKAGES = [
  ...ATTACHMENT_TRANSLATION_CONTRACTS_PRODUCTION_PACKAGES,
  { name: 'makosh-attachment-translation-persistence', role: 'workflow', owner: 'attachment_translation', surface: 'persistence' },
];

const ATTACHMENT_TRANSLATION_RUNTIME_ASSEMBLY_PRODUCTION_PACKAGES = [
  ...ATTACHMENT_TRANSLATION_PERSISTENCE_PRODUCTION_PACKAGES,
  { name: 'makosh-attachment-translation-runtime', role: 'workflow', owner: 'attachment_translation', surface: 'runtime' },
  { name: 'makosh-attachment-translation-assembly', role: 'workflow', owner: 'attachment_translation', surface: 'assembly' },
];

const CONTACTS_MAIL_IDENTITY_COMMAND_CONTRACT_CORE_PRODUCTION_PACKAGES =
  ATTACHMENT_TRANSLATION_RUNTIME_ASSEMBLY_PRODUCTION_PACKAGES.flatMap((packageDescriptor) => (
    packageDescriptor.name === 'makosh-tasks-core'
      ? [
          packageDescriptor,
          { name: 'makosh-contacts-command-api', role: 'domain', owner: 'contacts', surface: 'contract' },
          { name: 'makosh-contacts-core', role: 'domain', owner: 'contacts', surface: 'implementation' },
        ]
      : [packageDescriptor]
  ));

const CONTACTS_MAIL_IDENTITY_COMMAND_PERSISTENCE_PRODUCTION_PACKAGES =
  CONTACTS_MAIL_IDENTITY_COMMAND_CONTRACT_CORE_PRODUCTION_PACKAGES.flatMap((packageDescriptor) => (
    packageDescriptor.name === 'makosh-contacts-core'
      ? [
          packageDescriptor,
          { name: 'makosh-contacts-persistence', role: 'domain', owner: 'contacts', surface: 'persistence' },
        ]
      : [packageDescriptor]
  ));

const CONTACTS_MAIL_IDENTITY_COMMAND_RUNTIME_ASSEMBLY_PRODUCTION_PACKAGES =
  CONTACTS_MAIL_IDENTITY_COMMAND_PERSISTENCE_PRODUCTION_PACKAGES.flatMap((packageDescriptor) => (
    packageDescriptor.name === 'makosh-contacts-persistence'
      ? [
          packageDescriptor,
          { name: 'makosh-contacts-runtime', role: 'domain', owner: 'contacts', surface: 'runtime' },
          { name: 'makosh-contacts-assembly', role: 'domain', owner: 'contacts', surface: 'assembly' },
        ]
      : [packageDescriptor]
  ));

const MAIL_CONTACTS_SYNC_CONTRACT_CORE_PRODUCTION_PACKAGES = [
  ...CONTACTS_MAIL_IDENTITY_COMMAND_RUNTIME_ASSEMBLY_PRODUCTION_PACKAGES,
  { name: 'makosh-mail-address-book-contract', role: 'integration', owner: 'mail', surface: 'contract' },
  { name: 'makosh-mail-contacts-sync-api', role: 'workflow', owner: 'mail_contacts_sync', surface: 'contract' },
  { name: 'makosh-mail-contacts-sync-core', role: 'workflow', owner: 'mail_contacts_sync', surface: 'implementation' },
];

const MAIL_CONTACTS_SYNC_PERSISTENCE_PRODUCTION_PACKAGES = [
  ...MAIL_CONTACTS_SYNC_CONTRACT_CORE_PRODUCTION_PACKAGES,
  { name: 'makosh-mail-contacts-sync-persistence', role: 'workflow', owner: 'mail_contacts_sync', surface: 'persistence' },
];

const MAIL_CONTACTS_SYNC_RUNTIME_ADMISSION_PRODUCTION_PACKAGES =
  MAIL_CONTACTS_SYNC_PERSISTENCE_PRODUCTION_PACKAGES.flatMap((packageDescriptor) => (
    packageDescriptor.name === 'makosh-contacts-command-api'
      ? [
          packageDescriptor,
          { name: 'makosh-contacts-mail-sync-source-api', role: 'domain', owner: 'contacts', surface: 'contract' },
        ]
      : [packageDescriptor]
  )).concat([
    { name: 'makosh-mail-contacts-sync-runtime', role: 'workflow', owner: 'mail_contacts_sync', surface: 'runtime' },
  ]);

const MAIL_ADDRESS_BOOK_PROVIDER_ADAPTERS_PRODUCTION_PACKAGES = [
  ...MAIL_CONTACTS_SYNC_RUNTIME_ADMISSION_PRODUCTION_PACKAGES,
  { name: 'makosh-mail-google-people', role: 'integration', owner: 'mail', surface: 'implementation' },
  { name: 'makosh-mail-carddav', role: 'integration', owner: 'mail', surface: 'implementation' },
];

const MAIL_ADDRESS_BOOK_PERSISTENCE_AUTHORITY_PRODUCTION_PACKAGES =
  MAIL_ADDRESS_BOOK_PROVIDER_ADAPTERS_PRODUCTION_PACKAGES.flatMap((packageDescriptor) => (
    packageDescriptor.name === 'makosh-mail-address-book-contract'
      ? [
          packageDescriptor,
          { name: 'makosh-mail-address-book-persistence', role: 'integration', owner: 'mail', surface: 'persistence' },
        ]
      : [packageDescriptor]
  ));

const MAIL_ADDRESS_BOOK_RUNTIME_EXECUTION_PRODUCTION_PACKAGES =
  MAIL_ADDRESS_BOOK_PERSISTENCE_AUTHORITY_PRODUCTION_PACKAGES;

const MAIL_CONTACTS_SYNC_RELEASE_ASSEMBLY_PRODUCTION_PACKAGES = [
  ...MAIL_ADDRESS_BOOK_RUNTIME_EXECUTION_PRODUCTION_PACKAGES,
  { name: 'makosh-mail-contacts-sync-assembly', role: 'workflow', owner: 'mail_contacts_sync', surface: 'assembly' },
  { name: 'makosh-speech-to-text-api', role: 'engine', owner: 'speech_to_text', surface: 'contract' },
  { name: 'makosh-speech-to-text-core', role: 'engine', owner: 'speech_to_text', surface: 'implementation' },
  { name: 'makosh-speech-to-text-persistence', role: 'engine', owner: 'speech_to_text', surface: 'persistence' },
];

const DESKTOP_CALL_RECORDING_CONTRACT_CORE_PRODUCTION_PACKAGES = [
  ...MAIL_CONTACTS_SYNC_RELEASE_ASSEMBLY_PRODUCTION_PACKAGES,
  { name: 'makosh-desktop-call-recording-api', role: 'integration', owner: 'desktop_call_recording', surface: 'contract' },
  { name: 'makosh-desktop-call-recording-core', role: 'integration', owner: 'desktop_call_recording', surface: 'implementation' },
  { name: 'makosh-call-transcription-ingress', role: 'workflow', owner: 'call_transcription', surface: 'contract' },
];

const DESKTOP_CALL_RECORDING_PERSISTENCE_PRODUCTION_PACKAGES = [
  ...DESKTOP_CALL_RECORDING_CONTRACT_CORE_PRODUCTION_PACKAGES,
  { name: 'makosh-desktop-call-recording-persistence', role: 'integration', owner: 'desktop_call_recording', surface: 'persistence' },
];

const DESKTOP_CALL_RECORDING_RUNTIME_PRODUCTION_PACKAGES = [
  ...DESKTOP_CALL_RECORDING_PERSISTENCE_PRODUCTION_PACKAGES,
  { name: 'makosh-desktop-call-recording-runtime', role: 'integration', owner: 'desktop_call_recording', surface: 'runtime' },
];

const DESKTOP_CALL_RECORDING_RELEASE_ASSEMBLY_PRODUCTION_PACKAGES = [
  ...DESKTOP_CALL_RECORDING_RUNTIME_PRODUCTION_PACKAGES,
  { name: 'makosh-desktop-call-recording-assembly', role: 'integration', owner: 'desktop_call_recording', surface: 'assembly' },
];

const CALL_TRANSCRIPTION_CONTRACT_CORE_PRODUCTION_PACKAGES = [
  ...DESKTOP_CALL_RECORDING_RELEASE_ASSEMBLY_PRODUCTION_PACKAGES,
  { name: 'makosh-call-transcription-api', role: 'workflow', owner: 'call_transcription', surface: 'contract' },
  { name: 'makosh-call-transcription-core', role: 'workflow', owner: 'call_transcription', surface: 'implementation' },
];

const CALL_TRANSCRIPTION_PERSISTENCE_PRODUCTION_PACKAGES = [
  ...CALL_TRANSCRIPTION_CONTRACT_CORE_PRODUCTION_PACKAGES,
  { name: 'makosh-call-transcription-persistence', role: 'workflow', owner: 'call_transcription', surface: 'persistence' },
];

const CALL_TRANSCRIPTION_RUNTIME_PRODUCTION_PACKAGES = [
  ...CALL_TRANSCRIPTION_PERSISTENCE_PRODUCTION_PACKAGES,
  { name: 'makosh-call-transcription-runtime', role: 'workflow', owner: 'call_transcription', surface: 'runtime' },
];

const CALL_TRANSCRIPTION_RELEASE_ASSEMBLY_PRODUCTION_PACKAGES = [
  ...CALL_TRANSCRIPTION_RUNTIME_PRODUCTION_PACKAGES,
  { name: 'makosh-call-transcription-assembly', role: 'workflow', owner: 'call_transcription', surface: 'assembly' },
];

const PERSONS_CONTRACT_CORE_PRODUCTION_PACKAGES = [
  ...CALL_TRANSCRIPTION_RELEASE_ASSEMBLY_PRODUCTION_PACKAGES,
  { name: 'makosh-persons-api', role: 'domain', owner: 'persons', surface: 'contract' },
  { name: 'makosh-persons-core', role: 'domain', owner: 'persons', surface: 'implementation' },
];

const PERSONS_PERSISTENCE_PRODUCTION_PACKAGES = [
  ...PERSONS_CONTRACT_CORE_PRODUCTION_PACKAGES,
  { name: 'makosh-persons-persistence', role: 'domain', owner: 'persons', surface: 'persistence' },
];

const PERSONS_RUNTIME_PRODUCTION_PACKAGES = [
  ...PERSONS_PERSISTENCE_PRODUCTION_PACKAGES,
  { name: 'makosh-persons-runtime', role: 'domain', owner: 'persons', surface: 'runtime' },
];

const PERSONS_ASSEMBLY_PRODUCTION_PACKAGES = [
  ...PERSONS_RUNTIME_PRODUCTION_PACKAGES,
  { name: 'makosh-persons-assembly', role: 'domain', owner: 'persons', surface: 'assembly' },
];

const MAIL_PERSONS_SYNC_CONTRACT_CORE_PRODUCTION_PACKAGES = [
  ...PERSONS_ASSEMBLY_PRODUCTION_PACKAGES,
  { name: 'makosh-mail-persons-sync-api', role: 'workflow', owner: 'mail_persons_sync', surface: 'contract' },
  { name: 'makosh-mail-persons-sync-core', role: 'workflow', owner: 'mail_persons_sync', surface: 'implementation' },
  { name: 'makosh-mail-persons-sync-persistence', role: 'workflow', owner: 'mail_persons_sync', surface: 'persistence' },
  { name: 'makosh-mail-persons-sync-runtime', role: 'workflow', owner: 'mail_persons_sync', surface: 'runtime' },
  { name: 'makosh-mail-persons-sync-assembly', role: 'workflow', owner: 'mail_persons_sync', surface: 'assembly' },
  { name: 'makosh-review-person-match-candidate-api', role: 'domain', owner: 'review', surface: 'contract' },
  { name: 'makosh-review-person-match-candidate-core', role: 'domain', owner: 'review', surface: 'implementation' },
  { name: 'makosh-review-person-match-candidate-persistence', role: 'domain', owner: 'review', surface: 'persistence' },
  { name: 'makosh-review-person-match-candidate-runtime', role: 'domain', owner: 'review', surface: 'runtime' },
  { name: 'makosh-review-person-match-candidate-assembly', role: 'domain', owner: 'review', surface: 'assembly' },
  { name: 'makosh-review-person-match-candidate-promotion-api', role: 'domain', owner: 'review', surface: 'contract' },
  { name: 'makosh-reviewed-person-match-candidate-promotion-core', role: 'workflow', owner: 'reviewed_person_match_candidate_promotion', surface: 'implementation' },
  { name: 'makosh-reviewed-person-match-candidate-promotion-persistence', role: 'workflow', owner: 'reviewed_person_match_candidate_promotion', surface: 'persistence' },
  { name: 'makosh-reviewed-person-match-candidate-promotion-runtime', role: 'workflow', owner: 'reviewed_person_match_candidate_promotion', surface: 'runtime' },
  { name: 'makosh-reviewed-person-match-candidate-promotion-assembly', role: 'workflow', owner: 'reviewed_person_match_candidate_promotion', surface: 'assembly' },
];

const BLOB_FOUNDATION_WORKSPACE_DEPENDENCY_ALLOWLIST = {
  ...NATS_FOUNDATION_WORKSPACE_DEPENDENCY_ALLOWLIST,
  'makosh-blob-protocol': [],
};

const BLOB_RUNTIME_FOUNDATION_WORKSPACE_DEPENDENCY_ALLOWLIST = {
  ...BLOB_FOUNDATION_WORKSPACE_DEPENDENCY_ALLOWLIST,
  'makosh-blob-client-contract': [
    { name: 'makosh-runtime-protocol', kind: 'normal' },
  ],
  'makosh-blob-client': [
    { name: 'makosh-blob-client-contract', kind: 'normal' },
    { name: 'makosh-runtime-protocol', kind: 'normal' },
  ],
  'makosh-blob-runtime': [
    { name: 'makosh-blob-protocol', kind: 'normal' },
    { name: 'makosh-runtime-protocol', kind: 'normal' },
    { name: 'makosh-vault-protocol', kind: 'normal' },
  ],
  'makosh-blob-service': [
    { name: 'makosh-blob-protocol', kind: 'normal' },
    { name: 'makosh-blob-runtime', kind: 'normal' },
    { name: 'makosh-runtime-protocol', kind: 'normal' },
    { name: 'makosh-vault-protocol', kind: 'normal' },
  ],
};

const SCHEDULER_PROTOCOL_FOUNDATION_WORKSPACE_DEPENDENCY_ALLOWLIST = {
  ...BLOB_RUNTIME_FOUNDATION_WORKSPACE_DEPENDENCY_ALLOWLIST,
  'makosh-kernel': [
    ...BLOB_RUNTIME_FOUNDATION_WORKSPACE_DEPENDENCY_ALLOWLIST['makosh-kernel'],
    { name: 'makosh-scheduler-protocol', kind: 'normal' },
  ],
  'makosh-scheduler-protocol': [
    { name: 'makosh-clock-protocol', kind: 'normal' },
  ],
};

const SCHEDULER_FOUNDATION_WORKSPACE_DEPENDENCY_ALLOWLIST = {
  ...SCHEDULER_PROTOCOL_FOUNDATION_WORKSPACE_DEPENDENCY_ALLOWLIST,
  'makosh-scheduler': [
    { name: 'makosh-clock-protocol', kind: 'normal' },
    { name: 'makosh-events-protocol', kind: 'normal' },
    { name: 'makosh-scheduler-protocol', kind: 'normal' },
  ],
};

const SCHEDULER_PERSISTENCE_FOUNDATION_WORKSPACE_DEPENDENCY_ALLOWLIST = {
  ...SCHEDULER_FOUNDATION_WORKSPACE_DEPENDENCY_ALLOWLIST,
  'makosh-scheduler-persistence': [
    { name: 'makosh-clock-protocol', kind: 'normal' },
    { name: 'makosh-events-protocol', kind: 'normal' },
    { name: 'makosh-runtime-protocol', kind: 'normal' },
    { name: 'makosh-scheduler', kind: 'normal' },
    { name: 'makosh-scheduler-protocol', kind: 'normal' },
    { name: 'makosh-storage-protocol', kind: 'normal' },
  ],
};

const GATEWAY_SESSION_FOUNDATION_WORKSPACE_DEPENDENCY_ALLOWLIST = {
  ...SCHEDULER_PERSISTENCE_FOUNDATION_WORKSPACE_DEPENDENCY_ALLOWLIST,
  'makosh-kernel': [
    ...SCHEDULER_PERSISTENCE_FOUNDATION_WORKSPACE_DEPENDENCY_ALLOWLIST['makosh-kernel'],
    { name: 'makosh-gateway-session-contract', kind: 'normal' },
  ],
  'makosh-gateway-session-contract': [],
  'makosh-gateway-session': [
    { name: 'makosh-gateway-session-contract', kind: 'normal' },
  ],
};

const SCHEDULER_RECEIPT_DELIVERY_FOUNDATION_WORKSPACE_DEPENDENCY_ALLOWLIST = {
  ...GATEWAY_SESSION_FOUNDATION_WORKSPACE_DEPENDENCY_ALLOWLIST,
};

const SCHEDULER_JETSTREAM_FOUNDATION_WORKSPACE_DEPENDENCY_ALLOWLIST = {
  ...SCHEDULER_RECEIPT_DELIVERY_FOUNDATION_WORKSPACE_DEPENDENCY_ALLOWLIST,
  'makosh-scheduler-jetstream': [
    { name: 'makosh-events-protocol', kind: 'normal' },
    { name: 'makosh-runtime-protocol', kind: 'normal' },
    { name: 'makosh-scheduler-protocol', kind: 'normal' },
  ],
};

const SCHEDULER_RUNTIME_FOUNDATION_WORKSPACE_DEPENDENCY_ALLOWLIST = {
  ...SCHEDULER_JETSTREAM_FOUNDATION_WORKSPACE_DEPENDENCY_ALLOWLIST,
  'makosh-scheduler-runtime': [
    { name: 'makosh-clock-protocol', kind: 'normal' },
    { name: 'makosh-events-protocol', kind: 'normal' },
    { name: 'makosh-runtime-protocol', kind: 'normal' },
    { name: 'makosh-scheduler', kind: 'normal' },
    { name: 'makosh-scheduler-jetstream', kind: 'normal' },
    { name: 'makosh-scheduler-persistence', kind: 'normal' },
    { name: 'makosh-scheduler-protocol', kind: 'normal' },
    { name: 'makosh-secure-file', kind: 'normal' },
    { name: 'makosh-storage-vault', kind: 'normal' },
  ],
};

const GATEWAY_RUNTIME_FOUNDATION_WORKSPACE_DEPENDENCY_ALLOWLIST = {
  ...SCHEDULER_RUNTIME_FOUNDATION_WORKSPACE_DEPENDENCY_ALLOWLIST,
  'makosh-kernel': [
    { name: 'makosh-blob-client', kind: 'normal' },
    ...SCHEDULER_RUNTIME_FOUNDATION_WORKSPACE_DEPENDENCY_ALLOWLIST['makosh-kernel'],
    { name: 'makosh-gateway-runtime', kind: 'normal' },
    { name: 'makosh-gateway-session', kind: 'normal' },
    { name: 'makosh-vault-protocol', kind: 'normal' },
  ],
  'makosh-gateway-runtime': [
    { name: 'makosh-gateway-protocol', kind: 'normal' },
    { name: 'makosh-gateway-session', kind: 'normal' },
    { name: 'makosh-gateway-session-contract', kind: 'normal' },
  ],
};

const MAIL_COMMUNICATIONS_FOUNDATION_WORKSPACE_DEPENDENCY_ALLOWLIST = {
  ...GATEWAY_RUNTIME_FOUNDATION_WORKSPACE_DEPENDENCY_ALLOWLIST,
  'makosh-mail-api': [],
  'makosh-mail-core': [
    { name: 'makosh-mail-api', kind: 'normal' },
    { name: 'makosh-communications-ingress', kind: 'normal' },
  ],
  'makosh-mail-imap': [
    { name: 'makosh-mail-core', kind: 'normal' },
    { name: 'makosh-mail-api', kind: 'normal' },
  ],
  'makosh-mail-gmail': [
    { name: 'makosh-mail-api', kind: 'normal' },
  ],
  'makosh-mail-smtp': [
    { name: 'makosh-mail-api', kind: 'normal' },
  ],
  'makosh-mail-persistence': [
    { name: 'makosh-events-protocol', kind: 'normal' },
    { name: 'makosh-mail-api', kind: 'normal' },
    { name: 'makosh-storage-protocol', kind: 'normal' },
  ],
  'makosh-mail-runtime': [
    { name: 'makosh-mail-api', kind: 'normal' },
    { name: 'makosh-mail-core', kind: 'normal' },
    { name: 'makosh-mail-imap', kind: 'normal' },
    { name: 'makosh-mail-gmail', kind: 'normal' },
    { name: 'makosh-mail-smtp', kind: 'normal' },
    { name: 'makosh-mail-persistence', kind: 'normal' },
    { name: 'makosh-attachment-security-contract', kind: 'normal' },
    { name: 'makosh-communications-attachment-contract', kind: 'normal' },
    { name: 'makosh-communications-ingress', kind: 'normal' },
    { name: 'makosh-events-protocol', kind: 'normal' },
    { name: 'makosh-events-jetstream', kind: 'normal' },
    { name: 'makosh-blob-client', kind: 'normal' },
    { name: 'makosh-managed-vault-client', kind: 'normal' },
    { name: 'makosh-runtime-protocol', kind: 'normal' },
    { name: 'makosh-storage-protocol', kind: 'normal' },
    { name: 'makosh-storage-vault', kind: 'normal' },
    { name: 'makosh-vault-protocol', kind: 'normal' },
  ],
  'makosh-mail-assembly': [
    { name: 'makosh-mail-persistence', kind: 'normal' },
    { name: 'makosh-mail-runtime', kind: 'normal' },
    { name: 'makosh-runtime-protocol', kind: 'normal' },
    { name: 'makosh-storage-protocol', kind: 'normal' },
  ],
  'makosh-telegram-api': [],
  'makosh-telegram-core': [
    { name: 'makosh-telegram-api', kind: 'normal' },
    { name: 'makosh-communications-ingress', kind: 'normal' },
    { name: 'makosh-vault-protocol', kind: 'normal' },
  ],
  'makosh-telegram-tdlib': [
    { name: 'makosh-telegram-api', kind: 'normal' },
  ],
  'makosh-telegram-persistence': [
    { name: 'makosh-communications-ingress', kind: 'normal' },
    { name: 'makosh-events-protocol', kind: 'normal' },
    { name: 'makosh-telegram-api', kind: 'normal' },
    { name: 'makosh-storage-protocol', kind: 'normal' },
  ],
  'makosh-telegram-runtime': [
    { name: 'makosh-blob-client-contract', kind: 'normal' },
    { name: 'makosh-communications-ingress', kind: 'normal' },
    { name: 'makosh-events-protocol', kind: 'normal' },
    { name: 'makosh-events-jetstream', kind: 'normal' },
    { name: 'makosh-managed-vault-client', kind: 'normal' },
    { name: 'makosh-runtime-protocol', kind: 'normal' },
    { name: 'makosh-storage-protocol', kind: 'normal' },
    { name: 'makosh-storage-vault', kind: 'normal' },
    { name: 'makosh-telegram-api', kind: 'normal' },
    { name: 'makosh-telegram-core', kind: 'normal' },
    { name: 'makosh-telegram-persistence', kind: 'normal' },
    { name: 'makosh-telegram-tdlib', kind: 'normal' },
    { name: 'makosh-vault-protocol', kind: 'normal' },
    { name: 'makosh-blob-client', kind: 'normal' },
  ],
  'makosh-telegram-assembly': [
    { name: 'makosh-runtime-protocol', kind: 'normal' },
    { name: 'makosh-storage-protocol', kind: 'normal' },
    { name: 'makosh-telegram-persistence', kind: 'normal' },
    { name: 'makosh-telegram-runtime', kind: 'normal' },
  ],
  'makosh-whatsapp-api': [],
  'makosh-whatsapp-core': [
    { name: 'makosh-communications-ingress', kind: 'normal' },
    { name: 'makosh-whatsapp-api', kind: 'normal' },
  ],
  'makosh-whatsapp-persistence': [
    { name: 'makosh-events-protocol', kind: 'normal' },
    { name: 'makosh-storage-protocol', kind: 'normal' },
  ],
  'makosh-whatsapp-runtime': [
    { name: 'makosh-communications-ingress', kind: 'normal' },
    { name: 'makosh-events-jetstream', kind: 'normal' },
    { name: 'makosh-runtime-protocol', kind: 'normal' },
    { name: 'makosh-storage-protocol', kind: 'normal' },
    { name: 'makosh-storage-vault', kind: 'normal' },
    { name: 'makosh-vault-protocol', kind: 'normal' },
    { name: 'makosh-whatsapp-api', kind: 'normal' },
    { name: 'makosh-whatsapp-core', kind: 'normal' },
    { name: 'makosh-whatsapp-persistence', kind: 'normal' },
  ],
  'makosh-whatsapp-assembly': [
    { name: 'makosh-runtime-protocol', kind: 'normal' },
    { name: 'makosh-storage-protocol', kind: 'normal' },
    { name: 'makosh-whatsapp-persistence', kind: 'normal' },
    { name: 'makosh-whatsapp-runtime', kind: 'normal' },
  ],
  'makosh-zulip-api': [],
  'makosh-zulip-core': [
    { name: 'makosh-communications-ingress', kind: 'normal' },
    { name: 'makosh-zulip-api', kind: 'normal' },
    { name: 'makosh-vault-protocol', kind: 'normal' },
  ],
  'makosh-zulip-http': [{ name: 'makosh-zulip-api', kind: 'normal' }],
  'makosh-zulip-persistence': [
    { name: 'makosh-events-protocol', kind: 'normal' },
    { name: 'makosh-storage-protocol', kind: 'normal' },
    { name: 'makosh-zulip-api', kind: 'normal' },
  ],
  'makosh-zulip-runtime': [
    { name: 'makosh-blob-client', kind: 'normal' },
    { name: 'makosh-blob-client-contract', kind: 'normal' },
    { name: 'makosh-communications-ingress', kind: 'normal' },
    { name: 'makosh-events-jetstream', kind: 'normal' },
    { name: 'makosh-managed-vault-client', kind: 'normal' },
    { name: 'makosh-runtime-protocol', kind: 'normal' },
    { name: 'makosh-storage-protocol', kind: 'normal' },
    { name: 'makosh-zulip-api', kind: 'normal' },
    { name: 'makosh-zulip-core', kind: 'normal' },
    { name: 'makosh-zulip-http', kind: 'normal' },
    { name: 'makosh-zulip-persistence', kind: 'normal' },
    { name: 'makosh-storage-vault', kind: 'normal' },
    { name: 'makosh-vault-protocol', kind: 'normal' },
  ],
  'makosh-communications-ingress': [
    { name: 'makosh-events-protocol', kind: 'normal' },
    { name: 'makosh-runtime-protocol', kind: 'normal' },
  ],
  'makosh-communications-attachment-contract': [
    { name: 'makosh-events-protocol', kind: 'normal' },
    { name: 'makosh-runtime-protocol', kind: 'normal' },
  ],
  'makosh-communications-api': [],
  'makosh-communications-domain': [
    { name: 'makosh-communications-api', kind: 'normal' },
  ],
  'makosh-communications-persistence': [
    { name: 'makosh-communications-api', kind: 'normal' },
    { name: 'makosh-events-protocol', kind: 'normal' },
    { name: 'makosh-storage-protocol', kind: 'normal' },
  ],
  'makosh-communications-runtime': [
    { name: 'makosh-blob-client', kind: 'normal' },
    { name: 'makosh-communications-attachment-contract', kind: 'normal' },
    { name: 'makosh-communications-ingress', kind: 'normal' },
    { name: 'makosh-communications-api', kind: 'normal' },
    { name: 'makosh-communications-domain', kind: 'normal' },
    { name: 'makosh-communications-persistence', kind: 'normal' },
    { name: 'makosh-events-jetstream', kind: 'normal' },
    { name: 'makosh-events-protocol', kind: 'normal' },
    { name: 'makosh-managed-vault-client', kind: 'normal' },
    { name: 'makosh-runtime-protocol', kind: 'normal' },
    { name: 'makosh-storage-protocol', kind: 'normal' },
    { name: 'makosh-storage-vault', kind: 'normal' },
  ],
  'makosh-communications-assembly': [
    { name: 'makosh-communications-persistence', kind: 'normal' },
    { name: 'makosh-communications-runtime', kind: 'normal' },
    { name: 'makosh-runtime-protocol', kind: 'normal' },
    { name: 'makosh-storage-protocol', kind: 'normal' },
  ],
};

const FIRST_OWNER_WORKSPACE_DEPENDENCY_ALLOWLIST = Object.fromEntries(
  FIRST_OWNER_PRODUCTION_PACKAGES.map(({ name }) => [
    name,
    MAIL_COMMUNICATIONS_FOUNDATION_WORKSPACE_DEPENDENCY_ALLOWLIST[name],
  ]),
);

const ATTACHMENT_SECURITY_ENGINE_WORKSPACE_DEPENDENCY_ALLOWLIST = {
  ...FIRST_OWNER_WORKSPACE_DEPENDENCY_ALLOWLIST,
  'makosh-attachment-security-contract': [
    { name: 'makosh-events-protocol', kind: 'normal' },
    { name: 'makosh-runtime-protocol', kind: 'normal' },
  ],
  'makosh-attachment-security-core': [
    { name: 'makosh-attachment-security-contract', kind: 'normal' },
  ],
  'makosh-attachment-security-clamav': [
    { name: 'makosh-attachment-security-contract', kind: 'normal' },
    { name: 'makosh-attachment-security-core', kind: 'normal' },
  ],
  'makosh-attachment-security-persistence': [
    { name: 'makosh-attachment-archive-inspection-ingress', kind: 'normal' },
    { name: 'makosh-attachment-preview-ingress', kind: 'normal' },
    { name: 'makosh-attachment-text-extraction-ingress', kind: 'normal' },
    { name: 'makosh-attachment-security-core', kind: 'normal' },
    { name: 'makosh-communications-attachment-contract', kind: 'normal' },
    { name: 'makosh-events-protocol', kind: 'normal' },
    { name: 'makosh-storage-protocol', kind: 'normal' },
  ],
  'makosh-attachment-security-runtime': [
    { name: 'makosh-attachment-archive-inspection-ingress', kind: 'normal' },
    { name: 'makosh-attachment-preview-ingress', kind: 'normal' },
    { name: 'makosh-attachment-text-extraction-ingress', kind: 'normal' },
    { name: 'makosh-attachment-security-clamav', kind: 'normal' },
    { name: 'makosh-attachment-security-contract', kind: 'normal' },
    { name: 'makosh-attachment-security-core', kind: 'normal' },
    { name: 'makosh-attachment-security-persistence', kind: 'normal' },
    { name: 'makosh-blob-client', kind: 'normal' },
    { name: 'makosh-communications-attachment-contract', kind: 'normal' },
    { name: 'makosh-events-jetstream', kind: 'normal' },
    { name: 'makosh-events-protocol', kind: 'normal' },
    { name: 'makosh-runtime-protocol', kind: 'normal' },
    { name: 'makosh-storage-protocol', kind: 'normal' },
    { name: 'makosh-storage-vault', kind: 'normal' },
  ],
  'makosh-attachment-security-assembly': [
    { name: 'makosh-attachment-security-persistence', kind: 'normal' },
    { name: 'makosh-attachment-security-runtime', kind: 'normal' },
    { name: 'makosh-runtime-protocol', kind: 'normal' },
    { name: 'makosh-storage-protocol', kind: 'normal' },
  ],
};

const MAIL_OUTBOUND_MIME_ATTACHMENTS_WORKSPACE_DEPENDENCY_ALLOWLIST = {
  ...ATTACHMENT_SECURITY_ENGINE_WORKSPACE_DEPENDENCY_ALLOWLIST,
  ...Object.fromEntries(
    MAIL_OUTBOUND_MIME_ATTACHMENTS_PRODUCTION_PACKAGES
      .filter(({ owner }) => owner === 'mail')
      .map(({ name }) => [
        name,
        MAIL_COMMUNICATIONS_FOUNDATION_WORKSPACE_DEPENDENCY_ALLOWLIST[name],
      ]),
  ),
};

const COMMUNICATIONS_CONTENT_READ_WORKSPACE_DEPENDENCY_ALLOWLIST = {
  ...MAIL_OUTBOUND_MIME_ATTACHMENTS_WORKSPACE_DEPENDENCY_ALLOWLIST,
  'makosh-communications-content-api': [],
  'makosh-communications-runtime': [
    { name: 'makosh-blob-client', kind: 'normal' },
    { name: 'makosh-communications-attachment-contract', kind: 'normal' },
    { name: 'makosh-communications-content-api', kind: 'normal' },
    { name: 'makosh-communications-ingress', kind: 'normal' },
    { name: 'makosh-communications-api', kind: 'normal' },
    { name: 'makosh-communications-domain', kind: 'normal' },
    { name: 'makosh-communications-persistence', kind: 'normal' },
    { name: 'makosh-events-jetstream', kind: 'normal' },
    { name: 'makosh-events-protocol', kind: 'normal' },
    { name: 'makosh-managed-vault-client', kind: 'normal' },
    { name: 'makosh-runtime-protocol', kind: 'normal' },
    { name: 'makosh-storage-protocol', kind: 'normal' },
    { name: 'makosh-storage-vault', kind: 'normal' },
  ],
};

const COMMUNICATIONS_SAVED_SEARCH_WORKSPACE_DEPENDENCY_ALLOWLIST = {
  ...COMMUNICATIONS_CONTENT_READ_WORKSPACE_DEPENDENCY_ALLOWLIST,
  'makosh-communications-saved-query-api': [],
  'makosh-communications-runtime': [
    { name: 'makosh-blob-client', kind: 'normal' },
    { name: 'makosh-communications-attachment-contract', kind: 'normal' },
    { name: 'makosh-communications-content-api', kind: 'normal' },
    { name: 'makosh-communications-ingress', kind: 'normal' },
    { name: 'makosh-communications-api', kind: 'normal' },
    { name: 'makosh-communications-domain', kind: 'normal' },
    { name: 'makosh-communications-persistence', kind: 'normal' },
    { name: 'makosh-communications-saved-query-api', kind: 'normal' },
    { name: 'makosh-events-jetstream', kind: 'normal' },
    { name: 'makosh-events-protocol', kind: 'normal' },
    { name: 'makosh-managed-vault-client', kind: 'normal' },
    { name: 'makosh-runtime-protocol', kind: 'normal' },
    { name: 'makosh-storage-protocol', kind: 'normal' },
    { name: 'makosh-storage-vault', kind: 'normal' },
  ],
};

const COMMUNICATIONS_SENDER_INSIGHTS_WORKSPACE_DEPENDENCY_ALLOWLIST = {
  ...COMMUNICATIONS_SAVED_SEARCH_WORKSPACE_DEPENDENCY_ALLOWLIST,
  'makosh-communications-sender-insights-api': [],
  'makosh-communications-runtime': [
    { name: 'makosh-blob-client', kind: 'normal' },
    { name: 'makosh-communications-attachment-contract', kind: 'normal' },
    { name: 'makosh-communications-content-api', kind: 'normal' },
    { name: 'makosh-communications-ingress', kind: 'normal' },
    { name: 'makosh-communications-api', kind: 'normal' },
    { name: 'makosh-communications-domain', kind: 'normal' },
    { name: 'makosh-communications-persistence', kind: 'normal' },
    { name: 'makosh-communications-saved-query-api', kind: 'normal' },
    { name: 'makosh-communications-sender-insights-api', kind: 'normal' },
    { name: 'makosh-events-jetstream', kind: 'normal' },
    { name: 'makosh-events-protocol', kind: 'normal' },
    { name: 'makosh-managed-vault-client', kind: 'normal' },
    { name: 'makosh-runtime-protocol', kind: 'normal' },
    { name: 'makosh-storage-protocol', kind: 'normal' },
    { name: 'makosh-storage-vault', kind: 'normal' },
  ],
};

const COMMUNICATIONS_EXPORT_WORKSPACE_DEPENDENCY_ALLOWLIST = {
  ...COMMUNICATIONS_SENDER_INSIGHTS_WORKSPACE_DEPENDENCY_ALLOWLIST,
  'makosh-communications-evidence-export-source-api': [
    { name: 'makosh-events-protocol', kind: 'normal' },
    { name: 'makosh-runtime-protocol', kind: 'normal' },
  ],
  'makosh-communications-export-api': [],
  'makosh-communications-export-core': [],
  'makosh-communications-export-persistence': [
    { name: 'makosh-communications-export-core', kind: 'normal' },
    { name: 'makosh-events-protocol', kind: 'normal' },
    { name: 'makosh-storage-protocol', kind: 'normal' },
  ],
  'makosh-communications-export-runtime': [
    { name: 'makosh-blob-client', kind: 'normal' },
    { name: 'makosh-communications-evidence-export-source-api', kind: 'normal' },
    { name: 'makosh-communications-export-api', kind: 'normal' },
    { name: 'makosh-communications-export-core', kind: 'normal' },
    { name: 'makosh-communications-export-persistence', kind: 'normal' },
    { name: 'makosh-events-jetstream', kind: 'normal' },
    { name: 'makosh-events-protocol', kind: 'normal' },
    { name: 'makosh-managed-vault-client', kind: 'normal' },
    { name: 'makosh-runtime-protocol', kind: 'normal' },
    { name: 'makosh-storage-protocol', kind: 'normal' },
    { name: 'makosh-storage-vault', kind: 'normal' },
  ],
  'makosh-communications-export-assembly': [
    { name: 'makosh-communications-export-persistence', kind: 'normal' },
    { name: 'makosh-communications-export-runtime', kind: 'normal' },
    { name: 'makosh-runtime-protocol', kind: 'normal' },
    { name: 'makosh-storage-protocol', kind: 'normal' },
  ],
  'makosh-communications-runtime': [
    { name: 'makosh-blob-client', kind: 'normal' },
    { name: 'makosh-communications-attachment-contract', kind: 'normal' },
    { name: 'makosh-communications-content-api', kind: 'normal' },
    { name: 'makosh-communications-evidence-export-source-api', kind: 'normal' },
    { name: 'makosh-communications-ingress', kind: 'normal' },
    { name: 'makosh-communications-api', kind: 'normal' },
    { name: 'makosh-communications-domain', kind: 'normal' },
    { name: 'makosh-communications-persistence', kind: 'normal' },
    { name: 'makosh-communications-saved-query-api', kind: 'normal' },
    { name: 'makosh-communications-sender-insights-api', kind: 'normal' },
    { name: 'makosh-events-jetstream', kind: 'normal' },
    { name: 'makosh-events-protocol', kind: 'normal' },
    { name: 'makosh-managed-vault-client', kind: 'normal' },
    { name: 'makosh-runtime-protocol', kind: 'normal' },
    { name: 'makosh-storage-protocol', kind: 'normal' },
    { name: 'makosh-storage-vault', kind: 'normal' },
  ],
};

const PROTOCOL_THIRD_PARTY_DEPENDENCIES = [
  {
    name: 'prost', kind: 'normal', source: 'crates_io', version: '=0.14.4', defaultFeatures: true, features: [],
  },
  {
    name: 'prost-types', kind: 'normal', source: 'crates_io', version: '=0.14.4', defaultFeatures: true, features: [],
  },
  {
    name: 'prost-build', kind: 'build', source: 'crates_io', version: '=0.14.4', defaultFeatures: true, features: [],
  },
  {
    name: 'protoc-bin-vendored', kind: 'build', source: 'crates_io', version: '=3.2.0', defaultFeatures: true, features: [],
  },
];

const RECOVERY_THIRD_PARTY_DEPENDENCY_ALLOWLIST = {
  'makosh-events-protocol': [
    ...PROTOCOL_THIRD_PARTY_DEPENDENCIES,
    { name: 'hpke', kind: 'normal', source: 'crates_io', version: '=0.14.0', defaultFeatures: false, features: ['alloc', 'chacha', 'getrandom', 'x25519'] },
    { name: 'nats-jwt', kind: 'normal', source: 'crates_io', version: '=0.3.0', defaultFeatures: true, features: [] },
    { name: 'sha2', kind: 'normal', source: 'crates_io', version: '=0.11.0', defaultFeatures: false, features: [] },
    { name: 'zeroize', kind: 'normal', source: 'crates_io', version: '=1.9.0', defaultFeatures: true, features: [] },
  ],
  'makosh-runtime-protocol': [
    { name: 'libc', kind: 'normal', source: 'crates_io', version: '=0.2.186', defaultFeatures: true, features: [] },
    ...PROTOCOL_THIRD_PARTY_DEPENDENCIES,
    { name: 'getrandom', kind: 'normal', source: 'crates_io', version: '=0.4.3', defaultFeatures: false, features: [] },
  ],
  'makosh-gateway-protocol': PROTOCOL_THIRD_PARTY_DEPENDENCIES,
  'makosh-kernel-control-store': [
    { name: 'sha2', kind: 'normal', source: 'crates_io', version: '=0.11.0', defaultFeatures: false, features: [] },
  ],
  'makosh-kernel-control-store-sqlite': [
    {
      name: 'rusqlite', kind: 'normal', source: 'crates_io', version: '=0.32.0', defaultFeatures: false, features: ['backup', 'bundled'],
    },
    {
      name: 'sha2', kind: 'normal', source: 'crates_io', version: '=0.11.0', defaultFeatures: false, features: [],
    },
  ],
  'makosh-kernel': [
    {
      name: 'clap', kind: 'normal', source: 'crates_io', version: '=4.6.2', defaultFeatures: false, features: ['derive', 'error-context', 'help', 'std', 'usage'],
    },
    {
      name: 'directories', kind: 'normal', source: 'crates_io', version: '=6.0.0', defaultFeatures: true, features: [],
    },
    {
      name: 'p256', kind: 'normal', source: 'crates_io', version: '=0.14.0', defaultFeatures: false, features: ['ecdsa'],
    },
    {
      name: 'getrandom', kind: 'normal', source: 'crates_io', version: '=0.4.3', defaultFeatures: false, features: [],
    },
    {
      name: 'libc', kind: 'normal', source: 'crates_io', version: '=0.2.186', defaultFeatures: true, features: [],
    },
    {
      name: 'prost', kind: 'normal', source: 'crates_io', version: '=0.14.4', defaultFeatures: true, features: [],
    },
    {
      name: 'rcgen', kind: 'normal', source: 'crates_io', version: '=0.13.2', defaultFeatures: true, features: [],
    },
    {
      name: 'rustls', kind: 'normal', source: 'crates_io', version: '=0.23.37', defaultFeatures: false, features: ['ring', 'std'],
    },
    {
      name: 'sha2', kind: 'normal', source: 'crates_io', version: '=0.11.0', defaultFeatures: false, features: [],
    },
    {
      name: 'signal-hook', kind: 'normal', source: 'crates_io', version: '=0.3.18', defaultFeatures: true, features: [],
    },
    {
      name: 'tracing', kind: 'normal', source: 'crates_io', version: '=0.1.44', defaultFeatures: true, features: [],
    },
    {
      name: 'tracing-subscriber', kind: 'normal', source: 'crates_io', version: '=0.3.20', defaultFeatures: true, features: [],
    },
  ],
  'makosh-secure-file': [
    { name: 'libc', kind: 'normal', source: 'crates_io', version: '=0.2.186', defaultFeatures: true, features: [] },
  ],
};

const VAULT_FOUNDATION_THIRD_PARTY_DEPENDENCY_ALLOWLIST = {
  ...RECOVERY_THIRD_PARTY_DEPENDENCY_ALLOWLIST,
  'makosh-vault-protocol': [
    { name: 'hpke', kind: 'normal', source: 'crates_io', version: '=0.14.0', defaultFeatures: false, features: ['alloc', 'chacha', 'getrandom', 'x25519'] },
    { name: 'sha2', kind: 'normal', source: 'crates_io', version: '=0.11.0', defaultFeatures: false, features: [] },
    { name: 'zeroize', kind: 'normal', source: 'crates_io', version: '=1.9.0', defaultFeatures: true, features: [] },
  ],
  'makosh-managed-vault-client': [
    { name: 'getrandom', kind: 'normal', source: 'crates_io', version: '=0.4.3', defaultFeatures: false, features: [] },
    { name: 'prost', kind: 'normal', source: 'crates_io', version: '=0.14.4', defaultFeatures: true, features: [] },
    { name: 'zeroize', kind: 'normal', source: 'crates_io', version: '=1.9.0', defaultFeatures: true, features: [] },
  ],
  'makosh-vault-key-provider': [],
  'makosh-vault-key-provider-file': [
    { name: 'getrandom', kind: 'normal', source: 'crates_io', version: '=0.4.3', defaultFeatures: false, features: [] },
    { name: 'libc', kind: 'normal', source: 'crates_io', version: '=0.2.186', defaultFeatures: true, features: [] },
  ],
  'makosh-vault-store-sqlcipher': [
    { name: 'bip39', kind: 'normal', source: 'crates_io', version: '=2.2.2', defaultFeatures: false, features: ['std'] },
    { name: 'chacha20poly1305', kind: 'normal', source: 'crates_io', version: '=0.11.0', defaultFeatures: false, features: ['alloc', 'zeroize'] },
    { name: 'getrandom', kind: 'normal', source: 'crates_io', version: '=0.4.3', defaultFeatures: false, features: [] },
    { name: 'hkdf', kind: 'normal', source: 'crates_io', version: '=0.13.0', defaultFeatures: true, features: [] },
    { name: 'libc', kind: 'normal', source: 'crates_io', version: '=0.2.186', defaultFeatures: true, features: [] },
    { name: 'rusqlite', kind: 'normal', source: 'crates_io', version: '=0.32.0', defaultFeatures: false, features: ['backup', 'bundled-sqlcipher'] },
    { name: 'sha2', kind: 'normal', source: 'crates_io', version: '=0.11.0', defaultFeatures: false, features: [] },
    { name: 'zeroize', kind: 'normal', source: 'crates_io', version: '=1.9.0', defaultFeatures: true, features: [] },
  ],
  'makosh-vault-runtime': [
    { name: 'clap', kind: 'normal', source: 'crates_io', version: '=4.6.2', defaultFeatures: false, features: ['derive', 'error-context', 'help', 'std', 'usage'] },
    { name: 'getrandom', kind: 'normal', source: 'crates_io', version: '=0.4.3', defaultFeatures: false, features: [] },
    { name: 'hpke', kind: 'normal', source: 'crates_io', version: '=0.14.0', defaultFeatures: false, features: ['alloc', 'chacha', 'getrandom', 'x25519'] },
    { name: 'libc', kind: 'normal', source: 'crates_io', version: '=0.2.186', defaultFeatures: true, features: [] },
    { name: 'p256', kind: 'normal', source: 'crates_io', version: '=0.14.0', defaultFeatures: false, features: ['ecdsa'] },
    { name: 'prost', kind: 'normal', source: 'crates_io', version: '=0.14.4', defaultFeatures: true, features: [] },
    { name: 'tracing', kind: 'normal', source: 'crates_io', version: '=0.1.44', defaultFeatures: true, features: [] },
    { name: 'tracing-subscriber', kind: 'normal', source: 'crates_io', version: '=0.3.20', defaultFeatures: true, features: [] },
    { name: 'zeroize', kind: 'normal', source: 'crates_io', version: '=1.9.0', defaultFeatures: true, features: [] },
  ],
};

const CLOCK_THIRD_PARTY_DEPENDENCY_ALLOWLIST = {
  ...VAULT_FOUNDATION_THIRD_PARTY_DEPENDENCY_ALLOWLIST,
  'makosh-clock-protocol': [],
  'makosh-clock-runtime': [],
};

const TELEMETRY_FOUNDATION_THIRD_PARTY_DEPENDENCY_ALLOWLIST = {
  ...CLOCK_THIRD_PARTY_DEPENDENCY_ALLOWLIST,
  'makosh-telemetry-protocol': [],
  'makosh-telemetry-collector': [
    { name: 'libc', kind: 'normal', source: 'crates_io', version: '=0.2.186', defaultFeatures: true, features: [] },
    { name: 'prost', kind: 'normal', source: 'crates_io', version: '=0.14.4', defaultFeatures: true, features: [] },
    { name: 'tracing', kind: 'normal', source: 'crates_io', version: '=0.1.44', defaultFeatures: true, features: [] },
    { name: 'tracing-subscriber', kind: 'normal', source: 'crates_io', version: '=0.3.20', defaultFeatures: true, features: [] },
  ],
};

const STORAGE_FOUNDATION_THIRD_PARTY_DEPENDENCY_ALLOWLIST = {
  ...TELEMETRY_FOUNDATION_THIRD_PARTY_DEPENDENCY_ALLOWLIST,
  'makosh-storage-protocol': [
    { name: 'prost', kind: 'normal', source: 'crates_io', version: '=0.14.4', defaultFeatures: true, features: [] },
    { name: 'sha2', kind: 'normal', source: 'crates_io', version: '=0.11.0', defaultFeatures: false, features: [] },
    { name: 'prost-build', kind: 'build', source: 'crates_io', version: '=0.14.4', defaultFeatures: true, features: [] },
    { name: 'protoc-bin-vendored', kind: 'build', source: 'crates_io', version: '=3.2.0', defaultFeatures: true, features: [] },
  ],
  'makosh-storage-control': [],
  'makosh-storage-vault': [
    { name: 'getrandom', kind: 'normal', source: 'crates_io', version: '=0.4.3', defaultFeatures: false, features: [] },
    { name: 'prost', kind: 'normal', source: 'crates_io', version: '=0.14.4', defaultFeatures: true, features: [] },
    { name: 'zeroize', kind: 'normal', source: 'crates_io', version: '=1.9.0', defaultFeatures: true, features: [] },
  ],
  'makosh-storage-runtime': [
    { name: 'libc', kind: 'normal', source: 'crates_io', version: '=0.2.186', defaultFeatures: true, features: [] },
    { name: 'prost', kind: 'normal', source: 'crates_io', version: '=0.14.4', defaultFeatures: true, features: [] },
    { name: 'sha2', kind: 'normal', source: 'crates_io', version: '=0.11.0', defaultFeatures: false, features: [] },
    { name: 'tokio', kind: 'normal', source: 'crates_io', version: '=1.52.4', defaultFeatures: false, features: ['net', 'rt', 'time'] },
    { name: 'tracing', kind: 'normal', source: 'crates_io', version: '=0.1.44', defaultFeatures: true, features: [] },
    { name: 'tracing-subscriber', kind: 'normal', source: 'crates_io', version: '=0.3.20', defaultFeatures: true, features: [] },
    { name: 'zeroize', kind: 'normal', source: 'crates_io', version: '=1.9.0', defaultFeatures: true, features: [] },
  ],
  'makosh-storage-postgres': [
    { name: 'getrandom', kind: 'normal', source: 'crates_io', version: '=0.4.3', defaultFeatures: false, features: [] },
    { name: 'libc', kind: 'normal', source: 'crates_io', version: '=0.2.186', defaultFeatures: true, features: [] },
    { name: 'sha2', kind: 'normal', source: 'crates_io', version: '=0.11.0', defaultFeatures: false, features: [] },
    { name: 'sqlx', kind: 'normal', source: 'crates_io', version: '=0.9.0', defaultFeatures: false, features: ['postgres', 'runtime-tokio', 'tls-rustls-ring'] },
    { name: 'zeroize', kind: 'normal', source: 'crates_io', version: '=1.9.0', defaultFeatures: true, features: [] },
  ],
  'makosh-storage-pgbouncer': [
    { name: 'libc', kind: 'normal', source: 'crates_io', version: '=0.2.186', defaultFeatures: true, features: [] },
    { name: 'tokio', kind: 'normal', source: 'crates_io', version: '=1.52.4', defaultFeatures: false, features: ['rt', 'time'] },
    { name: 'tokio-postgres', kind: 'normal', source: 'crates_io', version: '=0.7.18', defaultFeatures: false, features: ['runtime'] },
    { name: 'zeroize', kind: 'normal', source: 'crates_io', version: '=1.9.0', defaultFeatures: true, features: [] },
  ],
  'makosh-storage-migrations': [
    { name: 'pg_query', kind: 'normal', source: 'crates_io', version: '=6.1.1', defaultFeatures: true, features: [] },
  ],
};

const NATS_FOUNDATION_THIRD_PARTY_DEPENDENCY_ALLOWLIST = {
  ...STORAGE_FOUNDATION_THIRD_PARTY_DEPENDENCY_ALLOWLIST,
  'makosh-events-jetstream': [
    { name: 'async-nats', kind: 'normal', source: 'crates_io', version: '=0.49.1', defaultFeatures: true, features: [] },
    { name: 'base64', kind: 'normal', source: 'crates_io', version: '=0.22.1', defaultFeatures: true, features: [] },
    { name: 'futures-util', kind: 'normal', source: 'crates_io', version: '=0.3.32', defaultFeatures: true, features: [] },
    { name: 'getrandom', kind: 'normal', source: 'crates_io', version: '=0.4.3', defaultFeatures: false, features: [] },
    { name: 'hpke', kind: 'normal', source: 'crates_io', version: '=0.14.0', defaultFeatures: false, features: ['alloc', 'chacha', 'getrandom', 'x25519'] },
    { name: 'nats-jwt', kind: 'normal', source: 'crates_io', version: '=0.3.0', defaultFeatures: true, features: [] },
    { name: 'prost', kind: 'normal', source: 'crates_io', version: '=0.14.4', defaultFeatures: true, features: [] },
    { name: 'serde', kind: 'normal', source: 'crates_io', version: '=1.0.228', defaultFeatures: false, features: ['derive'] },
    { name: 'serde_json', kind: 'normal', source: 'crates_io', version: '=1.0.150', defaultFeatures: true, features: [] },
    { name: 'tokio', kind: 'normal', source: 'crates_io', version: '=1.52.4', defaultFeatures: false, features: ['rt-multi-thread', 'time'] },
    { name: 'zeroize', kind: 'normal', source: 'crates_io', version: '=1.9.0', defaultFeatures: true, features: [] },
  ],
  'makosh-events-authority': [
    { name: 'zeroize', kind: 'normal', source: 'crates_io', version: '=1.9.0', defaultFeatures: true, features: [] },
  ],
  'makosh-events-authority-runtime-control': [
    { name: 'prost', kind: 'normal', source: 'crates_io', version: '=0.14.4', defaultFeatures: true, features: [] },
    { name: 'tokio', kind: 'normal', source: 'crates_io', version: '=1.52.4', defaultFeatures: false, features: ['net', 'rt', 'time'] },
  ],
  'makosh-events-authority-runtime': [
    { name: 'libc', kind: 'normal', source: 'crates_io', version: '=0.2.186', defaultFeatures: true, features: [] },
    { name: 'prost', kind: 'normal', source: 'crates_io', version: '=0.14.4', defaultFeatures: true, features: [] },
  ],
};

const BLOB_FOUNDATION_THIRD_PARTY_DEPENDENCY_ALLOWLIST = {
  ...NATS_FOUNDATION_THIRD_PARTY_DEPENDENCY_ALLOWLIST,
  'makosh-blob-protocol': [],
};

const BLOB_RUNTIME_FOUNDATION_THIRD_PARTY_DEPENDENCY_ALLOWLIST = {
  ...BLOB_FOUNDATION_THIRD_PARTY_DEPENDENCY_ALLOWLIST,
  'makosh-blob-client-contract': [],
  'makosh-blob-client': [
    { name: 'prost', kind: 'normal', source: 'crates_io', version: '=0.14.4', defaultFeatures: true, features: [] },
  ],
  'makosh-blob-runtime': [
    { name: 'chacha20poly1305', kind: 'normal', source: 'crates_io', version: '=0.11.0', defaultFeatures: false, features: ['alloc', 'zeroize'] },
    { name: 'getrandom', kind: 'normal', source: 'crates_io', version: '=0.4.3', defaultFeatures: false, features: [] },
    { name: 'libc', kind: 'normal', source: 'crates_io', version: '=0.2.186', defaultFeatures: true, features: [] },
    { name: 'sha2', kind: 'normal', source: 'crates_io', version: '=0.11.0', defaultFeatures: false, features: [] },
    { name: 'zeroize', kind: 'normal', source: 'crates_io', version: '=1.9.0', defaultFeatures: true, features: [] },
  ],
  'makosh-blob-service': [
    { name: 'libc', kind: 'normal', source: 'crates_io', version: '=0.2.186', defaultFeatures: true, features: [] },
    { name: 'p256', kind: 'normal', source: 'crates_io', version: '=0.14.0', defaultFeatures: false, features: ['ecdsa'] },
    { name: 'prost', kind: 'normal', source: 'crates_io', version: '=0.14.4', defaultFeatures: true, features: [] },
    { name: 'sha2', kind: 'normal', source: 'crates_io', version: '=0.11.0', defaultFeatures: false, features: [] },
  ],
};

const SCHEDULER_PROTOCOL_FOUNDATION_THIRD_PARTY_DEPENDENCY_ALLOWLIST = {
  ...BLOB_RUNTIME_FOUNDATION_THIRD_PARTY_DEPENDENCY_ALLOWLIST,
  'makosh-scheduler-protocol': [
    { name: 'prost', kind: 'normal', source: 'crates_io', version: '=0.14.4', defaultFeatures: true, features: [] },
    { name: 'prost-build', kind: 'build', source: 'crates_io', version: '=0.14.4', defaultFeatures: true, features: [] },
    { name: 'protoc-bin-vendored', kind: 'build', source: 'crates_io', version: '=3.2.0', defaultFeatures: true, features: [] },
  ],
};

const SCHEDULER_FOUNDATION_THIRD_PARTY_DEPENDENCY_ALLOWLIST = {
  ...SCHEDULER_PROTOCOL_FOUNDATION_THIRD_PARTY_DEPENDENCY_ALLOWLIST,
  'makosh-scheduler': [
    { name: 'prost', kind: 'normal', source: 'crates_io', version: '=0.14.4', defaultFeatures: true, features: [] },
    { name: 'prost-types', kind: 'normal', source: 'crates_io', version: '=0.14.4', defaultFeatures: true, features: [] },
  ],
};

const SCHEDULER_PERSISTENCE_FOUNDATION_THIRD_PARTY_DEPENDENCY_ALLOWLIST = {
  ...SCHEDULER_FOUNDATION_THIRD_PARTY_DEPENDENCY_ALLOWLIST,
  'makosh-scheduler-persistence': [
    { name: 'prost', kind: 'normal', source: 'crates_io', version: '=0.14.4', defaultFeatures: true, features: [] },
    { name: 'sha2', kind: 'normal', source: 'crates_io', version: '=0.11.0', defaultFeatures: false, features: [] },
    { name: 'sqlx', kind: 'normal', source: 'crates_io', version: '=0.9.0', defaultFeatures: false, features: ['postgres', 'runtime-tokio', 'tls-rustls-ring'] },
  ],
};

const GATEWAY_SESSION_FOUNDATION_THIRD_PARTY_DEPENDENCY_ALLOWLIST = {
  ...SCHEDULER_PERSISTENCE_FOUNDATION_THIRD_PARTY_DEPENDENCY_ALLOWLIST,
  'makosh-gateway-session-contract': [],
  'makosh-gateway-session': [
    { name: 'getrandom', kind: 'normal', source: 'crates_io', version: '=0.4.3', defaultFeatures: false, features: [] },
    { name: 'p256', kind: 'normal', source: 'crates_io', version: '=0.14.0', defaultFeatures: false, features: ['ecdsa'] },
    { name: 'serde_cbor_2', kind: 'normal', source: 'crates_io', version: '=0.13.0', defaultFeatures: true, features: [] },
    { name: 'url', kind: 'normal', source: 'crates_io', version: '=2.5.8', defaultFeatures: true, features: [] },
    { name: 'webauthn-rs-core', kind: 'normal', source: 'crates_io', version: '=0.5.5', defaultFeatures: true, features: [] },
  ],
};

const SCHEDULER_RECEIPT_DELIVERY_FOUNDATION_THIRD_PARTY_DEPENDENCY_ALLOWLIST = {
  ...GATEWAY_SESSION_FOUNDATION_THIRD_PARTY_DEPENDENCY_ALLOWLIST,
};

const SCHEDULER_JETSTREAM_FOUNDATION_THIRD_PARTY_DEPENDENCY_ALLOWLIST = {
  ...SCHEDULER_RECEIPT_DELIVERY_FOUNDATION_THIRD_PARTY_DEPENDENCY_ALLOWLIST,
  'makosh-scheduler-jetstream': [
    { name: 'async-nats', kind: 'normal', source: 'crates_io', version: '=0.49.1', defaultFeatures: true, features: [] },
    { name: 'futures-util', kind: 'normal', source: 'crates_io', version: '=0.3.32', defaultFeatures: true, features: [] },
    { name: 'getrandom', kind: 'normal', source: 'crates_io', version: '=0.4.3', defaultFeatures: false, features: [] },
    { name: 'nats-jwt', kind: 'normal', source: 'crates_io', version: '=0.3.0', defaultFeatures: true, features: [] },
    { name: 'prost', kind: 'normal', source: 'crates_io', version: '=0.14.4', defaultFeatures: true, features: [] },
    { name: 'tokio', kind: 'normal', source: 'crates_io', version: '=1.52.4', defaultFeatures: false, features: ['time'] },
  ],
};

const SCHEDULER_RUNTIME_FOUNDATION_THIRD_PARTY_DEPENDENCY_ALLOWLIST = {
  ...SCHEDULER_JETSTREAM_FOUNDATION_THIRD_PARTY_DEPENDENCY_ALLOWLIST,
  'makosh-scheduler-runtime': [
    { name: 'libc', kind: 'normal', source: 'crates_io', version: '=0.2.186', defaultFeatures: true, features: [] },
    { name: 'prost', kind: 'normal', source: 'crates_io', version: '=0.14.4', defaultFeatures: true, features: [] },
    { name: 'tokio', kind: 'normal', source: 'crates_io', version: '=1.52.4', defaultFeatures: false, features: ['net', 'rt-multi-thread', 'time'] },
    { name: 'zeroize', kind: 'normal', source: 'crates_io', version: '=1.9.0', defaultFeatures: true, features: [] },
  ],
};

const GATEWAY_RUNTIME_FOUNDATION_THIRD_PARTY_DEPENDENCY_ALLOWLIST = {
  ...SCHEDULER_RUNTIME_FOUNDATION_THIRD_PARTY_DEPENDENCY_ALLOWLIST,
  'makosh-gateway-protocol': PROTOCOL_THIRD_PARTY_DEPENDENCIES,
  'makosh-kernel': [
    ...SCHEDULER_RUNTIME_FOUNDATION_THIRD_PARTY_DEPENDENCY_ALLOWLIST['makosh-kernel'],
    { name: 'chacha20poly1305', kind: 'normal', source: 'crates_io', version: '=0.11.0', defaultFeatures: false, features: ['alloc', 'zeroize'] },
    { name: 'quinn', kind: 'normal', source: 'crates_io', version: '=0.11.7', defaultFeatures: true, features: [] },
    { name: 'tokio', kind: 'normal', source: 'crates_io', version: '=1.52.4', defaultFeatures: false, features: ['net', 'rt-multi-thread', 'sync', 'time'] },
    { name: 'tokio-rustls', kind: 'normal', source: 'crates_io', version: '=0.26.4', defaultFeatures: true, features: [] },
    { name: 'zeroize', kind: 'normal', source: 'crates_io', version: '=1.9.0', defaultFeatures: true, features: [] },
  ],
  'makosh-gateway-runtime': [
    { name: 'base64', kind: 'normal', source: 'crates_io', version: '=0.22.1', defaultFeatures: true, features: [] },
    { name: 'bytes', kind: 'normal', source: 'crates_io', version: '=1.12.1', defaultFeatures: true, features: [] },
    { name: 'futures-util', kind: 'normal', source: 'crates_io', version: '=0.3.32', defaultFeatures: true, features: [] },
    { name: 'h3', kind: 'normal', source: 'crates_io', version: '=0.0.8', defaultFeatures: true, features: [] },
    { name: 'h3-quinn', kind: 'normal', source: 'crates_io', version: '=0.0.10', defaultFeatures: true, features: [] },
    { name: 'http-body-util', kind: 'normal', source: 'crates_io', version: '=0.1.3', defaultFeatures: true, features: [] },
    { name: 'hyper', kind: 'normal', source: 'crates_io', version: '=1.10.1', defaultFeatures: false, features: ['http1', 'http2', 'server'] },
    { name: 'hyper-util', kind: 'normal', source: 'crates_io', version: '=0.1.20', defaultFeatures: false, features: ['tokio'] },
    { name: 'prost', kind: 'normal', source: 'crates_io', version: '=0.14.4', defaultFeatures: true, features: [] },
    { name: 'quinn', kind: 'normal', source: 'crates_io', version: '=0.11.7', defaultFeatures: true, features: [] },
    { name: 'serde', kind: 'normal', source: 'crates_io', version: '=1.0.228', defaultFeatures: true, features: ['derive'] },
    { name: 'serde_json', kind: 'normal', source: 'crates_io', version: '=1.0.150', defaultFeatures: true, features: [] },
    { name: 'tokio', kind: 'normal', source: 'crates_io', version: '=1.52.4', defaultFeatures: false, features: ['io-util', 'macros', 'net', 'rt', 'sync'] },
    { name: 'tokio-rustls', kind: 'normal', source: 'crates_io', version: '=0.26.4', defaultFeatures: true, features: [] },
    { name: 'tracing', kind: 'normal', source: 'crates_io', version: '=0.1.44', defaultFeatures: true, features: [] },
    { name: 'webauthn-rs-core', kind: 'normal', source: 'crates_io', version: '=0.5.5', defaultFeatures: true, features: [] },
  ],
};

const MAIL_COMMUNICATIONS_FOUNDATION_THIRD_PARTY_DEPENDENCY_ALLOWLIST = {
  ...GATEWAY_RUNTIME_FOUNDATION_THIRD_PARTY_DEPENDENCY_ALLOWLIST,
  'makosh-blob-client-contract': [],
  'makosh-blob-client': [
    { name: 'getrandom', kind: 'normal', source: 'crates_io', version: '=0.3.4', defaultFeatures: true, features: [] },
    { name: 'prost', kind: 'normal', source: 'crates_io', version: '=0.14.4', defaultFeatures: true, features: [] },
    { name: 'sha2', kind: 'normal', source: 'crates_io', version: '=0.10.9', defaultFeatures: true, features: [] },
  ],
  'makosh-mail-api': [
    { name: 'prost', kind: 'normal', source: 'crates_io', version: '=0.14.4', defaultFeatures: true, features: [] },
    { name: 'prost-build', kind: 'build', source: 'crates_io', version: '=0.14.4', defaultFeatures: true, features: [] },
    { name: 'protoc-bin-vendored', kind: 'build', source: 'crates_io', version: '=3.2.0', defaultFeatures: true, features: [] },
  ],
  'makosh-mail-core': [
    { name: 'base64', kind: 'normal', source: 'crates_io', version: '=0.22.1', defaultFeatures: true, features: [] },
    { name: 'sha2', kind: 'normal', source: 'crates_io', version: '=0.11.0', defaultFeatures: false, features: [] },
  ],
  'makosh-mail-imap': [
    { name: 'async-imap', kind: 'normal', source: 'crates_io', version: '=0.11.2', defaultFeatures: true, features: [] },
    { name: 'async-native-tls', kind: 'normal', source: 'crates_io', version: '=0.6.0', defaultFeatures: true, features: [] },
    { name: 'async-std', kind: 'normal', source: 'crates_io', version: '=1.13.2', defaultFeatures: true, features: [] },
    { name: 'futures-util', kind: 'normal', source: 'crates_io', version: '=0.3.32', defaultFeatures: true, features: [] },
    { name: 'imap-proto', kind: 'normal', source: 'crates_io', version: '=0.16.7', defaultFeatures: true, features: [] },
  ],
  'makosh-mail-gmail': [
    { name: 'async-native-tls', kind: 'normal', source: 'crates_io', version: '=0.6.0', defaultFeatures: true, features: [] },
    { name: 'async-std', kind: 'normal', source: 'crates_io', version: '=1.13.2', defaultFeatures: true, features: [] },
    { name: 'base64', kind: 'normal', source: 'crates_io', version: '=0.22.1', defaultFeatures: true, features: [] },
    { name: 'futures-util', kind: 'normal', source: 'crates_io', version: '=0.3.32', defaultFeatures: true, features: [] },
    { name: 'serde', kind: 'normal', source: 'crates_io', version: '=1.0.228', defaultFeatures: true, features: ['derive'] },
    { name: 'serde_json', kind: 'normal', source: 'crates_io', version: '=1.0.150', defaultFeatures: true, features: [] },
  ],
  'makosh-mail-smtp': [
    { name: 'async-native-tls', kind: 'normal', source: 'crates_io', version: '=0.6.0', defaultFeatures: true, features: [] },
    { name: 'async-std', kind: 'normal', source: 'crates_io', version: '=1.13.2', defaultFeatures: true, features: [] },
  ],
  'makosh-mail-persistence': [
    { name: 'sha2', kind: 'normal', source: 'crates_io', version: '=0.11.0', defaultFeatures: false, features: [] },
    { name: 'sqlx', kind: 'normal', source: 'crates_io', version: '=0.9.0', defaultFeatures: false, features: ['postgres', 'runtime-tokio', 'tls-rustls-ring'] },
    { name: 'zeroize', kind: 'normal', source: 'crates_io', version: '=1.9.0', defaultFeatures: true, features: [] },
  ],
  'makosh-mail-runtime': [
    { name: 'getrandom', kind: 'normal', source: 'crates_io', version: '=0.4.3', defaultFeatures: false, features: [] },
    { name: 'libc', kind: 'normal', source: 'crates_io', version: '=0.2.186', defaultFeatures: true, features: [] },
    { name: 'prost', kind: 'normal', source: 'crates_io', version: '=0.14.4', defaultFeatures: true, features: [] },
    { name: 'sha2', kind: 'normal', source: 'crates_io', version: '=0.11.0', defaultFeatures: false, features: [] },
    { name: 'tokio', kind: 'normal', source: 'crates_io', version: '=1.52.4', defaultFeatures: false, features: ['rt-multi-thread', 'time'] },
    { name: 'zeroize', kind: 'normal', source: 'crates_io', version: '=1.9.0', defaultFeatures: true, features: [] },
  ],
  'makosh-mail-assembly': [
    { name: 'prost', kind: 'normal', source: 'crates_io', version: '=0.14.4', defaultFeatures: true, features: [] },
    { name: 'serde', kind: 'normal', source: 'crates_io', version: '=1.0.228', defaultFeatures: false, features: ['derive', 'std'] },
    { name: 'serde_json', kind: 'normal', source: 'crates_io', version: '=1.0.150', defaultFeatures: true, features: [] },
  ],
  'makosh-telegram-api': [
    { name: 'serde', kind: 'normal', source: 'crates_io', version: '=1.0.228', defaultFeatures: false, features: ['derive'] },
    { name: 'prost', kind: 'normal', source: 'crates_io', version: '=0.14.4', defaultFeatures: true, features: [] },
    { name: 'prost-build', kind: 'build', source: 'crates_io', version: '=0.14.4', defaultFeatures: true, features: [] },
    { name: 'protoc-bin-vendored', kind: 'build', source: 'crates_io', version: '=3.2.0', defaultFeatures: true, features: [] },
  ],
  'makosh-telegram-core': [
    { name: 'serde_json', kind: 'normal', source: 'crates_io', version: '=1.0.150', defaultFeatures: true, features: [] },
    { name: 'sha2', kind: 'normal', source: 'crates_io', version: '=0.10.9', defaultFeatures: true, features: [] },
  ],
  'makosh-telegram-tdlib': [
    { name: 'base64', kind: 'normal', source: 'crates_io', version: '=0.22.1', defaultFeatures: true, features: [] },
    { name: 'libloading', kind: 'normal', source: 'crates_io', version: '=0.8.9', defaultFeatures: true, features: [] },
    { name: 'serde_json', kind: 'normal', source: 'crates_io', version: '=1.0.150', defaultFeatures: true, features: [] },
    { name: 'zeroize', kind: 'normal', source: 'crates_io', version: '=1.9.0', defaultFeatures: true, features: [] },
  ],
  'makosh-telegram-persistence': [
    { name: 'serde_json', kind: 'normal', source: 'crates_io', version: '=1.0.150', defaultFeatures: true, features: [] },
    { name: 'sha2', kind: 'normal', source: 'crates_io', version: '=0.10.9', defaultFeatures: false, features: [] },
    { name: 'sqlx', kind: 'normal', source: 'crates_io', version: '=0.9.0', defaultFeatures: false, features: ['json', 'postgres', 'runtime-tokio', 'tls-rustls-ring'] },
  ],
  'makosh-telegram-runtime': [
    { name: 'getrandom', kind: 'normal', source: 'crates_io', version: '=0.4.3', defaultFeatures: false, features: [] },
    { name: 'libc', kind: 'normal', source: 'crates_io', version: '=0.2.186', defaultFeatures: true, features: [] },
    { name: 'prost', kind: 'normal', source: 'crates_io', version: '=0.14.4', defaultFeatures: true, features: [] },
    { name: 'serde_json', kind: 'normal', source: 'crates_io', version: '=1.0.150', defaultFeatures: true, features: [] },
    { name: 'sha2', kind: 'normal', source: 'crates_io', version: '=0.10.9', defaultFeatures: false, features: [] },
    { name: 'tokio', kind: 'normal', source: 'crates_io', version: '=1.52.4', defaultFeatures: false, features: ['rt', 'rt-multi-thread', 'time'] },
    { name: 'zeroize', kind: 'normal', source: 'crates_io', version: '=1.9.0', defaultFeatures: true, features: [] },
  ],
  'makosh-telegram-assembly': [
    { name: 'prost', kind: 'normal', source: 'crates_io', version: '=0.14.4', defaultFeatures: true, features: [] },
    { name: 'serde', kind: 'normal', source: 'crates_io', version: '=1.0.228', defaultFeatures: false, features: ['derive', 'std'] },
    { name: 'serde_json', kind: 'normal', source: 'crates_io', version: '=1.0.150', defaultFeatures: true, features: [] },
  ],
  'makosh-whatsapp-api': [
    { name: 'prost', kind: 'normal', source: 'crates_io', version: '=0.14.4', defaultFeatures: true, features: [] },
    { name: 'prost-build', kind: 'build', source: 'crates_io', version: '=0.14.4', defaultFeatures: true, features: [] },
    { name: 'protoc-bin-vendored', kind: 'build', source: 'crates_io', version: '=3.2.0', defaultFeatures: true, features: [] },
    { name: 'serde', kind: 'normal', source: 'crates_io', version: '=1.0.228', defaultFeatures: false, features: ['alloc', 'derive'] },
  ],
  'makosh-whatsapp-core': [],
  'makosh-whatsapp-persistence': [
    { name: 'sha2', kind: 'normal', source: 'crates_io', version: '=0.11.0', defaultFeatures: false, features: [] },
    { name: 'sqlx', kind: 'normal', source: 'crates_io', version: '=0.9.0', defaultFeatures: false, features: ['postgres', 'runtime-tokio', 'tls-rustls-ring'] },
  ],
  'makosh-whatsapp-runtime': [
    { name: 'libc', kind: 'normal', source: 'crates_io', version: '=0.2.186', defaultFeatures: true, features: [] },
    { name: 'prost', kind: 'normal', source: 'crates_io', version: '=0.14.4', defaultFeatures: true, features: [] },
    { name: 'sha2', kind: 'normal', source: 'crates_io', version: '=0.11.0', defaultFeatures: false, features: [] },
    { name: 'tokio', kind: 'normal', source: 'crates_io', version: '=1.52.4', defaultFeatures: false, features: ['rt-multi-thread'] },
  ],
  'makosh-whatsapp-assembly': [
    { name: 'prost', kind: 'normal', source: 'crates_io', version: '=0.14.4', defaultFeatures: true, features: [] },
    { name: 'serde', kind: 'normal', source: 'crates_io', version: '=1.0.228', defaultFeatures: false, features: ['derive', 'std'] },
    { name: 'serde_json', kind: 'normal', source: 'crates_io', version: '=1.0.150', defaultFeatures: true, features: [] },
  ],
  'makosh-zulip-api': [
    { name: 'prost', kind: 'normal', source: 'crates_io', version: '=0.14.4', defaultFeatures: true, features: [] },
    { name: 'prost-build', kind: 'build', source: 'crates_io', version: '=0.14.4', defaultFeatures: true, features: [] },
    { name: 'protoc-bin-vendored', kind: 'build', source: 'crates_io', version: '=3.2.0', defaultFeatures: true, features: [] },
  ],
  'makosh-zulip-core': [],
  'makosh-zulip-http': [
    { name: 'async-native-tls', kind: 'normal', source: 'crates_io', version: '=0.6.0', defaultFeatures: true, features: [] },
    { name: 'async-std', kind: 'normal', source: 'crates_io', version: '=1.13.2', defaultFeatures: true, features: [] },
    { name: 'serde_json', kind: 'normal', source: 'crates_io', version: '=1.0.150', defaultFeatures: true, features: [] },
    { name: 'zeroize', kind: 'normal', source: 'crates_io', version: '=1.9.0', defaultFeatures: true, features: [] },
  ],
  'makosh-zulip-persistence': [{ name: 'sqlx', kind: 'normal', source: 'crates_io', version: '=0.9.0', defaultFeatures: false, features: ['postgres', 'runtime-tokio', 'tls-rustls-ring'] }],
  'makosh-zulip-runtime': [
    { name: 'getrandom', kind: 'normal', source: 'crates_io', version: '=0.4.3', defaultFeatures: false, features: [] },
    { name: 'libc', kind: 'normal', source: 'crates_io', version: '=0.2.186', defaultFeatures: true, features: [] },
    { name: 'prost', kind: 'normal', source: 'crates_io', version: '=0.14.4', defaultFeatures: true, features: [] },
    { name: 'sha2', kind: 'normal', source: 'crates_io', version: '=0.11.0', defaultFeatures: false, features: [] },
    { name: 'tokio', kind: 'normal', source: 'crates_io', version: '=1.52.4', defaultFeatures: false, features: ['rt-multi-thread', 'time'] },
    { name: 'zeroize', kind: 'normal', source: 'crates_io', version: '=1.9.0', defaultFeatures: true, features: [] },
  ],
  'makosh-communications-ingress': [
    { name: 'prost', kind: 'normal', source: 'crates_io', version: '=0.14.4', defaultFeatures: true, features: [] },
    { name: 'prost-types', kind: 'normal', source: 'crates_io', version: '=0.14.4', defaultFeatures: true, features: [] },
    { name: 'sha2', kind: 'normal', source: 'crates_io', version: '=0.11.0', defaultFeatures: false, features: [] },
    { name: 'prost-build', kind: 'build', source: 'crates_io', version: '=0.14.4', defaultFeatures: true, features: [] },
    { name: 'protoc-bin-vendored', kind: 'build', source: 'crates_io', version: '=3.2.0', defaultFeatures: true, features: [] },
    { name: 'sha2', kind: 'build', source: 'crates_io', version: '=0.11.0', defaultFeatures: false, features: [] },
  ],
  'makosh-communications-attachment-contract': [
    { name: 'prost', kind: 'normal', source: 'crates_io', version: '=0.14.4', defaultFeatures: true, features: [] },
    { name: 'prost-types', kind: 'normal', source: 'crates_io', version: '=0.14.4', defaultFeatures: true, features: [] },
    { name: 'sha2', kind: 'normal', source: 'crates_io', version: '=0.11.0', defaultFeatures: false, features: [] },
    { name: 'prost-build', kind: 'build', source: 'crates_io', version: '=0.14.4', defaultFeatures: true, features: [] },
    { name: 'protoc-bin-vendored', kind: 'build', source: 'crates_io', version: '=3.2.0', defaultFeatures: true, features: [] },
    { name: 'sha2', kind: 'build', source: 'crates_io', version: '=0.11.0', defaultFeatures: false, features: [] },
  ],
  'makosh-communications-api': [
    { name: 'prost', kind: 'normal', source: 'crates_io', version: '=0.14.4', defaultFeatures: true, features: [] },
    { name: 'prost-build', kind: 'build', source: 'crates_io', version: '=0.14.4', defaultFeatures: true, features: [] },
    { name: 'protoc-bin-vendored', kind: 'build', source: 'crates_io', version: '=3.2.0', defaultFeatures: true, features: [] },
    { name: 'sha2', kind: 'build', source: 'crates_io', version: '=0.11.0', defaultFeatures: false, features: [] },
  ],
  'makosh-communications-domain': [
    { name: 'sha2', kind: 'normal', source: 'crates_io', version: '=0.11.0', defaultFeatures: false, features: [] },
  ],
  'makosh-communications-persistence': [
    { name: 'sha2', kind: 'normal', source: 'crates_io', version: '=0.11.0', defaultFeatures: false, features: [] },
    { name: 'sqlx', kind: 'normal', source: 'crates_io', version: '=0.9.0', defaultFeatures: false, features: ['postgres', 'runtime-tokio', 'tls-rustls-ring'] },
  ],
  'makosh-communications-runtime': [
    { name: 'libc', kind: 'normal', source: 'crates_io', version: '=0.2.186', defaultFeatures: true, features: [] },
    { name: 'prost', kind: 'normal', source: 'crates_io', version: '=0.14.4', defaultFeatures: true, features: [] },
    { name: 'prost-types', kind: 'normal', source: 'crates_io', version: '=0.14.4', defaultFeatures: true, features: [] },
    { name: 'sha2', kind: 'normal', source: 'crates_io', version: '=0.11.0', defaultFeatures: false, features: [] },
    { name: 'tokio', kind: 'normal', source: 'crates_io', version: '=1.52.4', defaultFeatures: false, features: ['rt', 'rt-multi-thread', 'time'] },
    { name: 'zeroize', kind: 'normal', source: 'crates_io', version: '=1.9.0', defaultFeatures: true, features: [] },
  ],
  'makosh-communications-assembly': [
    { name: 'prost', kind: 'normal', source: 'crates_io', version: '=0.14.4', defaultFeatures: true, features: [] },
    { name: 'serde', kind: 'normal', source: 'crates_io', version: '=1.0.228', defaultFeatures: false, features: ['derive', 'std'] },
    { name: 'serde_json', kind: 'normal', source: 'crates_io', version: '=1.0.150', defaultFeatures: true, features: [] },
  ],
};

const FIRST_OWNER_THIRD_PARTY_DEPENDENCY_ALLOWLIST = Object.fromEntries(
  FIRST_OWNER_PRODUCTION_PACKAGES.map(({ name }) => [
    name,
    MAIL_COMMUNICATIONS_FOUNDATION_THIRD_PARTY_DEPENDENCY_ALLOWLIST[name],
  ]),
);

const ATTACHMENT_SECURITY_ENGINE_THIRD_PARTY_DEPENDENCY_ALLOWLIST = {
  ...FIRST_OWNER_THIRD_PARTY_DEPENDENCY_ALLOWLIST,
  'makosh-attachment-security-contract': [
    { name: 'prost', kind: 'normal', source: 'crates_io', version: '=0.14.4', defaultFeatures: true, features: [] },
    { name: 'prost-types', kind: 'normal', source: 'crates_io', version: '=0.14.4', defaultFeatures: true, features: [] },
    { name: 'sha2', kind: 'normal', source: 'crates_io', version: '=0.11.0', defaultFeatures: false, features: [] },
    { name: 'prost-build', kind: 'build', source: 'crates_io', version: '=0.14.4', defaultFeatures: true, features: [] },
    { name: 'protoc-bin-vendored', kind: 'build', source: 'crates_io', version: '=3.2.0', defaultFeatures: true, features: [] },
    { name: 'sha2', kind: 'build', source: 'crates_io', version: '=0.11.0', defaultFeatures: false, features: [] },
  ],
  'makosh-attachment-security-core': [
    { name: 'sha2', kind: 'normal', source: 'crates_io', version: '=0.11.0', defaultFeatures: false, features: [] },
  ],
  'makosh-attachment-security-clamav': [
    { name: 'sha2', kind: 'normal', source: 'crates_io', version: '=0.11.0', defaultFeatures: false, features: [] },
  ],
  'makosh-attachment-security-persistence': [
    { name: 'prost', kind: 'normal', source: 'crates_io', version: '=0.14.4', defaultFeatures: true, features: [] },
    { name: 'sha2', kind: 'normal', source: 'crates_io', version: '=0.11.0', defaultFeatures: false, features: [] },
    { name: 'sqlx', kind: 'normal', source: 'crates_io', version: '=0.9.0', defaultFeatures: false, features: ['postgres', 'runtime-tokio', 'tls-rustls-ring'] },
  ],
  'makosh-attachment-security-runtime': [
    { name: 'libc', kind: 'normal', source: 'crates_io', version: '=0.2.186', defaultFeatures: true, features: [] },
    { name: 'prost', kind: 'normal', source: 'crates_io', version: '=0.14.4', defaultFeatures: true, features: [] },
    { name: 'prost-types', kind: 'normal', source: 'crates_io', version: '=0.14.4', defaultFeatures: true, features: [] },
    { name: 'sha2', kind: 'normal', source: 'crates_io', version: '=0.11.0', defaultFeatures: false, features: [] },
    { name: 'tokio', kind: 'normal', source: 'crates_io', version: '=1.52.4', defaultFeatures: false, features: ['rt-multi-thread', 'time'] },
    { name: 'zeroize', kind: 'normal', source: 'crates_io', version: '=1.9.0', defaultFeatures: true, features: [] },
  ],
  'makosh-attachment-security-assembly': [
    { name: 'prost', kind: 'normal', source: 'crates_io', version: '=0.14.4', defaultFeatures: true, features: [] },
    { name: 'serde', kind: 'normal', source: 'crates_io', version: '=1.0.228', defaultFeatures: false, features: ['derive', 'std'] },
    { name: 'serde_json', kind: 'normal', source: 'crates_io', version: '=1.0.150', defaultFeatures: true, features: [] },
  ],
};

const MAIL_OUTBOUND_MIME_ATTACHMENTS_THIRD_PARTY_DEPENDENCY_ALLOWLIST = {
  ...ATTACHMENT_SECURITY_ENGINE_THIRD_PARTY_DEPENDENCY_ALLOWLIST,
  ...Object.fromEntries(
    MAIL_OUTBOUND_MIME_ATTACHMENTS_PRODUCTION_PACKAGES
      .filter(({ owner }) => owner === 'mail')
      .map(({ name }) => [
        name,
        MAIL_COMMUNICATIONS_FOUNDATION_THIRD_PARTY_DEPENDENCY_ALLOWLIST[name],
      ]),
  ),
};

const COMMUNICATIONS_CONTENT_READ_THIRD_PARTY_DEPENDENCY_ALLOWLIST = {
  ...MAIL_OUTBOUND_MIME_ATTACHMENTS_THIRD_PARTY_DEPENDENCY_ALLOWLIST,
  'makosh-communications-content-api': [
    { name: 'prost', kind: 'normal', source: 'crates_io', version: '=0.14.4', defaultFeatures: true, features: [] },
    { name: 'prost-build', kind: 'build', source: 'crates_io', version: '=0.14.4', defaultFeatures: true, features: [] },
    { name: 'protoc-bin-vendored', kind: 'build', source: 'crates_io', version: '=3.2.0', defaultFeatures: true, features: [] },
    { name: 'sha2', kind: 'build', source: 'crates_io', version: '=0.11.0', defaultFeatures: false, features: [] },
  ],
  'makosh-communications-runtime': [
    { name: 'getrandom', kind: 'normal', source: 'crates_io', version: '=0.4.3', defaultFeatures: true, features: [] },
    ...MAIL_COMMUNICATIONS_FOUNDATION_THIRD_PARTY_DEPENDENCY_ALLOWLIST[
      'makosh-communications-runtime'
    ],
  ],
};

const COMMUNICATIONS_SAVED_SEARCH_THIRD_PARTY_DEPENDENCY_ALLOWLIST = {
  ...COMMUNICATIONS_CONTENT_READ_THIRD_PARTY_DEPENDENCY_ALLOWLIST,
  'makosh-communications-saved-query-api': [
    { name: 'prost', kind: 'normal', source: 'crates_io', version: '=0.14.4', defaultFeatures: true, features: [] },
    { name: 'prost-build', kind: 'build', source: 'crates_io', version: '=0.14.4', defaultFeatures: true, features: [] },
    { name: 'protoc-bin-vendored', kind: 'build', source: 'crates_io', version: '=3.2.0', defaultFeatures: true, features: [] },
    { name: 'sha2', kind: 'build', source: 'crates_io', version: '=0.11.0', defaultFeatures: false, features: [] },
  ],
};

const COMMUNICATIONS_SENDER_INSIGHTS_THIRD_PARTY_DEPENDENCY_ALLOWLIST = {
  ...COMMUNICATIONS_SAVED_SEARCH_THIRD_PARTY_DEPENDENCY_ALLOWLIST,
  'makosh-communications-sender-insights-api': [
    { name: 'prost', kind: 'normal', source: 'crates_io', version: '=0.14.4', defaultFeatures: true, features: [] },
    { name: 'prost-build', kind: 'build', source: 'crates_io', version: '=0.14.4', defaultFeatures: true, features: [] },
    { name: 'protoc-bin-vendored', kind: 'build', source: 'crates_io', version: '=3.2.0', defaultFeatures: true, features: [] },
    { name: 'sha2', kind: 'build', source: 'crates_io', version: '=0.11.0', defaultFeatures: false, features: [] },
  ],
};

const COMMUNICATION_DELIVERY_INTENT_CONTRACT_CORE_WORKSPACE_DEPENDENCY_ALLOWLIST = {
  ...COMMUNICATIONS_EXPORT_WORKSPACE_DEPENDENCY_ALLOWLIST,
  'makosh-communication-delivery-intent-api': [],
  'makosh-communication-delivery-intent-core': [
    { name: 'makosh-communications-api', kind: 'normal' },
  ],
};

const COMMUNICATION_DELIVERY_INTENT_PERSISTENCE_WORKSPACE_DEPENDENCY_ALLOWLIST = {
  ...COMMUNICATION_DELIVERY_INTENT_CONTRACT_CORE_WORKSPACE_DEPENDENCY_ALLOWLIST,
  'makosh-communication-delivery-intent-persistence': [
    { name: 'makosh-communication-delivery-intent-core', kind: 'normal' },
    { name: 'makosh-events-protocol', kind: 'normal' },
    { name: 'makosh-storage-protocol', kind: 'normal' },
  ],
};

const COMMUNICATION_DELIVERY_INTENT_RUNTIME_WORKSPACE_DEPENDENCY_ALLOWLIST = {
  ...COMMUNICATION_DELIVERY_INTENT_PERSISTENCE_WORKSPACE_DEPENDENCY_ALLOWLIST,
  'makosh-communication-delivery-intent-runtime': [
    { name: 'makosh-communication-delivery-intent-api', kind: 'normal' },
    { name: 'makosh-communication-delivery-intent-core', kind: 'normal' },
    { name: 'makosh-communication-delivery-intent-event-adapters', kind: 'normal' },
    { name: 'makosh-communication-delivery-intent-persistence', kind: 'normal' },
    { name: 'makosh-communications-api', kind: 'normal' },
    { name: 'makosh-events-jetstream', kind: 'normal' },
    { name: 'makosh-runtime-protocol', kind: 'normal' },
    { name: 'makosh-storage-protocol', kind: 'normal' },
    { name: 'makosh-storage-vault', kind: 'normal' },
  ],
};

const COMMUNICATION_DELIVERY_INTENT_ASSEMBLY_WORKSPACE_DEPENDENCY_ALLOWLIST = {
  ...COMMUNICATION_DELIVERY_INTENT_RUNTIME_WORKSPACE_DEPENDENCY_ALLOWLIST,
  'makosh-communication-delivery-intent-assembly': [
    { name: 'makosh-communication-delivery-intent-persistence', kind: 'normal' },
    { name: 'makosh-communication-delivery-intent-runtime', kind: 'normal' },
    { name: 'makosh-runtime-protocol', kind: 'normal' },
    { name: 'makosh-storage-protocol', kind: 'normal' },
  ],
};

const DELIVERY_INTENT_TRANSACTIONAL_EVENT_ADAPTERS_WORKSPACE_DEPENDENCY_ALLOWLIST = {
  ...COMMUNICATION_DELIVERY_INTENT_ASSEMBLY_WORKSPACE_DEPENDENCY_ALLOWLIST,
  'makosh-mail-delivery-intent-contract': [
    { name: 'makosh-runtime-protocol', kind: 'normal' },
  ],
  'makosh-telegram-delivery-intent-contract': [
    { name: 'makosh-runtime-protocol', kind: 'normal' },
  ],
  'makosh-whatsapp-delivery-intent-contract': [
    { name: 'makosh-runtime-protocol', kind: 'normal' },
  ],
  'makosh-zulip-delivery-intent-contract': [
    { name: 'makosh-runtime-protocol', kind: 'normal' },
  ],
  'makosh-communication-delivery-intent-event-adapters': [
    { name: 'makosh-events-protocol', kind: 'normal' },
    { name: 'makosh-mail-delivery-intent-contract', kind: 'normal' },
    { name: 'makosh-runtime-protocol', kind: 'normal' },
    { name: 'makosh-telegram-delivery-intent-contract', kind: 'normal' },
    { name: 'makosh-whatsapp-delivery-intent-contract', kind: 'normal' },
    { name: 'makosh-zulip-delivery-intent-contract', kind: 'normal' },
  ],
};

const DELIVERY_INTENT_TARGET_BOUND_BLOB_WORKSPACE_DEPENDENCY_ALLOWLIST = {
  ...DELIVERY_INTENT_TRANSACTIONAL_EVENT_ADAPTERS_WORKSPACE_DEPENDENCY_ALLOWLIST,
  'makosh-mail-runtime': [
    { name: 'makosh-mail-api', kind: 'normal' },
    { name: 'makosh-mail-core', kind: 'normal' },
    { name: 'makosh-mail-imap', kind: 'normal' },
    { name: 'makosh-mail-gmail', kind: 'normal' },
    { name: 'makosh-mail-smtp', kind: 'normal' },
    { name: 'makosh-mail-persistence', kind: 'normal' },
    { name: 'makosh-mail-delivery-intent-contract', kind: 'normal' },
    { name: 'makosh-attachment-security-contract', kind: 'normal' },
    { name: 'makosh-communications-attachment-contract', kind: 'normal' },
    { name: 'makosh-communications-ingress', kind: 'normal' },
    { name: 'makosh-events-protocol', kind: 'normal' },
    { name: 'makosh-events-jetstream', kind: 'normal' },
    { name: 'makosh-blob-client', kind: 'normal' },
    { name: 'makosh-managed-vault-client', kind: 'normal' },
    { name: 'makosh-runtime-protocol', kind: 'normal' },
    { name: 'makosh-storage-protocol', kind: 'normal' },
    { name: 'makosh-storage-vault', kind: 'normal' },
    { name: 'makosh-vault-protocol', kind: 'normal' },
  ],
  'makosh-communication-delivery-intent-runtime': [
    { name: 'makosh-blob-client', kind: 'normal' },
    { name: 'makosh-communication-delivery-intent-api', kind: 'normal' },
    { name: 'makosh-communication-delivery-intent-core', kind: 'normal' },
    { name: 'makosh-communication-delivery-intent-event-adapters', kind: 'normal' },
    { name: 'makosh-communication-delivery-intent-persistence', kind: 'normal' },
    { name: 'makosh-communications-api', kind: 'normal' },
    { name: 'makosh-events-jetstream', kind: 'normal' },
    { name: 'makosh-mail-delivery-intent-contract', kind: 'normal' },
    { name: 'makosh-runtime-protocol', kind: 'normal' },
    { name: 'makosh-storage-protocol', kind: 'normal' },
    { name: 'makosh-storage-vault', kind: 'normal' },
    { name: 'makosh-telegram-delivery-intent-contract', kind: 'normal' },
    { name: 'makosh-whatsapp-delivery-intent-contract', kind: 'normal' },
    { name: 'makosh-zulip-delivery-intent-contract', kind: 'normal' },
  ],
};

const COMMUNICATION_BULK_ACTION_CONTRACT_CORE_WORKSPACE_DEPENDENCY_ALLOWLIST = {
  ...DELIVERY_INTENT_TARGET_BOUND_BLOB_WORKSPACE_DEPENDENCY_ALLOWLIST,
  'makosh-communication-bulk-action-api': [],
  'makosh-communication-bulk-action-core': [],
};

const COMMUNICATION_BULK_ACTION_PERSISTENCE_WORKSPACE_DEPENDENCY_ALLOWLIST = {
  ...COMMUNICATION_BULK_ACTION_CONTRACT_CORE_WORKSPACE_DEPENDENCY_ALLOWLIST,
  'makosh-communication-bulk-action-persistence': [
    { name: 'makosh-communication-bulk-action-core', kind: 'normal' },
    { name: 'makosh-storage-protocol', kind: 'normal' },
  ],
};

const COMMUNICATION_BULK_ACTION_RUNTIME_CORE_WORKSPACE_DEPENDENCY_ALLOWLIST = {
  ...COMMUNICATION_BULK_ACTION_PERSISTENCE_WORKSPACE_DEPENDENCY_ALLOWLIST,
  'makosh-communication-bulk-action-runtime': [
    { name: 'makosh-communication-bulk-action-api', kind: 'normal' },
    { name: 'makosh-communication-bulk-action-core', kind: 'normal' },
    { name: 'makosh-communication-bulk-action-persistence', kind: 'normal' },
    { name: 'makosh-communication-delivery-intent-api', kind: 'normal' },
    { name: 'makosh-runtime-protocol', kind: 'normal' },
    { name: 'makosh-storage-protocol', kind: 'normal' },
    { name: 'makosh-storage-vault', kind: 'normal' },
  ],
};

const COMMUNICATION_BULK_ACTION_ASSEMBLY_WORKSPACE_DEPENDENCY_ALLOWLIST = {
  ...COMMUNICATION_BULK_ACTION_RUNTIME_CORE_WORKSPACE_DEPENDENCY_ALLOWLIST,
  'makosh-communication-bulk-action-assembly': [
    { name: 'makosh-communication-bulk-action-persistence', kind: 'normal' },
    { name: 'makosh-communication-bulk-action-runtime', kind: 'normal' },
    { name: 'makosh-runtime-protocol', kind: 'normal' },
    { name: 'makosh-storage-protocol', kind: 'normal' },
  ],
};

const COMMUNICATION_DELAYED_DELIVERY_CONTRACT_CORE_WORKSPACE_DEPENDENCY_ALLOWLIST = {
  ...COMMUNICATION_BULK_ACTION_ASSEMBLY_WORKSPACE_DEPENDENCY_ALLOWLIST,
  'makosh-communication-delayed-delivery-api': [],
  'makosh-communication-delayed-delivery-core': [],
};

const COMMUNICATION_DELAYED_DELIVERY_PERSISTENCE_WORKSPACE_DEPENDENCY_ALLOWLIST = {
  ...COMMUNICATION_DELAYED_DELIVERY_CONTRACT_CORE_WORKSPACE_DEPENDENCY_ALLOWLIST,
  'makosh-communication-delayed-delivery-persistence': [
    { name: 'makosh-communication-delayed-delivery-core', kind: 'normal' },
    { name: 'makosh-storage-protocol', kind: 'normal' },
  ],
};

const COMMUNICATION_DELAYED_DELIVERY_EXECUTION_WORKSPACE_DEPENDENCY_ALLOWLIST = {
  ...COMMUNICATION_DELAYED_DELIVERY_PERSISTENCE_WORKSPACE_DEPENDENCY_ALLOWLIST,
  'makosh-communication-delayed-delivery-execution': [
    { name: 'makosh-communication-delivery-intent-api', kind: 'normal' },
  ],
};

const COMMUNICATION_DELAYED_DELIVERY_EVENT_ADAPTERS_WORKSPACE_DEPENDENCY_ALLOWLIST = {
  ...COMMUNICATION_DELAYED_DELIVERY_EXECUTION_WORKSPACE_DEPENDENCY_ALLOWLIST,
  'makosh-communication-delayed-delivery-event-adapters': [
    { name: 'makosh-communication-delayed-delivery-api', kind: 'normal' },
    { name: 'makosh-events-protocol', kind: 'normal' },
    { name: 'makosh-scheduler-protocol', kind: 'normal' },
  ],
};

const COMMUNICATION_DELAYED_DELIVERY_RUNTIME_ADAPTERS_WORKSPACE_DEPENDENCY_ALLOWLIST = {
  ...COMMUNICATION_DELAYED_DELIVERY_EVENT_ADAPTERS_WORKSPACE_DEPENDENCY_ALLOWLIST,
  'makosh-communication-delayed-delivery-runtime-adapters': [
    { name: 'makosh-blob-client', kind: 'normal' },
    { name: 'makosh-communication-delayed-delivery-api', kind: 'normal' },
    { name: 'makosh-communication-delayed-delivery-execution', kind: 'normal' },
    { name: 'makosh-communication-delivery-intent-api', kind: 'normal' },
    { name: 'makosh-runtime-protocol', kind: 'normal' },
  ],
};

const COMMUNICATION_DELAYED_DELIVERY_STORE_ADAPTERS_WORKSPACE_DEPENDENCY_ALLOWLIST = {
  ...COMMUNICATION_DELAYED_DELIVERY_RUNTIME_ADAPTERS_WORKSPACE_DEPENDENCY_ALLOWLIST,
  'makosh-communication-delayed-delivery-store-adapters': [
    { name: 'makosh-communication-delayed-delivery-execution', kind: 'normal' },
    { name: 'makosh-communication-delayed-delivery-persistence', kind: 'normal' },
  ],
};

const COMMUNICATION_DELAYED_DELIVERY_MANAGED_RUNTIME_WORKSPACE_DEPENDENCY_ALLOWLIST = {
  ...COMMUNICATION_DELAYED_DELIVERY_STORE_ADAPTERS_WORKSPACE_DEPENDENCY_ALLOWLIST,
  'makosh-communication-delayed-delivery-runtime': [
    { name: 'makosh-communication-delayed-delivery-api', kind: 'normal' },
    { name: 'makosh-communication-delayed-delivery-core', kind: 'normal' },
    { name: 'makosh-communication-delayed-delivery-event-adapters', kind: 'normal' },
    { name: 'makosh-communication-delayed-delivery-execution', kind: 'normal' },
    { name: 'makosh-communication-delayed-delivery-persistence', kind: 'normal' },
    { name: 'makosh-communication-delayed-delivery-runtime-adapters', kind: 'normal' },
    { name: 'makosh-communication-delayed-delivery-store-adapters', kind: 'normal' },
    { name: 'makosh-communication-delivery-intent-api', kind: 'normal' },
    { name: 'makosh-events-jetstream', kind: 'normal' },
    { name: 'makosh-runtime-protocol', kind: 'normal' },
    { name: 'makosh-scheduler-protocol', kind: 'normal' },
    { name: 'makosh-storage-protocol', kind: 'normal' },
    { name: 'makosh-storage-vault', kind: 'normal' },
  ],
};

const COMMUNICATION_DELAYED_DELIVERY_ASSEMBLY_WORKSPACE_DEPENDENCY_ALLOWLIST = {
  ...COMMUNICATION_DELAYED_DELIVERY_MANAGED_RUNTIME_WORKSPACE_DEPENDENCY_ALLOWLIST,
  'makosh-communication-delayed-delivery-assembly': [
    { name: 'makosh-communication-delayed-delivery-persistence', kind: 'normal' },
    { name: 'makosh-communication-delayed-delivery-runtime', kind: 'normal' },
    { name: 'makosh-runtime-protocol', kind: 'normal' },
    { name: 'makosh-storage-protocol', kind: 'normal' },
  ],
};

const COMMUNICATION_CROSS_CHANNEL_FORWARD_CONTRACT_CORE_WORKSPACE_DEPENDENCY_ALLOWLIST = {
  ...COMMUNICATION_DELAYED_DELIVERY_ASSEMBLY_WORKSPACE_DEPENDENCY_ALLOWLIST,
  'makosh-communication-cross-channel-forward-api': [],
  'makosh-communication-cross-channel-forward-core': [],
};

const COMMUNICATION_CROSS_CHANNEL_FORWARD_PERSISTENCE_WORKSPACE_DEPENDENCY_ALLOWLIST = {
  ...COMMUNICATION_CROSS_CHANNEL_FORWARD_CONTRACT_CORE_WORKSPACE_DEPENDENCY_ALLOWLIST,
  'makosh-communication-cross-channel-forward-persistence': [
    { name: 'makosh-communication-cross-channel-forward-core', kind: 'normal' },
    { name: 'makosh-storage-protocol', kind: 'normal' },
  ],
};

const COMMUNICATION_CROSS_CHANNEL_FORWARD_SOURCE_CONTRACT_WORKSPACE_DEPENDENCY_ALLOWLIST = {
  ...COMMUNICATION_CROSS_CHANNEL_FORWARD_PERSISTENCE_WORKSPACE_DEPENDENCY_ALLOWLIST,
  'makosh-communications-runtime':
    COMMUNICATION_CROSS_CHANNEL_FORWARD_PERSISTENCE_WORKSPACE_DEPENDENCY_ALLOWLIST[
      'makosh-communications-runtime'
    ].flatMap((dependency) => (
      dependency.name === 'makosh-communications-evidence-export-source-api'
        ? [
            dependency,
            {
              name: 'makosh-communications-cross-channel-forward-source-api',
              kind: 'normal',
            },
          ]
        : [dependency]
    )),
  'makosh-communications-cross-channel-forward-source-api': [
    { name: 'makosh-events-protocol', kind: 'normal' },
    { name: 'makosh-runtime-protocol', kind: 'normal' },
  ],
};

const COMMUNICATION_DELIVERY_INTENT_INGRESS_CONTRACT_WORKSPACE_DEPENDENCY_ALLOWLIST = {
  ...COMMUNICATION_CROSS_CHANNEL_FORWARD_SOURCE_CONTRACT_WORKSPACE_DEPENDENCY_ALLOWLIST,
  'makosh-communication-delivery-intent-ingress-api': [
    { name: 'makosh-events-protocol', kind: 'normal' },
    { name: 'makosh-runtime-protocol', kind: 'normal' },
  ],
};

const COMMUNICATION_CROSS_CHANNEL_FORWARD_EVENT_PERSISTENCE_WORKSPACE_DEPENDENCY_ALLOWLIST = {
  ...COMMUNICATION_DELIVERY_INTENT_INGRESS_CONTRACT_WORKSPACE_DEPENDENCY_ALLOWLIST,
  'makosh-communication-cross-channel-forward-persistence': [
    { name: 'makosh-communication-cross-channel-forward-core', kind: 'normal' },
    { name: 'makosh-events-protocol', kind: 'normal' },
    { name: 'makosh-storage-protocol', kind: 'normal' },
  ],
};

const COMMUNICATION_CROSS_CHANNEL_FORWARD_MANAGED_RUNTIME_WORKSPACE_DEPENDENCY_ALLOWLIST = {
  ...COMMUNICATION_CROSS_CHANNEL_FORWARD_EVENT_PERSISTENCE_WORKSPACE_DEPENDENCY_ALLOWLIST,
  'makosh-communication-cross-channel-forward-runtime': [
    { name: 'makosh-blob-client', kind: 'normal' },
    { name: 'makosh-communication-cross-channel-forward-api', kind: 'normal' },
    { name: 'makosh-communication-cross-channel-forward-core', kind: 'normal' },
    { name: 'makosh-communication-cross-channel-forward-persistence', kind: 'normal' },
    { name: 'makosh-communication-delivery-intent-ingress-api', kind: 'normal' },
    { name: 'makosh-communications-cross-channel-forward-source-api', kind: 'normal' },
    { name: 'makosh-events-jetstream', kind: 'normal' },
    { name: 'makosh-events-protocol', kind: 'normal' },
    { name: 'makosh-runtime-protocol', kind: 'normal' },
    { name: 'makosh-storage-protocol', kind: 'normal' },
    { name: 'makosh-storage-vault', kind: 'normal' },
  ],
};

const COMMUNICATION_DELIVERY_INTENT_EVENT_INGRESS_CONSUMER_WORKSPACE_DEPENDENCY_ALLOWLIST = {
  ...COMMUNICATION_CROSS_CHANNEL_FORWARD_MANAGED_RUNTIME_WORKSPACE_DEPENDENCY_ALLOWLIST,
  'makosh-communication-delivery-intent-runtime': [
    { name: 'makosh-blob-client', kind: 'normal' },
    { name: 'makosh-communication-delivery-intent-api', kind: 'normal' },
    { name: 'makosh-communication-delivery-intent-core', kind: 'normal' },
    { name: 'makosh-communication-delivery-intent-event-adapters', kind: 'normal' },
    { name: 'makosh-communication-delivery-intent-ingress-api', kind: 'normal' },
    { name: 'makosh-communication-delivery-intent-persistence', kind: 'normal' },
    { name: 'makosh-communications-api', kind: 'normal' },
    { name: 'makosh-events-jetstream', kind: 'normal' },
    { name: 'makosh-events-protocol', kind: 'normal' },
    { name: 'makosh-mail-delivery-intent-contract', kind: 'normal' },
    { name: 'makosh-runtime-protocol', kind: 'normal' },
    { name: 'makosh-storage-protocol', kind: 'normal' },
    { name: 'makosh-storage-vault', kind: 'normal' },
    { name: 'makosh-telegram-delivery-intent-contract', kind: 'normal' },
    { name: 'makosh-whatsapp-delivery-intent-contract', kind: 'normal' },
    { name: 'makosh-zulip-delivery-intent-contract', kind: 'normal' },
  ],
};

const COMMUNICATION_CROSS_CHANNEL_FORWARD_CLIENT_ASSEMBLY_WORKSPACE_DEPENDENCY_ALLOWLIST = {
  ...COMMUNICATION_DELIVERY_INTENT_EVENT_INGRESS_CONSUMER_WORKSPACE_DEPENDENCY_ALLOWLIST,
  'makosh-communication-cross-channel-forward-assembly': [
    { name: 'makosh-communication-cross-channel-forward-persistence', kind: 'normal' },
    { name: 'makosh-communication-cross-channel-forward-runtime', kind: 'normal' },
    { name: 'makosh-runtime-protocol', kind: 'normal' },
    { name: 'makosh-storage-protocol', kind: 'normal' },
  ],
};

const COMMUNICATIONS_CALL_EVIDENCE_CONTRACT_CORE_WORKSPACE_DEPENDENCY_ALLOWLIST = {
  ...COMMUNICATION_CROSS_CHANNEL_FORWARD_CLIENT_ASSEMBLY_WORKSPACE_DEPENDENCY_ALLOWLIST,
  'makosh-communications-call-evidence-ingress': [
    { name: 'makosh-events-protocol', kind: 'normal' },
    { name: 'makosh-runtime-protocol', kind: 'normal' },
  ],
  'makosh-communications-call-evidence-core': [
    { name: 'makosh-communications-call-evidence-ingress', kind: 'normal' },
  ],
};

const COMMUNICATIONS_CALL_EVIDENCE_PERSISTENCE_WORKSPACE_DEPENDENCY_ALLOWLIST = {
  ...COMMUNICATIONS_CALL_EVIDENCE_CONTRACT_CORE_WORKSPACE_DEPENDENCY_ALLOWLIST,
  'makosh-communications-call-evidence-persistence': [
    { name: 'makosh-communications-call-evidence-core', kind: 'normal' },
    { name: 'makosh-storage-protocol', kind: 'normal' },
  ],
};

const COMMUNICATIONS_CALL_EVIDENCE_MANAGED_CONSUMER_WORKSPACE_DEPENDENCY_ALLOWLIST = {
  ...COMMUNICATIONS_CALL_EVIDENCE_PERSISTENCE_WORKSPACE_DEPENDENCY_ALLOWLIST,
  'makosh-communications-runtime':
    COMMUNICATIONS_CALL_EVIDENCE_PERSISTENCE_WORKSPACE_DEPENDENCY_ALLOWLIST[
      'makosh-communications-runtime'
    ].flatMap((dependency) => (
      dependency.name === 'makosh-communications-attachment-contract'
        ? [
            dependency,
            { name: 'makosh-communications-call-evidence-core', kind: 'normal' },
            { name: 'makosh-communications-call-evidence-ingress', kind: 'normal' },
            { name: 'makosh-communications-call-evidence-persistence', kind: 'normal' },
          ]
        : [dependency]
    )),
  'makosh-communications-assembly':
    COMMUNICATIONS_CALL_EVIDENCE_PERSISTENCE_WORKSPACE_DEPENDENCY_ALLOWLIST[
      'makosh-communications-assembly'
    ].filter((dependency) => dependency.name !== 'makosh-communications-persistence'),
};

const COMMUNICATIONS_CALL_EVIDENCE_QUERY_REALTIME_WORKSPACE_DEPENDENCY_ALLOWLIST = {
  ...COMMUNICATIONS_CALL_EVIDENCE_MANAGED_CONSUMER_WORKSPACE_DEPENDENCY_ALLOWLIST,
  'makosh-communications-call-evidence-api': [],
  'makosh-communications-runtime':
    COMMUNICATIONS_CALL_EVIDENCE_MANAGED_CONSUMER_WORKSPACE_DEPENDENCY_ALLOWLIST[
      'makosh-communications-runtime'
    ].flatMap((dependency) => (
      dependency.name === 'makosh-communications-call-evidence-core'
        ? [
            { name: 'makosh-communications-call-evidence-api', kind: 'normal' },
            dependency,
          ]
        : [dependency]
    )),
};

const REVIEW_COMMUNICATIONS_ATTENTION_CONTRACT_CORE_WORKSPACE_DEPENDENCY_ALLOWLIST = {
  ...COMMUNICATIONS_CALL_EVIDENCE_QUERY_REALTIME_WORKSPACE_DEPENDENCY_ALLOWLIST,
  'makosh-review-attention-api': [],
  'makosh-review-attention-core': [],
};

const REVIEW_COMMUNICATIONS_ATTENTION_PERSISTENCE_WORKSPACE_DEPENDENCY_ALLOWLIST = {
  ...REVIEW_COMMUNICATIONS_ATTENTION_CONTRACT_CORE_WORKSPACE_DEPENDENCY_ALLOWLIST,
  'makosh-review-attention-persistence': [
    { name: 'makosh-review-attention-core', kind: 'normal' },
    { name: 'makosh-storage-protocol', kind: 'normal' },
  ],
};

const REVIEW_COMMUNICATIONS_ATTENTION_MANAGED_RUNTIME_WORKSPACE_DEPENDENCY_ALLOWLIST = {
  ...REVIEW_COMMUNICATIONS_ATTENTION_PERSISTENCE_WORKSPACE_DEPENDENCY_ALLOWLIST,
  'makosh-review-attention-runtime': [
    { name: 'makosh-review-attention-api', kind: 'normal' },
    { name: 'makosh-review-attention-core', kind: 'normal' },
    { name: 'makosh-review-attention-persistence', kind: 'normal' },
    { name: 'makosh-runtime-protocol', kind: 'normal' },
    { name: 'makosh-storage-protocol', kind: 'normal' },
    { name: 'makosh-storage-vault', kind: 'normal' },
  ],
};

const REVIEW_COMMUNICATIONS_ATTENTION_ASSEMBLY_WORKSPACE_DEPENDENCY_ALLOWLIST = {
  ...REVIEW_COMMUNICATIONS_ATTENTION_MANAGED_RUNTIME_WORKSPACE_DEPENDENCY_ALLOWLIST,
  'makosh-review-attention-assembly': [
    { name: 'makosh-review-attention-persistence', kind: 'normal' },
    { name: 'makosh-review-attention-runtime', kind: 'normal' },
    { name: 'makosh-runtime-protocol', kind: 'normal' },
    { name: 'makosh-storage-protocol', kind: 'normal' },
  ],
};

const COMMUNICATIONS_AI_SOURCE_CONTRACT_WORKSPACE_DEPENDENCY_ALLOWLIST = {
  ...REVIEW_COMMUNICATIONS_ATTENTION_ASSEMBLY_WORKSPACE_DEPENDENCY_ALLOWLIST,
  'makosh-communications-runtime':
    REVIEW_COMMUNICATIONS_ATTENTION_ASSEMBLY_WORKSPACE_DEPENDENCY_ALLOWLIST[
      'makosh-communications-runtime'
    ].flatMap((dependency) => (
      dependency.name === 'makosh-communications-attachment-contract'
        ? [
            { name: 'makosh-communications-ai-source-api', kind: 'normal' },
            dependency,
          ]
        : [dependency]
    )),
  'makosh-communications-ai-source-api': [
    { name: 'makosh-events-protocol', kind: 'normal' },
    { name: 'makosh-runtime-protocol', kind: 'normal' },
  ],
  'makosh-communication-reply-suggestion-api': [],
  'makosh-communication-reply-suggestion-core': [],
  'makosh-communication-reply-suggestion-persistence': [
    { name: 'makosh-communication-reply-suggestion-core', kind: 'normal' },
    { name: 'makosh-storage-protocol', kind: 'normal' },
  ],
  'makosh-communication-reply-suggestion-runtime': [
    { name: 'makosh-ai-contracts', kind: 'normal' },
    { name: 'makosh-blob-client', kind: 'normal' },
    { name: 'makosh-communication-reply-suggestion-api', kind: 'normal' },
    { name: 'makosh-communication-reply-suggestion-core', kind: 'normal' },
    { name: 'makosh-communication-reply-suggestion-persistence', kind: 'normal' },
    { name: 'makosh-communications-ai-source-api', kind: 'normal' },
    { name: 'makosh-events-jetstream', kind: 'normal' },
    { name: 'makosh-events-protocol', kind: 'normal' },
    { name: 'makosh-runtime-protocol', kind: 'normal' },
    { name: 'makosh-storage-protocol', kind: 'normal' },
    { name: 'makosh-storage-vault', kind: 'normal' },
  ],
  'makosh-communication-reply-suggestion-assembly': [
    { name: 'makosh-communication-reply-suggestion-persistence', kind: 'normal' },
    { name: 'makosh-communication-reply-suggestion-runtime', kind: 'normal' },
    { name: 'makosh-runtime-protocol', kind: 'normal' },
    { name: 'makosh-storage-protocol', kind: 'normal' },
  ],
  'makosh-ai-contracts': [
    { name: 'makosh-runtime-protocol', kind: 'normal' },
  ],
  'makosh-ai-inference-core': [
    { name: 'makosh-ai-contracts', kind: 'normal' },
  ],
  'makosh-ai-inference-persistence': [
    { name: 'makosh-ai-contracts', kind: 'normal' },
    { name: 'makosh-ai-inference-core', kind: 'normal' },
    { name: 'makosh-storage-protocol', kind: 'normal' },
  ],
  'makosh-ollama-ai-api': [
    { name: 'makosh-runtime-protocol', kind: 'normal' },
  ],
  'makosh-ollama-ai-assembly': [
    { name: 'makosh-ollama-ai-api', kind: 'normal' },
    { name: 'makosh-ollama-ai-persistence', kind: 'normal' },
    { name: 'makosh-ollama-ai-runtime', kind: 'normal' },
    { name: 'makosh-runtime-protocol', kind: 'normal' },
    { name: 'makosh-storage-protocol', kind: 'normal' },
  ],
  'makosh-ollama-ai-core': [
    { name: 'makosh-ai-contracts', kind: 'normal' },
    { name: 'makosh-ollama-ai-api', kind: 'normal' },
  ],
  'makosh-ollama-ai-http': [
    { name: 'makosh-ollama-ai-api', kind: 'normal' },
    { name: 'makosh-ollama-ai-core', kind: 'normal' },
  ],
  'makosh-ollama-ai-persistence': [
    { name: 'makosh-ai-contracts', kind: 'normal' },
    { name: 'makosh-ollama-ai-core', kind: 'normal' },
    { name: 'makosh-storage-protocol', kind: 'normal' },
  ],
  'makosh-ollama-ai-runtime': [
    { name: 'makosh-ai-contracts', kind: 'normal' },
    { name: 'makosh-ollama-ai-api', kind: 'normal' },
    { name: 'makosh-ollama-ai-core', kind: 'normal' },
    { name: 'makosh-ollama-ai-http', kind: 'normal' },
    { name: 'makosh-ollama-ai-persistence', kind: 'normal' },
    { name: 'makosh-runtime-protocol', kind: 'normal' },
    { name: 'makosh-storage-protocol', kind: 'normal' },
    { name: 'makosh-storage-vault', kind: 'normal' },
  ],
};

const ATTACHMENT_ARCHIVE_INSPECTION_CONTRACT_CORE_WORKSPACE_DEPENDENCY_ALLOWLIST = {
  ...COMMUNICATIONS_AI_SOURCE_CONTRACT_WORKSPACE_DEPENDENCY_ALLOWLIST,
  'makosh-attachment-archive-inspection-api': [],
  'makosh-attachment-archive-inspection-ingress': [
    { name: 'makosh-events-protocol', kind: 'normal' },
    { name: 'makosh-runtime-protocol', kind: 'normal' },
  ],
  'makosh-attachment-archive-inspection-core': [
    { name: 'makosh-attachment-archive-inspection-api', kind: 'normal' },
  ],
  'makosh-attachment-archive-inspection-zip': [
    { name: 'makosh-attachment-archive-inspection-core', kind: 'normal' },
  ],
};

const ATTACHMENT_ARCHIVE_INSPECTION_PERSISTENCE_WORKSPACE_DEPENDENCY_ALLOWLIST = {
  ...ATTACHMENT_ARCHIVE_INSPECTION_CONTRACT_CORE_WORKSPACE_DEPENDENCY_ALLOWLIST,
  'makosh-attachment-archive-inspection-persistence': [
    { name: 'makosh-attachment-archive-inspection-core', kind: 'normal' },
    { name: 'makosh-attachment-archive-inspection-ingress', kind: 'normal' },
    { name: 'makosh-events-protocol', kind: 'normal' },
    { name: 'makosh-storage-protocol', kind: 'normal' },
  ],
};

const ATTACHMENT_ARCHIVE_INSPECTION_RUNTIME_WORKSPACE_DEPENDENCY_ALLOWLIST = {
  ...ATTACHMENT_ARCHIVE_INSPECTION_PERSISTENCE_WORKSPACE_DEPENDENCY_ALLOWLIST,
  'makosh-attachment-archive-inspection-runtime': [
    { name: 'makosh-attachment-archive-inspection-api', kind: 'normal' },
    { name: 'makosh-attachment-archive-inspection-core', kind: 'normal' },
    { name: 'makosh-attachment-archive-inspection-ingress', kind: 'normal' },
    { name: 'makosh-attachment-archive-inspection-persistence', kind: 'normal' },
    { name: 'makosh-attachment-archive-inspection-zip', kind: 'normal' },
    { name: 'makosh-attachment-security-contract', kind: 'normal' },
    { name: 'makosh-blob-client', kind: 'normal' },
    { name: 'makosh-communications-attachment-contract', kind: 'normal' },
    { name: 'makosh-events-jetstream', kind: 'normal' },
    { name: 'makosh-events-protocol', kind: 'normal' },
    { name: 'makosh-runtime-protocol', kind: 'normal' },
    { name: 'makosh-storage-protocol', kind: 'normal' },
    { name: 'makosh-storage-vault', kind: 'normal' },
  ],
};

const ATTACHMENT_ARCHIVE_INSPECTION_ASSEMBLY_WORKSPACE_DEPENDENCY_ALLOWLIST = {
  ...ATTACHMENT_ARCHIVE_INSPECTION_RUNTIME_WORKSPACE_DEPENDENCY_ALLOWLIST,
  'makosh-attachment-archive-inspection-assembly': [
    { name: 'makosh-attachment-archive-inspection-api', kind: 'normal' },
    { name: 'makosh-attachment-archive-inspection-persistence', kind: 'normal' },
    { name: 'makosh-attachment-archive-inspection-runtime', kind: 'normal' },
    { name: 'makosh-runtime-protocol', kind: 'normal' },
    { name: 'makosh-storage-protocol', kind: 'normal' },
  ],
};

const COMMUNICATION_SUMMARY_BUILD_UNITS_WORKSPACE_DEPENDENCY_ALLOWLIST = {
  ...ATTACHMENT_ARCHIVE_INSPECTION_ASSEMBLY_WORKSPACE_DEPENDENCY_ALLOWLIST,
  'makosh-communication-summary-api': [],
  'makosh-communication-summary-core': [],
  'makosh-communication-summary-persistence': [
    { name: 'makosh-communication-summary-core', kind: 'normal' },
    { name: 'makosh-storage-protocol', kind: 'normal' },
  ],
  'makosh-communication-summary-runtime': [
    { name: 'makosh-ai-contracts', kind: 'normal' },
    { name: 'makosh-blob-client', kind: 'normal' },
    { name: 'makosh-communication-summary-api', kind: 'normal' },
    { name: 'makosh-communication-summary-core', kind: 'normal' },
    { name: 'makosh-communication-summary-persistence', kind: 'normal' },
    { name: 'makosh-communications-ai-source-api', kind: 'normal' },
    { name: 'makosh-events-jetstream', kind: 'normal' },
    { name: 'makosh-events-protocol', kind: 'normal' },
    { name: 'makosh-runtime-protocol', kind: 'normal' },
    { name: 'makosh-storage-protocol', kind: 'normal' },
    { name: 'makosh-storage-vault', kind: 'normal' },
  ],
  'makosh-communication-summary-assembly': [
    { name: 'makosh-communication-summary-persistence', kind: 'normal' },
    { name: 'makosh-communication-summary-runtime', kind: 'normal' },
    { name: 'makosh-runtime-protocol', kind: 'normal' },
    { name: 'makosh-storage-protocol', kind: 'normal' },
  ],
};

const COMMUNICATION_TRANSLATION_CONTRACT_CORE_WORKSPACE_DEPENDENCY_ALLOWLIST = {
  ...COMMUNICATION_SUMMARY_BUILD_UNITS_WORKSPACE_DEPENDENCY_ALLOWLIST,
  'makosh-communication-translation-api': [],
  'makosh-communication-translation-core': [],
};

const COMMUNICATION_TRANSLATION_PERSISTENCE_WORKSPACE_DEPENDENCY_ALLOWLIST = {
  ...COMMUNICATION_TRANSLATION_CONTRACT_CORE_WORKSPACE_DEPENDENCY_ALLOWLIST,
  'makosh-communication-translation-persistence': [
    { name: 'makosh-communication-translation-core', kind: 'normal' },
    { name: 'makosh-storage-protocol', kind: 'normal' },
  ],
};

const COMMUNICATION_TRANSLATION_RUNTIME_WORKSPACE_DEPENDENCY_ALLOWLIST = {
  ...COMMUNICATION_TRANSLATION_PERSISTENCE_WORKSPACE_DEPENDENCY_ALLOWLIST,
  'makosh-communication-translation-runtime': [
    { name: 'makosh-ai-contracts', kind: 'normal' },
    { name: 'makosh-blob-client', kind: 'normal' },
    { name: 'makosh-communication-translation-api', kind: 'normal' },
    { name: 'makosh-communication-translation-core', kind: 'normal' },
    { name: 'makosh-communication-translation-persistence', kind: 'normal' },
    { name: 'makosh-communications-ai-source-api', kind: 'normal' },
    { name: 'makosh-events-jetstream', kind: 'normal' },
    { name: 'makosh-events-protocol', kind: 'normal' },
    { name: 'makosh-runtime-protocol', kind: 'normal' },
    { name: 'makosh-storage-protocol', kind: 'normal' },
    { name: 'makosh-storage-vault', kind: 'normal' },
  ],
};

const COMMUNICATION_TRANSLATION_ASSEMBLY_WORKSPACE_DEPENDENCY_ALLOWLIST = {
  ...COMMUNICATION_TRANSLATION_RUNTIME_WORKSPACE_DEPENDENCY_ALLOWLIST,
  'makosh-communication-translation-assembly': [
    { name: 'makosh-communication-translation-persistence', kind: 'normal' },
    { name: 'makosh-communication-translation-runtime', kind: 'normal' },
    { name: 'makosh-runtime-protocol', kind: 'normal' },
    { name: 'makosh-storage-protocol', kind: 'normal' },
  ],
};

const COMMUNICATION_EXPLANATION_CONTRACT_CORE_WORKSPACE_DEPENDENCY_ALLOWLIST = {
  ...COMMUNICATION_TRANSLATION_ASSEMBLY_WORKSPACE_DEPENDENCY_ALLOWLIST,
  'makosh-communication-explanation-api': [],
  'makosh-communication-explanation-core': [],
};

const COMMUNICATION_EXPLANATION_PERSISTENCE_WORKSPACE_DEPENDENCY_ALLOWLIST = {
  ...COMMUNICATION_EXPLANATION_CONTRACT_CORE_WORKSPACE_DEPENDENCY_ALLOWLIST,
  'makosh-communication-explanation-persistence': [
    { name: 'makosh-communication-explanation-core', kind: 'normal' },
    { name: 'makosh-storage-protocol', kind: 'normal' },
  ],
};

const COMMUNICATION_EXPLANATION_RUNTIME_WORKSPACE_DEPENDENCY_ALLOWLIST = {
  ...COMMUNICATION_EXPLANATION_PERSISTENCE_WORKSPACE_DEPENDENCY_ALLOWLIST,
  'makosh-communication-explanation-runtime': [
    { name: 'makosh-ai-contracts', kind: 'normal' },
    { name: 'makosh-blob-client', kind: 'normal' },
    { name: 'makosh-communication-explanation-api', kind: 'normal' },
    { name: 'makosh-communication-explanation-core', kind: 'normal' },
    { name: 'makosh-communication-explanation-persistence', kind: 'normal' },
    { name: 'makosh-communications-ai-source-api', kind: 'normal' },
    { name: 'makosh-events-jetstream', kind: 'normal' },
    { name: 'makosh-events-protocol', kind: 'normal' },
    { name: 'makosh-runtime-protocol', kind: 'normal' },
    { name: 'makosh-storage-protocol', kind: 'normal' },
    { name: 'makosh-storage-vault', kind: 'normal' },
  ],
};

const COMMUNICATION_EXPLANATION_ASSEMBLY_WORKSPACE_DEPENDENCY_ALLOWLIST = {
  ...COMMUNICATION_EXPLANATION_RUNTIME_WORKSPACE_DEPENDENCY_ALLOWLIST,
  'makosh-communication-explanation-assembly': [
    { name: 'makosh-communication-explanation-persistence', kind: 'normal' },
    { name: 'makosh-communication-explanation-runtime', kind: 'normal' },
    { name: 'makosh-runtime-protocol', kind: 'normal' },
    { name: 'makosh-storage-protocol', kind: 'normal' },
  ],
};

const COMMUNICATION_RECIPIENT_SUGGESTION_CONTRACT_CORE_WORKSPACE_DEPENDENCY_ALLOWLIST = {
  ...COMMUNICATION_EXPLANATION_ASSEMBLY_WORKSPACE_DEPENDENCY_ALLOWLIST,
  'makosh-communication-recipient-suggestion-api': [],
  'makosh-communication-recipient-suggestion-core': [],
};

const COMMUNICATION_RECIPIENT_SUGGESTION_SOURCE_CONTRACT_WORKSPACE_DEPENDENCY_ALLOWLIST = {
  ...COMMUNICATION_RECIPIENT_SUGGESTION_CONTRACT_CORE_WORKSPACE_DEPENDENCY_ALLOWLIST,
  'makosh-communications-recipient-source-api': [
    { name: 'makosh-events-protocol', kind: 'normal' },
    { name: 'makosh-runtime-protocol', kind: 'normal' },
  ],
};

const COMMUNICATION_RECIPIENT_SUGGESTION_PERSISTENCE_WORKSPACE_DEPENDENCY_ALLOWLIST = {
  ...COMMUNICATION_RECIPIENT_SUGGESTION_CONTRACT_CORE_WORKSPACE_DEPENDENCY_ALLOWLIST,
  'makosh-communication-recipient-suggestion-persistence': [
    { name: 'makosh-communication-recipient-suggestion-core', kind: 'normal' },
    { name: 'makosh-storage-protocol', kind: 'normal' },
  ],
  'makosh-communications-recipient-source-api': [
    { name: 'makosh-events-protocol', kind: 'normal' },
    { name: 'makosh-runtime-protocol', kind: 'normal' },
  ],
};

const COMMUNICATION_RECIPIENT_SUGGESTION_RUNTIME_WORKSPACE_DEPENDENCY_ALLOWLIST = {
  ...COMMUNICATION_RECIPIENT_SUGGESTION_CONTRACT_CORE_WORKSPACE_DEPENDENCY_ALLOWLIST,
  'makosh-communication-recipient-suggestion-persistence': [
    { name: 'makosh-communication-recipient-suggestion-core', kind: 'normal' },
    { name: 'makosh-storage-protocol', kind: 'normal' },
  ],
  'makosh-communication-recipient-suggestion-runtime': [
    { name: 'makosh-blob-client', kind: 'normal' },
    { name: 'makosh-communication-recipient-suggestion-api', kind: 'normal' },
    { name: 'makosh-communication-recipient-suggestion-core', kind: 'normal' },
    { name: 'makosh-communication-recipient-suggestion-persistence', kind: 'normal' },
    { name: 'makosh-communications-recipient-source-api', kind: 'normal' },
    { name: 'makosh-events-jetstream', kind: 'normal' },
    { name: 'makosh-events-protocol', kind: 'normal' },
    { name: 'makosh-runtime-protocol', kind: 'normal' },
    { name: 'makosh-storage-protocol', kind: 'normal' },
    { name: 'makosh-storage-vault', kind: 'normal' },
  ],
  'makosh-communications-recipient-source-api': [
    { name: 'makosh-events-protocol', kind: 'normal' },
    { name: 'makosh-runtime-protocol', kind: 'normal' },
  ],
};

const COMMUNICATION_RECIPIENT_SUGGESTION_SOURCE_PRODUCER_WORKSPACE_DEPENDENCY_ALLOWLIST = {
  ...COMMUNICATION_RECIPIENT_SUGGESTION_RUNTIME_WORKSPACE_DEPENDENCY_ALLOWLIST,
  'makosh-communications-runtime': [
    ...COMMUNICATION_RECIPIENT_SUGGESTION_RUNTIME_WORKSPACE_DEPENDENCY_ALLOWLIST['makosh-communications-runtime'],
    { name: 'makosh-communications-recipient-source-api', kind: 'normal' },
  ],
};

const COMMUNICATION_RECIPIENT_SUGGESTION_ASSEMBLY_WORKSPACE_DEPENDENCY_ALLOWLIST = {
  ...COMMUNICATION_RECIPIENT_SUGGESTION_SOURCE_PRODUCER_WORKSPACE_DEPENDENCY_ALLOWLIST,
  'makosh-communication-recipient-suggestion-assembly': [
    { name: 'makosh-communication-recipient-suggestion-persistence', kind: 'normal' },
    { name: 'makosh-communication-recipient-suggestion-runtime', kind: 'normal' },
    { name: 'makosh-runtime-protocol', kind: 'normal' },
    { name: 'makosh-storage-protocol', kind: 'normal' },
  ],
};

const COMMUNICATION_TASK_CANDIDATE_CONTRACT_CORE_SOURCE_WORKSPACE_DEPENDENCY_ALLOWLIST = {
  ...COMMUNICATION_RECIPIENT_SUGGESTION_ASSEMBLY_WORKSPACE_DEPENDENCY_ALLOWLIST,
  'makosh-communication-task-candidate-api': [],
  'makosh-communication-task-candidate-core': [],
  'makosh-communications-task-source-api': [
    { name: 'makosh-events-protocol', kind: 'normal' },
    { name: 'makosh-runtime-protocol', kind: 'normal' },
  ],
};

const COMMUNICATION_TASK_CANDIDATE_PERSISTENCE_WORKSPACE_DEPENDENCY_ALLOWLIST = {
  ...COMMUNICATION_TASK_CANDIDATE_CONTRACT_CORE_SOURCE_WORKSPACE_DEPENDENCY_ALLOWLIST,
  'makosh-communication-task-candidate-persistence': [
    { name: 'makosh-communication-task-candidate-core', kind: 'normal' },
    { name: 'makosh-storage-protocol', kind: 'normal' },
  ],
};

const COMMUNICATION_TASK_CANDIDATE_RUNTIME_WORKSPACE_DEPENDENCY_ALLOWLIST = {
  ...COMMUNICATION_TASK_CANDIDATE_PERSISTENCE_WORKSPACE_DEPENDENCY_ALLOWLIST,
  'makosh-communication-task-candidate-runtime': [
    { name: 'makosh-blob-client', kind: 'normal' },
    { name: 'makosh-communication-task-candidate-api', kind: 'normal' },
    { name: 'makosh-communication-task-candidate-core', kind: 'normal' },
    { name: 'makosh-communication-task-candidate-persistence', kind: 'normal' },
    { name: 'makosh-communications-task-source-api', kind: 'normal' },
    { name: 'makosh-events-jetstream', kind: 'normal' },
    { name: 'makosh-events-protocol', kind: 'normal' },
    { name: 'makosh-review-task-candidate-api', kind: 'normal' },
    { name: 'makosh-runtime-protocol', kind: 'normal' },
    { name: 'makosh-storage-protocol', kind: 'normal' },
    { name: 'makosh-storage-vault', kind: 'normal' },
  ],
};

const COMMUNICATION_TASK_CANDIDATE_SOURCE_PRODUCER_WORKSPACE_DEPENDENCY_ALLOWLIST = {
  ...COMMUNICATION_TASK_CANDIDATE_RUNTIME_WORKSPACE_DEPENDENCY_ALLOWLIST,
  'makosh-communications-runtime': [
    ...COMMUNICATION_TASK_CANDIDATE_RUNTIME_WORKSPACE_DEPENDENCY_ALLOWLIST['makosh-communications-runtime'],
    { name: 'makosh-communications-task-source-api', kind: 'normal' },
  ],
};

const COMMUNICATION_TASK_CANDIDATE_ASSEMBLY_WORKSPACE_DEPENDENCY_ALLOWLIST = {
  ...COMMUNICATION_TASK_CANDIDATE_SOURCE_PRODUCER_WORKSPACE_DEPENDENCY_ALLOWLIST,
  'makosh-communication-task-candidate-assembly': [
    { name: 'makosh-communication-task-candidate-persistence', kind: 'normal' },
    { name: 'makosh-communication-task-candidate-runtime', kind: 'normal' },
    { name: 'makosh-runtime-protocol', kind: 'normal' },
    { name: 'makosh-storage-protocol', kind: 'normal' },
  ],
};

const REVIEW_TASK_CANDIDATE_CORE_WORKSPACE_DEPENDENCY_ALLOWLIST = {
  ...COMMUNICATION_TASK_CANDIDATE_ASSEMBLY_WORKSPACE_DEPENDENCY_ALLOWLIST,
  'makosh-review-task-candidate-api': [
    { name: 'makosh-events-protocol', kind: 'normal' },
    { name: 'makosh-runtime-protocol', kind: 'normal' },
  ],
  'makosh-review-task-candidate-core': [],
};

const REVIEW_TASK_CANDIDATE_PERSISTENCE_WORKSPACE_DEPENDENCY_ALLOWLIST = {
  ...REVIEW_TASK_CANDIDATE_CORE_WORKSPACE_DEPENDENCY_ALLOWLIST,
  'makosh-review-task-candidate-persistence': [
    { name: 'makosh-review-task-candidate-core', kind: 'normal' },
    { name: 'makosh-storage-protocol', kind: 'normal' },
  ],
};

const REVIEW_TASK_CANDIDATE_MANAGED_RUNTIME_WORKSPACE_DEPENDENCY_ALLOWLIST = {
  ...REVIEW_TASK_CANDIDATE_PERSISTENCE_WORKSPACE_DEPENDENCY_ALLOWLIST,
  'makosh-review-task-candidate-runtime': [
    { name: 'makosh-blob-client', kind: 'normal' },
    { name: 'makosh-events-jetstream', kind: 'normal' },
    { name: 'makosh-events-protocol', kind: 'normal' },
    { name: 'makosh-review-task-candidate-api', kind: 'normal' },
    { name: 'makosh-review-task-candidate-core', kind: 'normal' },
    { name: 'makosh-review-task-candidate-persistence', kind: 'normal' },
    { name: 'makosh-runtime-protocol', kind: 'normal' },
    { name: 'makosh-storage-protocol', kind: 'normal' },
    { name: 'makosh-storage-vault', kind: 'normal' },
  ],
};

const REVIEW_TASK_CANDIDATE_ASSEMBLY_WORKSPACE_DEPENDENCY_ALLOWLIST = {
  ...REVIEW_TASK_CANDIDATE_MANAGED_RUNTIME_WORKSPACE_DEPENDENCY_ALLOWLIST,
  'makosh-review-task-candidate-assembly': [
    { name: 'makosh-review-task-candidate-persistence', kind: 'normal' },
    { name: 'makosh-review-task-candidate-runtime', kind: 'normal' },
    { name: 'makosh-runtime-protocol', kind: 'normal' },
    { name: 'makosh-storage-protocol', kind: 'normal' },
  ],
};

const TASKS_REVIEWED_CANDIDATE_CONTRACT_CORE_WORKSPACE_DEPENDENCY_ALLOWLIST = {
  ...REVIEW_TASK_CANDIDATE_ASSEMBLY_WORKSPACE_DEPENDENCY_ALLOWLIST,
  'makosh-tasks-command-api': [
    { name: 'makosh-events-protocol', kind: 'normal' },
    { name: 'makosh-runtime-protocol', kind: 'normal' },
  ],
  'makosh-tasks-core': [],
};

const TASKS_REVIEWED_CANDIDATE_PERSISTENCE_WORKSPACE_DEPENDENCY_ALLOWLIST = {
  ...TASKS_REVIEWED_CANDIDATE_CONTRACT_CORE_WORKSPACE_DEPENDENCY_ALLOWLIST,
  'makosh-tasks-persistence': [
    { name: 'makosh-storage-protocol', kind: 'normal' },
    { name: 'makosh-tasks-core', kind: 'normal' },
  ],
};

const TASKS_REVIEWED_CANDIDATE_MANAGED_RUNTIME_WORKSPACE_DEPENDENCY_ALLOWLIST = {
  ...TASKS_REVIEWED_CANDIDATE_PERSISTENCE_WORKSPACE_DEPENDENCY_ALLOWLIST,
  'makosh-tasks-runtime': [
    { name: 'makosh-blob-client', kind: 'normal' },
    { name: 'makosh-events-jetstream', kind: 'normal' },
    { name: 'makosh-events-protocol', kind: 'normal' },
    { name: 'makosh-runtime-protocol', kind: 'normal' },
    { name: 'makosh-storage-protocol', kind: 'normal' },
    { name: 'makosh-storage-vault', kind: 'normal' },
    { name: 'makosh-tasks-command-api', kind: 'normal' },
    { name: 'makosh-tasks-core', kind: 'normal' },
    { name: 'makosh-tasks-persistence', kind: 'normal' },
  ],
};

const TASKS_REVIEWED_CANDIDATE_ASSEMBLY_WORKSPACE_DEPENDENCY_ALLOWLIST = {
  ...TASKS_REVIEWED_CANDIDATE_MANAGED_RUNTIME_WORKSPACE_DEPENDENCY_ALLOWLIST,
  'makosh-tasks-assembly': [
    { name: 'makosh-runtime-protocol', kind: 'normal' },
    { name: 'makosh-storage-protocol', kind: 'normal' },
    { name: 'makosh-tasks-persistence', kind: 'normal' },
    { name: 'makosh-tasks-runtime', kind: 'normal' },
  ],
};

const REVIEWED_TASK_CANDIDATE_PROMOTION_CONTRACT_CORE_WORKSPACE_DEPENDENCY_ALLOWLIST = {
  ...TASKS_REVIEWED_CANDIDATE_ASSEMBLY_WORKSPACE_DEPENDENCY_ALLOWLIST,
  'makosh-review-task-candidate-promotion-api': [
    { name: 'makosh-events-protocol', kind: 'normal' },
    { name: 'makosh-runtime-protocol', kind: 'normal' },
  ],
  'makosh-reviewed-task-candidate-promotion-core': [],
};

const REVIEWED_TASK_CANDIDATE_PROMOTION_PERSISTENCE_WORKSPACE_DEPENDENCY_ALLOWLIST = {
  ...REVIEWED_TASK_CANDIDATE_PROMOTION_CONTRACT_CORE_WORKSPACE_DEPENDENCY_ALLOWLIST,
  'makosh-reviewed-task-candidate-promotion-persistence': [
    { name: 'makosh-events-protocol', kind: 'normal' },
    { name: 'makosh-reviewed-task-candidate-promotion-core', kind: 'normal' },
    { name: 'makosh-storage-protocol', kind: 'normal' },
  ],
};

const REVIEWED_TASK_CANDIDATE_PROMOTION_RUNTIME_WORKSPACE_DEPENDENCY_ALLOWLIST = {
  ...REVIEWED_TASK_CANDIDATE_PROMOTION_PERSISTENCE_WORKSPACE_DEPENDENCY_ALLOWLIST,
  'makosh-reviewed-task-candidate-promotion-runtime': [
    { name: 'makosh-events-jetstream', kind: 'normal' },
    { name: 'makosh-events-protocol', kind: 'normal' },
    { name: 'makosh-review-task-candidate-api', kind: 'normal' },
    { name: 'makosh-review-task-candidate-promotion-api', kind: 'normal' },
    { name: 'makosh-reviewed-task-candidate-promotion-core', kind: 'normal' },
    { name: 'makosh-reviewed-task-candidate-promotion-persistence', kind: 'normal' },
    { name: 'makosh-runtime-protocol', kind: 'normal' },
    { name: 'makosh-storage-protocol', kind: 'normal' },
    { name: 'makosh-storage-vault', kind: 'normal' },
    { name: 'makosh-tasks-command-api', kind: 'normal' },
  ],
};

const REVIEW_TASK_CANDIDATE_PROMOTION_RESULT_CONSUMER_WORKSPACE_DEPENDENCY_ALLOWLIST = {
  ...REVIEWED_TASK_CANDIDATE_PROMOTION_RUNTIME_WORKSPACE_DEPENDENCY_ALLOWLIST,
  'makosh-review-task-candidate-runtime': [
    { name: 'makosh-blob-client', kind: 'normal' },
    { name: 'makosh-events-jetstream', kind: 'normal' },
    { name: 'makosh-events-protocol', kind: 'normal' },
    { name: 'makosh-review-task-candidate-api', kind: 'normal' },
    { name: 'makosh-review-task-candidate-core', kind: 'normal' },
    { name: 'makosh-review-task-candidate-persistence', kind: 'normal' },
    { name: 'makosh-review-task-candidate-promotion-api', kind: 'normal' },
    { name: 'makosh-runtime-protocol', kind: 'normal' },
    { name: 'makosh-storage-protocol', kind: 'normal' },
    { name: 'makosh-storage-vault', kind: 'normal' },
  ],
};

const REVIEWED_TASK_CANDIDATE_PROMOTION_ASSEMBLY_WORKSPACE_DEPENDENCY_ALLOWLIST = {
  ...REVIEW_TASK_CANDIDATE_PROMOTION_RESULT_CONSUMER_WORKSPACE_DEPENDENCY_ALLOWLIST,
  'makosh-reviewed-task-candidate-promotion-assembly': [
    { name: 'makosh-reviewed-task-candidate-promotion-persistence', kind: 'normal' },
    { name: 'makosh-reviewed-task-candidate-promotion-runtime', kind: 'normal' },
    { name: 'makosh-runtime-protocol', kind: 'normal' },
    { name: 'makosh-storage-protocol', kind: 'normal' },
  ],
};

const COMMUNICATION_NOTE_CANDIDATE_CONTRACT_CORE_WORKSPACE_DEPENDENCY_ALLOWLIST = {
  ...REVIEWED_TASK_CANDIDATE_PROMOTION_ASSEMBLY_WORKSPACE_DEPENDENCY_ALLOWLIST,
  'makosh-communication-note-candidate-api': [],
  'makosh-communication-note-candidate-core': [],
  'makosh-communications-note-source-api': [
    { name: 'makosh-events-protocol', kind: 'normal' },
    { name: 'makosh-runtime-protocol', kind: 'normal' },
  ],
};

const COMMUNICATION_NOTE_CANDIDATE_PERSISTENCE_WORKSPACE_DEPENDENCY_ALLOWLIST = {
  ...COMMUNICATION_NOTE_CANDIDATE_CONTRACT_CORE_WORKSPACE_DEPENDENCY_ALLOWLIST,
  'makosh-communication-note-candidate-persistence': [
    { name: 'makosh-communication-note-candidate-core', kind: 'normal' },
    { name: 'makosh-storage-protocol', kind: 'normal' },
  ],
};

const REVIEW_NOTE_CANDIDATE_CONTRACT_CORE_WORKSPACE_DEPENDENCY_ALLOWLIST = {
  ...COMMUNICATION_NOTE_CANDIDATE_PERSISTENCE_WORKSPACE_DEPENDENCY_ALLOWLIST,
  'makosh-review-note-candidate-api': [
    { name: 'makosh-events-protocol', kind: 'normal' },
    { name: 'makosh-runtime-protocol', kind: 'normal' },
  ],
  'makosh-review-note-candidate-core': [],
};

const KNOWLEDGE_VERIFIED_NOTE_CONTRACT_CORE_WORKSPACE_DEPENDENCY_ALLOWLIST = {
  ...REVIEW_NOTE_CANDIDATE_CONTRACT_CORE_WORKSPACE_DEPENDENCY_ALLOWLIST,
  'makosh-knowledge-command-api': [
    { name: 'makosh-events-protocol', kind: 'normal' },
    { name: 'makosh-runtime-protocol', kind: 'normal' },
  ],
  'makosh-knowledge-core': [],
};

const KNOWLEDGE_VERIFIED_NOTE_PERSISTENCE_WORKSPACE_DEPENDENCY_ALLOWLIST = {
  ...KNOWLEDGE_VERIFIED_NOTE_CONTRACT_CORE_WORKSPACE_DEPENDENCY_ALLOWLIST,
  'makosh-knowledge-persistence': [
    { name: 'makosh-knowledge-core', kind: 'normal' },
    { name: 'makosh-storage-protocol', kind: 'normal' },
  ],
};

const KNOWLEDGE_VERIFIED_NOTE_MANAGED_RUNTIME_WORKSPACE_DEPENDENCY_ALLOWLIST = {
  ...KNOWLEDGE_VERIFIED_NOTE_PERSISTENCE_WORKSPACE_DEPENDENCY_ALLOWLIST,
  'makosh-knowledge-runtime': [
    { name: 'makosh-blob-client', kind: 'normal' },
    { name: 'makosh-events-jetstream', kind: 'normal' },
    { name: 'makosh-events-protocol', kind: 'normal' },
    { name: 'makosh-knowledge-command-api', kind: 'normal' },
    { name: 'makosh-knowledge-core', kind: 'normal' },
    { name: 'makosh-knowledge-persistence', kind: 'normal' },
    { name: 'makosh-runtime-protocol', kind: 'normal' },
    { name: 'makosh-storage-protocol', kind: 'normal' },
    { name: 'makosh-storage-vault', kind: 'normal' },
  ],
};

const KNOWLEDGE_VERIFIED_NOTE_ASSEMBLY_WORKSPACE_DEPENDENCY_ALLOWLIST = {
  ...KNOWLEDGE_VERIFIED_NOTE_MANAGED_RUNTIME_WORKSPACE_DEPENDENCY_ALLOWLIST,
  'makosh-knowledge-assembly': [
    { name: 'makosh-knowledge-persistence', kind: 'normal' },
    { name: 'makosh-knowledge-runtime', kind: 'normal' },
    { name: 'makosh-runtime-protocol', kind: 'normal' },
    { name: 'makosh-storage-protocol', kind: 'normal' },
  ],
};

const REVIEW_NOTE_CANDIDATE_PERSISTENCE_WORKSPACE_DEPENDENCY_ALLOWLIST = {
  ...KNOWLEDGE_VERIFIED_NOTE_ASSEMBLY_WORKSPACE_DEPENDENCY_ALLOWLIST,
  'makosh-review-note-candidate-persistence': [
    { name: 'makosh-review-note-candidate-core', kind: 'normal' },
    { name: 'makosh-storage-protocol', kind: 'normal' },
  ],
};

const REVIEW_NOTE_CANDIDATE_MANAGED_RUNTIME_WORKSPACE_DEPENDENCY_ALLOWLIST = {
  ...REVIEW_NOTE_CANDIDATE_PERSISTENCE_WORKSPACE_DEPENDENCY_ALLOWLIST,
  'makosh-review-note-candidate-promotion-api': [
    { name: 'makosh-events-protocol', kind: 'normal' },
    { name: 'makosh-runtime-protocol', kind: 'normal' },
  ],
  'makosh-review-note-candidate-runtime': [
    { name: 'makosh-blob-client', kind: 'normal' },
    { name: 'makosh-events-jetstream', kind: 'normal' },
    { name: 'makosh-events-protocol', kind: 'normal' },
    { name: 'makosh-review-note-candidate-api', kind: 'normal' },
    { name: 'makosh-review-note-candidate-core', kind: 'normal' },
    { name: 'makosh-review-note-candidate-persistence', kind: 'normal' },
    { name: 'makosh-review-note-candidate-promotion-api', kind: 'normal' },
    { name: 'makosh-runtime-protocol', kind: 'normal' },
    { name: 'makosh-storage-protocol', kind: 'normal' },
    { name: 'makosh-storage-vault', kind: 'normal' },
  ],
};

const REVIEW_NOTE_CANDIDATE_ASSEMBLY_WORKSPACE_DEPENDENCY_ALLOWLIST = {
  ...REVIEW_NOTE_CANDIDATE_MANAGED_RUNTIME_WORKSPACE_DEPENDENCY_ALLOWLIST,
  'makosh-review-note-candidate-assembly': [
    { name: 'makosh-review-note-candidate-persistence', kind: 'normal' },
    { name: 'makosh-review-note-candidate-runtime', kind: 'normal' },
    { name: 'makosh-runtime-protocol', kind: 'normal' },
    { name: 'makosh-storage-protocol', kind: 'normal' },
  ],
};

const REVIEWED_NOTE_CANDIDATE_PROMOTION_WORKSPACE_DEPENDENCY_ALLOWLIST = {
  ...REVIEW_NOTE_CANDIDATE_ASSEMBLY_WORKSPACE_DEPENDENCY_ALLOWLIST,
  'makosh-reviewed-note-candidate-promotion-core': [],
  'makosh-reviewed-note-candidate-promotion-persistence': [
    { name: 'makosh-events-protocol', kind: 'normal' },
    { name: 'makosh-reviewed-note-candidate-promotion-core', kind: 'normal' },
    { name: 'makosh-storage-protocol', kind: 'normal' },
  ],
  'makosh-reviewed-note-candidate-promotion-runtime': [
    { name: 'makosh-blob-client', kind: 'normal' },
    { name: 'makosh-events-jetstream', kind: 'normal' },
    { name: 'makosh-events-protocol', kind: 'normal' },
    { name: 'makosh-knowledge-command-api', kind: 'normal' },
    { name: 'makosh-review-note-candidate-api', kind: 'normal' },
    { name: 'makosh-review-note-candidate-promotion-api', kind: 'normal' },
    { name: 'makosh-reviewed-note-candidate-promotion-core', kind: 'normal' },
    { name: 'makosh-reviewed-note-candidate-promotion-persistence', kind: 'normal' },
    { name: 'makosh-runtime-protocol', kind: 'normal' },
    { name: 'makosh-storage-protocol', kind: 'normal' },
    { name: 'makosh-storage-vault', kind: 'normal' },
  ],
  'makosh-reviewed-note-candidate-promotion-assembly': [
    { name: 'makosh-reviewed-note-candidate-promotion-persistence', kind: 'normal' },
    { name: 'makosh-reviewed-note-candidate-promotion-runtime', kind: 'normal' },
    { name: 'makosh-runtime-protocol', kind: 'normal' },
    { name: 'makosh-storage-protocol', kind: 'normal' },
  ],
};

const COMMUNICATION_NOTE_CANDIDATE_ASSEMBLY_WORKSPACE_DEPENDENCY_ALLOWLIST = {
  ...REVIEWED_NOTE_CANDIDATE_PROMOTION_WORKSPACE_DEPENDENCY_ALLOWLIST,
  'makosh-communication-note-candidate-runtime': [
    { name: 'makosh-blob-client', kind: 'normal' },
    { name: 'makosh-communication-note-candidate-api', kind: 'normal' },
    { name: 'makosh-communication-note-candidate-core', kind: 'normal' },
    { name: 'makosh-communication-note-candidate-persistence', kind: 'normal' },
    { name: 'makosh-communications-note-source-api', kind: 'normal' },
    { name: 'makosh-events-jetstream', kind: 'normal' },
    { name: 'makosh-events-protocol', kind: 'normal' },
    { name: 'makosh-review-note-candidate-api', kind: 'normal' },
    { name: 'makosh-runtime-protocol', kind: 'normal' },
    { name: 'makosh-storage-protocol', kind: 'normal' },
    { name: 'makosh-storage-vault', kind: 'normal' },
  ],
  'makosh-communication-note-candidate-assembly': [
    { name: 'makosh-communication-note-candidate-persistence', kind: 'normal' },
    { name: 'makosh-communication-note-candidate-runtime', kind: 'normal' },
    { name: 'makosh-runtime-protocol', kind: 'normal' },
    { name: 'makosh-storage-protocol', kind: 'normal' },
  ],
  'makosh-communications-runtime': [
    ...REVIEWED_NOTE_CANDIDATE_PROMOTION_WORKSPACE_DEPENDENCY_ALLOWLIST['makosh-communications-runtime'],
    { name: 'makosh-communications-note-source-api', kind: 'normal' },
  ],
};

const ATTACHMENT_TEXT_EXTRACTION_CONTRACT_CORE_WORKSPACE_DEPENDENCY_ALLOWLIST = {
  ...COMMUNICATION_NOTE_CANDIDATE_ASSEMBLY_WORKSPACE_DEPENDENCY_ALLOWLIST,
  'makosh-attachment-text-extraction-api': [],
  'makosh-attachment-text-extraction-ingress': [
    { name: 'makosh-events-protocol', kind: 'normal' },
    { name: 'makosh-runtime-protocol', kind: 'normal' },
  ],
  'makosh-attachment-text-extraction-core': [
    { name: 'makosh-attachment-text-extraction-api', kind: 'normal' },
  ],
};

const ATTACHMENT_TEXT_EXTRACTION_PARSER_ADAPTERS_WORKSPACE_DEPENDENCY_ALLOWLIST = {
  ...ATTACHMENT_TEXT_EXTRACTION_CONTRACT_CORE_WORKSPACE_DEPENDENCY_ALLOWLIST,
  'makosh-attachment-text-extraction-parser-contract': [],
  'makosh-attachment-text-extraction-plain': [
    { name: 'makosh-attachment-text-extraction-parser-contract', kind: 'normal' },
  ],
  'makosh-attachment-text-extraction-pdf': [
    { name: 'makosh-attachment-text-extraction-parser-contract', kind: 'normal' },
  ],
  'makosh-attachment-text-extraction-docx': [
    { name: 'makosh-attachment-text-extraction-parser-contract', kind: 'normal' },
  ],
  'makosh-attachment-text-extraction-ocr': [
    { name: 'makosh-attachment-text-extraction-parser-contract', kind: 'normal' },
  ],
};

const ATTACHMENT_TEXT_EXTRACTION_PERSISTENCE_WORKSPACE_DEPENDENCY_ALLOWLIST = {
  ...ATTACHMENT_TEXT_EXTRACTION_PARSER_ADAPTERS_WORKSPACE_DEPENDENCY_ALLOWLIST,
  'makosh-attachment-text-extraction-persistence': [
    { name: 'makosh-attachment-text-extraction-core', kind: 'normal' },
    { name: 'makosh-attachment-text-extraction-ingress', kind: 'normal' },
    { name: 'makosh-storage-protocol', kind: 'normal' },
  ],
};

const ATTACHMENT_TEXT_EXTRACTION_RUNTIME_ASSEMBLY_WORKSPACE_DEPENDENCY_ALLOWLIST = {
  ...ATTACHMENT_TEXT_EXTRACTION_PERSISTENCE_WORKSPACE_DEPENDENCY_ALLOWLIST,
  'makosh-attachment-text-extraction-runtime': [
    { name: 'makosh-attachment-security-contract', kind: 'normal' },
    { name: 'makosh-attachment-text-extraction-api', kind: 'normal' },
    { name: 'makosh-attachment-text-extraction-core', kind: 'normal' },
    { name: 'makosh-attachment-text-extraction-docx', kind: 'normal' },
    { name: 'makosh-attachment-text-extraction-ingress', kind: 'normal' },
    { name: 'makosh-attachment-text-extraction-ocr', kind: 'normal' },
    { name: 'makosh-attachment-text-extraction-parser-contract', kind: 'normal' },
    { name: 'makosh-attachment-text-extraction-pdf', kind: 'normal' },
    { name: 'makosh-attachment-text-extraction-persistence', kind: 'normal' },
    { name: 'makosh-attachment-text-extraction-plain', kind: 'normal' },
    { name: 'makosh-blob-client', kind: 'normal' },
    { name: 'makosh-communications-attachment-contract', kind: 'normal' },
    { name: 'makosh-events-jetstream', kind: 'normal' },
    { name: 'makosh-events-protocol', kind: 'normal' },
    { name: 'makosh-runtime-protocol', kind: 'normal' },
    { name: 'makosh-storage-protocol', kind: 'normal' },
    { name: 'makosh-storage-vault', kind: 'normal' },
  ],
  'makosh-attachment-text-extraction-assembly': [
    { name: 'makosh-attachment-text-extraction-api', kind: 'normal' },
    { name: 'makosh-attachment-text-extraction-persistence', kind: 'normal' },
    { name: 'makosh-attachment-text-extraction-runtime', kind: 'normal' },
    { name: 'makosh-runtime-protocol', kind: 'normal' },
    { name: 'makosh-storage-protocol', kind: 'normal' },
  ],
};

const ATTACHMENT_PREVIEW_FOUNDATION_WORKSPACE_DEPENDENCY_ALLOWLIST = {
  ...ATTACHMENT_TEXT_EXTRACTION_RUNTIME_ASSEMBLY_WORKSPACE_DEPENDENCY_ALLOWLIST,
  'makosh-attachment-preview-api': [],
  'makosh-attachment-preview-ingress': [
    { name: 'makosh-events-protocol', kind: 'normal' },
    { name: 'makosh-runtime-protocol', kind: 'normal' },
  ],
  'makosh-attachment-preview-core': [
    { name: 'makosh-attachment-preview-api', kind: 'normal' },
  ],
  'makosh-attachment-preview-renderer-contract': [
    { name: 'makosh-attachment-preview-api', kind: 'normal' },
  ],
};

const ATTACHMENT_PREVIEW_SAFE_ADAPTERS_WORKSPACE_DEPENDENCY_ALLOWLIST = {
  ...ATTACHMENT_PREVIEW_FOUNDATION_WORKSPACE_DEPENDENCY_ALLOWLIST,
  'makosh-attachment-preview-text': [
    { name: 'makosh-attachment-preview-api', kind: 'normal' },
    { name: 'makosh-attachment-preview-renderer-contract', kind: 'normal' },
  ],
  'makosh-attachment-preview-image': [
    { name: 'makosh-attachment-preview-api', kind: 'normal' },
    { name: 'makosh-attachment-preview-renderer-contract', kind: 'normal' },
  ],
  'makosh-attachment-preview-media': [
    { name: 'makosh-attachment-preview-api', kind: 'normal' },
    { name: 'makosh-attachment-preview-renderer-contract', kind: 'normal' },
  ],
};

const ATTACHMENT_PREVIEW_PDF_ADAPTER_WORKSPACE_DEPENDENCY_ALLOWLIST = {
  ...ATTACHMENT_PREVIEW_SAFE_ADAPTERS_WORKSPACE_DEPENDENCY_ALLOWLIST,
  'makosh-attachment-preview-pdf': [
    { name: 'makosh-attachment-preview-api', kind: 'normal' },
    { name: 'makosh-attachment-preview-renderer-contract', kind: 'normal' },
  ],
};

const ATTACHMENT_PREVIEW_DOCX_ADAPTER_WORKSPACE_DEPENDENCY_ALLOWLIST = {
  ...ATTACHMENT_PREVIEW_PDF_ADAPTER_WORKSPACE_DEPENDENCY_ALLOWLIST,
  'makosh-attachment-preview-docx': [
    { name: 'makosh-attachment-preview-api', kind: 'normal' },
    { name: 'makosh-attachment-preview-renderer-contract', kind: 'normal' },
  ],
};

const ATTACHMENT_PREVIEW_PERSISTENCE_WORKSPACE_DEPENDENCY_ALLOWLIST = {
  ...ATTACHMENT_PREVIEW_DOCX_ADAPTER_WORKSPACE_DEPENDENCY_ALLOWLIST,
  'makosh-attachment-preview-persistence': [
    { name: 'makosh-attachment-preview-api', kind: 'normal' },
    { name: 'makosh-attachment-preview-core', kind: 'normal' },
    { name: 'makosh-attachment-preview-ingress', kind: 'normal' },
    { name: 'makosh-storage-protocol', kind: 'normal' },
  ],
};

const ATTACHMENT_PREVIEW_RUNTIME_WORKSPACE_DEPENDENCY_ALLOWLIST = {
  ...ATTACHMENT_PREVIEW_PERSISTENCE_WORKSPACE_DEPENDENCY_ALLOWLIST,
  'makosh-attachment-preview-runtime': [
    { name: 'makosh-attachment-preview-api', kind: 'normal' },
    { name: 'makosh-attachment-preview-core', kind: 'normal' },
    { name: 'makosh-attachment-preview-docx', kind: 'normal' },
    { name: 'makosh-attachment-preview-image', kind: 'normal' },
    { name: 'makosh-attachment-preview-ingress', kind: 'normal' },
    { name: 'makosh-attachment-preview-media', kind: 'normal' },
    { name: 'makosh-attachment-preview-pdf', kind: 'normal' },
    { name: 'makosh-attachment-preview-persistence', kind: 'normal' },
    { name: 'makosh-attachment-preview-renderer-contract', kind: 'normal' },
    { name: 'makosh-attachment-preview-text', kind: 'normal' },
    { name: 'makosh-attachment-security-contract', kind: 'normal' },
    { name: 'makosh-blob-client', kind: 'normal' },
    { name: 'makosh-communications-attachment-contract', kind: 'normal' },
    { name: 'makosh-events-jetstream', kind: 'normal' },
    { name: 'makosh-events-protocol', kind: 'normal' },
    { name: 'makosh-runtime-protocol', kind: 'normal' },
    { name: 'makosh-storage-protocol', kind: 'normal' },
    { name: 'makosh-storage-vault', kind: 'normal' },
  ],
};

const ATTACHMENT_PREVIEW_ASSEMBLY_WORKSPACE_DEPENDENCY_ALLOWLIST = {
  ...ATTACHMENT_PREVIEW_RUNTIME_WORKSPACE_DEPENDENCY_ALLOWLIST,
  'makosh-attachment-preview-assembly': [
    { name: 'makosh-attachment-preview-api', kind: 'normal' },
    { name: 'makosh-attachment-preview-persistence', kind: 'normal' },
    { name: 'makosh-attachment-preview-runtime', kind: 'normal' },
    { name: 'makosh-runtime-protocol', kind: 'normal' },
    { name: 'makosh-storage-protocol', kind: 'normal' },
  ],
};

const ATTACHMENT_PREVIEW_RETAINED_EVIDENCE_REPLAY_WORKSPACE_DEPENDENCY_ALLOWLIST = {
  ...ATTACHMENT_PREVIEW_ASSEMBLY_WORKSPACE_DEPENDENCY_ALLOWLIST,
  'makosh-retained-evidence-replay-protocol': [],
  'makosh-attachment-preview-evidence-replay-api': [],
  'makosh-attachment-preview-evidence-replay-core': [
    { name: 'makosh-attachment-preview-evidence-replay-api', kind: 'normal' },
  ],
  'makosh-attachment-preview-evidence-replay-persistence': [
    { name: 'makosh-attachment-preview-evidence-replay-api', kind: 'normal' },
    { name: 'makosh-attachment-preview-evidence-replay-core', kind: 'normal' },
    { name: 'makosh-communications-retained-evidence-replay-contract', kind: 'normal' },
    { name: 'makosh-events-protocol', kind: 'normal' },
    { name: 'makosh-mail-retained-evidence-replay-contract', kind: 'normal' },
    { name: 'makosh-runtime-protocol', kind: 'normal' },
    { name: 'makosh-storage-protocol', kind: 'normal' },
  ],
  'makosh-attachment-preview-evidence-replay-runtime': [
    { name: 'makosh-attachment-preview-evidence-replay-api', kind: 'normal' },
    { name: 'makosh-attachment-preview-evidence-replay-core', kind: 'normal' },
    { name: 'makosh-attachment-preview-evidence-replay-persistence', kind: 'normal' },
    { name: 'makosh-communications-retained-evidence-replay-contract', kind: 'normal' },
    { name: 'makosh-events-jetstream', kind: 'normal' },
    { name: 'makosh-mail-retained-evidence-replay-contract', kind: 'normal' },
    { name: 'makosh-runtime-protocol', kind: 'normal' },
    { name: 'makosh-storage-protocol', kind: 'normal' },
    { name: 'makosh-storage-vault', kind: 'normal' },
  ],
  'makosh-attachment-preview-evidence-replay-assembly': [
    { name: 'makosh-attachment-preview-evidence-replay-persistence', kind: 'normal' },
    { name: 'makosh-attachment-preview-evidence-replay-runtime', kind: 'normal' },
    { name: 'makosh-runtime-protocol', kind: 'normal' },
    { name: 'makosh-storage-protocol', kind: 'normal' },
  ],
  'makosh-communications-retained-evidence-replay-persistence': [
    { name: 'makosh-communications-attachment-contract', kind: 'normal' },
    { name: 'makosh-communications-retained-evidence-replay-contract', kind: 'normal' },
    { name: 'makosh-events-protocol', kind: 'normal' },
    { name: 'makosh-storage-protocol', kind: 'normal' },
  ],
  'makosh-communications-runtime': [
    ...ATTACHMENT_PREVIEW_ASSEMBLY_WORKSPACE_DEPENDENCY_ALLOWLIST['makosh-communications-runtime'],
    { name: 'makosh-communications-retained-evidence-replay-persistence', kind: 'normal' },
    { name: 'makosh-communications-retained-evidence-replay-contract', kind: 'normal' },
  ],
  'makosh-mail-retained-evidence-replay-persistence': [
    { name: 'makosh-attachment-security-contract', kind: 'normal' },
    { name: 'makosh-events-protocol', kind: 'normal' },
    { name: 'makosh-mail-retained-evidence-replay-contract', kind: 'normal' },
    { name: 'makosh-storage-protocol', kind: 'normal' },
  ],
  'makosh-mail-runtime': [
    ...ATTACHMENT_PREVIEW_ASSEMBLY_WORKSPACE_DEPENDENCY_ALLOWLIST['makosh-mail-runtime'],
    { name: 'makosh-mail-retained-evidence-replay-persistence', kind: 'normal' },
    { name: 'makosh-mail-retained-evidence-replay-contract', kind: 'normal' },
  ],
  'makosh-communications-retained-evidence-replay-contract': [
    { name: 'makosh-events-protocol', kind: 'normal' },
    { name: 'makosh-runtime-protocol', kind: 'normal' },
  ],
  'makosh-mail-retained-evidence-replay-contract': [
    { name: 'makosh-events-protocol', kind: 'normal' },
    { name: 'makosh-runtime-protocol', kind: 'normal' },
  ],
};

const ATTACHMENT_TRANSLATION_CONTRACTS_WORKSPACE_DEPENDENCY_ALLOWLIST = {
  ...ATTACHMENT_PREVIEW_RETAINED_EVIDENCE_REPLAY_WORKSPACE_DEPENDENCY_ALLOWLIST,
  'makosh-attachment-translation-api': [],
  'makosh-attachment-translation-ingress': [
    { name: 'makosh-events-protocol', kind: 'normal' },
    { name: 'makosh-runtime-protocol', kind: 'normal' },
  ],
  'makosh-attachment-translation-core': [],
};

const ATTACHMENT_TRANSLATION_PERSISTENCE_WORKSPACE_DEPENDENCY_ALLOWLIST = {
  ...ATTACHMENT_TRANSLATION_CONTRACTS_WORKSPACE_DEPENDENCY_ALLOWLIST,
  'makosh-attachment-translation-persistence': [
    { name: 'makosh-attachment-translation-api', kind: 'normal' },
    { name: 'makosh-attachment-translation-core', kind: 'normal' },
    { name: 'makosh-storage-protocol', kind: 'normal' },
  ],
};

const ATTACHMENT_TRANSLATION_RUNTIME_ASSEMBLY_WORKSPACE_DEPENDENCY_ALLOWLIST = {
  ...ATTACHMENT_TRANSLATION_PERSISTENCE_WORKSPACE_DEPENDENCY_ALLOWLIST,
  'makosh-attachment-translation-runtime': [
    { name: 'makosh-ai-contracts', kind: 'normal' },
    { name: 'makosh-attachment-translation-api', kind: 'normal' },
    { name: 'makosh-attachment-translation-core', kind: 'normal' },
    { name: 'makosh-attachment-translation-ingress', kind: 'normal' },
    { name: 'makosh-attachment-translation-persistence', kind: 'normal' },
    { name: 'makosh-blob-client', kind: 'normal' },
    { name: 'makosh-events-jetstream', kind: 'normal' },
    { name: 'makosh-events-protocol', kind: 'normal' },
    { name: 'makosh-runtime-protocol', kind: 'normal' },
    { name: 'makosh-storage-protocol', kind: 'normal' },
    { name: 'makosh-storage-vault', kind: 'normal' },
  ],
  'makosh-attachment-translation-assembly': [
    { name: 'makosh-attachment-translation-persistence', kind: 'normal' },
    { name: 'makosh-attachment-translation-runtime', kind: 'normal' },
    { name: 'makosh-runtime-protocol', kind: 'normal' },
    { name: 'makosh-storage-protocol', kind: 'normal' },
  ],
};

const ATTACHMENT_TRANSLATION_SOURCE_PRODUCER_WORKSPACE_DEPENDENCY_ALLOWLIST = {
  ...ATTACHMENT_TRANSLATION_RUNTIME_ASSEMBLY_WORKSPACE_DEPENDENCY_ALLOWLIST,
  'makosh-attachment-text-extraction-runtime': [
    ...ATTACHMENT_TRANSLATION_RUNTIME_ASSEMBLY_WORKSPACE_DEPENDENCY_ALLOWLIST[
      'makosh-attachment-text-extraction-runtime'
    ].slice(0, 1),
    { name: 'makosh-attachment-translation-ingress', kind: 'normal' },
    ...ATTACHMENT_TRANSLATION_RUNTIME_ASSEMBLY_WORKSPACE_DEPENDENCY_ALLOWLIST[
      'makosh-attachment-text-extraction-runtime'
    ].slice(1),
  ],
};

const CONTACTS_MAIL_IDENTITY_COMMAND_CONTRACT_CORE_WORKSPACE_DEPENDENCY_ALLOWLIST = {
  ...ATTACHMENT_TRANSLATION_SOURCE_PRODUCER_WORKSPACE_DEPENDENCY_ALLOWLIST,
  'makosh-contacts-command-api': [
    { name: 'makosh-events-protocol', kind: 'normal' },
    { name: 'makosh-runtime-protocol', kind: 'normal' },
  ],
  'makosh-contacts-core': [],
};

const CONTACTS_MAIL_IDENTITY_COMMAND_PERSISTENCE_WORKSPACE_DEPENDENCY_ALLOWLIST = {
  ...CONTACTS_MAIL_IDENTITY_COMMAND_CONTRACT_CORE_WORKSPACE_DEPENDENCY_ALLOWLIST,
  'makosh-contacts-persistence': [
    { name: 'makosh-contacts-core', kind: 'normal' },
    { name: 'makosh-storage-protocol', kind: 'normal' },
  ],
};

const CONTACTS_MAIL_IDENTITY_COMMAND_RUNTIME_ASSEMBLY_WORKSPACE_DEPENDENCY_ALLOWLIST = {
  ...CONTACTS_MAIL_IDENTITY_COMMAND_PERSISTENCE_WORKSPACE_DEPENDENCY_ALLOWLIST,
  'makosh-contacts-runtime': [
    { name: 'makosh-contacts-command-api', kind: 'normal' },
    { name: 'makosh-contacts-core', kind: 'normal' },
    { name: 'makosh-contacts-persistence', kind: 'normal' },
    { name: 'makosh-events-jetstream', kind: 'normal' },
    { name: 'makosh-events-protocol', kind: 'normal' },
    { name: 'makosh-runtime-protocol', kind: 'normal' },
    { name: 'makosh-storage-protocol', kind: 'normal' },
    { name: 'makosh-storage-vault', kind: 'normal' },
  ],
  'makosh-contacts-assembly': [
    { name: 'makosh-contacts-persistence', kind: 'normal' },
    { name: 'makosh-contacts-runtime', kind: 'normal' },
    { name: 'makosh-runtime-protocol', kind: 'normal' },
    { name: 'makosh-storage-protocol', kind: 'normal' },
  ],
};

const MAIL_CONTACTS_SYNC_CONTRACT_CORE_WORKSPACE_DEPENDENCY_ALLOWLIST = {
  ...CONTACTS_MAIL_IDENTITY_COMMAND_RUNTIME_ASSEMBLY_WORKSPACE_DEPENDENCY_ALLOWLIST,
  'makosh-mail-address-book-contract': [
    { name: 'makosh-events-protocol', kind: 'normal' },
    { name: 'makosh-runtime-protocol', kind: 'normal' },
  ],
  'makosh-mail-contacts-sync-api': [
    { name: 'makosh-runtime-protocol', kind: 'normal' },
  ],
  'makosh-mail-contacts-sync-core': [],
};

const MAIL_CONTACTS_SYNC_PERSISTENCE_WORKSPACE_DEPENDENCY_ALLOWLIST = {
  ...MAIL_CONTACTS_SYNC_CONTRACT_CORE_WORKSPACE_DEPENDENCY_ALLOWLIST,
  'makosh-mail-contacts-sync-persistence': [
    { name: 'makosh-mail-contacts-sync-core', kind: 'normal' },
    { name: 'makosh-storage-protocol', kind: 'normal' },
  ],
};

const MAIL_CONTACTS_SYNC_RUNTIME_ADMISSION_WORKSPACE_DEPENDENCY_ALLOWLIST = {
  ...MAIL_CONTACTS_SYNC_PERSISTENCE_WORKSPACE_DEPENDENCY_ALLOWLIST,
  'makosh-contacts-mail-sync-source-api': [
    { name: 'makosh-events-protocol', kind: 'normal' },
    { name: 'makosh-runtime-protocol', kind: 'normal' },
  ],
  'makosh-contacts-runtime': [
    { name: 'makosh-blob-client', kind: 'normal' },
    { name: 'makosh-contacts-command-api', kind: 'normal' },
    { name: 'makosh-contacts-core', kind: 'normal' },
    { name: 'makosh-contacts-mail-sync-source-api', kind: 'normal' },
    { name: 'makosh-contacts-persistence', kind: 'normal' },
    { name: 'makosh-events-jetstream', kind: 'normal' },
    { name: 'makosh-events-protocol', kind: 'normal' },
    { name: 'makosh-runtime-protocol', kind: 'normal' },
    { name: 'makosh-storage-protocol', kind: 'normal' },
    { name: 'makosh-storage-vault', kind: 'normal' },
  ],
  'makosh-mail-contacts-sync-runtime': [
    { name: 'makosh-contacts-command-api', kind: 'normal' },
    { name: 'makosh-contacts-mail-sync-source-api', kind: 'normal' },
    { name: 'makosh-events-jetstream', kind: 'normal' },
    { name: 'makosh-events-protocol', kind: 'normal' },
    { name: 'makosh-mail-address-book-contract', kind: 'normal' },
    { name: 'makosh-mail-contacts-sync-api', kind: 'normal' },
    { name: 'makosh-mail-contacts-sync-core', kind: 'normal' },
    { name: 'makosh-mail-contacts-sync-persistence', kind: 'normal' },
    { name: 'makosh-runtime-protocol', kind: 'normal' },
    { name: 'makosh-scheduler-protocol', kind: 'normal' },
    { name: 'makosh-storage-protocol', kind: 'normal' },
    { name: 'makosh-storage-vault', kind: 'normal' },
  ],
};

const MAIL_ADDRESS_BOOK_PROVIDER_ADAPTERS_WORKSPACE_DEPENDENCY_ALLOWLIST = {
  ...MAIL_CONTACTS_SYNC_RUNTIME_ADMISSION_WORKSPACE_DEPENDENCY_ALLOWLIST,
  'makosh-mail-google-people': [],
  'makosh-mail-carddav': [],
};

const MAIL_ADDRESS_BOOK_PERSISTENCE_AUTHORITY_WORKSPACE_DEPENDENCY_ALLOWLIST = {
  ...MAIL_ADDRESS_BOOK_PROVIDER_ADAPTERS_WORKSPACE_DEPENDENCY_ALLOWLIST,
  'makosh-mail-address-book-persistence': [
    { name: 'makosh-events-protocol', kind: 'normal' },
    { name: 'makosh-mail-address-book-contract', kind: 'normal' },
    { name: 'makosh-storage-protocol', kind: 'normal' },
  ],
  'makosh-mail-runtime': MAIL_ADDRESS_BOOK_PROVIDER_ADAPTERS_WORKSPACE_DEPENDENCY_ALLOWLIST[
    'makosh-mail-runtime'
  ].flatMap((dependency) => (
    dependency.name === 'makosh-mail-persistence'
      ? [dependency, { name: 'makosh-mail-address-book-persistence', kind: 'normal' }]
      : [dependency]
  )),
};

const MAIL_ADDRESS_BOOK_RUNTIME_EXECUTION_WORKSPACE_DEPENDENCY_ALLOWLIST = {
  ...MAIL_ADDRESS_BOOK_PERSISTENCE_AUTHORITY_WORKSPACE_DEPENDENCY_ALLOWLIST,
  'makosh-mail-runtime': MAIL_ADDRESS_BOOK_PERSISTENCE_AUTHORITY_WORKSPACE_DEPENDENCY_ALLOWLIST[
    'makosh-mail-runtime'
  ].flatMap((dependency) => (
    dependency.name === 'makosh-mail-persistence'
      ? [
          dependency,
          { name: 'makosh-mail-address-book-contract', kind: 'normal' },
        ]
      : dependency.name === 'makosh-mail-address-book-persistence'
        ? [
            dependency,
            { name: 'makosh-mail-google-people', kind: 'normal' },
            { name: 'makosh-mail-carddav', kind: 'normal' },
            { name: 'makosh-contacts-mail-sync-source-api', kind: 'normal' },
          ]
      : [dependency]
  )),
};

const MAIL_CONTACTS_SYNC_RELEASE_ASSEMBLY_WORKSPACE_DEPENDENCY_ALLOWLIST = {
  ...MAIL_ADDRESS_BOOK_RUNTIME_EXECUTION_WORKSPACE_DEPENDENCY_ALLOWLIST,
  'makosh-mail-contacts-sync-assembly': [
    { name: 'makosh-mail-contacts-sync-persistence', kind: 'normal' },
    { name: 'makosh-mail-contacts-sync-runtime', kind: 'normal' },
      { name: 'makosh-runtime-protocol', kind: 'normal' },
    { name: 'makosh-storage-protocol', kind: 'normal' },
  ],
  'makosh-speech-to-text-api': [
    { name: 'makosh-runtime-protocol', kind: 'normal' },
  ],
  'makosh-speech-to-text-core': [],
  'makosh-speech-to-text-persistence': [
    { name: 'makosh-speech-to-text-core', kind: 'normal' },
    { name: 'makosh-storage-protocol', kind: 'normal' },
  ],
};

const DESKTOP_CALL_RECORDING_CONTRACT_CORE_WORKSPACE_DEPENDENCY_ALLOWLIST = {
  ...MAIL_CONTACTS_SYNC_RELEASE_ASSEMBLY_WORKSPACE_DEPENDENCY_ALLOWLIST,
  'makosh-desktop-call-recording-api': [
    { name: 'makosh-runtime-protocol', kind: 'normal' },
  ],
  'makosh-desktop-call-recording-core': [
    { name: 'makosh-desktop-call-recording-api', kind: 'normal' },
  ],
  'makosh-call-transcription-ingress': [
    { name: 'makosh-events-protocol', kind: 'normal' },
    { name: 'makosh-runtime-protocol', kind: 'normal' },
  ],
};

const DESKTOP_CALL_RECORDING_PERSISTENCE_WORKSPACE_DEPENDENCY_ALLOWLIST = {
  ...DESKTOP_CALL_RECORDING_CONTRACT_CORE_WORKSPACE_DEPENDENCY_ALLOWLIST,
  'makosh-desktop-call-recording-persistence': [
    { name: 'makosh-desktop-call-recording-core', kind: 'normal' },
    { name: 'makosh-storage-protocol', kind: 'normal' },
  ],
};

const DESKTOP_CALL_RECORDING_RUNTIME_WORKSPACE_DEPENDENCY_ALLOWLIST = {
  ...DESKTOP_CALL_RECORDING_PERSISTENCE_WORKSPACE_DEPENDENCY_ALLOWLIST,
  'makosh-desktop-call-recording-runtime': [
    { name: 'makosh-blob-client', kind: 'normal' },
    { name: 'makosh-call-transcription-ingress', kind: 'normal' },
    { name: 'makosh-desktop-call-recording-api', kind: 'normal' },
    { name: 'makosh-desktop-call-recording-core', kind: 'normal' },
    { name: 'makosh-desktop-call-recording-persistence', kind: 'normal' },
    { name: 'makosh-events-jetstream', kind: 'normal' },
    { name: 'makosh-events-protocol', kind: 'normal' },
    { name: 'makosh-runtime-protocol', kind: 'normal' },
    { name: 'makosh-storage-protocol', kind: 'normal' },
    { name: 'makosh-storage-vault', kind: 'normal' },
  ],
};

const DESKTOP_CALL_RECORDING_RELEASE_ASSEMBLY_WORKSPACE_DEPENDENCY_ALLOWLIST = {
  ...DESKTOP_CALL_RECORDING_RUNTIME_WORKSPACE_DEPENDENCY_ALLOWLIST,
  'makosh-desktop-call-recording-assembly': [
    { name: 'makosh-desktop-call-recording-persistence', kind: 'normal' },
    { name: 'makosh-desktop-call-recording-runtime', kind: 'normal' },
    { name: 'makosh-runtime-protocol', kind: 'normal' },
    { name: 'makosh-storage-protocol', kind: 'normal' },
  ],
};

const CALL_TRANSCRIPTION_CONTRACT_CORE_WORKSPACE_DEPENDENCY_ALLOWLIST = {
  ...DESKTOP_CALL_RECORDING_RELEASE_ASSEMBLY_WORKSPACE_DEPENDENCY_ALLOWLIST,
  'makosh-call-transcription-api': [
    { name: 'makosh-runtime-protocol', kind: 'normal' },
  ],
  'makosh-call-transcription-core': [
    { name: 'makosh-call-transcription-api', kind: 'normal' },
  ],
};

const CALL_TRANSCRIPTION_PERSISTENCE_WORKSPACE_DEPENDENCY_ALLOWLIST = {
  ...CALL_TRANSCRIPTION_CONTRACT_CORE_WORKSPACE_DEPENDENCY_ALLOWLIST,
  'makosh-call-transcription-persistence': [
    { name: 'makosh-call-transcription-api', kind: 'normal' },
    { name: 'makosh-call-transcription-core', kind: 'normal' },
    { name: 'makosh-storage-protocol', kind: 'normal' },
  ],
};

const CALL_TRANSCRIPTION_RUNTIME_WORKSPACE_DEPENDENCY_ALLOWLIST = {
  ...CALL_TRANSCRIPTION_PERSISTENCE_WORKSPACE_DEPENDENCY_ALLOWLIST,
  'makosh-call-transcription-runtime': [
    { name: 'makosh-blob-client', kind: 'normal' },
    { name: 'makosh-call-transcription-api', kind: 'normal' },
    { name: 'makosh-call-transcription-core', kind: 'normal' },
    { name: 'makosh-call-transcription-ingress', kind: 'normal' },
    { name: 'makosh-call-transcription-persistence', kind: 'normal' },
    { name: 'makosh-events-jetstream', kind: 'normal' },
    { name: 'makosh-events-protocol', kind: 'normal' },
    { name: 'makosh-runtime-protocol', kind: 'normal' },
    { name: 'makosh-speech-to-text-api', kind: 'normal' },
    { name: 'makosh-storage-protocol', kind: 'normal' },
    { name: 'makosh-storage-vault', kind: 'normal' },
  ],
};

const CALL_TRANSCRIPTION_RELEASE_ASSEMBLY_WORKSPACE_DEPENDENCY_ALLOWLIST = {
  ...CALL_TRANSCRIPTION_RUNTIME_WORKSPACE_DEPENDENCY_ALLOWLIST,
  'makosh-call-transcription-assembly': [
    { name: 'makosh-call-transcription-persistence', kind: 'normal' },
    { name: 'makosh-call-transcription-runtime', kind: 'normal' },
    { name: 'makosh-runtime-protocol', kind: 'normal' },
    { name: 'makosh-storage-protocol', kind: 'normal' },
  ],
};

const PERSONS_CONTRACT_CORE_WORKSPACE_DEPENDENCY_ALLOWLIST = {
  ...CALL_TRANSCRIPTION_RELEASE_ASSEMBLY_WORKSPACE_DEPENDENCY_ALLOWLIST,
  'makosh-persons-api': [
    { name: 'makosh-runtime-protocol', kind: 'normal' },
  ],
  'makosh-persons-core': [
    { name: 'makosh-persons-api', kind: 'normal' },
  ],
};

const PERSONS_PERSISTENCE_WORKSPACE_DEPENDENCY_ALLOWLIST = {
  ...PERSONS_CONTRACT_CORE_WORKSPACE_DEPENDENCY_ALLOWLIST,
  'makosh-persons-persistence': [
    { name: 'makosh-persons-core', kind: 'normal' },
    { name: 'makosh-storage-protocol', kind: 'normal' },
  ],
};

const PERSONS_RUNTIME_WORKSPACE_DEPENDENCY_ALLOWLIST = {
  ...PERSONS_PERSISTENCE_WORKSPACE_DEPENDENCY_ALLOWLIST,
  'makosh-persons-runtime': [
    { name: 'makosh-events-jetstream', kind: 'normal' },
    { name: 'makosh-events-protocol', kind: 'normal' },
    { name: 'makosh-persons-api', kind: 'normal' },
    { name: 'makosh-persons-core', kind: 'normal' },
    { name: 'makosh-persons-persistence', kind: 'normal' },
    { name: 'makosh-runtime-protocol', kind: 'normal' },
    { name: 'makosh-storage-protocol', kind: 'normal' },
    { name: 'makosh-storage-vault', kind: 'normal' },
  ],
};

const PERSONS_ASSEMBLY_WORKSPACE_DEPENDENCY_ALLOWLIST = {
  ...PERSONS_RUNTIME_WORKSPACE_DEPENDENCY_ALLOWLIST,
  'makosh-persons-assembly': [
    { name: 'makosh-persons-persistence', kind: 'normal' },
    { name: 'makosh-persons-runtime', kind: 'normal' },
    { name: 'makosh-runtime-protocol', kind: 'normal' },
    { name: 'makosh-storage-protocol', kind: 'normal' },
  ],
};

const MAIL_PERSONS_SYNC_CONTRACT_CORE_WORKSPACE_DEPENDENCY_ALLOWLIST = {
  ...PERSONS_ASSEMBLY_WORKSPACE_DEPENDENCY_ALLOWLIST,
  'makosh-mail-persons-sync-api': [
    { name: 'makosh-runtime-protocol', kind: 'normal' },
  ],
  'makosh-mail-persons-sync-core': [
    { name: 'makosh-mail-address-book-contract', kind: 'normal' },
    { name: 'makosh-mail-persons-sync-api', kind: 'normal' },
    { name: 'makosh-persons-api', kind: 'normal' },
  ],
  'makosh-mail-persons-sync-persistence': [
    { name: 'makosh-events-protocol', kind: 'normal' },
    { name: 'makosh-mail-persons-sync-api', kind: 'normal' },
    { name: 'makosh-mail-persons-sync-core', kind: 'normal' },
    { name: 'makosh-storage-protocol', kind: 'normal' },
  ],
  'makosh-mail-persons-sync-runtime': [
    { name: 'makosh-events-jetstream', kind: 'normal' },
    { name: 'makosh-events-protocol', kind: 'normal' },
    { name: 'makosh-mail-address-book-contract', kind: 'normal' },
    { name: 'makosh-mail-persons-sync-api', kind: 'normal' },
    { name: 'makosh-mail-persons-sync-core', kind: 'normal' },
    { name: 'makosh-mail-persons-sync-persistence', kind: 'normal' },
    { name: 'makosh-persons-api', kind: 'normal' },
    { name: 'makosh-runtime-protocol', kind: 'normal' },
    { name: 'makosh-scheduler-protocol', kind: 'normal' },
    { name: 'makosh-storage-protocol', kind: 'normal' },
    { name: 'makosh-storage-vault', kind: 'normal' },
  ],
  'makosh-mail-persons-sync-assembly': [
    { name: 'makosh-mail-persons-sync-persistence', kind: 'normal' },
    { name: 'makosh-mail-persons-sync-runtime', kind: 'normal' },
    { name: 'makosh-runtime-protocol', kind: 'normal' },
    { name: 'makosh-storage-protocol', kind: 'normal' },
  ],
  'makosh-review-person-match-candidate-api': [
    { name: 'makosh-events-protocol', kind: 'normal' },
    { name: 'makosh-runtime-protocol', kind: 'normal' },
  ],
  'makosh-review-person-match-candidate-core': [],
  'makosh-review-person-match-candidate-persistence': [
    { name: 'makosh-review-person-match-candidate-core', kind: 'normal' },
    { name: 'makosh-storage-protocol', kind: 'normal' },
  ],
  'makosh-review-person-match-candidate-runtime': [
    { name: 'makosh-events-jetstream', kind: 'normal' },
    { name: 'makosh-events-protocol', kind: 'normal' },
    { name: 'makosh-persons-api', kind: 'normal' },
    { name: 'makosh-review-person-match-candidate-api', kind: 'normal' },
    { name: 'makosh-review-person-match-candidate-core', kind: 'normal' },
    { name: 'makosh-review-person-match-candidate-persistence', kind: 'normal' },
    { name: 'makosh-review-person-match-candidate-promotion-api', kind: 'normal' },
    { name: 'makosh-runtime-protocol', kind: 'normal' },
    { name: 'makosh-storage-protocol', kind: 'normal' },
    { name: 'makosh-storage-vault', kind: 'normal' },
  ],
  'makosh-review-person-match-candidate-assembly': [
    { name: 'makosh-review-person-match-candidate-persistence', kind: 'normal' },
    { name: 'makosh-review-person-match-candidate-runtime', kind: 'normal' },
    { name: 'makosh-runtime-protocol', kind: 'normal' },
    { name: 'makosh-storage-protocol', kind: 'normal' },
  ],
  'makosh-review-person-match-candidate-promotion-api': [
    { name: 'makosh-events-protocol', kind: 'normal' },
    { name: 'makosh-runtime-protocol', kind: 'normal' },
  ],
  'makosh-reviewed-person-match-candidate-promotion-core': [
    { name: 'makosh-persons-api', kind: 'normal' },
    { name: 'makosh-review-person-match-candidate-api', kind: 'normal' },
  ],
  'makosh-reviewed-person-match-candidate-promotion-persistence': [
    { name: 'makosh-events-protocol', kind: 'normal' },
    { name: 'makosh-storage-protocol', kind: 'normal' },
  ],
  'makosh-reviewed-person-match-candidate-promotion-runtime': [
    { name: 'makosh-events-jetstream', kind: 'normal' },
    { name: 'makosh-events-protocol', kind: 'normal' },
    { name: 'makosh-persons-api', kind: 'normal' },
    { name: 'makosh-review-person-match-candidate-api', kind: 'normal' },
    { name: 'makosh-review-person-match-candidate-promotion-api', kind: 'normal' },
    { name: 'makosh-reviewed-person-match-candidate-promotion-core', kind: 'normal' },
    { name: 'makosh-reviewed-person-match-candidate-promotion-persistence', kind: 'normal' },
    { name: 'makosh-runtime-protocol', kind: 'normal' },
    { name: 'makosh-storage-protocol', kind: 'normal' },
    { name: 'makosh-storage-vault', kind: 'normal' },
  ],
  'makosh-reviewed-person-match-candidate-promotion-assembly': [
    { name: 'makosh-reviewed-person-match-candidate-promotion-persistence', kind: 'normal' },
    { name: 'makosh-reviewed-person-match-candidate-promotion-runtime', kind: 'normal' },
    { name: 'makosh-runtime-protocol', kind: 'normal' },
    { name: 'makosh-storage-protocol', kind: 'normal' },
  ],
};

const COMMUNICATIONS_EXPORT_THIRD_PARTY_DEPENDENCY_ALLOWLIST = {
  ...COMMUNICATIONS_SENDER_INSIGHTS_THIRD_PARTY_DEPENDENCY_ALLOWLIST,
  'makosh-communications-evidence-export-source-api': [
    { name: 'prost', kind: 'normal', source: 'crates_io', version: '=0.14.4', defaultFeatures: true, features: [] },
    { name: 'prost-types', kind: 'normal', source: 'crates_io', version: '=0.14.4', defaultFeatures: true, features: [] },
    { name: 'sha2', kind: 'normal', source: 'crates_io', version: '=0.11.0', defaultFeatures: false, features: [] },
    { name: 'prost-build', kind: 'build', source: 'crates_io', version: '=0.14.4', defaultFeatures: true, features: [] },
    { name: 'protoc-bin-vendored', kind: 'build', source: 'crates_io', version: '=3.2.0', defaultFeatures: true, features: [] },
    { name: 'sha2', kind: 'build', source: 'crates_io', version: '=0.11.0', defaultFeatures: false, features: [] },
  ],
  'makosh-communications-export-api': [
    { name: 'prost', kind: 'normal', source: 'crates_io', version: '=0.14.4', defaultFeatures: true, features: [] },
    { name: 'prost-build', kind: 'build', source: 'crates_io', version: '=0.14.4', defaultFeatures: true, features: [] },
    { name: 'protoc-bin-vendored', kind: 'build', source: 'crates_io', version: '=3.2.0', defaultFeatures: true, features: [] },
    { name: 'sha2', kind: 'build', source: 'crates_io', version: '=0.11.0', defaultFeatures: false, features: [] },
  ],
  'makosh-communications-export-core': [
    { name: 'serde', kind: 'normal', source: 'crates_io', version: '=1.0.228', defaultFeatures: false, features: ['derive', 'std'] },
    { name: 'serde_json', kind: 'normal', source: 'crates_io', version: '=1.0.150', defaultFeatures: true, features: [] },
    { name: 'sha2', kind: 'normal', source: 'crates_io', version: '=0.11.0', defaultFeatures: false, features: [] },
  ],
  'makosh-communications-export-persistence': [
    { name: 'sha2', kind: 'normal', source: 'crates_io', version: '=0.11.0', defaultFeatures: false, features: [] },
    { name: 'sqlx', kind: 'normal', source: 'crates_io', version: '=0.9.0', defaultFeatures: false, features: ['postgres', 'runtime-tokio', 'tls-rustls-ring'] },
  ],
  'makosh-communications-export-runtime': [
    { name: 'getrandom', kind: 'normal', source: 'crates_io', version: '=0.4.3', defaultFeatures: true, features: [] },
    { name: 'libc', kind: 'normal', source: 'crates_io', version: '=0.2.186', defaultFeatures: true, features: [] },
    { name: 'prost', kind: 'normal', source: 'crates_io', version: '=0.14.4', defaultFeatures: true, features: [] },
    { name: 'sha2', kind: 'normal', source: 'crates_io', version: '=0.11.0', defaultFeatures: false, features: [] },
    { name: 'tokio', kind: 'normal', source: 'crates_io', version: '=1.52.4', defaultFeatures: false, features: ['rt', 'rt-multi-thread', 'time'] },
    { name: 'zeroize', kind: 'normal', source: 'crates_io', version: '=1.9.0', defaultFeatures: true, features: [] },
  ],
  'makosh-communications-export-assembly': [
    { name: 'prost', kind: 'normal', source: 'crates_io', version: '=0.14.4', defaultFeatures: true, features: [] },
    { name: 'serde', kind: 'normal', source: 'crates_io', version: '=1.0.228', defaultFeatures: false, features: ['derive', 'std'] },
    { name: 'serde_json', kind: 'normal', source: 'crates_io', version: '=1.0.150', defaultFeatures: true, features: [] },
  ],
};

const COMMUNICATION_DELIVERY_INTENT_CONTRACT_CORE_THIRD_PARTY_DEPENDENCY_ALLOWLIST = {
  ...COMMUNICATIONS_EXPORT_THIRD_PARTY_DEPENDENCY_ALLOWLIST,
  'makosh-communication-delivery-intent-api': [
    { name: 'prost', kind: 'normal', source: 'crates_io', version: '=0.14.4', defaultFeatures: true, features: [] },
    { name: 'prost-build', kind: 'build', source: 'crates_io', version: '=0.14.4', defaultFeatures: true, features: [] },
    { name: 'protoc-bin-vendored', kind: 'build', source: 'crates_io', version: '=3.2.0', defaultFeatures: true, features: [] },
    { name: 'sha2', kind: 'build', source: 'crates_io', version: '=0.11.0', defaultFeatures: false, features: [] },
  ],
  'makosh-communication-delivery-intent-core': [],
};

const COMMUNICATION_DELIVERY_INTENT_PERSISTENCE_THIRD_PARTY_DEPENDENCY_ALLOWLIST = {
  ...COMMUNICATION_DELIVERY_INTENT_CONTRACT_CORE_THIRD_PARTY_DEPENDENCY_ALLOWLIST,
  'makosh-communication-delivery-intent-persistence': [
    { name: 'sha2', kind: 'normal', source: 'crates_io', version: '=0.11.0', defaultFeatures: false, features: [] },
    { name: 'sqlx', kind: 'normal', source: 'crates_io', version: '=0.9.0', defaultFeatures: false, features: ['postgres', 'runtime-tokio', 'tls-rustls-ring'] },
  ],
};

const COMMUNICATION_DELIVERY_INTENT_RUNTIME_THIRD_PARTY_DEPENDENCY_ALLOWLIST = {
  ...COMMUNICATION_DELIVERY_INTENT_PERSISTENCE_THIRD_PARTY_DEPENDENCY_ALLOWLIST,
  'makosh-communication-delivery-intent-runtime': [
    { name: 'libc', kind: 'normal', source: 'crates_io', version: '=0.2.186', defaultFeatures: true, features: [] },
    { name: 'prost', kind: 'normal', source: 'crates_io', version: '=0.14.4', defaultFeatures: true, features: [] },
    { name: 'sha2', kind: 'normal', source: 'crates_io', version: '=0.11.0', defaultFeatures: false, features: [] },
    { name: 'tokio', kind: 'normal', source: 'crates_io', version: '=1.52.4', defaultFeatures: false, features: ['rt', 'rt-multi-thread', 'time'] },
    { name: 'zeroize', kind: 'normal', source: 'crates_io', version: '=1.9.0', defaultFeatures: true, features: [] },
  ],
};

const COMMUNICATION_DELIVERY_INTENT_ASSEMBLY_THIRD_PARTY_DEPENDENCY_ALLOWLIST = {
  ...COMMUNICATION_DELIVERY_INTENT_RUNTIME_THIRD_PARTY_DEPENDENCY_ALLOWLIST,
  'makosh-communication-delivery-intent-assembly': [
    { name: 'prost', kind: 'normal', source: 'crates_io', version: '=0.14.4', defaultFeatures: true, features: [] },
    { name: 'serde', kind: 'normal', source: 'crates_io', version: '=1.0.228', defaultFeatures: false, features: ['derive', 'std'] },
    { name: 'serde_json', kind: 'normal', source: 'crates_io', version: '=1.0.150', defaultFeatures: true, features: [] },
  ],
};

const DELIVERY_INTENT_TRANSACTIONAL_EVENT_ADAPTERS_THIRD_PARTY_DEPENDENCY_ALLOWLIST = {
  ...COMMUNICATION_DELIVERY_INTENT_ASSEMBLY_THIRD_PARTY_DEPENDENCY_ALLOWLIST,
  'makosh-mail-delivery-intent-contract': [
    { name: 'prost', kind: 'normal', source: 'crates_io', version: '=0.14.4', defaultFeatures: true, features: [] },
    { name: 'prost-build', kind: 'build', source: 'crates_io', version: '=0.14.4', defaultFeatures: true, features: [] },
    { name: 'protoc-bin-vendored', kind: 'build', source: 'crates_io', version: '=3.2.0', defaultFeatures: true, features: [] },
    { name: 'sha2', kind: 'build', source: 'crates_io', version: '=0.11.0', defaultFeatures: false, features: [] },
  ],
  'makosh-telegram-delivery-intent-contract': [
    { name: 'prost', kind: 'normal', source: 'crates_io', version: '=0.14.4', defaultFeatures: true, features: [] },
    { name: 'prost-build', kind: 'build', source: 'crates_io', version: '=0.14.4', defaultFeatures: true, features: [] },
    { name: 'protoc-bin-vendored', kind: 'build', source: 'crates_io', version: '=3.2.0', defaultFeatures: true, features: [] },
    { name: 'sha2', kind: 'build', source: 'crates_io', version: '=0.11.0', defaultFeatures: false, features: [] },
  ],
  'makosh-whatsapp-delivery-intent-contract': [
    { name: 'prost', kind: 'normal', source: 'crates_io', version: '=0.14.4', defaultFeatures: true, features: [] },
    { name: 'prost-build', kind: 'build', source: 'crates_io', version: '=0.14.4', defaultFeatures: true, features: [] },
    { name: 'protoc-bin-vendored', kind: 'build', source: 'crates_io', version: '=3.2.0', defaultFeatures: true, features: [] },
    { name: 'sha2', kind: 'build', source: 'crates_io', version: '=0.11.0', defaultFeatures: false, features: [] },
  ],
  'makosh-zulip-delivery-intent-contract': [
    { name: 'prost', kind: 'normal', source: 'crates_io', version: '=0.14.4', defaultFeatures: true, features: [] },
    { name: 'prost-build', kind: 'build', source: 'crates_io', version: '=0.14.4', defaultFeatures: true, features: [] },
    { name: 'protoc-bin-vendored', kind: 'build', source: 'crates_io', version: '=3.2.0', defaultFeatures: true, features: [] },
    { name: 'sha2', kind: 'build', source: 'crates_io', version: '=0.11.0', defaultFeatures: false, features: [] },
  ],
  'makosh-communication-delivery-intent-event-adapters': [
    { name: 'prost', kind: 'normal', source: 'crates_io', version: '=0.14.4', defaultFeatures: true, features: [] },
    { name: 'prost-types', kind: 'normal', source: 'crates_io', version: '=0.14.4', defaultFeatures: true, features: [] },
    { name: 'sha2', kind: 'normal', source: 'crates_io', version: '=0.11.0', defaultFeatures: false, features: [] },
  ],
};

const DELIVERY_INTENT_TARGET_BOUND_BLOB_THIRD_PARTY_DEPENDENCY_ALLOWLIST = {
  ...DELIVERY_INTENT_TRANSACTIONAL_EVENT_ADAPTERS_THIRD_PARTY_DEPENDENCY_ALLOWLIST,
  'makosh-mail-runtime': [
    { name: 'getrandom', kind: 'normal', source: 'crates_io', version: '=0.4.3', defaultFeatures: false, features: [] },
    { name: 'libc', kind: 'normal', source: 'crates_io', version: '=0.2.186', defaultFeatures: true, features: [] },
    { name: 'prost', kind: 'normal', source: 'crates_io', version: '=0.14.4', defaultFeatures: true, features: [] },
    { name: 'prost-types', kind: 'normal', source: 'crates_io', version: '=0.14.4', defaultFeatures: true, features: [] },
    { name: 'sha2', kind: 'normal', source: 'crates_io', version: '=0.11.0', defaultFeatures: false, features: [] },
    { name: 'tokio', kind: 'normal', source: 'crates_io', version: '=1.52.4', defaultFeatures: false, features: ['rt-multi-thread', 'time'] },
    { name: 'zeroize', kind: 'normal', source: 'crates_io', version: '=1.9.0', defaultFeatures: true, features: [] },
  ],
};

const COMMUNICATION_BULK_ACTION_CONTRACT_CORE_THIRD_PARTY_DEPENDENCY_ALLOWLIST = {
  ...DELIVERY_INTENT_TARGET_BOUND_BLOB_THIRD_PARTY_DEPENDENCY_ALLOWLIST,
  'makosh-communication-bulk-action-api': [
    { name: 'prost', kind: 'normal', source: 'crates_io', version: '=0.14.4', defaultFeatures: true, features: [] },
    { name: 'prost-build', kind: 'build', source: 'crates_io', version: '=0.14.4', defaultFeatures: true, features: [] },
    { name: 'protoc-bin-vendored', kind: 'build', source: 'crates_io', version: '=3.2.0', defaultFeatures: true, features: [] },
    { name: 'sha2', kind: 'build', source: 'crates_io', version: '=0.11.0', defaultFeatures: false, features: [] },
  ],
  'makosh-communication-bulk-action-core': [],
};

const COMMUNICATION_BULK_ACTION_PERSISTENCE_THIRD_PARTY_DEPENDENCY_ALLOWLIST = {
  ...COMMUNICATION_BULK_ACTION_CONTRACT_CORE_THIRD_PARTY_DEPENDENCY_ALLOWLIST,
  'makosh-communication-bulk-action-persistence': [
    { name: 'sha2', kind: 'normal', source: 'crates_io', version: '=0.11.0', defaultFeatures: false, features: [] },
    { name: 'sqlx', kind: 'normal', source: 'crates_io', version: '=0.9.0', defaultFeatures: false, features: ['postgres', 'runtime-tokio', 'tls-rustls-ring'] },
  ],
};

const COMMUNICATION_BULK_ACTION_RUNTIME_CORE_THIRD_PARTY_DEPENDENCY_ALLOWLIST = {
  ...COMMUNICATION_BULK_ACTION_PERSISTENCE_THIRD_PARTY_DEPENDENCY_ALLOWLIST,
  'makosh-communication-bulk-action-runtime': [
    { name: 'libc', kind: 'normal', source: 'crates_io', version: '=0.2.186', defaultFeatures: true, features: [] },
    { name: 'prost', kind: 'normal', source: 'crates_io', version: '=0.14.4', defaultFeatures: true, features: [] },
    { name: 'sha2', kind: 'normal', source: 'crates_io', version: '=0.11.0', defaultFeatures: false, features: [] },
    { name: 'tokio', kind: 'normal', source: 'crates_io', version: '=1.52.4', defaultFeatures: false, features: ['rt-multi-thread', 'time'] },
    { name: 'zeroize', kind: 'normal', source: 'crates_io', version: '=1.9.0', defaultFeatures: true, features: [] },
  ],
};

const COMMUNICATION_BULK_ACTION_ASSEMBLY_THIRD_PARTY_DEPENDENCY_ALLOWLIST = {
  ...COMMUNICATION_BULK_ACTION_RUNTIME_CORE_THIRD_PARTY_DEPENDENCY_ALLOWLIST,
  'makosh-communication-bulk-action-assembly': [
    { name: 'prost', kind: 'normal', source: 'crates_io', version: '=0.14.4', defaultFeatures: true, features: [] },
    { name: 'serde', kind: 'normal', source: 'crates_io', version: '=1.0.228', defaultFeatures: false, features: ['derive', 'std'] },
    { name: 'serde_json', kind: 'normal', source: 'crates_io', version: '=1.0.150', defaultFeatures: true, features: [] },
  ],
};

const COMMUNICATION_DELAYED_DELIVERY_CONTRACT_CORE_THIRD_PARTY_DEPENDENCY_ALLOWLIST = {
  ...COMMUNICATION_BULK_ACTION_ASSEMBLY_THIRD_PARTY_DEPENDENCY_ALLOWLIST,
  'makosh-communication-delayed-delivery-api': [
    { name: 'prost', kind: 'normal', source: 'crates_io', version: '=0.14.4', defaultFeatures: true, features: [] },
    { name: 'prost-build', kind: 'build', source: 'crates_io', version: '=0.14.4', defaultFeatures: true, features: [] },
    { name: 'protoc-bin-vendored', kind: 'build', source: 'crates_io', version: '=3.2.0', defaultFeatures: true, features: [] },
    { name: 'sha2', kind: 'build', source: 'crates_io', version: '=0.11.0', defaultFeatures: false, features: [] },
  ],
  'makosh-communication-delayed-delivery-core': [],
};

const COMMUNICATION_DELAYED_DELIVERY_PERSISTENCE_THIRD_PARTY_DEPENDENCY_ALLOWLIST = {
  ...COMMUNICATION_DELAYED_DELIVERY_CONTRACT_CORE_THIRD_PARTY_DEPENDENCY_ALLOWLIST,
  'makosh-communication-delayed-delivery-persistence': [
    { name: 'sha2', kind: 'normal', source: 'crates_io', version: '=0.11.0', defaultFeatures: false, features: [] },
    { name: 'sqlx', kind: 'normal', source: 'crates_io', version: '=0.9.0', defaultFeatures: false, features: ['postgres', 'runtime-tokio', 'tls-rustls-ring'] },
  ],
};

const COMMUNICATION_DELAYED_DELIVERY_EXECUTION_THIRD_PARTY_DEPENDENCY_ALLOWLIST = {
  ...COMMUNICATION_DELAYED_DELIVERY_PERSISTENCE_THIRD_PARTY_DEPENDENCY_ALLOWLIST,
  'makosh-communication-delayed-delivery-execution': [
    { name: 'prost', kind: 'normal', source: 'crates_io', version: '=0.14.4', defaultFeatures: true, features: [] },
    { name: 'sha2', kind: 'normal', source: 'crates_io', version: '=0.11.0', defaultFeatures: false, features: [] },
    { name: 'tokio', kind: 'normal', source: 'crates_io', version: '=1.52.4', defaultFeatures: false, features: ['macros', 'rt'] },
    { name: 'zeroize', kind: 'normal', source: 'crates_io', version: '=1.9.0', defaultFeatures: true, features: [] },
  ],
};

const COMMUNICATION_DELAYED_DELIVERY_EVENT_ADAPTERS_THIRD_PARTY_DEPENDENCY_ALLOWLIST = {
  ...COMMUNICATION_DELAYED_DELIVERY_EXECUTION_THIRD_PARTY_DEPENDENCY_ALLOWLIST,
  'makosh-communication-delayed-delivery-event-adapters': [
    { name: 'prost', kind: 'normal', source: 'crates_io', version: '=0.14.4', defaultFeatures: true, features: [] },
    { name: 'prost-types', kind: 'normal', source: 'crates_io', version: '=0.14.4', defaultFeatures: true, features: [] },
    { name: 'sha2', kind: 'normal', source: 'crates_io', version: '=0.11.0', defaultFeatures: false, features: [] },
  ],
};

const COMMUNICATION_DELAYED_DELIVERY_RUNTIME_ADAPTERS_THIRD_PARTY_DEPENDENCY_ALLOWLIST = {
  ...COMMUNICATION_DELAYED_DELIVERY_EVENT_ADAPTERS_THIRD_PARTY_DEPENDENCY_ALLOWLIST,
  'makosh-communication-delayed-delivery-runtime-adapters': [
    { name: 'sha2', kind: 'normal', source: 'crates_io', version: '=0.11.0', defaultFeatures: false, features: [] },
  ],
};

const COMMUNICATION_DELAYED_DELIVERY_STORE_ADAPTERS_THIRD_PARTY_DEPENDENCY_ALLOWLIST = {
  ...COMMUNICATION_DELAYED_DELIVERY_RUNTIME_ADAPTERS_THIRD_PARTY_DEPENDENCY_ALLOWLIST,
  'makosh-communication-delayed-delivery-store-adapters': [],
};

const COMMUNICATION_DELAYED_DELIVERY_MANAGED_RUNTIME_THIRD_PARTY_DEPENDENCY_ALLOWLIST = {
  ...COMMUNICATION_DELAYED_DELIVERY_STORE_ADAPTERS_THIRD_PARTY_DEPENDENCY_ALLOWLIST,
  'makosh-communication-delayed-delivery-runtime': [
    { name: 'libc', kind: 'normal', source: 'crates_io', version: '=0.2.186', defaultFeatures: true, features: [] },
    { name: 'prost', kind: 'normal', source: 'crates_io', version: '=0.14.4', defaultFeatures: true, features: [] },
    { name: 'sha2', kind: 'normal', source: 'crates_io', version: '=0.11.0', defaultFeatures: false, features: [] },
    { name: 'tokio', kind: 'normal', source: 'crates_io', version: '=1.52.4', defaultFeatures: false, features: ['rt-multi-thread', 'time'] },
    { name: 'zeroize', kind: 'normal', source: 'crates_io', version: '=1.9.0', defaultFeatures: true, features: [] },
  ],
};

const COMMUNICATION_DELAYED_DELIVERY_ASSEMBLY_THIRD_PARTY_DEPENDENCY_ALLOWLIST = {
  ...COMMUNICATION_DELAYED_DELIVERY_MANAGED_RUNTIME_THIRD_PARTY_DEPENDENCY_ALLOWLIST,
  'makosh-communication-delayed-delivery-assembly': [
    { name: 'prost', kind: 'normal', source: 'crates_io', version: '=0.14.4', defaultFeatures: true, features: [] },
    { name: 'serde', kind: 'normal', source: 'crates_io', version: '=1.0.228', defaultFeatures: false, features: ['derive', 'std'] },
    { name: 'serde_json', kind: 'normal', source: 'crates_io', version: '=1.0.150', defaultFeatures: true, features: [] },
  ],
};

const COMMUNICATION_CROSS_CHANNEL_FORWARD_CONTRACT_CORE_THIRD_PARTY_DEPENDENCY_ALLOWLIST = {
  ...COMMUNICATION_DELAYED_DELIVERY_ASSEMBLY_THIRD_PARTY_DEPENDENCY_ALLOWLIST,
  'makosh-communication-cross-channel-forward-api': [
    { name: 'prost', kind: 'normal', source: 'crates_io', version: '=0.14.4', defaultFeatures: true, features: [] },
    { name: 'prost-build', kind: 'build', source: 'crates_io', version: '=0.14.4', defaultFeatures: true, features: [] },
    { name: 'protoc-bin-vendored', kind: 'build', source: 'crates_io', version: '=3.2.0', defaultFeatures: true, features: [] },
    { name: 'sha2', kind: 'build', source: 'crates_io', version: '=0.11.0', defaultFeatures: false, features: [] },
  ],
  'makosh-communication-cross-channel-forward-core': [],
};

const COMMUNICATION_CROSS_CHANNEL_FORWARD_PERSISTENCE_THIRD_PARTY_DEPENDENCY_ALLOWLIST = {
  ...COMMUNICATION_CROSS_CHANNEL_FORWARD_CONTRACT_CORE_THIRD_PARTY_DEPENDENCY_ALLOWLIST,
  'makosh-communication-cross-channel-forward-persistence': [
    { name: 'sha2', kind: 'normal', source: 'crates_io', version: '=0.11.0', defaultFeatures: false, features: [] },
    { name: 'sqlx', kind: 'normal', source: 'crates_io', version: '=0.9.0', defaultFeatures: false, features: ['postgres', 'runtime-tokio', 'tls-rustls-ring'] },
  ],
};

const COMMUNICATION_CROSS_CHANNEL_FORWARD_SOURCE_CONTRACT_THIRD_PARTY_DEPENDENCY_ALLOWLIST = {
  ...COMMUNICATION_CROSS_CHANNEL_FORWARD_PERSISTENCE_THIRD_PARTY_DEPENDENCY_ALLOWLIST,
  'makosh-communications-cross-channel-forward-source-api': [
    { name: 'prost', kind: 'normal', source: 'crates_io', version: '=0.14.4', defaultFeatures: true, features: [] },
    { name: 'prost-types', kind: 'normal', source: 'crates_io', version: '=0.14.4', defaultFeatures: true, features: [] },
    { name: 'sha2', kind: 'normal', source: 'crates_io', version: '=0.11.0', defaultFeatures: false, features: [] },
    { name: 'prost-build', kind: 'build', source: 'crates_io', version: '=0.14.4', defaultFeatures: true, features: [] },
    { name: 'protoc-bin-vendored', kind: 'build', source: 'crates_io', version: '=3.2.0', defaultFeatures: true, features: [] },
    { name: 'sha2', kind: 'build', source: 'crates_io', version: '=0.11.0', defaultFeatures: false, features: [] },
  ],
};

const COMMUNICATION_DELIVERY_INTENT_INGRESS_CONTRACT_THIRD_PARTY_DEPENDENCY_ALLOWLIST = {
  ...COMMUNICATION_CROSS_CHANNEL_FORWARD_SOURCE_CONTRACT_THIRD_PARTY_DEPENDENCY_ALLOWLIST,
  'makosh-communication-delivery-intent-ingress-api': [
    { name: 'prost', kind: 'normal', source: 'crates_io', version: '=0.14.4', defaultFeatures: true, features: [] },
    { name: 'prost-types', kind: 'normal', source: 'crates_io', version: '=0.14.4', defaultFeatures: true, features: [] },
    { name: 'sha2', kind: 'normal', source: 'crates_io', version: '=0.11.0', defaultFeatures: false, features: [] },
    { name: 'prost-build', kind: 'build', source: 'crates_io', version: '=0.14.4', defaultFeatures: true, features: [] },
    { name: 'protoc-bin-vendored', kind: 'build', source: 'crates_io', version: '=3.2.0', defaultFeatures: true, features: [] },
    { name: 'sha2', kind: 'build', source: 'crates_io', version: '=0.11.0', defaultFeatures: false, features: [] },
  ],
};

const COMMUNICATION_CROSS_CHANNEL_FORWARD_EVENT_PERSISTENCE_THIRD_PARTY_DEPENDENCY_ALLOWLIST =
  COMMUNICATION_DELIVERY_INTENT_INGRESS_CONTRACT_THIRD_PARTY_DEPENDENCY_ALLOWLIST;

const COMMUNICATION_CROSS_CHANNEL_FORWARD_MANAGED_RUNTIME_THIRD_PARTY_DEPENDENCY_ALLOWLIST = {
  ...COMMUNICATION_CROSS_CHANNEL_FORWARD_EVENT_PERSISTENCE_THIRD_PARTY_DEPENDENCY_ALLOWLIST,
  'makosh-communication-cross-channel-forward-runtime': [
    { name: 'libc', kind: 'normal', source: 'crates_io', version: '=0.2.186', defaultFeatures: true, features: [] },
    { name: 'prost', kind: 'normal', source: 'crates_io', version: '=0.14.4', defaultFeatures: true, features: [] },
    { name: 'sha2', kind: 'normal', source: 'crates_io', version: '=0.11.0', defaultFeatures: false, features: [] },
    { name: 'tokio', kind: 'normal', source: 'crates_io', version: '=1.52.4', defaultFeatures: false, features: ['rt-multi-thread', 'time'] },
    { name: 'zeroize', kind: 'normal', source: 'crates_io', version: '=1.9.0', defaultFeatures: true, features: [] },
  ],
};

const COMMUNICATION_DELIVERY_INTENT_EVENT_INGRESS_CONSUMER_THIRD_PARTY_DEPENDENCY_ALLOWLIST =
  COMMUNICATION_CROSS_CHANNEL_FORWARD_MANAGED_RUNTIME_THIRD_PARTY_DEPENDENCY_ALLOWLIST;

const COMMUNICATION_CROSS_CHANNEL_FORWARD_CLIENT_ASSEMBLY_THIRD_PARTY_DEPENDENCY_ALLOWLIST = {
  ...COMMUNICATION_DELIVERY_INTENT_EVENT_INGRESS_CONSUMER_THIRD_PARTY_DEPENDENCY_ALLOWLIST,
  'makosh-communication-cross-channel-forward-assembly': [
    { name: 'prost', kind: 'normal', source: 'crates_io', version: '=0.14.4', defaultFeatures: true, features: [] },
    { name: 'serde', kind: 'normal', source: 'crates_io', version: '=1.0.228', defaultFeatures: false, features: ['derive', 'std'] },
    { name: 'serde_json', kind: 'normal', source: 'crates_io', version: '=1.0.150', defaultFeatures: true, features: [] },
  ],
};

const COMMUNICATIONS_CALL_EVIDENCE_CONTRACT_CORE_THIRD_PARTY_DEPENDENCY_ALLOWLIST = {
  ...COMMUNICATION_CROSS_CHANNEL_FORWARD_CLIENT_ASSEMBLY_THIRD_PARTY_DEPENDENCY_ALLOWLIST,
  'makosh-communications-call-evidence-ingress': [
    { name: 'prost', kind: 'normal', source: 'crates_io', version: '=0.14.4', defaultFeatures: true, features: [] },
    { name: 'prost-types', kind: 'normal', source: 'crates_io', version: '=0.14.4', defaultFeatures: true, features: [] },
    { name: 'sha2', kind: 'normal', source: 'crates_io', version: '=0.11.0', defaultFeatures: false, features: [] },
    { name: 'prost-build', kind: 'build', source: 'crates_io', version: '=0.14.4', defaultFeatures: true, features: [] },
    { name: 'protoc-bin-vendored', kind: 'build', source: 'crates_io', version: '=3.2.0', defaultFeatures: true, features: [] },
    { name: 'sha2', kind: 'build', source: 'crates_io', version: '=0.11.0', defaultFeatures: false, features: [] },
  ],
  'makosh-communications-call-evidence-core': [
    { name: 'prost-types', kind: 'normal', source: 'crates_io', version: '=0.14.4', defaultFeatures: true, features: [] },
    { name: 'sha2', kind: 'normal', source: 'crates_io', version: '=0.11.0', defaultFeatures: false, features: [] },
  ],
};

const COMMUNICATIONS_CALL_EVIDENCE_PERSISTENCE_THIRD_PARTY_DEPENDENCY_ALLOWLIST = {
  ...COMMUNICATIONS_CALL_EVIDENCE_CONTRACT_CORE_THIRD_PARTY_DEPENDENCY_ALLOWLIST,
  'makosh-communications-call-evidence-persistence': [
    { name: 'sha2', kind: 'normal', source: 'crates_io', version: '=0.11.0', defaultFeatures: false, features: [] },
    { name: 'sqlx', kind: 'normal', source: 'crates_io', version: '=0.9.0', defaultFeatures: false, features: ['postgres', 'runtime-tokio', 'tls-rustls-ring'] },
  ],
};

const COMMUNICATIONS_CALL_EVIDENCE_QUERY_REALTIME_THIRD_PARTY_DEPENDENCY_ALLOWLIST = {
  ...COMMUNICATIONS_CALL_EVIDENCE_PERSISTENCE_THIRD_PARTY_DEPENDENCY_ALLOWLIST,
  'makosh-communications-call-evidence-api': [
    { name: 'prost', kind: 'normal', source: 'crates_io', version: '=0.14.4', defaultFeatures: true, features: [] },
    { name: 'prost-build', kind: 'build', source: 'crates_io', version: '=0.14.4', defaultFeatures: true, features: [] },
    { name: 'protoc-bin-vendored', kind: 'build', source: 'crates_io', version: '=3.2.0', defaultFeatures: true, features: [] },
    { name: 'sha2', kind: 'build', source: 'crates_io', version: '=0.11.0', defaultFeatures: false, features: [] },
  ],
};

const REVIEW_COMMUNICATIONS_ATTENTION_CONTRACT_CORE_THIRD_PARTY_DEPENDENCY_ALLOWLIST = {
  ...COMMUNICATIONS_CALL_EVIDENCE_QUERY_REALTIME_THIRD_PARTY_DEPENDENCY_ALLOWLIST,
  'makosh-review-attention-api': [
    { name: 'prost', kind: 'normal', source: 'crates_io', version: '=0.14.4', defaultFeatures: true, features: [] },
    { name: 'prost-build', kind: 'build', source: 'crates_io', version: '=0.14.4', defaultFeatures: true, features: [] },
    { name: 'protoc-bin-vendored', kind: 'build', source: 'crates_io', version: '=3.2.0', defaultFeatures: true, features: [] },
    { name: 'sha2', kind: 'build', source: 'crates_io', version: '=0.11.0', defaultFeatures: false, features: [] },
  ],
  'makosh-review-attention-core': [
    { name: 'sha2', kind: 'normal', source: 'crates_io', version: '=0.11.0', defaultFeatures: false, features: [] },
  ],
};

const REVIEW_COMMUNICATIONS_ATTENTION_PERSISTENCE_THIRD_PARTY_DEPENDENCY_ALLOWLIST = {
  ...REVIEW_COMMUNICATIONS_ATTENTION_CONTRACT_CORE_THIRD_PARTY_DEPENDENCY_ALLOWLIST,
  'makosh-review-attention-persistence': [
    { name: 'sha2', kind: 'normal', source: 'crates_io', version: '=0.11.0', defaultFeatures: false, features: [] },
    { name: 'sqlx', kind: 'normal', source: 'crates_io', version: '=0.9.0', defaultFeatures: false, features: ['postgres', 'runtime-tokio', 'tls-rustls-ring'] },
  ],
};

const REVIEW_COMMUNICATIONS_ATTENTION_MANAGED_RUNTIME_THIRD_PARTY_DEPENDENCY_ALLOWLIST = {
  ...REVIEW_COMMUNICATIONS_ATTENTION_PERSISTENCE_THIRD_PARTY_DEPENDENCY_ALLOWLIST,
  'makosh-review-attention-runtime': [
    { name: 'libc', kind: 'normal', source: 'crates_io', version: '=0.2.186', defaultFeatures: true, features: [] },
    { name: 'prost', kind: 'normal', source: 'crates_io', version: '=0.14.4', defaultFeatures: true, features: [] },
    { name: 'sha2', kind: 'normal', source: 'crates_io', version: '=0.11.0', defaultFeatures: false, features: [] },
    { name: 'tokio', kind: 'normal', source: 'crates_io', version: '=1.52.4', defaultFeatures: false, features: ['rt-multi-thread', 'time'] },
    { name: 'zeroize', kind: 'normal', source: 'crates_io', version: '=1.9.0', defaultFeatures: true, features: [] },
  ],
};

const REVIEW_COMMUNICATIONS_ATTENTION_ASSEMBLY_THIRD_PARTY_DEPENDENCY_ALLOWLIST = {
  ...REVIEW_COMMUNICATIONS_ATTENTION_MANAGED_RUNTIME_THIRD_PARTY_DEPENDENCY_ALLOWLIST,
  'makosh-review-attention-assembly': [
    { name: 'prost', kind: 'normal', source: 'crates_io', version: '=0.14.4', defaultFeatures: true, features: [] },
    { name: 'serde', kind: 'normal', source: 'crates_io', version: '=1.0.228', defaultFeatures: false, features: ['derive', 'std'] },
    { name: 'serde_json', kind: 'normal', source: 'crates_io', version: '=1.0.150', defaultFeatures: true, features: [] },
  ],
};

const COMMUNICATIONS_AI_SOURCE_CONTRACT_THIRD_PARTY_DEPENDENCY_ALLOWLIST = {
  ...REVIEW_COMMUNICATIONS_ATTENTION_ASSEMBLY_THIRD_PARTY_DEPENDENCY_ALLOWLIST,
  'makosh-communications-ai-source-api': [
    { name: 'prost', kind: 'normal', source: 'crates_io', version: '=0.14.4', defaultFeatures: true, features: [] },
    { name: 'prost-types', kind: 'normal', source: 'crates_io', version: '=0.14.4', defaultFeatures: true, features: [] },
    { name: 'sha2', kind: 'normal', source: 'crates_io', version: '=0.11.0', defaultFeatures: false, features: [] },
    { name: 'prost-build', kind: 'build', source: 'crates_io', version: '=0.14.4', defaultFeatures: true, features: [] },
    { name: 'protoc-bin-vendored', kind: 'build', source: 'crates_io', version: '=3.2.0', defaultFeatures: true, features: [] },
    { name: 'sha2', kind: 'build', source: 'crates_io', version: '=0.11.0', defaultFeatures: false, features: [] },
  ],
  'makosh-communication-reply-suggestion-api': [
    { name: 'prost', kind: 'normal', source: 'crates_io', version: '=0.14.4', defaultFeatures: true, features: [] },
    { name: 'prost-build', kind: 'build', source: 'crates_io', version: '=0.14.4', defaultFeatures: true, features: [] },
    { name: 'protoc-bin-vendored', kind: 'build', source: 'crates_io', version: '=3.2.0', defaultFeatures: true, features: [] },
    { name: 'sha2', kind: 'build', source: 'crates_io', version: '=0.11.0', defaultFeatures: false, features: [] },
  ],
  'makosh-communication-reply-suggestion-core': [],
  'makosh-communication-reply-suggestion-persistence': [
    { name: 'sha2', kind: 'normal', source: 'crates_io', version: '=0.11.0', defaultFeatures: false, features: [] },
    { name: 'sqlx', kind: 'normal', source: 'crates_io', version: '=0.9.0', defaultFeatures: false, features: ['postgres', 'runtime-tokio', 'tls-rustls-ring'] },
  ],
  'makosh-communication-reply-suggestion-runtime': [
    { name: 'libc', kind: 'normal', source: 'crates_io', version: '=0.2.186', defaultFeatures: true, features: [] },
    { name: 'prost', kind: 'normal', source: 'crates_io', version: '=0.14.4', defaultFeatures: true, features: [] },
    { name: 'sha2', kind: 'normal', source: 'crates_io', version: '=0.11.0', defaultFeatures: false, features: [] },
    { name: 'tokio', kind: 'normal', source: 'crates_io', version: '=1.52.4', defaultFeatures: false, features: ['rt-multi-thread', 'time'] },
    { name: 'zeroize', kind: 'normal', source: 'crates_io', version: '=1.9.0', defaultFeatures: true, features: [] },
  ],
  'makosh-communication-reply-suggestion-assembly': [
    { name: 'prost', kind: 'normal', source: 'crates_io', version: '=0.14.4', defaultFeatures: true, features: [] },
    { name: 'serde', kind: 'normal', source: 'crates_io', version: '=1.0.228', defaultFeatures: false, features: ['derive', 'std'] },
    { name: 'serde_json', kind: 'normal', source: 'crates_io', version: '=1.0.150', defaultFeatures: true, features: [] },
  ],
  'makosh-ai-contracts': [
    { name: 'prost', kind: 'normal', source: 'crates_io', version: '=0.14.4', defaultFeatures: true, features: [] },
    { name: 'sha2', kind: 'normal', source: 'crates_io', version: '=0.11.0', defaultFeatures: false, features: [] },
    { name: 'prost-build', kind: 'build', source: 'crates_io', version: '=0.14.4', defaultFeatures: true, features: [] },
    { name: 'protoc-bin-vendored', kind: 'build', source: 'crates_io', version: '=3.2.0', defaultFeatures: true, features: [] },
    { name: 'sha2', kind: 'build', source: 'crates_io', version: '=0.11.0', defaultFeatures: false, features: [] },
  ],
  'makosh-ai-inference-core': [
    { name: 'sha2', kind: 'normal', source: 'crates_io', version: '=0.11.0', defaultFeatures: false, features: [] },
  ],
  'makosh-ai-inference-persistence': [
    { name: 'sha2', kind: 'normal', source: 'crates_io', version: '=0.11.0', defaultFeatures: false, features: [] },
    { name: 'sqlx', kind: 'normal', source: 'crates_io', version: '=0.9.0', defaultFeatures: false, features: ['postgres', 'runtime-tokio', 'tls-rustls-ring'] },
  ],
  'makosh-ollama-ai-api': [
    { name: 'prost', kind: 'normal', source: 'crates_io', version: '=0.14.4', defaultFeatures: true, features: [] },
  ],
  'makosh-ollama-ai-assembly': [
    { name: 'prost', kind: 'normal', source: 'crates_io', version: '=0.14.4', defaultFeatures: true, features: [] },
    { name: 'serde', kind: 'normal', source: 'crates_io', version: '=1.0.228', defaultFeatures: false, features: ['derive', 'std'] },
    { name: 'serde_json', kind: 'normal', source: 'crates_io', version: '=1.0.150', defaultFeatures: true, features: [] },
  ],
  'makosh-ollama-ai-core': [
    { name: 'serde', kind: 'normal', source: 'crates_io', version: '=1.0.228', defaultFeatures: false, features: ['derive', 'std'] },
    { name: 'serde_json', kind: 'normal', source: 'crates_io', version: '=1.0.150', defaultFeatures: true, features: [] },
    { name: 'zeroize', kind: 'normal', source: 'crates_io', version: '=1.9.0', defaultFeatures: true, features: [] },
  ],
  'makosh-ollama-ai-http': [
    { name: 'async-std', kind: 'normal', source: 'crates_io', version: '=1.13.2', defaultFeatures: true, features: [] },
    { name: 'serde', kind: 'normal', source: 'crates_io', version: '=1.0.228', defaultFeatures: false, features: ['derive', 'std'] },
    { name: 'serde_json', kind: 'normal', source: 'crates_io', version: '=1.0.150', defaultFeatures: true, features: [] },
    { name: 'zeroize', kind: 'normal', source: 'crates_io', version: '=1.9.0', defaultFeatures: true, features: [] },
  ],
  'makosh-ollama-ai-persistence': [
    { name: 'sha2', kind: 'normal', source: 'crates_io', version: '=0.11.0', defaultFeatures: false, features: [] },
    { name: 'sqlx', kind: 'normal', source: 'crates_io', version: '=0.9.0', defaultFeatures: false, features: ['postgres', 'runtime-tokio', 'tls-rustls-ring'] },
  ],
  'makosh-ollama-ai-runtime': [
    { name: 'libc', kind: 'normal', source: 'crates_io', version: '=0.2.186', defaultFeatures: true, features: [] },
    { name: 'prost', kind: 'normal', source: 'crates_io', version: '=0.14.4', defaultFeatures: true, features: [] },
    { name: 'sha2', kind: 'normal', source: 'crates_io', version: '=0.11.0', defaultFeatures: false, features: [] },
    { name: 'tokio', kind: 'normal', source: 'crates_io', version: '=1.52.4', defaultFeatures: false, features: ['rt-multi-thread', 'time'] },
    { name: 'zeroize', kind: 'normal', source: 'crates_io', version: '=1.9.0', defaultFeatures: true, features: [] },
  ],
};

const ATTACHMENT_ARCHIVE_INSPECTION_CONTRACT_CORE_THIRD_PARTY_DEPENDENCY_ALLOWLIST = {
  ...COMMUNICATIONS_AI_SOURCE_CONTRACT_THIRD_PARTY_DEPENDENCY_ALLOWLIST,
  'makosh-attachment-archive-inspection-api': [
    { name: 'prost', kind: 'normal', source: 'crates_io', version: '=0.14.4', defaultFeatures: true, features: [] },
    { name: 'prost-build', kind: 'build', source: 'crates_io', version: '=0.14.4', defaultFeatures: true, features: [] },
    { name: 'protoc-bin-vendored', kind: 'build', source: 'crates_io', version: '=3.2.0', defaultFeatures: true, features: [] },
    { name: 'sha2', kind: 'build', source: 'crates_io', version: '=0.11.0', defaultFeatures: false, features: [] },
  ],
  'makosh-attachment-archive-inspection-ingress': [
    { name: 'prost', kind: 'normal', source: 'crates_io', version: '=0.14.4', defaultFeatures: true, features: [] },
    { name: 'prost-types', kind: 'normal', source: 'crates_io', version: '=0.14.4', defaultFeatures: true, features: [] },
    { name: 'sha2', kind: 'normal', source: 'crates_io', version: '=0.11.0', defaultFeatures: false, features: [] },
    { name: 'prost-build', kind: 'build', source: 'crates_io', version: '=0.14.4', defaultFeatures: true, features: [] },
    { name: 'protoc-bin-vendored', kind: 'build', source: 'crates_io', version: '=3.2.0', defaultFeatures: true, features: [] },
    { name: 'sha2', kind: 'build', source: 'crates_io', version: '=0.11.0', defaultFeatures: false, features: [] },
  ],
  'makosh-attachment-archive-inspection-core': [
    { name: 'sha2', kind: 'normal', source: 'crates_io', version: '=0.11.0', defaultFeatures: false, features: [] },
  ],
  'makosh-attachment-archive-inspection-zip': [
    { name: 'zip', kind: 'normal', source: 'crates_io', version: '=6.0.0', defaultFeatures: false, features: ['deflate-flate2-zlib-rs'] },
  ],
};

const ATTACHMENT_ARCHIVE_INSPECTION_PERSISTENCE_THIRD_PARTY_DEPENDENCY_ALLOWLIST = {
  ...ATTACHMENT_ARCHIVE_INSPECTION_CONTRACT_CORE_THIRD_PARTY_DEPENDENCY_ALLOWLIST,
  'makosh-attachment-archive-inspection-persistence': [
    { name: 'prost', kind: 'normal', source: 'crates_io', version: '=0.14.4', defaultFeatures: true, features: [] },
    { name: 'sha2', kind: 'normal', source: 'crates_io', version: '=0.11.0', defaultFeatures: false, features: [] },
    { name: 'sqlx', kind: 'normal', source: 'crates_io', version: '=0.9.0', defaultFeatures: false, features: ['postgres', 'runtime-tokio', 'tls-rustls-ring'] },
  ],
};

const ATTACHMENT_ARCHIVE_INSPECTION_RUNTIME_THIRD_PARTY_DEPENDENCY_ALLOWLIST = {
  ...ATTACHMENT_ARCHIVE_INSPECTION_PERSISTENCE_THIRD_PARTY_DEPENDENCY_ALLOWLIST,
  'makosh-attachment-archive-inspection-runtime': [
    { name: 'libc', kind: 'normal', source: 'crates_io', version: '=0.2.186', defaultFeatures: true, features: [] },
    { name: 'prost', kind: 'normal', source: 'crates_io', version: '=0.14.4', defaultFeatures: true, features: [] },
    { name: 'prost-types', kind: 'normal', source: 'crates_io', version: '=0.14.4', defaultFeatures: true, features: [] },
    { name: 'sha2', kind: 'normal', source: 'crates_io', version: '=0.11.0', defaultFeatures: false, features: [] },
    { name: 'tokio', kind: 'normal', source: 'crates_io', version: '=1.52.4', defaultFeatures: false, features: ['rt-multi-thread', 'time'] },
    { name: 'zeroize', kind: 'normal', source: 'crates_io', version: '=1.9.0', defaultFeatures: true, features: [] },
  ],
};

const ATTACHMENT_ARCHIVE_INSPECTION_ASSEMBLY_THIRD_PARTY_DEPENDENCY_ALLOWLIST = {
  ...ATTACHMENT_ARCHIVE_INSPECTION_RUNTIME_THIRD_PARTY_DEPENDENCY_ALLOWLIST,
  'makosh-attachment-archive-inspection-assembly': [
    { name: 'prost', kind: 'normal', source: 'crates_io', version: '=0.14.4', defaultFeatures: true, features: [] },
    { name: 'serde', kind: 'normal', source: 'crates_io', version: '=1.0.228', defaultFeatures: false, features: ['derive', 'std'] },
    { name: 'serde_json', kind: 'normal', source: 'crates_io', version: '=1.0.150', defaultFeatures: true, features: [] },
  ],
};

const COMMUNICATION_SUMMARY_BUILD_UNITS_THIRD_PARTY_DEPENDENCY_ALLOWLIST = {
  ...ATTACHMENT_ARCHIVE_INSPECTION_ASSEMBLY_THIRD_PARTY_DEPENDENCY_ALLOWLIST,
  'makosh-communication-summary-api': [
    { name: 'prost', kind: 'normal', source: 'crates_io', version: '=0.14.4', defaultFeatures: true, features: [] },
    { name: 'prost-build', kind: 'build', source: 'crates_io', version: '=0.14.4', defaultFeatures: true, features: [] },
    { name: 'protoc-bin-vendored', kind: 'build', source: 'crates_io', version: '=3.2.0', defaultFeatures: true, features: [] },
    { name: 'sha2', kind: 'build', source: 'crates_io', version: '=0.11.0', defaultFeatures: false, features: [] },
  ],
  'makosh-communication-summary-core': [],
  'makosh-communication-summary-persistence': [
    { name: 'sha2', kind: 'normal', source: 'crates_io', version: '=0.11.0', defaultFeatures: false, features: [] },
    { name: 'sqlx', kind: 'normal', source: 'crates_io', version: '=0.9.0', defaultFeatures: false, features: ['postgres', 'runtime-tokio', 'tls-rustls-ring'] },
  ],
  'makosh-communication-summary-runtime': [
    { name: 'libc', kind: 'normal', source: 'crates_io', version: '=0.2.186', defaultFeatures: true, features: [] },
    { name: 'prost', kind: 'normal', source: 'crates_io', version: '=0.14.4', defaultFeatures: true, features: [] },
    { name: 'sha2', kind: 'normal', source: 'crates_io', version: '=0.11.0', defaultFeatures: false, features: [] },
    { name: 'tokio', kind: 'normal', source: 'crates_io', version: '=1.52.4', defaultFeatures: false, features: ['rt-multi-thread', 'time'] },
    { name: 'zeroize', kind: 'normal', source: 'crates_io', version: '=1.9.0', defaultFeatures: true, features: [] },
  ],
  'makosh-communication-summary-assembly': [
    { name: 'prost', kind: 'normal', source: 'crates_io', version: '=0.14.4', defaultFeatures: true, features: [] },
    { name: 'serde', kind: 'normal', source: 'crates_io', version: '=1.0.228', defaultFeatures: false, features: ['derive', 'std'] },
    { name: 'serde_json', kind: 'normal', source: 'crates_io', version: '=1.0.150', defaultFeatures: true, features: [] },
  ],
};

const COMMUNICATION_TRANSLATION_CONTRACT_CORE_THIRD_PARTY_DEPENDENCY_ALLOWLIST = {
  ...COMMUNICATION_SUMMARY_BUILD_UNITS_THIRD_PARTY_DEPENDENCY_ALLOWLIST,
  'makosh-communication-translation-api': [
    { name: 'prost', kind: 'normal', source: 'crates_io', version: '=0.14.4', defaultFeatures: true, features: [] },
    { name: 'prost-build', kind: 'build', source: 'crates_io', version: '=0.14.4', defaultFeatures: true, features: [] },
    { name: 'protoc-bin-vendored', kind: 'build', source: 'crates_io', version: '=3.2.0', defaultFeatures: true, features: [] },
    { name: 'sha2', kind: 'build', source: 'crates_io', version: '=0.11.0', defaultFeatures: false, features: [] },
  ],
  'makosh-communication-translation-core': [],
};

const COMMUNICATION_TRANSLATION_PERSISTENCE_THIRD_PARTY_DEPENDENCY_ALLOWLIST = {
  ...COMMUNICATION_TRANSLATION_CONTRACT_CORE_THIRD_PARTY_DEPENDENCY_ALLOWLIST,
  'makosh-communication-translation-persistence': [
    { name: 'sha2', kind: 'normal', source: 'crates_io', version: '=0.11.0', defaultFeatures: false, features: [] },
    { name: 'sqlx', kind: 'normal', source: 'crates_io', version: '=0.9.0', defaultFeatures: false, features: ['postgres', 'runtime-tokio', 'tls-rustls-ring'] },
  ],
};

const COMMUNICATION_TRANSLATION_RUNTIME_THIRD_PARTY_DEPENDENCY_ALLOWLIST = {
  ...COMMUNICATION_TRANSLATION_PERSISTENCE_THIRD_PARTY_DEPENDENCY_ALLOWLIST,
  'makosh-communication-translation-runtime': [
    { name: 'libc', kind: 'normal', source: 'crates_io', version: '=0.2.186', defaultFeatures: true, features: [] },
    { name: 'prost', kind: 'normal', source: 'crates_io', version: '=0.14.4', defaultFeatures: true, features: [] },
    { name: 'sha2', kind: 'normal', source: 'crates_io', version: '=0.11.0', defaultFeatures: false, features: [] },
    { name: 'tokio', kind: 'normal', source: 'crates_io', version: '=1.52.4', defaultFeatures: false, features: ['rt-multi-thread', 'time'] },
    { name: 'zeroize', kind: 'normal', source: 'crates_io', version: '=1.9.0', defaultFeatures: true, features: [] },
  ],
};

const COMMUNICATION_TRANSLATION_ASSEMBLY_THIRD_PARTY_DEPENDENCY_ALLOWLIST = {
  ...COMMUNICATION_TRANSLATION_RUNTIME_THIRD_PARTY_DEPENDENCY_ALLOWLIST,
  'makosh-communication-translation-assembly': [
    { name: 'prost', kind: 'normal', source: 'crates_io', version: '=0.14.4', defaultFeatures: true, features: [] },
    { name: 'serde', kind: 'normal', source: 'crates_io', version: '=1.0.228', defaultFeatures: false, features: ['derive', 'std'] },
    { name: 'serde_json', kind: 'normal', source: 'crates_io', version: '=1.0.150', defaultFeatures: true, features: [] },
  ],
};

const COMMUNICATION_EXPLANATION_CONTRACT_CORE_THIRD_PARTY_DEPENDENCY_ALLOWLIST = {
  ...COMMUNICATION_TRANSLATION_ASSEMBLY_THIRD_PARTY_DEPENDENCY_ALLOWLIST,
  'makosh-communication-explanation-api': [
    { name: 'prost', kind: 'normal', source: 'crates_io', version: '=0.14.4', defaultFeatures: true, features: [] },
    { name: 'prost-build', kind: 'build', source: 'crates_io', version: '=0.14.4', defaultFeatures: true, features: [] },
    { name: 'protoc-bin-vendored', kind: 'build', source: 'crates_io', version: '=3.2.0', defaultFeatures: true, features: [] },
    { name: 'sha2', kind: 'build', source: 'crates_io', version: '=0.11.0', defaultFeatures: false, features: [] },
  ],
  'makosh-communication-explanation-core': [],
};

const COMMUNICATION_EXPLANATION_PERSISTENCE_THIRD_PARTY_DEPENDENCY_ALLOWLIST = {
  ...COMMUNICATION_EXPLANATION_CONTRACT_CORE_THIRD_PARTY_DEPENDENCY_ALLOWLIST,
  'makosh-communication-explanation-persistence': [
    { name: 'sha2', kind: 'normal', source: 'crates_io', version: '=0.11.0', defaultFeatures: false, features: [] },
    { name: 'sqlx', kind: 'normal', source: 'crates_io', version: '=0.9.0', defaultFeatures: false, features: ['postgres', 'runtime-tokio', 'tls-rustls-ring'] },
  ],
};

const COMMUNICATION_EXPLANATION_RUNTIME_THIRD_PARTY_DEPENDENCY_ALLOWLIST = {
  ...COMMUNICATION_EXPLANATION_PERSISTENCE_THIRD_PARTY_DEPENDENCY_ALLOWLIST,
  'makosh-communication-explanation-runtime': [
    { name: 'libc', kind: 'normal', source: 'crates_io', version: '=0.2.186', defaultFeatures: true, features: [] },
    { name: 'prost', kind: 'normal', source: 'crates_io', version: '=0.14.4', defaultFeatures: true, features: [] },
    { name: 'sha2', kind: 'normal', source: 'crates_io', version: '=0.11.0', defaultFeatures: false, features: [] },
    { name: 'tokio', kind: 'normal', source: 'crates_io', version: '=1.52.4', defaultFeatures: false, features: ['rt-multi-thread', 'time'] },
    { name: 'zeroize', kind: 'normal', source: 'crates_io', version: '=1.9.0', defaultFeatures: true, features: [] },
  ],
};

const COMMUNICATION_EXPLANATION_ASSEMBLY_THIRD_PARTY_DEPENDENCY_ALLOWLIST = {
  ...COMMUNICATION_EXPLANATION_RUNTIME_THIRD_PARTY_DEPENDENCY_ALLOWLIST,
  'makosh-communication-explanation-assembly': [
    { name: 'prost', kind: 'normal', source: 'crates_io', version: '=0.14.4', defaultFeatures: true, features: [] },
    { name: 'serde', kind: 'normal', source: 'crates_io', version: '=1.0.228', defaultFeatures: false, features: ['derive', 'std'] },
    { name: 'serde_json', kind: 'normal', source: 'crates_io', version: '=1.0.150', defaultFeatures: true, features: [] },
  ],
};

const COMMUNICATION_RECIPIENT_SUGGESTION_CONTRACT_CORE_THIRD_PARTY_DEPENDENCY_ALLOWLIST = {
  ...COMMUNICATION_EXPLANATION_ASSEMBLY_THIRD_PARTY_DEPENDENCY_ALLOWLIST,
  'makosh-communication-recipient-suggestion-api': [
    { name: 'prost', kind: 'normal', source: 'crates_io', version: '=0.14.4', defaultFeatures: true, features: [] },
    { name: 'prost-build', kind: 'build', source: 'crates_io', version: '=0.14.4', defaultFeatures: true, features: [] },
    { name: 'protoc-bin-vendored', kind: 'build', source: 'crates_io', version: '=3.2.0', defaultFeatures: true, features: [] },
    { name: 'sha2', kind: 'build', source: 'crates_io', version: '=0.11.0', defaultFeatures: false, features: [] },
  ],
  'makosh-communication-recipient-suggestion-core': [],
};

const COMMUNICATION_RECIPIENT_SUGGESTION_SOURCE_CONTRACT_THIRD_PARTY_DEPENDENCY_ALLOWLIST = {
  ...COMMUNICATION_RECIPIENT_SUGGESTION_CONTRACT_CORE_THIRD_PARTY_DEPENDENCY_ALLOWLIST,
  'makosh-communications-recipient-source-api': [
    { name: 'prost', kind: 'normal', source: 'crates_io', version: '=0.14.4', defaultFeatures: true, features: [] },
    { name: 'prost-types', kind: 'normal', source: 'crates_io', version: '=0.14.4', defaultFeatures: true, features: [] },
    { name: 'sha2', kind: 'normal', source: 'crates_io', version: '=0.11.0', defaultFeatures: false, features: [] },
    { name: 'prost-build', kind: 'build', source: 'crates_io', version: '=0.14.4', defaultFeatures: true, features: [] },
    { name: 'protoc-bin-vendored', kind: 'build', source: 'crates_io', version: '=3.2.0', defaultFeatures: true, features: [] },
    { name: 'sha2', kind: 'build', source: 'crates_io', version: '=0.11.0', defaultFeatures: false, features: [] },
  ],
};

const COMMUNICATION_RECIPIENT_SUGGESTION_PERSISTENCE_THIRD_PARTY_DEPENDENCY_ALLOWLIST = {
  ...COMMUNICATION_RECIPIENT_SUGGESTION_CONTRACT_CORE_THIRD_PARTY_DEPENDENCY_ALLOWLIST,
  'makosh-communication-recipient-suggestion-persistence': [
    { name: 'sha2', kind: 'normal', source: 'crates_io', version: '=0.11.0', defaultFeatures: false, features: [] },
    { name: 'sqlx', kind: 'normal', source: 'crates_io', version: '=0.9.0', defaultFeatures: false, features: ['postgres', 'runtime-tokio', 'tls-rustls-ring'] },
  ],
  'makosh-communications-recipient-source-api': [
    { name: 'prost', kind: 'normal', source: 'crates_io', version: '=0.14.4', defaultFeatures: true, features: [] },
    { name: 'prost-types', kind: 'normal', source: 'crates_io', version: '=0.14.4', defaultFeatures: true, features: [] },
    { name: 'sha2', kind: 'normal', source: 'crates_io', version: '=0.11.0', defaultFeatures: false, features: [] },
    { name: 'prost-build', kind: 'build', source: 'crates_io', version: '=0.14.4', defaultFeatures: true, features: [] },
    { name: 'protoc-bin-vendored', kind: 'build', source: 'crates_io', version: '=3.2.0', defaultFeatures: true, features: [] },
    { name: 'sha2', kind: 'build', source: 'crates_io', version: '=0.11.0', defaultFeatures: false, features: [] },
  ],
};

const COMMUNICATION_RECIPIENT_SUGGESTION_RUNTIME_THIRD_PARTY_DEPENDENCY_ALLOWLIST = {
  ...COMMUNICATION_RECIPIENT_SUGGESTION_CONTRACT_CORE_THIRD_PARTY_DEPENDENCY_ALLOWLIST,
  'makosh-communication-recipient-suggestion-persistence': [
    { name: 'sha2', kind: 'normal', source: 'crates_io', version: '=0.11.0', defaultFeatures: false, features: [] },
    { name: 'sqlx', kind: 'normal', source: 'crates_io', version: '=0.9.0', defaultFeatures: false, features: ['postgres', 'runtime-tokio', 'tls-rustls-ring'] },
  ],
  'makosh-communication-recipient-suggestion-runtime': [
    { name: 'libc', kind: 'normal', source: 'crates_io', version: '=0.2.186', defaultFeatures: true, features: [] },
    { name: 'prost', kind: 'normal', source: 'crates_io', version: '=0.14.4', defaultFeatures: true, features: [] },
    { name: 'sha2', kind: 'normal', source: 'crates_io', version: '=0.11.0', defaultFeatures: false, features: [] },
    { name: 'tokio', kind: 'normal', source: 'crates_io', version: '=1.52.4', defaultFeatures: false, features: ['rt-multi-thread', 'time'] },
    { name: 'zeroize', kind: 'normal', source: 'crates_io', version: '=1.9.0', defaultFeatures: true, features: [] },
  ],
  'makosh-communications-recipient-source-api': [
    { name: 'prost', kind: 'normal', source: 'crates_io', version: '=0.14.4', defaultFeatures: true, features: [] },
    { name: 'prost-types', kind: 'normal', source: 'crates_io', version: '=0.14.4', defaultFeatures: true, features: [] },
    { name: 'sha2', kind: 'normal', source: 'crates_io', version: '=0.11.0', defaultFeatures: false, features: [] },
    { name: 'prost-build', kind: 'build', source: 'crates_io', version: '=0.14.4', defaultFeatures: true, features: [] },
    { name: 'protoc-bin-vendored', kind: 'build', source: 'crates_io', version: '=3.2.0', defaultFeatures: true, features: [] },
    { name: 'sha2', kind: 'build', source: 'crates_io', version: '=0.11.0', defaultFeatures: false, features: [] },
  ],
};

const COMMUNICATION_RECIPIENT_SUGGESTION_ASSEMBLY_THIRD_PARTY_DEPENDENCY_ALLOWLIST = {
  ...COMMUNICATION_RECIPIENT_SUGGESTION_RUNTIME_THIRD_PARTY_DEPENDENCY_ALLOWLIST,
  'makosh-communication-recipient-suggestion-assembly': [
    { name: 'prost', kind: 'normal', source: 'crates_io', version: '=0.14.4', defaultFeatures: true, features: [] },
    { name: 'serde', kind: 'normal', source: 'crates_io', version: '=1.0.228', defaultFeatures: false, features: ['derive', 'std'] },
    { name: 'serde_json', kind: 'normal', source: 'crates_io', version: '=1.0.150', defaultFeatures: true, features: [] },
  ],
};

const COMMUNICATION_TASK_CANDIDATE_CONTRACT_CORE_SOURCE_THIRD_PARTY_DEPENDENCY_ALLOWLIST = {
  ...COMMUNICATION_RECIPIENT_SUGGESTION_ASSEMBLY_THIRD_PARTY_DEPENDENCY_ALLOWLIST,
  'makosh-communication-task-candidate-api': [
    { name: 'prost', kind: 'normal', source: 'crates_io', version: '=0.14.4', defaultFeatures: true, features: [] },
    { name: 'prost-build', kind: 'build', source: 'crates_io', version: '=0.14.4', defaultFeatures: true, features: [] },
    { name: 'protoc-bin-vendored', kind: 'build', source: 'crates_io', version: '=3.2.0', defaultFeatures: true, features: [] },
    { name: 'sha2', kind: 'build', source: 'crates_io', version: '=0.11.0', defaultFeatures: false, features: [] },
  ],
  'makosh-communication-task-candidate-core': [
    { name: 'sha2', kind: 'normal', source: 'crates_io', version: '=0.11.0', defaultFeatures: false, features: [] },
  ],
  'makosh-communications-task-source-api': [
    { name: 'prost', kind: 'normal', source: 'crates_io', version: '=0.14.4', defaultFeatures: true, features: [] },
    { name: 'prost-types', kind: 'normal', source: 'crates_io', version: '=0.14.4', defaultFeatures: true, features: [] },
    { name: 'sha2', kind: 'normal', source: 'crates_io', version: '=0.11.0', defaultFeatures: false, features: [] },
    { name: 'prost-build', kind: 'build', source: 'crates_io', version: '=0.14.4', defaultFeatures: true, features: [] },
    { name: 'protoc-bin-vendored', kind: 'build', source: 'crates_io', version: '=3.2.0', defaultFeatures: true, features: [] },
    { name: 'sha2', kind: 'build', source: 'crates_io', version: '=0.11.0', defaultFeatures: false, features: [] },
  ],
};

const COMMUNICATION_TASK_CANDIDATE_PERSISTENCE_THIRD_PARTY_DEPENDENCY_ALLOWLIST = {
  ...COMMUNICATION_TASK_CANDIDATE_CONTRACT_CORE_SOURCE_THIRD_PARTY_DEPENDENCY_ALLOWLIST,
  'makosh-communication-task-candidate-persistence': [
    { name: 'sha2', kind: 'normal', source: 'crates_io', version: '=0.11.0', defaultFeatures: false, features: [] },
    { name: 'sqlx', kind: 'normal', source: 'crates_io', version: '=0.9.0', defaultFeatures: false, features: ['postgres', 'runtime-tokio', 'tls-rustls-ring'] },
  ],
};

const COMMUNICATION_TASK_CANDIDATE_RUNTIME_THIRD_PARTY_DEPENDENCY_ALLOWLIST = {
  ...COMMUNICATION_TASK_CANDIDATE_PERSISTENCE_THIRD_PARTY_DEPENDENCY_ALLOWLIST,
  'makosh-communication-task-candidate-runtime': [
    { name: 'libc', kind: 'normal', source: 'crates_io', version: '=0.2.186', defaultFeatures: true, features: [] },
    { name: 'prost', kind: 'normal', source: 'crates_io', version: '=0.14.4', defaultFeatures: true, features: [] },
    { name: 'sha2', kind: 'normal', source: 'crates_io', version: '=0.11.0', defaultFeatures: false, features: [] },
    { name: 'tokio', kind: 'normal', source: 'crates_io', version: '=1.52.4', defaultFeatures: false, features: ['rt-multi-thread', 'time'] },
    { name: 'zeroize', kind: 'normal', source: 'crates_io', version: '=1.9.0', defaultFeatures: true, features: [] },
  ],
};

const COMMUNICATION_TASK_CANDIDATE_ASSEMBLY_THIRD_PARTY_DEPENDENCY_ALLOWLIST = {
  ...COMMUNICATION_TASK_CANDIDATE_RUNTIME_THIRD_PARTY_DEPENDENCY_ALLOWLIST,
  'makosh-communication-task-candidate-assembly': [
    { name: 'prost', kind: 'normal', source: 'crates_io', version: '=0.14.4', defaultFeatures: true, features: [] },
    { name: 'serde', kind: 'normal', source: 'crates_io', version: '=1.0.228', defaultFeatures: false, features: ['derive', 'std'] },
    { name: 'serde_json', kind: 'normal', source: 'crates_io', version: '=1.0.150', defaultFeatures: true, features: [] },
  ],
};

const REVIEW_TASK_CANDIDATE_CORE_THIRD_PARTY_DEPENDENCY_ALLOWLIST = {
  ...COMMUNICATION_TASK_CANDIDATE_ASSEMBLY_THIRD_PARTY_DEPENDENCY_ALLOWLIST,
  'makosh-review-task-candidate-api': [
    { name: 'prost', kind: 'normal', source: 'crates_io', version: '=0.14.4', defaultFeatures: true, features: [] },
    { name: 'prost-types', kind: 'normal', source: 'crates_io', version: '=0.14.4', defaultFeatures: true, features: [] },
    { name: 'sha2', kind: 'normal', source: 'crates_io', version: '=0.11.0', defaultFeatures: false, features: [] },
    { name: 'prost-build', kind: 'build', source: 'crates_io', version: '=0.14.4', defaultFeatures: true, features: [] },
    { name: 'protoc-bin-vendored', kind: 'build', source: 'crates_io', version: '=3.2.0', defaultFeatures: true, features: [] },
    { name: 'sha2', kind: 'build', source: 'crates_io', version: '=0.11.0', defaultFeatures: false, features: [] },
  ],
  'makosh-review-task-candidate-core': [
    { name: 'sha2', kind: 'normal', source: 'crates_io', version: '=0.11.0', defaultFeatures: false, features: [] },
  ],
};

const REVIEW_TASK_CANDIDATE_PERSISTENCE_THIRD_PARTY_DEPENDENCY_ALLOWLIST = {
  ...REVIEW_TASK_CANDIDATE_CORE_THIRD_PARTY_DEPENDENCY_ALLOWLIST,
  'makosh-review-task-candidate-persistence': [
    { name: 'sha2', kind: 'normal', source: 'crates_io', version: '=0.11.0', defaultFeatures: false, features: [] },
    { name: 'sqlx', kind: 'normal', source: 'crates_io', version: '=0.9.0', defaultFeatures: false, features: ['postgres', 'runtime-tokio', 'tls-rustls-ring'] },
  ],
};

const REVIEW_TASK_CANDIDATE_MANAGED_RUNTIME_THIRD_PARTY_DEPENDENCY_ALLOWLIST = {
  ...REVIEW_TASK_CANDIDATE_PERSISTENCE_THIRD_PARTY_DEPENDENCY_ALLOWLIST,
  'makosh-review-task-candidate-runtime': [
    { name: 'libc', kind: 'normal', source: 'crates_io', version: '=0.2.186', defaultFeatures: true, features: [] },
    { name: 'prost', kind: 'normal', source: 'crates_io', version: '=0.14.4', defaultFeatures: true, features: [] },
    { name: 'sha2', kind: 'normal', source: 'crates_io', version: '=0.11.0', defaultFeatures: false, features: [] },
    { name: 'tokio', kind: 'normal', source: 'crates_io', version: '=1.52.4', defaultFeatures: false, features: ['rt-multi-thread', 'time'] },
    { name: 'zeroize', kind: 'normal', source: 'crates_io', version: '=1.9.0', defaultFeatures: true, features: [] },
  ],
};

const REVIEW_TASK_CANDIDATE_ASSEMBLY_THIRD_PARTY_DEPENDENCY_ALLOWLIST = {
  ...REVIEW_TASK_CANDIDATE_MANAGED_RUNTIME_THIRD_PARTY_DEPENDENCY_ALLOWLIST,
  'makosh-review-task-candidate-assembly': [
    { name: 'prost', kind: 'normal', source: 'crates_io', version: '=0.14.4', defaultFeatures: true, features: [] },
    { name: 'serde', kind: 'normal', source: 'crates_io', version: '=1.0.228', defaultFeatures: false, features: ['derive', 'std'] },
    { name: 'serde_json', kind: 'normal', source: 'crates_io', version: '=1.0.150', defaultFeatures: true, features: [] },
  ],
};

const TASKS_REVIEWED_CANDIDATE_CONTRACT_CORE_THIRD_PARTY_DEPENDENCY_ALLOWLIST = {
  ...REVIEW_TASK_CANDIDATE_ASSEMBLY_THIRD_PARTY_DEPENDENCY_ALLOWLIST,
  'makosh-tasks-command-api': [
    { name: 'prost', kind: 'normal', source: 'crates_io', version: '=0.14.4', defaultFeatures: true, features: [] },
    { name: 'prost-types', kind: 'normal', source: 'crates_io', version: '=0.14.4', defaultFeatures: true, features: [] },
    { name: 'sha2', kind: 'normal', source: 'crates_io', version: '=0.11.0', defaultFeatures: false, features: [] },
    { name: 'prost-build', kind: 'build', source: 'crates_io', version: '=0.14.4', defaultFeatures: true, features: [] },
    { name: 'protoc-bin-vendored', kind: 'build', source: 'crates_io', version: '=3.2.0', defaultFeatures: true, features: [] },
    { name: 'sha2', kind: 'build', source: 'crates_io', version: '=0.11.0', defaultFeatures: false, features: [] },
  ],
  'makosh-tasks-core': [
    { name: 'sha2', kind: 'normal', source: 'crates_io', version: '=0.11.0', defaultFeatures: false, features: [] },
  ],
};

const TASKS_REVIEWED_CANDIDATE_PERSISTENCE_THIRD_PARTY_DEPENDENCY_ALLOWLIST = {
  ...TASKS_REVIEWED_CANDIDATE_CONTRACT_CORE_THIRD_PARTY_DEPENDENCY_ALLOWLIST,
  'makosh-tasks-persistence': [
    { name: 'sha2', kind: 'normal', source: 'crates_io', version: '=0.11.0', defaultFeatures: false, features: [] },
    { name: 'sqlx', kind: 'normal', source: 'crates_io', version: '=0.9.0', defaultFeatures: false, features: ['postgres', 'runtime-tokio', 'tls-rustls-ring'] },
  ],
};

const TASKS_REVIEWED_CANDIDATE_MANAGED_RUNTIME_THIRD_PARTY_DEPENDENCY_ALLOWLIST = {
  ...TASKS_REVIEWED_CANDIDATE_PERSISTENCE_THIRD_PARTY_DEPENDENCY_ALLOWLIST,
  'makosh-tasks-runtime': [
    { name: 'libc', kind: 'normal', source: 'crates_io', version: '=0.2.186', defaultFeatures: true, features: [] },
    { name: 'prost', kind: 'normal', source: 'crates_io', version: '=0.14.4', defaultFeatures: true, features: [] },
    { name: 'sha2', kind: 'normal', source: 'crates_io', version: '=0.11.0', defaultFeatures: false, features: [] },
    { name: 'tokio', kind: 'normal', source: 'crates_io', version: '=1.52.4', defaultFeatures: false, features: ['rt-multi-thread', 'time'] },
    { name: 'zeroize', kind: 'normal', source: 'crates_io', version: '=1.9.0', defaultFeatures: true, features: [] },
  ],
};

const TASKS_REVIEWED_CANDIDATE_ASSEMBLY_THIRD_PARTY_DEPENDENCY_ALLOWLIST = {
  ...TASKS_REVIEWED_CANDIDATE_MANAGED_RUNTIME_THIRD_PARTY_DEPENDENCY_ALLOWLIST,
  'makosh-tasks-assembly': [
    { name: 'prost', kind: 'normal', source: 'crates_io', version: '=0.14.4', defaultFeatures: true, features: [] },
    { name: 'serde', kind: 'normal', source: 'crates_io', version: '=1.0.228', defaultFeatures: false, features: ['derive', 'std'] },
    { name: 'serde_json', kind: 'normal', source: 'crates_io', version: '=1.0.150', defaultFeatures: true, features: [] },
  ],
};

const REVIEWED_TASK_CANDIDATE_PROMOTION_CONTRACT_CORE_THIRD_PARTY_DEPENDENCY_ALLOWLIST = {
  ...TASKS_REVIEWED_CANDIDATE_ASSEMBLY_THIRD_PARTY_DEPENDENCY_ALLOWLIST,
  'makosh-review-task-candidate-promotion-api': [
    { name: 'prost', kind: 'normal', source: 'crates_io', version: '=0.14.4', defaultFeatures: true, features: [] },
    { name: 'prost-types', kind: 'normal', source: 'crates_io', version: '=0.14.4', defaultFeatures: true, features: [] },
    { name: 'sha2', kind: 'normal', source: 'crates_io', version: '=0.11.0', defaultFeatures: false, features: [] },
    { name: 'prost-build', kind: 'build', source: 'crates_io', version: '=0.14.4', defaultFeatures: true, features: [] },
    { name: 'protoc-bin-vendored', kind: 'build', source: 'crates_io', version: '=3.2.0', defaultFeatures: true, features: [] },
    { name: 'sha2', kind: 'build', source: 'crates_io', version: '=0.11.0', defaultFeatures: false, features: [] },
  ],
  'makosh-reviewed-task-candidate-promotion-core': [
    { name: 'sha2', kind: 'normal', source: 'crates_io', version: '=0.11.0', defaultFeatures: false, features: [] },
  ],
};

const REVIEWED_TASK_CANDIDATE_PROMOTION_PERSISTENCE_THIRD_PARTY_DEPENDENCY_ALLOWLIST = {
  ...REVIEWED_TASK_CANDIDATE_PROMOTION_CONTRACT_CORE_THIRD_PARTY_DEPENDENCY_ALLOWLIST,
  'makosh-reviewed-task-candidate-promotion-persistence': [
    { name: 'sha2', kind: 'normal', source: 'crates_io', version: '=0.11.0', defaultFeatures: false, features: [] },
    { name: 'sqlx', kind: 'normal', source: 'crates_io', version: '=0.9.0', defaultFeatures: false, features: ['postgres', 'runtime-tokio', 'tls-rustls-ring'] },
  ],
};

const REVIEWED_TASK_CANDIDATE_PROMOTION_RUNTIME_THIRD_PARTY_DEPENDENCY_ALLOWLIST = {
  ...REVIEWED_TASK_CANDIDATE_PROMOTION_PERSISTENCE_THIRD_PARTY_DEPENDENCY_ALLOWLIST,
  'makosh-reviewed-task-candidate-promotion-runtime': [
    { name: 'libc', kind: 'normal', source: 'crates_io', version: '=0.2.186', defaultFeatures: true, features: [] },
    { name: 'prost', kind: 'normal', source: 'crates_io', version: '=0.14.4', defaultFeatures: true, features: [] },
    { name: 'sha2', kind: 'normal', source: 'crates_io', version: '=0.11.0', defaultFeatures: false, features: [] },
    { name: 'tokio', kind: 'normal', source: 'crates_io', version: '=1.52.4', defaultFeatures: false, features: ['rt-multi-thread', 'time'] },
    { name: 'zeroize', kind: 'normal', source: 'crates_io', version: '=1.9.0', defaultFeatures: true, features: [] },
  ],
};

const REVIEWED_TASK_CANDIDATE_PROMOTION_ASSEMBLY_THIRD_PARTY_DEPENDENCY_ALLOWLIST = {
  ...REVIEWED_TASK_CANDIDATE_PROMOTION_RUNTIME_THIRD_PARTY_DEPENDENCY_ALLOWLIST,
  'makosh-reviewed-task-candidate-promotion-assembly': [
    { name: 'prost', kind: 'normal', source: 'crates_io', version: '=0.14.4', defaultFeatures: true, features: [] },
    { name: 'serde', kind: 'normal', source: 'crates_io', version: '=1.0.228', defaultFeatures: false, features: ['derive', 'std'] },
    { name: 'serde_json', kind: 'normal', source: 'crates_io', version: '=1.0.150', defaultFeatures: true, features: [] },
  ],
};

const COMMUNICATION_NOTE_CANDIDATE_CONTRACT_CORE_THIRD_PARTY_DEPENDENCY_ALLOWLIST = {
  ...REVIEWED_TASK_CANDIDATE_PROMOTION_ASSEMBLY_THIRD_PARTY_DEPENDENCY_ALLOWLIST,
  'makosh-communication-note-candidate-api': [
    { name: 'prost', kind: 'normal', source: 'crates_io', version: '=0.14.4', defaultFeatures: true, features: [] },
    { name: 'prost-build', kind: 'build', source: 'crates_io', version: '=0.14.4', defaultFeatures: true, features: [] },
    { name: 'protoc-bin-vendored', kind: 'build', source: 'crates_io', version: '=3.2.0', defaultFeatures: true, features: [] },
    { name: 'sha2', kind: 'build', source: 'crates_io', version: '=0.11.0', defaultFeatures: false, features: [] },
  ],
  'makosh-communication-note-candidate-core': [
    { name: 'sha2', kind: 'normal', source: 'crates_io', version: '=0.11.0', defaultFeatures: false, features: [] },
  ],
  'makosh-communications-note-source-api': [
    { name: 'prost', kind: 'normal', source: 'crates_io', version: '=0.14.4', defaultFeatures: true, features: [] },
    { name: 'prost-types', kind: 'normal', source: 'crates_io', version: '=0.14.4', defaultFeatures: true, features: [] },
    { name: 'sha2', kind: 'normal', source: 'crates_io', version: '=0.11.0', defaultFeatures: false, features: [] },
    { name: 'prost-build', kind: 'build', source: 'crates_io', version: '=0.14.4', defaultFeatures: true, features: [] },
    { name: 'protoc-bin-vendored', kind: 'build', source: 'crates_io', version: '=3.2.0', defaultFeatures: true, features: [] },
    { name: 'sha2', kind: 'build', source: 'crates_io', version: '=0.11.0', defaultFeatures: false, features: [] },
  ],
};

const COMMUNICATION_NOTE_CANDIDATE_PERSISTENCE_THIRD_PARTY_DEPENDENCY_ALLOWLIST = {
  ...COMMUNICATION_NOTE_CANDIDATE_CONTRACT_CORE_THIRD_PARTY_DEPENDENCY_ALLOWLIST,
  'makosh-communication-note-candidate-persistence': [
    { name: 'sha2', kind: 'normal', source: 'crates_io', version: '=0.11.0', defaultFeatures: false, features: [] },
    { name: 'sqlx', kind: 'normal', source: 'crates_io', version: '=0.9.0', defaultFeatures: false, features: ['postgres', 'runtime-tokio', 'tls-rustls-ring'] },
  ],
};

const REVIEW_NOTE_CANDIDATE_CONTRACT_CORE_THIRD_PARTY_DEPENDENCY_ALLOWLIST = {
  ...COMMUNICATION_NOTE_CANDIDATE_PERSISTENCE_THIRD_PARTY_DEPENDENCY_ALLOWLIST,
  'makosh-review-note-candidate-api': [
    { name: 'prost', kind: 'normal', source: 'crates_io', version: '=0.14.4', defaultFeatures: true, features: [] },
    { name: 'prost-types', kind: 'normal', source: 'crates_io', version: '=0.14.4', defaultFeatures: true, features: [] },
    { name: 'sha2', kind: 'normal', source: 'crates_io', version: '=0.11.0', defaultFeatures: false, features: [] },
    { name: 'prost-build', kind: 'build', source: 'crates_io', version: '=0.14.4', defaultFeatures: true, features: [] },
    { name: 'protoc-bin-vendored', kind: 'build', source: 'crates_io', version: '=3.2.0', defaultFeatures: true, features: [] },
    { name: 'sha2', kind: 'build', source: 'crates_io', version: '=0.11.0', defaultFeatures: false, features: [] },
  ],
  'makosh-review-note-candidate-core': [
    { name: 'sha2', kind: 'normal', source: 'crates_io', version: '=0.11.0', defaultFeatures: false, features: [] },
  ],
};

const KNOWLEDGE_VERIFIED_NOTE_CONTRACT_CORE_THIRD_PARTY_DEPENDENCY_ALLOWLIST = {
  ...REVIEW_NOTE_CANDIDATE_CONTRACT_CORE_THIRD_PARTY_DEPENDENCY_ALLOWLIST,
  'makosh-knowledge-command-api': [
    { name: 'prost', kind: 'normal', source: 'crates_io', version: '=0.14.4', defaultFeatures: true, features: [] },
    { name: 'prost-types', kind: 'normal', source: 'crates_io', version: '=0.14.4', defaultFeatures: true, features: [] },
    { name: 'sha2', kind: 'normal', source: 'crates_io', version: '=0.11.0', defaultFeatures: false, features: [] },
    { name: 'prost-build', kind: 'build', source: 'crates_io', version: '=0.14.4', defaultFeatures: true, features: [] },
    { name: 'protoc-bin-vendored', kind: 'build', source: 'crates_io', version: '=3.2.0', defaultFeatures: true, features: [] },
    { name: 'sha2', kind: 'build', source: 'crates_io', version: '=0.11.0', defaultFeatures: false, features: [] },
  ],
  'makosh-knowledge-core': [
    { name: 'sha2', kind: 'normal', source: 'crates_io', version: '=0.11.0', defaultFeatures: false, features: [] },
  ],
};

const KNOWLEDGE_VERIFIED_NOTE_PERSISTENCE_THIRD_PARTY_DEPENDENCY_ALLOWLIST = {
  ...KNOWLEDGE_VERIFIED_NOTE_CONTRACT_CORE_THIRD_PARTY_DEPENDENCY_ALLOWLIST,
  'makosh-knowledge-persistence': [
    { name: 'sha2', kind: 'normal', source: 'crates_io', version: '=0.11.0', defaultFeatures: false, features: [] },
    { name: 'sqlx', kind: 'normal', source: 'crates_io', version: '=0.9.0', defaultFeatures: false, features: ['postgres', 'runtime-tokio', 'tls-rustls-ring'] },
  ],
};

const KNOWLEDGE_VERIFIED_NOTE_MANAGED_RUNTIME_THIRD_PARTY_DEPENDENCY_ALLOWLIST = {
  ...KNOWLEDGE_VERIFIED_NOTE_PERSISTENCE_THIRD_PARTY_DEPENDENCY_ALLOWLIST,
  'makosh-knowledge-runtime': [
    { name: 'libc', kind: 'normal', source: 'crates_io', version: '=0.2.186', defaultFeatures: true, features: [] },
    { name: 'prost', kind: 'normal', source: 'crates_io', version: '=0.14.4', defaultFeatures: true, features: [] },
    { name: 'sha2', kind: 'normal', source: 'crates_io', version: '=0.11.0', defaultFeatures: false, features: [] },
    { name: 'tokio', kind: 'normal', source: 'crates_io', version: '=1.52.4', defaultFeatures: false, features: ['rt-multi-thread', 'time'] },
    { name: 'zeroize', kind: 'normal', source: 'crates_io', version: '=1.9.0', defaultFeatures: true, features: [] },
  ],
};

const KNOWLEDGE_VERIFIED_NOTE_ASSEMBLY_THIRD_PARTY_DEPENDENCY_ALLOWLIST = {
  ...KNOWLEDGE_VERIFIED_NOTE_MANAGED_RUNTIME_THIRD_PARTY_DEPENDENCY_ALLOWLIST,
  'makosh-knowledge-assembly': [
    { name: 'prost', kind: 'normal', source: 'crates_io', version: '=0.14.4', defaultFeatures: true, features: [] },
    { name: 'serde', kind: 'normal', source: 'crates_io', version: '=1.0.228', defaultFeatures: false, features: ['derive', 'std'] },
    { name: 'serde_json', kind: 'normal', source: 'crates_io', version: '=1.0.150', defaultFeatures: true, features: [] },
  ],
};

const REVIEW_NOTE_CANDIDATE_PERSISTENCE_THIRD_PARTY_DEPENDENCY_ALLOWLIST = {
  ...KNOWLEDGE_VERIFIED_NOTE_ASSEMBLY_THIRD_PARTY_DEPENDENCY_ALLOWLIST,
  'makosh-review-note-candidate-persistence': [
    { name: 'sha2', kind: 'normal', source: 'crates_io', version: '=0.11.0', defaultFeatures: false, features: [] },
    { name: 'sqlx', kind: 'normal', source: 'crates_io', version: '=0.9.0', defaultFeatures: false, features: ['postgres', 'runtime-tokio', 'tls-rustls-ring'] },
  ],
};

const REVIEW_NOTE_CANDIDATE_MANAGED_RUNTIME_THIRD_PARTY_DEPENDENCY_ALLOWLIST = {
  ...REVIEW_NOTE_CANDIDATE_PERSISTENCE_THIRD_PARTY_DEPENDENCY_ALLOWLIST,
  'makosh-review-note-candidate-promotion-api': [
    { name: 'prost', kind: 'normal', source: 'crates_io', version: '=0.14.4', defaultFeatures: true, features: [] },
    { name: 'prost-types', kind: 'normal', source: 'crates_io', version: '=0.14.4', defaultFeatures: true, features: [] },
    { name: 'sha2', kind: 'normal', source: 'crates_io', version: '=0.11.0', defaultFeatures: false, features: [] },
    { name: 'prost-build', kind: 'build', source: 'crates_io', version: '=0.14.4', defaultFeatures: true, features: [] },
    { name: 'protoc-bin-vendored', kind: 'build', source: 'crates_io', version: '=3.2.0', defaultFeatures: true, features: [] },
    { name: 'sha2', kind: 'build', source: 'crates_io', version: '=0.11.0', defaultFeatures: false, features: [] },
  ],
  'makosh-review-note-candidate-runtime': [
    { name: 'libc', kind: 'normal', source: 'crates_io', version: '=0.2.186', defaultFeatures: true, features: [] },
    { name: 'prost', kind: 'normal', source: 'crates_io', version: '=0.14.4', defaultFeatures: true, features: [] },
    { name: 'sha2', kind: 'normal', source: 'crates_io', version: '=0.11.0', defaultFeatures: false, features: [] },
    { name: 'tokio', kind: 'normal', source: 'crates_io', version: '=1.52.4', defaultFeatures: false, features: ['rt-multi-thread', 'time'] },
    { name: 'zeroize', kind: 'normal', source: 'crates_io', version: '=1.9.0', defaultFeatures: true, features: [] },
  ],
};

const REVIEW_NOTE_CANDIDATE_ASSEMBLY_THIRD_PARTY_DEPENDENCY_ALLOWLIST = {
  ...REVIEW_NOTE_CANDIDATE_MANAGED_RUNTIME_THIRD_PARTY_DEPENDENCY_ALLOWLIST,
  'makosh-review-note-candidate-assembly': [
    { name: 'prost', kind: 'normal', source: 'crates_io', version: '=0.14.4', defaultFeatures: true, features: [] },
    { name: 'serde', kind: 'normal', source: 'crates_io', version: '=1.0.228', defaultFeatures: false, features: ['derive', 'std'] },
    { name: 'serde_json', kind: 'normal', source: 'crates_io', version: '=1.0.150', defaultFeatures: true, features: [] },
  ],
};

const REVIEWED_NOTE_CANDIDATE_PROMOTION_THIRD_PARTY_DEPENDENCY_ALLOWLIST = {
  ...REVIEW_NOTE_CANDIDATE_ASSEMBLY_THIRD_PARTY_DEPENDENCY_ALLOWLIST,
  'makosh-reviewed-note-candidate-promotion-core': [
    { name: 'sha2', kind: 'normal', source: 'crates_io', version: '=0.11.0', defaultFeatures: false, features: [] },
  ],
  'makosh-reviewed-note-candidate-promotion-persistence': [
    { name: 'sha2', kind: 'normal', source: 'crates_io', version: '=0.11.0', defaultFeatures: false, features: [] },
    { name: 'sqlx', kind: 'normal', source: 'crates_io', version: '=0.9.0', defaultFeatures: false, features: ['postgres', 'runtime-tokio', 'tls-rustls-ring'] },
  ],
  'makosh-reviewed-note-candidate-promotion-runtime': [
    { name: 'libc', kind: 'normal', source: 'crates_io', version: '=0.2.186', defaultFeatures: true, features: [] },
    { name: 'prost', kind: 'normal', source: 'crates_io', version: '=0.14.4', defaultFeatures: true, features: [] },
    { name: 'sha2', kind: 'normal', source: 'crates_io', version: '=0.11.0', defaultFeatures: false, features: [] },
    { name: 'tokio', kind: 'normal', source: 'crates_io', version: '=1.52.4', defaultFeatures: false, features: ['rt-multi-thread', 'time'] },
    { name: 'zeroize', kind: 'normal', source: 'crates_io', version: '=1.9.0', defaultFeatures: true, features: [] },
  ],
  'makosh-reviewed-note-candidate-promotion-assembly': [
    { name: 'prost', kind: 'normal', source: 'crates_io', version: '=0.14.4', defaultFeatures: true, features: [] },
    { name: 'serde', kind: 'normal', source: 'crates_io', version: '=1.0.228', defaultFeatures: false, features: ['derive', 'std'] },
    { name: 'serde_json', kind: 'normal', source: 'crates_io', version: '=1.0.150', defaultFeatures: true, features: [] },
  ],
};

const COMMUNICATION_NOTE_CANDIDATE_ASSEMBLY_THIRD_PARTY_DEPENDENCY_ALLOWLIST = {
  ...REVIEWED_NOTE_CANDIDATE_PROMOTION_THIRD_PARTY_DEPENDENCY_ALLOWLIST,
  'makosh-communication-note-candidate-runtime': [
    { name: 'libc', kind: 'normal', source: 'crates_io', version: '=0.2.186', defaultFeatures: true, features: [] },
    { name: 'prost', kind: 'normal', source: 'crates_io', version: '=0.14.4', defaultFeatures: true, features: [] },
    { name: 'sha2', kind: 'normal', source: 'crates_io', version: '=0.11.0', defaultFeatures: false, features: [] },
    { name: 'tokio', kind: 'normal', source: 'crates_io', version: '=1.52.4', defaultFeatures: false, features: ['rt-multi-thread', 'time'] },
    { name: 'zeroize', kind: 'normal', source: 'crates_io', version: '=1.9.0', defaultFeatures: true, features: [] },
  ],
  'makosh-communication-note-candidate-assembly': [
    { name: 'prost', kind: 'normal', source: 'crates_io', version: '=0.14.4', defaultFeatures: true, features: [] },
    { name: 'serde', kind: 'normal', source: 'crates_io', version: '=1.0.228', defaultFeatures: false, features: ['derive', 'std'] },
    { name: 'serde_json', kind: 'normal', source: 'crates_io', version: '=1.0.150', defaultFeatures: true, features: [] },
  ],
};

const ATTACHMENT_TEXT_EXTRACTION_CONTRACT_CORE_THIRD_PARTY_DEPENDENCY_ALLOWLIST = {
  ...COMMUNICATION_NOTE_CANDIDATE_ASSEMBLY_THIRD_PARTY_DEPENDENCY_ALLOWLIST,
  'makosh-attachment-text-extraction-api': [
    { name: 'prost', kind: 'normal', source: 'crates_io', version: '=0.14.4', defaultFeatures: true, features: [] },
    { name: 'prost-build', kind: 'build', source: 'crates_io', version: '=0.14.4', defaultFeatures: true, features: [] },
    { name: 'protoc-bin-vendored', kind: 'build', source: 'crates_io', version: '=3.2.0', defaultFeatures: true, features: [] },
    { name: 'sha2', kind: 'build', source: 'crates_io', version: '=0.11.0', defaultFeatures: false, features: [] },
  ],
  'makosh-attachment-text-extraction-ingress': [
    { name: 'prost', kind: 'normal', source: 'crates_io', version: '=0.14.4', defaultFeatures: true, features: [] },
    { name: 'prost-types', kind: 'normal', source: 'crates_io', version: '=0.14.4', defaultFeatures: true, features: [] },
    { name: 'sha2', kind: 'normal', source: 'crates_io', version: '=0.11.0', defaultFeatures: false, features: [] },
    { name: 'prost-build', kind: 'build', source: 'crates_io', version: '=0.14.4', defaultFeatures: true, features: [] },
    { name: 'protoc-bin-vendored', kind: 'build', source: 'crates_io', version: '=3.2.0', defaultFeatures: true, features: [] },
    { name: 'sha2', kind: 'build', source: 'crates_io', version: '=0.11.0', defaultFeatures: false, features: [] },
  ],
  'makosh-attachment-text-extraction-core': [
    { name: 'sha2', kind: 'normal', source: 'crates_io', version: '=0.11.0', defaultFeatures: false, features: [] },
  ],
};

const ATTACHMENT_TEXT_EXTRACTION_PARSER_ADAPTERS_THIRD_PARTY_DEPENDENCY_ALLOWLIST = {
  ...ATTACHMENT_TEXT_EXTRACTION_CONTRACT_CORE_THIRD_PARTY_DEPENDENCY_ALLOWLIST,
  'makosh-attachment-text-extraction-parser-contract': [],
  'makosh-attachment-text-extraction-plain': [],
  'makosh-attachment-text-extraction-pdf': [
    { name: 'pdf-text-extract', kind: 'normal', source: 'crates_io', version: '=0.2.0', defaultFeatures: false, features: [] },
  ],
  'makosh-attachment-text-extraction-docx': [
    { name: 'quick-xml', kind: 'normal', source: 'crates_io', version: '=0.41.0', defaultFeatures: false, features: [] },
    { name: 'zip', kind: 'normal', source: 'crates_io', version: '=6.0.0', defaultFeatures: false, features: ['deflate-flate2-zlib-rs'] },
  ],
  'makosh-attachment-text-extraction-ocr': [
    { name: 'sha2', kind: 'normal', source: 'crates_io', version: '=0.11.0', defaultFeatures: false, features: [] },
  ],
};

const ATTACHMENT_TEXT_EXTRACTION_PERSISTENCE_THIRD_PARTY_DEPENDENCY_ALLOWLIST = {
  ...ATTACHMENT_TEXT_EXTRACTION_PARSER_ADAPTERS_THIRD_PARTY_DEPENDENCY_ALLOWLIST,
  'makosh-attachment-text-extraction-persistence': [
    { name: 'sha2', kind: 'normal', source: 'crates_io', version: '=0.11.0', defaultFeatures: false, features: [] },
    { name: 'sqlx', kind: 'normal', source: 'crates_io', version: '=0.9.0', defaultFeatures: false, features: ['postgres', 'runtime-tokio', 'tls-rustls-ring'] },
  ],
};

const ATTACHMENT_TEXT_EXTRACTION_RUNTIME_ASSEMBLY_THIRD_PARTY_DEPENDENCY_ALLOWLIST = {
  ...ATTACHMENT_TEXT_EXTRACTION_PERSISTENCE_THIRD_PARTY_DEPENDENCY_ALLOWLIST,
  'makosh-attachment-text-extraction-runtime': [
    { name: 'libc', kind: 'normal', source: 'crates_io', version: '=0.2.186', defaultFeatures: true, features: [] },
    { name: 'prost', kind: 'normal', source: 'crates_io', version: '=0.14.4', defaultFeatures: true, features: [] },
    { name: 'sha2', kind: 'normal', source: 'crates_io', version: '=0.11.0', defaultFeatures: false, features: [] },
    { name: 'tokio', kind: 'normal', source: 'crates_io', version: '=1.52.4', defaultFeatures: false, features: ['rt-multi-thread', 'time'] },
    { name: 'zeroize', kind: 'normal', source: 'crates_io', version: '=1.9.0', defaultFeatures: true, features: [] },
  ],
  'makosh-attachment-text-extraction-assembly': [
    { name: 'prost', kind: 'normal', source: 'crates_io', version: '=0.14.4', defaultFeatures: true, features: [] },
    { name: 'serde', kind: 'normal', source: 'crates_io', version: '=1.0.228', defaultFeatures: false, features: ['derive', 'std'] },
    { name: 'serde_json', kind: 'normal', source: 'crates_io', version: '=1.0.150', defaultFeatures: true, features: [] },
  ],
};

const ATTACHMENT_PREVIEW_FOUNDATION_THIRD_PARTY_DEPENDENCY_ALLOWLIST = {
  ...ATTACHMENT_TEXT_EXTRACTION_RUNTIME_ASSEMBLY_THIRD_PARTY_DEPENDENCY_ALLOWLIST,
  'makosh-attachment-preview-api': [
    { name: 'prost', kind: 'normal', source: 'crates_io', version: '=0.14.4', defaultFeatures: true, features: [] },
    { name: 'prost-build', kind: 'build', source: 'crates_io', version: '=0.14.4', defaultFeatures: true, features: [] },
    { name: 'protoc-bin-vendored', kind: 'build', source: 'crates_io', version: '=3.2.0', defaultFeatures: true, features: [] },
    { name: 'sha2', kind: 'build', source: 'crates_io', version: '=0.11.0', defaultFeatures: false, features: [] },
  ],
  'makosh-attachment-preview-ingress': [
    { name: 'prost', kind: 'normal', source: 'crates_io', version: '=0.14.4', defaultFeatures: true, features: [] },
    { name: 'prost-types', kind: 'normal', source: 'crates_io', version: '=0.14.4', defaultFeatures: true, features: [] },
    { name: 'sha2', kind: 'normal', source: 'crates_io', version: '=0.11.0', defaultFeatures: false, features: [] },
    { name: 'prost-build', kind: 'build', source: 'crates_io', version: '=0.14.4', defaultFeatures: true, features: [] },
    { name: 'protoc-bin-vendored', kind: 'build', source: 'crates_io', version: '=3.2.0', defaultFeatures: true, features: [] },
    { name: 'sha2', kind: 'build', source: 'crates_io', version: '=0.11.0', defaultFeatures: false, features: [] },
  ],
  'makosh-attachment-preview-core': [],
  'makosh-attachment-preview-renderer-contract': [],
};

const ATTACHMENT_PREVIEW_SAFE_ADAPTERS_THIRD_PARTY_DEPENDENCY_ALLOWLIST = {
  ...ATTACHMENT_PREVIEW_FOUNDATION_THIRD_PARTY_DEPENDENCY_ALLOWLIST,
  'makosh-attachment-preview-text': [],
  'makosh-attachment-preview-image': [
    { name: 'image', kind: 'normal', source: 'crates_io', version: '=0.25.9', defaultFeatures: false, features: ['gif', 'jpeg', 'png', 'webp'] },
  ],
  'makosh-attachment-preview-media': [],
};

const ATTACHMENT_PREVIEW_PDF_ADAPTER_THIRD_PARTY_DEPENDENCY_ALLOWLIST = {
  ...ATTACHMENT_PREVIEW_SAFE_ADAPTERS_THIRD_PARTY_DEPENDENCY_ALLOWLIST,
  'makosh-attachment-preview-pdf': [
    { name: 'image', kind: 'normal', source: 'crates_io', version: '=0.25.9', defaultFeatures: false, features: ['png'] },
    { name: 'hayro', kind: 'normal', source: 'crates_io', version: '=0.7.1', defaultFeatures: true, features: [] },
  ],
};

const ATTACHMENT_PREVIEW_DOCX_ADAPTER_THIRD_PARTY_DEPENDENCY_ALLOWLIST = {
  ...ATTACHMENT_PREVIEW_PDF_ADAPTER_THIRD_PARTY_DEPENDENCY_ALLOWLIST,
  'makosh-attachment-preview-docx': [
    { name: 'image', kind: 'normal', source: 'crates_io', version: '=0.25.9', defaultFeatures: false, features: ['png'] },
    { name: 'quick-xml', kind: 'normal', source: 'crates_io', version: '=0.41.0', defaultFeatures: false, features: [] },
    { name: 'swash', kind: 'normal', source: 'crates_io', version: '=0.2.10', defaultFeatures: false, features: ['render', 'std'] },
    { name: 'zip', kind: 'normal', source: 'crates_io', version: '=6.0.0', defaultFeatures: false, features: ['deflate-flate2-zlib-rs'] },
  ],
};

const ATTACHMENT_PREVIEW_PERSISTENCE_THIRD_PARTY_DEPENDENCY_ALLOWLIST = {
  ...ATTACHMENT_PREVIEW_DOCX_ADAPTER_THIRD_PARTY_DEPENDENCY_ALLOWLIST,
  'makosh-attachment-preview-persistence': [
    { name: 'sha2', kind: 'normal', source: 'crates_io', version: '=0.11.0', defaultFeatures: false, features: [] },
    { name: 'sqlx', kind: 'normal', source: 'crates_io', version: '=0.9.0', defaultFeatures: false, features: ['postgres', 'runtime-tokio', 'tls-rustls-ring'] },
  ],
};

const ATTACHMENT_PREVIEW_RUNTIME_THIRD_PARTY_DEPENDENCY_ALLOWLIST = {
  ...ATTACHMENT_PREVIEW_PERSISTENCE_THIRD_PARTY_DEPENDENCY_ALLOWLIST,
  'makosh-attachment-preview-runtime': [
    { name: 'getrandom', kind: 'normal', source: 'crates_io', version: '=0.4.3', defaultFeatures: false, features: [] },
    { name: 'libc', kind: 'normal', source: 'crates_io', version: '=0.2.186', defaultFeatures: true, features: [] },
    { name: 'prost', kind: 'normal', source: 'crates_io', version: '=0.14.4', defaultFeatures: true, features: [] },
    { name: 'sha2', kind: 'normal', source: 'crates_io', version: '=0.11.0', defaultFeatures: false, features: [] },
    { name: 'tokio', kind: 'normal', source: 'crates_io', version: '=1.52.4', defaultFeatures: false, features: ['rt-multi-thread', 'time'] },
    { name: 'zeroize', kind: 'normal', source: 'crates_io', version: '=1.9.0', defaultFeatures: true, features: [] },
  ],
};

const ATTACHMENT_PREVIEW_ASSEMBLY_THIRD_PARTY_DEPENDENCY_ALLOWLIST = {
  ...ATTACHMENT_PREVIEW_RUNTIME_THIRD_PARTY_DEPENDENCY_ALLOWLIST,
  'makosh-attachment-preview-assembly': [
    { name: 'prost', kind: 'normal', source: 'crates_io', version: '=0.14.4', defaultFeatures: true, features: [] },
    { name: 'serde', kind: 'normal', source: 'crates_io', version: '=1.0.228', defaultFeatures: false, features: ['derive', 'std'] },
    { name: 'serde_json', kind: 'normal', source: 'crates_io', version: '=1.0.150', defaultFeatures: true, features: [] },
  ],
};

const ATTACHMENT_PREVIEW_RETAINED_EVIDENCE_REPLAY_THIRD_PARTY_DEPENDENCY_ALLOWLIST = {
  ...ATTACHMENT_PREVIEW_ASSEMBLY_THIRD_PARTY_DEPENDENCY_ALLOWLIST,
  'makosh-retained-evidence-replay-protocol': [
    { name: 'prost', kind: 'normal', source: 'crates_io', version: '=0.14.4', defaultFeatures: true, features: [] },
    { name: 'prost-build', kind: 'build', source: 'crates_io', version: '=0.14.4', defaultFeatures: true, features: [] },
    { name: 'protoc-bin-vendored', kind: 'build', source: 'crates_io', version: '=3.2.0', defaultFeatures: true, features: [] },
    { name: 'sha2', kind: 'build', source: 'crates_io', version: '=0.11.0', defaultFeatures: false, features: [] },
  ],
  'makosh-attachment-preview-evidence-replay-api': [
    { name: 'prost', kind: 'normal', source: 'crates_io', version: '=0.14.4', defaultFeatures: true, features: [] },
    { name: 'prost-build', kind: 'build', source: 'crates_io', version: '=0.14.4', defaultFeatures: true, features: [] },
    { name: 'protoc-bin-vendored', kind: 'build', source: 'crates_io', version: '=3.2.0', defaultFeatures: true, features: [] },
    { name: 'sha2', kind: 'build', source: 'crates_io', version: '=0.11.0', defaultFeatures: false, features: [] },
  ],
  'makosh-attachment-preview-evidence-replay-core': [],
  'makosh-attachment-preview-evidence-replay-persistence': [
    { name: 'prost', kind: 'normal', source: 'crates_io', version: '=0.14.4', defaultFeatures: true, features: [] },
    { name: 'sha2', kind: 'normal', source: 'crates_io', version: '=0.11.0', defaultFeatures: false, features: [] },
    { name: 'sqlx', kind: 'normal', source: 'crates_io', version: '=0.9.0', defaultFeatures: false, features: ['postgres', 'runtime-tokio', 'tls-rustls-ring'] },
  ],
  'makosh-attachment-preview-evidence-replay-runtime': [
    { name: 'libc', kind: 'normal', source: 'crates_io', version: '=0.2.186', defaultFeatures: true, features: [] },
    { name: 'prost', kind: 'normal', source: 'crates_io', version: '=0.14.4', defaultFeatures: true, features: [] },
    { name: 'sha2', kind: 'normal', source: 'crates_io', version: '=0.11.0', defaultFeatures: false, features: [] },
    { name: 'tokio', kind: 'normal', source: 'crates_io', version: '=1.52.4', defaultFeatures: false, features: ['rt-multi-thread', 'time'] },
    { name: 'zeroize', kind: 'normal', source: 'crates_io', version: '=1.9.0', defaultFeatures: true, features: [] },
  ],
  'makosh-attachment-preview-evidence-replay-assembly': [
    { name: 'prost', kind: 'normal', source: 'crates_io', version: '=0.14.4', defaultFeatures: true, features: [] },
    { name: 'serde', kind: 'normal', source: 'crates_io', version: '=1.0.228', defaultFeatures: false, features: ['derive', 'std'] },
    { name: 'serde_json', kind: 'normal', source: 'crates_io', version: '=1.0.150', defaultFeatures: true, features: [] },
  ],
  'makosh-communications-retained-evidence-replay-persistence': [
    { name: 'prost', kind: 'normal', source: 'crates_io', version: '=0.14.4', defaultFeatures: true, features: [] },
    { name: 'prost', kind: 'dev', source: 'crates_io', version: '=0.14.4', defaultFeatures: true, features: [] },
    { name: 'prost-types', kind: 'dev', source: 'crates_io', version: '=0.14.4', defaultFeatures: true, features: [] },
    { name: 'sha2', kind: 'normal', source: 'crates_io', version: '=0.11.0', defaultFeatures: false, features: [] },
    { name: 'sqlx', kind: 'normal', source: 'crates_io', version: '=0.9.0', defaultFeatures: false, features: ['postgres', 'runtime-tokio', 'tls-rustls-ring'] },
  ],
  'makosh-mail-retained-evidence-replay-persistence': [
    { name: 'prost', kind: 'normal', source: 'crates_io', version: '=0.14.4', defaultFeatures: true, features: [] },
    { name: 'prost', kind: 'dev', source: 'crates_io', version: '=0.14.4', defaultFeatures: true, features: [] },
    { name: 'prost-types', kind: 'dev', source: 'crates_io', version: '=0.14.4', defaultFeatures: true, features: [] },
    { name: 'sha2', kind: 'normal', source: 'crates_io', version: '=0.11.0', defaultFeatures: false, features: [] },
    { name: 'sqlx', kind: 'normal', source: 'crates_io', version: '=0.9.0', defaultFeatures: false, features: ['postgres', 'runtime-tokio', 'tls-rustls-ring'] },
  ],
  'makosh-communications-retained-evidence-replay-contract': [
    { name: 'prost', kind: 'normal', source: 'crates_io', version: '=0.14.4', defaultFeatures: true, features: [] },
    { name: 'prost-types', kind: 'normal', source: 'crates_io', version: '=0.14.4', defaultFeatures: true, features: [] },
    { name: 'prost-build', kind: 'build', source: 'crates_io', version: '=0.14.4', defaultFeatures: true, features: [] },
    { name: 'protoc-bin-vendored', kind: 'build', source: 'crates_io', version: '=3.2.0', defaultFeatures: true, features: [] },
    { name: 'sha2', kind: 'normal', source: 'crates_io', version: '=0.11.0', defaultFeatures: false, features: [] },
    { name: 'sha2', kind: 'build', source: 'crates_io', version: '=0.11.0', defaultFeatures: false, features: [] },
  ],
  'makosh-mail-retained-evidence-replay-contract': [
    { name: 'prost', kind: 'normal', source: 'crates_io', version: '=0.14.4', defaultFeatures: true, features: [] },
    { name: 'prost-types', kind: 'normal', source: 'crates_io', version: '=0.14.4', defaultFeatures: true, features: [] },
    { name: 'prost-build', kind: 'build', source: 'crates_io', version: '=0.14.4', defaultFeatures: true, features: [] },
    { name: 'protoc-bin-vendored', kind: 'build', source: 'crates_io', version: '=3.2.0', defaultFeatures: true, features: [] },
    { name: 'sha2', kind: 'normal', source: 'crates_io', version: '=0.11.0', defaultFeatures: false, features: [] },
    { name: 'sha2', kind: 'build', source: 'crates_io', version: '=0.11.0', defaultFeatures: false, features: [] },
  ],
};

const ATTACHMENT_TRANSLATION_CONTRACTS_THIRD_PARTY_DEPENDENCY_ALLOWLIST = {
  ...ATTACHMENT_PREVIEW_RETAINED_EVIDENCE_REPLAY_THIRD_PARTY_DEPENDENCY_ALLOWLIST,
  'makosh-attachment-translation-api': [
    { name: 'prost', kind: 'normal', source: 'crates_io', version: '=0.14.4', defaultFeatures: true, features: [] },
    { name: 'prost-build', kind: 'build', source: 'crates_io', version: '=0.14.4', defaultFeatures: true, features: [] },
    { name: 'protoc-bin-vendored', kind: 'build', source: 'crates_io', version: '=3.2.0', defaultFeatures: true, features: [] },
    { name: 'sha2', kind: 'build', source: 'crates_io', version: '=0.11.0', defaultFeatures: false, features: [] },
  ],
  'makosh-attachment-translation-ingress': [
    { name: 'prost', kind: 'normal', source: 'crates_io', version: '=0.14.4', defaultFeatures: true, features: [] },
    { name: 'prost-build', kind: 'build', source: 'crates_io', version: '=0.14.4', defaultFeatures: true, features: [] },
    { name: 'prost-types', kind: 'normal', source: 'crates_io', version: '=0.14.4', defaultFeatures: true, features: [] },
    { name: 'protoc-bin-vendored', kind: 'build', source: 'crates_io', version: '=3.2.0', defaultFeatures: true, features: [] },
    { name: 'sha2', kind: 'normal', source: 'crates_io', version: '=0.11.0', defaultFeatures: false, features: [] },
    { name: 'sha2', kind: 'build', source: 'crates_io', version: '=0.11.0', defaultFeatures: false, features: [] },
  ],
  'makosh-attachment-translation-core': [],
};

const ATTACHMENT_TRANSLATION_PERSISTENCE_THIRD_PARTY_DEPENDENCY_ALLOWLIST = {
  ...ATTACHMENT_TRANSLATION_CONTRACTS_THIRD_PARTY_DEPENDENCY_ALLOWLIST,
  'makosh-attachment-translation-persistence': [
    { name: 'sha2', kind: 'normal', source: 'crates_io', version: '=0.11.0', defaultFeatures: false, features: [] },
    { name: 'sqlx', kind: 'normal', source: 'crates_io', version: '=0.9.0', defaultFeatures: false, features: ['postgres', 'runtime-tokio', 'tls-rustls-ring'] },
  ],
};

const ATTACHMENT_TRANSLATION_RUNTIME_ASSEMBLY_THIRD_PARTY_DEPENDENCY_ALLOWLIST = {
  ...ATTACHMENT_TRANSLATION_PERSISTENCE_THIRD_PARTY_DEPENDENCY_ALLOWLIST,
  'makosh-attachment-translation-runtime': [
    { name: 'getrandom', kind: 'normal', source: 'crates_io', version: '=0.4.3', defaultFeatures: true, features: [] },
    { name: 'libc', kind: 'normal', source: 'crates_io', version: '=0.2.186', defaultFeatures: true, features: [] },
    { name: 'prost', kind: 'normal', source: 'crates_io', version: '=0.14.4', defaultFeatures: true, features: [] },
    { name: 'sha2', kind: 'normal', source: 'crates_io', version: '=0.11.0', defaultFeatures: false, features: [] },
    { name: 'tokio', kind: 'normal', source: 'crates_io', version: '=1.52.4', defaultFeatures: false, features: ['rt-multi-thread', 'time'] },
    { name: 'zeroize', kind: 'normal', source: 'crates_io', version: '=1.9.0', defaultFeatures: true, features: [] },
  ],
  'makosh-attachment-translation-assembly': [
    { name: 'prost', kind: 'normal', source: 'crates_io', version: '=0.14.4', defaultFeatures: true, features: [] },
    { name: 'serde', kind: 'normal', source: 'crates_io', version: '=1.0.228', defaultFeatures: false, features: ['derive', 'std'] },
    { name: 'serde_json', kind: 'normal', source: 'crates_io', version: '=1.0.150', defaultFeatures: true, features: [] },
  ],
};

const CONTACTS_MAIL_IDENTITY_COMMAND_CONTRACT_CORE_THIRD_PARTY_DEPENDENCY_ALLOWLIST = {
  ...ATTACHMENT_TRANSLATION_RUNTIME_ASSEMBLY_THIRD_PARTY_DEPENDENCY_ALLOWLIST,
  'makosh-contacts-command-api': [
    { name: 'prost', kind: 'normal', source: 'crates_io', version: '=0.14.4', defaultFeatures: true, features: [] },
    { name: 'prost-types', kind: 'normal', source: 'crates_io', version: '=0.14.4', defaultFeatures: true, features: [] },
    { name: 'sha2', kind: 'normal', source: 'crates_io', version: '=0.11.0', defaultFeatures: false, features: [] },
    { name: 'prost-build', kind: 'build', source: 'crates_io', version: '=0.14.4', defaultFeatures: true, features: [] },
    { name: 'protoc-bin-vendored', kind: 'build', source: 'crates_io', version: '=3.2.0', defaultFeatures: true, features: [] },
    { name: 'sha2', kind: 'build', source: 'crates_io', version: '=0.11.0', defaultFeatures: false, features: [] },
  ],
  'makosh-contacts-core': [
    { name: 'sha2', kind: 'normal', source: 'crates_io', version: '=0.11.0', defaultFeatures: false, features: [] },
  ],
};

const CONTACTS_MAIL_IDENTITY_COMMAND_PERSISTENCE_THIRD_PARTY_DEPENDENCY_ALLOWLIST = {
  ...CONTACTS_MAIL_IDENTITY_COMMAND_CONTRACT_CORE_THIRD_PARTY_DEPENDENCY_ALLOWLIST,
  'makosh-contacts-persistence': [
    { name: 'sha2', kind: 'normal', source: 'crates_io', version: '=0.11.0', defaultFeatures: false, features: [] },
    { name: 'sqlx', kind: 'normal', source: 'crates_io', version: '=0.9.0', defaultFeatures: false, features: ['postgres', 'runtime-tokio', 'tls-rustls-ring'] },
  ],
};

const CONTACTS_MAIL_IDENTITY_COMMAND_RUNTIME_ASSEMBLY_THIRD_PARTY_DEPENDENCY_ALLOWLIST = {
  ...CONTACTS_MAIL_IDENTITY_COMMAND_PERSISTENCE_THIRD_PARTY_DEPENDENCY_ALLOWLIST,
  'makosh-contacts-runtime': [
    { name: 'libc', kind: 'normal', source: 'crates_io', version: '=0.2.186', defaultFeatures: true, features: [] },
    { name: 'prost', kind: 'normal', source: 'crates_io', version: '=0.14.4', defaultFeatures: true, features: [] },
    { name: 'prost-types', kind: 'normal', source: 'crates_io', version: '=0.14.4', defaultFeatures: true, features: [] },
    { name: 'sha2', kind: 'normal', source: 'crates_io', version: '=0.11.0', defaultFeatures: false, features: [] },
    { name: 'tokio', kind: 'normal', source: 'crates_io', version: '=1.52.4', defaultFeatures: false, features: ['rt-multi-thread', 'time'] },
    { name: 'zeroize', kind: 'normal', source: 'crates_io', version: '=1.9.0', defaultFeatures: true, features: [] },
  ],
  'makosh-contacts-assembly': [
    { name: 'prost', kind: 'normal', source: 'crates_io', version: '=0.14.4', defaultFeatures: true, features: [] },
    { name: 'serde', kind: 'normal', source: 'crates_io', version: '=1.0.228', defaultFeatures: false, features: ['derive', 'std'] },
    { name: 'serde_json', kind: 'normal', source: 'crates_io', version: '=1.0.150', defaultFeatures: true, features: [] },
  ],
};

const MAIL_CONTACTS_SYNC_CONTRACT_CORE_THIRD_PARTY_DEPENDENCY_ALLOWLIST = {
  ...CONTACTS_MAIL_IDENTITY_COMMAND_RUNTIME_ASSEMBLY_THIRD_PARTY_DEPENDENCY_ALLOWLIST,
  'makosh-mail-address-book-contract': [
    { name: 'prost', kind: 'normal', source: 'crates_io', version: '=0.14.4', defaultFeatures: true, features: [] },
    { name: 'prost-types', kind: 'normal', source: 'crates_io', version: '=0.14.4', defaultFeatures: true, features: [] },
    { name: 'sha2', kind: 'normal', source: 'crates_io', version: '=0.11.0', defaultFeatures: false, features: [] },
    { name: 'prost-build', kind: 'build', source: 'crates_io', version: '=0.14.4', defaultFeatures: true, features: [] },
    { name: 'protoc-bin-vendored', kind: 'build', source: 'crates_io', version: '=3.2.0', defaultFeatures: true, features: [] },
    { name: 'sha2', kind: 'build', source: 'crates_io', version: '=0.11.0', defaultFeatures: false, features: [] },
  ],
  'makosh-mail-contacts-sync-api': [
    { name: 'prost', kind: 'normal', source: 'crates_io', version: '=0.14.4', defaultFeatures: true, features: [] },
    { name: 'prost-build', kind: 'build', source: 'crates_io', version: '=0.14.4', defaultFeatures: true, features: [] },
    { name: 'protoc-bin-vendored', kind: 'build', source: 'crates_io', version: '=3.2.0', defaultFeatures: true, features: [] },
    { name: 'sha2', kind: 'build', source: 'crates_io', version: '=0.11.0', defaultFeatures: false, features: [] },
  ],
  'makosh-mail-contacts-sync-core': [],
};

const MAIL_CONTACTS_SYNC_PERSISTENCE_THIRD_PARTY_DEPENDENCY_ALLOWLIST = {
  ...MAIL_CONTACTS_SYNC_CONTRACT_CORE_THIRD_PARTY_DEPENDENCY_ALLOWLIST,
  'makosh-mail-contacts-sync-persistence': [
    { name: 'sha2', kind: 'normal', source: 'crates_io', version: '=0.11.0', defaultFeatures: false, features: [] },
    { name: 'sqlx', kind: 'normal', source: 'crates_io', version: '=0.9.0', defaultFeatures: false, features: ['postgres', 'runtime-tokio', 'tls-rustls-ring'] },
  ],
};

const MAIL_CONTACTS_SYNC_RUNTIME_ADMISSION_THIRD_PARTY_DEPENDENCY_ALLOWLIST = {
  ...MAIL_CONTACTS_SYNC_PERSISTENCE_THIRD_PARTY_DEPENDENCY_ALLOWLIST,
  'makosh-contacts-mail-sync-source-api': [
    { name: 'prost', kind: 'normal', source: 'crates_io', version: '=0.14.4', defaultFeatures: true, features: [] },
    { name: 'prost-types', kind: 'normal', source: 'crates_io', version: '=0.14.4', defaultFeatures: true, features: [] },
    { name: 'sha2', kind: 'normal', source: 'crates_io', version: '=0.11.0', defaultFeatures: false, features: [] },
    { name: 'prost-build', kind: 'build', source: 'crates_io', version: '=0.14.4', defaultFeatures: true, features: [] },
    { name: 'protoc-bin-vendored', kind: 'build', source: 'crates_io', version: '=3.2.0', defaultFeatures: true, features: [] },
    { name: 'sha2', kind: 'build', source: 'crates_io', version: '=0.11.0', defaultFeatures: false, features: [] },
  ],
  'makosh-mail-contacts-sync-runtime': [
    { name: 'libc', kind: 'normal', source: 'crates_io', version: '=0.2.186', defaultFeatures: true, features: [] },
    { name: 'prost', kind: 'normal', source: 'crates_io', version: '=0.14.4', defaultFeatures: true, features: [] },
    { name: 'prost-types', kind: 'normal', source: 'crates_io', version: '=0.14.4', defaultFeatures: true, features: [] },
    { name: 'sha2', kind: 'normal', source: 'crates_io', version: '=0.11.0', defaultFeatures: false, features: [] },
    { name: 'tokio', kind: 'normal', source: 'crates_io', version: '=1.52.4', defaultFeatures: false, features: ['rt-multi-thread', 'time'] },
    { name: 'zeroize', kind: 'normal', source: 'crates_io', version: '=1.9.0', defaultFeatures: true, features: [] },
  ],
};

const MAIL_ADDRESS_BOOK_PROVIDER_ADAPTERS_THIRD_PARTY_DEPENDENCY_ALLOWLIST = {
  ...MAIL_CONTACTS_SYNC_RUNTIME_ADMISSION_THIRD_PARTY_DEPENDENCY_ALLOWLIST,
  'makosh-mail-google-people': [
    { name: 'async-native-tls', kind: 'normal', source: 'crates_io', version: '=0.6.0', defaultFeatures: true, features: [] },
    { name: 'async-std', kind: 'normal', source: 'crates_io', version: '=1.13.2', defaultFeatures: true, features: [] },
    { name: 'futures-util', kind: 'normal', source: 'crates_io', version: '=0.3.32', defaultFeatures: true, features: [] },
    { name: 'serde', kind: 'normal', source: 'crates_io', version: '=1.0.228', defaultFeatures: true, features: ['derive'] },
    { name: 'serde_json', kind: 'normal', source: 'crates_io', version: '=1.0.150', defaultFeatures: true, features: [] },
  ],
  'makosh-mail-carddav': [
    { name: 'async-native-tls', kind: 'normal', source: 'crates_io', version: '=0.6.0', defaultFeatures: true, features: [] },
    { name: 'async-std', kind: 'normal', source: 'crates_io', version: '=1.13.2', defaultFeatures: true, features: [] },
    { name: 'base64', kind: 'normal', source: 'crates_io', version: '=0.22.1', defaultFeatures: true, features: [] },
    { name: 'futures-util', kind: 'normal', source: 'crates_io', version: '=0.3.32', defaultFeatures: true, features: [] },
    { name: 'quick-xml', kind: 'normal', source: 'crates_io', version: '=0.41.0', defaultFeatures: false, features: [] },
  ],
};

const MAIL_ADDRESS_BOOK_PERSISTENCE_AUTHORITY_THIRD_PARTY_DEPENDENCY_ALLOWLIST = {
  ...MAIL_ADDRESS_BOOK_PROVIDER_ADAPTERS_THIRD_PARTY_DEPENDENCY_ALLOWLIST,
  'makosh-mail-address-book-persistence': [
    { name: 'prost', kind: 'normal', source: 'crates_io', version: '=0.14.4', defaultFeatures: true, features: [] },
    { name: 'sha2', kind: 'normal', source: 'crates_io', version: '=0.11.0', defaultFeatures: false, features: [] },
    { name: 'sqlx', kind: 'normal', source: 'crates_io', version: '=0.9.0', defaultFeatures: false, features: ['postgres', 'runtime-tokio', 'tls-rustls-ring'] },
  ],
};

const MAIL_ADDRESS_BOOK_RUNTIME_EXECUTION_THIRD_PARTY_DEPENDENCY_ALLOWLIST =
  MAIL_ADDRESS_BOOK_PERSISTENCE_AUTHORITY_THIRD_PARTY_DEPENDENCY_ALLOWLIST;

const MAIL_CONTACTS_SYNC_RELEASE_ASSEMBLY_THIRD_PARTY_DEPENDENCY_ALLOWLIST = {
  ...MAIL_ADDRESS_BOOK_RUNTIME_EXECUTION_THIRD_PARTY_DEPENDENCY_ALLOWLIST,
  'makosh-mail-contacts-sync-assembly': [
    { name: 'prost', kind: 'normal', source: 'crates_io', version: '=0.14.4', defaultFeatures: true, features: [] },
    { name: 'serde', kind: 'normal', source: 'crates_io', version: '=1.0.228', defaultFeatures: false, features: ['derive', 'std'] },
    { name: 'serde_json', kind: 'normal', source: 'crates_io', version: '=1.0.150', defaultFeatures: true, features: [] },
  ],
  'makosh-speech-to-text-api': [
    { name: 'prost', kind: 'normal', source: 'crates_io', version: '=0.14.4', defaultFeatures: true, features: [] },
    { name: 'sha2', kind: 'normal', source: 'crates_io', version: '=0.11.0', defaultFeatures: false, features: [] },
    { name: 'prost-build', kind: 'build', source: 'crates_io', version: '=0.14.4', defaultFeatures: true, features: [] },
    { name: 'protoc-bin-vendored', kind: 'build', source: 'crates_io', version: '=3.2.0', defaultFeatures: true, features: [] },
    { name: 'sha2', kind: 'build', source: 'crates_io', version: '=0.11.0', defaultFeatures: false, features: [] },
  ],
  'makosh-speech-to-text-core': [],
  'makosh-speech-to-text-persistence': [
    { name: 'sha2', kind: 'normal', source: 'crates_io', version: '=0.11.0', defaultFeatures: false, features: [] },
    { name: 'sqlx', kind: 'normal', source: 'crates_io', version: '=0.9.0', defaultFeatures: false, features: ['postgres', 'runtime-tokio', 'tls-rustls-ring'] },
  ],
};

const DESKTOP_CALL_RECORDING_CONTRACT_CORE_THIRD_PARTY_DEPENDENCY_ALLOWLIST = {
  ...MAIL_CONTACTS_SYNC_RELEASE_ASSEMBLY_THIRD_PARTY_DEPENDENCY_ALLOWLIST,
  'makosh-desktop-call-recording-api': [
    { name: 'prost', kind: 'normal', source: 'crates_io', version: '=0.14.4', defaultFeatures: true, features: [] },
    { name: 'sha2', kind: 'normal', source: 'crates_io', version: '=0.11.0', defaultFeatures: false, features: [] },
    { name: 'prost-build', kind: 'build', source: 'crates_io', version: '=0.14.4', defaultFeatures: true, features: [] },
    { name: 'protoc-bin-vendored', kind: 'build', source: 'crates_io', version: '=3.2.0', defaultFeatures: true, features: [] },
    { name: 'sha2', kind: 'build', source: 'crates_io', version: '=0.11.0', defaultFeatures: false, features: [] },
  ],
  'makosh-desktop-call-recording-core': [
    { name: 'sha2', kind: 'normal', source: 'crates_io', version: '=0.11.0', defaultFeatures: false, features: [] },
  ],
  'makosh-call-transcription-ingress': [
    { name: 'prost', kind: 'normal', source: 'crates_io', version: '=0.14.4', defaultFeatures: true, features: [] },
    { name: 'prost-types', kind: 'normal', source: 'crates_io', version: '=0.14.4', defaultFeatures: true, features: [] },
    { name: 'sha2', kind: 'normal', source: 'crates_io', version: '=0.11.0', defaultFeatures: false, features: [] },
    { name: 'prost-build', kind: 'build', source: 'crates_io', version: '=0.14.4', defaultFeatures: true, features: [] },
    { name: 'protoc-bin-vendored', kind: 'build', source: 'crates_io', version: '=3.2.0', defaultFeatures: true, features: [] },
    { name: 'sha2', kind: 'build', source: 'crates_io', version: '=0.11.0', defaultFeatures: false, features: [] },
  ],
};

const DESKTOP_CALL_RECORDING_PERSISTENCE_THIRD_PARTY_DEPENDENCY_ALLOWLIST = {
  ...DESKTOP_CALL_RECORDING_CONTRACT_CORE_THIRD_PARTY_DEPENDENCY_ALLOWLIST,
  'makosh-desktop-call-recording-persistence': [
    { name: 'sha2', kind: 'normal', source: 'crates_io', version: '=0.11.0', defaultFeatures: false, features: [] },
    { name: 'sqlx', kind: 'normal', source: 'crates_io', version: '=0.9.0', defaultFeatures: false, features: ['postgres', 'runtime-tokio', 'tls-rustls-ring'] },
  ],
};

const DESKTOP_CALL_RECORDING_RUNTIME_THIRD_PARTY_DEPENDENCY_ALLOWLIST = {
  ...DESKTOP_CALL_RECORDING_PERSISTENCE_THIRD_PARTY_DEPENDENCY_ALLOWLIST,
  'makosh-desktop-call-recording-runtime': [
    { name: 'libc', kind: 'normal', source: 'crates_io', version: '=0.2.186', defaultFeatures: true, features: [] },
    { name: 'prost', kind: 'normal', source: 'crates_io', version: '=0.14.4', defaultFeatures: true, features: [] },
    { name: 'sha2', kind: 'normal', source: 'crates_io', version: '=0.11.0', defaultFeatures: false, features: [] },
    { name: 'tokio', kind: 'normal', source: 'crates_io', version: '=1.52.4', defaultFeatures: false, features: ['rt-multi-thread', 'time'] },
    { name: 'zeroize', kind: 'normal', source: 'crates_io', version: '=1.9.0', defaultFeatures: true, features: [] },
  ],
};

const DESKTOP_CALL_RECORDING_RELEASE_ASSEMBLY_THIRD_PARTY_DEPENDENCY_ALLOWLIST = {
  ...DESKTOP_CALL_RECORDING_RUNTIME_THIRD_PARTY_DEPENDENCY_ALLOWLIST,
  'makosh-desktop-call-recording-assembly': [
    { name: 'prost', kind: 'normal', source: 'crates_io', version: '=0.14.4', defaultFeatures: true, features: [] },
    { name: 'serde', kind: 'normal', source: 'crates_io', version: '=1.0.228', defaultFeatures: false, features: ['derive', 'std'] },
    { name: 'serde_json', kind: 'normal', source: 'crates_io', version: '=1.0.150', defaultFeatures: true, features: [] },
  ],
};

const CALL_TRANSCRIPTION_CONTRACT_CORE_THIRD_PARTY_DEPENDENCY_ALLOWLIST = {
  ...DESKTOP_CALL_RECORDING_RELEASE_ASSEMBLY_THIRD_PARTY_DEPENDENCY_ALLOWLIST,
  'makosh-call-transcription-api': [
    { name: 'prost', kind: 'normal', source: 'crates_io', version: '=0.14.4', defaultFeatures: true, features: [] },
    { name: 'sha2', kind: 'normal', source: 'crates_io', version: '=0.11.0', defaultFeatures: false, features: [] },
    { name: 'prost-build', kind: 'build', source: 'crates_io', version: '=0.14.4', defaultFeatures: true, features: [] },
    { name: 'protoc-bin-vendored', kind: 'build', source: 'crates_io', version: '=3.2.0', defaultFeatures: true, features: [] },
    { name: 'sha2', kind: 'build', source: 'crates_io', version: '=0.11.0', defaultFeatures: false, features: [] },
  ],
  'makosh-call-transcription-core': [
    { name: 'sha2', kind: 'normal', source: 'crates_io', version: '=0.11.0', defaultFeatures: false, features: [] },
  ],
};

const CALL_TRANSCRIPTION_PERSISTENCE_THIRD_PARTY_DEPENDENCY_ALLOWLIST = {
  ...CALL_TRANSCRIPTION_CONTRACT_CORE_THIRD_PARTY_DEPENDENCY_ALLOWLIST,
  'makosh-call-transcription-persistence': [
    { name: 'sha2', kind: 'normal', source: 'crates_io', version: '=0.11.0', defaultFeatures: false, features: [] },
    { name: 'sqlx', kind: 'normal', source: 'crates_io', version: '=0.9.0', defaultFeatures: false, features: ['postgres', 'runtime-tokio', 'tls-rustls-ring'] },
  ],
};

const CALL_TRANSCRIPTION_RUNTIME_THIRD_PARTY_DEPENDENCY_ALLOWLIST = {
  ...CALL_TRANSCRIPTION_PERSISTENCE_THIRD_PARTY_DEPENDENCY_ALLOWLIST,
  'makosh-call-transcription-runtime': [
    { name: 'getrandom', kind: 'normal', source: 'crates_io', version: '=0.4.3', defaultFeatures: true, features: [] },
    { name: 'libc', kind: 'normal', source: 'crates_io', version: '=0.2.186', defaultFeatures: true, features: [] },
    { name: 'prost', kind: 'normal', source: 'crates_io', version: '=0.14.4', defaultFeatures: true, features: [] },
    { name: 'sha2', kind: 'normal', source: 'crates_io', version: '=0.11.0', defaultFeatures: false, features: [] },
    { name: 'tokio', kind: 'normal', source: 'crates_io', version: '=1.52.4', defaultFeatures: false, features: ['rt-multi-thread', 'time'] },
    { name: 'zeroize', kind: 'normal', source: 'crates_io', version: '=1.9.0', defaultFeatures: true, features: [] },
  ],
};

const CALL_TRANSCRIPTION_RELEASE_ASSEMBLY_THIRD_PARTY_DEPENDENCY_ALLOWLIST = {
  ...CALL_TRANSCRIPTION_RUNTIME_THIRD_PARTY_DEPENDENCY_ALLOWLIST,
  'makosh-call-transcription-assembly': [
    { name: 'prost', kind: 'normal', source: 'crates_io', version: '=0.14.4', defaultFeatures: true, features: [] },
    { name: 'serde', kind: 'normal', source: 'crates_io', version: '=1.0.228', defaultFeatures: false, features: ['derive', 'std'] },
    { name: 'serde_json', kind: 'normal', source: 'crates_io', version: '=1.0.150', defaultFeatures: true, features: [] },
  ],
};

const PERSONS_CONTRACT_CORE_THIRD_PARTY_DEPENDENCY_ALLOWLIST = {
  ...CALL_TRANSCRIPTION_RELEASE_ASSEMBLY_THIRD_PARTY_DEPENDENCY_ALLOWLIST,
  'makosh-persons-api': [
    { name: 'prost', kind: 'normal', source: 'crates_io', version: '=0.14.4', defaultFeatures: true, features: [] },
    { name: 'sha2', kind: 'normal', source: 'crates_io', version: '=0.11.0', defaultFeatures: false, features: [] },
    { name: 'prost-build', kind: 'build', source: 'crates_io', version: '=0.14.4', defaultFeatures: true, features: [] },
    { name: 'protoc-bin-vendored', kind: 'build', source: 'crates_io', version: '=3.2.0', defaultFeatures: true, features: [] },
    { name: 'sha2', kind: 'build', source: 'crates_io', version: '=0.11.0', defaultFeatures: false, features: [] },
  ],
  'makosh-persons-core': [
    { name: 'sha2', kind: 'normal', source: 'crates_io', version: '=0.11.0', defaultFeatures: false, features: [] },
  ],
};

const PERSONS_PERSISTENCE_THIRD_PARTY_DEPENDENCY_ALLOWLIST = {
  ...PERSONS_CONTRACT_CORE_THIRD_PARTY_DEPENDENCY_ALLOWLIST,
  'makosh-persons-persistence': [
    { name: 'sha2', kind: 'normal', source: 'crates_io', version: '=0.11.0', defaultFeatures: false, features: [] },
    { name: 'sqlx', kind: 'normal', source: 'crates_io', version: '=0.9.0', defaultFeatures: false, features: ['postgres', 'runtime-tokio', 'tls-rustls-ring'] },
  ],
};

const PERSONS_RUNTIME_THIRD_PARTY_DEPENDENCY_ALLOWLIST = {
  ...PERSONS_PERSISTENCE_THIRD_PARTY_DEPENDENCY_ALLOWLIST,
  'makosh-persons-runtime': [
    { name: 'libc', kind: 'normal', source: 'crates_io', version: '=0.2.186', defaultFeatures: true, features: [] },
    { name: 'prost', kind: 'normal', source: 'crates_io', version: '=0.14.4', defaultFeatures: true, features: [] },
    { name: 'prost-types', kind: 'normal', source: 'crates_io', version: '=0.14.4', defaultFeatures: true, features: [] },
    { name: 'sha2', kind: 'normal', source: 'crates_io', version: '=0.11.0', defaultFeatures: false, features: [] },
    { name: 'tokio', kind: 'normal', source: 'crates_io', version: '=1.52.4', defaultFeatures: false, features: ['rt-multi-thread', 'time'] },
    { name: 'zeroize', kind: 'normal', source: 'crates_io', version: '=1.9.0', defaultFeatures: true, features: [] },
  ],
};

const PERSONS_ASSEMBLY_THIRD_PARTY_DEPENDENCY_ALLOWLIST = {
  ...PERSONS_RUNTIME_THIRD_PARTY_DEPENDENCY_ALLOWLIST,
  'makosh-persons-assembly': [
    { name: 'prost', kind: 'normal', source: 'crates_io', version: '=0.14.4', defaultFeatures: true, features: [] },
    { name: 'serde', kind: 'normal', source: 'crates_io', version: '=1.0.228', defaultFeatures: false, features: ['derive', 'std'] },
    { name: 'serde_json', kind: 'normal', source: 'crates_io', version: '=1.0.150', defaultFeatures: true, features: [] },
  ],
};

const MAIL_PERSONS_SYNC_CONTRACT_CORE_THIRD_PARTY_DEPENDENCY_ALLOWLIST = {
  ...PERSONS_ASSEMBLY_THIRD_PARTY_DEPENDENCY_ALLOWLIST,
  'makosh-mail-persons-sync-api': [
    { name: 'prost', kind: 'normal', source: 'crates_io', version: '=0.14.4', defaultFeatures: true, features: [] },
    { name: 'prost-types', kind: 'normal', source: 'crates_io', version: '=0.14.4', defaultFeatures: true, features: [] },
    { name: 'sha2', kind: 'normal', source: 'crates_io', version: '=0.11.0', defaultFeatures: false, features: [] },
    { name: 'prost-build', kind: 'build', source: 'crates_io', version: '=0.14.4', defaultFeatures: true, features: [] },
    { name: 'protoc-bin-vendored', kind: 'build', source: 'crates_io', version: '=3.2.0', defaultFeatures: true, features: [] },
    { name: 'sha2', kind: 'build', source: 'crates_io', version: '=0.11.0', defaultFeatures: false, features: [] },
  ],
  'makosh-mail-persons-sync-core': [
    { name: 'sha2', kind: 'normal', source: 'crates_io', version: '=0.11.0', defaultFeatures: false, features: [] },
    { name: 'prost-types', kind: 'dev', source: 'crates_io', version: '=0.14.4', defaultFeatures: true, features: [] },
  ],
  'makosh-mail-persons-sync-persistence': [
    { name: 'sha2', kind: 'normal', source: 'crates_io', version: '=0.11.0', defaultFeatures: false, features: [] },
    { name: 'sqlx', kind: 'normal', source: 'crates_io', version: '=0.9.0', defaultFeatures: false, features: ['postgres', 'runtime-tokio', 'tls-rustls-ring'] },
  ],
  'makosh-mail-persons-sync-runtime': [
    { name: 'libc', kind: 'normal', source: 'crates_io', version: '=0.2.186', defaultFeatures: true, features: [] },
    { name: 'prost', kind: 'normal', source: 'crates_io', version: '=0.14.4', defaultFeatures: true, features: [] },
    { name: 'prost-types', kind: 'normal', source: 'crates_io', version: '=0.14.4', defaultFeatures: true, features: [] },
    { name: 'sha2', kind: 'normal', source: 'crates_io', version: '=0.11.0', defaultFeatures: false, features: [] },
    { name: 'tokio', kind: 'normal', source: 'crates_io', version: '=1.52.4', defaultFeatures: false, features: ['rt-multi-thread', 'time'] },
    { name: 'zeroize', kind: 'normal', source: 'crates_io', version: '=1.9.0', defaultFeatures: true, features: [] },
  ],
  'makosh-mail-persons-sync-assembly': [
    { name: 'prost', kind: 'normal', source: 'crates_io', version: '=0.14.4', defaultFeatures: true, features: [] },
    { name: 'serde', kind: 'normal', source: 'crates_io', version: '=1.0.228', defaultFeatures: false, features: ['derive', 'std'] },
    { name: 'serde_json', kind: 'normal', source: 'crates_io', version: '=1.0.150', defaultFeatures: true, features: [] },
  ],
  'makosh-review-person-match-candidate-api': [
    { name: 'prost', kind: 'normal', source: 'crates_io', version: '=0.14.4', defaultFeatures: true, features: [] },
    { name: 'prost-types', kind: 'normal', source: 'crates_io', version: '=0.14.4', defaultFeatures: true, features: [] },
    { name: 'sha2', kind: 'normal', source: 'crates_io', version: '=0.11.0', defaultFeatures: false, features: [] },
    { name: 'prost-build', kind: 'build', source: 'crates_io', version: '=0.14.4', defaultFeatures: true, features: [] },
    { name: 'protoc-bin-vendored', kind: 'build', source: 'crates_io', version: '=3.2.0', defaultFeatures: true, features: [] },
    { name: 'sha2', kind: 'build', source: 'crates_io', version: '=0.11.0', defaultFeatures: false, features: [] },
  ],
  'makosh-review-person-match-candidate-core': [
    { name: 'sha2', kind: 'normal', source: 'crates_io', version: '=0.11.0', defaultFeatures: false, features: [] },
  ],
  'makosh-review-person-match-candidate-persistence': [
    { name: 'sha2', kind: 'normal', source: 'crates_io', version: '=0.11.0', defaultFeatures: false, features: [] },
    { name: 'sqlx', kind: 'normal', source: 'crates_io', version: '=0.9.0', defaultFeatures: false, features: ['postgres', 'runtime-tokio', 'tls-rustls-ring'] },
  ],
  'makosh-review-person-match-candidate-runtime': [
    { name: 'libc', kind: 'normal', source: 'crates_io', version: '=0.2.186', defaultFeatures: true, features: [] },
    { name: 'prost', kind: 'normal', source: 'crates_io', version: '=0.14.4', defaultFeatures: true, features: [] },
    { name: 'prost-types', kind: 'normal', source: 'crates_io', version: '=0.14.4', defaultFeatures: true, features: [] },
    { name: 'sha2', kind: 'normal', source: 'crates_io', version: '=0.11.0', defaultFeatures: false, features: [] },
    { name: 'tokio', kind: 'normal', source: 'crates_io', version: '=1.52.4', defaultFeatures: false, features: ['rt-multi-thread', 'time'] },
    { name: 'zeroize', kind: 'normal', source: 'crates_io', version: '=1.9.0', defaultFeatures: true, features: [] },
  ],
  'makosh-review-person-match-candidate-assembly': [
    { name: 'prost', kind: 'normal', source: 'crates_io', version: '=0.14.4', defaultFeatures: true, features: [] },
    { name: 'serde', kind: 'normal', source: 'crates_io', version: '=1.0.228', defaultFeatures: false, features: ['derive', 'std'] },
    { name: 'serde_json', kind: 'normal', source: 'crates_io', version: '=1.0.150', defaultFeatures: true, features: [] },
  ],
  'makosh-review-person-match-candidate-promotion-api': [
    { name: 'prost', kind: 'normal', source: 'crates_io', version: '=0.14.4', defaultFeatures: true, features: [] },
    { name: 'prost-types', kind: 'normal', source: 'crates_io', version: '=0.14.4', defaultFeatures: true, features: [] },
    { name: 'sha2', kind: 'normal', source: 'crates_io', version: '=0.11.0', defaultFeatures: false, features: [] },
    { name: 'prost-build', kind: 'build', source: 'crates_io', version: '=0.14.4', defaultFeatures: true, features: [] },
    { name: 'protoc-bin-vendored', kind: 'build', source: 'crates_io', version: '=3.2.0', defaultFeatures: true, features: [] },
    { name: 'sha2', kind: 'build', source: 'crates_io', version: '=0.11.0', defaultFeatures: false, features: [] },
  ],
  'makosh-reviewed-person-match-candidate-promotion-core': [
    { name: 'prost', kind: 'normal', source: 'crates_io', version: '=0.14.4', defaultFeatures: true, features: [] },
    { name: 'sha2', kind: 'normal', source: 'crates_io', version: '=0.11.0', defaultFeatures: false, features: [] },
  ],
  'makosh-reviewed-person-match-candidate-promotion-persistence': [
    { name: 'sha2', kind: 'normal', source: 'crates_io', version: '=0.11.0', defaultFeatures: false, features: [] },
    { name: 'sqlx', kind: 'normal', source: 'crates_io', version: '=0.9.0', defaultFeatures: false, features: ['postgres', 'runtime-tokio', 'tls-rustls-ring'] },
  ],
  'makosh-reviewed-person-match-candidate-promotion-runtime': [
    { name: 'libc', kind: 'normal', source: 'crates_io', version: '=0.2.186', defaultFeatures: true, features: [] },
    { name: 'prost', kind: 'normal', source: 'crates_io', version: '=0.14.4', defaultFeatures: true, features: [] },
    { name: 'prost-types', kind: 'normal', source: 'crates_io', version: '=0.14.4', defaultFeatures: true, features: [] },
    { name: 'sha2', kind: 'normal', source: 'crates_io', version: '=0.11.0', defaultFeatures: false, features: [] },
    { name: 'tokio', kind: 'normal', source: 'crates_io', version: '=1.52.4', defaultFeatures: false, features: ['rt-multi-thread', 'time'] },
    { name: 'zeroize', kind: 'normal', source: 'crates_io', version: '=1.9.0', defaultFeatures: true, features: [] },
  ],
  'makosh-reviewed-person-match-candidate-promotion-assembly': [
    { name: 'prost', kind: 'normal', source: 'crates_io', version: '=0.14.4', defaultFeatures: true, features: [] },
    { name: 'serde', kind: 'normal', source: 'crates_io', version: '=1.0.228', defaultFeatures: false, features: ['derive', 'std'] },
    { name: 'serde_json', kind: 'normal', source: 'crates_io', version: '=1.0.150', defaultFeatures: true, features: [] },
  ],
};

const FORBIDDEN_DEPENDENCIES = [
  'async-nats',
  'nats',
  'sqlx',
  'tokio-postgres',
  'postgres',
  'diesel',
  'sea-orm',
  'deadpool-postgres',
  'bb8-postgres',
  'reqwest',
  'ureq',
  'isahc',
  'surf',
  'awc',
];

const RECOVERY_FORBIDDEN_DEPENDENCY_PREFIXES = [
  'makosh-vault-',
  'makosh-storage-',
  'makosh-integration-',
  'makosh-provider-',
];

const VAULT_FOUNDATION_FORBIDDEN_DEPENDENCY_PREFIXES = [
  'makosh-storage-',
  'makosh-integration-',
  'makosh-provider-',
];

const STORAGE_FOUNDATION_FORBIDDEN_DEPENDENCY_PREFIXES = [
  'makosh-integration-',
  'makosh-provider-',
];

const KERNEL_PROFILE_KEYS = [
  'maximumState',
  'allowedStates',
  'forbiddenStates',
  'activeComponents',
  'transport',
  'onlineOperations',
  'bootstrapOperations',
  'offlineOperations',
  'externalServices',
  'managedChildren',
  'publicGatewayEnabled',
  'networkListenerEnabled',
  'moduleRegistrationEnabled',
  'managedLaunchEnabled',
  'natsDataPlaneEnabled',
  'businessDataPlaneEnabled',
  'wholeInstanceBackupEnabled',
  'clock',
];

const KERNEL_PROFILE = {
  maximumState: 'recovery_only',
  allowedStates: [
    'cold_start',
    'bootstrap',
    'recovery_only',
    'quiescing',
    'draining',
    'stopped',
    'fatal',
  ],
  forbiddenStates: [
    'infrastructure_starting',
    'modules_starting',
    'ready',
    'degraded',
  ],
  activeComponents: ['supervisor', 'core_gateway'],
  transport: 'local_ipc_only',
  onlineOperations: [
    'status',
    'control_store_validate',
    'control_store_export',
    'shutdown',
  ],
  bootstrapOperations: ['initial_owner_enrollment_inherited_fd'],
  offlineOperations: ['control_store_restore', 'control_store_reset'],
  externalServices: [],
  managedChildren: [],
  networkListenerEnabled: false,
  moduleRegistrationEnabled: false,
  managedLaunchEnabled: false,
};

const MODULE_CONTROL_PROFILE = {
  maximumState: 'module_control_plane',
  allowedStates: ['cold_start', 'bootstrap', 'recovery_only', 'module_control_plane', 'quiescing', 'draining', 'stopped', 'fatal'],
  forbiddenStates: ['infrastructure_starting', 'modules_starting', 'ready', 'degraded'],
  activeComponents: ['supervisor', 'module_registry', 'capability_router', 'core_gateway', 'settings_registry'],
  transport: 'local_ipc_only',
  onlineOperations: ['status', 'control_store_validate', 'control_store_export', 'shutdown', 'module_registration', 'owner_control', 'external_runtime_session'],
  bootstrapOperations: ['initial_owner_enrollment_inherited_fd'],
  offlineOperations: ['control_store_restore', 'control_store_reset'],
  externalServices: [],
  managedChildren: [],
  networkListenerEnabled: false,
  moduleRegistrationEnabled: true,
  managedLaunchEnabled: false,
};

const SERVER_BOOTSTRAP_PAIRING_PROFILE = {
  maximumState: 'module_control_plane',
  allowedStates: ['cold_start', 'bootstrap', 'recovery_only', 'module_control_plane', 'quiescing', 'draining', 'stopped', 'fatal'],
  forbiddenStates: ['infrastructure_starting', 'modules_starting', 'ready', 'degraded'],
  activeComponents: ['supervisor', 'module_registry', 'capability_router', 'core_gateway', 'settings_registry'],
  transport: 'local_ipc_and_one_shot_bootstrap_tls',
  onlineOperations: ['status', 'control_store_validate', 'control_store_export', 'shutdown', 'module_registration', 'owner_control', 'external_runtime_session'],
  bootstrapOperations: ['initial_owner_enrollment_inherited_fd', 'server_bootstrap_pairing'],
  offlineOperations: ['control_store_restore', 'control_store_reset'],
  externalServices: [],
  managedChildren: [],
  networkListenerEnabled: true,
  moduleRegistrationEnabled: true,
  managedLaunchEnabled: false,
};

const MANAGED_LAUNCH_TRUST_PROFILE = {
  maximumState: 'module_control_plane',
  allowedStates: ['cold_start', 'bootstrap', 'recovery_only', 'module_control_plane', 'quiescing', 'draining', 'stopped', 'fatal'],
  forbiddenStates: ['infrastructure_starting', 'modules_starting', 'ready', 'degraded'],
  activeComponents: ['supervisor', 'module_registry', 'capability_router', 'core_gateway', 'settings_registry'],
  transport: 'local_ipc_and_one_shot_bootstrap_tls',
  onlineOperations: ['status', 'control_store_validate', 'control_store_export', 'shutdown', 'module_registration', 'owner_control', 'external_runtime_session'],
  bootstrapOperations: ['initial_owner_enrollment_inherited_fd', 'server_bootstrap_pairing'],
  offlineOperations: ['control_store_restore', 'control_store_reset'],
  externalServices: [],
  managedChildren: ['bundled_native_module_runtime'],
  networkListenerEnabled: true,
  moduleRegistrationEnabled: true,
  managedLaunchEnabled: true,
};

const FIRST_OWNER_PROFILE = {
  ...MANAGED_LAUNCH_TRUST_PROFILE,
  publicGatewayEnabled: true,
  natsDataPlaneEnabled: true,
  businessDataPlaneEnabled: true,
  wholeInstanceBackupEnabled: true,
};

const FIRST_OWNER_INVENTORY = {
  domains: ['communications'],
  integrations: [],
  workflows: [],
  engines: [],
  businessCapabilities: [
    'communications.attachment.blob-admission.observe.v1',
    'communications.attachment.safety-verdict.observe.v1',
    'communications.blob.v1',
    'communications.events.v1',
    'communications.observe.v1',
    'communications.query.v1',
    'communications.search.index.v1',
    'communications.storage.v1',
  ],
};

const ATTACHMENT_SECURITY_ENGINE_INVENTORY = {
  domains: ['communications'],
  integrations: [],
  workflows: [],
  engines: ['attachment_security'],
  businessCapabilities: [
    'attachment_security.blob.v1',
    'attachment_security.candidate.observe.v1',
    'attachment_security.communications-state.observe.v1',
    'attachment_security.storage.v1',
    'attachment_security.verdict.publish.v1',
    ...FIRST_OWNER_INVENTORY.businessCapabilities,
  ],
};

const MAIL_OUTBOUND_MIME_ATTACHMENTS_INVENTORY = {
  domains: ['communications'],
  integrations: ['mail'],
  workflows: [],
  engines: ['attachment_security'],
  businessCapabilities: [
    ...ATTACHMENT_SECURITY_ENGINE_INVENTORY.businessCapabilities,
    'mail.attachment-anchor.consume.v1',
    'mail.attachment-blob-admission.publish.v1',
    'mail.attachment-safety-state.consume.v1',
    'mail.attachment.scan-candidate.publish.v1',
    'mail.blob.v1',
    'mail.communication-observed.publish.v1',
    'mail.delivery.query.v1',
    'mail.delivery.v1',
    'mail.gmail.credentials.v1',
    'mail.gmail.oauth-refresh.credentials.v1',
    'mail.gmail.oauth-setup.credentials.v1',
    'mail.imap.credentials.v1',
    'mail.oauth.complete.v1',
    'mail.oauth.query.v1',
    'mail.oauth.refresh.v1',
    'mail.oauth.start.v1',
    'mail.smtp.credentials.v1',
    'mail.storage.v1',
    'mail.sync.v1',
  ],
};

const COMMUNICATIONS_CONTENT_READ_INVENTORY = {
  ...MAIL_OUTBOUND_MIME_ATTACHMENTS_INVENTORY,
  businessCapabilities: [
    ...MAIL_OUTBOUND_MIME_ATTACHMENTS_INVENTORY.businessCapabilities,
    'communications.content.v1',
  ].sort(),
};

const COMMUNICATIONS_SAVED_SEARCH_INVENTORY = {
  ...COMMUNICATIONS_CONTENT_READ_INVENTORY,
  businessCapabilities: [
    ...COMMUNICATIONS_CONTENT_READ_INVENTORY.businessCapabilities,
    'communications.saved-search.v1',
  ].sort(),
};

const COMMUNICATIONS_SENDER_INSIGHTS_INVENTORY = {
  ...COMMUNICATIONS_SAVED_SEARCH_INVENTORY,
  businessCapabilities: [
    ...COMMUNICATIONS_SAVED_SEARCH_INVENTORY.businessCapabilities,
    'communications.sender-insights.v1',
  ].sort(),
};

const COMMUNICATIONS_EXPORT_INVENTORY = {
  ...COMMUNICATIONS_SENDER_INSIGHTS_INVENTORY,
  workflows: ['communications_export'],
  businessCapabilities: [
    ...COMMUNICATIONS_SENDER_INSIGHTS_INVENTORY.businessCapabilities,
    'communications.export-source.blob.v1',
    'communications.export-source.v1',
    'communications.export.v1',
    'communications_export.blob.v1',
    'communications_export.events.v1',
    'communications_export.storage.v1',
  ].sort(),
};

const COMMUNICATION_DELIVERY_INTENT_INVENTORY = {
  ...COMMUNICATIONS_EXPORT_INVENTORY,
  workflows: [
    'communication_cross_channel_forward',
    'communication_delivery_intent',
    'communications_export',
  ],
  businessCapabilities: [
    ...COMMUNICATIONS_EXPORT_INVENTORY.businessCapabilities,
    'communication.cross_channel_forward.v1',
    'communication_cross_channel_forward.blob.v1',
    'communication_cross_channel_forward.delivery_rejected.v1',
    'communication_cross_channel_forward.delivery_submit.v1',
    'communication_cross_channel_forward.delivery_submitted.v1',
    'communication_cross_channel_forward.source_prepare.v1',
    'communication_cross_channel_forward.source_prepared.v1',
    'communication_cross_channel_forward.source_rejected.v1',
    'communication_cross_channel_forward.storage.v1',
    'communication_delivery_intent.blob.v1',
    'communication_delivery_intent.ingress_rejected.v1',
    'communication_delivery_intent.ingress_submit.v1',
    'communication_delivery_intent.ingress_submitted.v1',
    'communication_delivery_intent.mail.events.v1',
    'communication_delivery_intent.storage.v1',
    'communication_delivery_intent.telegram.events.v1',
    'communication_delivery_intent.whatsapp.events.v1',
    'communication_delivery_intent.zulip.events.v1',
    'communications.cross-channel-forward-source.blob.v1',
    'communications.cross-channel-forward-source.v1',
  ].sort(),
};

const REVIEW_COMMUNICATIONS_ATTENTION_CONTRACT_CORE_INVENTORY = {
  ...COMMUNICATION_DELIVERY_INTENT_INVENTORY,
  domains: ['communications', 'review'],
  businessCapabilities: [
    ...COMMUNICATION_DELIVERY_INTENT_INVENTORY.businessCapabilities,
    'review.communication-attention.command.v1',
    'review.communication-attention.query.v1',
    'review.communication-attention.realtime.v1',
  ].sort(),
};

const REVIEW_COMMUNICATIONS_ATTENTION_LIVE_INVENTORY = {
  ...REVIEW_COMMUNICATIONS_ATTENTION_CONTRACT_CORE_INVENTORY,
  businessCapabilities: [
    ...REVIEW_COMMUNICATIONS_ATTENTION_CONTRACT_CORE_INVENTORY.businessCapabilities,
    'review.communication-attention.storage.v1',
  ].sort(),
};

const COMMUNICATIONS_AI_SOURCE_CONTRACT_INVENTORY = {
  ...REVIEW_COMMUNICATIONS_ATTENTION_LIVE_INVENTORY,
  workflows: [
    ...REVIEW_COMMUNICATIONS_ATTENTION_LIVE_INVENTORY.workflows,
    'communication_reply_suggestion',
  ].sort(),
  engines: [
    ...REVIEW_COMMUNICATIONS_ATTENTION_LIVE_INVENTORY.engines,
    'ai',
  ].sort(),
  businessCapabilities: [
    ...REVIEW_COMMUNICATIONS_ATTENTION_LIVE_INVENTORY.businessCapabilities,
    'communications.ai-reply-source.blob.v1',
    'communications.ai-reply-source.v1',
    'communications.ai-summary-source.blob.v1',
    'communications.ai-summary-source.v1',
  ].sort(),
};

const ATTACHMENT_ARCHIVE_INSPECTION_CONTRACT_CORE_INVENTORY = {
  ...COMMUNICATIONS_AI_SOURCE_CONTRACT_INVENTORY,
  engines: [
    ...COMMUNICATIONS_AI_SOURCE_CONTRACT_INVENTORY.engines,
    'attachment_archive_inspection',
  ].sort(),
  businessCapabilities: [
    ...COMMUNICATIONS_AI_SOURCE_CONTRACT_INVENTORY.businessCapabilities,
    'attachment_security.archive-delegation-result.publish.v1',
    'attachment_security.archive-inspection-delegation.v1',
  ].sort(),
};

const ATTACHMENT_ARCHIVE_INSPECTION_RUNTIME_INVENTORY = {
  ...ATTACHMENT_ARCHIVE_INSPECTION_CONTRACT_CORE_INVENTORY,
  businessCapabilities: [
    ...ATTACHMENT_ARCHIVE_INSPECTION_CONTRACT_CORE_INVENTORY.businessCapabilities,
    'attachment_archive_inspection.blob.v1',
    'attachment_archive_inspection.candidate.observe.v1',
    'attachment_archive_inspection.custody-request.publish.v1',
    'attachment_archive_inspection.custody-result.consume.v1',
    'attachment_archive_inspection.safety-state.observe.v1',
    'attachment_archive_inspection.storage.v1',
  ].sort(),
};

const ATTACHMENT_ARCHIVE_INSPECTION_CLIENT_INVENTORY = {
  ...ATTACHMENT_ARCHIVE_INSPECTION_RUNTIME_INVENTORY,
  businessCapabilities: [
    ...ATTACHMENT_ARCHIVE_INSPECTION_RUNTIME_INVENTORY.businessCapabilities,
    'attachment.archive_inspection.v1',
  ].sort(),
};

const COMMUNICATION_SUMMARY_BUILD_UNITS_INVENTORY = {
  ...ATTACHMENT_ARCHIVE_INSPECTION_CLIENT_INVENTORY,
  workflows: [
    ...ATTACHMENT_ARCHIVE_INSPECTION_CLIENT_INVENTORY.workflows,
    'communication_summary',
  ].sort(),
  businessCapabilities: [
    ...ATTACHMENT_ARCHIVE_INSPECTION_CLIENT_INVENTORY.businessCapabilities,
    'communication.summary.v1',
    'communication_summary.inference.v1',
    'communication_summary.source.blob.v1',
    'communication_summary.source_prepare.v1',
    'communication_summary.source_prepared.v1',
    'communication_summary.source_rejected.v1',
    'communication_summary.storage.v1',
  ].sort(),
};

const COMMUNICATION_TRANSLATION_CONTRACT_CORE_INVENTORY = {
  ...COMMUNICATION_SUMMARY_BUILD_UNITS_INVENTORY,
  workflows: [
    ...COMMUNICATION_SUMMARY_BUILD_UNITS_INVENTORY.workflows,
    'communication_translation',
  ].sort(),
  businessCapabilities: [
    ...COMMUNICATION_SUMMARY_BUILD_UNITS_INVENTORY.businessCapabilities,
    'communication.translation.v1',
  ].sort(),
};

const COMMUNICATION_TRANSLATION_CROSS_OWNER_CONTRACTS_INVENTORY = {
  ...COMMUNICATION_TRANSLATION_CONTRACT_CORE_INVENTORY,
  businessCapabilities: [
    ...COMMUNICATION_TRANSLATION_CONTRACT_CORE_INVENTORY.businessCapabilities,
    'ai.provider.translate.v1',
    'ai.translation.request.v1',
    'communication_translation.inference.v1',
    'communication_translation.source.blob.v1',
    'communication_translation.source_prepare.v1',
    'communication_translation.source_prepared.v1',
    'communication_translation.source_rejected.v1',
    'communications.ai-translation-source.blob.v1',
    'communications.ai-translation-source.v1',
  ].sort(),
};

const COMMUNICATION_TRANSLATION_PERSISTENCE_INVENTORY = {
  ...COMMUNICATION_TRANSLATION_CROSS_OWNER_CONTRACTS_INVENTORY,
  businessCapabilities: [
    ...COMMUNICATION_TRANSLATION_CROSS_OWNER_CONTRACTS_INVENTORY.businessCapabilities,
    'communication_translation.storage.v1',
  ].sort(),
};

const COMMUNICATION_TRANSLATION_RUNTIME_INVENTORY = {
  ...COMMUNICATION_TRANSLATION_PERSISTENCE_INVENTORY,
};

const COMMUNICATION_EXPLANATION_CONTRACT_CORE_INVENTORY = {
  ...COMMUNICATION_TRANSLATION_RUNTIME_INVENTORY,
  workflows: [
    ...COMMUNICATION_TRANSLATION_RUNTIME_INVENTORY.workflows,
    'communication_explanation',
  ].sort(),
  businessCapabilities: [
    ...COMMUNICATION_TRANSLATION_RUNTIME_INVENTORY.businessCapabilities,
    'communication.explanation.v1',
  ].sort(),
};

const COMMUNICATION_EXPLANATION_CROSS_OWNER_CONTRACTS_INVENTORY = {
  ...COMMUNICATION_EXPLANATION_CONTRACT_CORE_INVENTORY,
  businessCapabilities: [
    ...COMMUNICATION_EXPLANATION_CONTRACT_CORE_INVENTORY.businessCapabilities,
    'ai.explanation.request.v1',
    'ai.provider.explain.v1',
    'communication_explanation.inference.v1',
    'communication_explanation.source.blob.v1',
    'communication_explanation.source_prepare.v1',
    'communication_explanation.source_prepared.v1',
    'communication_explanation.source_rejected.v1',
    'communications.ai-explanation-source.blob.v1',
    'communications.ai-explanation-source.v1',
  ].sort(),
};

const COMMUNICATION_EXPLANATION_PERSISTENCE_INVENTORY = {
  ...COMMUNICATION_EXPLANATION_CROSS_OWNER_CONTRACTS_INVENTORY,
  businessCapabilities: [
    ...COMMUNICATION_EXPLANATION_CROSS_OWNER_CONTRACTS_INVENTORY.businessCapabilities,
    'communication_explanation.storage.v1',
  ].sort(),
};

const COMMUNICATION_EXPLANATION_RUNTIME_INVENTORY = {
  ...COMMUNICATION_EXPLANATION_PERSISTENCE_INVENTORY,
};

const COMMUNICATION_RECIPIENT_SUGGESTION_CONTRACT_CORE_INVENTORY = {
  ...COMMUNICATION_EXPLANATION_RUNTIME_INVENTORY,
  workflows: [
    ...COMMUNICATION_EXPLANATION_RUNTIME_INVENTORY.workflows,
    'communication_recipient_suggestion',
  ].sort(),
  businessCapabilities: [
    ...COMMUNICATION_EXPLANATION_RUNTIME_INVENTORY.businessCapabilities,
    'communication.recipient-suggestion.v1',
  ].sort(),
};

const COMMUNICATION_RECIPIENT_SUGGESTION_SOURCE_CONTRACT_INVENTORY = {
  ...COMMUNICATION_RECIPIENT_SUGGESTION_CONTRACT_CORE_INVENTORY,
  businessCapabilities: [
    ...COMMUNICATION_RECIPIENT_SUGGESTION_CONTRACT_CORE_INVENTORY.businessCapabilities,
    'communication_recipient_suggestion.source.blob.v1',
    'communication_recipient_suggestion.source_prepare.v1',
    'communication_recipient_suggestion.source_prepared.v1',
    'communication_recipient_suggestion.source_rejected.v1',
    'communications.recipient-source.v1',
  ].sort(),
};

const COMMUNICATION_RECIPIENT_SUGGESTION_PERSISTENCE_INVENTORY = {
  ...COMMUNICATION_RECIPIENT_SUGGESTION_SOURCE_CONTRACT_INVENTORY,
  businessCapabilities: [
    ...COMMUNICATION_RECIPIENT_SUGGESTION_SOURCE_CONTRACT_INVENTORY.businessCapabilities,
    'communication_recipient_suggestion.storage.v1',
  ].sort(),
};

const COMMUNICATION_RECIPIENT_SUGGESTION_SOURCE_PRODUCER_INVENTORY = {
  ...COMMUNICATION_RECIPIENT_SUGGESTION_PERSISTENCE_INVENTORY,
  businessCapabilities: [
    ...COMMUNICATION_RECIPIENT_SUGGESTION_PERSISTENCE_INVENTORY.businessCapabilities,
    'communications.recipient-source.blob.v1',
  ].sort(),
};

const COMMUNICATION_TASK_CANDIDATE_CONTRACT_CORE_SOURCE_INVENTORY = {
  ...COMMUNICATION_RECIPIENT_SUGGESTION_SOURCE_PRODUCER_INVENTORY,
  workflows: [
    ...COMMUNICATION_RECIPIENT_SUGGESTION_SOURCE_PRODUCER_INVENTORY.workflows,
    'communication_task_candidate_extraction',
  ].sort(),
  businessCapabilities: [
    ...COMMUNICATION_RECIPIENT_SUGGESTION_SOURCE_PRODUCER_INVENTORY.businessCapabilities,
    'communication.task-candidate-extraction.v1',
    'communication_task_candidate_extraction.source.blob.v1',
    'communication_task_candidate_extraction.source_prepare.v1',
    'communication_task_candidate_extraction.source_prepared.v1',
    'communication_task_candidate_extraction.source_rejected.v1',
    'communications.task-source.v1',
  ].sort(),
};

const COMMUNICATION_TASK_CANDIDATE_PERSISTENCE_INVENTORY = {
  ...COMMUNICATION_TASK_CANDIDATE_CONTRACT_CORE_SOURCE_INVENTORY,
  businessCapabilities: [
    ...COMMUNICATION_TASK_CANDIDATE_CONTRACT_CORE_SOURCE_INVENTORY.businessCapabilities,
    'communication_task_candidate_extraction.storage.v1',
  ].sort(),
};

const COMMUNICATION_TASK_CANDIDATE_SOURCE_PRODUCER_INVENTORY = {
  ...COMMUNICATION_TASK_CANDIDATE_PERSISTENCE_INVENTORY,
  businessCapabilities: [
    ...COMMUNICATION_TASK_CANDIDATE_PERSISTENCE_INVENTORY.businessCapabilities,
    'communications.task-source.blob.v1',
  ].sort(),
};

const REVIEW_TASK_CANDIDATE_CORE_INVENTORY = {
  ...COMMUNICATION_TASK_CANDIDATE_SOURCE_PRODUCER_INVENTORY,
  businessCapabilities: [
    ...COMMUNICATION_TASK_CANDIDATE_SOURCE_PRODUCER_INVENTORY.businessCapabilities,
    'review.task-candidate.blob.v1',
    'review.task-candidate.client.v1',
    'review.task-candidate.promotion.v1',
    'review.task-candidate.submission.v1',
  ].sort(),
};

const REVIEW_TASK_CANDIDATE_PERSISTENCE_INVENTORY = {
  ...REVIEW_TASK_CANDIDATE_CORE_INVENTORY,
  businessCapabilities: [
    ...REVIEW_TASK_CANDIDATE_CORE_INVENTORY.businessCapabilities,
    'review.task-candidate.storage.v1',
  ].sort(),
};

const TASKS_REVIEWED_CANDIDATE_CONTRACT_CORE_INVENTORY = {
  ...REVIEW_TASK_CANDIDATE_PERSISTENCE_INVENTORY,
  domains: [...REVIEW_TASK_CANDIDATE_PERSISTENCE_INVENTORY.domains, 'tasks'].sort(),
  businessCapabilities: [
    ...REVIEW_TASK_CANDIDATE_PERSISTENCE_INVENTORY.businessCapabilities,
    'tasks.reviewed-candidate.blob.v1',
    'tasks.reviewed-candidate.command.v1',
  ].sort(),
};

const TASKS_REVIEWED_CANDIDATE_PERSISTENCE_INVENTORY = {
  ...TASKS_REVIEWED_CANDIDATE_CONTRACT_CORE_INVENTORY,
  businessCapabilities: [
    ...TASKS_REVIEWED_CANDIDATE_CONTRACT_CORE_INVENTORY.businessCapabilities,
    'tasks.storage.v1',
  ].sort(),
};

const REVIEWED_TASK_CANDIDATE_PROMOTION_CONTRACT_CORE_INVENTORY = {
  ...TASKS_REVIEWED_CANDIDATE_PERSISTENCE_INVENTORY,
  workflows: [
    ...TASKS_REVIEWED_CANDIDATE_PERSISTENCE_INVENTORY.workflows,
    'reviewed_task_candidate_promotion',
  ].sort(),
  businessCapabilities: [
    ...TASKS_REVIEWED_CANDIDATE_PERSISTENCE_INVENTORY.businessCapabilities,
    'review.task-candidate.promotion-result.v1',
  ].sort(),
};

const REVIEWED_TASK_CANDIDATE_PROMOTION_PERSISTENCE_INVENTORY = {
  ...REVIEWED_TASK_CANDIDATE_PROMOTION_CONTRACT_CORE_INVENTORY,
  businessCapabilities: [
    ...REVIEWED_TASK_CANDIDATE_PROMOTION_CONTRACT_CORE_INVENTORY.businessCapabilities,
    'reviewed_task_candidate_promotion.storage.v1',
  ].sort(),
};

const REVIEWED_TASK_CANDIDATE_PROMOTION_RUNTIME_INVENTORY = {
  ...REVIEWED_TASK_CANDIDATE_PROMOTION_PERSISTENCE_INVENTORY,
  businessCapabilities: [
    ...REVIEWED_TASK_CANDIDATE_PROMOTION_PERSISTENCE_INVENTORY.businessCapabilities,
    'reviewed_task_candidate_promotion.review-approved.consume.v1',
    'reviewed_task_candidate_promotion.review-result.publish.v1',
    'reviewed_task_candidate_promotion.tasks-command.publish.v1',
    'reviewed_task_candidate_promotion.tasks-created.consume.v1',
    'reviewed_task_candidate_promotion.tasks-rejected.consume.v1',
  ].sort(),
};

const REVIEW_TASK_CANDIDATE_PROMOTION_RESULT_CONSUMER_INVENTORY = {
  ...REVIEWED_TASK_CANDIDATE_PROMOTION_RUNTIME_INVENTORY,
  businessCapabilities: [
    ...REVIEWED_TASK_CANDIDATE_PROMOTION_RUNTIME_INVENTORY.businessCapabilities,
    'review.task-candidate.promotion-result.consumer.v1',
  ].sort(),
};

const COMMUNICATION_NOTE_CANDIDATE_CONTRACT_CORE_INVENTORY = {
  ...REVIEW_TASK_CANDIDATE_PROMOTION_RESULT_CONSUMER_INVENTORY,
  workflows: [
    ...REVIEW_TASK_CANDIDATE_PROMOTION_RESULT_CONSUMER_INVENTORY.workflows,
    'communication_note_candidate_extraction',
  ].sort(),
  businessCapabilities: [
    ...REVIEW_TASK_CANDIDATE_PROMOTION_RESULT_CONSUMER_INVENTORY.businessCapabilities,
    'communication.note-candidate-extraction.v1',
    'communication_note_candidate_extraction.source.blob.v1',
    'communications.note-source.v1',
  ].sort(),
};

const COMMUNICATION_NOTE_CANDIDATE_PERSISTENCE_INVENTORY = {
  ...COMMUNICATION_NOTE_CANDIDATE_CONTRACT_CORE_INVENTORY,
  businessCapabilities: [
    ...COMMUNICATION_NOTE_CANDIDATE_CONTRACT_CORE_INVENTORY.businessCapabilities,
    'communication_note_candidate_extraction.storage.v1',
  ].sort(),
};

const REVIEW_NOTE_CANDIDATE_CONTRACT_CORE_INVENTORY = {
  ...COMMUNICATION_NOTE_CANDIDATE_PERSISTENCE_INVENTORY,
  businessCapabilities: [
    ...COMMUNICATION_NOTE_CANDIDATE_PERSISTENCE_INVENTORY.businessCapabilities,
    'review.note-candidate.blob.v1',
    'review.note-candidate.client.v1',
    'review.note-candidate.promotion.v1',
    'review.note-candidate.submission.v1',
  ].sort(),
};

const KNOWLEDGE_VERIFIED_NOTE_CONTRACT_CORE_INVENTORY = {
  ...REVIEW_NOTE_CANDIDATE_CONTRACT_CORE_INVENTORY,
  domains: [...REVIEW_NOTE_CANDIDATE_CONTRACT_CORE_INVENTORY.domains, 'knowledge'].sort(),
  businessCapabilities: [
    ...REVIEW_NOTE_CANDIDATE_CONTRACT_CORE_INVENTORY.businessCapabilities,
    'knowledge.reviewed-candidate.blob.v1',
    'knowledge.reviewed-candidate.command.v1',
  ].sort(),
};

const KNOWLEDGE_VERIFIED_NOTE_MANAGED_RUNTIME_INVENTORY = {
  ...KNOWLEDGE_VERIFIED_NOTE_CONTRACT_CORE_INVENTORY,
  businessCapabilities: [
    ...KNOWLEDGE_VERIFIED_NOTE_CONTRACT_CORE_INVENTORY.businessCapabilities,
    'knowledge.reviewed-candidate.created.publisher.v1',
    'knowledge.reviewed-candidate.rejected.publisher.v1',
    'knowledge.storage.v1',
  ].sort(),
};

const REVIEW_NOTE_CANDIDATE_MANAGED_RUNTIME_INVENTORY = {
  ...KNOWLEDGE_VERIFIED_NOTE_MANAGED_RUNTIME_INVENTORY,
  businessCapabilities: [
    ...KNOWLEDGE_VERIFIED_NOTE_MANAGED_RUNTIME_INVENTORY.businessCapabilities,
    'review.note-candidate.promotion-result.consumer.v1',
    'review.note-candidate.promotion-result.v1',
    'review.note-candidate.storage.v1',
  ].sort(),
};

const REVIEWED_NOTE_CANDIDATE_PROMOTION_INVENTORY = {
  ...REVIEW_NOTE_CANDIDATE_MANAGED_RUNTIME_INVENTORY,
  workflows: [
    ...REVIEW_NOTE_CANDIDATE_MANAGED_RUNTIME_INVENTORY.workflows,
    'reviewed_note_candidate_promotion',
  ].sort(),
  businessCapabilities: [
    ...REVIEW_NOTE_CANDIDATE_MANAGED_RUNTIME_INVENTORY.businessCapabilities,
    'reviewed-note-candidate-promotion.source.blob.v1',
    'reviewed_note_candidate_promotion.knowledge-command.publish.v1',
    'reviewed_note_candidate_promotion.knowledge-created.consume.v1',
    'reviewed_note_candidate_promotion.knowledge-rejected.consume.v1',
    'reviewed_note_candidate_promotion.review-approved.consume.v1',
    'reviewed_note_candidate_promotion.review-result.publish.v1',
    'reviewed_note_candidate_promotion.storage.v1',
  ].sort(),
};

const COMMUNICATION_NOTE_CANDIDATE_ASSEMBLY_INVENTORY = {
  ...REVIEWED_NOTE_CANDIDATE_PROMOTION_INVENTORY,
  businessCapabilities: [
    ...REVIEWED_NOTE_CANDIDATE_PROMOTION_INVENTORY.businessCapabilities,
    'communication_note_candidate_extraction.source_prepare.v1',
    'communication_note_candidate_extraction.source_prepared.v1',
    'communication_note_candidate_extraction.source_rejected.v1',
    'communications.note-source.blob.v1',
  ].sort(),
};

const ATTACHMENT_TEXT_EXTRACTION_CONTRACT_CORE_INVENTORY = {
  ...COMMUNICATION_NOTE_CANDIDATE_ASSEMBLY_INVENTORY,
  workflows: [
    ...COMMUNICATION_NOTE_CANDIDATE_ASSEMBLY_INVENTORY.workflows,
    'attachment_text_extraction',
  ].sort(),
  businessCapabilities: [
    ...COMMUNICATION_NOTE_CANDIDATE_ASSEMBLY_INVENTORY.businessCapabilities,
    'attachment.text_extraction.v1',
    'attachment_security.text-extraction-delegation-result.publish.v1',
    'attachment_security.text-extraction-delegation.v1',
  ].sort(),
};

const ATTACHMENT_PREVIEW_FOUNDATION_INVENTORY = {
  ...ATTACHMENT_TEXT_EXTRACTION_CONTRACT_CORE_INVENTORY,
  workflows: [
    ...ATTACHMENT_TEXT_EXTRACTION_CONTRACT_CORE_INVENTORY.workflows,
    'attachment_preview',
  ].sort(),
  businessCapabilities: [
    ...ATTACHMENT_TEXT_EXTRACTION_CONTRACT_CORE_INVENTORY.businessCapabilities,
    'attachment.preview.v1',
  ].sort(),
};

const ATTACHMENT_PREVIEW_RETAINED_EVIDENCE_REPLAY_INVENTORY = {
  ...ATTACHMENT_PREVIEW_FOUNDATION_INVENTORY,
  workflows: [
    ...ATTACHMENT_PREVIEW_FOUNDATION_INVENTORY.workflows,
    'attachment_preview_evidence_replay',
  ].sort(),
  businessCapabilities: [
    ...ATTACHMENT_PREVIEW_FOUNDATION_INVENTORY.businessCapabilities,
    'attachment-preview-evidence-replay.command.v1',
  ].sort(),
};

const ATTACHMENT_TRANSLATION_CONTRACTS_INVENTORY = {
  ...ATTACHMENT_PREVIEW_RETAINED_EVIDENCE_REPLAY_INVENTORY,
  workflows: [
    ...ATTACHMENT_PREVIEW_RETAINED_EVIDENCE_REPLAY_INVENTORY.workflows,
    'attachment_translation',
  ].sort(),
};

const ATTACHMENT_TRANSLATION_AI_ENGINE_INVENTORY = {
  ...ATTACHMENT_TRANSLATION_CONTRACTS_INVENTORY,
  businessCapabilities: [
    ...ATTACHMENT_TRANSLATION_CONTRACTS_INVENTORY.businessCapabilities,
    'ai.attachment-translation.request.v1',
  ].sort(),
};

const ATTACHMENT_TRANSLATION_RUNTIME_ASSEMBLY_INVENTORY = {
  ...ATTACHMENT_TRANSLATION_AI_ENGINE_INVENTORY,
  businessCapabilities: [
    ...ATTACHMENT_TRANSLATION_AI_ENGINE_INVENTORY.businessCapabilities,
    'attachment.translation.v1',
    'attachment_translation.blob.v1',
    'attachment_translation.inference.v1',
    'attachment_translation.source_prepared.v1',
    'attachment_translation.source_rejected.v1',
    'attachment_translation.source_requested.v1',
    'attachment_translation.storage.v1',
  ].sort(),
};

const ATTACHMENT_TRANSLATION_SOURCE_PRODUCER_INVENTORY = {
  ...ATTACHMENT_TRANSLATION_RUNTIME_ASSEMBLY_INVENTORY,
  businessCapabilities: [
    ...ATTACHMENT_TRANSLATION_RUNTIME_ASSEMBLY_INVENTORY.businessCapabilities,
    'attachment_text_extraction.translation-source.v1',
  ].sort(),
};

const CONTACTS_MAIL_IDENTITY_COMMAND_CONTRACT_CORE_INVENTORY = {
  ...ATTACHMENT_TRANSLATION_SOURCE_PRODUCER_INVENTORY,
  domains: [
    ...ATTACHMENT_TRANSLATION_SOURCE_PRODUCER_INVENTORY.domains,
    'contacts',
  ].sort(),
  businessCapabilities: [
    ...ATTACHMENT_TRANSLATION_SOURCE_PRODUCER_INVENTORY.businessCapabilities,
    'contacts.mail-identity.command.v1',
  ].sort(),
};

const MAIL_CONTACTS_SYNC_CONTRACT_CORE_INVENTORY = {
  ...CONTACTS_MAIL_IDENTITY_COMMAND_CONTRACT_CORE_INVENTORY,
  workflows: [
    ...CONTACTS_MAIL_IDENTITY_COMMAND_CONTRACT_CORE_INVENTORY.workflows,
    'mail_contacts_sync',
  ].sort(),
  businessCapabilities: [
    ...CONTACTS_MAIL_IDENTITY_COMMAND_CONTRACT_CORE_INVENTORY.businessCapabilities,
    'mail.address-book.provider.v1',
    'mail.contacts-sync.v1',
  ].sort(),
};

const MAIL_CONTACTS_SYNC_PERSISTENCE_INVENTORY = {
  ...MAIL_CONTACTS_SYNC_CONTRACT_CORE_INVENTORY,
  businessCapabilities: [
    ...MAIL_CONTACTS_SYNC_CONTRACT_CORE_INVENTORY.businessCapabilities,
    'mail_contacts_sync.storage.v1',
  ].sort(),
};

const MAIL_CONTACTS_SYNC_RUNTIME_ADMISSION_INVENTORY = {
  ...MAIL_CONTACTS_SYNC_PERSISTENCE_INVENTORY,
  businessCapabilities: [
    ...MAIL_CONTACTS_SYNC_PERSISTENCE_INVENTORY.businessCapabilities,
    'mail_contacts_sync.contacts.command.v1',
    'mail_contacts_sync.contacts.changed.v1',
    'mail_contacts_sync.contacts.rejected.v1',
    'mail_contacts_sync.contacts.source-prepare.v1',
    'mail_contacts_sync.contacts.source-prepared.v1',
    'mail_contacts_sync.contacts.source-rejected.v1',
    'mail_contacts_sync.contacts.upserted.v1',
    'mail_contacts_sync.mail.entry-observed.v1',
    'mail_contacts_sync.mail.entry-upsert-rejected.v1',
    'mail_contacts_sync.mail.entry-upserted.v1',
    'mail_contacts_sync.mail.fetch-page.v1',
    'mail_contacts_sync.mail.page-completed.v1',
    'mail_contacts_sync.mail.page-rejected.v1',
    'mail_contacts_sync.mail.upsert-entry.v1',
    'mail_contacts_sync.scheduler.receipt.v1',
    'mail_contacts_sync.scheduler.v1',
    'contacts.mail-sync-source.blob-writer.v1',
    'contacts.mail-sync-source.changed.v1',
    'contacts.mail-sync-source.v1',
  ].sort(),
};

const MAIL_ADDRESS_BOOK_RUNTIME_EXECUTION_INVENTORY = {
  ...MAIL_CONTACTS_SYNC_RUNTIME_ADMISSION_INVENTORY,
  businessCapabilities: [
    ...MAIL_CONTACTS_SYNC_RUNTIME_ADMISSION_INVENTORY.businessCapabilities,
    'mail.address-book.contact-source.blob.v1',
  ].sort(),
};

const DESKTOP_CALL_RECORDING_CONTRACT_CORE_INVENTORY = {
  ...MAIL_ADDRESS_BOOK_RUNTIME_EXECUTION_INVENTORY,
  integrations: [
    ...MAIL_ADDRESS_BOOK_RUNTIME_EXECUTION_INVENTORY.integrations,
    'desktop_call_recording',
  ].sort(),
  workflows: [
    ...MAIL_ADDRESS_BOOK_RUNTIME_EXECUTION_INVENTORY.workflows,
    'call_transcription',
  ].sort(),
  engines: [
    ...MAIL_ADDRESS_BOOK_RUNTIME_EXECUTION_INVENTORY.engines,
    'speech_to_text',
  ].sort(),
};

const CALL_TRANSCRIPTION_CONTRACT_CORE_INVENTORY = {
  ...DESKTOP_CALL_RECORDING_CONTRACT_CORE_INVENTORY,
  businessCapabilities: [
    ...DESKTOP_CALL_RECORDING_CONTRACT_CORE_INVENTORY.businessCapabilities,
    'call_transcription.v1',
  ].sort(),
};

const CALL_TRANSCRIPTION_PERSISTENCE_INVENTORY = {
  ...CALL_TRANSCRIPTION_CONTRACT_CORE_INVENTORY,
  businessCapabilities: [
    ...CALL_TRANSCRIPTION_CONTRACT_CORE_INVENTORY.businessCapabilities,
    'call_transcription.storage.v1',
  ].sort(),
};

const CALL_TRANSCRIPTION_RUNTIME_INVENTORY = {
  ...CALL_TRANSCRIPTION_PERSISTENCE_INVENTORY,
  businessCapabilities: [
    ...CALL_TRANSCRIPTION_PERSISTENCE_INVENTORY.businessCapabilities,
    'call_transcription.blob.v1',
    'call_transcription.recording_ready.v1',
    'call_transcription.recording_rejected.v1',
    'call_transcription.stt.v1',
  ].sort(),
};

const MAIL_OUTBOUND_MIME_ATTACHMENTS_CARGO_FEATURE_ALLOWLIST = {
  'makosh-communication-cross-channel-forward-persistence': {
    default: [],
    'conformance-test-support': [],
  },
  'makosh-communication-delayed-delivery-persistence': {
    default: [],
    'conformance-test-support': [],
  },
  'makosh-communication-delivery-intent-persistence': {
    default: [],
    'conformance-test-support': [],
  },
  'makosh-reviewed-task-candidate-promotion-persistence': {
    default: [],
    'conformance-test-support': [],
  },
  'makosh-reviewed-note-candidate-promotion-persistence': {
    default: [],
    'conformance-test-support': [],
  },
  'makosh-mail-api': {
    default: [],
    'conformance-test-support': [],
  },
  'makosh-mail-imap': {
    default: [],
    'conformance-test-support': [],
  },
  'makosh-mail-gmail': {
    default: [],
    'conformance-test-support': ['makosh-mail-api/conformance-test-support'],
  },
  'makosh-mail-persistence': {
    default: [],
    'conformance-test-support': [],
  },
  'makosh-mail-runtime': {
    default: [],
    'conformance-test-support': [
      'makosh-mail-api/conformance-test-support',
      'makosh-mail-gmail/conformance-test-support',
      'makosh-mail-imap/conformance-test-support',
    ],
  },
};

const CONTACTS_MAIL_IDENTITY_COMMAND_PERSISTENCE_CARGO_FEATURE_ALLOWLIST = {
  ...MAIL_OUTBOUND_MIME_ATTACHMENTS_CARGO_FEATURE_ALLOWLIST,
  'makosh-contacts-persistence': {
    default: [],
    'conformance-test-support': [],
  },
};

const MAIL_CONTACTS_SYNC_PERSISTENCE_CARGO_FEATURE_ALLOWLIST = {
  ...CONTACTS_MAIL_IDENTITY_COMMAND_PERSISTENCE_CARGO_FEATURE_ALLOWLIST,
  'makosh-mail-contacts-sync-persistence': {
    default: [],
    'conformance-test-support': [],
  },
};

const MAIL_ADDRESS_BOOK_PROVIDER_ADAPTERS_CARGO_FEATURE_ALLOWLIST = {
  ...MAIL_CONTACTS_SYNC_PERSISTENCE_CARGO_FEATURE_ALLOWLIST,
  'makosh-mail-google-people': {
    default: [],
    'conformance-test-support': [],
  },
  'makosh-mail-carddav': {
    default: [],
    'conformance-test-support': [],
  },
};

const MAIL_ADDRESS_BOOK_PERSISTENCE_AUTHORITY_CARGO_FEATURE_ALLOWLIST = {
  ...MAIL_ADDRESS_BOOK_PROVIDER_ADAPTERS_CARGO_FEATURE_ALLOWLIST,
  'makosh-mail-address-book-persistence': {
    default: [],
    'conformance-test-support': [],
  },
};

const MAIL_ADDRESS_BOOK_RUNTIME_EXECUTION_CARGO_FEATURE_ALLOWLIST = {
  ...MAIL_ADDRESS_BOOK_PERSISTENCE_AUTHORITY_CARGO_FEATURE_ALLOWLIST,
  'makosh-mail-runtime': {
    default: [],
    'conformance-test-support': [
      'makosh-mail-api/conformance-test-support',
      'makosh-mail-carddav/conformance-test-support',
      'makosh-mail-gmail/conformance-test-support',
      'makosh-mail-google-people/conformance-test-support',
      'makosh-mail-imap/conformance-test-support',
    ],
  },
};

const PERSONS_PERSISTENCE_CARGO_FEATURE_ALLOWLIST = {
  ...MAIL_ADDRESS_BOOK_RUNTIME_EXECUTION_CARGO_FEATURE_ALLOWLIST,
  'makosh-persons-persistence': {
    default: [],
    'conformance-test-support': [],
  },
};

const MAIL_PERSONS_SYNC_PERSISTENCE_CARGO_FEATURE_ALLOWLIST = {
  ...PERSONS_PERSISTENCE_CARGO_FEATURE_ALLOWLIST,
  'makosh-mail-persons-sync-persistence': {
    default: [],
    'conformance-test-support': [],
  },
  'makosh-review-person-match-candidate-persistence': {
    default: [],
    'conformance-test-support': [],
  },
  'makosh-reviewed-person-match-candidate-promotion-persistence': {
    default: [],
    'conformance-test-support': [],
  },
};

const PERSONS_ADMISSION_RETIRED_PACKAGE_NAMES = new Set([
  'makosh-contacts-command-api',
  'makosh-contacts-mail-sync-source-api',
  'makosh-contacts-core',
  'makosh-contacts-persistence',
  'makosh-contacts-runtime',
  'makosh-contacts-assembly',
  'makosh-mail-contacts-sync-api',
  'makosh-mail-contacts-sync-core',
  'makosh-mail-contacts-sync-persistence',
  'makosh-mail-contacts-sync-runtime',
  'makosh-mail-contacts-sync-assembly',
]);

const PERSONS_ADMISSION_PRODUCTION_PACKAGES =
  MAIL_PERSONS_SYNC_CONTRACT_CORE_PRODUCTION_PACKAGES.filter(
    ({ name }) => !PERSONS_ADMISSION_RETIRED_PACKAGE_NAMES.has(name),
  );

const PERSONS_ADMISSION_WORKSPACE_DEPENDENCY_ALLOWLIST = Object.fromEntries(
  Object.entries(MAIL_PERSONS_SYNC_CONTRACT_CORE_WORKSPACE_DEPENDENCY_ALLOWLIST)
    .filter(([name]) => !PERSONS_ADMISSION_RETIRED_PACKAGE_NAMES.has(name))
    .map(([name, dependencies]) => [
      name,
      dependencies.filter(({ name: dependency }) => (
        !PERSONS_ADMISSION_RETIRED_PACKAGE_NAMES.has(dependency)
      )),
    ]),
);

const PERSONS_ADMISSION_THIRD_PARTY_DEPENDENCY_ALLOWLIST = Object.fromEntries(
  Object.entries(MAIL_PERSONS_SYNC_CONTRACT_CORE_THIRD_PARTY_DEPENDENCY_ALLOWLIST)
    .filter(([name]) => !PERSONS_ADMISSION_RETIRED_PACKAGE_NAMES.has(name)),
);

const PERSONS_ADMISSION_CARGO_FEATURE_ALLOWLIST = Object.fromEntries(
  Object.entries(MAIL_PERSONS_SYNC_PERSISTENCE_CARGO_FEATURE_ALLOWLIST)
    .filter(([name]) => !PERSONS_ADMISSION_RETIRED_PACKAGE_NAMES.has(name)),
);

const PERSONS_ADMISSION_RETIRED_CAPABILITIES = [
  'contacts.mail-identity.command.v1',
  'contacts.mail-sync-source.blob-writer.v1',
  'contacts.mail-sync-source.changed.v1',
  'contacts.mail-sync-source.v1',
  'mail.address-book.contact-source.blob.v1',
  'mail.address-book.provider.v1',
  'mail.contacts-sync.v1',
  'mail_contacts_sync.contacts.changed.v1',
  'mail_contacts_sync.contacts.command.v1',
  'mail_contacts_sync.contacts.rejected.v1',
  'mail_contacts_sync.contacts.source-prepare.v1',
  'mail_contacts_sync.contacts.source-prepared.v1',
  'mail_contacts_sync.contacts.source-rejected.v1',
  'mail_contacts_sync.contacts.upserted.v1',
  'mail_contacts_sync.mail.entry-observed.v1',
  'mail_contacts_sync.mail.entry-upsert-rejected.v1',
  'mail_contacts_sync.mail.entry-upserted.v1',
  'mail_contacts_sync.mail.fetch-page.v1',
  'mail_contacts_sync.mail.page-completed.v1',
  'mail_contacts_sync.mail.page-rejected.v1',
  'mail_contacts_sync.mail.upsert-entry.v1',
  'mail_contacts_sync.scheduler.receipt.v1',
  'mail_contacts_sync.scheduler.v1',
  'mail_contacts_sync.storage.v1',
];

const PERSONS_ADMISSION_CAPABILITIES = [
  'mail.person-source.provider.v1',
  'mail_persons_sync.mail.account-ready.v1',
  'mail_persons_sync.mail.account-retired.v1',
  'mail_persons_sync.mail.fetch-page.v1',
  'mail_persons_sync.mail.page-completed.v1',
  'mail_persons_sync.mail.page-rejected.v1',
  'mail_persons_sync.mail.source-observed.v1',
  'mail_persons_sync.mail.source-removed.v1',
  'mail_persons_sync.mail.source-updated.v1',
  'mail_persons_sync.page-receipt.v1',
  'mail_persons_sync.persons.command-rejected.v1',
  'mail_persons_sync.persons.command-succeeded.v1',
  'mail_persons_sync.persons.command.v1',
  'mail_persons_sync.run-result.v1',
  'mail_persons_sync.scheduler.receipt.v1',
  'mail_persons_sync.scheduler.v1',
  'mail_persons_sync.scheduler_schedule_command.v1',
  'mail_persons_sync.scheduler_schedule_result.v1',
  'mail_persons_sync.storage.v1',
  'persons.client.v1',
  'persons.command-rejected.v1',
  'persons.command-succeeded.v1',
  'persons.command.v1',
  'persons.owner-event.v1',
  'persons.review-candidate.v1',
  'persons.storage.v1',
  'review.person-match-candidate.approved.publisher.v1',
  'review.person-match-candidate.client.v1',
  'review.person-match-candidate.decision.consumer.v1',
  'review.person-match-candidate.persons-candidate.consumer.v1',
  'review.person-match-candidate.promotion-result.consumer.v1',
  'review.person-match-candidate.storage.v1',
  'review.person-match-candidate.submission-rejected.publisher.v1',
  'review.person-match-candidate.submitted.publisher.v1',
  'reviewed-person-match-candidate-promotion.approval.consumer.v1',
  'reviewed-person-match-candidate-promotion.persons-command.publisher.v1',
  'reviewed-person-match-candidate-promotion.persons-rejected.consumer.v1',
  'reviewed-person-match-candidate-promotion.persons-succeeded.consumer.v1',
  'reviewed-person-match-candidate-promotion.result.publisher.v1',
  'reviewed-person-match-candidate-promotion.storage.v1',
];

const PERSONS_ADMISSION_INVENTORY = {
  ...CALL_TRANSCRIPTION_RUNTIME_INVENTORY,
  domains: [
    ...CALL_TRANSCRIPTION_RUNTIME_INVENTORY.domains.filter((owner) => owner !== 'contacts'),
    'persons',
  ].sort(),
  workflows: [
    ...CALL_TRANSCRIPTION_RUNTIME_INVENTORY.workflows.filter(
      (owner) => owner !== 'mail_contacts_sync',
    ),
    'mail_persons_sync',
    'reviewed_person_match_candidate_promotion',
  ].sort(),
  businessCapabilities: [
    ...CALL_TRANSCRIPTION_RUNTIME_INVENTORY.businessCapabilities.filter(
      (capability) => !PERSONS_ADMISSION_RETIRED_CAPABILITIES.includes(capability),
    ),
    ...PERSONS_ADMISSION_CAPABILITIES,
  ].sort(),
};

const COMMUNICATION_BULK_DELAYED_DELIVERY_ADMISSION_INVENTORY = {
  ...PERSONS_ADMISSION_INVENTORY,
  workflows: [
    ...PERSONS_ADMISSION_INVENTORY.workflows,
    'communication_bulk_action',
    'communication_delayed_delivery',
  ].sort(),
  businessCapabilities: [
    ...PERSONS_ADMISSION_INVENTORY.businessCapabilities,
    'communication.bulk_action.v1',
    'communication.delayed_delivery.blob.v1',
    'communication.delayed_delivery.clock.v1',
    'communication.delayed_delivery.delivery_intent.v1',
    'communication.delayed_delivery.scheduler_due.v1',
    'communication.delayed_delivery.scheduler_receipt.v1',
    'communication.delayed_delivery.scheduler_schedule_command.v1',
    'communication.delayed_delivery.scheduler_schedule_result.v1',
    'communication.delayed_delivery.storage.v1',
    'communication.delayed_delivery.v1',
    'communication_bulk_action.delivery_intent.v1',
    'communication_bulk_action.storage.v1',
  ].sort(),
};

const AI_INFERENCE_OLLAMA_ADMISSION_PRODUCTION_PACKAGES =
  PERSONS_ADMISSION_PRODUCTION_PACKAGES.flatMap((descriptor) => (
    descriptor.name === 'makosh-ai-inference-persistence'
      ? [
        descriptor,
        { name: 'makosh-ai-inference-runtime', role: 'engine', owner: 'ai', surface: 'runtime' },
        { name: 'makosh-ai-inference-assembly', role: 'engine', owner: 'ai', surface: 'assembly' },
      ]
      : [descriptor]
  ));

const AI_INFERENCE_OLLAMA_ADMISSION_WORKSPACE_DEPENDENCY_ALLOWLIST = {
  ...PERSONS_ADMISSION_WORKSPACE_DEPENDENCY_ALLOWLIST,
  'makosh-ai-inference-runtime': [
    { name: 'makosh-ai-contracts', kind: 'normal' },
    { name: 'makosh-ai-inference-core', kind: 'normal' },
    { name: 'makosh-ai-inference-persistence', kind: 'normal' },
    { name: 'makosh-blob-client', kind: 'normal' },
    { name: 'makosh-runtime-protocol', kind: 'normal' },
    { name: 'makosh-storage-protocol', kind: 'normal' },
    { name: 'makosh-storage-vault', kind: 'normal' },
  ],
  'makosh-ai-inference-assembly': [
    { name: 'makosh-ai-inference-persistence', kind: 'normal' },
    { name: 'makosh-ai-inference-runtime', kind: 'normal' },
    { name: 'makosh-runtime-protocol', kind: 'normal' },
    { name: 'makosh-storage-protocol', kind: 'normal' },
  ],
};

const AI_INFERENCE_OLLAMA_ADMISSION_THIRD_PARTY_DEPENDENCY_ALLOWLIST = {
  ...PERSONS_ADMISSION_THIRD_PARTY_DEPENDENCY_ALLOWLIST,
  'makosh-ai-inference-runtime': [
    { name: 'libc', kind: 'normal', source: 'crates_io', version: '=0.2.186', defaultFeatures: true, features: [] },
    { name: 'prost', kind: 'normal', source: 'crates_io', version: '=0.14.4', defaultFeatures: true, features: [] },
    { name: 'sha2', kind: 'normal', source: 'crates_io', version: '=0.11.0', defaultFeatures: false, features: [] },
    { name: 'tokio', kind: 'normal', source: 'crates_io', version: '=1.52.4', defaultFeatures: false, features: ['rt-multi-thread', 'time'] },
    { name: 'zeroize', kind: 'normal', source: 'crates_io', version: '=1.9.0', defaultFeatures: true, features: [] },
  ],
  'makosh-ai-inference-assembly': [
    { name: 'prost', kind: 'normal', source: 'crates_io', version: '=0.14.4', defaultFeatures: true, features: [] },
    { name: 'serde', kind: 'normal', source: 'crates_io', version: '=1.0.228', defaultFeatures: false, features: ['derive', 'std'] },
    { name: 'serde_json', kind: 'normal', source: 'crates_io', version: '=1.0.150', defaultFeatures: true, features: [] },
  ],
};

const AI_INFERENCE_OLLAMA_ADMISSION_INVENTORY = {
  ...COMMUNICATION_BULK_DELAYED_DELIVERY_ADMISSION_INVENTORY,
  integrations: [
    ...COMMUNICATION_BULK_DELAYED_DELIVERY_ADMISSION_INVENTORY.integrations,
    'ollama',
  ].sort(),
  businessCapabilities: [
    ...COMMUNICATION_BULK_DELAYED_DELIVERY_ADMISSION_INVENTORY.businessCapabilities,
    'ai.inference.blob.v1',
    'ai.inference.request.v1',
    'ai.inference.storage.v1',
    'ai.provider.generate.v1',
    'ai.provider.summarize.v1',
    'ai.summary.request.v1',
    'ollama.ai.storage.v1',
  ].sort(),
};

const SPEECH_TO_TEXT_WHISPER_ADMISSION_PRODUCTION_PACKAGES =
  AI_INFERENCE_OLLAMA_ADMISSION_PRODUCTION_PACKAGES.flatMap((descriptor) => (
    descriptor.name === 'makosh-speech-to-text-persistence'
      ? [
        descriptor,
        { name: 'makosh-speech-to-text-runtime', role: 'engine', owner: 'speech_to_text', surface: 'runtime' },
        { name: 'makosh-speech-to-text-assembly', role: 'engine', owner: 'speech_to_text', surface: 'assembly' },
        { name: 'makosh-speech-transcript-artifact', role: 'engine', owner: 'speech_to_text', surface: 'contract' },
        { name: 'makosh-whisper-stt-core', role: 'integration', owner: 'whisper_stt', surface: 'implementation' },
        { name: 'makosh-whisper-stt-assembly', role: 'integration', owner: 'whisper_stt', surface: 'assembly' },
        { name: 'makosh-whisper-stt-persistence', role: 'integration', owner: 'whisper_stt', surface: 'persistence' },
        { name: 'makosh-whisper-stt-process', role: 'integration', owner: 'whisper_stt', surface: 'implementation' },
        { name: 'makosh-whisper-stt-runtime', role: 'integration', owner: 'whisper_stt', surface: 'runtime' },
      ]
      : [descriptor]
  ));

const SPEECH_TO_TEXT_WHISPER_ADMISSION_WORKSPACE_DEPENDENCY_ALLOWLIST = {
  ...AI_INFERENCE_OLLAMA_ADMISSION_WORKSPACE_DEPENDENCY_ALLOWLIST,
  'makosh-speech-to-text-runtime': [
    { name: 'makosh-blob-client', kind: 'normal' },
    { name: 'makosh-runtime-protocol', kind: 'normal' },
    { name: 'makosh-speech-to-text-api', kind: 'normal' },
    { name: 'makosh-speech-to-text-core', kind: 'normal' },
    { name: 'makosh-speech-to-text-persistence', kind: 'normal' },
    { name: 'makosh-storage-protocol', kind: 'normal' },
    { name: 'makosh-storage-vault', kind: 'normal' },
  ],
  'makosh-speech-to-text-assembly': [
    { name: 'makosh-runtime-protocol', kind: 'normal' },
    { name: 'makosh-speech-to-text-persistence', kind: 'normal' },
    { name: 'makosh-speech-to-text-runtime', kind: 'normal' },
    { name: 'makosh-storage-protocol', kind: 'normal' },
  ],
  'makosh-speech-transcript-artifact': [],
  'makosh-whisper-stt-core': [
    { name: 'makosh-speech-to-text-api', kind: 'normal' },
    { name: 'makosh-speech-transcript-artifact', kind: 'normal' },
  ],
  'makosh-whisper-stt-assembly': [
    { name: 'makosh-runtime-protocol', kind: 'normal' },
    { name: 'makosh-storage-protocol', kind: 'normal' },
    { name: 'makosh-whisper-stt-persistence', kind: 'normal' },
    { name: 'makosh-whisper-stt-runtime', kind: 'normal' },
  ],
  'makosh-whisper-stt-persistence': [
    { name: 'makosh-storage-protocol', kind: 'normal' },
  ],
  'makosh-whisper-stt-process': [
    { name: 'makosh-speech-to-text-api', kind: 'normal' },
    { name: 'makosh-whisper-stt-core', kind: 'normal' },
  ],
  'makosh-whisper-stt-runtime': [
    { name: 'makosh-blob-client', kind: 'normal' },
    { name: 'makosh-runtime-protocol', kind: 'normal' },
    { name: 'makosh-speech-to-text-api', kind: 'normal' },
    { name: 'makosh-storage-protocol', kind: 'normal' },
    { name: 'makosh-storage-vault', kind: 'normal' },
    { name: 'makosh-whisper-stt-core', kind: 'normal' },
    { name: 'makosh-whisper-stt-persistence', kind: 'normal' },
    { name: 'makosh-whisper-stt-process', kind: 'normal' },
  ],
};

const SPEECH_TO_TEXT_WHISPER_ADMISSION_THIRD_PARTY_DEPENDENCY_ALLOWLIST = {
  ...AI_INFERENCE_OLLAMA_ADMISSION_THIRD_PARTY_DEPENDENCY_ALLOWLIST,
  'makosh-speech-to-text-runtime': [
    { name: 'libc', kind: 'normal', source: 'crates_io', version: '=0.2.186', defaultFeatures: true, features: [] },
    { name: 'prost', kind: 'normal', source: 'crates_io', version: '=0.14.4', defaultFeatures: true, features: [] },
    { name: 'sha2', kind: 'normal', source: 'crates_io', version: '=0.11.0', defaultFeatures: false, features: [] },
    { name: 'tokio', kind: 'normal', source: 'crates_io', version: '=1.52.4', defaultFeatures: false, features: ['rt-multi-thread', 'time'] },
    { name: 'zeroize', kind: 'normal', source: 'crates_io', version: '=1.9.0', defaultFeatures: true, features: [] },
  ],
  'makosh-speech-to-text-assembly': [
    { name: 'prost', kind: 'normal', source: 'crates_io', version: '=0.14.4', defaultFeatures: true, features: [] },
    { name: 'serde', kind: 'normal', source: 'crates_io', version: '=1.0.228', defaultFeatures: false, features: ['derive', 'std'] },
    { name: 'serde_json', kind: 'normal', source: 'crates_io', version: '=1.0.150', defaultFeatures: true, features: [] },
  ],
  'makosh-speech-transcript-artifact': [
    { name: 'prost', kind: 'normal', source: 'crates_io', version: '=0.14.4', defaultFeatures: true, features: [] },
    { name: 'sha2', kind: 'normal', source: 'crates_io', version: '=0.11.0', defaultFeatures: false, features: [] },
    { name: 'prost-build', kind: 'build', source: 'crates_io', version: '=0.14.4', defaultFeatures: true, features: [] },
    { name: 'protoc-bin-vendored', kind: 'build', source: 'crates_io', version: '=3.2.0', defaultFeatures: true, features: [] },
    { name: 'sha2', kind: 'build', source: 'crates_io', version: '=0.11.0', defaultFeatures: false, features: [] },
  ],
  'makosh-whisper-stt-core': [],
  'makosh-whisper-stt-assembly': [
    { name: 'prost', kind: 'normal', source: 'crates_io', version: '=0.14.4', defaultFeatures: true, features: [] },
    { name: 'serde', kind: 'normal', source: 'crates_io', version: '=1.0.228', defaultFeatures: false, features: ['derive', 'std'] },
    { name: 'serde_json', kind: 'normal', source: 'crates_io', version: '=1.0.150', defaultFeatures: true, features: [] },
  ],
  'makosh-whisper-stt-persistence': [
    { name: 'sha2', kind: 'normal', source: 'crates_io', version: '=0.11.0', defaultFeatures: false, features: [] },
    { name: 'sqlx', kind: 'normal', source: 'crates_io', version: '=0.9.0', defaultFeatures: false, features: ['postgres', 'runtime-tokio', 'tls-rustls-ring'] },
  ],
  'makosh-whisper-stt-process': [
    { name: 'serde', kind: 'normal', source: 'crates_io', version: '=1.0.228', defaultFeatures: false, features: ['derive', 'std'] },
    { name: 'serde_json', kind: 'normal', source: 'crates_io', version: '=1.0.150', defaultFeatures: true, features: [] },
  ],
  'makosh-whisper-stt-runtime': [
    { name: 'libc', kind: 'normal', source: 'crates_io', version: '=0.2.186', defaultFeatures: true, features: [] },
    { name: 'prost', kind: 'normal', source: 'crates_io', version: '=0.14.4', defaultFeatures: true, features: [] },
    { name: 'sha2', kind: 'normal', source: 'crates_io', version: '=0.11.0', defaultFeatures: false, features: [] },
    { name: 'tokio', kind: 'normal', source: 'crates_io', version: '=1.52.4', defaultFeatures: false, features: ['rt-multi-thread', 'time'] },
    { name: 'zeroize', kind: 'normal', source: 'crates_io', version: '=1.9.0', defaultFeatures: true, features: [] },
  ],
};

const SPEECH_TO_TEXT_WHISPER_ADMISSION_INVENTORY = {
  ...AI_INFERENCE_OLLAMA_ADMISSION_INVENTORY,
  integrations: [
    ...AI_INFERENCE_OLLAMA_ADMISSION_INVENTORY.integrations,
    'whisper_stt',
  ].sort(),
  businessCapabilities: [
    ...AI_INFERENCE_OLLAMA_ADMISSION_INVENTORY.businessCapabilities,
    'speech_to_text.blob.v1',
    'speech_to_text.provider.v1',
    'speech_to_text.storage.v1',
    'speech_to_text.transcribe.v1',
    'whisper_stt.blob.v1',
    'whisper_stt.native.v1',
    'whisper_stt.provider.v1',
    'whisper_stt.storage.v1',
  ].sort(),
};

const CLOCK_KEYS = ['wallTime', 'elapsedTime', 'testTime', 'moduleCapabilityEnabled'];

const EXIT_GATES = [
  'boots_without_external_services',
  'foundation_protocol_v1_conformance',
  'private_control_store_create_open_validate',
  'missing_or_invalid_store_recovery_only',
  'local_ipc_status_validate_export_shutdown',
  'pristine_inherited_fd_owner_enrollment',
  'server_bootstrap_pairing_tls_conformance',
  'file_release_authority_conformance',
  'managed_launch_toctou_conformance',
  'online_mutations_fail_closed',
  'exclusive_data_directory_lock',
  'bounded_shutdown',
  'wall_monotonic_fake_clock_conformance',
  'diagnostics_exclude_secrets_private_content',
];

const DEVELOPMENT_PROFILE_KEYS = [
  'id',
  'purpose',
  'workspaceRoot',
  'packages',
  'selection',
  'deviceProof',
  'privateKeyStorage',
  'persistentSecretsAllowed',
  'productDataAllowed',
  'networkListenerEnabled',
  'remotePairingEnabled',
  'externalServicesEnabled',
  'vaultEnabled',
  'releaseArtifactAllowed',
  'productionGateEvidenceAllowed',
  'visibleInsecureWarningRequired',
  'automaticProductionFallbackAllowed',
  'simulatedTargets',
];

const DEVELOPMENT_PACKAGE_KEYS = ['package', 'surface'];

function hasExactKeys(value, expectedKeys) {
  if (!value || typeof value !== 'object' || Array.isArray(value)) return false;
  const keys = Object.keys(value);
  return keys.length === expectedKeys.length
    && keys.every((key) => expectedKeys.includes(key));
}

function isExactOrderedStringList(value, expected) {
  return Array.isArray(expected)
    && Array.isArray(value)
    && value.length === expected.length
    && duplicates(value).length === 0
    && value.every((entry, index) => entry === expected[index]);
}

function isExactPackageInventory(packages, expectedPackages) {
  return Array.isArray(expectedPackages)
    && Array.isArray(packages)
    && packages.length === expectedPackages.length
    && packages.every((entry, index) => {
      const expected = expectedPackages[index];
      return hasExactKeys(entry, ['name', 'role', 'owner', 'surface'])
        && entry.name === expected.name
        && entry.role === expected.role
        && entry.owner === expected.owner
        && entry.surface === expected.surface;
    });
}

function isEmptyOwnerInventory(inventory) {
  const ownerClasses = [
    'domains',
    'integrations',
    'workflows',
    'engines',
    'businessCapabilities',
  ];
  return hasExactKeys(inventory, ownerClasses)
    && ownerClasses
      .every((ownerClass) => Array.isArray(inventory[ownerClass]) && inventory[ownerClass].length === 0);
}

function isExactOwnerInventory(inventory, expected) {
  const ownerClasses = [
    'domains',
    'integrations',
    'workflows',
    'engines',
    'businessCapabilities',
  ];
  return hasExactKeys(inventory, ownerClasses)
    && hasExactKeys(expected, ownerClasses)
    && ownerClasses.every((ownerClass) => (
      isExactOrderedStringList(inventory[ownerClass], expected[ownerClass])
    ));
}

function isExactWorkspaceDependencyAllowlist(allowlist, expectedPackages, expectedAllowlist) {
  if (!Array.isArray(expectedPackages) || !expectedAllowlist) return false;
  const packageNames = expectedPackages.map(({ name }) => name);
  return hasExactKeys(allowlist, packageNames)
    && packageNames.every((packageName) => isExactDependencyList(
      allowlist[packageName],
      expectedAllowlist[packageName],
    ));
}

function isExactDependencyList(actual, expected) {
  return Array.isArray(expected)
    && Array.isArray(actual)
    && actual.length === expected.length
    && actual.every((entry, index) => {
      const expectedEntry = expected[index];
      return hasExactKeys(entry, Object.keys(expectedEntry))
        && Object.entries(expectedEntry).every(([key, value]) => (
          Array.isArray(value)
            ? isExactOrderedStringList(entry[key], value)
            : entry[key] === value
        ));
    });
}

function isExactThirdPartyDependencyAllowlist(allowlist, expectedPackages, expectedAllowlist) {
  if (!Array.isArray(expectedPackages) || !expectedAllowlist) return false;
  const packageNames = expectedPackages.map(({ name }) => name);
  return hasExactKeys(allowlist, packageNames)
    && packageNames.every((packageName) => isExactDependencyList(
      allowlist[packageName],
      expectedAllowlist[packageName],
    ));
}

function isExactTargetPolicy(targetPolicy, expectedPackages) {
  if (!Array.isArray(expectedPackages)) return false;
  const packageNames = expectedPackages.map(({ name }) => name);
  if (!hasExactKeys(targetPolicy, packageNames)) return false;
  return packageNames.every((packageName) => {
    const target = targetPolicy[packageName];
    const packageDescriptor = expectedPackages.find(({ name }) => name === packageName);
    const protocolPackage = [
      'makosh-events-protocol',
      'makosh-retained-evidence-replay-protocol',
      'makosh-attachment-preview-evidence-replay-api',
      'makosh-communications-retained-evidence-replay-contract',
      'makosh-mail-retained-evidence-replay-contract',
      'makosh-runtime-protocol',
      'makosh-gateway-protocol',
      'makosh-storage-protocol',
      'makosh-scheduler-protocol',
      'makosh-whatsapp-api',
      'makosh-telegram-api',
      'makosh-zulip-api',
      'makosh-mail-api',
      'makosh-communications-ingress',
      'makosh-communications-call-evidence-ingress',
      'makosh-communications-call-evidence-api',
      'makosh-communications-attachment-contract',
      'makosh-communications-api',
      'makosh-communications-content-api',
      'makosh-communications-saved-query-api',
      'makosh-communications-sender-insights-api',
      'makosh-communications-evidence-export-source-api',
      'makosh-communications-cross-channel-forward-source-api',
      'makosh-communications-ai-source-api',
      'makosh-communication-reply-suggestion-api',
      'makosh-communication-summary-api',
      'makosh-communication-translation-api',
      'makosh-communication-explanation-api',
      'makosh-communication-recipient-suggestion-api',
      'makosh-communications-recipient-source-api',
      'makosh-communication-task-candidate-api',
      'makosh-communications-task-source-api',
      'makosh-communication-note-candidate-api',
      'makosh-communications-note-source-api',
      'makosh-knowledge-command-api',
      'makosh-ai-contracts',
      'makosh-attachment-archive-inspection-api',
      'makosh-attachment-archive-inspection-ingress',
      'makosh-attachment-text-extraction-api',
      'makosh-attachment-text-extraction-ingress',
      'makosh-attachment-preview-api',
      'makosh-attachment-preview-ingress',
      'makosh-attachment-translation-api',
      'makosh-attachment-translation-ingress',
      'makosh-communications-export-api',
      'makosh-communication-delivery-intent-api',
      'makosh-communication-delivery-intent-ingress-api',
      'makosh-communication-bulk-action-api',
      'makosh-communication-delayed-delivery-api',
      'makosh-communication-cross-channel-forward-api',
      'makosh-review-attention-api',
      'makosh-review-note-candidate-api',
      'makosh-review-note-candidate-promotion-api',
      'makosh-review-task-candidate-api',
      'makosh-review-task-candidate-promotion-api',
      'makosh-tasks-command-api',
      'makosh-contacts-command-api',
      'makosh-contacts-mail-sync-source-api',
      'makosh-mail-address-book-contract',
      'makosh-mail-contacts-sync-api',
      'makosh-speech-to-text-api',
      'makosh-speech-transcript-artifact',
      'makosh-desktop-call-recording-api',
      'makosh-call-transcription-ingress',
      'makosh-call-transcription-api',
      'makosh-persons-api',
      'makosh-mail-persons-sync-api',
      'makosh-review-person-match-candidate-api',
      'makosh-review-person-match-candidate-promotion-api',
      'makosh-mail-delivery-intent-contract',
      'makosh-telegram-delivery-intent-contract',
      'makosh-whatsapp-delivery-intent-contract',
      'makosh-zulip-delivery-intent-contract',
      'makosh-attachment-security-contract',
    ].includes(packageName);
    return hasExactKeys(target, ['primaryKind', 'customBuildAllowed'])
      && target.primaryKind === (
        ['runtime', 'assembly'].includes(packageDescriptor?.surface) ? 'bin' : 'lib'
      )
      && target.customBuildAllowed === protocolPackage;
  });
}

function isExactCargoFeatureAllowlist(actual, expected) {
  if (!hasExactKeys(actual, Object.keys(expected))) return false;
  return Object.entries(expected).every(([packageName, expectedFeatures]) => {
    const actualFeatures = actual[packageName];
    if (!hasExactKeys(actualFeatures, Object.keys(expectedFeatures))) return false;
    return Object.entries(expectedFeatures).every(([featureName, featureMembers]) => (
      isExactOrderedStringList(actualFeatures[featureName], featureMembers)
    ));
  });
}

function expectedSlice(currentSlice) {
  if (currentSlice === 'kernel_recovery_only_v1') {
    return {
      profile: KERNEL_PROFILE,
      packages: RECOVERY_PRODUCTION_PACKAGES,
      workspaceDependencies: RECOVERY_WORKSPACE_DEPENDENCY_ALLOWLIST,
      thirdPartyDependencies: RECOVERY_THIRD_PARTY_DEPENDENCY_ALLOWLIST,
      forbiddenDependencyPrefixes: RECOVERY_FORBIDDEN_DEPENDENCY_PREFIXES,
    };
  }
  if (currentSlice === 'module_control_plane_v1') {
    return {
      profile: MODULE_CONTROL_PROFILE,
      packages: RECOVERY_PRODUCTION_PACKAGES,
      workspaceDependencies: RECOVERY_WORKSPACE_DEPENDENCY_ALLOWLIST,
      thirdPartyDependencies: RECOVERY_THIRD_PARTY_DEPENDENCY_ALLOWLIST,
      forbiddenDependencyPrefixes: RECOVERY_FORBIDDEN_DEPENDENCY_PREFIXES,
    };
  }
  if (currentSlice === 'server_bootstrap_pairing_v1') {
    return {
      profile: SERVER_BOOTSTRAP_PAIRING_PROFILE,
      packages: RECOVERY_PRODUCTION_PACKAGES,
      workspaceDependencies: RECOVERY_WORKSPACE_DEPENDENCY_ALLOWLIST,
      thirdPartyDependencies: RECOVERY_THIRD_PARTY_DEPENDENCY_ALLOWLIST,
      forbiddenDependencyPrefixes: RECOVERY_FORBIDDEN_DEPENDENCY_PREFIXES,
    };
  }
  if (currentSlice === 'managed_launch_trust_v1') {
    return {
      profile: MANAGED_LAUNCH_TRUST_PROFILE,
      packages: RECOVERY_PRODUCTION_PACKAGES,
      workspaceDependencies: RECOVERY_WORKSPACE_DEPENDENCY_ALLOWLIST,
      thirdPartyDependencies: RECOVERY_THIRD_PARTY_DEPENDENCY_ALLOWLIST,
      forbiddenDependencyPrefixes: RECOVERY_FORBIDDEN_DEPENDENCY_PREFIXES,
    };
  }
  if (currentSlice === 'vault_foundation_v1' || currentSlice === 'vault_v1') {
    return {
      profile: MANAGED_LAUNCH_TRUST_PROFILE,
      packages: VAULT_FOUNDATION_PRODUCTION_PACKAGES,
      workspaceDependencies: VAULT_FOUNDATION_WORKSPACE_DEPENDENCY_ALLOWLIST,
      thirdPartyDependencies: VAULT_FOUNDATION_THIRD_PARTY_DEPENDENCY_ALLOWLIST,
      forbiddenDependencyPrefixes: VAULT_FOUNDATION_FORBIDDEN_DEPENDENCY_PREFIXES,
    };
  }
  if (currentSlice === 'clock_v1') {
    return {
      profile: MANAGED_LAUNCH_TRUST_PROFILE,
      packages: CLOCK_PRODUCTION_PACKAGES,
      workspaceDependencies: CLOCK_WORKSPACE_DEPENDENCY_ALLOWLIST,
      thirdPartyDependencies: CLOCK_THIRD_PARTY_DEPENDENCY_ALLOWLIST,
      forbiddenDependencyPrefixes: VAULT_FOUNDATION_FORBIDDEN_DEPENDENCY_PREFIXES,
    };
  }
  if (currentSlice === 'telemetry_foundation_v1') {
    return {
      profile: MANAGED_LAUNCH_TRUST_PROFILE,
      packages: TELEMETRY_FOUNDATION_PRODUCTION_PACKAGES,
      workspaceDependencies: TELEMETRY_FOUNDATION_WORKSPACE_DEPENDENCY_ALLOWLIST,
      thirdPartyDependencies: TELEMETRY_FOUNDATION_THIRD_PARTY_DEPENDENCY_ALLOWLIST,
      forbiddenDependencyPrefixes: VAULT_FOUNDATION_FORBIDDEN_DEPENDENCY_PREFIXES,
    };
  }
  if (currentSlice === 'storage_foundation_v1') {
    return { profile: MANAGED_LAUNCH_TRUST_PROFILE, packages: STORAGE_FOUNDATION_PRODUCTION_PACKAGES, workspaceDependencies: STORAGE_FOUNDATION_WORKSPACE_DEPENDENCY_ALLOWLIST, thirdPartyDependencies: STORAGE_FOUNDATION_THIRD_PARTY_DEPENDENCY_ALLOWLIST, forbiddenDependencyPrefixes: STORAGE_FOUNDATION_FORBIDDEN_DEPENDENCY_PREFIXES };
  }
  if (currentSlice === 'nats_foundation_v1') {
    return { profile: MANAGED_LAUNCH_TRUST_PROFILE, packages: NATS_FOUNDATION_PRODUCTION_PACKAGES, workspaceDependencies: NATS_FOUNDATION_WORKSPACE_DEPENDENCY_ALLOWLIST, thirdPartyDependencies: NATS_FOUNDATION_THIRD_PARTY_DEPENDENCY_ALLOWLIST, forbiddenDependencyPrefixes: STORAGE_FOUNDATION_FORBIDDEN_DEPENDENCY_PREFIXES };
  }
  if (currentSlice === 'blob_foundation_v1') {
    return { profile: MANAGED_LAUNCH_TRUST_PROFILE, packages: BLOB_FOUNDATION_PRODUCTION_PACKAGES, workspaceDependencies: BLOB_FOUNDATION_WORKSPACE_DEPENDENCY_ALLOWLIST, thirdPartyDependencies: BLOB_FOUNDATION_THIRD_PARTY_DEPENDENCY_ALLOWLIST, forbiddenDependencyPrefixes: STORAGE_FOUNDATION_FORBIDDEN_DEPENDENCY_PREFIXES };
  }
  if (currentSlice === 'blob_runtime_foundation_v1') {
    return { profile: MANAGED_LAUNCH_TRUST_PROFILE, packages: BLOB_RUNTIME_FOUNDATION_PRODUCTION_PACKAGES, workspaceDependencies: BLOB_RUNTIME_FOUNDATION_WORKSPACE_DEPENDENCY_ALLOWLIST, thirdPartyDependencies: BLOB_RUNTIME_FOUNDATION_THIRD_PARTY_DEPENDENCY_ALLOWLIST, forbiddenDependencyPrefixes: STORAGE_FOUNDATION_FORBIDDEN_DEPENDENCY_PREFIXES };
  }
  if (currentSlice === 'scheduler_protocol_foundation_v1') {
    return { profile: MANAGED_LAUNCH_TRUST_PROFILE, packages: SCHEDULER_PROTOCOL_FOUNDATION_PRODUCTION_PACKAGES, workspaceDependencies: SCHEDULER_PROTOCOL_FOUNDATION_WORKSPACE_DEPENDENCY_ALLOWLIST, thirdPartyDependencies: SCHEDULER_PROTOCOL_FOUNDATION_THIRD_PARTY_DEPENDENCY_ALLOWLIST, forbiddenDependencyPrefixes: STORAGE_FOUNDATION_FORBIDDEN_DEPENDENCY_PREFIXES };
  }
  if (currentSlice === 'scheduler_foundation_v1') {
    return { profile: MANAGED_LAUNCH_TRUST_PROFILE, packages: SCHEDULER_FOUNDATION_PRODUCTION_PACKAGES, workspaceDependencies: SCHEDULER_FOUNDATION_WORKSPACE_DEPENDENCY_ALLOWLIST, thirdPartyDependencies: SCHEDULER_FOUNDATION_THIRD_PARTY_DEPENDENCY_ALLOWLIST, forbiddenDependencyPrefixes: STORAGE_FOUNDATION_FORBIDDEN_DEPENDENCY_PREFIXES };
  }
  if (currentSlice === 'scheduler_persistence_foundation_v1') {
    return { profile: MANAGED_LAUNCH_TRUST_PROFILE, packages: SCHEDULER_PERSISTENCE_FOUNDATION_PRODUCTION_PACKAGES, workspaceDependencies: SCHEDULER_PERSISTENCE_FOUNDATION_WORKSPACE_DEPENDENCY_ALLOWLIST, thirdPartyDependencies: SCHEDULER_PERSISTENCE_FOUNDATION_THIRD_PARTY_DEPENDENCY_ALLOWLIST, forbiddenDependencyPrefixes: STORAGE_FOUNDATION_FORBIDDEN_DEPENDENCY_PREFIXES };
  }
  if (currentSlice === 'gateway_session_foundation_v1') {
    return { profile: MANAGED_LAUNCH_TRUST_PROFILE, packages: GATEWAY_SESSION_FOUNDATION_PRODUCTION_PACKAGES, workspaceDependencies: GATEWAY_SESSION_FOUNDATION_WORKSPACE_DEPENDENCY_ALLOWLIST, thirdPartyDependencies: GATEWAY_SESSION_FOUNDATION_THIRD_PARTY_DEPENDENCY_ALLOWLIST, forbiddenDependencyPrefixes: STORAGE_FOUNDATION_FORBIDDEN_DEPENDENCY_PREFIXES };
  }
  if (currentSlice === 'scheduler_receipt_delivery_foundation_v1') {
    return { profile: MANAGED_LAUNCH_TRUST_PROFILE, packages: SCHEDULER_RECEIPT_DELIVERY_FOUNDATION_PRODUCTION_PACKAGES, workspaceDependencies: SCHEDULER_RECEIPT_DELIVERY_FOUNDATION_WORKSPACE_DEPENDENCY_ALLOWLIST, thirdPartyDependencies: SCHEDULER_RECEIPT_DELIVERY_FOUNDATION_THIRD_PARTY_DEPENDENCY_ALLOWLIST, forbiddenDependencyPrefixes: STORAGE_FOUNDATION_FORBIDDEN_DEPENDENCY_PREFIXES };
  }
  if (currentSlice === 'scheduler_jetstream_foundation_v1') {
    return { profile: MANAGED_LAUNCH_TRUST_PROFILE, packages: SCHEDULER_JETSTREAM_FOUNDATION_PRODUCTION_PACKAGES, workspaceDependencies: SCHEDULER_JETSTREAM_FOUNDATION_WORKSPACE_DEPENDENCY_ALLOWLIST, thirdPartyDependencies: SCHEDULER_JETSTREAM_FOUNDATION_THIRD_PARTY_DEPENDENCY_ALLOWLIST, forbiddenDependencyPrefixes: STORAGE_FOUNDATION_FORBIDDEN_DEPENDENCY_PREFIXES };
  }
  if (currentSlice === 'scheduler_runtime_foundation_v1') {
    return { profile: MANAGED_LAUNCH_TRUST_PROFILE, packages: SCHEDULER_RUNTIME_FOUNDATION_PRODUCTION_PACKAGES, workspaceDependencies: SCHEDULER_RUNTIME_FOUNDATION_WORKSPACE_DEPENDENCY_ALLOWLIST, thirdPartyDependencies: SCHEDULER_RUNTIME_FOUNDATION_THIRD_PARTY_DEPENDENCY_ALLOWLIST, forbiddenDependencyPrefixes: STORAGE_FOUNDATION_FORBIDDEN_DEPENDENCY_PREFIXES };
  }
  if (currentSlice === 'gateway_runtime_foundation_v1') {
    return { profile: MANAGED_LAUNCH_TRUST_PROFILE, packages: GATEWAY_RUNTIME_FOUNDATION_PRODUCTION_PACKAGES, workspaceDependencies: GATEWAY_RUNTIME_FOUNDATION_WORKSPACE_DEPENDENCY_ALLOWLIST, thirdPartyDependencies: GATEWAY_RUNTIME_FOUNDATION_THIRD_PARTY_DEPENDENCY_ALLOWLIST, forbiddenDependencyPrefixes: STORAGE_FOUNDATION_FORBIDDEN_DEPENDENCY_PREFIXES };
  }
  if (currentSlice === 'gateway_runtime_plus_mail_telegram_whatsapp_communications_v1') {
    return {
      profile: MANAGED_LAUNCH_TRUST_PROFILE,
      packages: MAIL_COMMUNICATIONS_FOUNDATION_PRODUCTION_PACKAGES,
      workspaceDependencies: MAIL_COMMUNICATIONS_FOUNDATION_WORKSPACE_DEPENDENCY_ALLOWLIST,
      thirdPartyDependencies: MAIL_COMMUNICATIONS_FOUNDATION_THIRD_PARTY_DEPENDENCY_ALLOWLIST,
      forbiddenDependencyPrefixes: STORAGE_FOUNDATION_FORBIDDEN_DEPENDENCY_PREFIXES,
    };
  }
  if (currentSlice === 'first_owner_v1') {
    return {
      profile: FIRST_OWNER_PROFILE,
      ownerInventory: FIRST_OWNER_INVENTORY,
      packages: FIRST_OWNER_PRODUCTION_PACKAGES,
      workspaceDependencies: FIRST_OWNER_WORKSPACE_DEPENDENCY_ALLOWLIST,
      thirdPartyDependencies: FIRST_OWNER_THIRD_PARTY_DEPENDENCY_ALLOWLIST,
      forbiddenDependencyPrefixes: STORAGE_FOUNDATION_FORBIDDEN_DEPENDENCY_PREFIXES,
    };
  }
  if (currentSlice === 'attachment_security_engine_v1') {
    return {
      profile: FIRST_OWNER_PROFILE,
      ownerInventory: ATTACHMENT_SECURITY_ENGINE_INVENTORY,
      packages: ATTACHMENT_SECURITY_ENGINE_PRODUCTION_PACKAGES,
      workspaceDependencies: ATTACHMENT_SECURITY_ENGINE_WORKSPACE_DEPENDENCY_ALLOWLIST,
      thirdPartyDependencies: ATTACHMENT_SECURITY_ENGINE_THIRD_PARTY_DEPENDENCY_ALLOWLIST,
      forbiddenDependencyPrefixes: STORAGE_FOUNDATION_FORBIDDEN_DEPENDENCY_PREFIXES,
    };
  }
  if (currentSlice === 'mail_outbound_mime_attachments_v1') {
    return {
      profile: FIRST_OWNER_PROFILE,
      ownerInventory: MAIL_OUTBOUND_MIME_ATTACHMENTS_INVENTORY,
      cargoFeatures: MAIL_OUTBOUND_MIME_ATTACHMENTS_CARGO_FEATURE_ALLOWLIST,
      packages: MAIL_OUTBOUND_MIME_ATTACHMENTS_PRODUCTION_PACKAGES,
      workspaceDependencies: MAIL_OUTBOUND_MIME_ATTACHMENTS_WORKSPACE_DEPENDENCY_ALLOWLIST,
      thirdPartyDependencies: MAIL_OUTBOUND_MIME_ATTACHMENTS_THIRD_PARTY_DEPENDENCY_ALLOWLIST,
      forbiddenDependencyPrefixes: STORAGE_FOUNDATION_FORBIDDEN_DEPENDENCY_PREFIXES,
    };
  }
  if (currentSlice === 'communications_content_read_v1') {
    return {
      profile: FIRST_OWNER_PROFILE,
      ownerInventory: COMMUNICATIONS_CONTENT_READ_INVENTORY,
      cargoFeatures: MAIL_OUTBOUND_MIME_ATTACHMENTS_CARGO_FEATURE_ALLOWLIST,
      packages: COMMUNICATIONS_CONTENT_READ_PRODUCTION_PACKAGES,
      workspaceDependencies: COMMUNICATIONS_CONTENT_READ_WORKSPACE_DEPENDENCY_ALLOWLIST,
      thirdPartyDependencies: COMMUNICATIONS_CONTENT_READ_THIRD_PARTY_DEPENDENCY_ALLOWLIST,
      forbiddenDependencyPrefixes: STORAGE_FOUNDATION_FORBIDDEN_DEPENDENCY_PREFIXES,
    };
  }
  if (currentSlice === 'communications_saved_search_v1') {
    return {
      profile: FIRST_OWNER_PROFILE,
      ownerInventory: COMMUNICATIONS_SAVED_SEARCH_INVENTORY,
      cargoFeatures: MAIL_OUTBOUND_MIME_ATTACHMENTS_CARGO_FEATURE_ALLOWLIST,
      packages: COMMUNICATIONS_SAVED_SEARCH_PRODUCTION_PACKAGES,
      workspaceDependencies: COMMUNICATIONS_SAVED_SEARCH_WORKSPACE_DEPENDENCY_ALLOWLIST,
      thirdPartyDependencies: COMMUNICATIONS_SAVED_SEARCH_THIRD_PARTY_DEPENDENCY_ALLOWLIST,
      forbiddenDependencyPrefixes: STORAGE_FOUNDATION_FORBIDDEN_DEPENDENCY_PREFIXES,
    };
  }
  if (currentSlice === 'communications_sender_insights_v1') {
    return {
      profile: FIRST_OWNER_PROFILE,
      ownerInventory: COMMUNICATIONS_SENDER_INSIGHTS_INVENTORY,
      cargoFeatures: MAIL_OUTBOUND_MIME_ATTACHMENTS_CARGO_FEATURE_ALLOWLIST,
      packages: COMMUNICATIONS_SENDER_INSIGHTS_PRODUCTION_PACKAGES,
      workspaceDependencies: COMMUNICATIONS_SENDER_INSIGHTS_WORKSPACE_DEPENDENCY_ALLOWLIST,
      thirdPartyDependencies: COMMUNICATIONS_SENDER_INSIGHTS_THIRD_PARTY_DEPENDENCY_ALLOWLIST,
      forbiddenDependencyPrefixes: STORAGE_FOUNDATION_FORBIDDEN_DEPENDENCY_PREFIXES,
    };
  }
  if (currentSlice === 'communications_export_v1') {
    return {
      profile: FIRST_OWNER_PROFILE,
      ownerInventory: COMMUNICATIONS_EXPORT_INVENTORY,
      cargoFeatures: MAIL_OUTBOUND_MIME_ATTACHMENTS_CARGO_FEATURE_ALLOWLIST,
      packages: COMMUNICATIONS_EXPORT_PRODUCTION_PACKAGES,
      workspaceDependencies: COMMUNICATIONS_EXPORT_WORKSPACE_DEPENDENCY_ALLOWLIST,
      thirdPartyDependencies: COMMUNICATIONS_EXPORT_THIRD_PARTY_DEPENDENCY_ALLOWLIST,
      forbiddenDependencyPrefixes: STORAGE_FOUNDATION_FORBIDDEN_DEPENDENCY_PREFIXES,
    };
  }
  if (currentSlice === 'communication_delivery_intent_contract_core_v1') {
    return {
      profile: FIRST_OWNER_PROFILE,
      ownerInventory: COMMUNICATIONS_EXPORT_INVENTORY,
      cargoFeatures: MAIL_OUTBOUND_MIME_ATTACHMENTS_CARGO_FEATURE_ALLOWLIST,
      packages: COMMUNICATION_DELIVERY_INTENT_CONTRACT_CORE_PRODUCTION_PACKAGES,
      workspaceDependencies:
        COMMUNICATION_DELIVERY_INTENT_CONTRACT_CORE_WORKSPACE_DEPENDENCY_ALLOWLIST,
      thirdPartyDependencies:
        COMMUNICATION_DELIVERY_INTENT_CONTRACT_CORE_THIRD_PARTY_DEPENDENCY_ALLOWLIST,
      forbiddenDependencyPrefixes: STORAGE_FOUNDATION_FORBIDDEN_DEPENDENCY_PREFIXES,
    };
  }
  if (currentSlice === 'communication_delivery_intent_persistence_v1') {
    return {
      profile: FIRST_OWNER_PROFILE,
      ownerInventory: COMMUNICATIONS_EXPORT_INVENTORY,
      cargoFeatures: MAIL_OUTBOUND_MIME_ATTACHMENTS_CARGO_FEATURE_ALLOWLIST,
      packages: COMMUNICATION_DELIVERY_INTENT_PERSISTENCE_PRODUCTION_PACKAGES,
      workspaceDependencies:
        COMMUNICATION_DELIVERY_INTENT_PERSISTENCE_WORKSPACE_DEPENDENCY_ALLOWLIST,
      thirdPartyDependencies:
        COMMUNICATION_DELIVERY_INTENT_PERSISTENCE_THIRD_PARTY_DEPENDENCY_ALLOWLIST,
      forbiddenDependencyPrefixes: STORAGE_FOUNDATION_FORBIDDEN_DEPENDENCY_PREFIXES,
    };
  }
  if (currentSlice === 'communication_delivery_intent_runtime_v1') {
    return {
      profile: FIRST_OWNER_PROFILE,
      ownerInventory: COMMUNICATIONS_EXPORT_INVENTORY,
      cargoFeatures: MAIL_OUTBOUND_MIME_ATTACHMENTS_CARGO_FEATURE_ALLOWLIST,
      packages: COMMUNICATION_DELIVERY_INTENT_RUNTIME_PRODUCTION_PACKAGES,
      workspaceDependencies:
        COMMUNICATION_DELIVERY_INTENT_RUNTIME_WORKSPACE_DEPENDENCY_ALLOWLIST,
      thirdPartyDependencies:
        COMMUNICATION_DELIVERY_INTENT_RUNTIME_THIRD_PARTY_DEPENDENCY_ALLOWLIST,
      forbiddenDependencyPrefixes: STORAGE_FOUNDATION_FORBIDDEN_DEPENDENCY_PREFIXES,
    };
  }
  if (currentSlice === 'communication_delivery_intent_assembly_v1') {
    return {
      profile: FIRST_OWNER_PROFILE,
      ownerInventory: COMMUNICATIONS_EXPORT_INVENTORY,
      cargoFeatures: MAIL_OUTBOUND_MIME_ATTACHMENTS_CARGO_FEATURE_ALLOWLIST,
      packages: COMMUNICATION_DELIVERY_INTENT_ASSEMBLY_PRODUCTION_PACKAGES,
      workspaceDependencies:
        COMMUNICATION_DELIVERY_INTENT_ASSEMBLY_WORKSPACE_DEPENDENCY_ALLOWLIST,
      thirdPartyDependencies:
        COMMUNICATION_DELIVERY_INTENT_ASSEMBLY_THIRD_PARTY_DEPENDENCY_ALLOWLIST,
      forbiddenDependencyPrefixes: STORAGE_FOUNDATION_FORBIDDEN_DEPENDENCY_PREFIXES,
    };
  }
  if (currentSlice === 'provider_delivery_intent_contracts_v1') {
    return {
      profile: FIRST_OWNER_PROFILE,
      ownerInventory: COMMUNICATIONS_EXPORT_INVENTORY,
      cargoFeatures: MAIL_OUTBOUND_MIME_ATTACHMENTS_CARGO_FEATURE_ALLOWLIST,
      packages: COMMUNICATION_DELIVERY_INTENT_ASSEMBLY_PRODUCTION_PACKAGES,
      workspaceDependencies:
        COMMUNICATION_DELIVERY_INTENT_ASSEMBLY_WORKSPACE_DEPENDENCY_ALLOWLIST,
      thirdPartyDependencies:
        COMMUNICATION_DELIVERY_INTENT_ASSEMBLY_THIRD_PARTY_DEPENDENCY_ALLOWLIST,
      forbiddenDependencyPrefixes: STORAGE_FOUNDATION_FORBIDDEN_DEPENDENCY_PREFIXES,
    };
  }
  if (currentSlice === 'delivery_intent_transactional_event_adapters_v1') {
    return {
      profile: FIRST_OWNER_PROFILE,
      ownerInventory: COMMUNICATIONS_EXPORT_INVENTORY,
      cargoFeatures: MAIL_OUTBOUND_MIME_ATTACHMENTS_CARGO_FEATURE_ALLOWLIST,
      packages: DELIVERY_INTENT_TRANSACTIONAL_EVENT_ADAPTERS_PRODUCTION_PACKAGES,
      workspaceDependencies:
        DELIVERY_INTENT_TRANSACTIONAL_EVENT_ADAPTERS_WORKSPACE_DEPENDENCY_ALLOWLIST,
      thirdPartyDependencies:
        DELIVERY_INTENT_TRANSACTIONAL_EVENT_ADAPTERS_THIRD_PARTY_DEPENDENCY_ALLOWLIST,
      forbiddenDependencyPrefixes: STORAGE_FOUNDATION_FORBIDDEN_DEPENDENCY_PREFIXES,
    };
  }
  if (currentSlice === 'delivery_intent_target_bound_blob_v1') {
    return {
      profile: FIRST_OWNER_PROFILE,
      ownerInventory: COMMUNICATION_DELIVERY_INTENT_INVENTORY,
      cargoFeatures: MAIL_OUTBOUND_MIME_ATTACHMENTS_CARGO_FEATURE_ALLOWLIST,
      packages: DELIVERY_INTENT_TRANSACTIONAL_EVENT_ADAPTERS_PRODUCTION_PACKAGES,
      workspaceDependencies:
        DELIVERY_INTENT_TARGET_BOUND_BLOB_WORKSPACE_DEPENDENCY_ALLOWLIST,
      thirdPartyDependencies:
        DELIVERY_INTENT_TARGET_BOUND_BLOB_THIRD_PARTY_DEPENDENCY_ALLOWLIST,
      forbiddenDependencyPrefixes: STORAGE_FOUNDATION_FORBIDDEN_DEPENDENCY_PREFIXES,
    };
  }
  if (currentSlice === 'communication_bulk_action_contract_core_v1') {
    return {
      profile: FIRST_OWNER_PROFILE,
      ownerInventory: COMMUNICATION_DELIVERY_INTENT_INVENTORY,
      cargoFeatures: MAIL_OUTBOUND_MIME_ATTACHMENTS_CARGO_FEATURE_ALLOWLIST,
      packages: COMMUNICATION_BULK_ACTION_CONTRACT_CORE_PRODUCTION_PACKAGES,
      workspaceDependencies:
        COMMUNICATION_BULK_ACTION_CONTRACT_CORE_WORKSPACE_DEPENDENCY_ALLOWLIST,
      thirdPartyDependencies:
        COMMUNICATION_BULK_ACTION_CONTRACT_CORE_THIRD_PARTY_DEPENDENCY_ALLOWLIST,
      forbiddenDependencyPrefixes: STORAGE_FOUNDATION_FORBIDDEN_DEPENDENCY_PREFIXES,
    };
  }
  if (currentSlice === 'communication_bulk_action_persistence_v1') {
    return {
      profile: FIRST_OWNER_PROFILE,
      ownerInventory: COMMUNICATION_DELIVERY_INTENT_INVENTORY,
      cargoFeatures: MAIL_OUTBOUND_MIME_ATTACHMENTS_CARGO_FEATURE_ALLOWLIST,
      packages: COMMUNICATION_BULK_ACTION_PERSISTENCE_PRODUCTION_PACKAGES,
      workspaceDependencies:
        COMMUNICATION_BULK_ACTION_PERSISTENCE_WORKSPACE_DEPENDENCY_ALLOWLIST,
      thirdPartyDependencies:
        COMMUNICATION_BULK_ACTION_PERSISTENCE_THIRD_PARTY_DEPENDENCY_ALLOWLIST,
      forbiddenDependencyPrefixes: STORAGE_FOUNDATION_FORBIDDEN_DEPENDENCY_PREFIXES,
    };
  }
  if (currentSlice === 'communication_bulk_action_managed_runtime_v1') {
    return {
      profile: FIRST_OWNER_PROFILE,
      ownerInventory: COMMUNICATION_DELIVERY_INTENT_INVENTORY,
      cargoFeatures: MAIL_OUTBOUND_MIME_ATTACHMENTS_CARGO_FEATURE_ALLOWLIST,
      packages: COMMUNICATION_BULK_ACTION_RUNTIME_CORE_PRODUCTION_PACKAGES,
      workspaceDependencies:
        COMMUNICATION_BULK_ACTION_RUNTIME_CORE_WORKSPACE_DEPENDENCY_ALLOWLIST,
      thirdPartyDependencies:
        COMMUNICATION_BULK_ACTION_RUNTIME_CORE_THIRD_PARTY_DEPENDENCY_ALLOWLIST,
      forbiddenDependencyPrefixes: STORAGE_FOUNDATION_FORBIDDEN_DEPENDENCY_PREFIXES,
    };
  }
  if (
    currentSlice === 'communication_bulk_action_assembly_v1'
    || currentSlice === 'communication_bulk_action_v1'
  ) {
    return {
      profile: FIRST_OWNER_PROFILE,
      ownerInventory: COMMUNICATION_DELIVERY_INTENT_INVENTORY,
      cargoFeatures: MAIL_OUTBOUND_MIME_ATTACHMENTS_CARGO_FEATURE_ALLOWLIST,
      packages: COMMUNICATION_BULK_ACTION_ASSEMBLY_PRODUCTION_PACKAGES,
      workspaceDependencies:
        COMMUNICATION_BULK_ACTION_ASSEMBLY_WORKSPACE_DEPENDENCY_ALLOWLIST,
      thirdPartyDependencies:
        COMMUNICATION_BULK_ACTION_ASSEMBLY_THIRD_PARTY_DEPENDENCY_ALLOWLIST,
      forbiddenDependencyPrefixes: STORAGE_FOUNDATION_FORBIDDEN_DEPENDENCY_PREFIXES,
    };
  }
  if (currentSlice === 'communication_delayed_delivery_contract_core_v1') {
    return {
      profile: FIRST_OWNER_PROFILE,
      ownerInventory: COMMUNICATION_DELIVERY_INTENT_INVENTORY,
      cargoFeatures: MAIL_OUTBOUND_MIME_ATTACHMENTS_CARGO_FEATURE_ALLOWLIST,
      packages: COMMUNICATION_DELAYED_DELIVERY_CONTRACT_CORE_PRODUCTION_PACKAGES,
      workspaceDependencies:
        COMMUNICATION_DELAYED_DELIVERY_CONTRACT_CORE_WORKSPACE_DEPENDENCY_ALLOWLIST,
      thirdPartyDependencies:
        COMMUNICATION_DELAYED_DELIVERY_CONTRACT_CORE_THIRD_PARTY_DEPENDENCY_ALLOWLIST,
      forbiddenDependencyPrefixes: STORAGE_FOUNDATION_FORBIDDEN_DEPENDENCY_PREFIXES,
    };
  }
  if (currentSlice === 'communication_delayed_delivery_persistence_v1') {
    return {
      profile: FIRST_OWNER_PROFILE,
      ownerInventory: COMMUNICATION_DELIVERY_INTENT_INVENTORY,
      cargoFeatures: MAIL_OUTBOUND_MIME_ATTACHMENTS_CARGO_FEATURE_ALLOWLIST,
      packages: COMMUNICATION_DELAYED_DELIVERY_PERSISTENCE_PRODUCTION_PACKAGES,
      workspaceDependencies:
        COMMUNICATION_DELAYED_DELIVERY_PERSISTENCE_WORKSPACE_DEPENDENCY_ALLOWLIST,
      thirdPartyDependencies:
        COMMUNICATION_DELAYED_DELIVERY_PERSISTENCE_THIRD_PARTY_DEPENDENCY_ALLOWLIST,
      forbiddenDependencyPrefixes: STORAGE_FOUNDATION_FORBIDDEN_DEPENDENCY_PREFIXES,
    };
  }
  if (
    currentSlice === 'communication_delayed_delivery_runtime_adapters_v1'
    || currentSlice === 'communication_delayed_delivery_due_event_adapter_v1'
    || currentSlice === 'communication_delayed_delivery_store_adapter_v1'
    || currentSlice === 'communication_delayed_delivery_persistence_runtime_surfaces_v1'
  ) {
    return {
      profile: FIRST_OWNER_PROFILE,
      ownerInventory: COMMUNICATION_DELIVERY_INTENT_INVENTORY,
      cargoFeatures: MAIL_OUTBOUND_MIME_ATTACHMENTS_CARGO_FEATURE_ALLOWLIST,
      packages: [
        'communication_delayed_delivery_store_adapter_v1',
        'communication_delayed_delivery_persistence_runtime_surfaces_v1',
      ].includes(currentSlice)
        ? COMMUNICATION_DELAYED_DELIVERY_STORE_ADAPTERS_PRODUCTION_PACKAGES
        : COMMUNICATION_DELAYED_DELIVERY_RUNTIME_ADAPTERS_PRODUCTION_PACKAGES,
      workspaceDependencies:
        [
          'communication_delayed_delivery_store_adapter_v1',
          'communication_delayed_delivery_persistence_runtime_surfaces_v1',
        ].includes(currentSlice)
          ? COMMUNICATION_DELAYED_DELIVERY_STORE_ADAPTERS_WORKSPACE_DEPENDENCY_ALLOWLIST
          : COMMUNICATION_DELAYED_DELIVERY_RUNTIME_ADAPTERS_WORKSPACE_DEPENDENCY_ALLOWLIST,
      thirdPartyDependencies:
        [
          'communication_delayed_delivery_store_adapter_v1',
          'communication_delayed_delivery_persistence_runtime_surfaces_v1',
        ].includes(currentSlice)
          ? COMMUNICATION_DELAYED_DELIVERY_STORE_ADAPTERS_THIRD_PARTY_DEPENDENCY_ALLOWLIST
          : COMMUNICATION_DELAYED_DELIVERY_RUNTIME_ADAPTERS_THIRD_PARTY_DEPENDENCY_ALLOWLIST,
      forbiddenDependencyPrefixes: STORAGE_FOUNDATION_FORBIDDEN_DEPENDENCY_PREFIXES,
    };
  }
  if (currentSlice === 'communication_delayed_delivery_managed_runtime_v1') {
    return {
      profile: FIRST_OWNER_PROFILE,
      ownerInventory: COMMUNICATION_DELIVERY_INTENT_INVENTORY,
      cargoFeatures: MAIL_OUTBOUND_MIME_ATTACHMENTS_CARGO_FEATURE_ALLOWLIST,
      packages: COMMUNICATION_DELAYED_DELIVERY_MANAGED_RUNTIME_PRODUCTION_PACKAGES,
      workspaceDependencies:
        COMMUNICATION_DELAYED_DELIVERY_MANAGED_RUNTIME_WORKSPACE_DEPENDENCY_ALLOWLIST,
      thirdPartyDependencies:
        COMMUNICATION_DELAYED_DELIVERY_MANAGED_RUNTIME_THIRD_PARTY_DEPENDENCY_ALLOWLIST,
      forbiddenDependencyPrefixes: STORAGE_FOUNDATION_FORBIDDEN_DEPENDENCY_PREFIXES,
    };
  }
  if (currentSlice === 'communication_delayed_delivery_assembly_v1') {
    return {
      profile: FIRST_OWNER_PROFILE,
      ownerInventory: COMMUNICATION_DELIVERY_INTENT_INVENTORY,
      cargoFeatures: MAIL_OUTBOUND_MIME_ATTACHMENTS_CARGO_FEATURE_ALLOWLIST,
      packages: COMMUNICATION_DELAYED_DELIVERY_ASSEMBLY_PRODUCTION_PACKAGES,
      workspaceDependencies:
        COMMUNICATION_DELAYED_DELIVERY_ASSEMBLY_WORKSPACE_DEPENDENCY_ALLOWLIST,
      thirdPartyDependencies:
        COMMUNICATION_DELAYED_DELIVERY_ASSEMBLY_THIRD_PARTY_DEPENDENCY_ALLOWLIST,
      forbiddenDependencyPrefixes: STORAGE_FOUNDATION_FORBIDDEN_DEPENDENCY_PREFIXES,
    };
  }
  if (currentSlice === 'communication_cross_channel_forward_contract_core_v1') {
    return {
      profile: FIRST_OWNER_PROFILE,
      ownerInventory: COMMUNICATION_DELIVERY_INTENT_INVENTORY,
      cargoFeatures: MAIL_OUTBOUND_MIME_ATTACHMENTS_CARGO_FEATURE_ALLOWLIST,
      packages: COMMUNICATION_CROSS_CHANNEL_FORWARD_CONTRACT_CORE_PRODUCTION_PACKAGES,
      workspaceDependencies:
        COMMUNICATION_CROSS_CHANNEL_FORWARD_CONTRACT_CORE_WORKSPACE_DEPENDENCY_ALLOWLIST,
      thirdPartyDependencies:
        COMMUNICATION_CROSS_CHANNEL_FORWARD_CONTRACT_CORE_THIRD_PARTY_DEPENDENCY_ALLOWLIST,
      forbiddenDependencyPrefixes: STORAGE_FOUNDATION_FORBIDDEN_DEPENDENCY_PREFIXES,
    };
  }
  if (currentSlice === 'communication_cross_channel_forward_persistence_v1') {
    return {
      profile: FIRST_OWNER_PROFILE,
      ownerInventory: COMMUNICATION_DELIVERY_INTENT_INVENTORY,
      cargoFeatures: MAIL_OUTBOUND_MIME_ATTACHMENTS_CARGO_FEATURE_ALLOWLIST,
      packages: COMMUNICATION_CROSS_CHANNEL_FORWARD_PERSISTENCE_PRODUCTION_PACKAGES,
      workspaceDependencies:
        COMMUNICATION_CROSS_CHANNEL_FORWARD_PERSISTENCE_WORKSPACE_DEPENDENCY_ALLOWLIST,
      thirdPartyDependencies:
        COMMUNICATION_CROSS_CHANNEL_FORWARD_PERSISTENCE_THIRD_PARTY_DEPENDENCY_ALLOWLIST,
      forbiddenDependencyPrefixes: STORAGE_FOUNDATION_FORBIDDEN_DEPENDENCY_PREFIXES,
    };
  }
  if (currentSlice === 'communication_cross_channel_forward_source_contract_v1') {
    return {
      profile: FIRST_OWNER_PROFILE,
      ownerInventory: COMMUNICATION_DELIVERY_INTENT_INVENTORY,
      cargoFeatures: MAIL_OUTBOUND_MIME_ATTACHMENTS_CARGO_FEATURE_ALLOWLIST,
      packages: COMMUNICATION_CROSS_CHANNEL_FORWARD_SOURCE_CONTRACT_PRODUCTION_PACKAGES,
      workspaceDependencies:
        COMMUNICATION_CROSS_CHANNEL_FORWARD_SOURCE_CONTRACT_WORKSPACE_DEPENDENCY_ALLOWLIST,
      thirdPartyDependencies:
        COMMUNICATION_CROSS_CHANNEL_FORWARD_SOURCE_CONTRACT_THIRD_PARTY_DEPENDENCY_ALLOWLIST,
      forbiddenDependencyPrefixes: STORAGE_FOUNDATION_FORBIDDEN_DEPENDENCY_PREFIXES,
    };
  }
  if (currentSlice === 'communication_delivery_intent_ingress_contract_v1') {
    return {
      profile: FIRST_OWNER_PROFILE,
      ownerInventory: COMMUNICATION_DELIVERY_INTENT_INVENTORY,
      cargoFeatures: MAIL_OUTBOUND_MIME_ATTACHMENTS_CARGO_FEATURE_ALLOWLIST,
      packages: COMMUNICATION_DELIVERY_INTENT_INGRESS_CONTRACT_PRODUCTION_PACKAGES,
      workspaceDependencies:
        COMMUNICATION_DELIVERY_INTENT_INGRESS_CONTRACT_WORKSPACE_DEPENDENCY_ALLOWLIST,
      thirdPartyDependencies:
        COMMUNICATION_DELIVERY_INTENT_INGRESS_CONTRACT_THIRD_PARTY_DEPENDENCY_ALLOWLIST,
      forbiddenDependencyPrefixes: STORAGE_FOUNDATION_FORBIDDEN_DEPENDENCY_PREFIXES,
    };
  }
  if (currentSlice === 'communication_cross_channel_forward_event_persistence_v1') {
    return {
      profile: FIRST_OWNER_PROFILE,
      ownerInventory: COMMUNICATION_DELIVERY_INTENT_INVENTORY,
      cargoFeatures: MAIL_OUTBOUND_MIME_ATTACHMENTS_CARGO_FEATURE_ALLOWLIST,
      packages: COMMUNICATION_CROSS_CHANNEL_FORWARD_EVENT_PERSISTENCE_PRODUCTION_PACKAGES,
      workspaceDependencies:
        COMMUNICATION_CROSS_CHANNEL_FORWARD_EVENT_PERSISTENCE_WORKSPACE_DEPENDENCY_ALLOWLIST,
      thirdPartyDependencies:
        COMMUNICATION_CROSS_CHANNEL_FORWARD_EVENT_PERSISTENCE_THIRD_PARTY_DEPENDENCY_ALLOWLIST,
      forbiddenDependencyPrefixes: STORAGE_FOUNDATION_FORBIDDEN_DEPENDENCY_PREFIXES,
    };
  }
  if (currentSlice === 'communication_cross_channel_forward_managed_runtime_v1') {
    return {
      profile: FIRST_OWNER_PROFILE,
      ownerInventory: COMMUNICATION_DELIVERY_INTENT_INVENTORY,
      cargoFeatures: MAIL_OUTBOUND_MIME_ATTACHMENTS_CARGO_FEATURE_ALLOWLIST,
      packages: COMMUNICATION_CROSS_CHANNEL_FORWARD_MANAGED_RUNTIME_PRODUCTION_PACKAGES,
      workspaceDependencies:
        COMMUNICATION_CROSS_CHANNEL_FORWARD_MANAGED_RUNTIME_WORKSPACE_DEPENDENCY_ALLOWLIST,
      thirdPartyDependencies:
        COMMUNICATION_CROSS_CHANNEL_FORWARD_MANAGED_RUNTIME_THIRD_PARTY_DEPENDENCY_ALLOWLIST,
      forbiddenDependencyPrefixes: STORAGE_FOUNDATION_FORBIDDEN_DEPENDENCY_PREFIXES,
    };
  }
  if (currentSlice === 'communication_cross_channel_forward_terminal_results_v1') {
    return {
      profile: FIRST_OWNER_PROFILE,
      ownerInventory: COMMUNICATION_DELIVERY_INTENT_INVENTORY,
      cargoFeatures: MAIL_OUTBOUND_MIME_ATTACHMENTS_CARGO_FEATURE_ALLOWLIST,
      packages: COMMUNICATION_CROSS_CHANNEL_FORWARD_MANAGED_RUNTIME_PRODUCTION_PACKAGES,
      workspaceDependencies:
        COMMUNICATION_DELIVERY_INTENT_EVENT_INGRESS_CONSUMER_WORKSPACE_DEPENDENCY_ALLOWLIST,
      thirdPartyDependencies:
        COMMUNICATION_DELIVERY_INTENT_EVENT_INGRESS_CONSUMER_THIRD_PARTY_DEPENDENCY_ALLOWLIST,
      forbiddenDependencyPrefixes: STORAGE_FOUNDATION_FORBIDDEN_DEPENDENCY_PREFIXES,
    };
  }
  if (currentSlice === 'communication_cross_channel_forward_client_assembly_v1') {
    return {
      profile: FIRST_OWNER_PROFILE,
      ownerInventory: COMMUNICATION_DELIVERY_INTENT_INVENTORY,
      cargoFeatures: MAIL_OUTBOUND_MIME_ATTACHMENTS_CARGO_FEATURE_ALLOWLIST,
      packages: COMMUNICATION_CROSS_CHANNEL_FORWARD_CLIENT_ASSEMBLY_PRODUCTION_PACKAGES,
      workspaceDependencies:
        COMMUNICATION_CROSS_CHANNEL_FORWARD_CLIENT_ASSEMBLY_WORKSPACE_DEPENDENCY_ALLOWLIST,
      thirdPartyDependencies:
        COMMUNICATION_CROSS_CHANNEL_FORWARD_CLIENT_ASSEMBLY_THIRD_PARTY_DEPENDENCY_ALLOWLIST,
      forbiddenDependencyPrefixes: STORAGE_FOUNDATION_FORBIDDEN_DEPENDENCY_PREFIXES,
    };
  }
  if (currentSlice === 'communications_call_evidence_contract_core_v1') {
    return {
      profile: FIRST_OWNER_PROFILE,
      ownerInventory: COMMUNICATION_DELIVERY_INTENT_INVENTORY,
      cargoFeatures: MAIL_OUTBOUND_MIME_ATTACHMENTS_CARGO_FEATURE_ALLOWLIST,
      packages: COMMUNICATIONS_CALL_EVIDENCE_CONTRACT_CORE_PRODUCTION_PACKAGES,
      workspaceDependencies:
        COMMUNICATIONS_CALL_EVIDENCE_CONTRACT_CORE_WORKSPACE_DEPENDENCY_ALLOWLIST,
      thirdPartyDependencies:
        COMMUNICATIONS_CALL_EVIDENCE_CONTRACT_CORE_THIRD_PARTY_DEPENDENCY_ALLOWLIST,
      forbiddenDependencyPrefixes: STORAGE_FOUNDATION_FORBIDDEN_DEPENDENCY_PREFIXES,
    };
  }
  if (currentSlice === 'communications_call_evidence_persistence_v1') {
    return {
      profile: FIRST_OWNER_PROFILE,
      ownerInventory: COMMUNICATION_DELIVERY_INTENT_INVENTORY,
      cargoFeatures: MAIL_OUTBOUND_MIME_ATTACHMENTS_CARGO_FEATURE_ALLOWLIST,
      packages: COMMUNICATIONS_CALL_EVIDENCE_PERSISTENCE_PRODUCTION_PACKAGES,
      workspaceDependencies:
        COMMUNICATIONS_CALL_EVIDENCE_PERSISTENCE_WORKSPACE_DEPENDENCY_ALLOWLIST,
      thirdPartyDependencies:
        COMMUNICATIONS_CALL_EVIDENCE_PERSISTENCE_THIRD_PARTY_DEPENDENCY_ALLOWLIST,
      forbiddenDependencyPrefixes: STORAGE_FOUNDATION_FORBIDDEN_DEPENDENCY_PREFIXES,
    };
  }
  if (currentSlice === 'communications_call_evidence_managed_consumer_v1') {
    return {
      profile: FIRST_OWNER_PROFILE,
      ownerInventory: COMMUNICATION_DELIVERY_INTENT_INVENTORY,
      cargoFeatures: MAIL_OUTBOUND_MIME_ATTACHMENTS_CARGO_FEATURE_ALLOWLIST,
      packages: COMMUNICATIONS_CALL_EVIDENCE_PERSISTENCE_PRODUCTION_PACKAGES,
      workspaceDependencies:
        COMMUNICATIONS_CALL_EVIDENCE_MANAGED_CONSUMER_WORKSPACE_DEPENDENCY_ALLOWLIST,
      thirdPartyDependencies:
        COMMUNICATIONS_CALL_EVIDENCE_PERSISTENCE_THIRD_PARTY_DEPENDENCY_ALLOWLIST,
      forbiddenDependencyPrefixes: STORAGE_FOUNDATION_FORBIDDEN_DEPENDENCY_PREFIXES,
    };
  }
  if (currentSlice === 'communications_call_evidence_query_realtime_v1') {
    return {
      profile: FIRST_OWNER_PROFILE,
      ownerInventory: COMMUNICATION_DELIVERY_INTENT_INVENTORY,
      cargoFeatures: MAIL_OUTBOUND_MIME_ATTACHMENTS_CARGO_FEATURE_ALLOWLIST,
      packages: COMMUNICATIONS_CALL_EVIDENCE_QUERY_REALTIME_PRODUCTION_PACKAGES,
      workspaceDependencies:
        COMMUNICATIONS_CALL_EVIDENCE_QUERY_REALTIME_WORKSPACE_DEPENDENCY_ALLOWLIST,
      thirdPartyDependencies:
        COMMUNICATIONS_CALL_EVIDENCE_QUERY_REALTIME_THIRD_PARTY_DEPENDENCY_ALLOWLIST,
      forbiddenDependencyPrefixes: STORAGE_FOUNDATION_FORBIDDEN_DEPENDENCY_PREFIXES,
    };
  }
  if (currentSlice === 'review_communications_attention_contract_core_v1') {
    return {
      profile: FIRST_OWNER_PROFILE,
      ownerInventory: REVIEW_COMMUNICATIONS_ATTENTION_CONTRACT_CORE_INVENTORY,
      cargoFeatures: MAIL_OUTBOUND_MIME_ATTACHMENTS_CARGO_FEATURE_ALLOWLIST,
      packages: REVIEW_COMMUNICATIONS_ATTENTION_CONTRACT_CORE_PRODUCTION_PACKAGES,
      workspaceDependencies:
        REVIEW_COMMUNICATIONS_ATTENTION_CONTRACT_CORE_WORKSPACE_DEPENDENCY_ALLOWLIST,
      thirdPartyDependencies:
        REVIEW_COMMUNICATIONS_ATTENTION_CONTRACT_CORE_THIRD_PARTY_DEPENDENCY_ALLOWLIST,
      forbiddenDependencyPrefixes: STORAGE_FOUNDATION_FORBIDDEN_DEPENDENCY_PREFIXES,
    };
  }
  if (
    currentSlice === 'review_communications_attention_persistence_v1'
    || currentSlice === 'review_communications_attention_read_realtime_persistence_v1'
  ) {
    return {
      profile: FIRST_OWNER_PROFILE,
      ownerInventory: REVIEW_COMMUNICATIONS_ATTENTION_CONTRACT_CORE_INVENTORY,
      cargoFeatures: MAIL_OUTBOUND_MIME_ATTACHMENTS_CARGO_FEATURE_ALLOWLIST,
      packages: REVIEW_COMMUNICATIONS_ATTENTION_PERSISTENCE_PRODUCTION_PACKAGES,
      workspaceDependencies:
        REVIEW_COMMUNICATIONS_ATTENTION_PERSISTENCE_WORKSPACE_DEPENDENCY_ALLOWLIST,
      thirdPartyDependencies:
        REVIEW_COMMUNICATIONS_ATTENTION_PERSISTENCE_THIRD_PARTY_DEPENDENCY_ALLOWLIST,
      forbiddenDependencyPrefixes: STORAGE_FOUNDATION_FORBIDDEN_DEPENDENCY_PREFIXES,
    };
  }
  if (currentSlice === 'review_communications_attention_managed_runtime_v1') {
    return {
      profile: FIRST_OWNER_PROFILE,
      ownerInventory: REVIEW_COMMUNICATIONS_ATTENTION_CONTRACT_CORE_INVENTORY,
      cargoFeatures: MAIL_OUTBOUND_MIME_ATTACHMENTS_CARGO_FEATURE_ALLOWLIST,
      packages: REVIEW_COMMUNICATIONS_ATTENTION_MANAGED_RUNTIME_PRODUCTION_PACKAGES,
      workspaceDependencies:
        REVIEW_COMMUNICATIONS_ATTENTION_MANAGED_RUNTIME_WORKSPACE_DEPENDENCY_ALLOWLIST,
      thirdPartyDependencies:
        REVIEW_COMMUNICATIONS_ATTENTION_MANAGED_RUNTIME_THIRD_PARTY_DEPENDENCY_ALLOWLIST,
      forbiddenDependencyPrefixes: STORAGE_FOUNDATION_FORBIDDEN_DEPENDENCY_PREFIXES,
    };
  }
  if (currentSlice === 'review_communications_attention_release_assembly_v1') {
    return {
      profile: FIRST_OWNER_PROFILE,
      ownerInventory: REVIEW_COMMUNICATIONS_ATTENTION_CONTRACT_CORE_INVENTORY,
      cargoFeatures: MAIL_OUTBOUND_MIME_ATTACHMENTS_CARGO_FEATURE_ALLOWLIST,
      packages: REVIEW_COMMUNICATIONS_ATTENTION_ASSEMBLY_PRODUCTION_PACKAGES,
      workspaceDependencies:
        REVIEW_COMMUNICATIONS_ATTENTION_ASSEMBLY_WORKSPACE_DEPENDENCY_ALLOWLIST,
      thirdPartyDependencies:
        REVIEW_COMMUNICATIONS_ATTENTION_ASSEMBLY_THIRD_PARTY_DEPENDENCY_ALLOWLIST,
      forbiddenDependencyPrefixes: STORAGE_FOUNDATION_FORBIDDEN_DEPENDENCY_PREFIXES,
    };
  }
  if (currentSlice === 'review_communications_attention_v1') {
    return {
      profile: FIRST_OWNER_PROFILE,
      ownerInventory: COMMUNICATIONS_AI_SOURCE_CONTRACT_INVENTORY,
      cargoFeatures: MAIL_OUTBOUND_MIME_ATTACHMENTS_CARGO_FEATURE_ALLOWLIST,
      packages: COMMUNICATIONS_AI_SOURCE_CONTRACT_PRODUCTION_PACKAGES,
      workspaceDependencies:
        COMMUNICATIONS_AI_SOURCE_CONTRACT_WORKSPACE_DEPENDENCY_ALLOWLIST,
      thirdPartyDependencies:
        COMMUNICATIONS_AI_SOURCE_CONTRACT_THIRD_PARTY_DEPENDENCY_ALLOWLIST,
      forbiddenDependencyPrefixes: STORAGE_FOUNDATION_FORBIDDEN_DEPENDENCY_PREFIXES,
    };
  }
  if (currentSlice === 'attachment_archive_inspection_contract_core_v1') {
    return {
      profile: FIRST_OWNER_PROFILE,
      ownerInventory: ATTACHMENT_ARCHIVE_INSPECTION_CONTRACT_CORE_INVENTORY,
      cargoFeatures: MAIL_OUTBOUND_MIME_ATTACHMENTS_CARGO_FEATURE_ALLOWLIST,
      packages: ATTACHMENT_ARCHIVE_INSPECTION_CONTRACT_CORE_PRODUCTION_PACKAGES,
      workspaceDependencies:
        ATTACHMENT_ARCHIVE_INSPECTION_CONTRACT_CORE_WORKSPACE_DEPENDENCY_ALLOWLIST,
      thirdPartyDependencies:
        ATTACHMENT_ARCHIVE_INSPECTION_CONTRACT_CORE_THIRD_PARTY_DEPENDENCY_ALLOWLIST,
      forbiddenDependencyPrefixes: STORAGE_FOUNDATION_FORBIDDEN_DEPENDENCY_PREFIXES,
    };
  }
  if (
    currentSlice === 'attachment_archive_inspection_persistence_join_v1'
    || currentSlice === 'blob_current_custodian_redelegation_v1'
    || currentSlice === 'attachment_archive_inspection_ingress_contract_v1'
    || currentSlice === 'attachment_archive_inspection_event_replay_persistence_v1'
    || currentSlice === 'attachment_security_archive_delegation_runtime_v1'
  ) {
    return {
      profile: FIRST_OWNER_PROFILE,
      ownerInventory: ATTACHMENT_ARCHIVE_INSPECTION_CONTRACT_CORE_INVENTORY,
      cargoFeatures: MAIL_OUTBOUND_MIME_ATTACHMENTS_CARGO_FEATURE_ALLOWLIST,
      packages: ATTACHMENT_ARCHIVE_INSPECTION_PERSISTENCE_PRODUCTION_PACKAGES,
      workspaceDependencies:
        ATTACHMENT_ARCHIVE_INSPECTION_PERSISTENCE_WORKSPACE_DEPENDENCY_ALLOWLIST,
      thirdPartyDependencies:
        ATTACHMENT_ARCHIVE_INSPECTION_PERSISTENCE_THIRD_PARTY_DEPENDENCY_ALLOWLIST,
      forbiddenDependencyPrefixes: STORAGE_FOUNDATION_FORBIDDEN_DEPENDENCY_PREFIXES,
    };
  }
  if (currentSlice === 'attachment_archive_inspection_managed_runtime_v1') {
    return {
      profile: FIRST_OWNER_PROFILE,
      ownerInventory: ATTACHMENT_ARCHIVE_INSPECTION_RUNTIME_INVENTORY,
      cargoFeatures: MAIL_OUTBOUND_MIME_ATTACHMENTS_CARGO_FEATURE_ALLOWLIST,
      packages: ATTACHMENT_ARCHIVE_INSPECTION_RUNTIME_PRODUCTION_PACKAGES,
      workspaceDependencies:
        ATTACHMENT_ARCHIVE_INSPECTION_RUNTIME_WORKSPACE_DEPENDENCY_ALLOWLIST,
      thirdPartyDependencies:
        ATTACHMENT_ARCHIVE_INSPECTION_RUNTIME_THIRD_PARTY_DEPENDENCY_ALLOWLIST,
      forbiddenDependencyPrefixes: STORAGE_FOUNDATION_FORBIDDEN_DEPENDENCY_PREFIXES,
    };
  }
  if (currentSlice === 'attachment_archive_inspection_release_assembly_v1') {
    return {
      profile: FIRST_OWNER_PROFILE,
      ownerInventory: ATTACHMENT_ARCHIVE_INSPECTION_RUNTIME_INVENTORY,
      cargoFeatures: MAIL_OUTBOUND_MIME_ATTACHMENTS_CARGO_FEATURE_ALLOWLIST,
      packages: ATTACHMENT_ARCHIVE_INSPECTION_ASSEMBLY_PRODUCTION_PACKAGES,
      workspaceDependencies:
        ATTACHMENT_ARCHIVE_INSPECTION_ASSEMBLY_WORKSPACE_DEPENDENCY_ALLOWLIST,
      thirdPartyDependencies:
        ATTACHMENT_ARCHIVE_INSPECTION_ASSEMBLY_THIRD_PARTY_DEPENDENCY_ALLOWLIST,
      forbiddenDependencyPrefixes: STORAGE_FOUNDATION_FORBIDDEN_DEPENDENCY_PREFIXES,
    };
  }
  if (
    currentSlice === 'attachment_archive_inspection_v1'
    || currentSlice === 'ollama_ai_provider_v1'
    || currentSlice === 'ai_inference_v1'
    || currentSlice === 'communication_reply_suggestion_v1'
  ) {
    return {
      profile: FIRST_OWNER_PROFILE,
      ownerInventory: ATTACHMENT_ARCHIVE_INSPECTION_CLIENT_INVENTORY,
      cargoFeatures: MAIL_OUTBOUND_MIME_ATTACHMENTS_CARGO_FEATURE_ALLOWLIST,
      packages: ATTACHMENT_ARCHIVE_INSPECTION_ASSEMBLY_PRODUCTION_PACKAGES,
      workspaceDependencies:
        ATTACHMENT_ARCHIVE_INSPECTION_ASSEMBLY_WORKSPACE_DEPENDENCY_ALLOWLIST,
      thirdPartyDependencies:
        ATTACHMENT_ARCHIVE_INSPECTION_ASSEMBLY_THIRD_PARTY_DEPENDENCY_ALLOWLIST,
      forbiddenDependencyPrefixes: STORAGE_FOUNDATION_FORBIDDEN_DEPENDENCY_PREFIXES,
    };
  }
  if (currentSlice === 'communication_summary_v1') {
    return {
      profile: FIRST_OWNER_PROFILE,
      ownerInventory: COMMUNICATION_SUMMARY_BUILD_UNITS_INVENTORY,
      cargoFeatures: MAIL_OUTBOUND_MIME_ATTACHMENTS_CARGO_FEATURE_ALLOWLIST,
      packages: COMMUNICATION_SUMMARY_BUILD_UNITS_PRODUCTION_PACKAGES,
      workspaceDependencies: COMMUNICATION_SUMMARY_BUILD_UNITS_WORKSPACE_DEPENDENCY_ALLOWLIST,
      thirdPartyDependencies: COMMUNICATION_SUMMARY_BUILD_UNITS_THIRD_PARTY_DEPENDENCY_ALLOWLIST,
      forbiddenDependencyPrefixes: STORAGE_FOUNDATION_FORBIDDEN_DEPENDENCY_PREFIXES,
    };
  }
  if (currentSlice === 'communication_translation_contract_core_v1') {
    return {
      profile: FIRST_OWNER_PROFILE,
      ownerInventory: COMMUNICATION_TRANSLATION_CONTRACT_CORE_INVENTORY,
      cargoFeatures: MAIL_OUTBOUND_MIME_ATTACHMENTS_CARGO_FEATURE_ALLOWLIST,
      packages: COMMUNICATION_TRANSLATION_CONTRACT_CORE_PRODUCTION_PACKAGES,
      workspaceDependencies: COMMUNICATION_TRANSLATION_CONTRACT_CORE_WORKSPACE_DEPENDENCY_ALLOWLIST,
      thirdPartyDependencies: COMMUNICATION_TRANSLATION_CONTRACT_CORE_THIRD_PARTY_DEPENDENCY_ALLOWLIST,
      forbiddenDependencyPrefixes: STORAGE_FOUNDATION_FORBIDDEN_DEPENDENCY_PREFIXES,
    };
  }
  if (currentSlice === 'communication_translation_cross_owner_contracts_v1') {
    return {
      profile: FIRST_OWNER_PROFILE,
      ownerInventory: COMMUNICATION_TRANSLATION_CROSS_OWNER_CONTRACTS_INVENTORY,
      cargoFeatures: MAIL_OUTBOUND_MIME_ATTACHMENTS_CARGO_FEATURE_ALLOWLIST,
      packages: COMMUNICATION_TRANSLATION_CONTRACT_CORE_PRODUCTION_PACKAGES,
      workspaceDependencies: COMMUNICATION_TRANSLATION_CONTRACT_CORE_WORKSPACE_DEPENDENCY_ALLOWLIST,
      thirdPartyDependencies: COMMUNICATION_TRANSLATION_CONTRACT_CORE_THIRD_PARTY_DEPENDENCY_ALLOWLIST,
      forbiddenDependencyPrefixes: STORAGE_FOUNDATION_FORBIDDEN_DEPENDENCY_PREFIXES,
    };
  }
  if (currentSlice === 'communication_translation_persistence_v1') {
    return {
      profile: FIRST_OWNER_PROFILE,
      ownerInventory: COMMUNICATION_TRANSLATION_PERSISTENCE_INVENTORY,
      cargoFeatures: MAIL_OUTBOUND_MIME_ATTACHMENTS_CARGO_FEATURE_ALLOWLIST,
      packages: COMMUNICATION_TRANSLATION_PERSISTENCE_PRODUCTION_PACKAGES,
      workspaceDependencies: COMMUNICATION_TRANSLATION_PERSISTENCE_WORKSPACE_DEPENDENCY_ALLOWLIST,
      thirdPartyDependencies: COMMUNICATION_TRANSLATION_PERSISTENCE_THIRD_PARTY_DEPENDENCY_ALLOWLIST,
      forbiddenDependencyPrefixes: STORAGE_FOUNDATION_FORBIDDEN_DEPENDENCY_PREFIXES,
    };
  }
  if (currentSlice === 'communication_translation_managed_runtime_v1') {
    return {
      profile: FIRST_OWNER_PROFILE,
      ownerInventory: COMMUNICATION_TRANSLATION_RUNTIME_INVENTORY,
      cargoFeatures: MAIL_OUTBOUND_MIME_ATTACHMENTS_CARGO_FEATURE_ALLOWLIST,
      packages: COMMUNICATION_TRANSLATION_RUNTIME_PRODUCTION_PACKAGES,
      workspaceDependencies: COMMUNICATION_TRANSLATION_RUNTIME_WORKSPACE_DEPENDENCY_ALLOWLIST,
      thirdPartyDependencies: COMMUNICATION_TRANSLATION_RUNTIME_THIRD_PARTY_DEPENDENCY_ALLOWLIST,
      forbiddenDependencyPrefixes: STORAGE_FOUNDATION_FORBIDDEN_DEPENDENCY_PREFIXES,
    };
  }
  if (currentSlice === 'communication_translation_ai_runtime_v1') {
    return {
      profile: FIRST_OWNER_PROFILE,
      ownerInventory: COMMUNICATION_TRANSLATION_RUNTIME_INVENTORY,
      cargoFeatures: MAIL_OUTBOUND_MIME_ATTACHMENTS_CARGO_FEATURE_ALLOWLIST,
      packages: COMMUNICATION_TRANSLATION_RUNTIME_PRODUCTION_PACKAGES,
      workspaceDependencies: COMMUNICATION_TRANSLATION_RUNTIME_WORKSPACE_DEPENDENCY_ALLOWLIST,
      thirdPartyDependencies: COMMUNICATION_TRANSLATION_RUNTIME_THIRD_PARTY_DEPENDENCY_ALLOWLIST,
      forbiddenDependencyPrefixes: STORAGE_FOUNDATION_FORBIDDEN_DEPENDENCY_PREFIXES,
    };
  }
  if (currentSlice === 'communication_translation_ollama_runtime_v1') {
    return {
      profile: FIRST_OWNER_PROFILE,
      ownerInventory: COMMUNICATION_TRANSLATION_RUNTIME_INVENTORY,
      cargoFeatures: MAIL_OUTBOUND_MIME_ATTACHMENTS_CARGO_FEATURE_ALLOWLIST,
      packages: COMMUNICATION_TRANSLATION_RUNTIME_PRODUCTION_PACKAGES,
      workspaceDependencies: COMMUNICATION_TRANSLATION_RUNTIME_WORKSPACE_DEPENDENCY_ALLOWLIST,
      thirdPartyDependencies: COMMUNICATION_TRANSLATION_RUNTIME_THIRD_PARTY_DEPENDENCY_ALLOWLIST,
      forbiddenDependencyPrefixes: STORAGE_FOUNDATION_FORBIDDEN_DEPENDENCY_PREFIXES,
    };
  }
  if (currentSlice === 'communication_translation_v1') {
    return {
      profile: FIRST_OWNER_PROFILE,
      ownerInventory: COMMUNICATION_TRANSLATION_RUNTIME_INVENTORY,
      cargoFeatures: MAIL_OUTBOUND_MIME_ATTACHMENTS_CARGO_FEATURE_ALLOWLIST,
      packages: COMMUNICATION_TRANSLATION_ASSEMBLY_PRODUCTION_PACKAGES,
      workspaceDependencies: COMMUNICATION_TRANSLATION_ASSEMBLY_WORKSPACE_DEPENDENCY_ALLOWLIST,
      thirdPartyDependencies: COMMUNICATION_TRANSLATION_ASSEMBLY_THIRD_PARTY_DEPENDENCY_ALLOWLIST,
      forbiddenDependencyPrefixes: STORAGE_FOUNDATION_FORBIDDEN_DEPENDENCY_PREFIXES,
    };
  }
  if (currentSlice === 'communication_explanation_contract_core_v1') {
    return {
      profile: FIRST_OWNER_PROFILE,
      ownerInventory: COMMUNICATION_EXPLANATION_CONTRACT_CORE_INVENTORY,
      cargoFeatures: MAIL_OUTBOUND_MIME_ATTACHMENTS_CARGO_FEATURE_ALLOWLIST,
      packages: COMMUNICATION_EXPLANATION_CONTRACT_CORE_PRODUCTION_PACKAGES,
      workspaceDependencies: COMMUNICATION_EXPLANATION_CONTRACT_CORE_WORKSPACE_DEPENDENCY_ALLOWLIST,
      thirdPartyDependencies: COMMUNICATION_EXPLANATION_CONTRACT_CORE_THIRD_PARTY_DEPENDENCY_ALLOWLIST,
      forbiddenDependencyPrefixes: STORAGE_FOUNDATION_FORBIDDEN_DEPENDENCY_PREFIXES,
    };
  }
  if (currentSlice === 'communication_explanation_cross_owner_contracts_v1') {
    return {
      profile: FIRST_OWNER_PROFILE,
      ownerInventory: COMMUNICATION_EXPLANATION_CROSS_OWNER_CONTRACTS_INVENTORY,
      cargoFeatures: MAIL_OUTBOUND_MIME_ATTACHMENTS_CARGO_FEATURE_ALLOWLIST,
      packages: COMMUNICATION_EXPLANATION_CONTRACT_CORE_PRODUCTION_PACKAGES,
      workspaceDependencies: COMMUNICATION_EXPLANATION_CONTRACT_CORE_WORKSPACE_DEPENDENCY_ALLOWLIST,
      thirdPartyDependencies: COMMUNICATION_EXPLANATION_CONTRACT_CORE_THIRD_PARTY_DEPENDENCY_ALLOWLIST,
      forbiddenDependencyPrefixes: STORAGE_FOUNDATION_FORBIDDEN_DEPENDENCY_PREFIXES,
    };
  }
  if (currentSlice === 'communication_explanation_persistence_v1') {
    return {
      profile: FIRST_OWNER_PROFILE,
      ownerInventory: COMMUNICATION_EXPLANATION_PERSISTENCE_INVENTORY,
      cargoFeatures: MAIL_OUTBOUND_MIME_ATTACHMENTS_CARGO_FEATURE_ALLOWLIST,
      packages: COMMUNICATION_EXPLANATION_PERSISTENCE_PRODUCTION_PACKAGES,
      workspaceDependencies: COMMUNICATION_EXPLANATION_PERSISTENCE_WORKSPACE_DEPENDENCY_ALLOWLIST,
      thirdPartyDependencies: COMMUNICATION_EXPLANATION_PERSISTENCE_THIRD_PARTY_DEPENDENCY_ALLOWLIST,
      forbiddenDependencyPrefixes: STORAGE_FOUNDATION_FORBIDDEN_DEPENDENCY_PREFIXES,
    };
  }
  if (currentSlice === 'communication_explanation_managed_runtime_v1') {
    return {
      profile: FIRST_OWNER_PROFILE,
      ownerInventory: COMMUNICATION_EXPLANATION_RUNTIME_INVENTORY,
      cargoFeatures: MAIL_OUTBOUND_MIME_ATTACHMENTS_CARGO_FEATURE_ALLOWLIST,
      packages: COMMUNICATION_EXPLANATION_RUNTIME_PRODUCTION_PACKAGES,
      workspaceDependencies: COMMUNICATION_EXPLANATION_RUNTIME_WORKSPACE_DEPENDENCY_ALLOWLIST,
      thirdPartyDependencies: COMMUNICATION_EXPLANATION_RUNTIME_THIRD_PARTY_DEPENDENCY_ALLOWLIST,
      forbiddenDependencyPrefixes: STORAGE_FOUNDATION_FORBIDDEN_DEPENDENCY_PREFIXES,
    };
  }
  if (currentSlice === 'communication_explanation_ai_runtime_v1') {
    return {
      profile: FIRST_OWNER_PROFILE,
      ownerInventory: COMMUNICATION_EXPLANATION_RUNTIME_INVENTORY,
      cargoFeatures: MAIL_OUTBOUND_MIME_ATTACHMENTS_CARGO_FEATURE_ALLOWLIST,
      packages: COMMUNICATION_EXPLANATION_RUNTIME_PRODUCTION_PACKAGES,
      workspaceDependencies: COMMUNICATION_EXPLANATION_RUNTIME_WORKSPACE_DEPENDENCY_ALLOWLIST,
      thirdPartyDependencies: COMMUNICATION_EXPLANATION_RUNTIME_THIRD_PARTY_DEPENDENCY_ALLOWLIST,
      forbiddenDependencyPrefixes: STORAGE_FOUNDATION_FORBIDDEN_DEPENDENCY_PREFIXES,
    };
  }
  if (currentSlice === 'communication_explanation_ollama_runtime_v1') {
    return {
      profile: FIRST_OWNER_PROFILE,
      ownerInventory: COMMUNICATION_EXPLANATION_RUNTIME_INVENTORY,
      cargoFeatures: MAIL_OUTBOUND_MIME_ATTACHMENTS_CARGO_FEATURE_ALLOWLIST,
      packages: COMMUNICATION_EXPLANATION_RUNTIME_PRODUCTION_PACKAGES,
      workspaceDependencies: COMMUNICATION_EXPLANATION_RUNTIME_WORKSPACE_DEPENDENCY_ALLOWLIST,
      thirdPartyDependencies: COMMUNICATION_EXPLANATION_RUNTIME_THIRD_PARTY_DEPENDENCY_ALLOWLIST,
      forbiddenDependencyPrefixes: STORAGE_FOUNDATION_FORBIDDEN_DEPENDENCY_PREFIXES,
    };
  }
  if (currentSlice === 'communication_explanation_assembly_v1') {
    return {
      profile: FIRST_OWNER_PROFILE,
      ownerInventory: COMMUNICATION_EXPLANATION_RUNTIME_INVENTORY,
      cargoFeatures: MAIL_OUTBOUND_MIME_ATTACHMENTS_CARGO_FEATURE_ALLOWLIST,
      packages: COMMUNICATION_EXPLANATION_ASSEMBLY_PRODUCTION_PACKAGES,
      workspaceDependencies: COMMUNICATION_EXPLANATION_ASSEMBLY_WORKSPACE_DEPENDENCY_ALLOWLIST,
      thirdPartyDependencies: COMMUNICATION_EXPLANATION_ASSEMBLY_THIRD_PARTY_DEPENDENCY_ALLOWLIST,
      forbiddenDependencyPrefixes: STORAGE_FOUNDATION_FORBIDDEN_DEPENDENCY_PREFIXES,
    };
  }
  if (currentSlice === 'communication_explanation_managed_conformance_v1') {
    return {
      profile: FIRST_OWNER_PROFILE,
      ownerInventory: COMMUNICATION_EXPLANATION_RUNTIME_INVENTORY,
      cargoFeatures: MAIL_OUTBOUND_MIME_ATTACHMENTS_CARGO_FEATURE_ALLOWLIST,
      packages: COMMUNICATION_EXPLANATION_ASSEMBLY_PRODUCTION_PACKAGES,
      workspaceDependencies: COMMUNICATION_EXPLANATION_ASSEMBLY_WORKSPACE_DEPENDENCY_ALLOWLIST,
      thirdPartyDependencies: COMMUNICATION_EXPLANATION_ASSEMBLY_THIRD_PARTY_DEPENDENCY_ALLOWLIST,
      forbiddenDependencyPrefixes: STORAGE_FOUNDATION_FORBIDDEN_DEPENDENCY_PREFIXES,
    };
  }
  if (currentSlice === 'communication_recipient_suggestion_contract_core_v1') {
    return {
      profile: FIRST_OWNER_PROFILE,
      ownerInventory: COMMUNICATION_RECIPIENT_SUGGESTION_CONTRACT_CORE_INVENTORY,
      cargoFeatures: MAIL_OUTBOUND_MIME_ATTACHMENTS_CARGO_FEATURE_ALLOWLIST,
      packages: COMMUNICATION_RECIPIENT_SUGGESTION_CONTRACT_CORE_PRODUCTION_PACKAGES,
      workspaceDependencies: COMMUNICATION_RECIPIENT_SUGGESTION_CONTRACT_CORE_WORKSPACE_DEPENDENCY_ALLOWLIST,
      thirdPartyDependencies: COMMUNICATION_RECIPIENT_SUGGESTION_CONTRACT_CORE_THIRD_PARTY_DEPENDENCY_ALLOWLIST,
      forbiddenDependencyPrefixes: STORAGE_FOUNDATION_FORBIDDEN_DEPENDENCY_PREFIXES,
    };
  }
  if (currentSlice === 'communication_recipient_suggestion_source_contract_v1') {
    return {
      profile: FIRST_OWNER_PROFILE,
      ownerInventory: COMMUNICATION_RECIPIENT_SUGGESTION_SOURCE_CONTRACT_INVENTORY,
      cargoFeatures: MAIL_OUTBOUND_MIME_ATTACHMENTS_CARGO_FEATURE_ALLOWLIST,
      packages: COMMUNICATION_RECIPIENT_SUGGESTION_SOURCE_CONTRACT_PRODUCTION_PACKAGES,
      workspaceDependencies: COMMUNICATION_RECIPIENT_SUGGESTION_SOURCE_CONTRACT_WORKSPACE_DEPENDENCY_ALLOWLIST,
      thirdPartyDependencies: COMMUNICATION_RECIPIENT_SUGGESTION_SOURCE_CONTRACT_THIRD_PARTY_DEPENDENCY_ALLOWLIST,
      forbiddenDependencyPrefixes: STORAGE_FOUNDATION_FORBIDDEN_DEPENDENCY_PREFIXES,
    };
  }
  if (currentSlice === 'communication_recipient_suggestion_persistence_v1') {
    return {
      profile: FIRST_OWNER_PROFILE,
      ownerInventory: COMMUNICATION_RECIPIENT_SUGGESTION_PERSISTENCE_INVENTORY,
      cargoFeatures: MAIL_OUTBOUND_MIME_ATTACHMENTS_CARGO_FEATURE_ALLOWLIST,
      packages: COMMUNICATION_RECIPIENT_SUGGESTION_PERSISTENCE_PRODUCTION_PACKAGES,
      workspaceDependencies: COMMUNICATION_RECIPIENT_SUGGESTION_PERSISTENCE_WORKSPACE_DEPENDENCY_ALLOWLIST,
      thirdPartyDependencies: COMMUNICATION_RECIPIENT_SUGGESTION_PERSISTENCE_THIRD_PARTY_DEPENDENCY_ALLOWLIST,
      forbiddenDependencyPrefixes: STORAGE_FOUNDATION_FORBIDDEN_DEPENDENCY_PREFIXES,
    };
  }
  if (currentSlice === 'communication_recipient_suggestion_managed_runtime_v1') {
    return {
      profile: FIRST_OWNER_PROFILE,
      ownerInventory: COMMUNICATION_RECIPIENT_SUGGESTION_PERSISTENCE_INVENTORY,
      cargoFeatures: MAIL_OUTBOUND_MIME_ATTACHMENTS_CARGO_FEATURE_ALLOWLIST,
      packages: COMMUNICATION_RECIPIENT_SUGGESTION_RUNTIME_PRODUCTION_PACKAGES,
      workspaceDependencies: COMMUNICATION_RECIPIENT_SUGGESTION_RUNTIME_WORKSPACE_DEPENDENCY_ALLOWLIST,
      thirdPartyDependencies: COMMUNICATION_RECIPIENT_SUGGESTION_RUNTIME_THIRD_PARTY_DEPENDENCY_ALLOWLIST,
      forbiddenDependencyPrefixes: STORAGE_FOUNDATION_FORBIDDEN_DEPENDENCY_PREFIXES,
    };
  }
  if (currentSlice === 'communication_recipient_suggestion_source_producer_v1') {
    return {
      profile: FIRST_OWNER_PROFILE,
      ownerInventory: COMMUNICATION_RECIPIENT_SUGGESTION_SOURCE_PRODUCER_INVENTORY,
      cargoFeatures: MAIL_OUTBOUND_MIME_ATTACHMENTS_CARGO_FEATURE_ALLOWLIST,
      packages: COMMUNICATION_RECIPIENT_SUGGESTION_RUNTIME_PRODUCTION_PACKAGES,
      workspaceDependencies: COMMUNICATION_RECIPIENT_SUGGESTION_SOURCE_PRODUCER_WORKSPACE_DEPENDENCY_ALLOWLIST,
      thirdPartyDependencies: COMMUNICATION_RECIPIENT_SUGGESTION_RUNTIME_THIRD_PARTY_DEPENDENCY_ALLOWLIST,
      forbiddenDependencyPrefixes: STORAGE_FOUNDATION_FORBIDDEN_DEPENDENCY_PREFIXES,
    };
  }
  if (currentSlice === 'communication_recipient_suggestion_assembly_v1') {
    return {
      profile: FIRST_OWNER_PROFILE,
      ownerInventory: COMMUNICATION_RECIPIENT_SUGGESTION_SOURCE_PRODUCER_INVENTORY,
      cargoFeatures: MAIL_OUTBOUND_MIME_ATTACHMENTS_CARGO_FEATURE_ALLOWLIST,
      packages: COMMUNICATION_RECIPIENT_SUGGESTION_ASSEMBLY_PRODUCTION_PACKAGES,
      workspaceDependencies: COMMUNICATION_RECIPIENT_SUGGESTION_ASSEMBLY_WORKSPACE_DEPENDENCY_ALLOWLIST,
      thirdPartyDependencies: COMMUNICATION_RECIPIENT_SUGGESTION_ASSEMBLY_THIRD_PARTY_DEPENDENCY_ALLOWLIST,
      forbiddenDependencyPrefixes: STORAGE_FOUNDATION_FORBIDDEN_DEPENDENCY_PREFIXES,
    };
  }
  if (currentSlice === 'communication_recipient_suggestion_managed_conformance_v1') {
    return {
      profile: FIRST_OWNER_PROFILE,
      ownerInventory: COMMUNICATION_RECIPIENT_SUGGESTION_SOURCE_PRODUCER_INVENTORY,
      cargoFeatures: MAIL_OUTBOUND_MIME_ATTACHMENTS_CARGO_FEATURE_ALLOWLIST,
      packages: COMMUNICATION_RECIPIENT_SUGGESTION_ASSEMBLY_PRODUCTION_PACKAGES,
      workspaceDependencies: COMMUNICATION_RECIPIENT_SUGGESTION_ASSEMBLY_WORKSPACE_DEPENDENCY_ALLOWLIST,
      thirdPartyDependencies: COMMUNICATION_RECIPIENT_SUGGESTION_ASSEMBLY_THIRD_PARTY_DEPENDENCY_ALLOWLIST,
      forbiddenDependencyPrefixes: STORAGE_FOUNDATION_FORBIDDEN_DEPENDENCY_PREFIXES,
    };
  }
  if (currentSlice === 'communication_explanation_live_provider_conformance_v1') {
    return {
      profile: FIRST_OWNER_PROFILE,
      ownerInventory: COMMUNICATION_RECIPIENT_SUGGESTION_SOURCE_PRODUCER_INVENTORY,
      cargoFeatures: MAIL_OUTBOUND_MIME_ATTACHMENTS_CARGO_FEATURE_ALLOWLIST,
      packages: COMMUNICATION_RECIPIENT_SUGGESTION_ASSEMBLY_PRODUCTION_PACKAGES,
      workspaceDependencies: COMMUNICATION_RECIPIENT_SUGGESTION_ASSEMBLY_WORKSPACE_DEPENDENCY_ALLOWLIST,
      thirdPartyDependencies: COMMUNICATION_RECIPIENT_SUGGESTION_ASSEMBLY_THIRD_PARTY_DEPENDENCY_ALLOWLIST,
      forbiddenDependencyPrefixes: STORAGE_FOUNDATION_FORBIDDEN_DEPENDENCY_PREFIXES,
    };
  }
  if (currentSlice === 'communication_task_candidate_contract_core_source_v1') {
    return {
      profile: FIRST_OWNER_PROFILE,
      ownerInventory: COMMUNICATION_TASK_CANDIDATE_CONTRACT_CORE_SOURCE_INVENTORY,
      cargoFeatures: MAIL_OUTBOUND_MIME_ATTACHMENTS_CARGO_FEATURE_ALLOWLIST,
      packages: COMMUNICATION_TASK_CANDIDATE_CONTRACT_CORE_SOURCE_PRODUCTION_PACKAGES,
      workspaceDependencies: COMMUNICATION_TASK_CANDIDATE_CONTRACT_CORE_SOURCE_WORKSPACE_DEPENDENCY_ALLOWLIST,
      thirdPartyDependencies: COMMUNICATION_TASK_CANDIDATE_CONTRACT_CORE_SOURCE_THIRD_PARTY_DEPENDENCY_ALLOWLIST,
      forbiddenDependencyPrefixes: STORAGE_FOUNDATION_FORBIDDEN_DEPENDENCY_PREFIXES,
    };
  }
  if (currentSlice === 'communication_task_candidate_persistence_v1') {
    return {
      profile: FIRST_OWNER_PROFILE,
      ownerInventory: COMMUNICATION_TASK_CANDIDATE_PERSISTENCE_INVENTORY,
      cargoFeatures: MAIL_OUTBOUND_MIME_ATTACHMENTS_CARGO_FEATURE_ALLOWLIST,
      packages: COMMUNICATION_TASK_CANDIDATE_PERSISTENCE_PRODUCTION_PACKAGES,
      workspaceDependencies: COMMUNICATION_TASK_CANDIDATE_PERSISTENCE_WORKSPACE_DEPENDENCY_ALLOWLIST,
      thirdPartyDependencies: COMMUNICATION_TASK_CANDIDATE_PERSISTENCE_THIRD_PARTY_DEPENDENCY_ALLOWLIST,
      forbiddenDependencyPrefixes: STORAGE_FOUNDATION_FORBIDDEN_DEPENDENCY_PREFIXES,
    };
  }
  if (currentSlice === 'communication_task_candidate_runtime_v1') {
    return {
      profile: FIRST_OWNER_PROFILE,
      ownerInventory: COMMUNICATION_TASK_CANDIDATE_PERSISTENCE_INVENTORY,
      cargoFeatures: MAIL_OUTBOUND_MIME_ATTACHMENTS_CARGO_FEATURE_ALLOWLIST,
      packages: COMMUNICATION_TASK_CANDIDATE_RUNTIME_PRODUCTION_PACKAGES,
      workspaceDependencies: COMMUNICATION_TASK_CANDIDATE_RUNTIME_WORKSPACE_DEPENDENCY_ALLOWLIST,
      thirdPartyDependencies: COMMUNICATION_TASK_CANDIDATE_RUNTIME_THIRD_PARTY_DEPENDENCY_ALLOWLIST,
      forbiddenDependencyPrefixes: STORAGE_FOUNDATION_FORBIDDEN_DEPENDENCY_PREFIXES,
    };
  }
  if (currentSlice === 'communication_task_candidate_source_producer_v1') {
    return {
      profile: FIRST_OWNER_PROFILE,
      ownerInventory: COMMUNICATION_TASK_CANDIDATE_SOURCE_PRODUCER_INVENTORY,
      cargoFeatures: MAIL_OUTBOUND_MIME_ATTACHMENTS_CARGO_FEATURE_ALLOWLIST,
      packages: COMMUNICATION_TASK_CANDIDATE_RUNTIME_PRODUCTION_PACKAGES,
      workspaceDependencies: COMMUNICATION_TASK_CANDIDATE_SOURCE_PRODUCER_WORKSPACE_DEPENDENCY_ALLOWLIST,
      thirdPartyDependencies: COMMUNICATION_TASK_CANDIDATE_RUNTIME_THIRD_PARTY_DEPENDENCY_ALLOWLIST,
      forbiddenDependencyPrefixes: STORAGE_FOUNDATION_FORBIDDEN_DEPENDENCY_PREFIXES,
    };
  }
  if (currentSlice === 'communication_task_candidate_assembly_v1') {
    return {
      profile: FIRST_OWNER_PROFILE,
      ownerInventory: COMMUNICATION_TASK_CANDIDATE_SOURCE_PRODUCER_INVENTORY,
      cargoFeatures: MAIL_OUTBOUND_MIME_ATTACHMENTS_CARGO_FEATURE_ALLOWLIST,
      packages: COMMUNICATION_TASK_CANDIDATE_ASSEMBLY_PRODUCTION_PACKAGES,
      workspaceDependencies: COMMUNICATION_TASK_CANDIDATE_ASSEMBLY_WORKSPACE_DEPENDENCY_ALLOWLIST,
      thirdPartyDependencies: COMMUNICATION_TASK_CANDIDATE_ASSEMBLY_THIRD_PARTY_DEPENDENCY_ALLOWLIST,
      forbiddenDependencyPrefixes: STORAGE_FOUNDATION_FORBIDDEN_DEPENDENCY_PREFIXES,
    };
  }
  if (currentSlice === 'review_task_candidate_core_v1') {
    return {
      profile: FIRST_OWNER_PROFILE,
      ownerInventory: REVIEW_TASK_CANDIDATE_CORE_INVENTORY,
      cargoFeatures: MAIL_OUTBOUND_MIME_ATTACHMENTS_CARGO_FEATURE_ALLOWLIST,
      packages: REVIEW_TASK_CANDIDATE_CORE_PRODUCTION_PACKAGES,
      workspaceDependencies: REVIEW_TASK_CANDIDATE_CORE_WORKSPACE_DEPENDENCY_ALLOWLIST,
      thirdPartyDependencies: REVIEW_TASK_CANDIDATE_CORE_THIRD_PARTY_DEPENDENCY_ALLOWLIST,
      forbiddenDependencyPrefixes: STORAGE_FOUNDATION_FORBIDDEN_DEPENDENCY_PREFIXES,
    };
  }
  if (currentSlice === 'review_task_candidate_persistence_v1') {
    return {
      profile: FIRST_OWNER_PROFILE,
      ownerInventory: REVIEW_TASK_CANDIDATE_PERSISTENCE_INVENTORY,
      cargoFeatures: MAIL_OUTBOUND_MIME_ATTACHMENTS_CARGO_FEATURE_ALLOWLIST,
      packages: REVIEW_TASK_CANDIDATE_PERSISTENCE_PRODUCTION_PACKAGES,
      workspaceDependencies: REVIEW_TASK_CANDIDATE_PERSISTENCE_WORKSPACE_DEPENDENCY_ALLOWLIST,
      thirdPartyDependencies: REVIEW_TASK_CANDIDATE_PERSISTENCE_THIRD_PARTY_DEPENDENCY_ALLOWLIST,
      forbiddenDependencyPrefixes: STORAGE_FOUNDATION_FORBIDDEN_DEPENDENCY_PREFIXES,
    };
  }
  if (currentSlice === 'authenticated_client_device_context_v1') {
    return {
      profile: FIRST_OWNER_PROFILE,
      ownerInventory: REVIEW_TASK_CANDIDATE_PERSISTENCE_INVENTORY,
      cargoFeatures: MAIL_OUTBOUND_MIME_ATTACHMENTS_CARGO_FEATURE_ALLOWLIST,
      packages: REVIEW_TASK_CANDIDATE_PERSISTENCE_PRODUCTION_PACKAGES,
      workspaceDependencies: REVIEW_TASK_CANDIDATE_PERSISTENCE_WORKSPACE_DEPENDENCY_ALLOWLIST,
      thirdPartyDependencies: REVIEW_TASK_CANDIDATE_PERSISTENCE_THIRD_PARTY_DEPENDENCY_ALLOWLIST,
      forbiddenDependencyPrefixes: STORAGE_FOUNDATION_FORBIDDEN_DEPENDENCY_PREFIXES,
    };
  }
  if (currentSlice === 'review_task_candidate_event_contracts_v1') {
    return {
      profile: FIRST_OWNER_PROFILE,
      ownerInventory: REVIEW_TASK_CANDIDATE_PERSISTENCE_INVENTORY,
      cargoFeatures: MAIL_OUTBOUND_MIME_ATTACHMENTS_CARGO_FEATURE_ALLOWLIST,
      packages: REVIEW_TASK_CANDIDATE_PERSISTENCE_PRODUCTION_PACKAGES,
      workspaceDependencies: REVIEW_TASK_CANDIDATE_PERSISTENCE_WORKSPACE_DEPENDENCY_ALLOWLIST,
      thirdPartyDependencies: REVIEW_TASK_CANDIDATE_PERSISTENCE_THIRD_PARTY_DEPENDENCY_ALLOWLIST,
      forbiddenDependencyPrefixes: STORAGE_FOUNDATION_FORBIDDEN_DEPENDENCY_PREFIXES,
    };
  }
  if (currentSlice === 'review_task_candidate_managed_runtime_v1') {
    return {
      profile: FIRST_OWNER_PROFILE,
      ownerInventory: REVIEW_TASK_CANDIDATE_PERSISTENCE_INVENTORY,
      cargoFeatures: MAIL_OUTBOUND_MIME_ATTACHMENTS_CARGO_FEATURE_ALLOWLIST,
      packages: REVIEW_TASK_CANDIDATE_MANAGED_RUNTIME_PRODUCTION_PACKAGES,
      workspaceDependencies: REVIEW_TASK_CANDIDATE_MANAGED_RUNTIME_WORKSPACE_DEPENDENCY_ALLOWLIST,
      thirdPartyDependencies: REVIEW_TASK_CANDIDATE_MANAGED_RUNTIME_THIRD_PARTY_DEPENDENCY_ALLOWLIST,
      forbiddenDependencyPrefixes: STORAGE_FOUNDATION_FORBIDDEN_DEPENDENCY_PREFIXES,
    };
  }
  if (currentSlice === 'review_task_candidate_assembly_v1') {
    return {
      profile: FIRST_OWNER_PROFILE,
      ownerInventory: REVIEW_TASK_CANDIDATE_PERSISTENCE_INVENTORY,
      cargoFeatures: MAIL_OUTBOUND_MIME_ATTACHMENTS_CARGO_FEATURE_ALLOWLIST,
      packages: REVIEW_TASK_CANDIDATE_ASSEMBLY_PRODUCTION_PACKAGES,
      workspaceDependencies: REVIEW_TASK_CANDIDATE_ASSEMBLY_WORKSPACE_DEPENDENCY_ALLOWLIST,
      thirdPartyDependencies: REVIEW_TASK_CANDIDATE_ASSEMBLY_THIRD_PARTY_DEPENDENCY_ALLOWLIST,
      forbiddenDependencyPrefixes: STORAGE_FOUNDATION_FORBIDDEN_DEPENDENCY_PREFIXES,
    };
  }
  if (currentSlice === 'tasks_reviewed_candidate_contract_core_v1') {
    return {
      profile: FIRST_OWNER_PROFILE,
      ownerInventory: TASKS_REVIEWED_CANDIDATE_CONTRACT_CORE_INVENTORY,
      cargoFeatures: MAIL_OUTBOUND_MIME_ATTACHMENTS_CARGO_FEATURE_ALLOWLIST,
      packages: TASKS_REVIEWED_CANDIDATE_CONTRACT_CORE_PRODUCTION_PACKAGES,
      workspaceDependencies: TASKS_REVIEWED_CANDIDATE_CONTRACT_CORE_WORKSPACE_DEPENDENCY_ALLOWLIST,
      thirdPartyDependencies: TASKS_REVIEWED_CANDIDATE_CONTRACT_CORE_THIRD_PARTY_DEPENDENCY_ALLOWLIST,
      forbiddenDependencyPrefixes: STORAGE_FOUNDATION_FORBIDDEN_DEPENDENCY_PREFIXES,
    };
  }
  if (currentSlice === 'tasks_reviewed_candidate_persistence_v1') {
    return {
      profile: FIRST_OWNER_PROFILE,
      ownerInventory: TASKS_REVIEWED_CANDIDATE_PERSISTENCE_INVENTORY,
      cargoFeatures: MAIL_OUTBOUND_MIME_ATTACHMENTS_CARGO_FEATURE_ALLOWLIST,
      packages: TASKS_REVIEWED_CANDIDATE_PERSISTENCE_PRODUCTION_PACKAGES,
      workspaceDependencies: TASKS_REVIEWED_CANDIDATE_PERSISTENCE_WORKSPACE_DEPENDENCY_ALLOWLIST,
      thirdPartyDependencies: TASKS_REVIEWED_CANDIDATE_PERSISTENCE_THIRD_PARTY_DEPENDENCY_ALLOWLIST,
      forbiddenDependencyPrefixes: STORAGE_FOUNDATION_FORBIDDEN_DEPENDENCY_PREFIXES,
    };
  }
  if (currentSlice === 'tasks_reviewed_candidate_managed_runtime_v1') {
    return {
      profile: FIRST_OWNER_PROFILE,
      ownerInventory: TASKS_REVIEWED_CANDIDATE_PERSISTENCE_INVENTORY,
      cargoFeatures: MAIL_OUTBOUND_MIME_ATTACHMENTS_CARGO_FEATURE_ALLOWLIST,
      packages: TASKS_REVIEWED_CANDIDATE_MANAGED_RUNTIME_PRODUCTION_PACKAGES,
      workspaceDependencies: TASKS_REVIEWED_CANDIDATE_MANAGED_RUNTIME_WORKSPACE_DEPENDENCY_ALLOWLIST,
      thirdPartyDependencies: TASKS_REVIEWED_CANDIDATE_MANAGED_RUNTIME_THIRD_PARTY_DEPENDENCY_ALLOWLIST,
      forbiddenDependencyPrefixes: STORAGE_FOUNDATION_FORBIDDEN_DEPENDENCY_PREFIXES,
    };
  }
  if (currentSlice === 'tasks_reviewed_candidate_assembly_v1') {
    return {
      profile: FIRST_OWNER_PROFILE,
      ownerInventory: TASKS_REVIEWED_CANDIDATE_PERSISTENCE_INVENTORY,
      cargoFeatures: MAIL_OUTBOUND_MIME_ATTACHMENTS_CARGO_FEATURE_ALLOWLIST,
      packages: TASKS_REVIEWED_CANDIDATE_ASSEMBLY_PRODUCTION_PACKAGES,
      workspaceDependencies: TASKS_REVIEWED_CANDIDATE_ASSEMBLY_WORKSPACE_DEPENDENCY_ALLOWLIST,
      thirdPartyDependencies: TASKS_REVIEWED_CANDIDATE_ASSEMBLY_THIRD_PARTY_DEPENDENCY_ALLOWLIST,
      forbiddenDependencyPrefixes: STORAGE_FOUNDATION_FORBIDDEN_DEPENDENCY_PREFIXES,
    };
  }
  if (currentSlice === 'communication_task_candidate_managed_admission_v1') {
    return {
      profile: FIRST_OWNER_PROFILE,
      ownerInventory: TASKS_REVIEWED_CANDIDATE_PERSISTENCE_INVENTORY,
      cargoFeatures: MAIL_OUTBOUND_MIME_ATTACHMENTS_CARGO_FEATURE_ALLOWLIST,
      packages: TASKS_REVIEWED_CANDIDATE_ASSEMBLY_PRODUCTION_PACKAGES,
      workspaceDependencies: TASKS_REVIEWED_CANDIDATE_ASSEMBLY_WORKSPACE_DEPENDENCY_ALLOWLIST,
      thirdPartyDependencies: TASKS_REVIEWED_CANDIDATE_ASSEMBLY_THIRD_PARTY_DEPENDENCY_ALLOWLIST,
      forbiddenDependencyPrefixes: STORAGE_FOUNDATION_FORBIDDEN_DEPENDENCY_PREFIXES,
    };
  }
  if (currentSlice === 'reviewed_task_candidate_promotion_contract_core_v1') {
    return {
      profile: FIRST_OWNER_PROFILE,
      ownerInventory: REVIEWED_TASK_CANDIDATE_PROMOTION_CONTRACT_CORE_INVENTORY,
      cargoFeatures: MAIL_OUTBOUND_MIME_ATTACHMENTS_CARGO_FEATURE_ALLOWLIST,
      packages: REVIEWED_TASK_CANDIDATE_PROMOTION_CONTRACT_CORE_PRODUCTION_PACKAGES,
      workspaceDependencies: REVIEWED_TASK_CANDIDATE_PROMOTION_CONTRACT_CORE_WORKSPACE_DEPENDENCY_ALLOWLIST,
      thirdPartyDependencies: REVIEWED_TASK_CANDIDATE_PROMOTION_CONTRACT_CORE_THIRD_PARTY_DEPENDENCY_ALLOWLIST,
      forbiddenDependencyPrefixes: STORAGE_FOUNDATION_FORBIDDEN_DEPENDENCY_PREFIXES,
    };
  }
  if (currentSlice === 'reviewed_task_candidate_promotion_persistence_v1') {
    return {
      profile: FIRST_OWNER_PROFILE,
      ownerInventory: REVIEWED_TASK_CANDIDATE_PROMOTION_PERSISTENCE_INVENTORY,
      cargoFeatures: MAIL_OUTBOUND_MIME_ATTACHMENTS_CARGO_FEATURE_ALLOWLIST,
      packages: REVIEWED_TASK_CANDIDATE_PROMOTION_PERSISTENCE_PRODUCTION_PACKAGES,
      workspaceDependencies: REVIEWED_TASK_CANDIDATE_PROMOTION_PERSISTENCE_WORKSPACE_DEPENDENCY_ALLOWLIST,
      thirdPartyDependencies: REVIEWED_TASK_CANDIDATE_PROMOTION_PERSISTENCE_THIRD_PARTY_DEPENDENCY_ALLOWLIST,
      forbiddenDependencyPrefixes: STORAGE_FOUNDATION_FORBIDDEN_DEPENDENCY_PREFIXES,
    };
  }
  if (currentSlice === 'reviewed_task_candidate_promotion_runtime_v1') {
    return {
      profile: FIRST_OWNER_PROFILE,
      ownerInventory: REVIEWED_TASK_CANDIDATE_PROMOTION_RUNTIME_INVENTORY,
      cargoFeatures: MAIL_OUTBOUND_MIME_ATTACHMENTS_CARGO_FEATURE_ALLOWLIST,
      packages: REVIEWED_TASK_CANDIDATE_PROMOTION_RUNTIME_PRODUCTION_PACKAGES,
      workspaceDependencies: REVIEWED_TASK_CANDIDATE_PROMOTION_RUNTIME_WORKSPACE_DEPENDENCY_ALLOWLIST,
      thirdPartyDependencies: REVIEWED_TASK_CANDIDATE_PROMOTION_RUNTIME_THIRD_PARTY_DEPENDENCY_ALLOWLIST,
      forbiddenDependencyPrefixes: STORAGE_FOUNDATION_FORBIDDEN_DEPENDENCY_PREFIXES,
    };
  }
  if (currentSlice === 'review_task_candidate_promotion_result_consumer_v1') {
    return {
      profile: FIRST_OWNER_PROFILE,
      ownerInventory: REVIEW_TASK_CANDIDATE_PROMOTION_RESULT_CONSUMER_INVENTORY,
      cargoFeatures: MAIL_OUTBOUND_MIME_ATTACHMENTS_CARGO_FEATURE_ALLOWLIST,
      packages: REVIEWED_TASK_CANDIDATE_PROMOTION_RUNTIME_PRODUCTION_PACKAGES,
      workspaceDependencies: REVIEW_TASK_CANDIDATE_PROMOTION_RESULT_CONSUMER_WORKSPACE_DEPENDENCY_ALLOWLIST,
      thirdPartyDependencies: REVIEWED_TASK_CANDIDATE_PROMOTION_RUNTIME_THIRD_PARTY_DEPENDENCY_ALLOWLIST,
      forbiddenDependencyPrefixes: STORAGE_FOUNDATION_FORBIDDEN_DEPENDENCY_PREFIXES,
    };
  }
  if (currentSlice === 'reviewed_task_candidate_promotion_assembly_v1') {
    return {
      profile: FIRST_OWNER_PROFILE,
      ownerInventory: REVIEW_TASK_CANDIDATE_PROMOTION_RESULT_CONSUMER_INVENTORY,
      cargoFeatures: MAIL_OUTBOUND_MIME_ATTACHMENTS_CARGO_FEATURE_ALLOWLIST,
      packages: REVIEWED_TASK_CANDIDATE_PROMOTION_ASSEMBLY_PRODUCTION_PACKAGES,
      workspaceDependencies: REVIEWED_TASK_CANDIDATE_PROMOTION_ASSEMBLY_WORKSPACE_DEPENDENCY_ALLOWLIST,
      thirdPartyDependencies: REVIEWED_TASK_CANDIDATE_PROMOTION_ASSEMBLY_THIRD_PARTY_DEPENDENCY_ALLOWLIST,
      forbiddenDependencyPrefixes: STORAGE_FOUNDATION_FORBIDDEN_DEPENDENCY_PREFIXES,
    };
  }
  if (currentSlice === 'reviewed_task_candidate_promotion_gateway_sse_v1') {
    return {
      profile: FIRST_OWNER_PROFILE,
      ownerInventory: REVIEW_TASK_CANDIDATE_PROMOTION_RESULT_CONSUMER_INVENTORY,
      cargoFeatures: MAIL_OUTBOUND_MIME_ATTACHMENTS_CARGO_FEATURE_ALLOWLIST,
      packages: REVIEWED_TASK_CANDIDATE_PROMOTION_ASSEMBLY_PRODUCTION_PACKAGES,
      workspaceDependencies: REVIEWED_TASK_CANDIDATE_PROMOTION_ASSEMBLY_WORKSPACE_DEPENDENCY_ALLOWLIST,
      thirdPartyDependencies: REVIEWED_TASK_CANDIDATE_PROMOTION_ASSEMBLY_THIRD_PARTY_DEPENDENCY_ALLOWLIST,
      forbiddenDependencyPrefixes: STORAGE_FOUNDATION_FORBIDDEN_DEPENDENCY_PREFIXES,
    };
  }
  if (currentSlice === 'communication_note_candidate_contract_core_v1') {
    return {
      profile: FIRST_OWNER_PROFILE,
      ownerInventory: COMMUNICATION_NOTE_CANDIDATE_CONTRACT_CORE_INVENTORY,
      cargoFeatures: MAIL_OUTBOUND_MIME_ATTACHMENTS_CARGO_FEATURE_ALLOWLIST,
      packages: COMMUNICATION_NOTE_CANDIDATE_CONTRACT_CORE_PRODUCTION_PACKAGES,
      workspaceDependencies: COMMUNICATION_NOTE_CANDIDATE_CONTRACT_CORE_WORKSPACE_DEPENDENCY_ALLOWLIST,
      thirdPartyDependencies: COMMUNICATION_NOTE_CANDIDATE_CONTRACT_CORE_THIRD_PARTY_DEPENDENCY_ALLOWLIST,
      forbiddenDependencyPrefixes: STORAGE_FOUNDATION_FORBIDDEN_DEPENDENCY_PREFIXES,
    };
  }
  if (currentSlice === 'communication_note_candidate_persistence_v1') {
    return {
      profile: FIRST_OWNER_PROFILE,
      ownerInventory: COMMUNICATION_NOTE_CANDIDATE_PERSISTENCE_INVENTORY,
      cargoFeatures: MAIL_OUTBOUND_MIME_ATTACHMENTS_CARGO_FEATURE_ALLOWLIST,
      packages: COMMUNICATION_NOTE_CANDIDATE_PERSISTENCE_PRODUCTION_PACKAGES,
      workspaceDependencies: COMMUNICATION_NOTE_CANDIDATE_PERSISTENCE_WORKSPACE_DEPENDENCY_ALLOWLIST,
      thirdPartyDependencies: COMMUNICATION_NOTE_CANDIDATE_PERSISTENCE_THIRD_PARTY_DEPENDENCY_ALLOWLIST,
      forbiddenDependencyPrefixes: STORAGE_FOUNDATION_FORBIDDEN_DEPENDENCY_PREFIXES,
    };
  }
  if (currentSlice === 'review_note_candidate_contract_core_v1') {
    return {
      profile: FIRST_OWNER_PROFILE,
      ownerInventory: REVIEW_NOTE_CANDIDATE_CONTRACT_CORE_INVENTORY,
      cargoFeatures: MAIL_OUTBOUND_MIME_ATTACHMENTS_CARGO_FEATURE_ALLOWLIST,
      packages: REVIEW_NOTE_CANDIDATE_CONTRACT_CORE_PRODUCTION_PACKAGES,
      workspaceDependencies: REVIEW_NOTE_CANDIDATE_CONTRACT_CORE_WORKSPACE_DEPENDENCY_ALLOWLIST,
      thirdPartyDependencies: REVIEW_NOTE_CANDIDATE_CONTRACT_CORE_THIRD_PARTY_DEPENDENCY_ALLOWLIST,
      forbiddenDependencyPrefixes: STORAGE_FOUNDATION_FORBIDDEN_DEPENDENCY_PREFIXES,
    };
  }
  if (currentSlice === 'knowledge_verified_note_contract_core_v1') {
    return {
      profile: FIRST_OWNER_PROFILE,
      ownerInventory: KNOWLEDGE_VERIFIED_NOTE_CONTRACT_CORE_INVENTORY,
      cargoFeatures: MAIL_OUTBOUND_MIME_ATTACHMENTS_CARGO_FEATURE_ALLOWLIST,
      packages: KNOWLEDGE_VERIFIED_NOTE_CONTRACT_CORE_PRODUCTION_PACKAGES,
      workspaceDependencies: KNOWLEDGE_VERIFIED_NOTE_CONTRACT_CORE_WORKSPACE_DEPENDENCY_ALLOWLIST,
      thirdPartyDependencies: KNOWLEDGE_VERIFIED_NOTE_CONTRACT_CORE_THIRD_PARTY_DEPENDENCY_ALLOWLIST,
      forbiddenDependencyPrefixes: STORAGE_FOUNDATION_FORBIDDEN_DEPENDENCY_PREFIXES,
    };
  }
  if (currentSlice === 'knowledge_verified_note_persistence_v1') {
    return {
      profile: FIRST_OWNER_PROFILE,
      ownerInventory: KNOWLEDGE_VERIFIED_NOTE_CONTRACT_CORE_INVENTORY,
      cargoFeatures: MAIL_OUTBOUND_MIME_ATTACHMENTS_CARGO_FEATURE_ALLOWLIST,
      packages: KNOWLEDGE_VERIFIED_NOTE_PERSISTENCE_PRODUCTION_PACKAGES,
      workspaceDependencies: KNOWLEDGE_VERIFIED_NOTE_PERSISTENCE_WORKSPACE_DEPENDENCY_ALLOWLIST,
      thirdPartyDependencies: KNOWLEDGE_VERIFIED_NOTE_PERSISTENCE_THIRD_PARTY_DEPENDENCY_ALLOWLIST,
      forbiddenDependencyPrefixes: STORAGE_FOUNDATION_FORBIDDEN_DEPENDENCY_PREFIXES,
    };
  }
  if (currentSlice === 'knowledge_verified_note_managed_runtime_v1') {
    return {
      profile: FIRST_OWNER_PROFILE,
      ownerInventory: KNOWLEDGE_VERIFIED_NOTE_MANAGED_RUNTIME_INVENTORY,
      cargoFeatures: MAIL_OUTBOUND_MIME_ATTACHMENTS_CARGO_FEATURE_ALLOWLIST,
      packages: KNOWLEDGE_VERIFIED_NOTE_MANAGED_RUNTIME_PRODUCTION_PACKAGES,
      workspaceDependencies: KNOWLEDGE_VERIFIED_NOTE_MANAGED_RUNTIME_WORKSPACE_DEPENDENCY_ALLOWLIST,
      thirdPartyDependencies: KNOWLEDGE_VERIFIED_NOTE_MANAGED_RUNTIME_THIRD_PARTY_DEPENDENCY_ALLOWLIST,
      forbiddenDependencyPrefixes: STORAGE_FOUNDATION_FORBIDDEN_DEPENDENCY_PREFIXES,
    };
  }
  if (currentSlice === 'knowledge_verified_note_assembly_v1') {
    return {
      profile: FIRST_OWNER_PROFILE,
      ownerInventory: KNOWLEDGE_VERIFIED_NOTE_MANAGED_RUNTIME_INVENTORY,
      cargoFeatures: MAIL_OUTBOUND_MIME_ATTACHMENTS_CARGO_FEATURE_ALLOWLIST,
      packages: KNOWLEDGE_VERIFIED_NOTE_ASSEMBLY_PRODUCTION_PACKAGES,
      workspaceDependencies: KNOWLEDGE_VERIFIED_NOTE_ASSEMBLY_WORKSPACE_DEPENDENCY_ALLOWLIST,
      thirdPartyDependencies: KNOWLEDGE_VERIFIED_NOTE_ASSEMBLY_THIRD_PARTY_DEPENDENCY_ALLOWLIST,
      forbiddenDependencyPrefixes: STORAGE_FOUNDATION_FORBIDDEN_DEPENDENCY_PREFIXES,
    };
  }
  if (currentSlice === 'review_note_candidate_persistence_v1') {
    return {
      profile: FIRST_OWNER_PROFILE,
      ownerInventory: KNOWLEDGE_VERIFIED_NOTE_MANAGED_RUNTIME_INVENTORY,
      cargoFeatures: MAIL_OUTBOUND_MIME_ATTACHMENTS_CARGO_FEATURE_ALLOWLIST,
      packages: REVIEW_NOTE_CANDIDATE_PERSISTENCE_PRODUCTION_PACKAGES,
      workspaceDependencies: REVIEW_NOTE_CANDIDATE_PERSISTENCE_WORKSPACE_DEPENDENCY_ALLOWLIST,
      thirdPartyDependencies: REVIEW_NOTE_CANDIDATE_PERSISTENCE_THIRD_PARTY_DEPENDENCY_ALLOWLIST,
      forbiddenDependencyPrefixes: STORAGE_FOUNDATION_FORBIDDEN_DEPENDENCY_PREFIXES,
    };
  }
  if (currentSlice === 'review_note_candidate_managed_runtime_v1') {
    return {
      profile: FIRST_OWNER_PROFILE,
      ownerInventory: REVIEW_NOTE_CANDIDATE_MANAGED_RUNTIME_INVENTORY,
      cargoFeatures: MAIL_OUTBOUND_MIME_ATTACHMENTS_CARGO_FEATURE_ALLOWLIST,
      packages: REVIEW_NOTE_CANDIDATE_MANAGED_RUNTIME_PRODUCTION_PACKAGES,
      workspaceDependencies: REVIEW_NOTE_CANDIDATE_MANAGED_RUNTIME_WORKSPACE_DEPENDENCY_ALLOWLIST,
      thirdPartyDependencies: REVIEW_NOTE_CANDIDATE_MANAGED_RUNTIME_THIRD_PARTY_DEPENDENCY_ALLOWLIST,
      forbiddenDependencyPrefixes: STORAGE_FOUNDATION_FORBIDDEN_DEPENDENCY_PREFIXES,
    };
  }
  if (currentSlice === 'review_note_candidate_assembly_v1') {
    return {
      profile: FIRST_OWNER_PROFILE,
      ownerInventory: REVIEW_NOTE_CANDIDATE_MANAGED_RUNTIME_INVENTORY,
      cargoFeatures: MAIL_OUTBOUND_MIME_ATTACHMENTS_CARGO_FEATURE_ALLOWLIST,
      packages: REVIEW_NOTE_CANDIDATE_ASSEMBLY_PRODUCTION_PACKAGES,
      workspaceDependencies: REVIEW_NOTE_CANDIDATE_ASSEMBLY_WORKSPACE_DEPENDENCY_ALLOWLIST,
      thirdPartyDependencies: REVIEW_NOTE_CANDIDATE_ASSEMBLY_THIRD_PARTY_DEPENDENCY_ALLOWLIST,
      forbiddenDependencyPrefixes: STORAGE_FOUNDATION_FORBIDDEN_DEPENDENCY_PREFIXES,
    };
  }
  if (currentSlice === 'reviewed_note_candidate_promotion_assembly_v1') {
    return {
      profile: FIRST_OWNER_PROFILE,
      ownerInventory: REVIEWED_NOTE_CANDIDATE_PROMOTION_INVENTORY,
      cargoFeatures: MAIL_OUTBOUND_MIME_ATTACHMENTS_CARGO_FEATURE_ALLOWLIST,
      packages: REVIEWED_NOTE_CANDIDATE_PROMOTION_PRODUCTION_PACKAGES,
      workspaceDependencies: REVIEWED_NOTE_CANDIDATE_PROMOTION_WORKSPACE_DEPENDENCY_ALLOWLIST,
      thirdPartyDependencies: REVIEWED_NOTE_CANDIDATE_PROMOTION_THIRD_PARTY_DEPENDENCY_ALLOWLIST,
      forbiddenDependencyPrefixes: STORAGE_FOUNDATION_FORBIDDEN_DEPENDENCY_PREFIXES,
    };
  }
  if (currentSlice === 'communication_note_candidate_assembly_v1') {
    return {
      profile: FIRST_OWNER_PROFILE,
      ownerInventory: COMMUNICATION_NOTE_CANDIDATE_ASSEMBLY_INVENTORY,
      cargoFeatures: MAIL_OUTBOUND_MIME_ATTACHMENTS_CARGO_FEATURE_ALLOWLIST,
      packages: COMMUNICATION_NOTE_CANDIDATE_ASSEMBLY_PRODUCTION_PACKAGES,
      workspaceDependencies: COMMUNICATION_NOTE_CANDIDATE_ASSEMBLY_WORKSPACE_DEPENDENCY_ALLOWLIST,
      thirdPartyDependencies: COMMUNICATION_NOTE_CANDIDATE_ASSEMBLY_THIRD_PARTY_DEPENDENCY_ALLOWLIST,
      forbiddenDependencyPrefixes: STORAGE_FOUNDATION_FORBIDDEN_DEPENDENCY_PREFIXES,
    };
  }
  if (currentSlice === 'attachment_text_extraction_contract_core_v1') {
    return {
      profile: FIRST_OWNER_PROFILE,
      ownerInventory: ATTACHMENT_TEXT_EXTRACTION_CONTRACT_CORE_INVENTORY,
      cargoFeatures: MAIL_OUTBOUND_MIME_ATTACHMENTS_CARGO_FEATURE_ALLOWLIST,
      packages: ATTACHMENT_TEXT_EXTRACTION_CONTRACT_CORE_PRODUCTION_PACKAGES,
      workspaceDependencies: ATTACHMENT_TEXT_EXTRACTION_CONTRACT_CORE_WORKSPACE_DEPENDENCY_ALLOWLIST,
      thirdPartyDependencies: ATTACHMENT_TEXT_EXTRACTION_CONTRACT_CORE_THIRD_PARTY_DEPENDENCY_ALLOWLIST,
      forbiddenDependencyPrefixes: STORAGE_FOUNDATION_FORBIDDEN_DEPENDENCY_PREFIXES,
    };
  }
  if (currentSlice === 'attachment_text_extraction_parser_adapters_v1') {
    return {
      profile: FIRST_OWNER_PROFILE,
      ownerInventory: ATTACHMENT_TEXT_EXTRACTION_CONTRACT_CORE_INVENTORY,
      cargoFeatures: MAIL_OUTBOUND_MIME_ATTACHMENTS_CARGO_FEATURE_ALLOWLIST,
      packages: ATTACHMENT_TEXT_EXTRACTION_PARSER_ADAPTERS_PRODUCTION_PACKAGES,
      workspaceDependencies: ATTACHMENT_TEXT_EXTRACTION_PARSER_ADAPTERS_WORKSPACE_DEPENDENCY_ALLOWLIST,
      thirdPartyDependencies: ATTACHMENT_TEXT_EXTRACTION_PARSER_ADAPTERS_THIRD_PARTY_DEPENDENCY_ALLOWLIST,
      forbiddenDependencyPrefixes: STORAGE_FOUNDATION_FORBIDDEN_DEPENDENCY_PREFIXES,
    };
  }
  if (currentSlice === 'attachment_text_extraction_persistence_v1') {
    return {
      profile: FIRST_OWNER_PROFILE,
      ownerInventory: ATTACHMENT_TEXT_EXTRACTION_CONTRACT_CORE_INVENTORY,
      cargoFeatures: MAIL_OUTBOUND_MIME_ATTACHMENTS_CARGO_FEATURE_ALLOWLIST,
      packages: ATTACHMENT_TEXT_EXTRACTION_PERSISTENCE_PRODUCTION_PACKAGES,
      workspaceDependencies: ATTACHMENT_TEXT_EXTRACTION_PERSISTENCE_WORKSPACE_DEPENDENCY_ALLOWLIST,
      thirdPartyDependencies: ATTACHMENT_TEXT_EXTRACTION_PERSISTENCE_THIRD_PARTY_DEPENDENCY_ALLOWLIST,
      forbiddenDependencyPrefixes: STORAGE_FOUNDATION_FORBIDDEN_DEPENDENCY_PREFIXES,
    };
  }
  if (currentSlice === 'attachment_text_extraction_runtime_assembly_v1') {
    return {
      profile: FIRST_OWNER_PROFILE,
      ownerInventory: ATTACHMENT_TEXT_EXTRACTION_CONTRACT_CORE_INVENTORY,
      cargoFeatures: MAIL_OUTBOUND_MIME_ATTACHMENTS_CARGO_FEATURE_ALLOWLIST,
      packages: ATTACHMENT_TEXT_EXTRACTION_RUNTIME_ASSEMBLY_PRODUCTION_PACKAGES,
      workspaceDependencies: ATTACHMENT_TEXT_EXTRACTION_RUNTIME_ASSEMBLY_WORKSPACE_DEPENDENCY_ALLOWLIST,
      thirdPartyDependencies: ATTACHMENT_TEXT_EXTRACTION_RUNTIME_ASSEMBLY_THIRD_PARTY_DEPENDENCY_ALLOWLIST,
      forbiddenDependencyPrefixes: STORAGE_FOUNDATION_FORBIDDEN_DEPENDENCY_PREFIXES,
    };
  }
  if (currentSlice === 'attachment_preview_foundation_v1') {
    return {
      profile: FIRST_OWNER_PROFILE,
      ownerInventory: ATTACHMENT_PREVIEW_FOUNDATION_INVENTORY,
      cargoFeatures: MAIL_OUTBOUND_MIME_ATTACHMENTS_CARGO_FEATURE_ALLOWLIST,
      packages: ATTACHMENT_PREVIEW_FOUNDATION_PRODUCTION_PACKAGES,
      workspaceDependencies: ATTACHMENT_PREVIEW_FOUNDATION_WORKSPACE_DEPENDENCY_ALLOWLIST,
      thirdPartyDependencies: ATTACHMENT_PREVIEW_FOUNDATION_THIRD_PARTY_DEPENDENCY_ALLOWLIST,
      forbiddenDependencyPrefixes: STORAGE_FOUNDATION_FORBIDDEN_DEPENDENCY_PREFIXES,
    };
  }
  if (currentSlice === 'attachment_preview_safe_adapters_v1') {
    return {
      profile: FIRST_OWNER_PROFILE,
      ownerInventory: ATTACHMENT_PREVIEW_FOUNDATION_INVENTORY,
      cargoFeatures: MAIL_OUTBOUND_MIME_ATTACHMENTS_CARGO_FEATURE_ALLOWLIST,
      packages: ATTACHMENT_PREVIEW_SAFE_ADAPTERS_PRODUCTION_PACKAGES,
      workspaceDependencies: ATTACHMENT_PREVIEW_SAFE_ADAPTERS_WORKSPACE_DEPENDENCY_ALLOWLIST,
      thirdPartyDependencies: ATTACHMENT_PREVIEW_SAFE_ADAPTERS_THIRD_PARTY_DEPENDENCY_ALLOWLIST,
      forbiddenDependencyPrefixes: STORAGE_FOUNDATION_FORBIDDEN_DEPENDENCY_PREFIXES,
    };
  }
  if (currentSlice === 'attachment_preview_pdf_adapter_v1') {
    return {
      profile: FIRST_OWNER_PROFILE,
      ownerInventory: ATTACHMENT_PREVIEW_FOUNDATION_INVENTORY,
      cargoFeatures: MAIL_OUTBOUND_MIME_ATTACHMENTS_CARGO_FEATURE_ALLOWLIST,
      packages: ATTACHMENT_PREVIEW_PDF_ADAPTER_PRODUCTION_PACKAGES,
      workspaceDependencies: ATTACHMENT_PREVIEW_PDF_ADAPTER_WORKSPACE_DEPENDENCY_ALLOWLIST,
      thirdPartyDependencies: ATTACHMENT_PREVIEW_PDF_ADAPTER_THIRD_PARTY_DEPENDENCY_ALLOWLIST,
      forbiddenDependencyPrefixes: STORAGE_FOUNDATION_FORBIDDEN_DEPENDENCY_PREFIXES,
    };
  }
  if (currentSlice === 'attachment_preview_docx_adapter_v1') {
    return {
      profile: FIRST_OWNER_PROFILE,
      ownerInventory: ATTACHMENT_PREVIEW_FOUNDATION_INVENTORY,
      cargoFeatures: MAIL_OUTBOUND_MIME_ATTACHMENTS_CARGO_FEATURE_ALLOWLIST,
      packages: ATTACHMENT_PREVIEW_DOCX_ADAPTER_PRODUCTION_PACKAGES,
      workspaceDependencies: ATTACHMENT_PREVIEW_DOCX_ADAPTER_WORKSPACE_DEPENDENCY_ALLOWLIST,
      thirdPartyDependencies: ATTACHMENT_PREVIEW_DOCX_ADAPTER_THIRD_PARTY_DEPENDENCY_ALLOWLIST,
      forbiddenDependencyPrefixes: STORAGE_FOUNDATION_FORBIDDEN_DEPENDENCY_PREFIXES,
    };
  }
  if (currentSlice === 'attachment_preview_persistence_v1') {
    return {
      profile: FIRST_OWNER_PROFILE,
      ownerInventory: ATTACHMENT_PREVIEW_FOUNDATION_INVENTORY,
      cargoFeatures: MAIL_OUTBOUND_MIME_ATTACHMENTS_CARGO_FEATURE_ALLOWLIST,
      packages: ATTACHMENT_PREVIEW_PERSISTENCE_PRODUCTION_PACKAGES,
      workspaceDependencies: ATTACHMENT_PREVIEW_PERSISTENCE_WORKSPACE_DEPENDENCY_ALLOWLIST,
      thirdPartyDependencies: ATTACHMENT_PREVIEW_PERSISTENCE_THIRD_PARTY_DEPENDENCY_ALLOWLIST,
      forbiddenDependencyPrefixes: STORAGE_FOUNDATION_FORBIDDEN_DEPENDENCY_PREFIXES,
    };
  }
  if (currentSlice === 'attachment_preview_runtime_v1') {
    return {
      profile: FIRST_OWNER_PROFILE,
      ownerInventory: ATTACHMENT_PREVIEW_FOUNDATION_INVENTORY,
      cargoFeatures: MAIL_OUTBOUND_MIME_ATTACHMENTS_CARGO_FEATURE_ALLOWLIST,
      packages: ATTACHMENT_PREVIEW_RUNTIME_PRODUCTION_PACKAGES,
      workspaceDependencies: ATTACHMENT_PREVIEW_RUNTIME_WORKSPACE_DEPENDENCY_ALLOWLIST,
      thirdPartyDependencies: ATTACHMENT_PREVIEW_RUNTIME_THIRD_PARTY_DEPENDENCY_ALLOWLIST,
      forbiddenDependencyPrefixes: STORAGE_FOUNDATION_FORBIDDEN_DEPENDENCY_PREFIXES,
    };
  }
  if (
    currentSlice === 'attachment_preview_assembly_v1'
    || currentSlice === 'attachment_preview_managed_admission_v1'
    || currentSlice === 'attachment_preview_managed_formats_v1'
    || currentSlice === 'attachment_preview_failure_boundaries_v1'
    || currentSlice === 'attachment_preview_stale_outage_input_boundaries_v1'
    || currentSlice === 'attachment_preview_job_authority_fences_v1'
  ) {
    return {
      profile: FIRST_OWNER_PROFILE,
      ownerInventory: ATTACHMENT_PREVIEW_FOUNDATION_INVENTORY,
      cargoFeatures: MAIL_OUTBOUND_MIME_ATTACHMENTS_CARGO_FEATURE_ALLOWLIST,
      packages: ATTACHMENT_PREVIEW_ASSEMBLY_PRODUCTION_PACKAGES,
      workspaceDependencies: ATTACHMENT_PREVIEW_ASSEMBLY_WORKSPACE_DEPENDENCY_ALLOWLIST,
      thirdPartyDependencies: ATTACHMENT_PREVIEW_ASSEMBLY_THIRD_PARTY_DEPENDENCY_ALLOWLIST,
      forbiddenDependencyPrefixes: STORAGE_FOUNDATION_FORBIDDEN_DEPENDENCY_PREFIXES,
    };
  }
  if (currentSlice === 'attachment_preview_retained_evidence_replay_v1') {
    return {
      profile: FIRST_OWNER_PROFILE,
      ownerInventory: ATTACHMENT_PREVIEW_RETAINED_EVIDENCE_REPLAY_INVENTORY,
      cargoFeatures: MAIL_OUTBOUND_MIME_ATTACHMENTS_CARGO_FEATURE_ALLOWLIST,
      packages: ATTACHMENT_PREVIEW_RETAINED_EVIDENCE_REPLAY_PRODUCTION_PACKAGES,
      workspaceDependencies: ATTACHMENT_PREVIEW_RETAINED_EVIDENCE_REPLAY_WORKSPACE_DEPENDENCY_ALLOWLIST,
      thirdPartyDependencies: ATTACHMENT_PREVIEW_RETAINED_EVIDENCE_REPLAY_THIRD_PARTY_DEPENDENCY_ALLOWLIST,
      forbiddenDependencyPrefixes: STORAGE_FOUNDATION_FORBIDDEN_DEPENDENCY_PREFIXES,
    };
  }
  if (currentSlice === 'attachment_translation_contracts_v1') {
    return {
      profile: FIRST_OWNER_PROFILE,
      ownerInventory: ATTACHMENT_TRANSLATION_CONTRACTS_INVENTORY,
      cargoFeatures: MAIL_OUTBOUND_MIME_ATTACHMENTS_CARGO_FEATURE_ALLOWLIST,
      packages: ATTACHMENT_TRANSLATION_CONTRACTS_PRODUCTION_PACKAGES,
      workspaceDependencies: ATTACHMENT_TRANSLATION_CONTRACTS_WORKSPACE_DEPENDENCY_ALLOWLIST,
      thirdPartyDependencies: ATTACHMENT_TRANSLATION_CONTRACTS_THIRD_PARTY_DEPENDENCY_ALLOWLIST,
      forbiddenDependencyPrefixes: STORAGE_FOUNDATION_FORBIDDEN_DEPENDENCY_PREFIXES,
    };
  }
  if (currentSlice === 'attachment_translation_persistence_v1') {
    return {
      profile: FIRST_OWNER_PROFILE,
      ownerInventory: ATTACHMENT_TRANSLATION_CONTRACTS_INVENTORY,
      cargoFeatures: MAIL_OUTBOUND_MIME_ATTACHMENTS_CARGO_FEATURE_ALLOWLIST,
      packages: ATTACHMENT_TRANSLATION_PERSISTENCE_PRODUCTION_PACKAGES,
      workspaceDependencies: ATTACHMENT_TRANSLATION_PERSISTENCE_WORKSPACE_DEPENDENCY_ALLOWLIST,
      thirdPartyDependencies: ATTACHMENT_TRANSLATION_PERSISTENCE_THIRD_PARTY_DEPENDENCY_ALLOWLIST,
      forbiddenDependencyPrefixes: STORAGE_FOUNDATION_FORBIDDEN_DEPENDENCY_PREFIXES,
    };
  }
  if (currentSlice === 'attachment_translation_ai_engine_v1') {
    return {
      profile: FIRST_OWNER_PROFILE,
      ownerInventory: ATTACHMENT_TRANSLATION_AI_ENGINE_INVENTORY,
      cargoFeatures: MAIL_OUTBOUND_MIME_ATTACHMENTS_CARGO_FEATURE_ALLOWLIST,
      packages: ATTACHMENT_TRANSLATION_PERSISTENCE_PRODUCTION_PACKAGES,
      workspaceDependencies: ATTACHMENT_TRANSLATION_PERSISTENCE_WORKSPACE_DEPENDENCY_ALLOWLIST,
      thirdPartyDependencies: ATTACHMENT_TRANSLATION_PERSISTENCE_THIRD_PARTY_DEPENDENCY_ALLOWLIST,
      forbiddenDependencyPrefixes: STORAGE_FOUNDATION_FORBIDDEN_DEPENDENCY_PREFIXES,
    };
  }
  if (currentSlice === 'attachment_translation_runtime_assembly_v1') {
    return {
      profile: FIRST_OWNER_PROFILE,
      ownerInventory: ATTACHMENT_TRANSLATION_RUNTIME_ASSEMBLY_INVENTORY,
      cargoFeatures: MAIL_OUTBOUND_MIME_ATTACHMENTS_CARGO_FEATURE_ALLOWLIST,
      packages: ATTACHMENT_TRANSLATION_RUNTIME_ASSEMBLY_PRODUCTION_PACKAGES,
      workspaceDependencies: ATTACHMENT_TRANSLATION_RUNTIME_ASSEMBLY_WORKSPACE_DEPENDENCY_ALLOWLIST,
      thirdPartyDependencies: ATTACHMENT_TRANSLATION_RUNTIME_ASSEMBLY_THIRD_PARTY_DEPENDENCY_ALLOWLIST,
      forbiddenDependencyPrefixes: STORAGE_FOUNDATION_FORBIDDEN_DEPENDENCY_PREFIXES,
    };
  }
  if (
    currentSlice === 'attachment_translation_source_producer_v1'
    || currentSlice === 'attachment_translation_v1'
  ) {
    return {
      profile: FIRST_OWNER_PROFILE,
      ownerInventory: ATTACHMENT_TRANSLATION_SOURCE_PRODUCER_INVENTORY,
      cargoFeatures: MAIL_OUTBOUND_MIME_ATTACHMENTS_CARGO_FEATURE_ALLOWLIST,
      packages: ATTACHMENT_TRANSLATION_RUNTIME_ASSEMBLY_PRODUCTION_PACKAGES,
      workspaceDependencies: ATTACHMENT_TRANSLATION_SOURCE_PRODUCER_WORKSPACE_DEPENDENCY_ALLOWLIST,
      thirdPartyDependencies: ATTACHMENT_TRANSLATION_RUNTIME_ASSEMBLY_THIRD_PARTY_DEPENDENCY_ALLOWLIST,
      forbiddenDependencyPrefixes: STORAGE_FOUNDATION_FORBIDDEN_DEPENDENCY_PREFIXES,
    };
  }
  if (currentSlice === 'contacts_mail_identity_command_contract_core_v1') {
    return {
      profile: FIRST_OWNER_PROFILE,
      ownerInventory: CONTACTS_MAIL_IDENTITY_COMMAND_CONTRACT_CORE_INVENTORY,
      cargoFeatures: MAIL_OUTBOUND_MIME_ATTACHMENTS_CARGO_FEATURE_ALLOWLIST,
      packages: CONTACTS_MAIL_IDENTITY_COMMAND_CONTRACT_CORE_PRODUCTION_PACKAGES,
      workspaceDependencies: CONTACTS_MAIL_IDENTITY_COMMAND_CONTRACT_CORE_WORKSPACE_DEPENDENCY_ALLOWLIST,
      thirdPartyDependencies: CONTACTS_MAIL_IDENTITY_COMMAND_CONTRACT_CORE_THIRD_PARTY_DEPENDENCY_ALLOWLIST,
      forbiddenDependencyPrefixes: STORAGE_FOUNDATION_FORBIDDEN_DEPENDENCY_PREFIXES,
    };
  }
  if (currentSlice === 'contacts_mail_identity_command_persistence_v1') {
    return {
      profile: FIRST_OWNER_PROFILE,
      ownerInventory: CONTACTS_MAIL_IDENTITY_COMMAND_CONTRACT_CORE_INVENTORY,
      cargoFeatures: CONTACTS_MAIL_IDENTITY_COMMAND_PERSISTENCE_CARGO_FEATURE_ALLOWLIST,
      packages: CONTACTS_MAIL_IDENTITY_COMMAND_PERSISTENCE_PRODUCTION_PACKAGES,
      workspaceDependencies: CONTACTS_MAIL_IDENTITY_COMMAND_PERSISTENCE_WORKSPACE_DEPENDENCY_ALLOWLIST,
      thirdPartyDependencies: CONTACTS_MAIL_IDENTITY_COMMAND_PERSISTENCE_THIRD_PARTY_DEPENDENCY_ALLOWLIST,
      forbiddenDependencyPrefixes: STORAGE_FOUNDATION_FORBIDDEN_DEPENDENCY_PREFIXES,
    };
  }
  if (currentSlice === 'contacts_mail_identity_command_runtime_assembly_v1') {
    return {
      profile: FIRST_OWNER_PROFILE,
      ownerInventory: CONTACTS_MAIL_IDENTITY_COMMAND_CONTRACT_CORE_INVENTORY,
      cargoFeatures: CONTACTS_MAIL_IDENTITY_COMMAND_PERSISTENCE_CARGO_FEATURE_ALLOWLIST,
      packages: CONTACTS_MAIL_IDENTITY_COMMAND_RUNTIME_ASSEMBLY_PRODUCTION_PACKAGES,
      workspaceDependencies: CONTACTS_MAIL_IDENTITY_COMMAND_RUNTIME_ASSEMBLY_WORKSPACE_DEPENDENCY_ALLOWLIST,
      thirdPartyDependencies: CONTACTS_MAIL_IDENTITY_COMMAND_RUNTIME_ASSEMBLY_THIRD_PARTY_DEPENDENCY_ALLOWLIST,
      forbiddenDependencyPrefixes: STORAGE_FOUNDATION_FORBIDDEN_DEPENDENCY_PREFIXES,
    };
  }
  if (currentSlice === 'mail_contacts_sync_contract_core_v1') {
    return {
      profile: FIRST_OWNER_PROFILE,
      ownerInventory: MAIL_CONTACTS_SYNC_CONTRACT_CORE_INVENTORY,
      cargoFeatures: CONTACTS_MAIL_IDENTITY_COMMAND_PERSISTENCE_CARGO_FEATURE_ALLOWLIST,
      packages: MAIL_CONTACTS_SYNC_CONTRACT_CORE_PRODUCTION_PACKAGES,
      workspaceDependencies: MAIL_CONTACTS_SYNC_CONTRACT_CORE_WORKSPACE_DEPENDENCY_ALLOWLIST,
      thirdPartyDependencies: MAIL_CONTACTS_SYNC_CONTRACT_CORE_THIRD_PARTY_DEPENDENCY_ALLOWLIST,
      forbiddenDependencyPrefixes: STORAGE_FOUNDATION_FORBIDDEN_DEPENDENCY_PREFIXES,
    };
  }
  if (currentSlice === 'mail_contacts_sync_persistence_v1') {
    return {
      profile: FIRST_OWNER_PROFILE,
      ownerInventory: MAIL_CONTACTS_SYNC_PERSISTENCE_INVENTORY,
      cargoFeatures: MAIL_CONTACTS_SYNC_PERSISTENCE_CARGO_FEATURE_ALLOWLIST,
      packages: MAIL_CONTACTS_SYNC_PERSISTENCE_PRODUCTION_PACKAGES,
      workspaceDependencies: MAIL_CONTACTS_SYNC_PERSISTENCE_WORKSPACE_DEPENDENCY_ALLOWLIST,
      thirdPartyDependencies: MAIL_CONTACTS_SYNC_PERSISTENCE_THIRD_PARTY_DEPENDENCY_ALLOWLIST,
      forbiddenDependencyPrefixes: STORAGE_FOUNDATION_FORBIDDEN_DEPENDENCY_PREFIXES,
    };
  }
  if (currentSlice === 'mail_contacts_sync_runtime_admission_v1') {
    return {
      profile: FIRST_OWNER_PROFILE,
      ownerInventory: MAIL_CONTACTS_SYNC_RUNTIME_ADMISSION_INVENTORY,
      cargoFeatures: MAIL_CONTACTS_SYNC_PERSISTENCE_CARGO_FEATURE_ALLOWLIST,
      packages: MAIL_CONTACTS_SYNC_RUNTIME_ADMISSION_PRODUCTION_PACKAGES,
      workspaceDependencies: MAIL_CONTACTS_SYNC_RUNTIME_ADMISSION_WORKSPACE_DEPENDENCY_ALLOWLIST,
      thirdPartyDependencies: MAIL_CONTACTS_SYNC_RUNTIME_ADMISSION_THIRD_PARTY_DEPENDENCY_ALLOWLIST,
      forbiddenDependencyPrefixes: STORAGE_FOUNDATION_FORBIDDEN_DEPENDENCY_PREFIXES,
    };
  }
  if (currentSlice === 'mail_address_book_provider_adapters_v1') {
    return {
      profile: FIRST_OWNER_PROFILE,
      ownerInventory: MAIL_CONTACTS_SYNC_RUNTIME_ADMISSION_INVENTORY,
      cargoFeatures: MAIL_ADDRESS_BOOK_PROVIDER_ADAPTERS_CARGO_FEATURE_ALLOWLIST,
      packages: MAIL_ADDRESS_BOOK_PROVIDER_ADAPTERS_PRODUCTION_PACKAGES,
      workspaceDependencies: MAIL_ADDRESS_BOOK_PROVIDER_ADAPTERS_WORKSPACE_DEPENDENCY_ALLOWLIST,
      thirdPartyDependencies: MAIL_ADDRESS_BOOK_PROVIDER_ADAPTERS_THIRD_PARTY_DEPENDENCY_ALLOWLIST,
      forbiddenDependencyPrefixes: STORAGE_FOUNDATION_FORBIDDEN_DEPENDENCY_PREFIXES,
    };
  }
  if (currentSlice === 'mail_address_book_persistence_authority_v1') {
    return {
      profile: FIRST_OWNER_PROFILE,
      ownerInventory: MAIL_CONTACTS_SYNC_RUNTIME_ADMISSION_INVENTORY,
      cargoFeatures: MAIL_ADDRESS_BOOK_PERSISTENCE_AUTHORITY_CARGO_FEATURE_ALLOWLIST,
      packages: MAIL_ADDRESS_BOOK_PERSISTENCE_AUTHORITY_PRODUCTION_PACKAGES,
      workspaceDependencies: MAIL_ADDRESS_BOOK_PERSISTENCE_AUTHORITY_WORKSPACE_DEPENDENCY_ALLOWLIST,
      thirdPartyDependencies: MAIL_ADDRESS_BOOK_PERSISTENCE_AUTHORITY_THIRD_PARTY_DEPENDENCY_ALLOWLIST,
      forbiddenDependencyPrefixes: STORAGE_FOUNDATION_FORBIDDEN_DEPENDENCY_PREFIXES,
    };
  }
  if (currentSlice === 'mail_address_book_managed_provider_conformance_v1') {
    return {
      profile: FIRST_OWNER_PROFILE,
      ownerInventory: MAIL_ADDRESS_BOOK_RUNTIME_EXECUTION_INVENTORY,
      cargoFeatures: MAIL_ADDRESS_BOOK_RUNTIME_EXECUTION_CARGO_FEATURE_ALLOWLIST,
      packages: MAIL_ADDRESS_BOOK_RUNTIME_EXECUTION_PRODUCTION_PACKAGES,
      workspaceDependencies: MAIL_ADDRESS_BOOK_RUNTIME_EXECUTION_WORKSPACE_DEPENDENCY_ALLOWLIST,
      thirdPartyDependencies: MAIL_ADDRESS_BOOK_RUNTIME_EXECUTION_THIRD_PARTY_DEPENDENCY_ALLOWLIST,
      forbiddenDependencyPrefixes: STORAGE_FOUNDATION_FORBIDDEN_DEPENDENCY_PREFIXES,
    };
  }
  if (
    currentSlice === 'mail_contacts_sync_release_assembly_v1'
    || currentSlice === 'mail_contacts_sync_managed_provider_to_contacts_v1'
    || currentSlice === 'mail_contacts_sync_managed_scheduled_provider_to_contacts_v1'
    || currentSlice === 'mail_contacts_sync_managed_reverse_google_update_v1'
  ) {
    return {
      profile: FIRST_OWNER_PROFILE,
      ownerInventory: MAIL_ADDRESS_BOOK_RUNTIME_EXECUTION_INVENTORY,
      cargoFeatures: MAIL_ADDRESS_BOOK_RUNTIME_EXECUTION_CARGO_FEATURE_ALLOWLIST,
      packages: MAIL_CONTACTS_SYNC_RELEASE_ASSEMBLY_PRODUCTION_PACKAGES,
      workspaceDependencies: MAIL_CONTACTS_SYNC_RELEASE_ASSEMBLY_WORKSPACE_DEPENDENCY_ALLOWLIST,
      thirdPartyDependencies: MAIL_CONTACTS_SYNC_RELEASE_ASSEMBLY_THIRD_PARTY_DEPENDENCY_ALLOWLIST,
      forbiddenDependencyPrefixes: STORAGE_FOUNDATION_FORBIDDEN_DEPENDENCY_PREFIXES,
    };
  }
  if (currentSlice === 'desktop_call_recording_contract_core_v1') {
    return {
      profile: FIRST_OWNER_PROFILE,
      ownerInventory: DESKTOP_CALL_RECORDING_CONTRACT_CORE_INVENTORY,
      cargoFeatures: MAIL_ADDRESS_BOOK_RUNTIME_EXECUTION_CARGO_FEATURE_ALLOWLIST,
      packages: DESKTOP_CALL_RECORDING_CONTRACT_CORE_PRODUCTION_PACKAGES,
      workspaceDependencies: DESKTOP_CALL_RECORDING_CONTRACT_CORE_WORKSPACE_DEPENDENCY_ALLOWLIST,
      thirdPartyDependencies: DESKTOP_CALL_RECORDING_CONTRACT_CORE_THIRD_PARTY_DEPENDENCY_ALLOWLIST,
      forbiddenDependencyPrefixes: STORAGE_FOUNDATION_FORBIDDEN_DEPENDENCY_PREFIXES,
    };
  }
  if (currentSlice === 'desktop_call_recording_persistence_v1') {
    return {
      profile: FIRST_OWNER_PROFILE,
      ownerInventory: DESKTOP_CALL_RECORDING_CONTRACT_CORE_INVENTORY,
      cargoFeatures: MAIL_ADDRESS_BOOK_RUNTIME_EXECUTION_CARGO_FEATURE_ALLOWLIST,
      packages: DESKTOP_CALL_RECORDING_PERSISTENCE_PRODUCTION_PACKAGES,
      workspaceDependencies: DESKTOP_CALL_RECORDING_PERSISTENCE_WORKSPACE_DEPENDENCY_ALLOWLIST,
      thirdPartyDependencies: DESKTOP_CALL_RECORDING_PERSISTENCE_THIRD_PARTY_DEPENDENCY_ALLOWLIST,
      forbiddenDependencyPrefixes: STORAGE_FOUNDATION_FORBIDDEN_DEPENDENCY_PREFIXES,
    };
  }
  if (currentSlice === 'desktop_call_recording_runtime_v1') {
    return {
      profile: FIRST_OWNER_PROFILE,
      ownerInventory: DESKTOP_CALL_RECORDING_CONTRACT_CORE_INVENTORY,
      cargoFeatures: MAIL_ADDRESS_BOOK_RUNTIME_EXECUTION_CARGO_FEATURE_ALLOWLIST,
      packages: DESKTOP_CALL_RECORDING_RUNTIME_PRODUCTION_PACKAGES,
      workspaceDependencies: DESKTOP_CALL_RECORDING_RUNTIME_WORKSPACE_DEPENDENCY_ALLOWLIST,
      thirdPartyDependencies: DESKTOP_CALL_RECORDING_RUNTIME_THIRD_PARTY_DEPENDENCY_ALLOWLIST,
      forbiddenDependencyPrefixes: STORAGE_FOUNDATION_FORBIDDEN_DEPENDENCY_PREFIXES,
    };
  }
  if (currentSlice === 'desktop_call_recording_release_assembly_v1') {
    return {
      profile: FIRST_OWNER_PROFILE,
      ownerInventory: DESKTOP_CALL_RECORDING_CONTRACT_CORE_INVENTORY,
      cargoFeatures: MAIL_ADDRESS_BOOK_RUNTIME_EXECUTION_CARGO_FEATURE_ALLOWLIST,
      packages: DESKTOP_CALL_RECORDING_RELEASE_ASSEMBLY_PRODUCTION_PACKAGES,
      workspaceDependencies: DESKTOP_CALL_RECORDING_RELEASE_ASSEMBLY_WORKSPACE_DEPENDENCY_ALLOWLIST,
      thirdPartyDependencies: DESKTOP_CALL_RECORDING_RELEASE_ASSEMBLY_THIRD_PARTY_DEPENDENCY_ALLOWLIST,
      forbiddenDependencyPrefixes: STORAGE_FOUNDATION_FORBIDDEN_DEPENDENCY_PREFIXES,
    };
  }
  if (currentSlice === 'call_transcription_contract_core_v1') {
    return {
      profile: FIRST_OWNER_PROFILE,
      ownerInventory: CALL_TRANSCRIPTION_CONTRACT_CORE_INVENTORY,
      cargoFeatures: MAIL_ADDRESS_BOOK_RUNTIME_EXECUTION_CARGO_FEATURE_ALLOWLIST,
      packages: CALL_TRANSCRIPTION_CONTRACT_CORE_PRODUCTION_PACKAGES,
      workspaceDependencies: CALL_TRANSCRIPTION_CONTRACT_CORE_WORKSPACE_DEPENDENCY_ALLOWLIST,
      thirdPartyDependencies: CALL_TRANSCRIPTION_CONTRACT_CORE_THIRD_PARTY_DEPENDENCY_ALLOWLIST,
      forbiddenDependencyPrefixes: STORAGE_FOUNDATION_FORBIDDEN_DEPENDENCY_PREFIXES,
    };
  }
  if (currentSlice === 'call_transcription_persistence_v1') {
    return {
      profile: FIRST_OWNER_PROFILE,
      ownerInventory: CALL_TRANSCRIPTION_PERSISTENCE_INVENTORY,
      cargoFeatures: MAIL_ADDRESS_BOOK_RUNTIME_EXECUTION_CARGO_FEATURE_ALLOWLIST,
      packages: CALL_TRANSCRIPTION_PERSISTENCE_PRODUCTION_PACKAGES,
      workspaceDependencies: CALL_TRANSCRIPTION_PERSISTENCE_WORKSPACE_DEPENDENCY_ALLOWLIST,
      thirdPartyDependencies: CALL_TRANSCRIPTION_PERSISTENCE_THIRD_PARTY_DEPENDENCY_ALLOWLIST,
      forbiddenDependencyPrefixes: STORAGE_FOUNDATION_FORBIDDEN_DEPENDENCY_PREFIXES,
    };
  }
  if (currentSlice === 'call_transcription_runtime_v1') {
    return {
      profile: FIRST_OWNER_PROFILE,
      ownerInventory: CALL_TRANSCRIPTION_RUNTIME_INVENTORY,
      cargoFeatures: MAIL_ADDRESS_BOOK_RUNTIME_EXECUTION_CARGO_FEATURE_ALLOWLIST,
      packages: CALL_TRANSCRIPTION_RUNTIME_PRODUCTION_PACKAGES,
      workspaceDependencies: CALL_TRANSCRIPTION_RUNTIME_WORKSPACE_DEPENDENCY_ALLOWLIST,
      thirdPartyDependencies: CALL_TRANSCRIPTION_RUNTIME_THIRD_PARTY_DEPENDENCY_ALLOWLIST,
      forbiddenDependencyPrefixes: STORAGE_FOUNDATION_FORBIDDEN_DEPENDENCY_PREFIXES,
    };
  }
  if (currentSlice === 'call_transcription_managed_conformance_v1') {
    return {
      profile: FIRST_OWNER_PROFILE,
      ownerInventory: CALL_TRANSCRIPTION_RUNTIME_INVENTORY,
      cargoFeatures: MAIL_PERSONS_SYNC_PERSISTENCE_CARGO_FEATURE_ALLOWLIST,
      packages: MAIL_PERSONS_SYNC_CONTRACT_CORE_PRODUCTION_PACKAGES,
      workspaceDependencies: MAIL_PERSONS_SYNC_CONTRACT_CORE_WORKSPACE_DEPENDENCY_ALLOWLIST,
      thirdPartyDependencies: MAIL_PERSONS_SYNC_CONTRACT_CORE_THIRD_PARTY_DEPENDENCY_ALLOWLIST,
      forbiddenDependencyPrefixes: STORAGE_FOUNDATION_FORBIDDEN_DEPENDENCY_PREFIXES,
    };
  }
  if (currentSlice === 'persons_admission_v1') {
    return {
      profile: FIRST_OWNER_PROFILE,
      ownerInventory: PERSONS_ADMISSION_INVENTORY,
      cargoFeatures: PERSONS_ADMISSION_CARGO_FEATURE_ALLOWLIST,
      packages: PERSONS_ADMISSION_PRODUCTION_PACKAGES,
      workspaceDependencies: PERSONS_ADMISSION_WORKSPACE_DEPENDENCY_ALLOWLIST,
      thirdPartyDependencies: PERSONS_ADMISSION_THIRD_PARTY_DEPENDENCY_ALLOWLIST,
      forbiddenDependencyPrefixes: STORAGE_FOUNDATION_FORBIDDEN_DEPENDENCY_PREFIXES,
    };
  }
  if (currentSlice === 'communication_bulk_delayed_delivery_admission_v1') {
    return {
      profile: FIRST_OWNER_PROFILE,
      ownerInventory: COMMUNICATION_BULK_DELAYED_DELIVERY_ADMISSION_INVENTORY,
      cargoFeatures: PERSONS_ADMISSION_CARGO_FEATURE_ALLOWLIST,
      packages: PERSONS_ADMISSION_PRODUCTION_PACKAGES,
      workspaceDependencies: PERSONS_ADMISSION_WORKSPACE_DEPENDENCY_ALLOWLIST,
      thirdPartyDependencies: PERSONS_ADMISSION_THIRD_PARTY_DEPENDENCY_ALLOWLIST,
      forbiddenDependencyPrefixes: STORAGE_FOUNDATION_FORBIDDEN_DEPENDENCY_PREFIXES,
    };
  }
  if (currentSlice === 'ai_inference_ollama_admission_v1') {
    return {
      profile: FIRST_OWNER_PROFILE,
      ownerInventory: AI_INFERENCE_OLLAMA_ADMISSION_INVENTORY,
      cargoFeatures: PERSONS_ADMISSION_CARGO_FEATURE_ALLOWLIST,
      packages: AI_INFERENCE_OLLAMA_ADMISSION_PRODUCTION_PACKAGES,
      workspaceDependencies: AI_INFERENCE_OLLAMA_ADMISSION_WORKSPACE_DEPENDENCY_ALLOWLIST,
      thirdPartyDependencies: AI_INFERENCE_OLLAMA_ADMISSION_THIRD_PARTY_DEPENDENCY_ALLOWLIST,
      forbiddenDependencyPrefixes: STORAGE_FOUNDATION_FORBIDDEN_DEPENDENCY_PREFIXES,
    };
  }
  if (currentSlice === 'speech_to_text_whisper_admission_v1') {
    return {
      profile: FIRST_OWNER_PROFILE,
      ownerInventory: SPEECH_TO_TEXT_WHISPER_ADMISSION_INVENTORY,
      cargoFeatures: PERSONS_ADMISSION_CARGO_FEATURE_ALLOWLIST,
      packages: SPEECH_TO_TEXT_WHISPER_ADMISSION_PRODUCTION_PACKAGES,
      workspaceDependencies: SPEECH_TO_TEXT_WHISPER_ADMISSION_WORKSPACE_DEPENDENCY_ALLOWLIST,
      thirdPartyDependencies: SPEECH_TO_TEXT_WHISPER_ADMISSION_THIRD_PARTY_DEPENDENCY_ALLOWLIST,
      forbiddenDependencyPrefixes: STORAGE_FOUNDATION_FORBIDDEN_DEPENDENCY_PREFIXES,
    };
  }
  return null;
}

function isExactDevelopmentProfile(profile) {
  return hasExactKeys(profile, DEVELOPMENT_PROFILE_KEYS)
    && profile.id === 'development_full_platform_v1'
    && profile.purpose === 'full_local_platform_development_with_simulated_trust'
    && profile.workspaceRoot === 'development'
    && Array.isArray(profile.packages)
    && profile.packages.length === 2
    && profile.packages.every((entry) => hasExactKeys(entry, DEVELOPMENT_PACKAGE_KEYS))
    && profile.packages[0].package === 'makosh-development-kernel-operator'
    && profile.packages[0].surface === 'runtime'
    && profile.packages[1].package === 'makosh-development-assembly'
    && profile.packages[1].surface === 'assembly'
    && profile.selection === 'explicit_development_invocation_only'
    && profile.deviceProof === 'file_adapter_es256'
    && profile.privateKeyStorage === 'owner_private_file_adapter'
    && profile.persistentSecretsAllowed === true
    && profile.productDataAllowed === true
    && profile.networkListenerEnabled === true
    && profile.remotePairingEnabled === true
    && profile.externalServicesEnabled === true
    && profile.vaultEnabled === true
    && profile.releaseArtifactAllowed === false
    && profile.productionGateEvidenceAllowed === false
    && profile.visibleInsecureWarningRequired === true
    && profile.automaticProductionFallbackAllowed === false
    && isExactOrderedStringList(profile.simulatedTargets, [
      'macos_tauri_embedded_v1',
      'linux_docker_server_v1',
    ]);
}

function isExactClock(clock) {
  return hasExactKeys(clock, CLOCK_KEYS)
    && clock.wallTime === 'system_time_utc_timestamps_only'
    && clock.elapsedTime === 'monotonic_deadlines_and_timeouts'
    && clock.testTime === 'injected_deterministic_fake'
    && clock.moduleCapabilityEnabled === false;
}

function isExactKernelProfile(profile, constitutionalComponents, expected) {
  return expected !== null
    && expected !== undefined
    && hasExactKeys(profile, KERNEL_PROFILE_KEYS)
    && profile.maximumState === expected.maximumState
    && isExactOrderedStringList(profile.allowedStates, expected.allowedStates)
    && isExactOrderedStringList(profile.forbiddenStates, expected.forbiddenStates)
    && isExactOrderedStringList(profile.activeComponents, expected.activeComponents)
    && profile.activeComponents.every((component) => constitutionalComponents.includes(component))
    && profile.transport === expected.transport
    && isExactOrderedStringList(profile.onlineOperations, expected.onlineOperations)
    && isExactOrderedStringList(profile.bootstrapOperations, expected.bootstrapOperations)
    && isExactOrderedStringList(profile.offlineOperations, expected.offlineOperations)
    && isExactOrderedStringList(profile.externalServices, expected.externalServices)
    && isExactOrderedStringList(profile.managedChildren, expected.managedChildren)
    && profile.publicGatewayEnabled === (expected.publicGatewayEnabled ?? false)
    && profile.networkListenerEnabled === expected.networkListenerEnabled
    && profile.moduleRegistrationEnabled === expected.moduleRegistrationEnabled
    && profile.managedLaunchEnabled === expected.managedLaunchEnabled
    && profile.natsDataPlaneEnabled === (expected.natsDataPlaneEnabled ?? false)
    && profile.businessDataPlaneEnabled === (expected.businessDataPlaneEnabled ?? false)
    && profile.wholeInstanceBackupEnabled === (expected.wholeInstanceBackupEnabled ?? false)
    && isExactClock(profile.clock);
}

export function validateImplementationSlicePolicy(policy) {
  const implementation = policy?.implementation;
  const slice = expectedSlice(implementation?.currentSlice);
  const checks = {
    implementation_keys: hasExactKeys(implementation, IMPLEMENTATION_KEYS),
    supported_slice: slice !== null,
    package_mode: implementation?.productionPackageMode === 'exact_allowlist',
    package_inventory: isExactPackageInventory(implementation?.productionPackages, slice?.packages),
    workspace_dependencies: isExactWorkspaceDependencyAllowlist(
      implementation?.workspaceDependencyAllowlist,
      slice?.packages,
      slice?.workspaceDependencies,
    ),
    third_party_dependencies: isExactThirdPartyDependencyAllowlist(
      implementation?.thirdPartyDependencyAllowlist,
      slice?.packages,
      slice?.thirdPartyDependencies,
    ),
    forbidden_dependencies: isExactOrderedStringList(
      implementation?.forbiddenDependencies,
      FORBIDDEN_DEPENDENCIES,
    ),
    forbidden_dependency_prefixes: isExactOrderedStringList(
      implementation?.forbiddenDependencyPrefixes,
      slice?.forbiddenDependencyPrefixes,
    ),
    cargo_features: implementation?.cargoFeaturesEnabled === false,
    cargo_feature_allowlist: isExactCargoFeatureAllowlist(
      implementation?.cargoFeatureAllowlist,
      slice?.cargoFeatures ?? {},
    ),
    target_policy: isExactTargetPolicy(implementation?.targetPolicy, slice?.packages),
    development_profile: isExactDevelopmentProfile(implementation?.developmentProfile),
    owner_inventory: slice?.ownerInventory
      ? isExactOwnerInventory(implementation?.ownerInventory, slice.ownerInventory)
      : isEmptyOwnerInventory(implementation?.ownerInventory),
    kernel_profile: isExactKernelProfile(
      implementation?.kernelProfile,
      list(policy?.kernel?.constitutionalComponents),
      slice?.profile,
    ),
    exit_gates: isExactOrderedStringList(implementation?.exitGates, EXIT_GATES),
  };
  const invalidChecks = Object.entries(checks)
    .filter(([, valid]) => !valid)
    .map(([name]) => name);

  return invalidChecks.length === 0 ? [] : [violation(
    'implementation_slice_policy',
    'implementation',
    `current implementation must remain the exact authorized Kernel slice; invalid=${invalidChecks.join(',')}`,
  )];
}
