// Windows Taskbar Thumbnail Toolbar (ITaskbarList3)
// Provides Spotify-style Prev, Play/Pause, Next mini media buttons on taskbar hover preview

#[cfg(target_os = "windows")]
pub mod win {
    use std::ffi::c_void;
    use std::sync::atomic::{AtomicBool, AtomicIsize, Ordering};
    use tauri::{AppHandle, Emitter, Manager};

    static APP_HANDLE: std::sync::OnceLock<AppHandle> = std::sync::OnceLock::new();
    static IS_PLAYING: AtomicBool = AtomicBool::new(false);
    static TASKBAR_LIST: AtomicIsize = AtomicIsize::new(0);
    static MAIN_HWND: AtomicIsize = AtomicIsize::new(0);

    const WM_COMMAND: u32 = 0x0111;
    const THBN_CLICKED: usize = 0x1800;

    const BTN_PREV: usize = 101;
    const BTN_PLAY_PAUSE: usize = 102;
    const BTN_NEXT: usize = 103;

    const THB_ICON: u32 = 0x0002;
    const THB_TOOLTIP: u32 = 0x0004;
    const THB_FLAGS: u32 = 0x0008;

    const THBF_ENABLED: u32 = 0x0000;

    #[repr(C)]
    struct GUID {
        data1: u32,
        data2: u16,
        data3: u16,
        data4: [u8; 8],
    }

    const CLSID_TASKBAR_LIST: GUID = GUID {
        data1: 0x56FDF344,
        data2: 0xFD6D,
        data3: 0x11D0,
        data4: [0x95, 0x8A, 0x00, 0x60, 0x97, 0xC9, 0xA0, 0x90],
    };

    const IID_ITASKBAR_LIST3: GUID = GUID {
        data1: 0xEA1AFB91,
        data2: 0x9E28,
        data3: 0x4B86,
        data4: [0x90, 0xE9, 0x9E, 0x9F, 0x8A, 0x5E, 0xEF, 0xAF],
    };

    #[repr(C)]
    #[derive(Copy, Clone)]
    struct ThumbButton {
        dw_mask: u32,
        i_id: u32,
        i_bitmap: u32,
        h_icon: isize,
        sz_tip: [u16; 260],
        dw_flags: u32,
    }

    #[repr(C)]
    struct ITaskbarList3Vtbl {
        // IUnknown
        query_interface: unsafe extern "system" fn(*mut c_void, *const GUID, *mut *mut c_void) -> i32,
        add_ref: unsafe extern "system" fn(*mut c_void) -> u32,
        release: unsafe extern "system" fn(*mut c_void) -> u32,
        // ITaskbarList
        hr_init: unsafe extern "system" fn(*mut c_void) -> i32,
        add_tab: unsafe extern "system" fn(*mut c_void, isize) -> i32,
        delete_tab: unsafe extern "system" fn(*mut c_void, isize) -> i32,
        activate_tab: unsafe extern "system" fn(*mut c_void, isize) -> i32,
        set_active_alt: unsafe extern "system" fn(*mut c_void, isize) -> i32,
        // ITaskbarList2
        mark_fullscreen_window: unsafe extern "system" fn(*mut c_void, isize, i32) -> i32,
        // ITaskbarList3
        set_progress_value: unsafe extern "system" fn(*mut c_void, isize, u64, u64) -> i32,
        set_progress_state: unsafe extern "system" fn(*mut c_void, isize, u32) -> i32,
        register_tab: unsafe extern "system" fn(*mut c_void, isize, isize) -> i32,
        unregister_tab: unsafe extern "system" fn(*mut c_void, isize) -> i32,
        set_tab_order: unsafe extern "system" fn(*mut c_void, isize, isize) -> i32,
        set_tab_active: unsafe extern "system" fn(*mut c_void, isize, isize, u32) -> i32,
        thumb_bar_add_buttons: unsafe extern "system" fn(*mut c_void, isize, u32, *const ThumbButton) -> i32,
        thumb_bar_update_buttons: unsafe extern "system" fn(*mut c_void, isize, u32, *const ThumbButton) -> i32,
    }

    #[repr(C)]
    struct ITaskbarList3 {
        vtbl: *const ITaskbarList3Vtbl,
    }

    unsafe extern "system" {
        fn CoCreateInstance(
            rclsid: *const GUID,
            p_unk_outer: *mut c_void,
            dw_cls_context: u32,
            riid: *const GUID,
            ppv: *mut *mut c_void,
        ) -> i32;
        fn CreateIcon(
            h_instance: isize,
            n_width: i32,
            n_height: i32,
            c_planes: u8,
            c_bits_pixel: u8,
            lpb_and_bits: *const u8,
            lpb_xor_bits: *const u8,
        ) -> isize;
        fn SetWindowSubclass(
            h_wnd: isize,
            pfn_subclass: unsafe extern "system" fn(isize, u32, usize, isize, usize, usize) -> isize,
            u_id_subclass: usize,
            dw_ref_data: usize,
        ) -> i32;
        fn DefSubclassProc(h_wnd: isize, u_msg: u32, w_param: usize, l_param: isize) -> isize;
    }

    fn to_sz_tip(s: &str) -> [u16; 260] {
        let mut out = [0u16; 260];
        for (i, c) in s.encode_utf16().take(259).enumerate() {
            out[i] = c;
        }
        out
    }

    // Generate crisp 20x20 1-bpp monochrome media icons directly in memory
    fn create_media_icon(shape: &str) -> isize {
        const W: usize = 20;
        const H: usize = 20;
        const STRIDE: usize = 4; // 32-bit aligned for 20 pixels = 4 bytes per row
        let mut and_mask = [0xFFu8; STRIDE * H];
        let mut xor_mask = [0x00u8; STRIDE * H];

        for y in 0..H {
            for x in 0..W {
                let mut pixel_white = false;
                match shape {
                    "prev" => {
                        // Left bar
                        if x >= 4 && x <= 5 && y >= 4 && y <= 15 {
                            pixel_white = true;
                        }
                        // Left triangle: from x=15 to x=6, centered on y=9.5
                        if x >= 6 && x <= 15 {
                            let half = ((x - 6) as f32 * 0.6) as usize;
                            if y >= 10 - half && y <= 9 + half {
                                pixel_white = true;
                            }
                        }
                    }
                    "next" => {
                        // Right bar
                        if x >= 14 && x <= 15 && y >= 4 && y <= 15 {
                            pixel_white = true;
                        }
                        // Right triangle: from x=4 to x=13
                        if x >= 4 && x <= 13 {
                            let half = ((13 - x) as f32 * 0.6) as usize;
                            if y >= 10 - half && y <= 9 + half {
                                pixel_white = true;
                            }
                        }
                    }
                    "play" => {
                        // Play triangle: from x=6 to x=15
                        if x >= 6 && x <= 15 {
                            let half = ((15 - x) as f32 * 0.65) as usize;
                            if y >= 10 - half && y <= 9 + half {
                                pixel_white = true;
                            }
                        }
                    }
                    "pause" => {
                        // Two vertical bars
                        if (x >= 6 && x <= 8) || (x >= 11 && x <= 13) {
                            if y >= 4 && y <= 15 {
                                pixel_white = true;
                            }
                        }
                    }
                    _ => {}
                }

                if pixel_white {
                    let byte_idx = y * STRIDE + (x / 8);
                    let bit_idx = 7 - (x % 8);
                    and_mask[byte_idx] &= !(1 << bit_idx); // transparent AND
                    xor_mask[byte_idx] |= 1 << bit_idx;   // white XOR
                }
            }
        }

        unsafe {
            CreateIcon(
                0,
                W as i32,
                H as i32,
                1,
                1,
                and_mask.as_ptr(),
                xor_mask.as_ptr(),
            )
        }
    }

    unsafe extern "system" fn taskbar_subclass_proc(
        hwnd: isize,
        msg: u32,
        w_param: usize,
        l_param: isize,
        _id: usize,
        _data: usize,
    ) -> isize {
        if msg == WM_COMMAND {
            let hi = (w_param >> 16) & 0xFFFF;
            let id = w_param & 0xFFFF;
            if hi == THBN_CLICKED && (id == BTN_PREV || id == BTN_PLAY_PAUSE || id == BTN_NEXT) {
                if let Some(app) = APP_HANDLE.get() {
                    match id {
                        BTN_PREV => {
                            let _ = app.emit("media-command", "prev");
                            if let Some(w) = app.get_webview_window("main") {
                                let _ = w.eval("window.__raserMediaCommand && window.__raserMediaCommand('prev');");
                            }
                        }
                        BTN_PLAY_PAUSE => {
                            let _ = app.emit("media-command", "toggle");
                            if let Some(w) = app.get_webview_window("main") {
                                let _ = w.eval("window.__raserMediaCommand && window.__raserMediaCommand('toggle');");
                            }
                        }
                        BTN_NEXT => {
                            let _ = app.emit("media-command", "next");
                            if let Some(w) = app.get_webview_window("main") {
                                let _ = w.eval("window.__raserMediaCommand && window.__raserMediaCommand('next');");
                            }
                        }
                        _ => {}
                    }
                }
                return 0;
            }
        }
        unsafe {
            DefSubclassProc(hwnd, msg, w_param, l_param)
        }
    }

    pub fn setup_taskbar_buttons(app: &AppHandle, is_tr: bool) {
        let _ = APP_HANDLE.set(app.clone());
        if let Some(window) = app.get_webview_window("main") {
            if let Ok(hwnd) = window.hwnd() {
                let hwnd_val = hwnd.0 as isize;
                MAIN_HWND.store(hwnd_val, Ordering::SeqCst);

                unsafe {
                    let mut taskbar_ptr: *mut c_void = std::ptr::null_mut();
                    let hr = CoCreateInstance(
                        &CLSID_TASKBAR_LIST,
                        std::ptr::null_mut(),
                        1, // CLSCTX_INPROC_SERVER
                        &IID_ITASKBAR_LIST3,
                        &mut taskbar_ptr,
                    );

                    if hr >= 0 && !taskbar_ptr.is_null() {
                        let taskbar = taskbar_ptr as *mut ITaskbarList3;
                        let _ = ((*(*taskbar).vtbl).hr_init)(taskbar_ptr);
                        TASKBAR_LIST.store(taskbar_ptr as isize, Ordering::SeqCst);

                        let tip_prev = if is_tr { "Önceki Şarkı" } else { "Previous Track" };
                        let tip_play = if is_tr { "Oynat / Durdur" } else { "Play / Pause" };
                        let tip_next = if is_tr { "Sonraki Şarkı" } else { "Next Track" };

                        let h_icon_prev = create_media_icon("prev");
                        let h_icon_play = create_media_icon("play");
                        let h_icon_next = create_media_icon("next");

                        let buttons: [ThumbButton; 3] = [
                            ThumbButton {
                                dw_mask: THB_ICON | THB_TOOLTIP | THB_FLAGS,
                                i_id: BTN_PREV as u32,
                                i_bitmap: 0,
                                h_icon: h_icon_prev,
                                sz_tip: to_sz_tip(tip_prev),
                                dw_flags: THBF_ENABLED,
                            },
                            ThumbButton {
                                dw_mask: THB_ICON | THB_TOOLTIP | THB_FLAGS,
                                i_id: BTN_PLAY_PAUSE as u32,
                                i_bitmap: 0,
                                h_icon: h_icon_play,
                                sz_tip: to_sz_tip(tip_play),
                                dw_flags: THBF_ENABLED,
                            },
                            ThumbButton {
                                dw_mask: THB_ICON | THB_TOOLTIP | THB_FLAGS,
                                i_id: BTN_NEXT as u32,
                                i_bitmap: 0,
                                h_icon: h_icon_next,
                                sz_tip: to_sz_tip(tip_next),
                                dw_flags: THBF_ENABLED,
                            },
                        ];

                        let _ = ((*(*taskbar).vtbl).thumb_bar_add_buttons)(
                            taskbar_ptr,
                            hwnd_val,
                            3,
                            buttons.as_ptr(),
                        );

                        // Hook window messages to capture thumbnail button clicks
                        SetWindowSubclass(hwnd_val, taskbar_subclass_proc, 4242, 0);
                    }
                }
            }
        }
    }

    pub fn update_playback_state(playing: bool, is_tr: bool) {
        IS_PLAYING.store(playing, Ordering::SeqCst);
        let taskbar_ptr = TASKBAR_LIST.load(Ordering::SeqCst);
        let hwnd_val = MAIN_HWND.load(Ordering::SeqCst);

        if taskbar_ptr != 0 && hwnd_val != 0 {
            unsafe {
                let taskbar = taskbar_ptr as *mut ITaskbarList3;
                let shape = if playing { "pause" } else { "play" };
                let tip = if playing {
                    if is_tr { "Durdur" } else { "Pause" }
                } else {
                    if is_tr { "Oynat" } else { "Play" }
                };

                let h_icon = create_media_icon(shape);
                let btn = ThumbButton {
                    dw_mask: THB_ICON | THB_TOOLTIP | THB_FLAGS,
                    i_id: BTN_PLAY_PAUSE as u32,
                    i_bitmap: 0,
                    h_icon,
                    sz_tip: to_sz_tip(tip),
                    dw_flags: THBF_ENABLED,
                };

                let _ = ((*(*taskbar).vtbl).thumb_bar_update_buttons)(
                    taskbar_ptr as *mut c_void,
                    hwnd_val,
                    1,
                    &btn,
                );
            }
        }
    }
}

#[cfg(not(target_os = "windows"))]
pub mod win {
    use tauri::AppHandle;
    pub fn setup_taskbar_buttons(_app: &AppHandle, _is_tr: bool) {}
    pub fn update_playback_state(_playing: bool, _is_tr: bool) {}
}
