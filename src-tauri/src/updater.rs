use std::fs::File;
use std::io::Write;
use std::path::PathBuf;
use std::process::Command;
use tauri::Emitter;

const CURRENT_VERSION: &str = "1.0.0";
const VERSION_ENDPOINT: &str = "https://app.raserapp.online/api/desktop-version";

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
pub struct RemoteVersionResponse {
    pub version: String,
    pub min_version: Option<String>,
    pub mandatory: Option<bool>,
    pub download_url: Option<String>,
    pub notes: Option<String>,
    pub pub_date: Option<String>,
}

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
pub struct UpdateInfo {
    pub should_update: bool,
    pub current_version: String,
    pub latest_version: String,
    pub mandatory: bool,
    pub download_url: String,
    pub notes: String,
}

#[derive(serde::Serialize, Clone, Debug)]
pub struct DownloadProgress {
    pub percent: u8,
    pub downloaded_bytes: u64,
    pub total_bytes: u64,
}

fn parse_semver(v: &str) -> (u32, u32, u32) {
    let clean = v.trim().trim_start_matches('v');
    let mut parts = clean.split('.');
    let major = parts.next().and_then(|s| s.parse().ok()).unwrap_or(0);
    let minor = parts.next().and_then(|s| s.parse().ok()).unwrap_or(0);
    let patch = parts.next().and_then(|s| s.parse().ok()).unwrap_or(0);
    (major, minor, patch)
}

pub fn is_newer_version(current: &str, target: &str) -> bool {
    let c = parse_semver(current);
    let t = parse_semver(target);
    t > c
}

pub async fn check_for_update() -> Result<UpdateInfo, String> {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .build()
        .map_err(|e| e.to_string())?;

    let res = client
        .get(VERSION_ENDPOINT)
        .header("User-Agent", format!("RaserNative/{}", CURRENT_VERSION))
        .send()
        .await
        .map_err(|e| e.to_string())?;

    if !res.status().is_success() {
        return Err(format!("Server returned HTTP {}", res.status()));
    }

    let data: RemoteVersionResponse = res.json().await.map_err(|e| e.to_string())?;
    let should_update = is_newer_version(CURRENT_VERSION, &data.version)
        || data
            .min_version
            .as_deref()
            .map(|min| is_newer_version(CURRENT_VERSION, min))
            .unwrap_or(false);

    let default_url = format!(
        "https://github.com/YourDarkstFear/RaserNative/releases/download/v{}/RaserNative-Setup.exe",
        data.version
    );

    Ok(UpdateInfo {
        should_update,
        current_version: CURRENT_VERSION.to_string(),
        latest_version: data.version,
        mandatory: data.mandatory.unwrap_or(true), // Always mandatory by default
        download_url: data.download_url.unwrap_or(default_url),
        notes: data
            .notes
            .unwrap_or_else(|| "Yeni hata düzeltmeleri ve performans geliştirmeleri.".into()),
    })
}

pub async fn download_and_install_update(
    app: tauri::AppHandle,
    download_url: String,
) -> Result<(), String> {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(300))
        .build()
        .map_err(|e| e.to_string())?;

    let mut res = client
        .get(&download_url)
        .header("User-Agent", format!("RaserNative/{}", CURRENT_VERSION))
        .send()
        .await
        .map_err(|e| e.to_string())?;

    if !res.status().is_success() {
        return Err(format!("Download failed with status {}", res.status()));
    }

    let total_bytes = res.content_length().unwrap_or(1);
    let mut downloaded_bytes: u64 = 0;

    let temp_dir = std::env::temp_dir();
    let installer_path: PathBuf = temp_dir.join("RaserNative-Update-Setup.exe");

    let mut file = File::create(&installer_path).map_err(|e| e.to_string())?;

    while let Some(chunk) = res.chunk().await.map_err(|e| e.to_string())? {
        file.write_all(&chunk).map_err(|e| e.to_string())?;
        downloaded_bytes += chunk.len() as u64;

        let percent = ((downloaded_bytes as f64 / total_bytes as f64) * 100.0).min(100.0) as u8;
        let _ = app.emit(
            "update-download-progress",
            DownloadProgress {
                percent,
                downloaded_bytes,
                total_bytes,
            },
        );
    }

    file.flush().map_err(|e| e.to_string())?;
    drop(file);

    // Launch installer
    #[cfg(target_os = "windows")]
    {
        // Execute installer
        let _ = Command::new(&installer_path).spawn();
        // Exit this process so file replacement succeeds
        std::thread::sleep(std::time::Duration::from_millis(500));
        std::process::exit(0);
    }

    #[cfg(not(target_os = "windows"))]
    {
        let _ = Command::new(&installer_path).spawn();
        std::process::exit(0);
    }
}
