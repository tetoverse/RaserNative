use discord_rich_presence::{activity, DiscordIpc, DiscordIpcClient};
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};

const CLIENT_ID: &str = "1566492575345676440";

static RPC_CLIENT: Mutex<Option<DiscordIpcClient>> = Mutex::new(None);

#[derive(serde::Deserialize, Clone, Debug)]
pub struct DiscordTrackInfo {
    pub title: Option<String>,
    pub artist: Option<String>,
    pub thumbnail: Option<String>,
    #[serde(rename = "startTimestamp")]
    pub start_timestamp: Option<i64>,
    pub duration: Option<f64>,
    pub url: Option<String>,
}

pub fn update_presence(track: DiscordTrackInfo) {
    std::thread::spawn(move || {
        let title = track.title.unwrap_or_else(|| "Unknown Track".into());
        let artist = track.artist.unwrap_or_else(|| "Unknown Artist".into());
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as i64;
        let start = track.start_timestamp.unwrap_or(now);

        let mut lock = match RPC_CLIENT.lock() {
            Ok(l) => l,
            Err(_) => return,
        };

        if lock.is_none() {
            if let Ok(mut client) = DiscordIpcClient::new(CLIENT_ID) {
                if client.connect().is_ok() {
                    *lock = Some(client);
                } else {
                    return;
                }
            } else {
                return;
            }
        }

        if let Some(client) = lock.as_mut() {
            let mut timestamps = activity::Timestamps::new().start(start);
            if let Some(dur) = track.duration {
                if dur > 0.0 {
                    timestamps = timestamps.end(start + dur as i64);
                }
            }

            let mut assets = activity::Assets::new();
            if let Some(thumb) = &track.thumbnail {
                if !thumb.is_empty() {
                    assets = assets.large_image(thumb);
                }
            }
            assets = assets.large_text(&title);

            let mut act = activity::Activity::new()
                .details(&title)
                .state(&artist)
                .timestamps(timestamps)
                .assets(assets);

            if let Some(u) = &track.url {
                if u.starts_with("https://") {
                    act = act.buttons(vec![activity::Button::new("Listen", u)]);
                }
            }

            if client.set_activity(act).is_err() {
                // If set_activity failed, disconnect and reset so next call can reconnect
                let _ = client.close();
                *lock = None;
            }
        }
    });
}

pub fn clear_presence() {
    std::thread::spawn(|| {
        let mut lock = match RPC_CLIENT.lock() {
            Ok(l) => l,
            Err(_) => return,
        };
        if let Some(client) = lock.as_mut() {
            let _ = client.clear_activity();
        }
    });
}
