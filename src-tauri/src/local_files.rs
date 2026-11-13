use std::path::{Path, PathBuf};

#[derive(serde::Serialize, Clone, Debug)]
pub struct LocalTrack {
    pub id: String,
    pub title: String,
    pub artist: String,
    pub album: String,
    pub path: String,
    pub url: String,
    pub thumbnail: String,
    pub cover: String,
    pub platform: String,
}

fn path_to_local_track(path: &Path) -> LocalTrack {
    let file_stem = path
        .file_stem()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_else(|| "Unknown Track".into());

    let (artist, title) = if let Some((a, t)) = file_stem.split_once(" - ") {
        (a.trim().to_string(), t.trim().to_string())
    } else {
        ("Unknown Artist".to_string(), file_stem)
    };

    let path_str = path.to_string_lossy().to_string();

    // Hash path for consistent unique ID
    let mut hash: u32 = 5381;
    for b in path_str.bytes() {
        hash = ((hash << 5).wrapping_add(hash)).wrapping_add(b as u32);
    }
    let id = format!("local_{:x}", hash);

    let clean_path = path_str.replace('\\', "/");
    let url = format!("http://asset.localhost/{}", clean_path);

    LocalTrack {
        id,
        title,
        artist,
        album: "".into(),
        path: path_str,
        url,
        thumbnail: "".into(),
        cover: "".into(),
        platform: "local".into(),
    }
}

pub fn pick_files() -> Vec<LocalTrack> {
    let files = rfd::FileDialog::new()
        .add_filter(
            "Audio Files",
            &["mp3", "flac", "wav", "ogg", "m4a", "aac", "opus", "wma"],
        )
        .pick_files();

    match files {
        Some(paths) => paths.iter().map(|p| path_to_local_track(p)).collect(),
        None => Vec::new(),
    }
}

fn scan_dir_recursive(dir: &Path, paths: &mut Vec<PathBuf>) {
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                scan_dir_recursive(&path, paths);
            } else if let Some(ext) = path.extension().and_then(|s| s.to_str()) {
                let ext_lower = ext.to_lowercase();
                if matches!(
                    ext_lower.as_str(),
                    "mp3" | "flac" | "wav" | "ogg" | "m4a" | "aac" | "opus" | "wma"
                ) {
                    paths.push(path);
                }
            }
        }
    }
}

pub fn pick_folder() -> Vec<LocalTrack> {
    let folder = rfd::FileDialog::new().pick_folder();
    match folder {
        Some(dir) => {
            let mut paths = Vec::new();
            scan_dir_recursive(&dir, &mut paths);
            paths.iter().map(|p| path_to_local_track(p)).collect()
        }
        None => Vec::new(),
    }
}
