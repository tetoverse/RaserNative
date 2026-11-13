// Raser Native - Mandatory Auto-Updater Client (TR / EN i18n support)
(function initUpdater() {
  const invoke = (cmd, args) => {
    if (window.__TAURI__ && window.__TAURI__.core && typeof window.__TAURI__.core.invoke === 'function') {
      return window.__TAURI__.core.invoke(cmd, args);
    }
    if (window.__TAURI_INTERNALS__ && typeof window.__TAURI_INTERNALS__.invoke === 'function') {
      return window.__TAURI_INTERNALS__.invoke(cmd, args);
    }
    return Promise.resolve();
  };

  function getLang() {
    try {
      const saved = localStorage.getItem('riff_ui_lang') || localStorage.getItem('raser_ui_lang');
      if (saved) return saved.toLowerCase().startsWith('tr') ? 'tr' : 'en';
      if (typeof navigator !== 'undefined' && navigator.language && navigator.language.toLowerCase().startsWith('tr')) {
        return 'tr';
      }
      return 'en';
    } catch (_) {
      return 'en';
    }
  }

  const I18N = {
    tr: {
      title: 'Yeni Güncelleme Mevcut!',
      desc: "Raser'ı kullanmaya devam edebilmek için güncelleme yapılması zorunludur. Güncelleme tamamlanana kadar uygulama kilitlidir.",
      notesHeader: 'Yenilikler:',
      defaultNotes: 'Yeni hata düzeltmeleri ve performans geliştirmeleri.',
      btnUpdate: 'Şimdi Güncelle ve Yeniden Başlat',
      downloading: 'İndiriliyor...',
      installing: 'Kuruluyor ve yeniden başlatılıyor...',
      downloadError: 'İndirme hatası: '
    },
    en: {
      title: 'Update Available!',
      desc: 'A mandatory update is required to continue using Raser. The application is locked until the update is complete.',
      notesHeader: "What's New:",
      defaultNotes: 'Bug fixes and performance improvements.',
      btnUpdate: 'Update Now & Restart',
      downloading: 'Downloading...',
      installing: 'Installing and restarting...',
      downloadError: 'Download error: '
    }
  };

  function createUpdateUI(updateInfo) {
    if (document.getElementById('raser-mandatory-update-modal')) return;

    const lang = getLang();
    const t = I18N[lang] || I18N.en;

    const modal = document.createElement('div');
    modal.id = 'raser-mandatory-update-modal';
    modal.style.cssText = `
      position: fixed;
      inset: 0;
      z-index: 2147483647;
      display: flex;
      align-items: center;
      justify-content: center;
      background: rgba(10, 11, 14, 0.92);
      backdrop-filter: blur(36px);
      -webkit-backdrop-filter: blur(36px);
      padding: 20px;
      user-select: none;
      font-family: 'Google Sans', system-ui, sans-serif;
    `;

    modal.innerHTML = `
      <div style="
        background: #1c1d22;
        border: 1px solid rgba(255, 255, 255, 0.12);
        border-radius: 28px;
        padding: 36px 32px;
        width: 100%;
        max-width: 460px;
        text-align: center;
        box-shadow: 0 24px 64px rgba(0, 0, 0, 0.75);
        display: flex;
        flex-direction: column;
        align-items: center;
        gap: 16px;
      ">
        <div style="
          width: 64px;
          height: 64px;
          border-radius: 50%;
          background: rgba(168, 199, 250, 0.15);
          display: flex;
          align-items: center;
          justify-content: center;
          color: #a8c7fa;
          margin-bottom: 4px;
        ">
          <svg width="32" height="32" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.2" stroke-linecap="round" stroke-linejoin="round">
            <path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"></path>
            <polyline points="7 10 12 15 17 10"></polyline>
            <line x1="12" y1="15" x2="12" y2="3"></line>
          </svg>
        </div>

        <h2 style="font-size: 22px; font-weight: 700; color: #fff; margin: 0;">
          ${t.title}
        </h2>

        <div style="
          display: inline-flex;
          align-items: center;
          gap: 6px;
          background: rgba(255, 255, 255, 0.08);
          border: 1px solid rgba(255, 255, 255, 0.1);
          padding: 4px 14px;
          border-radius: 9999px;
          font-size: 13px;
          font-weight: 600;
          color: #a8c7fa;
        ">
          <span>v${updateInfo.current_version}</span>
          <span style="opacity: 0.5;">→</span>
          <span style="color: #6dd5ed;">v${updateInfo.latest_version}</span>
        </div>

        <p style="
          font-size: 14px;
          color: #c4c6cf;
          line-height: 1.5;
          margin: 0;
        ">
          ${t.desc}
        </p>

        <div style="
          width: 100%;
          max-height: 110px;
          overflow-y: auto;
          background: rgba(0, 0, 0, 0.3);
          border: 1px solid rgba(255, 255, 255, 0.06);
          border-radius: 14px;
          padding: 12px;
          text-align: left;
          font-size: 12px;
          color: #9aa0a6;
          line-height: 1.4;
        ">
          <strong style="color: #e2e2e6; display: block; margin-bottom: 4px;">${t.notesHeader}</strong>
          ${updateInfo.notes || t.defaultNotes}
        </div>

        <div id="updater-progress-container" style="display: none; width: 100%; flex-direction: column; gap: 8px;">
          <div style="display: flex; justify-content: space-between; font-size: 13px; color: #a8c7fa;">
            <span id="updater-progress-label">${t.downloading}</span>
            <span id="updater-progress-percent">0%</span>
          </div>
          <div style="width: 100%; height: 8px; background: rgba(255, 255, 255, 0.1); border-radius: 9999px; overflow: hidden;">
            <div id="updater-progress-bar" style="width: 0%; height: 100%; background: #a8c7fa; border-radius: 9999px; transition: width 0.15s ease;"></div>
          </div>
        </div>

        <button id="btn-do-update" type="button" style="
          width: 100%;
          padding: 14px 20px;
          background: #a8c7fa;
          color: #04315a;
          border: none;
          border-radius: 9999px;
          font-size: 15px;
          font-weight: 700;
          cursor: pointer;
          transition: transform 0.15s ease, opacity 0.15s ease;
          display: flex;
          align-items: center;
          justify-content: center;
          gap: 8px;
          margin-top: 6px;
        ">
          ${t.btnUpdate}
        </button>
      </div>
    `;

    document.body.appendChild(modal);

    window.addEventListener('keydown', (e) => {
      e.stopImmediatePropagation();
    }, true);

    const btnUpdate = document.getElementById('btn-do-update');
    const progressBox = document.getElementById('updater-progress-container');
    const progressBar = document.getElementById('updater-progress-bar');
    const progressPercent = document.getElementById('updater-progress-percent');
    const progressLabel = document.getElementById('updater-progress-label');

    btnUpdate.addEventListener('click', async () => {
      btnUpdate.disabled = true;
      btnUpdate.style.opacity = '0.6';
      btnUpdate.style.cursor = 'not-allowed';
      progressBox.style.display = 'flex';

      if (window.__TAURI__ && window.__TAURI__.event) {
        try {
          window.__TAURI__.event.listen('update-download-progress', (e) => {
            const data = e.payload;
            if (data && typeof data.percent === 'number') {
              progressBar.style.width = `${data.percent}%`;
              progressPercent.textContent = `${data.percent}%`;
              if (data.percent >= 100) {
                progressLabel.textContent = t.installing;
              }
            }
          });
        } catch (_) {}
      }

      try {
        await invoke('install_update', { downloadUrl: updateInfo.download_url });
      } catch (err) {
        progressLabel.textContent = t.downloadError + err;
        progressLabel.style.color = '#ff5449';
        btnUpdate.disabled = false;
        btnUpdate.style.opacity = '1';
        btnUpdate.style.cursor = 'pointer';
      }
    });
  }

  async function checkMandatoryUpdate() {
    try {
      const res = await invoke('check_update');
      if (res && res.should_update && res.mandatory) {
        createUpdateUI(res);
      }
    } catch (_) {}
  }

  // Run on startup
  if (document.readyState === 'loading') {
    document.addEventListener('DOMContentLoaded', () => {
      setTimeout(checkMandatoryUpdate, 1500);
    });
  } else {
    setTimeout(checkMandatoryUpdate, 1500);
  }
})();
