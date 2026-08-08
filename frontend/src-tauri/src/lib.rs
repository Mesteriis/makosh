use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, Ordering};

use tauri::{AppHandle, Manager, Runtime};
use tauri_plugin_shell::ShellExt;
use tauri_plugin_shell::process::{CommandChild, CommandEvent};

#[cfg(feature = "desktop-call-recording-host")]
mod desktop_call_recording_host;
mod owner_vault_provisioning;
#[cfg(feature = "whatsapp-host-webview")]
mod whatsapp_companion;
#[cfg(feature = "telemost-host-companion")]
mod yandex_telemost_companion;

#[derive(Default)]
struct KernelSidecar {
    child: Mutex<Option<CommandChild>>,
    stopping: AtomicBool,
}

const MAX_KERNEL_RESTARTS: u8 = 3;

impl KernelSidecar {
    fn stopping(&self) -> bool {
        self.stopping.load(Ordering::Acquire)
    }
}

impl Drop for KernelSidecar {
    fn drop(&mut self) {
        self.stopping.store(true, Ordering::Release);
        if let Ok(mut child) = self.child.lock()
            && let Some(child) = child.take()
        {
            let _ = child.kill();
        }
    }
}

pub fn run() {
    let builder = tauri::Builder::default().plugin(tauri_plugin_shell::init());
    #[cfg(not(any(feature = "whatsapp-host-webview", feature = "telemost-host-companion")))]
    let builder = builder.invoke_handler(tauri::generate_handler![
        owner_vault_provisioning::owner_vault_provisioning_host_start,
        owner_vault_provisioning::owner_vault_provisioning_host_seal,
        owner_vault_provisioning::owner_vault_provisioning_host_open_receipt,
        owner_vault_provisioning::owner_vault_provisioning_host_cancel,
        #[cfg(feature = "desktop-call-recording-host")]
        desktop_call_recording_host::desktop_call_recording_host_connect,
        #[cfg(feature = "desktop-call-recording-host")]
        desktop_call_recording_host::desktop_call_recording_host_disconnect,
    ]);
    #[cfg(all(
        feature = "whatsapp-host-webview",
        not(feature = "telemost-host-companion")
    ))]
    let builder = builder.invoke_handler(tauri::generate_handler![
        owner_vault_provisioning::owner_vault_provisioning_host_start,
        owner_vault_provisioning::owner_vault_provisioning_host_seal,
        owner_vault_provisioning::owner_vault_provisioning_host_open_receipt,
        owner_vault_provisioning::owner_vault_provisioning_host_cancel,
        whatsapp_companion::start_hidden_whatsapp_webview,
        whatsapp_companion::whatsapp_web_companion_manifest,
        whatsapp_companion::open_whatsapp_web_companion,
        whatsapp_companion::hide_whatsapp_web_companion,
        whatsapp_companion::connect_whatsapp_runtime_bridge,
        whatsapp_companion::whatsapp_web_companion_relay_runtime_state,
        #[cfg(feature = "desktop-call-recording-host")]
        desktop_call_recording_host::desktop_call_recording_host_connect,
        #[cfg(feature = "desktop-call-recording-host")]
        desktop_call_recording_host::desktop_call_recording_host_disconnect,
    ]);
    #[cfg(all(
        feature = "telemost-host-companion",
        not(feature = "whatsapp-host-webview")
    ))]
    let builder = builder.invoke_handler(tauri::generate_handler![
        owner_vault_provisioning::owner_vault_provisioning_host_start,
        owner_vault_provisioning::owner_vault_provisioning_host_seal,
        owner_vault_provisioning::owner_vault_provisioning_host_open_receipt,
        owner_vault_provisioning::owner_vault_provisioning_host_cancel,
        yandex_telemost_companion::open_yandex_telemost_companion,
        yandex_telemost_companion::yandex_telemost_companion_manifest,
        #[cfg(feature = "desktop-call-recording-host")]
        desktop_call_recording_host::desktop_call_recording_host_connect,
        #[cfg(feature = "desktop-call-recording-host")]
        desktop_call_recording_host::desktop_call_recording_host_disconnect,
    ]);
    #[cfg(all(feature = "whatsapp-host-webview", feature = "telemost-host-companion"))]
    let builder = builder.invoke_handler(tauri::generate_handler![
        owner_vault_provisioning::owner_vault_provisioning_host_start,
        owner_vault_provisioning::owner_vault_provisioning_host_seal,
        owner_vault_provisioning::owner_vault_provisioning_host_open_receipt,
        owner_vault_provisioning::owner_vault_provisioning_host_cancel,
        whatsapp_companion::start_hidden_whatsapp_webview,
        whatsapp_companion::whatsapp_web_companion_manifest,
        whatsapp_companion::open_whatsapp_web_companion,
        whatsapp_companion::hide_whatsapp_web_companion,
        whatsapp_companion::connect_whatsapp_runtime_bridge,
        whatsapp_companion::whatsapp_web_companion_relay_runtime_state,
        yandex_telemost_companion::open_yandex_telemost_companion,
        yandex_telemost_companion::yandex_telemost_companion_manifest,
        #[cfg(feature = "desktop-call-recording-host")]
        desktop_call_recording_host::desktop_call_recording_host_connect,
        #[cfg(feature = "desktop-call-recording-host")]
        desktop_call_recording_host::desktop_call_recording_host_disconnect,
    ]);

    builder
        .setup(|app| {
            if cfg!(debug_assertions) {
                app.handle().plugin(
                    tauri_plugin_log::Builder::default()
                        .level(log::LevelFilter::Info)
                        .build(),
                )?;
            }
            app.manage(KernelSidecar::default());
            app.manage(owner_vault_provisioning::OwnerVaultProvisioningHostStateV1::default());
            #[cfg(feature = "desktop-call-recording-host")]
            app.manage(desktop_call_recording_host::DesktopCallRecordingHostStateV1::default());
            #[cfg(feature = "desktop-call-recording-host")]
            desktop_call_recording_host::watch_for_route_admission(
                app.handle().clone(),
                app.state(),
            )?;
            #[cfg(feature = "whatsapp-host-webview")]
            app.manage(whatsapp_companion::WhatsAppHostRoutes::default());
            if !cfg!(debug_assertions) {
                start_kernel_sidecar(app.handle().clone(), 0)?;
            }
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

fn start_kernel_sidecar<R: Runtime>(
    app: AppHandle<R>,
    restart_attempt: u8,
) -> Result<(), Box<dyn std::error::Error>> {
    if app.state::<KernelSidecar>().stopping() {
        return Ok(());
    }

    let command = app
        .shell()
        .sidecar("makosh-kernel")?
        .env_clear()
        .arg("--data-dir")
        .arg(app.path().app_local_data_dir()?)
        .arg("serve");

    let (mut events, child) = command.spawn()?;
    let pid = child.pid();
    app.state::<KernelSidecar>()
        .child
        .lock()
        .map_err(|_| std::io::Error::other("kernel sidecar state lock poisoned"))?
        .replace(child);

    let app_for_events = app.clone();
    tauri::async_runtime::spawn(async move {
        log::info!("Макошь Kernel sidecar started with pid {pid}");
        while let Some(event) = events.recv().await {
            match event {
                CommandEvent::Stdout(_) | CommandEvent::Stderr(_) => {
                    log::debug!("Макошь Kernel sidecar emitted output (suppressed)");
                }
                CommandEvent::Error(_) => log::error!("Макошь Kernel sidecar event error"),
                CommandEvent::Terminated(payload) => {
                    log::warn!(
                        "Макошь Kernel sidecar terminated: code={:?} signal={:?}",
                        payload.code,
                        payload.signal
                    );
                    if !app_for_events.state::<KernelSidecar>().stopping()
                        && restart_attempt < MAX_KERNEL_RESTARTS
                        && let Err(error) =
                            start_kernel_sidecar(app_for_events.clone(), restart_attempt + 1)
                    {
                        log::error!("Макошь Kernel bounded restart failed: {error}");
                    }
                    return;
                }
                _ => {}
            }
        }
    });

    Ok(())
}
