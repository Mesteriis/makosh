# Макошь Communications — WhatsApp Channel

Historical policy: [ADR-0182](../../archive/adr/ADR-0182-whatsapp-hidden-webview-runtime-only.md).

WhatsApp is a Communications integration, never a product domain. Its only
runtime is an account-scoped, hidden Tauri WebView for `https://web.whatsapp.com`.
It emits metadata-only observations through the runtime bridge; Communications
persists canonical evidence and Signal Hub performs review/routing.

The integration has no Business Cloud API, Native MD, external headless browser,
edge proxy or in-process command executor. The WebView never becomes visible,
focused or a source of direct domain mutations. Session secrets remain in the
host vault; PostgreSQL contains references and metadata only.

Historical rows with retired provider strings remain readable for data
preservation, but cannot be selected, started or dispatched.
