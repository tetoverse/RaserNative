mod discord;
mod local_files;
mod updater;

use tauri::{
    menu::{Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    Emitter, Manager,
};
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
    // Hide to system tray instead of closing process so playback continues
    let _ = window.hide();
}

#[tauri::command]
fn exit_app() {
    std::process::exit(0);
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

#[tauri::command]
fn discord_rpc_update(track: discord::DiscordTrackInfo) {
    discord::update_presence(track);
}

#[tauri::command]
fn discord_rpc_clear() {
    discord::clear_presence();
}

#[tauri::command]
async fn check_update() -> Result<updater::UpdateInfo, String> {
    updater::check_for_update().await
}

#[tauri::command]
async fn install_update(app: tauri::AppHandle, download_url: String) -> Result<(), String> {
    updater::download_and_install_update(app, download_url).await
}

#[tauri::command]
fn pick_local_files() -> Vec<local_files::LocalTrack> {
    local_files::pick_files()
}

#[tauri::command]
fn pick_local_folder() -> Vec<local_files::LocalTrack> {
    local_files::pick_folder()
}

fn is_tr_locale() -> bool {
    #[cfg(target_os = "windows")]
    unsafe {
        unsafe extern "system" {
            fn GetUserDefaultUILanguage() -> u16;
        }
        if GetUserDefaultUILanguage() == 0x041F {
            return true;
        }
    }
    if let Ok(lang) = std::env::var("LANG") {
        if lang.to_lowercase().starts_with("tr") {
            return true;
        }
    }
    false
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
            exit_app,
            open_external,
            discord_rpc_update,
            discord_rpc_clear,
            check_update,
            install_update,
            pick_local_files,
            pick_local_folder
        ])
        .setup(|app| {
            let _ = app.global_shortcut().register(Shortcut::new(None, Code::MediaPlayPause));
            let _ = app.global_shortcut().register(Shortcut::new(None, Code::MediaTrackNext));
            let _ = app.global_shortcut().register(Shortcut::new(None, Code::MediaTrackPrevious));
            let _ = app.global_shortcut().register(Shortcut::new(None, Code::MediaStop));

            // Setup System Tray
            let is_tr = is_tr_locale();
            let show_text = if is_tr { "Raser'ı Göster" } else { "Show Raser" };
            let toggle_text = if is_tr { "Oynat / Durdur" } else { "Play / Pause" };
            let next_text = if is_tr { "Sonraki Şarkı" } else { "Next Track" };
            let prev_text = if is_tr { "Önceki Şarkı" } else { "Previous Track" };
            let quit_text = if is_tr { "Çıkış" } else { "Quit" };

            let show_i = MenuItem::with_id(app, "show", show_text, true, None::<&str>)?;
            let toggle_i = MenuItem::with_id(app, "toggle_play", toggle_text, true, None::<&str>)?;
            let next_i = MenuItem::with_id(app, "next_track", next_text, true, None::<&str>)?;
            let prev_i = MenuItem::with_id(app, "prev_track", prev_text, true, None::<&str>)?;
            let quit_i = MenuItem::with_id(app, "quit", quit_text, true, None::<&str>)?;

            let tray_menu = Menu::with_items(app, &[
                &show_i,
                &toggle_i,
                &next_i,
                &prev_i,
                &quit_i,
            ])?;

            let mut tray_builder = TrayIconBuilder::new()
                .menu(&tray_menu)
                .tooltip("Raser")
                .on_menu_event(|app, event| {
                    match event.id.as_ref() {
                        "show" => {
                            if let Some(w) = app.get_webview_window("main") {
                                let _ = w.show();
                                let _ = w.unminimize();
                                let _ = w.set_focus();
                            }
                        }
                        "toggle_play" => send_media_command(app, "toggle"),
                        "next_track" => send_media_command(app, "next"),
                        "prev_track" => send_media_command(app, "prev"),
                        "quit" => {
                            std::process::exit(0);
                        }
                        _ => {}
                    }
                })
                .on_tray_icon_event(|tray, event| {
                    if let TrayIconEvent::Click {
                        button: MouseButton::Left,
                        button_state: MouseButtonState::Up,
                        ..
                    } = event
                    {
                        let app = tray.app_handle();
                        if let Some(w) = app.get_webview_window("main") {
                            if w.is_visible().unwrap_or(false) {
                                let _ = w.hide();
                            } else {
                                let _ = w.show();
                                let _ = w.unminimize();
                                let _ = w.set_focus();
                            }
                        }
                    }
                });

            if let Some(icon) = app.default_window_icon() {
                tray_builder = tray_builder.icon(icon.clone());
            }

            let _ = tray_builder.build(app)?;

            if let Some(window) = app.get_webview_window("main") {
                let _ = window.show();
            }
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while building tauri application");
}
