//! Hidden HWND integration tests. No input hooks, foreground changes, or tray icons.
use super::*;
use std::sync::{atomic::*, Mutex, RwLock};

#[test]
fn hidden_native_preset_rows_follow_live_dpi_changes() {
    unsafe {
        let _graphics = GraphicsToken::start().unwrap();
        let ui = hidden(1.);
        let mut rect: RECT = zeroed();
        GetWindowRect(ui.hwnd as HWND, &mut rect);
        SendMessageW(
            ui.hwnd as HWND,
            WM_DPICHANGED,
            192 | (192 << 16),
            (&rect as *const RECT) as isize,
        );
        assert_eq!(
            SendMessageW(ui.handle(3003) as HWND, CB_GETITEMHEIGHT, usize::MAX, 0),
            56
        );
        assert_eq!(
            SendMessageW(ui.handle(3003) as HWND, CB_GETITEMHEIGHT, 0, 0),
            56
        );
        assert_eq!(
            SendMessageW(ui.handle(PRESET) as HWND, CB_GETITEMHEIGHT, usize::MAX, 0),
            56
        );
        assert_eq!(
            SendMessageW(ui.handle(PRESET) as HWND, CB_GETITEMHEIGHT, 0, 0),
            64
        );
        assert_eq!(IsWindowVisible(ui.hwnd as HWND), 0);
        DestroyWindow(ui.hwnd as HWND);
        DeleteObject(ui.brush as HGDIOBJ);
    }
}

#[test]
fn hidden_native_profile_errors_preserve_saved_and_active_settings() {
    unsafe {
        let _graphics = GraphicsToken::start().unwrap();
        let mut ui = hidden(1.);
        set(ui.handle(3002), "Editor.exe");
        set(ui.handle(3004), "72");
        set(ui.handle(3005), "35");
        SendMessageW(ui.hwnd as HWND, WM_COMMAND, 3006, 0);
        let good_path = ui.path.clone();
        let original = std::fs::read(&good_path).unwrap();
        set(ui.handle(3004), "0");
        SendMessageW(ui.hwnd as HWND, WM_COMMAND, 3006, 0);
        assert_eq!(std::fs::read(&good_path).unwrap(), original);
        assert_eq!(
            ui.shared
                .settings
                .read()
                .unwrap()
                .for_app("Editor.exe")
                .distance,
            72.
        );
        set(ui.handle(3004), "80");
        ui.path = good_path.join("missing/settings.json");
        SendMessageW(ui.hwnd as HWND, WM_COMMAND, 3006, 0);
        SendMessageW(ui.hwnd as HWND, WM_COMMAND, 3007, 0);
        assert_eq!(ui.shared.settings.read().unwrap().profiles.len(), 1);
        assert_eq!(std::fs::read(&good_path).unwrap(), original);
        std::fs::remove_file(good_path).unwrap();
        DestroyWindow(ui.hwnd as HWND);
        DeleteObject(ui.brush as HGDIOBJ);
    }
}

#[test]
fn hidden_native_new_pages_fit_and_render_at_supported_scales() {
    unsafe {
        let _graphics = GraphicsToken::start().unwrap();
        for scale in [1., 1.25, 1.5, 1.75, 2.] {
            for width in [500, 940, 1140] {
                let mut ui = hidden(scale);
                let hwnd = ui.hwnd as HWND;
                SetWindowPos(
                    hwnd,
                    null_mut(),
                    0,
                    0,
                    ui.px(width),
                    ui.px(768),
                    SWP_NOMOVE | SWP_NOZORDER | SWP_NOACTIVATE,
                );
                ui.layout();
                set(ui.handle(3002), "Editor.exe");
                SendMessageW(hwnd, WM_COMMAND, 3006, 0);
                for (page, name) in [(NAV_PROFILES, "app-profiles"), (NAV_ACTIVITY, "activity")] {
                    ui.navigate(page);
                    let surface = Surface::new(ui.px(ui.width), ui.px(ui.height));
                    ui.paint_main(surface.dc);
                    let saved = SaveDC(surface.dc);
                    SetViewportOrgEx(
                        surface.dc,
                        ui.px(ui.shell().body.x),
                        ui.px(ui.shell().body.y),
                        null_mut(),
                    );
                    ui.paint_body(surface.dc);
                    RestoreDC(surface.dc, saved);
                    for &(id, handle, in_body) in &ui.controls {
                        if GetWindowLongW(handle as HWND, GWL_STYLE) as u32 & WS_VISIBLE == 0 {
                            continue;
                        }
                        let mut r: RECT = zeroed();
                        GetWindowRect(handle as HWND, &mut r);
                        let mut origin = POINT {
                            x: r.left,
                            y: r.top,
                        };
                        ScreenToClient(hwnd, &mut origin);
                        if in_body {
                            assert!(
                                origin.x >= ui.px(ui.shell().body.x),
                                "Content must stay to the right of navigation"
                            );
                            assert!(origin.x + r.right - r.left <= ui.px(ui.width));
                        }
                        let child =
                            Surface::new((r.right - r.left).max(1), (r.bottom - r.top).max(1));
                        SendMessageW(handle as HWND, WM_ERASEBKGND, child.dc as usize, 0);
                        SendMessageW(
                            handle as HWND,
                            WM_PRINTCLIENT,
                            child.dc as usize,
                            PRF_CLIENT as isize,
                        );
                        if id == 3003 {
                            GdiFlush();
                            let pixels = std::slice::from_raw_parts(
                                child.pixels,
                                (child.w * child.h) as usize,
                            );
                            assert!(
                                pixels.iter().filter(|p| **p & 0xffffff == WHITE).count()
                                    > pixels.len() / 2,
                                "App profile selector must paint an opaque light field"
                            );
                        }
                        let saved = SaveDC(surface.dc);
                        if in_body {
                            let b = ui.shell().body;
                            IntersectClipRect(
                                surface.dc,
                                ui.px(b.x),
                                ui.px(b.y),
                                ui.px(b.x + b.w),
                                ui.px(b.y + b.h),
                            );
                        }
                        BitBlt(
                            surface.dc, origin.x, origin.y, child.w, child.h, child.dc, 0, 0,
                            SRCCOPY,
                        );
                        RestoreDC(surface.dc, saved);
                    }
                    surface.save(&std::env::current_dir().unwrap().join(format!(
                        "target/headless-ui/{name}-{}-{width}.bmp",
                        (scale * 100.) as i32
                    )));
                    if page == NAV_PROFILES {
                        ui.scroll = 10000;
                        ui.layout();
                        let mut r: RECT = zeroed();
                        GetWindowRect(ui.handle(3012) as HWND, &mut r);
                        let mut bottom = POINT {
                            x: r.right,
                            y: r.bottom,
                        };
                        ScreenToClient(ui.body as HWND, &mut bottom);
                        assert!(
                            bottom.y <= ui.px(ui.shell().body.h),
                            "Last profile help must be reachable"
                        );
                        ui.scroll = 0;
                    }
                }
                assert_eq!(IsWindowVisible(hwnd), 0);
                std::fs::remove_file(&ui.path).unwrap();
                DestroyWindow(hwnd);
                DeleteObject(ui.brush as HGDIOBJ);
            }
        }
    }
}

#[test]
fn hidden_native_app_profiles_save_without_committing_global_drafts() {
    unsafe {
        let _graphics = GraphicsToken::start().unwrap();
        let ui = hidden(1.);
        assert_ne!(ui.handle(3002), 0, "The app profile editor must exist");
        set(ui.inputs[0], "177");
        set(ui.handle(3002), "Editor.exe");
        set(ui.handle(3004), "72");
        set(ui.handle(3005), "35");
        SendMessageW(ui.hwnd as HWND, WM_COMMAND, 3006, 0);
        let saved = settings::load(&ui.path).unwrap();
        assert_eq!(saved.distance, 120.);
        assert_eq!(saved.for_app("EDITOR.EXE").distance, 72.);
        assert_eq!(
            ui.shared
                .settings
                .read()
                .unwrap()
                .for_app("Editor.exe")
                .omega,
            35.
        );
        assert_eq!(text(ui.inputs[0]), "177");
        SendMessageW(ui.handle(3003) as HWND, CB_SETCURSEL, 2, 0);
        SendMessageW(
            ui.hwnd as HWND,
            WM_COMMAND,
            3003 | ((CBN_SELCHANGE as usize) << 16),
            0,
        );
        SendMessageW(ui.hwnd as HWND, WM_COMMAND, 3006, 0);
        let saved = settings::load(&ui.path).unwrap();
        assert_eq!(saved.for_app("Editor.exe").omega, SMOOTH_OMEGA);
        assert_eq!(IsWindowEnabled(ui.handle(3004) as HWND), 0);
        SendMessageW(ui.hwnd as HWND, WM_COMMAND, 3007, 0);
        assert!(settings::load(&ui.path).unwrap().profiles.is_empty());
        std::fs::remove_file(&ui.path).unwrap();
        DestroyWindow(ui.hwnd as HWND);
        DeleteObject(ui.brush as HGDIOBJ);
    }
}

#[test]
fn hidden_native_maximize_exposes_the_snap_hit_target() {
    unsafe {
        let _graphics = GraphicsToken::start().unwrap();
        let ui = hidden(1.);
        let mut rect: RECT = zeroed();
        GetWindowRect(ui.handle(WINDOW_MAXIMIZE) as HWND, &mut rect);
        let x = (rect.left + rect.right) / 2;
        let y = (rect.top + rect.bottom) / 2;
        let point = ((x as u16 as u32) | ((y as u16 as u32) << 16)) as isize;
        assert_eq!(
            SendMessageW(ui.handle(WINDOW_MAXIMIZE) as HWND, WM_NCHITTEST, 0, point),
            HTMAXBUTTON as isize
        );
        assert_eq!(
            SendMessageW(ui.hwnd as HWND, WM_NCHITTEST, 0, point),
            HTMAXBUTTON as isize
        );
        let mut bounds: MINMAXINFO = zeroed();
        SendMessageW(
            ui.hwnd as HWND,
            WM_GETMINMAXINFO,
            0,
            (&mut bounds as *mut MINMAXINFO) as isize,
        );
        assert!(bounds.ptMinTrackSize.x <= 500);
        DestroyWindow(ui.hwnd as HWND);
        DeleteObject(ui.brush as HGDIOBJ);
    }
}

#[test]
fn hidden_native_fixed_presets_apply_and_default_remembers_custom_tuning() {
    unsafe {
        let _graphics = GraphicsToken::start().unwrap();
        let mut ui = hidden(1.);
        assert!(!text(ui.handle(1110)).contains("Custom"));
        set(ui.inputs[0], "177");
        assert!(text(ui.handle(1110)).contains("Custom"));
        ui.apply();
        assert!(text(ui.handle(1110)).contains("Custom"));
        check(ui.reverse, true); // Unrelated option draft must not be auto-applied.
        let combo = ui.handle(PRESET) as HWND;
        SendMessageW(combo, CB_SETCURSEL, 2, 0);
        ui.select_preset();
        assert_eq!(
            ui.shared.settings.read().unwrap().preset,
            Preset::SuperSmooth
        );
        assert_eq!(settings::load(&ui.path).unwrap().omega, SMOOTH_OMEGA);
        assert!(ui.shared.settings.read().unwrap().wheel().free_spin);
        assert!(text(ui.handle(1110)).contains("Flick to coast"));
        assert!(!ui.shared.settings.read().unwrap().reverse);
        assert_eq!(IsWindowEnabled(ui.inputs[0] as HWND), 0);
        let before = text(ui.inputs[0]);
        ui.step(400);
        assert_eq!(text(ui.inputs[0]), before);
        SendMessageW(combo, CB_SETCURSEL, 0, 0);
        ui.select_preset();
        assert_eq!(text(ui.inputs[0]), "177");
        assert!(text(ui.handle(1110)).contains("Custom"));
        assert_eq!(ui.shared.settings.read().unwrap().distance, 177.);
        assert!(!ui.shared.settings.read().unwrap().wheel().free_spin);
        assert_ne!(IsWindowEnabled(ui.inputs[0] as HWND), 0);
        assert!(checked(ui.reverse));
        std::fs::remove_file(&ui.path).unwrap();
        DestroyWindow(ui.hwnd as HWND);
        DeleteObject(ui.brush as HGDIOBJ);
    }
}

#[test]
fn hidden_native_failed_preset_save_keeps_the_active_preset() {
    unsafe {
        let _graphics = GraphicsToken::start().unwrap();
        let mut ui = hidden(1.);
        ui.path = ui.path.join("missing/settings.json");
        SendMessageW(ui.handle(PRESET) as HWND, CB_SETCURSEL, 1, 0);
        ui.select_preset();
        assert_eq!(ui.preset, Preset::Default);
        assert_eq!(
            SendMessageW(ui.handle(PRESET) as HWND, CB_GETCURSEL, 0, 0),
            0
        );
        assert_eq!(ui.shared.settings.read().unwrap().preset, Preset::Default);
        DestroyWindow(ui.hwnd as HWND);
        DeleteObject(ui.brush as HGDIOBJ);
    }
}

/// Super smooth's fixed response, as the preset table defines it.
const SMOOTH_OMEGA: f64 = 12.;

unsafe fn shared() -> Arc<Shared> {
    Arc::new(Shared {
        settings: RwLock::new(Settings::default()),
        engine: Mutex::new(Default::default()),
        enabled: AtomicBool::new(false),
        quitting: AtomicBool::new(false),
        interrupted: AtomicBool::new(false),
        target: RwLock::new(Default::default()),
        ui: AtomicUsize::new(0),
        pad: AtomicUsize::new(0),
        latch: AtomicUsize::new(0),
        shift_latch: AtomicBool::new(false),
        queued: Default::default(),
        wake: Handle(0),
        epoch: std::time::Instant::now(),
        hook_thread: AtomicU32::new(0),
        hook_live: AtomicBool::new(false),
        sent: AtomicU64::new(0),
        ticks: AtomicU64::new(0),
        hook_max_us: AtomicU64::new(0),
        fault: Mutex::new(String::new()),
        diagnostics: scrollpilot::diagnostics::Diagnostics::default(),
    })
}

unsafe fn hidden(scale: f64) -> Box<Ui> {
    let hinst = GetModuleHandleW(null());
    let classes: [(&str, WNDPROC); 4] = [
        ("ScrollMage.Headless", Some(window)),
        ("ScrollMage.HeadlessBody", Some(body)),
        ("ScrollPilot.TestPad", Some(preview::pad)),
        ("ScrollMage.Exclusions", Some(exclusions::panel)),
    ];
    for (name, proc) in classes {
        RegisterClassW(&WNDCLASSW {
            lpfnWndProc: proc,
            hInstance: hinst,
            lpszClassName: wide(name).as_ptr(),
            ..zeroed()
        });
    }
    let brush = CreateSolidBrush(color(WHITE));
    let output = std::env::current_dir().unwrap().join("target/headless-ui");
    std::fs::create_dir_all(&output).unwrap();
    let path = output.join(format!("scroll-mage-ui-{}.json", std::process::id()));
    let mut ui =
        Box::new(Ui::new(shared(), path, String::new(), scale, brush as usize, 0).unwrap());
    let hwnd = CreateWindowExW(
        WS_EX_CONTROLPARENT,
        wide("ScrollMage.Headless").as_ptr(),
        wide("Scroll Mage hidden test").as_ptr(),
        WS_OVERLAPPEDWINDOW | WS_CLIPCHILDREN,
        0,
        0,
        ui.px(DEFAULT_WIDTH),
        ui.px(DEFAULT_HEIGHT),
        null_mut(),
        null_mut(),
        hinst,
        (&mut *ui as *mut Ui).cast(),
    );
    assert!(!hwnd.is_null());
    SetWindowPos(
        hwnd,
        null_mut(),
        0,
        0,
        0,
        0,
        SWP_NOMOVE | SWP_NOSIZE | SWP_NOZORDER | SWP_NOACTIVATE | SWP_FRAMECHANGED,
    );
    ui.body = CreateWindowExW(
        WS_EX_CONTROLPARENT,
        wide("ScrollMage.HeadlessBody").as_ptr(),
        wide("Settings content").as_ptr(),
        WS_CHILD | WS_VISIBLE | WS_VSCROLL | WS_CLIPCHILDREN,
        0,
        0,
        1,
        1,
        hwnd,
        null_mut(),
        hinst,
        null(),
    ) as usize;
    ui.build();
    assert_eq!(IsWindowVisible(hwnd), 0);
    ui
}

struct Surface {
    dc: HDC,
    bitmap: HBITMAP,
    old: HGDIOBJ,
    pixels: *mut u32,
    w: i32,
    h: i32,
}

#[test]
fn hidden_native_app_icons_are_embedded_at_the_requested_sizes() {
    unsafe {
        let module = GetModuleHandleW(null());
        for resource in [101u16, 102] {
            for size in [16, 20, 24, 28, 32, 40, 48, 64] {
                let icon = LoadImageW(
                    module,
                    resource as usize as *const u16,
                    IMAGE_ICON,
                    size,
                    size,
                    LR_DEFAULTCOLOR,
                ) as HICON;
                assert!(
                    !icon.is_null(),
                    "Embedded icon {resource} at {size}px must load from the executable"
                );
                let mut info: ICONINFO = zeroed();
                assert_ne!(GetIconInfo(icon, &mut info), 0);
                let mut bitmap: BITMAP = zeroed();
                GetObjectW(
                    info.hbmColor,
                    size_of::<BITMAP>() as i32,
                    (&mut bitmap as *mut BITMAP).cast(),
                );
                assert_eq!((bitmap.bmWidth, bitmap.bmHeight), (size, size));
                DeleteObject(info.hbmColor);
                DeleteObject(info.hbmMask);
                let surface = Surface::new(size, size);
                fill(surface.dc, Rect::new(0, 0, size, size), 1., 0x793e75);
                assert_ne!(
                    DrawIconEx(surface.dc, 0, 0, icon, size, size, 0, null_mut(), DI_NORMAL),
                    0
                );
                assert_eq!(
                    surface.pixel(0, 0),
                    0x793e75,
                    "The squircle must have transparent corners"
                );
                let white = (0..size)
                    .flat_map(|y| (0..size).map(move |x| (x, y)))
                    .filter(|&(x, y)| {
                        let p = surface.pixel(x, y);
                        (p & 255) > 220 && (p >> 8 & 255) > 220 && (p >> 16 & 255) > 220
                    })
                    .count();
                assert!(
                    white > (size * size / 50) as usize,
                    "The white mouse must remain visible at {size}px"
                );
                DestroyIcon(icon);
            }
        }
        let _graphics = GraphicsToken::start().unwrap();
        let mut ui = hidden(1.);
        ui.ready = false; // Exercise icon assignment without notifying the real tray.
        for scale in [1., 1.25, 1.5, 1.75, 2.] {
            ui.scale = scale;
            ui.refresh_icons();
            let icons = ui
                .app_icons
                .as_ref()
                .expect("The UI must retain ownership of its icons");
            assert_eq!(
                SendMessageW(ui.hwnd as HWND, WM_GETICON, ICON_BIG as usize, 0),
                icons.large as isize
            );
            assert_eq!(
                SendMessageW(ui.hwnd as HWND, WM_GETICON, ICON_SMALL as usize, 0),
                icons.small as isize
            );
            assert_eq!(ui.icon, icons.tray as usize);
            let mut info: ICONINFO = zeroed();
            GetIconInfo(icons.large, &mut info);
            let mut bitmap: BITMAP = zeroed();
            GetObjectW(
                info.hbmColor,
                size_of::<BITMAP>() as i32,
                (&mut bitmap as *mut BITMAP).cast(),
            );
            assert_eq!(bitmap.bmWidth, ui.px(32));
            DeleteObject(info.hbmColor);
            DeleteObject(info.hbmMask);
        }
        SetClassLongPtrW(ui.hwnd as HWND, GCLP_HICON, 0);
        DestroyWindow(ui.hwnd as HWND);
        DeleteObject(ui.brush as HGDIOBJ);
    }
}

#[test]
fn hidden_native_pill_paint_stays_inside_its_client_surface() {
    unsafe {
        let _graphics = GraphicsToken::start().unwrap();
        let mut faults = Vec::new();
        let mut audit = Vec::new();
        for scale in [1., 1.25, 1.5, 1.75, 2.] {
            let ui = hidden(scale);
            let combo = ui.handle(PRESET) as HWND;
            let mut r: RECT = zeroed();
            GetClientRect(combo, &mut r);
            let surface = Surface::new(r.right + 8, r.bottom + 8);
            let brush = CreateSolidBrush(color(WHITE));
            let canvas = RECT {
                left: 0,
                top: 0,
                right: surface.w,
                bottom: surface.h,
            };
            FillRect(surface.dc, &canvas, brush);
            DeleteObject(brush);
            SetViewportOrgEx(surface.dc, 4, 4, null_mut());
            SendMessageW(
                combo,
                WM_PRINTCLIENT,
                surface.dc as usize,
                PRF_CLIENT as isize,
            );
            GdiFlush();
            let before =
                std::slice::from_raw_parts(surface.pixels, (surface.w * surface.h) as usize)
                    .to_vec();
            let mut info: windows_sys::Win32::UI::Controls::COMBOBOXINFO = zeroed();
            info.cbSize = size_of_val(&info) as u32;
            windows_sys::Win32::UI::Controls::GetComboBoxInfo(combo, &mut info);
            ui.draw_preset_item(&DRAWITEMSTRUCT {
                hwndItem: combo,
                hDC: surface.dc,
                itemID: 0,
                itemState: windows_sys::Win32::UI::Controls::ODS_COMBOBOXEDIT | ODS_SELECTED,
                rcItem: info.rcItem,
                ..zeroed()
            });
            GdiFlush();
            if before
                != std::slice::from_raw_parts(surface.pixels, (surface.w * surface.h) as usize)
            {
                faults.push(format!(
                    "{}%: native collapsed-item redraw overwrites the custom capsule",
                    scale * 100.
                ));
            }
            SetViewportOrgEx(surface.dc, 0, 0, null_mut());
            for y in 0..surface.h {
                if (0..surface.w).any(|x| {
                    (x < 4 || x >= r.right + 4 || y < 4 || y >= r.bottom + 4)
                        && surface.pixel(x, y) != WHITE
                }) {
                    faults.push(format!(
                        "{}%: pill paints outside its client bounds on guard row {y} (height {})",
                        scale * 100.,
                        r.bottom
                    ));
                }
            }
            audit.push(format!(
                "scale={} client={}x{} style={:x} exstyle={:x}",
                scale,
                r.right,
                r.bottom,
                GetWindowLongW(combo, GWL_STYLE),
                GetWindowLongW(combo, GWL_EXSTYLE)
            ));
            let mut child = GetWindow(ui.body as HWND, GW_CHILD);
            while !child.is_null() {
                let mut bounds: RECT = zeroed();
                GetWindowRect(child, &mut bounds);
                audit.push(format!(
                    "layer id={} bounds={},{},{},{}",
                    GetDlgCtrlID(child),
                    bounds.left,
                    bounds.top,
                    bounds.right,
                    bounds.bottom
                ));
                child = GetWindow(child, GW_HWNDNEXT);
            }
            surface.save(&std::env::current_dir().unwrap().join(format!(
                "target/headless-ui/pill-guard-{}.bmp",
                (scale * 100.) as i32
            )));
            DestroyWindow(ui.hwnd as HWND);
            DeleteObject(ui.brush as HGDIOBJ);
        }
        std::fs::write("target/headless-ui/pill-layer-audit.txt", audit.join("\n")).unwrap();
        assert!(
            faults.is_empty(),
            "The rounded stroke would be clipped by the HWND:\n{}",
            faults.join("\n")
        );
    }
}

#[test]
fn hidden_native_header_controls_do_not_overlap() {
    unsafe {
        let _graphics = GraphicsToken::start().unwrap();
        let mut overlaps = Vec::new();
        for scale in [1., 1.25, 1.5, 1.75, 2.] {
            let mut ui = hidden(scale);
            let root = ui.hwnd as HWND;
            for (width, height) in [(1140, 768), (1100, 760), (940, 680), (1280, 920)] {
                SetWindowPos(
                    root,
                    null_mut(),
                    0,
                    0,
                    ui.px(width),
                    ui.px(height),
                    SWP_NOMOVE | SWP_NOZORDER | SWP_NOACTIVATE,
                );
                ui.layout();
                let rect = |id| {
                    let mut r: RECT = zeroed();
                    GetWindowRect(ui.handle(id) as HWND, &mut r);
                    r
                };
                let pair = |a: RECT, b: RECT| {
                    a.left < b.right && b.left < a.right && a.top < b.bottom && b.top < a.bottom
                };
                let (entry, _) = ui.shell().exclusion_entry();
                let mut origin = POINT { x: 0, y: 0 };
                ClientToScreen(ui.body as HWND, &mut origin);
                let entry = RECT {
                    left: origin.x + ui.px(entry.x),
                    top: origin.y + ui.px(entry.y - ui.scroll),
                    right: origin.x + ui.px(entry.x + entry.w),
                    bottom: origin.y + ui.px(entry.y - ui.scroll + entry.h),
                };
                for (name, a, b) in [
                    ("Profile pill and helper text", rect(PRESET), rect(1110)),
                    ("EXE entry frame and heading", entry, rect(1102)),
                    ("Add button and heading", rect(EXCLUDE_ADD), rect(1102)),
                    ("EXE entry frame and app list", entry, rect(EXCLUSION_PANEL)),
                ] {
                    if pair(a, b) {
                        overlaps.push(format!("{name}: {width}x{height} at {}%", scale * 100.));
                    }
                }
                let edit = rect(215);
                assert!(
                    edit.top > entry.top && edit.bottom < entry.bottom,
                    "The edit's native surface must leave the frame border visible"
                );
                assert_eq!(IsWindowVisible(root), 0);
            }
            DestroyWindow(root);
            DeleteObject(ui.brush as HGDIOBJ);
        }
        assert!(
            overlaps.is_empty(),
            "Overlapping native surfaces clip the painted controls:\n{}",
            overlaps.join("\n")
        );
    }
}
impl Surface {
    unsafe fn new(w: i32, h: i32) -> Self {
        let dc = CreateCompatibleDC(null_mut());
        let info = BITMAPINFO {
            bmiHeader: BITMAPINFOHEADER {
                biSize: size_of::<BITMAPINFOHEADER>() as u32,
                biWidth: w,
                biHeight: -h,
                biPlanes: 1,
                biBitCount: 32,
                biCompression: BI_RGB,
                ..zeroed()
            },
            ..zeroed()
        };
        let mut pixels = null_mut();
        let bitmap = CreateDIBSection(dc, &info, DIB_RGB_COLORS, &mut pixels, null_mut(), 0);
        assert!(!bitmap.is_null() && !pixels.is_null());
        let old = SelectObject(dc, bitmap);
        Self {
            dc,
            bitmap,
            old,
            pixels: pixels.cast(),
            w,
            h,
        }
    }
    unsafe fn pixel(&self, x: i32, y: i32) -> u32 {
        GdiFlush();
        *self.pixels.offset((y * self.w + x) as isize) & 0xffffff
    }
    unsafe fn save(&self, path: &std::path::Path) {
        GdiFlush();
        let size = (self.w * self.h * 4) as u32;
        let mut bytes = Vec::new();
        bytes.extend_from_slice(b"BM");
        bytes.extend_from_slice(&(size + 54).to_le_bytes());
        bytes.extend_from_slice(&[0; 4]);
        bytes.extend_from_slice(&54u32.to_le_bytes());
        bytes.extend_from_slice(&40u32.to_le_bytes());
        bytes.extend_from_slice(&self.w.to_le_bytes());
        bytes.extend_from_slice(&(-self.h).to_le_bytes());
        bytes.extend_from_slice(&1u16.to_le_bytes());
        bytes.extend_from_slice(&32u16.to_le_bytes());
        bytes.extend_from_slice(&[0; 24]);
        bytes.extend_from_slice(std::slice::from_raw_parts(
            self.pixels.cast::<u8>(),
            size as usize,
        ));
        std::fs::write(path, bytes).unwrap();
    }
}
impl Drop for Surface {
    fn drop(&mut self) {
        unsafe {
            SelectObject(self.dc, self.old);
            DeleteObject(self.bitmap);
            DeleteDC(self.dc);
        }
    }
}

#[test]
fn hidden_native_controls_keep_tuning_layout_and_pixels_consistent() {
    unsafe {
        let _graphics = GraphicsToken::start().unwrap();
        for scale in [1., 1.25, 1.5, 1.75, 2.] {
            let mut ui = hidden(scale);
            let hwnd = ui.hwnd as HWND;
            // Add must persist and update runtime policy without applying other drafts.
            let initial_count = ui.excluded_apps.len();
            set(ui.inputs[0], "177");
            set(ui.exclusion_input, "Game.exe");
            SendMessageW(ui.handle(EXCLUDE_ADD) as HWND, BM_CLICK, 0, 0);
            assert!(
                ui.shared.settings.read().unwrap().excluded("game.exe"),
                "Add must update the active exclusion policy immediately"
            );
            assert!(settings::load(&ui.path).unwrap().excluded("Game.exe"));
            assert_eq!(ui.shared.settings.read().unwrap().distance, 120.);
            assert_eq!(text(ui.inputs[0]), "177");
            set(ui.exclusion_input, "\"C:\\Games\\GAME.exe\"");
            SendMessageW(
                ui.exclusion_input as HWND,
                WM_KEYDOWN,
                VK_RETURN as usize,
                0,
            );
            assert_eq!(
                ui.excluded_apps.len(),
                initial_count + 1,
                "Paths and names should deduplicate ignoring case"
            );
            set(ui.exclusion_input, "\"C:\\Games\\Other Game.exe\"");
            SendMessageW(
                ui.exclusion_input as HWND,
                WM_KEYDOWN,
                VK_RETURN as usize,
                0,
            );
            assert!(settings::load(&ui.path).unwrap().excluded("Other Game.exe"));
            let good_path = ui.path.clone();
            ui.path = good_path.join("missing/settings.json");
            set(ui.exclusion_input, "Fail.exe");
            ui.add_exclusion();
            assert_eq!(
                ui.excluded_apps.len(),
                initial_count + 2,
                "Failed writes must preserve UI state"
            );
            assert!(!ui.shared.settings.read().unwrap().excluded("Fail.exe"));
            ui.remove_exclusion(initial_count);
            assert!(ui.shared.settings.read().unwrap().excluded("Game.exe"));
            ui.path = good_path;
            set(ui.exclusion_input, "not-an-exe.txt");
            ui.add_exclusion();
            assert_eq!(ui.excluded_apps.len(), initial_count + 2);
            SendMessageW(
                ui.handle(EXCLUSION_ROW + initial_count as u16 * 2 + 1) as HWND,
                BM_CLICK,
                0,
                0,
            );
            SendMessageW(
                ui.handle(EXCLUSION_ROW + initial_count as u16 * 2 + 1) as HWND,
                BM_CLICK,
                0,
                0,
            );
            let mut message: MSG = zeroed();
            while PeekMessageW(
                &mut message,
                hwnd,
                REMOVE_EXCLUSION,
                REMOVE_EXCLUSION,
                PM_REMOVE,
            ) != 0
            {
                DispatchMessageW(&message);
            }
            assert!(!settings::load(&ui.path).unwrap().excluded("Game.exe"));
            assert!(!ui.shared.settings.read().unwrap().excluded("Game.exe"));
            assert!(
                ui.shared
                    .settings
                    .read()
                    .unwrap()
                    .excluded("Other Game.exe"),
                "Repeated clicks queued for one chip must not delete its neighbor"
            );
            ui.remove_exclusion(initial_count);
            set(ui.exclusion_input, "");
            set(ui.inputs[0], "120");
            // The actual native popup stays hidden while we verify alpha and clipping.
            let mut combo_info: windows_sys::Win32::UI::Controls::COMBOBOXINFO = zeroed();
            combo_info.cbSize = size_of_val(&combo_info) as u32;
            assert_ne!(
                windows_sys::Win32::UI::Controls::GetComboBoxInfo(
                    ui.handle(PRESET) as HWND,
                    &mut combo_info
                ),
                0
            );
            let list = combo_info.hwndList;
            assert_eq!(
                GetWindowLongW(list, GWL_EXSTYLE) as u32 & WS_EX_LAYERED,
                0,
                "Only the background is translucent; menu labels must remain opaque"
            );
            let mut combo_bounds: RECT = zeroed();
            GetWindowRect(ui.handle(PRESET) as HWND, &mut combo_bounds);
            SetWindowPos(
                list,
                null_mut(),
                combo_bounds.left,
                combo_bounds.bottom + ui.px(5),
                ui.px(140),
                ui.px(104),
                SWP_NOACTIVATE | SWP_NOZORDER,
            );
            ui.prepare_profile_backdrop(list);
            let original = ui
                .profile_backdrop
                .borrow()
                .as_ref()
                .unwrap()
                .pixels
                .clone();
            assert!(original.iter().all(|p| p >> 24 == 255));
            let first = original[0];
            assert!(
                original.iter().any(|p| *p != first),
                "Frost must contain the app's rendered content"
            );
            let cached = ui
                .profile_backdrop
                .borrow()
                .as_ref()
                .unwrap()
                .pixels
                .as_ptr();
            ui.prepare_profile_backdrop(list);
            assert_eq!(
                cached,
                ui.profile_backdrop
                    .borrow()
                    .as_ref()
                    .unwrap()
                    .pixels
                    .as_ptr()
            );
            set(ui.inputs[0], "300");
            ui.profile_backdrop.borrow_mut().take();
            ui.prepare_profile_backdrop(list);
            assert_ne!(
                original,
                ui.profile_backdrop.borrow().as_ref().unwrap().pixels,
                "The blur must follow the owned controls behind the popup"
            );
            set(ui.inputs[0], "120");
            ui.profile_backdrop.borrow_mut().take();
            let region = CreateRectRgn(0, 0, 0, 0);
            assert_ne!(GetWindowRgn(list, region), 0);
            assert_eq!(PtInRegion(region, 0, 0), 0);
            assert_ne!(PtInRegion(region, ui.px(70), ui.px(20)), 0);
            DeleteObject(region);
            assert_eq!(IsWindowVisible(list), 0);
            let reverse = ui.handle(213) as HWND;
            assert!(!checked(reverse as usize));
            SendMessageW(reverse, BM_CLICK, 0, 0);
            assert!(checked(reverse as usize));
            assert_eq!(
                ui.toggle_motion[1].position(),
                1.,
                "Hidden controls snap rather than run background animation"
            );
            SendMessageW(reverse, BM_CLICK, 0, 0);
            assert!(!checked(reverse as usize));
            let title_logo = ui.handle(906) as HWND;
            assert!(
                !title_logo.is_null(),
                "The caption should contain the native logo control"
            );
            let mut title_rect = zeroed();
            GetWindowRect(title_logo, &mut title_rect);
            let mut title_origin = POINT {
                x: title_rect.left,
                y: title_rect.top,
            };
            ScreenToClient(hwnd, &mut title_origin);
            assert!(title_origin.x >= ui.px(16) && title_origin.y >= ui.px(4));
            assert!(title_origin.y + title_rect.bottom - title_rect.top <= ui.px(44));
            assert!(title_origin.x + title_rect.right - title_rect.left < ui.px(ui.width - 160));
            assert_eq!(
                SendMessageW(title_logo, WM_NCHITTEST, 0, 0),
                HTTRANSPARENT as isize,
                "The title logo must pass dragging and resize hits to the frame"
            );
            let center = POINT {
                x: title_rect.left + (title_rect.right - title_rect.left) / 2,
                y: title_rect.top + (title_rect.bottom - title_rect.top) / 2,
            };
            let screen_point = ((center.y as u32 as usize) << 16) | (center.x as u16 as usize);
            assert_eq!(
                SendMessageW(hwnd, WM_NCHITTEST, 0, screen_point as isize),
                HTCAPTION as isize
            );
            let title_surface = Surface::new(
                title_rect.right - title_rect.left,
                title_rect.bottom - title_rect.top,
            );
            SendMessageW(
                title_logo,
                WM_PRINTCLIENT,
                title_surface.dc as usize,
                PRF_CLIENT as isize,
            );
            GdiFlush();
            let title_pixels = std::slice::from_raw_parts(
                title_surface.pixels,
                (title_surface.w * title_surface.h) as usize,
            );
            assert!(
                title_pixels
                    .iter()
                    .filter(|pixel| **pixel & 0xffffff == 0x8818ff)
                    .count()
                    > 10,
                "The small caption logo must retain its purple MAGE lettering"
            );

            // Keyboard stepping must reach the same field as the paired buttons.
            for (i, default) in [120., 30., 70., 1., 4., 22.].into_iter().enumerate() {
                SendMessageW(hwnd, WM_COMMAND, 400 + i * 2, 0);
                assert_eq!(text(ui.inputs[i]).parse::<f64>().unwrap(), default + 1.);
                SendMessageW(ui.inputs[i] as HWND, WM_KEYDOWN, VK_DOWN as usize, 0);
                assert_eq!(text(ui.inputs[i]).parse::<f64>().unwrap(), default);
            }
            set(ui.inputs[3], "1");
            SendMessageW(hwnd, WM_COMMAND, 407, 0);
            assert_eq!(text(ui.inputs[3]), "1");
            let combo = ui.handle(PRESET) as HWND;
            if scale == 1.25 {
                let dc = CreateCompatibleDC(null_mut());
                SelectObject(dc, ui.fonts.body as HGDIOBJ);
                let mut face = [0u16; 128];
                let count = GetTextFaceW(dc, face.len() as i32, face.as_mut_ptr());
                eprintln!(
                    "Resolved body font: {}",
                    String::from_utf16_lossy(&face[..(count as usize).saturating_sub(1)])
                );
                DeleteDC(dc);
            }
            assert_eq!(SendMessageW(combo, CB_GETCURSEL, 0, 0), 0);
            SendMessageW(combo, CB_SETCURSEL, 2, 0);
            SendMessageW(
                hwnd,
                WM_COMMAND,
                PRESET as usize | ((CBN_SELENDOK as usize) << 16),
                combo as isize,
            );
            assert_eq!(text(ui.inputs[1]), SMOOTH_OMEGA.to_string());
            assert_eq!(ui.preset, Preset::SuperSmooth);
            assert_eq!(ui.shared.settings.read().unwrap().omega, SMOOTH_OMEGA); // Fixed presets auto-apply.
            ui.apply();
            let saved = settings::load(&ui.path).unwrap();
            assert_eq!(saved.preset, Preset::SuperSmooth);
            assert_eq!(saved.omega, SMOOTH_OMEGA);
            assert_eq!(saved.exclusions, Settings::default().exclusions);
            std::fs::remove_file(&ui.path).unwrap();
            SendMessageW(combo, CB_SETCURSEL, 0, 0);
            ui.select_preset();
            ui.note.clear();
            ui.update();
            let geometry = (ui.width, ui.height, ui.shell().cards()[3].h);
            SendMessageW(hwnd, WM_SIZE, SIZE_MINIMIZED as usize, 0);
            assert_eq!((ui.width, ui.height, ui.shell().cards()[3].h), geometry);
            let style = GetWindowLongPtrW(hwnd, GWL_STYLE);
            SetWindowLongPtrW(hwnd, GWL_STYLE, style | WS_MINIMIZE as isize);
            SetWindowPos(
                hwnd,
                null_mut(),
                0,
                0,
                160,
                28,
                SWP_NOMOVE | SWP_NOZORDER | SWP_NOACTIVATE,
            );
            assert_ne!(IsIconic(hwnd), 0);
            ui.layout();
            assert_eq!(
                (ui.width, ui.height, ui.shell().cards()[3].h),
                geometry,
                "Minimized client dimensions must not replace the saved layout"
            );
            SetWindowLongPtrW(hwnd, GWL_STYLE, style);
            SetWindowPos(
                hwnd,
                null_mut(),
                0,
                0,
                ui.px(DEFAULT_WIDTH),
                ui.px(DEFAULT_HEIGHT),
                SWP_NOMOVE | SWP_NOZORDER | SWP_NOACTIVATE | SWP_FRAMECHANGED,
            );
            ui.layout();
            assert_eq!(ui.handle(104), 0); // Quit remains in tray only.
            let mut button_rect = zeroed();
            GetWindowRect(ui.toggle as HWND, &mut button_rect);
            let mut origin = POINT {
                x: button_rect.left,
                y: button_rect.top,
            };
            ScreenToClient(hwnd, &mut origin);
            assert!(origin.x < ui.px(ui.shell().rail));
            // A fractional-sized card must paint the final physical pixel, not a black seam.
            let mut pad_rect = zeroed();
            GetClientRect(ui.pad as HWND, &mut pad_rect);
            let surface = Surface::new(pad_rect.right, pad_rect.bottom);
            SendMessageW(
                ui.pad as HWND,
                WM_PRINTCLIENT,
                surface.dc as usize,
                PRF_CLIENT as isize,
            );
            for y in [4, pad_rect.bottom / 2, pad_rect.bottom - 5] {
                let edge = surface.pixel(pad_rect.right - 1, y);
                for channel in [edge & 255, (edge >> 8) & 255, (edge >> 16) & 255] {
                    assert!(channel >= 180, "Dark edge {edge:06x} at scale {scale}");
                }
            }
            // 128 entries stay inside their own scroll region and the last entry is reachable.
            ui.excluded_apps = (0..128).map(|i| format!("application{i:03}.exe")).collect();
            ui.rebuild_exclusions();
            ui.layout();
            ui.reveal_exclusion(127);
            let panel = ui.handle(EXCLUSION_PANEL) as HWND;
            let mut row = zeroed();
            GetWindowRect(ui.handle(EXCLUSION_ROW + 255) as HWND, &mut row);
            let mut point = POINT {
                x: row.left,
                y: row.top,
            };
            ScreenToClient(panel, &mut point);
            assert!(point.y >= 0);
            assert!(point.y + row.bottom - row.top <= ui.px(ui.shell().exclusion_viewport().h));
            assert!(ui.shell().cards()[2].h <= 200);
            ui.excluded_apps = Settings::default().exclusions;
            ui.exclusion_scroll = 0;
            ui.rebuild_exclusions();
            ui.layout();
            let surface = Surface::new(ui.px(ui.width), ui.px(ui.height));
            ui.paint_main(surface.dc);
            let saved = SaveDC(surface.dc);
            SetViewportOrgEx(
                surface.dc,
                ui.px(ui.shell().body.x),
                ui.px(ui.shell().body.y),
                null_mut(),
            );
            ui.paint_body(surface.dc);
            RestoreDC(surface.dc, saved);
            // Explicitly compose hidden child surfaces: recursive WM_PRINT may skip
            // custom classes when their root is intentionally hidden.
            for &(id, handle, _) in &ui.controls {
                if GetWindowLongW(handle as HWND, GWL_STYLE) as u32 & WS_VISIBLE == 0 {
                    continue;
                }
                let mut rect = zeroed();
                GetWindowRect(handle as HWND, &mut rect);
                let mut origin = POINT {
                    x: rect.left,
                    y: rect.top,
                };
                ScreenToClient(hwnd, &mut origin);
                let saved = SaveDC(surface.dc);
                IntersectClipRect(
                    surface.dc,
                    origin.x,
                    origin.y,
                    origin.x + rect.right - rect.left,
                    origin.y + rect.bottom - rect.top,
                );
                if (EXCLUSION_ROW..EXCLUSION_ROW + 256).contains(&id) {
                    let mut panel_rect = zeroed();
                    GetWindowRect(panel, &mut panel_rect);
                    let mut panel_origin = POINT {
                        x: panel_rect.left,
                        y: panel_rect.top,
                    };
                    ScreenToClient(hwnd, &mut panel_origin);
                    IntersectClipRect(
                        surface.dc,
                        panel_origin.x,
                        panel_origin.y,
                        panel_origin.x + panel_rect.right - panel_rect.left,
                        panel_origin.y + panel_rect.bottom - panel_rect.top,
                    );
                }
                let child = Surface::new(
                    (rect.right - rect.left).max(1),
                    (rect.bottom - rect.top).max(1),
                );
                SendMessageW(
                    handle as HWND,
                    WM_PRINTCLIENT,
                    child.dc as usize,
                    PRF_CLIENT as isize,
                );
                if scale == 1.25 && matches!(id, RESET_PREVIEW | PRESET) {
                    eprintln!(
                        "Control {id}: origin {},{} size {},{}",
                        origin.x, origin.y, child.w, child.h
                    );
                    child.save(
                        &std::env::current_dir()
                            .unwrap()
                            .join(format!("target/headless-ui/control-{id}.bmp")),
                    );
                }
                BitBlt(
                    surface.dc, origin.x, origin.y, child.w, child.h, child.dc, 0, 0, SRCCOPY,
                );
                RestoreDC(surface.dc, saved);
            }
            let output = std::env::current_dir().unwrap().join("target/headless-ui");
            std::fs::create_dir_all(&output).unwrap();
            surface.save(&output.join(format!("settings-{}.bmp", (scale * 100.) as i32)));
            if scale == 1. {
                let stepper = ui.handle(400) as HWND;
                let normal = Surface::new(25, 17);
                ui.paint_button(stepper, normal.dc);
                normal.save(&output.join("stepper-raised.bmp"));
                SendMessageW(stepper, BM_SETSTATE, 1, 0);
                let pressed = Surface::new(25, 17);
                ui.paint_button(stepper, pressed.dc);
                pressed.save(&output.join("stepper-pressed.bmp"));
                GdiFlush();
                assert_ne!(
                    std::slice::from_raw_parts(normal.pixels, 25 * 17),
                    std::slice::from_raw_parts(pressed.pixels, 25 * 17)
                );
                SendMessageW(stepper, BM_SETSTATE, 0, 0);
                let menu = Surface::new(140, 96);
                for i in 0..3 {
                    ui.draw_preset_item(&DRAWITEMSTRUCT {
                        itemID: i,
                        itemState: if i == 0 { ODS_SELECTED } else { 0 },
                        hDC: menu.dc,
                        rcItem: RECT {
                            left: 0,
                            top: i as i32 * 32,
                            right: 140,
                            bottom: i as i32 * 32 + 32,
                        },
                        ..zeroed()
                    });
                }
                menu.save(&output.join("profile-menu-content.bmp"));
                ui.toggle_motion[1].target(false, false);
                ui.toggle_motion[1].target(true, true);
                ui.toggle_motion[1].step(0.06);
                let midway = Surface::new(370, 44);
                ui.paint_button(ui.handle(213) as HWND, midway.dc);
                midway.save(&output.join("toggle-midway.bmp"));
                ui.snap_toggles();
            }
            assert_eq!(IsWindowVisible(hwnd), 0);
            DestroyWindow(hwnd);
            DeleteObject(ui.brush as HGDIOBJ);
        }
        let mut foreground_pid = 0;
        GetWindowThreadProcessId(GetForegroundWindow(), &mut foreground_pid);
        assert_ne!(
            foreground_pid,
            windows_sys::Win32::System::Threading::GetCurrentProcessId(),
            "Hidden tests must never activate their own window"
        );
    }
}
