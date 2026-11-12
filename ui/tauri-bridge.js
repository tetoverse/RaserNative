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

  // Wire up window controls and system features for Tauri
  window.electronAPI = Object.assign(window.electronAPI || {}, {
    isWindows: true,
    isTauri: true,
    platform: 'win32',
    minimizeWindow: () => invoke('minimize_window'),
    maximizeWindow: () => invoke('toggle_maximize_window'),
    closeWindow: () => invoke('close_window'),
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
    discordRpcUpdate: () => {},
    discordRpcClear: () => {},
    getCacheSize: async () => 0,
    clearCache: async () => {}
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
