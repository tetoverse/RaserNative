// Raser Native - Tauri v2 IPC Bridge
(function initTauriBridge() {
  const invoke = (cmd, args) => {
    if (window.__TAURI__ && window.__TAURI__.core && typeof window.__TAURI__.core.invoke === 'function') {
      return window.__TAURI__.core.invoke(cmd, args);
    }
    if (window.__TAURI_INTERNALS__ && typeof window.__TAURI_INTERNALS__.invoke === 'function') {
      return window.__TAURI_INTERNALS__.invoke(cmd, args);
    }
    return Promise.resolve();
  };

  const mediaCommandListeners = [];

  function handleMediaDispatch(cmd) {
    if (!cmd) return;
    mediaCommandListeners.forEach(fn => {
      try { fn(cmd); } catch (_) {}
    });
  }

  window.__raserMediaCommand = handleMediaDispatch;

  const setupTauriEventListener = () => {
    if (window.__TAURI__ && window.__TAURI__.event && typeof window.__TAURI__.event.listen === 'function') {
      window.__TAURI__.event.listen('media-command', (e) => {
        handleMediaDispatch(e.payload);
      });
    }
  };
  setupTauriEventListener();
  window.addEventListener('DOMContentLoaded', setupTauriEventListener);

  // Wire up window controls and system features for Tauri
  window.electronAPI = Object.assign(window.electronAPI || {}, {
    isWindows: true,
    isTauri: true,
    platform: 'win32',
    minimizeWindow: () => invoke('minimize_window'),
    maximizeWindow: () => invoke('toggle_maximize_window'),
    closeWindow: () => invoke('close_window'),
    onMediaCommand: (cb) => {
      if (typeof cb === 'function') mediaCommandListeners.push(cb);
    },
    setMediaKeys: () => {},
    mediaKeysActive: () => {},
    openExternal: (url) => {
      try {
        invoke('open_external', { url });
        return;
      } catch (_) {}
      try {
        if (window.__TAURI__ && window.__TAURI__.opener && typeof window.__TAURI__.opener.openUrl === 'function') {
          return window.__TAURI__.opener.openUrl(url);
        }
      } catch (_) {}
      window.open(url, '_blank');
    },
    getServerPort: async () => 3850,
    getGpuCompositing: async () => 'enabled',
    onWindowVisibility: (cb) => {
      document.addEventListener('visibilitychange', () => cb(!document.hidden));
    },
    setZoomFactor: () => {},
    getZoomFactor: () => 1.0,
    discordRpcUpdate: (track) => invoke('discord_rpc_update', { track }),
    discordRpcClear: () => invoke('discord_rpc_clear'),
    pickLocalFiles: async () => {
      const tracks = await invoke('pick_local_files');
      if (Array.isArray(tracks)) {
        return tracks.map(t => {
          if (t.path && window.__TAURI__ && window.__TAURI__.core && typeof window.__TAURI__.core.convertFileSrc === 'function') {
            try { t.url = window.__TAURI__.core.convertFileSrc(t.path); } catch (_) {}
          }
          return t;
        });
      }
      return tracks || [];
    },
    pickLocalFolder: async () => {
      const tracks = await invoke('pick_local_folder');
      if (Array.isArray(tracks)) {
        return tracks.map(t => {
          if (t.path && window.__TAURI__ && window.__TAURI__.core && typeof window.__TAURI__.core.convertFileSrc === 'function') {
            try { t.url = window.__TAURI__.core.convertFileSrc(t.path); } catch (_) {}
          }
          return t;
        });
      }
      return tracks || [];
    },
    rescanLocalMetadata: async () => [],
    storageInitSync: () => {
      if (window.__RASER_INITIAL_STORAGE__ && typeof window.__RASER_INITIAL_STORAGE__ === 'object') {
        return window.__RASER_INITIAL_STORAGE__;
      }
      const out = {};
      try {
        for (let i = 0; i < localStorage.length; i++) {
          const k = localStorage.key(i);
          if (k) out[k] = localStorage.getItem(k);
        }
      } catch (_) {}
      return out;
    },
    storageSaveSync: (data) => {
      invoke('save_disk_storage', { data });
    },
    storageSaveBatch: (data) => {
      invoke('save_disk_storage', { data });
    },
    updateTaskbarState: (playing) => invoke('taskbar_playback_state', { playing: !!playing }),
    setThumbarButtons: () => {},
    getCacheSize: async () => 0,
    clearCache: async () => {}
  });

  // Restore playlists, favorites, and history from disk storage into active runtime
  const applyDiskStorage = (data) => {
    if (!data || typeof data !== 'object') return;
    let changed = false;
    for (const [k, v] of Object.entries(data)) {
      try {
        const valStr = typeof v === 'string' ? v : JSON.stringify(v);
        const existing = localStorage.getItem(k);
        if (!existing || (k === 'devsize_playlists' && (existing === '[]' || existing.length < 5))) {
          localStorage.setItem(k, valStr);
          changed = true;
        }
      } catch (_) {}
    }
    if (changed) {
      try { window.dispatchEvent(new CustomEvent('raser:cloud-library-updated')); } catch (_) {}
      try { window.dispatchEvent(new CustomEvent('raser:playlist-changed')); } catch (_) {}
    }
  };

  if (window.__RASER_INITIAL_STORAGE__) {
    applyDiskStorage(window.__RASER_INITIAL_STORAGE__);
  }
  invoke('get_disk_storage').then(data => {
    if (data) {
      window.__RASER_INITIAL_STORAGE__ = data;
      applyDiskStorage(data);
    }
  }).catch(() => {});

  // Automatically sync playback state with Windows Taskbar Thumbnail Toolbar (Play/Pause button icon)
  document.addEventListener('DOMContentLoaded', () => {
    const attachAudioListeners = () => {
      const audio = document.getElementById('native-audio') || document.querySelector('audio');
      if (audio && !audio.__taskbarAttached) {
        audio.__taskbarAttached = true;
        audio.addEventListener('play', () => invoke('taskbar_playback_state', { playing: true }));
        audio.addEventListener('playing', () => invoke('taskbar_playback_state', { playing: true }));
        audio.addEventListener('pause', () => invoke('taskbar_playback_state', { playing: false }));
        audio.addEventListener('ended', () => invoke('taskbar_playback_state', { playing: false }));
      }
    };
    attachAudioListeners();
    setTimeout(attachAudioListeners, 1000);
  });

  // Enable seamless dragging for frameless Material You titlebar
  document.addEventListener('DOMContentLoaded', () => {
    const dragArea = document.querySelector('.titlebar-drag');
    if (dragArea) {
      dragArea.addEventListener('mousedown', (e) => {
        if (e.button === 0 && !e.target.closest('button, input, select, a')) {
          if (window.__TAURI__ && window.__TAURI__.window) {
            try {
              window.__TAURI__.window.getCurrentWindow().startDragging();
            } catch (_) {}
          }
        }
      });
    }
  });
})();
