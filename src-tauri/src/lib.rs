use tauri::{Emitter, Manager};
use tauri_plugin_global_shortcut::{Code, GlobalShortcutExt, Shortcut, ShortcutState};

#[tauri::command]
fn minimize_window(window: tauri::Window) {
    let _ = window.minimize();
}

#[tauri::command]
fn toggle_maximize_window(window: tauri::Window) {
    if let Ok(is_maximized) = window.is_maximized() {
        if is_maximized {
            let _ = window.unmaximize();
        } else {
            let _ = window.maximize();
        }
    }
}

#[tauri::command]
fn close_window(window: tauri::Window) {
    let _ = window.close();
}

#[tauri::command]
fn open_external(url: String) {
    #[cfg(target_os = "windows")]
    {
        let _ = std::process::Command::new("rundll32")
            .args(["url.dll,FileProtocolHandler", &url])
            .spawn();
    }
    #[cfg(target_os = "macos")]
    {
        let _ = std::process::Command::new("open").arg(&url).spawn();
    }
    #[cfg(target_os = "linux")]
    {
        let _ = std::process::Command::new("xdg-open").arg(&url).spawn();
    }
}

fn send_media_command(app: &tauri::AppHandle, cmd: &str) {
    let _ = app.emit("media-command", cmd);
    if let Some(window) = app.get_webview_window("main") {
        let script = format!("window.__raserMediaCommand && window.__raserMediaCommand('{}');", cmd);
        let _ = window.eval(&script);
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_log::Builder::default().build())
        .plugin(
            tauri_plugin_global_shortcut::Builder::new()
                .with_handler(|app, shortcut, event| {
                    if event.state() == ShortcutState::Pressed {
                        match shortcut.key {
                            Code::MediaPlayPause => send_media_command(app, "toggle"),
                            Code::MediaTrackNext => send_media_command(app, "next"),
                            Code::MediaTrackPrevious => send_media_command(app, "prev"),
                            Code::MediaStop => send_media_command(app, "stop"),
                            _ => {}
                        }
                    }
                })
                .build(),
        )
        .invoke_handler(tauri::generate_handler![
            minimize_window,
            toggle_maximize_window,
            close_window,
            open_external
        ])
        .setup(|app| {
            let _ = app.global_shortcut().register(Shortcut::new(None, Code::MediaPlayPause));
            let _ = app.global_shortcut().register(Shortcut::new(None, Code::MediaTrackNext));
            let _ = app.global_shortcut().register(Shortcut::new(None, Code::MediaTrackPrevious));
            let _ = app.global_shortcut().register(Shortcut::new(None, Code::MediaStop));

            if let Some(window) = app.get_webview_window("main") {
                let _ = window.show();
            }
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while building tauri application");
}
