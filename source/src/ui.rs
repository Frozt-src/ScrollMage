mod actions;
mod app_icons;
mod backdrop;
mod control_motion;
mod exclusions;
mod frame;
mod presets;
mod preview;
mod profiles;
mod startup;
mod theme;
mod tuning;
use crate::native::{wide, Handle, Runtime, Shared, REFRESH, UPDATE};
use presets::preset_combo;
use scrollpilot::{
    layout::{Rect, Shell, DEFAULT_HEIGHT, DEFAULT_WIDTH, FORM_ROW_GAP, FORM_ROW_START},
    settings::{self, Preset, Settings},
};
use startup::{set_startup, startup_enabled};
use std::{
    mem::{size_of, zeroed},
    path::PathBuf,
    ptr::{null, null_mut},
    sync::{atomic::Ordering, Arc},
};
use theme::*;
use windows_sys::Win32::{
    Foundation::*,
    Graphics::Gdi::*,
    System::{LibraryLoader::*, Registry::*, Threading::CreateMutexW},
    UI::{
        Accessibility::*,
        Controls::{
            SetScrollInfo, DRAWITEMSTRUCT, EM_SETLIMITTEXT, MEASUREITEMSTRUCT, ODS_SELECTED,
            WM_MOUSELEAVE,
        },
        HiDpi::*,
        Input::KeyboardAndMouse::*,
        Shell::*,
        WindowsAndMessaging::*,
    },
};

const TRAY: u32 = WM_APP + 3;
const APPLY: u16 = 101;
const TOGGLE: u16 = 102;
const QUIT: u16 = 104;
const STATUS_SWITCH: u16 = 110;
const NAV_SETTINGS: u16 = 120;
const NAV_PROFILES: u16 = 121;
const NAV_ABOUT: u16 = 122;
const NAV_ACTIVITY: u16 = 123;
const EXCLUDE_ADD: u16 = 130;
const PRESET: u16 = 135;
const EXCLUSION_PANEL: u16 = 217;
const RESTORE: u32 = WM_APP + 4;
const CONTROL_MOTION: u32 = WM_APP + 5;
const REMOVE_EXCLUSION: u32 = WM_APP + 6;
const TITLE_LOGO: u16 = 906;
const WINDOW_MINIMIZE: u16 = 150;
const WINDOW_MAXIMIZE: u16 = 151;
const WINDOW_CLOSE: u16 = 152;
const EXCLUSION_ROW: u16 = 2000;
const RESET_PREVIEW: u16 = 134;

struct Ui {
    brand: scrollpilot::brand_lockup::Lockup,
    logo_animated: bool,
    toggle_motion: [scrollpilot::control_motion::Toggle; 4],
    toggle_initialized: bool,
    toggle_time: f64,
    shared: Arc<Shared>,
    path: PathBuf,
    inputs: [usize; 6],
    excluded_apps: Vec<String>,
    exclusion_scroll: i32,
    exclusion_generation: usize,
    preset: Preset,
    profile_backdrop: std::cell::RefCell<Option<backdrop::Frost>>,
    reverse: usize,
    horizontal: usize,
    exclusion_input: usize,
    startup: usize,
    status: usize,
    status_title: usize,
    status_switch: usize,
    toggle: usize,
    pad: usize,
    hwnd: usize,
    body: usize,
    controls: Vec<(u16, usize, bool)>,
    fonts: Fonts,
    brush: usize,
    icon: usize,
    app_icons: Option<app_icons::Icons>,
    scale: f64,
    width: i32,
    height: i32,
    view: u16,
    scroll: i32,
    ready: bool,
    loading_controls: bool,
    wheel_total: i64,
    horizontal_total: i64,
    events: u64,
    preview: [scrollpilot::motion::GestureAxis; 2],
    drag: Option<(i32, i32, f64)>,
    frame_time: f64,
    wheel_lines: [u32; 2],
    note: String,
    taskbar_msg: u32,
}
unsafe fn text(hwnd: usize) -> String {
    let len = GetWindowTextLengthW(hwnd as HWND).clamp(0, 16384);
    let mut b = vec![0u16; len as usize + 1];
    GetWindowTextW(hwnd as HWND, b.as_mut_ptr(), b.len() as i32);
    String::from_utf16_lossy(&b[..len as usize])
}
unsafe fn set(hwnd: usize, s: &str) {
    if hwnd != 0 {
        SetWindowTextW(hwnd as HWND, wide(s).as_ptr());
    }
}
unsafe fn checked(hwnd: usize) -> bool {
    SendMessageW(hwnd as HWND, BM_GETCHECK, 0, 0) == 1
}
unsafe fn check(hwnd: usize, b: bool) {
    SendMessageW(hwnd as HWND, BM_SETCHECK, usize::from(b), 0);
    InvalidateRect(hwnd as HWND, null(), 0);
}
unsafe fn state(hwnd: HWND) -> *mut Ui {
    GetWindowLongPtrW(GetAncestor(hwnd, GA_ROOT), GWLP_USERDATA) as *mut Ui
}
impl Ui {
    unsafe fn new(
        shared: Arc<Shared>,
        path: PathBuf,
        note: String,
        scale: f64,
        brush: usize,
        icon: usize,
    ) -> Result<Self, String> {
        Ok(Self {
            brand: scrollpilot::brand_lockup::Lockup::new()?,
            logo_animated: false,
            toggle_motion: Default::default(),
            toggle_initialized: false,
            toggle_time: 0.,
            shared,
            path,
            inputs: [0; 6],
            excluded_apps: Vec::new(),
            exclusion_scroll: 0,
            exclusion_generation: 0,
            preset: Preset::Default,
            profile_backdrop: Default::default(),
            reverse: 0,
            horizontal: 0,
            exclusion_input: 0,
            startup: 0,
            status: 0,
            status_title: 0,
            status_switch: 0,
            toggle: 0,
            pad: 0,
            hwnd: 0,
            body: 0,
            controls: Vec::new(),
            fonts: Fonts::new(scale),
            brush,
            icon,
            app_icons: None,
            scale,
            width: DEFAULT_WIDTH,
            height: DEFAULT_HEIGHT,
            view: NAV_SETTINGS,
            scroll: 0,
            ready: false,
            loading_controls: true,
            wheel_total: 0,
            horizontal_total: 0,
            events: 0,
            preview: [
                scrollpilot::motion::GestureAxis::new(0., 500., 462.),
                scrollpilot::motion::GestureAxis::new(600., 2400., 168.),
            ],
            drag: None,
            frame_time: 0.,
            wheel_lines: [3, 3],
            note,
            taskbar_msg: RegisterWindowMessageW(wide("TaskbarCreated").as_ptr()),
        })
    }
    fn px(&self, v: i32) -> i32 {
        theme::px(v, self.scale)
    }
    unsafe fn create(
        &mut self,
        class: &str,
        title: &str,
        style: u32,
        id: u16,
        body: bool,
        font: usize,
    ) -> usize {
        let parent = if body { self.body } else { self.hwnd };
        let h = CreateWindowExW(
            0,
            wide(class).as_ptr(),
            wide(title).as_ptr(),
            WS_CHILD | WS_VISIBLE | style,
            0,
            0,
            1,
            1,
            parent as HWND,
            id as usize as HMENU,
            GetModuleHandleW(null()),
            null(),
        );
        SendMessageW(h, WM_SETFONT, font, 0);
        if class == "BUTTON" {
            SetWindowSubclass(h, Some(button), 1, 0);
        }
        if class == "COMBOBOX" {
            SetWindowSubclass(h, Some(preset_combo), 4, 0);
        }
        if class == "STATIC" {
            SetWindowSubclass(h, Some(static_label), 2, 0);
        }
        if class == "EDIT" {
            SetWindowSubclass(h, Some(edit), 3, 0);
            windows_sys::Win32::UI::Controls::SetWindowTheme(
                h,
                wide("").as_ptr(),
                wide("").as_ptr(),
            );
            SendMessageW(h, EM_SETLIMITTEXT, 16384, 0);
        }
        self.controls.push((id, h as usize, body));
        h as usize
    }
    unsafe fn static_text(&mut self, title: &str, id: u16, body: bool, font: usize) -> usize {
        self.create("STATIC", title, 0, id, body, font)
    }
    unsafe fn button(&mut self, title: &str, id: u16, body: bool, checkable: bool) -> usize {
        self.create(
            "BUTTON",
            title,
            WS_TABSTOP
                | if matches!(id, NAV_SETTINGS | NAV_PROFILES | NAV_ACTIVITY | NAV_ABOUT) {
                    BS_AUTORADIOBUTTON as u32 | if id == NAV_SETTINGS { WS_GROUP } else { 0 }
                } else if checkable {
                    BS_AUTOCHECKBOX as u32
                } else {
                    BS_PUSHBUTTON as u32
                },
            id,
            body,
            self.fonts.body,
        )
    }
    fn handle(&self, id: u16) -> usize {
        self.controls.iter().find(|c| c.0 == id).map_or(0, |c| c.1)
    }
    unsafe fn at(&self, id: u16, r: Rect) {
        let h = self.handle(id) as HWND;
        MoveWindow(h, self.px(r.x), self.px(r.y), self.px(r.w), self.px(r.h), 1);
        ShowWindow(h, SW_SHOW);
    }
    unsafe fn build(&mut self) {
        SystemParametersInfoW(
            SPI_GETWHEELSCROLLCHARS,
            0,
            (&mut self.wheel_lines[0] as *mut u32).cast(),
            0,
        );
        SystemParametersInfoW(
            SPI_GETWHEELSCROLLLINES,
            0,
            (&mut self.wheel_lines[1] as *mut u32).cast(),
            0,
        );
        self.static_text("Scroll Mage logo", TITLE_LOGO, false, self.fonts.brand);
        self.button("Minimize", WINDOW_MINIMIZE, false, false);
        self.button("Maximize", WINDOW_MAXIMIZE, false, false);
        self.button("Close to tray", WINDOW_CLOSE, false, false);
        self.button("Settings", NAV_SETTINGS, false, false);
        self.button("App profiles", NAV_PROFILES, false, false);
        self.button("Activity", NAV_ACTIVITY, false, false);
        self.button("About", NAV_ABOUT, false, false);
        self.button("Apply settings", APPLY, false, false);
        self.toggle = self.button("Enable smoothing", TOGGLE, false, false);
        self.static_text("Alt + P pauses or resumes.", 904, false, self.fonts.small);
        let opts = self.shared.settings.read().unwrap().clone();
        self.preset = opts.preset;
        let combo = self.create(
            "COMBOBOX",
            "Scrolling preset",
            WS_TABSTOP
                | WS_VSCROLL
                | CBS_DROPDOWNLIST as u32
                | CBS_OWNERDRAWFIXED as u32
                | CBS_HASSTRINGS as u32,
            PRESET,
            true,
            self.fonts.small,
        );
        for preset in Preset::ALL {
            SendMessageW(
                combo as HWND,
                CB_ADDSTRING,
                0,
                wide(preset.label()).as_ptr() as isize,
            );
        }
        SendMessageW(combo as HWND, CB_SETCURSEL, self.preset as usize, 0);
        SendMessageW(combo as HWND, CB_SETITEMHEIGHT, 0, self.px(32) as isize);
        SendMessageW(
            combo as HWND,
            CB_SETITEMHEIGHT,
            usize::MAX,
            self.px(28) as isize,
        );
        let labels = [
            "Distance",
            "Spring response",
            "Acceleration window",
            "Maximum acceleration",
            "Test momentum friction",
            "Test elastic response",
        ];
        for (i, value) in [
            opts.distance,
            opts.omega,
            opts.acceleration_ms,
            opts.acceleration_max,
            opts.friction,
            opts.elastic_omega,
        ]
        .into_iter()
        .enumerate()
        {
            self.static_text(labels[i], 1000 + i as u16, true, self.fonts.body);
            self.inputs[i] = self.create(
                "EDIT",
                &format!("{value}"),
                WS_TABSTOP | ES_AUTOHSCROLL as u32,
                201 + i as u16,
                true,
                self.fonts.number,
            );
            self.button(
                &format!("Increase {}", labels[i].to_lowercase()),
                400 + i as u16 * 2,
                true,
                false,
            );
            self.button(
                &format!("Decrease {}", labels[i].to_lowercase()),
                401 + i as u16 * 2,
                true,
                false,
            );
        }
        self.horizontal = self.button(
            "Horizontal scrolling\nUse Shift + wheel to move sideways.",
            212,
            true,
            true,
        );
        check(self.horizontal, opts.horizontal);
        self.reverse = self.button(
            "Reverse direction\nInvert the scrolling direction.",
            213,
            true,
            true,
        );
        check(self.reverse, opts.reverse);
        self.startup = self.button(
            "Start at sign in\nOpen Scroll Mage when you sign in.",
            214,
            true,
            true,
        );
        check(self.startup, startup_enabled());
        self.status_title = self.static_text("", 902, true, self.fonts.section);
        self.status = self.static_text("", 903, true, self.fonts.small);
        self.status_switch = self.button("Smoothing", STATUS_SWITCH, true, true);
        self.excluded_apps = opts.exclusions.clone();
        self.static_text("Excluded apps", 1102, true, self.fonts.section);
        self.exclusion_input = self.create(
            "EDIT",
            "",
            WS_TABSTOP | ES_AUTOHSCROLL as u32,
            215,
            true,
            self.fonts.body,
        );
        self.button("Add", EXCLUDE_ADD, true, false);
        for (id, title) in [(1100, "Scroll settings"), (1101, "Options")] {
            self.static_text(title, id, true, self.fonts.section);
        }
        self.static_text("Tune your wheel, then apply.", 1110, true, self.fonts.small);
        self.static_text("EXE or path", 1112, true, self.fonts.small);
        self.static_text("1 = off", 1013, true, self.fonts.small);
        let panel = self.create(
            "ScrollMage.Exclusions",
            "Excluded apps list",
            WS_CLIPCHILDREN | WS_VSCROLL,
            EXCLUSION_PANEL,
            true,
            self.fonts.small,
        );
        SetWindowLongPtrW(panel as HWND, GWL_EXSTYLE, WS_EX_CONTROLPARENT as isize);
        let empty = self.static_text("No excluded apps yet.", 1116, true, self.fonts.small);
        SetParent(empty as HWND, panel as HWND);
        self.rebuild_exclusions();
        self.pad = self.create(
            "ScrollPilot.TestPad",
            "Scroll Test. Use the wheel, drag, or press arrow keys.",
            WS_TABSTOP,
            216,
            true,
            self.fonts.body,
        );
        self.shared.pad.store(self.pad, Ordering::Release);
        self.button("Reset", RESET_PREVIEW, true, false);
        self.static_text("Scroll Mage", 1120, true, self.fonts.brand);
        self.static_text(concat!("Version ", env!("CARGO_PKG_VERSION"), "\r\n\r\nScroll Mage is a scrolling extension for Windows that makes mouse wheel scrolling feel more natural, consistent, and customizable. It is built for desktop mice; touchpad scrolling is left to Windows. Your settings stay on this computer.\r\n\r\nAdd an app to Excluded apps to keep its own scrolling. Scroll settings apply to all other apps.\r\n\r\nScroll Test lets you try momentum and elastic edges. Other apps control how they move their content.\r\n\r\nClose the window to keep Scroll Mage in the tray. Use the tray menu to quit."),1121,true,self.fonts.body);
        self.build_profiles();
        self.static_text("Scrolling activity", 3110, true, self.fonts.section);
        self.static_text("", 3111, true, self.fonts.body);
        self.ready = true;
        self.loading_controls = false;
        self.sync_preset_controls();
        check(self.handle(NAV_SETTINGS), true);
        self.layout();
        self.update();
        self.sync_toggles();
        presets::style_popup(self.handle(PRESET) as HWND, self.scale);
    }
    fn shell(&self) -> Shell {
        Shell::with_exclusions(self.width, self.height, self.excluded_apps.len())
    }
    unsafe fn layout(&mut self) {
        if !self.ready || IsIconic(self.hwnd as HWND) != 0 {
            return;
        }
        let mut r = zeroed();
        GetClientRect(self.hwnd as HWND, &mut r);
        self.width = (f64::from(r.right) / self.scale).round() as i32;
        self.height = (f64::from(r.bottom) / self.scale).round() as i32;
        let shell = self.shell();
        let x = shell.body.x;
        let w = shell.body.w;
        for (i, id) in [WINDOW_MINIMIZE, WINDOW_MAXIMIZE, WINDOW_CLOSE]
            .into_iter()
            .enumerate()
        {
            self.at(id, Rect::new(self.width - 152 + i as i32 * 48, 8, 44, 32));
        }
        set(
            self.handle(WINDOW_MAXIMIZE),
            if IsZoomed(self.hwnd as HWND) != 0 {
                "Restore"
            } else {
                "Maximize"
            },
        );
        self.at(TITLE_LOGO, Rect::new(20, 4, 120, 40));
        for (i, id) in [NAV_SETTINGS, NAV_PROFILES, NAV_ACTIVITY, NAV_ABOUT]
            .into_iter()
            .enumerate()
        {
            self.at(id, Rect::new(16, 72 + i as i32 * 52, shell.rail - 32, 44));
        }
        self.at(TOGGLE, Rect::new(16, self.height - 66, shell.rail - 32, 40));
        let action_y = shell.footer.y;
        let action_right = x + w;
        if self.view == NAV_SETTINGS {
            self.at(APPLY, Rect::new(action_right - 150, action_y, 150, 42));
        } else {
            ShowWindow(self.handle(APPLY) as HWND, SW_HIDE);
        }
        self.at(
            904,
            Rect::new(
                x,
                action_y + 3,
                if self.view == NAV_SETTINGS {
                    (w - 170).max(120)
                } else {
                    w
                },
                38,
            ),
        );
        MoveWindow(
            self.body as HWND,
            self.px(x),
            self.px(shell.body.y),
            self.px(w),
            self.px(shell.body.h.max(50)),
            1,
        );
        for &(_, h, in_body) in &self.controls {
            if in_body {
                ShowWindow(h as HWND, SW_HIDE);
            }
        }
        let content_h = if self.view == NAV_SETTINGS {
            shell.content_height()
        } else if self.view == NAV_PROFILES {
            720
        } else if self.view == NAV_ACTIVITY {
            520
        } else {
            shell.body.h
        };
        self.scroll = self.scroll.clamp(0, (content_h - shell.body.h).max(0));
        let info = SCROLLINFO {
            cbSize: size_of::<SCROLLINFO>() as u32,
            fMask: SIF_RANGE | SIF_PAGE | SIF_POS,
            nMin: 0,
            nMax: content_h - 1,
            nPage: shell.body.h.max(1) as u32,
            nPos: self.scroll,
            ..zeroed()
        };
        SetScrollInfo(self.body as HWND, SB_VERT, &info, 1);
        if self.view == NAV_SETTINGS {
            let cards = shell.cards().map(|mut c| {
                c.y -= self.scroll;
                c
            });
            for (id, index) in [(1100, 0), (1101, 1), (1102, 2)] {
                let c = cards[index];
                self.at(
                    id,
                    Rect::new(
                        c.x + 20,
                        c.y + 16,
                        if index == 0 { c.w - 190 } else { c.w - 40 },
                        26,
                    ),
                );
            }
            let c = cards[0];
            // Keep this opaque label clear of the native combo's closed surface.
            self.at(1110, Rect::new(c.x + 20, c.y + 45, c.w - 190, 22));
            self.at(PRESET, Rect::new(c.x + c.w - 160, c.y + 14, 140, 160));
            for i in 0..6 {
                let y = c.y + FORM_ROW_START + i * FORM_ROW_GAP + if i >= 4 { 8 } else { 0 };
                let input_x = c.x + c.w - 124;
                self.at(
                    1000 + i as u16,
                    Rect::new(c.x + 20, y + if i == 3 { 0 } else { 6 }, c.w - 154, 24),
                );
                self.at(201 + i as u16, Rect::new(input_x + 10, y + 8, 60, 22));
                self.at(400 + i as u16 * 2, Rect::new(input_x + 77, y + 1, 25, 17));
                self.at(401 + i as u16 * 2, Rect::new(input_x + 77, y + 18, 25, 17));
                if i == 3 {
                    self.at(1013, Rect::new(c.x + 20, y + 22, c.w - 154, 16));
                }
            }
            let c = cards[1];
            for (i, id) in [212, 213, 214].into_iter().enumerate() {
                self.at(
                    id,
                    Rect::new(c.x + 20, c.y + 48 + i as i32 * 44, c.w - 40, 44),
                );
            }
            self.at(902, Rect::new(c.x + 54, c.y + 204, c.w - 160, 24));
            self.at(903, Rect::new(c.x + 54, c.y + 232, c.w - 74, 32));
            self.at(STATUS_SWITCH, Rect::new(c.x + c.w - 99, c.y + 196, 80, 42));
            let (entry, add) = shell.exclusion_entry();
            self.at(
                215,
                Rect::new(entry.x + 8, entry.y + 5 - self.scroll, entry.w - 16, 22),
            );
            self.at(
                EXCLUDE_ADD,
                Rect::new(add.x, add.y - self.scroll, add.w, add.h),
            );
            self.at(
                1112,
                Rect::new(
                    cards[2].x + 20,
                    entry.y + 5 - self.scroll,
                    entry.x - cards[2].x - 28,
                    22,
                ),
            );
            let mut viewport = shell.exclusion_viewport();
            viewport.y -= self.scroll;
            self.at(EXCLUSION_PANEL, viewport);
            self.layout_exclusions();
            let c = cards[3];
            self.at(216, Rect::new(c.x, c.y, c.w, c.h));
            self.at(RESET_PREVIEW, Rect::new(c.x + c.w - 99, c.y + 14, 79, 30));
        } else if self.view == NAV_PROFILES {
            self.layout_profiles();
        } else if self.view == NAV_ACTIVITY {
            self.at(3110, Rect::new(20, 20 - self.scroll, w - 40, 32));
            self.at(3111, Rect::new(20, 70 - self.scroll, w - 40, 430));
        } else {
            self.at(1120, Rect::new(32, 30, w - 80, 46));
            self.at(
                1121,
                Rect::new(32, 100, w - 90, (shell.body.h - 124).max(100)),
            );
        }
        InvalidateRect(self.hwnd as HWND, null(), 0);
        InvalidateRect(self.body as HWND, null(), 0);
        for id in [NAV_SETTINGS, NAV_PROFILES, NAV_ACTIVITY, NAV_ABOUT] {
            InvalidateRect(self.handle(id) as HWND, null(), 0);
        }
    }
    unsafe fn update(&self) {
        let enabled = self.shared.enabled.load(Ordering::Acquire);
        let (title, copy) = self.shared.status_text();
        set(self.status_title, &title);
        set(self.status, copy.lines().next().unwrap_or(""));
        set(self.handle(3111), &format!("{title}\r\n\r\n{copy}\r\n\r\nPoint at an app and scroll to see whether its input is accepted or passed through. Counters show this session's activity, not visible frame rate.\r\n\r\nFine-resolution and external injected input pass through. Other apps decide whether small wheel events produce pixel movement or line steps.\r\n\r\nSuper smooth adds a fast coast after a quick wheel flick. Reverse, click or pause to stop it. Momentum friction and elastic response belong to dragging in Scroll Test. Ordinary wheel output cannot read another app's content bounds or add elastic edges."));
        set(
            self.toggle,
            if enabled {
                "Pause smoothing"
            } else {
                "Enable smoothing"
            },
        );
        check(self.status_switch, enabled);
        self.update_pending();
        InvalidateRect(self.hwnd as HWND, null(), 0);
        InvalidateRect(self.toggle as HWND, null(), 0);
    }
    unsafe fn navigate(&mut self, view: u16) {
        self.view = view;
        for id in [NAV_SETTINGS, NAV_PROFILES, NAV_ACTIVITY, NAV_ABOUT] {
            check(self.handle(id), id == view);
        }
        self.scroll = 0;
        self.drag = None;
        self.stop_preview();
        KillTimer(self.pad as HWND, 1);
        self.layout();
        SetFocus(self.handle(view) as HWND);
    }
    unsafe fn step(&mut self, id: u16) {
        if self.preset.fixed() {
            return;
        }
        let index = ((id - 400) / 2) as usize;
        let (min, max) = [
            (1., 960.),
            (10., 80.),
            (10., 500.),
            (1., 10.),
            (1., 12.),
            (10., 50.),
        ][index];
        if let Ok(value) = text(self.inputs[index]).parse::<f64>() {
            let increment = if (id - 400).is_multiple_of(2) {
                1.
            } else {
                -1.
            };
            set(
                self.inputs[index],
                &format!("{}", (value + increment).clamp(min, max)),
            );
        }
    }
    unsafe fn stop_preview(&mut self) {
        for a in &mut self.preview {
            *a = scrollpilot::motion::GestureAxis::new(a.position, a.maximum, a.viewport);
        }
        KillTimer(self.pad as HWND, 1);
    }
    unsafe fn logo_timer(&mut self, visible: bool) {
        if !visible || IsIconic(self.hwnd as HWND) != 0 {
            self.snap_toggles();
        }
        let mut enabled: i32 = 0;
        let read = SystemParametersInfoW(
            SPI_GETCLIENTAREAANIMATION,
            0,
            (&mut enabled as *mut i32).cast(),
            0,
        ) != 0;
        self.logo_animated = scrollpilot::branding::should_animate(
            visible,
            IsIconic(self.hwnd as HWND) != 0,
            !read || enabled == 0,
        );
        if self.logo_animated {
            SetTimer(self.hwnd as HWND, 5, 25, None);
        } else {
            KillTimer(self.hwnd as HWND, 5);
        }
        InvalidateRect(self.handle(TITLE_LOGO) as HWND, null(), 0);
    }
    unsafe fn paint_main(&self, dc: HDC) {
        let shell = self.shell();
        fill(
            dc,
            Rect::new(0, 0, self.width, self.height),
            self.scale,
            CANVAS,
        );
        fill(
            dc,
            Rect::new(0, 48, shell.rail, self.height - 48),
            self.scale,
            RAIL,
        );
        line(
            dc,
            shell.rail,
            48,
            shell.rail,
            self.height,
            self.scale,
            BORDER,
            1,
        );
        line(dc, 0, 47, self.width, 47, self.scale, BORDER, 1);
        label(
            dc,
            Rect::new(28, self.height - 118, shell.rail - 48, 36),
            self.scale,
            self.fonts.small,
            MUTED,
            "Scrolling controls\nfor your desktop.",
            DT_WORDBREAK,
        );
        label(
            dc,
            Rect::new(28, self.height - 146, shell.rail - 48, 24),
            self.scale,
            self.fonts.small,
            MUTED,
            concat!("Version ", env!("CARGO_PKG_VERSION")),
            DT_SINGLELINE,
        );
    }
    unsafe fn paint_body(&self, dc: HDC) {
        let shell = self.shell();
        fill(
            dc,
            Rect::new(0, 0, shell.body.w, shell.body.h),
            self.scale,
            CANVAS,
        );
        if self.view == NAV_SETTINGS {
            let cards = shell.cards();
            for mut c in cards.into_iter().take(3) {
                c.y -= self.scroll;
                round(dc, c, 4, self.scale, WHITE, Some(BORDER));
            }
            let c = cards[0];
            for i in 0..6 {
                let y = c.y + FORM_ROW_START + i * FORM_ROW_GAP + if i >= 4 { 8 } else { 0 }
                    - self.scroll;
                round(
                    dc,
                    Rect::new(c.x + c.w - 124, y, 104, 36),
                    4,
                    self.scale,
                    WHITE,
                    Some(if GetFocus() == self.inputs[i as usize] as HWND {
                        ACCENT
                    } else {
                        BORDER
                    }),
                );
            }
            let c = cards[1];
            let y = c.y - self.scroll;
            line(
                dc,
                c.x + 20,
                y + 192,
                c.x + c.w - 20,
                y + 192,
                self.scale,
                BORDER,
                1,
            );
            round(
                dc,
                Rect::new(c.x + 20, y + 207, 23, 23),
                12,
                self.scale,
                if self.shared.enabled.load(Ordering::Acquire) {
                    ACCENT
                } else {
                    AMBER
                },
                None,
            );
            let (mut entry, _) = shell.exclusion_entry();
            entry.y -= self.scroll;
            round(
                dc,
                entry,
                4,
                self.scale,
                WHITE,
                Some(if GetFocus() == self.exclusion_input as HWND {
                    ACCENT
                } else {
                    BORDER
                }),
            );
        } else {
            round(
                dc,
                Rect::new(0, 0, shell.body.w, shell.body.h - 2),
                4,
                self.scale,
                WHITE,
                Some(BORDER),
            );
        }
    }
    unsafe fn paint_preview(&self, hwnd: HWND) {
        buffered(hwnd, |dc, width, height| {
            self.paint_preview_surface(hwnd, dc, width, height)
        });
    }
    unsafe fn paint_preview_surface(&self, hwnd: HWND, dc: HDC, width: i32, height: i32) {
        let w = (f64::from(width) / self.scale).ceil() as i32;
        let h = (f64::from(height) / self.scale).ceil() as i32;
        fill(dc, Rect::new(0, 0, w, h), self.scale, CANVAS);
        round(
            dc,
            Rect::new(0, 0, w, h),
            4,
            self.scale,
            WHITE,
            Some(BORDER),
        );
        let card_clip = SaveDC(dc);
        let region = CreateRoundRectRgn(
            self.px(1),
            self.px(1),
            width - self.px(1),
            height - self.px(1),
            self.px(6),
            self.px(6),
        );
        let mut origin = zeroed();
        GetViewportOrgEx(dc, &mut origin);
        OffsetRgn(region, origin.x, origin.y);
        ExtSelectClipRgn(dc, region, RGN_AND);
        DeleteObject(region);
        let saved = SaveDC(dc);
        IntersectClipRect(
            dc,
            self.px(1),
            self.px(scrollpilot::layout::TEST_HEADER),
            width - self.px(1),
            height - self.px(1),
        );
        let offset = self.preview[1].rendered();
        let first = (offset / 34.).floor() as i32;
        for row in (first - 1).max(0)..=(first + h / 34 + 3).min(99) {
            let y =
                (f64::from(row * 34 + scrollpilot::layout::TEST_HEADER) - offset).round() as i32;
            if row % 2 == 0 {
                fill(dc, Rect::new(1, y, w - 2, 34), self.scale, TEST_OFF_WHITE);
            }
            label(
                dc,
                Rect::new(14, y, w - 28, 34),
                self.scale,
                self.fonts.number,
                MUTED,
                &format!("{:03}", row + 1),
                DT_SINGLELINE | DT_VCENTER,
            );
            label(
                dc,
                Rect::new(
                    65 - self.preview[0].rendered().round() as i32,
                    y,
                    w - 158,
                    34,
                ),
                self.scale,
                self.fonts.body,
                INK,
                "Try a longer drag, then reverse direction.",
                DT_SINGLELINE | DT_VCENTER | DT_END_ELLIPSIS,
            );
            label(
                dc,
                Rect::new(w - 94, y, 80, 34),
                self.scale,
                self.fonts.small,
                MUTED,
                "Scroll Mage",
                DT_SINGLELINE | DT_VCENTER | DT_RIGHT,
            );
        }
        RestoreDC(dc, saved);
        fill(dc, Rect::new(1, 1, w - 2, 72), self.scale, WHITE);
        label(
            dc,
            Rect::new(20, 14, w - 130, 28),
            self.scale,
            self.fonts.section,
            INK,
            "Scroll Test",
            DT_SINGLELINE | DT_VCENTER,
        );
        label(
            dc,
            Rect::new(20, 46, w - 40, 22),
            self.scale,
            self.fonts.small,
            MUTED,
            "Use your wheel, drag, or arrow keys.",
            DT_SINGLELINE | DT_END_ELLIPSIS,
        );
        fill(dc, Rect::new(1, 73, w - 2, 30), self.scale, TEST_OFF_WHITE);
        label(
            dc,
            Rect::new(20, 73, w - 40, 30),
            self.scale,
            self.fonts.small,
            INK,
            &format!(
                "Wheel: {}     Horizontal: {}     Events: {}",
                self.wheel_total, self.horizontal_total, self.events
            ),
            DT_SINGLELINE | DT_VCENTER | DT_END_ELLIPSIS,
        );
        if GetFocus() == hwnd {
            let pen = CreatePen(PS_DOT, 1, color(ACCENT));
            let old = SelectObject(dc, pen);
            let brush = SelectObject(dc, GetStockObject(HOLLOW_BRUSH));
            Rectangle(
                dc,
                self.px(2),
                self.px(2),
                width - self.px(2),
                height - self.px(2),
            );
            SelectObject(dc, brush);
            SelectObject(dc, old);
            DeleteObject(pen);
        }
        RestoreDC(dc, card_clip);
    }
    unsafe fn paint_button(&self, h: HWND, dc: HDC) {
        let id = GetDlgCtrlID(h) as u16;
        let mut client = zeroed();
        GetClientRect(h, &mut client);
        let w = (f64::from(client.right) / self.scale).ceil() as i32;
        let height = (f64::from(client.bottom) / self.scale).ceil() as i32;
        let is_nav = matches!(id, NAV_SETTINGS | NAV_PROFILES | NAV_ACTIVITY | NAV_ABOUT);
        let in_body = self.controls.iter().any(|c| c.1 == h as usize && c.2);
        let bg = if is_nav || id == TOGGLE {
            RAIL
        } else if in_body {
            WHITE
        } else {
            CANVAS
        };
        fill(dc, Rect::new(0, 0, w, height), self.scale, bg);
        let focused = GetFocus() == h;
        let pressed = SendMessageW(h, BM_GETSTATE, 0, 0) & BST_PUSHED as isize != 0;
        let caption = if id == TOGGLE && w < 140 {
            if self.shared.enabled.load(Ordering::Acquire) {
                "Pause".into()
            } else {
                "Enable".into()
            }
        } else {
            text(h as usize)
        };
        let is_toggle = [212, 213, 214, STATUS_SWITCH].contains(&id);
        if [WINDOW_MINIMIZE, WINDOW_MAXIMIZE, WINDOW_CLOSE].contains(&id) {
            self.paint_caption_button(h, dc, id, w, height, pressed);
        } else if (EXCLUSION_ROW..EXCLUSION_ROW + 256).contains(&id) && id % 2 == 1 {
            round(
                dc,
                Rect::new(0, 0, w, height),
                4,
                self.scale,
                if pressed { SELECTED } else { 0xf5edf1 },
                Some(if focused { ACCENT } else { 0xe5d6df }),
            );
            let x = w / 2;
            let y = height / 2;
            line(dc, x - 4, y - 4, x + 4, y + 4, self.scale, MUTED, 1);
            line(dc, x + 4, y - 4, x - 4, y + 4, self.scale, MUTED, 1);
        } else if is_toggle {
            let on = checked(h as usize);
            let index = [212, 213, 214, STATUS_SWITCH]
                .iter()
                .position(|&v| v == id)
                .unwrap();
            let motion = self.toggle_motion[index];
            let progress = if self.toggle_initialized {
                motion.position()
            } else {
                f64::from(on)
            };
            let stretch = if self.toggle_initialized {
                motion.stretch()
            } else {
                0.
            };
            let top = if id == STATUS_SWITCH { 9 } else { 8 };
            round(
                dc,
                Rect::new(1, top, 44, 25),
                13,
                self.scale,
                blend(0x858f9f, ACCENT, progress),
                None,
            );
            round(
                dc,
                Rect::new(
                    (4. + 19. * progress - stretch / 2.).round().clamp(3., 24.) as i32,
                    top + 3 + (stretch / 4.).round() as i32,
                    19 + stretch.round() as i32,
                    19 - (stretch / 2.).round() as i32,
                ),
                10,
                self.scale,
                WHITE,
                None,
            );
            if id == STATUS_SWITCH {
                label(
                    dc,
                    Rect::new(54, 0, w - 54, height),
                    self.scale,
                    self.fonts.body,
                    MUTED,
                    if on { "On" } else { "Off" },
                    DT_SINGLELINE | DT_VCENTER,
                );
            } else {
                let mut lines = caption.lines();
                label(
                    dc,
                    Rect::new(60, 0, w - 60, 24),
                    self.scale,
                    self.fonts.body,
                    INK,
                    lines.next().unwrap_or(""),
                    DT_SINGLELINE | DT_VCENTER | DT_END_ELLIPSIS,
                );
                label(
                    dc,
                    Rect::new(60, 25, w - 60, 21),
                    self.scale,
                    self.fonts.small,
                    MUTED,
                    lines.next().unwrap_or(""),
                    DT_SINGLELINE | DT_END_ELLIPSIS,
                );
            }
        } else if is_nav {
            let selected = id == self.view;
            if selected || pressed {
                round(
                    dc,
                    Rect::new(0, 0, w, height),
                    9,
                    self.scale,
                    SELECTED,
                    None,
                );
            }
            let tint = if selected { ACCENT_TEXT } else { MUTED };
            let glyph = if id == NAV_SETTINGS { "⚙" } else { "☷" };
            if w >= 140 {
                label(
                    dc,
                    Rect::new(16, 0, 28, height),
                    self.scale,
                    self.fonts.section,
                    tint,
                    glyph,
                    DT_SINGLELINE | DT_VCENTER | DT_CENTER,
                );
            }
            label(
                dc,
                Rect::new(
                    if w >= 140 { 54 } else { 8 },
                    0,
                    w - if w >= 140 { 64 } else { 16 },
                    height,
                ),
                self.scale,
                self.fonts.body,
                tint,
                &caption,
                DT_SINGLELINE | DT_VCENTER,
            );
        } else if (400..412).contains(&id) {
            round(
                dc,
                Rect::new(0, 2, w, height - 2),
                4,
                self.scale,
                0xd8dbe5,
                None,
            );
            round(
                dc,
                Rect::new(0, if pressed { 2 } else { 0 }, w, height - 2),
                4,
                self.scale,
                if pressed { SELECTED } else { 0xf3f4f8 },
                if focused { Some(ACCENT) } else { None },
            );
            let up = (id - 400).is_multiple_of(2);
            let center = w / 2;
            let top = height / 2 + if pressed { 2 } else { 0 };
            line(
                dc,
                center - 3,
                if up { top + 2 } else { top - 2 },
                center,
                if up { top - 1 } else { top + 1 },
                self.scale,
                MUTED,
                1,
            );
            line(
                dc,
                center,
                if up { top - 1 } else { top + 1 },
                center + 3,
                if up { top + 2 } else { top - 2 },
                self.scale,
                MUTED,
                1,
            );
        } else {
            let primary = id == TOGGLE;
            round(
                dc,
                Rect::new(0, 0, w, height),
                9,
                self.scale,
                if primary {
                    if pressed {
                        ACCENT_PRESSED
                    } else {
                        ACCENT
                    }
                } else if pressed {
                    SELECTED
                } else {
                    0xefebee
                },
                if primary { None } else { Some(BORDER) },
            );
            label(
                dc,
                Rect::new(8, 0, w - 16, height),
                self.scale,
                self.fonts.body,
                if primary { WHITE } else { INK },
                &caption,
                DT_SINGLELINE | DT_VCENTER | DT_CENTER,
            );
        }
        if focused
            && !(400..412).contains(&id)
            && !(EXCLUSION_ROW..EXCLUSION_ROW + 256).contains(&id)
        {
            outline(
                dc,
                Rect::new(2, 2, w - 4, height - 4),
                if is_toggle { 10 } else { 7 },
                self.scale,
                ACCENT,
            );
        }
    }
}

unsafe extern "system" fn static_label(
    h: HWND,
    msg: u32,
    w: usize,
    l: isize,
    _: usize,
    _: usize,
) -> isize {
    let ptr = state(h);
    if ptr.is_null() {
        return DefSubclassProc(h, msg, w, l);
    }
    let ui = &*ptr;
    if msg == WM_NCHITTEST && GetDlgCtrlID(h) as u16 == TITLE_LOGO {
        return HTTRANSPARENT as isize;
    }
    let paint = |dc: HDC, width: i32, height: i32| {
        let id = GetDlgCtrlID(h) as u16;
        if id == TITLE_LOGO {
            let surface = CANVAS;
            let scale = (width as f32 / scrollpilot::brand_lockup::WIDTH)
                .min(height as f32 / scrollpilot::brand_lockup::HEIGHT);
            fill(
                dc,
                Rect::new(
                    0,
                    0,
                    (f64::from(width) / ui.scale).ceil() as i32,
                    (f64::from(height) / ui.scale).ceil() as i32,
                ),
                ui.scale,
                surface,
            );
            let mut graphics = null_mut();
            if windows_sys::Win32::Graphics::GdiPlus::GdipCreateFromHDC(dc, &mut graphics) == 0 {
                ui.brand.draw(
                    graphics,
                    0.,
                    (height as f32 - scrollpilot::brand_lockup::HEIGHT * scale) / 2.,
                    scale,
                    ui.shared.now() / 1000.,
                    ui.logo_animated,
                    0xff000000,
                    0xff000000 | surface,
                );
                windows_sys::Win32::Graphics::GdiPlus::GdipDeleteGraphics(graphics);
            }
            return;
        }

        let muted = id == 901
            || id == 903
            || id == 904
            || (1010..1016).contains(&id)
            || (1110..1116).contains(&id);
        let rect = Rect::new(
            0,
            0,
            (f64::from(width) / ui.scale).round() as i32,
            (f64::from(height) / ui.scale).round() as i32,
        );
        fill(
            dc,
            rect,
            ui.scale,
            if id == 904 || (EXCLUSION_ROW..EXCLUSION_ROW + 256).contains(&id) {
                CANVAS
            } else {
                WHITE
            },
        );
        label(
            dc,
            rect,
            ui.scale,
            SendMessageW(h, WM_GETFONT, 0, 0) as usize,
            if muted { MUTED } else { INK },
            &text(h as usize),
            if (EXCLUSION_ROW..EXCLUSION_ROW + 256).contains(&id) {
                DT_SINGLELINE | DT_VCENTER | DT_END_ELLIPSIS
            } else {
                DT_WORDBREAK
            },
        );
    };
    if msg == WM_PAINT {
        buffered(h, paint);
        return 0;
    }
    if msg == WM_ERASEBKGND {
        return 1;
    }
    if msg == WM_PRINTCLIENT {
        let mut r = zeroed();
        GetClientRect(h, &mut r);
        paint(w as HDC, r.right, r.bottom);
        return 0;
    }
    let result = DefSubclassProc(h, msg, w, l);
    if matches!(msg, WM_SETTEXT | WM_SETFONT) {
        InvalidateRect(h, null(), 1);
        UpdateWindow(h);
    }
    result
}
unsafe extern "system" fn edit(h: HWND, msg: u32, w: usize, l: isize, _: usize, _: usize) -> isize {
    let id = GetDlgCtrlID(h) as u16;
    if (201..207).contains(&id) && msg == WM_GETDLGCODE {
        return DefSubclassProc(h, msg, w, l) | DLGC_WANTARROWS as isize;
    }
    if id == 215 && msg == WM_GETDLGCODE && w as u16 == VK_RETURN {
        return (DLGC_WANTCHARS | DLGC_WANTALLKEYS) as isize;
    }
    if id == 215 && msg == WM_KEYDOWN && w as u16 == VK_RETURN {
        SendMessageW(GetAncestor(h, GA_ROOT), WM_COMMAND, EXCLUDE_ADD as usize, 0);
        return 0;
    }
    if msg == WM_KEYDOWN && (201..207).contains(&id) && (w as u16 == VK_UP || w as u16 == VK_DOWN) {
        let command = 400 + (id - 201) * 2 + u16::from(w as u16 == VK_DOWN);
        SendMessageW(GetAncestor(h, GA_ROOT), WM_COMMAND, command as usize, 0);
        return 0;
    }
    let result = DefSubclassProc(h, msg, w, l);
    if msg == WM_SETFOCUS || msg == WM_KILLFOCUS {
        InvalidateRect(GetParent(h), null(), 0);
    }
    result
}
unsafe extern "system" fn button(
    h: HWND,
    msg: u32,
    w: usize,
    l: isize,
    _: usize,
    _: usize,
) -> isize {
    if GetDlgCtrlID(h) as u16 == WINDOW_MAXIMIZE {
        if msg == WM_NCHITTEST {
            return HTMAXBUTTON as isize;
        }
        if matches!(
            msg,
            WM_NCMOUSEMOVE
                | WM_NCMOUSELEAVE
                | WM_NCLBUTTONDOWN
                | WM_NCLBUTTONUP
                | WM_NCLBUTTONDBLCLK
        ) {
            InvalidateRect(h, null(), 0);
            return SendMessageW(GetAncestor(h, GA_ROOT), msg, w, l);
        }
    }
    let ptr = state(h);
    if ptr.is_null() {
        return DefSubclassProc(h, msg, w, l);
    }
    if msg == WM_PAINT {
        let ui = &*ptr;
        buffered(h, |dc, _, _| ui.paint_button(h, dc));
        return 0;
    }
    if msg == WM_PRINTCLIENT {
        (&*ptr).paint_button(h, w as HDC);
        return 0;
    }
    if msg == WM_ERASEBKGND {
        return 1;
    }
    if msg == WM_MOUSEMOVE {
        let mut tracking = TRACKMOUSEEVENT {
            cbSize: size_of::<TRACKMOUSEEVENT>() as u32,
            dwFlags: TME_LEAVE,
            hwndTrack: h,
            dwHoverTime: 0,
        };
        TrackMouseEvent(&mut tracking);
    }
    if msg == WM_SETFOCUS {
        let id = GetDlgCtrlID(h) as u16;
        if (EXCLUSION_ROW..EXCLUSION_ROW + 256).contains(&id) {
            (&mut *ptr).reveal_exclusion(((id - EXCLUSION_ROW) / 2) as usize);
        }
    }
    let result = DefSubclassProc(h, msg, w, l);
    if IsWindow(h) == 0 {
        return result;
    }
    if msg == BM_SETCHECK && [212, 213, 214, STATUS_SWITCH].contains(&(GetDlgCtrlID(h) as u16)) {
        PostMessageW(GetAncestor(h, GA_ROOT), CONTROL_MOTION, 0, 0);
    }
    if matches!(
        msg,
        WM_SETFOCUS
            | WM_KILLFOCUS
            | WM_LBUTTONDOWN
            | WM_LBUTTONUP
            | WM_KEYDOWN
            | WM_KEYUP
            | BM_SETCHECK
            | BM_SETSTATE
            | WM_ENABLE
            | WM_SETTEXT
            | WM_SETFONT
            | WM_MOUSEMOVE
            | WM_MOUSELEAVE
    ) {
        InvalidateRect(h, null(), 0);
        UpdateWindow(h);
    }
    result
}
unsafe fn colors(ui: &Ui, msg: u32, w: usize, l: isize) -> Option<isize> {
    if msg == WM_CTLCOLORLISTBOX {
        SetTextColor(w as HDC, color(INK));
        SetBkColor(w as HDC, color(WHITE));
        return Some(ui.brush as isize);
    }
    if msg == WM_CTLCOLOREDIT {
        SetTextColor(w as HDC, color(INK));
        SetBkColor(w as HDC, color(WHITE));
        return Some(ui.brush as isize);
    }
    if msg == WM_CTLCOLORSTATIC {
        let id = GetDlgCtrlID(l as HWND) as u16;
        SetTextColor(
            w as HDC,
            color(
                if id == 901
                    || id == 903
                    || id == 904
                    || (1010..1016).contains(&id)
                    || (1110..1116).contains(&id)
                {
                    MUTED
                } else {
                    INK
                },
            ),
        );
        SetBkMode(w as HDC, TRANSPARENT as i32);
        return Some(GetStockObject(HOLLOW_BRUSH) as isize);
    }
    None
}
unsafe extern "system" fn body(h: HWND, msg: u32, w: usize, l: isize) -> isize {
    let ptr = state(h);
    if ptr.is_null() {
        return DefWindowProcW(h, msg, w, l);
    }
    let ui = &mut *ptr;
    if let Some(result) = colors(ui, msg, w, l) {
        return result;
    }
    match msg {
        WM_PRINTCLIENT => {
            ui.paint_body(w as HDC);
            0
        }
        WM_PAINT => {
            buffered(h, |dc, _, _| ui.paint_body(dc));
            0
        }
        WM_ERASEBKGND => 1,
        WM_COMMAND | WM_DRAWITEM | WM_MEASUREITEM => SendMessageW(ui.hwnd as HWND, msg, w, l),
        WM_VSCROLL => {
            let shell = ui.shell();
            let mut info = SCROLLINFO {
                cbSize: size_of::<SCROLLINFO>() as u32,
                fMask: SIF_TRACKPOS,
                ..zeroed()
            };
            GetScrollInfo(h, SB_VERT, &mut info);
            let old = ui.scroll;
            ui.scroll = match (w & 0xffff) as i32 {
                SB_LINEUP => old - 36,
                SB_LINEDOWN => old + 36,
                SB_PAGEUP => old - shell.body.h,
                SB_PAGEDOWN => old + shell.body.h,
                SB_THUMBTRACK => info.nTrackPos,
                _ => old,
            };
            ui.layout();
            0
        }
        WM_MOUSEWHEEL => {
            let delta = ((w >> 16) as u16 as i16) as i32;
            // Multiply first: high-resolution wheels send deltas below one notch.
            ui.scroll -= delta * 72 / 120;
            ui.layout();
            0
        }
        _ => DefWindowProcW(h, msg, w, l),
    }
}
unsafe extern "system" fn foreground(
    _: HWINEVENTHOOK,
    _: u32,
    _: HWND,
    _: i32,
    _: i32,
    _: u32,
    _: u32,
) {
    if let Some(s) = APP.get() {
        PostMessageW(s.load(Ordering::Acquire) as HWND, REFRESH, 0, 0);
    }
}
static APP: std::sync::OnceLock<std::sync::atomic::AtomicUsize> = std::sync::OnceLock::new();
unsafe extern "system" fn window(h: HWND, msg: u32, w: usize, l: isize) -> isize {
    if msg == WM_NCCREATE {
        let c = &*(l as *const CREATESTRUCTW);
        SetWindowLongPtrW(h, GWLP_USERDATA, c.lpCreateParams as isize);
        (*(c.lpCreateParams as *mut Ui)).hwnd = h as usize;
    }
    let ptr = GetWindowLongPtrW(h, GWLP_USERDATA) as *mut Ui;
    if ptr.is_null() {
        return DefWindowProcW(h, msg, w, l);
    }
    let ui = &mut *ptr;
    if ui.taskbar_msg != 0 && msg == ui.taskbar_msg {
        ui.refresh_icons();
        ui.tray(NIM_ADD);
        return 0;
    }
    if let Some(result) = colors(ui, msg, w, l) {
        return result;
    }
    if let Some(result) = ui.frame_message(h, msg, w, l) {
        return result;
    }
    match msg {
        WM_PRINTCLIENT => {
            ui.paint_main(w as HDC);
            0
        }
        WM_PAINT => {
            buffered(h, |dc, _, _| ui.paint_main(dc));
            0
        }
        WM_ERASEBKGND => 1,
        WM_SIZE => {
            if w != SIZE_MINIMIZED as usize {
                ui.layout();
            }
            if ui.ready {
                if w == SIZE_MINIMIZED as usize {
                    ui.stop_preview();
                }
                ui.logo_timer(w != SIZE_MINIMIZED as usize && IsWindowVisible(h) != 0);
            }
            0
        }
        WM_SHOWWINDOW if ui.ready => {
            if w != 0 {
                ui.layout();
            }
            ui.logo_timer(w != 0);
            0
        }
        WM_SETTINGCHANGE if ui.ready => {
            let _ = scrollpilot::frame_style::apply_small_corners(h);
            ui.logo_timer(IsWindowVisible(h) != 0);
            ui.sync_toggles();
            0
        }
        CONTROL_MOTION => {
            ui.sync_toggles();
            0
        }
        REMOVE_EXCLUSION => {
            let id = w as u16;
            if (EXCLUSION_ROW..EXCLUSION_ROW + 256).contains(&id)
                && id % 2 == 1
                && l as usize == ui.exclusion_generation
            {
                ui.remove_exclusion(((id - EXCLUSION_ROW) / 2) as usize);
            }
            0
        }
        WM_TIMER if w == 6 => {
            let now = ui.shared.now();
            ui.step_toggles((now - ui.toggle_time).max(0.) / 1000.);
            ui.toggle_time = now;
            0
        }
        WM_TIMER if w == 7 => {
            ui.shared.refresh_target();
            if IsWindowVisible(h) != 0 && IsIconic(h) == 0 {
                ui.update();
            }
            0
        }
        WM_TIMER if w == 5 => {
            if IsWindowVisible(h) == 0 || IsIconic(h) != 0 {
                ui.logo_timer(false);
            } else {
                let logo = ui.handle(TITLE_LOGO) as HWND;
                if IsWindowVisible(logo) != 0 {
                    InvalidateRect(logo, null(), 0);
                }
            }
            0
        }
        WM_GETMINMAXINFO => {
            let m = &mut *(l as *mut MINMAXINFO);
            m.ptMinTrackSize = POINT {
                x: ui.px(500),
                y: ui.px(420),
            };
            0
        }
        WM_DPICHANGED => {
            ui.scale = f64::from((w & 0xffff) as u16) / 96.;
            ui.refresh_icons();
            presets::style_popup(ui.handle(PRESET) as HWND, ui.scale);
            let old_fonts = std::mem::replace(&mut ui.fonts, Fonts::new(ui.scale));
            // Map each control's current font to its rescaled equivalent, so the
            // role chosen at creation survives every DPI change.
            let roles = [
                (old_fonts.body, ui.fonts.body),
                (old_fonts.small, ui.fonts.small),
                (old_fonts.section, ui.fonts.section),
                (old_fonts.display, ui.fonts.display),
                (old_fonts.brand, ui.fonts.brand),
                (old_fonts.number, ui.fonts.number),
            ];
            for &(_, control, _) in &ui.controls {
                let current = SendMessageW(control as HWND, WM_GETFONT, 0, 0) as usize;
                let font = roles
                    .iter()
                    .find(|(old, _)| *old == current)
                    .map_or(ui.fonts.body, |(_, new)| *new);
                SendMessageW(control as HWND, WM_SETFONT, font, 1);
            }
            for (id, row_height) in [(PRESET, 32), (3003, 28)] {
                SendMessageW(
                    ui.handle(id) as HWND,
                    CB_SETITEMHEIGHT,
                    0,
                    ui.px(row_height) as isize,
                );
                SendMessageW(
                    ui.handle(id) as HWND,
                    CB_SETITEMHEIGHT,
                    usize::MAX,
                    ui.px(28) as isize,
                );
            }
            drop(old_fonts);
            let r = &*(l as *const RECT);
            SetWindowPos(
                h,
                null_mut(),
                r.left,
                r.top,
                r.right - r.left,
                r.bottom - r.top,
                SWP_NOZORDER | SWP_NOACTIVATE,
            );
            ui.layout();
            0
        }
        UPDATE => {
            ui.update();
            ui.tray(NIM_MODIFY);
            0
        }
        REFRESH => {
            ui.shared.refresh_target();
            0
        }
        RESTORE => {
            ui.restore();
            0
        }
        WM_DRAWITEM if w == PRESET as usize => {
            ui.draw_preset_item(&*(l as *const DRAWITEMSTRUCT));
            1
        }
        WM_DRAWITEM if w == 3003 => {
            ui.draw_app_profile_item(&*(l as *const DRAWITEMSTRUCT));
            1
        }
        WM_MEASUREITEM if w == 3003 => {
            (*(l as *mut MEASUREITEMSTRUCT)).itemHeight = ui.px(28) as u32;
            1
        }
        WM_MEASUREITEM if w == PRESET as usize => {
            (*(l as *mut MEASUREITEMSTRUCT)).itemHeight = ui.px(32) as u32;
            1
        }
        WM_COMMAND => {
            let id = (w & 0xffff) as u16;
            if ui.profile_command(id, (w >> 16) as u32) {
                return 0;
            }
            if (201..207).contains(&id) && (w >> 16) as u32 == EN_CHANGE {
                if ui.ready && !ui.loading_controls {
                    ui.note.clear();
                    ui.update_pending();
                }
                return 0;
            }
            if id != PRESET && (w >> 16) as u32 != BN_CLICKED {
                return 0;
            }
            match id {
                212..=214 => {
                    ui.note.clear();
                    ui.sync_toggles();
                    ui.update_pending();
                }
                APPLY => ui.apply(),
                PRESET if (w >> 16) as u32 == CBN_DROPDOWN => {
                    presets::style_popup(ui.handle(PRESET) as HWND, ui.scale);
                }
                PRESET
                    if (w >> 16) as u32 == CBN_SELENDOK
                        || ((w >> 16) as u32 == CBN_SELCHANGE
                            && SendMessageW(
                                ui.handle(PRESET) as HWND,
                                CB_GETDROPPEDSTATE,
                                0,
                                0,
                            ) == 0) =>
                {
                    ui.select_preset()
                }
                TOGGLE | STATUS_SWITCH => ui.toggle(),
                NAV_SETTINGS | NAV_PROFILES | NAV_ACTIVITY | NAV_ABOUT => {
                    ui.navigate((w & 0xffff) as u16)
                }
                EXCLUDE_ADD => ui.add_exclusion(),
                id if (EXCLUSION_ROW..EXCLUSION_ROW + 256).contains(&id) && id % 2 == 1 => {
                    PostMessageW(
                        h,
                        REMOVE_EXCLUSION,
                        id as usize,
                        ui.exclusion_generation as isize,
                    );
                }
                WINDOW_MINIMIZE => {
                    ui.stop_preview();
                    ShowWindow(h, SW_MINIMIZE);
                }
                WINDOW_MAXIMIZE => {
                    ShowWindow(
                        h,
                        if IsZoomed(h) != 0 {
                            SW_RESTORE
                        } else {
                            SW_MAXIMIZE
                        },
                    );
                }
                WINDOW_CLOSE => {
                    PostMessageW(h, WM_CLOSE, 0, 0);
                }
                400..=411 => ui.step((w & 0xffff) as u16),
                RESET_PREVIEW => {
                    ui.preview = [
                        scrollpilot::motion::GestureAxis::new(0., 500., ui.preview[0].viewport),
                        scrollpilot::motion::GestureAxis::new(
                            600f64.min(ui.preview[1].maximum),
                            ui.preview[1].maximum,
                            ui.preview[1].viewport,
                        ),
                    ];
                    ui.wheel_total = 0;
                    ui.horizontal_total = 0;
                    ui.events = 0;
                    KillTimer(ui.pad as HWND, 1);
                    InvalidateRect(ui.pad as HWND, null(), 0);
                }
                QUIT => {
                    DestroyWindow(h);
                }
                _ => {}
            }
            0
        }
        WM_HOTKEY => {
            ui.toggle();
            0
        }
        WM_CLOSE => {
            ui.stop_preview();
            if ui.tray(NIM_MODIFY) {
                ShowWindow(h, SW_HIDE);
            } else {
                DestroyWindow(h);
            }
            0
        }
        WM_DESTROY => {
            KillTimer(h, 5);
            KillTimer(h, 6);
            KillTimer(ui.pad as HWND, 1);
            ui.tray(NIM_DELETE);
            UnregisterHotKey(h, 1);
            ui.shared.pause();
            ui.shared.ui.store(0, Ordering::Release);
            PostQuitMessage(0);
            0
        }
        TRAY => {
            if l as u32 == WM_LBUTTONUP || l as u32 == WM_LBUTTONDBLCLK {
                ui.restore();
            } else if l as u32 == WM_RBUTTONUP {
                let menu = CreatePopupMenu();
                AppendMenuW(menu, MF_STRING, 105, wide("Open Scroll Mage").as_ptr());
                AppendMenuW(
                    menu,
                    MF_STRING,
                    TOGGLE as usize,
                    wide(if ui.shared.enabled.load(Ordering::Acquire) {
                        "Pause smoothing"
                    } else {
                        "Enable smoothing"
                    })
                    .as_ptr(),
                );
                AppendMenuW(menu, MF_SEPARATOR, 0, null());
                AppendMenuW(
                    menu,
                    MF_STRING,
                    QUIT as usize,
                    wide("Quit Scroll Mage").as_ptr(),
                );
                let mut p = zeroed();
                GetCursorPos(&mut p);
                SetForegroundWindow(h);
                let cmd = TrackPopupMenu(
                    menu,
                    TPM_RETURNCMD | TPM_RIGHTBUTTON,
                    p.x,
                    p.y,
                    0,
                    h,
                    null(),
                );
                DestroyMenu(menu);
                if cmd == 105 {
                    ui.restore();
                } else if cmd > 0 {
                    PostMessageW(h, WM_COMMAND, cmd as usize, 0);
                }
                PostMessageW(h, WM_NULL, 0, 0);
            }
            0
        }
        _ => DefWindowProcW(h, msg, w, l),
    }
}
pub fn run(runtime: &Runtime, path: PathBuf, note: String, tray: bool) -> Result<(), String> {
    unsafe {
        let _graphics = GraphicsToken::start()?;
        let hinst = GetModuleHandleW(null());
        let scale = f64::from(GetDpiForSystem()) / 96.;
        let brush = CreateSolidBrush(color(WHITE));
        let icons = app_icons::Icons::load((scale * 96.).round() as u32)?;
        let classes: [(&str, WNDPROC); 4] = [
            ("ScrollPilot.Main", Some(window)),
            ("ScrollPilot.Body", Some(body)),
            ("ScrollPilot.TestPad", Some(preview::pad)),
            ("ScrollMage.Exclusions", Some(exclusions::panel)),
        ];
        for (name, proc) in classes {
            let class = wide(name);
            let wc = WNDCLASSW {
                lpfnWndProc: proc,
                hInstance: hinst,
                hIcon: if name == "ScrollPilot.Main" {
                    icons.large
                } else {
                    null_mut()
                },
                hCursor: LoadCursorW(null_mut(), IDC_ARROW),
                lpszClassName: class.as_ptr(),
                ..zeroed()
            };
            if RegisterClassW(&wc) == 0 {
                return Err("Windows could not register the interface.".into());
            }
        }
        let mut ui = Box::new(Ui::new(
            runtime.shared.clone(),
            path,
            note,
            scale,
            brush as usize,
            icons.tray as usize,
        )?);
        ui.app_icons = Some(icons);
        let mut work: RECT = zeroed();
        SystemParametersInfoW(SPI_GETWORKAREA, 0, (&mut work as *mut RECT).cast(), 0);
        let rect = RECT {
            left: 0,
            top: 0,
            right: ui.px(DEFAULT_WIDTH).min(work.right - work.left - 40),
            bottom: ui.px(DEFAULT_HEIGHT).min(work.bottom - work.top - 40),
        };
        let style = WS_OVERLAPPEDWINDOW | WS_CLIPCHILDREN;
        let h = CreateWindowExW(
            WS_EX_CONTROLPARENT,
            wide("ScrollPilot.Main").as_ptr(),
            wide("Scroll Mage").as_ptr(),
            style,
            CW_USEDEFAULT,
            CW_USEDEFAULT,
            rect.right - rect.left,
            rect.bottom - rect.top,
            null_mut(),
            null_mut(),
            hinst,
            (&mut *ui as *mut Ui).cast(),
        );
        if h.is_null() {
            return Err("Windows could not open Scroll Mage.".into());
        }
        ui.refresh_icons();
        ui.body = CreateWindowExW(
            WS_EX_CONTROLPARENT,
            wide("ScrollPilot.Body").as_ptr(),
            wide("Settings content").as_ptr(),
            WS_CHILD | WS_VISIBLE | WS_VSCROLL | WS_CLIPCHILDREN,
            0,
            0,
            1,
            1,
            h,
            null_mut(),
            hinst,
            null(),
        ) as usize;
        runtime.shared.ui.store(h as usize, Ordering::Release);
        APP.set(std::sync::atomic::AtomicUsize::new(h as usize))
            .ok();
        SetWindowPos(
            h,
            null_mut(),
            0,
            0,
            0,
            0,
            SWP_NOMOVE | SWP_NOSIZE | SWP_NOZORDER | SWP_NOACTIVATE | SWP_FRAMECHANGED,
        );
        let _ = scrollpilot::frame_style::apply_small_corners(h);
        ui.build();
        let tray_ok = ui.tray(NIM_ADD);
        if !tray || !tray_ok {
            ShowWindow(h, SW_SHOW);
            UpdateWindow(h);
        }
        if RegisterHotKey(h, 1, MOD_ALT | MOD_NOREPEAT, u32::from(b'P')) == 0 {
            ui.note = "The pause shortcut is in use. Use the tray menu to pause.".into();
            ui.update();
        }
        let event = SetWinEventHook(
            EVENT_SYSTEM_FOREGROUND,
            EVENT_SYSTEM_FOREGROUND,
            null_mut(),
            Some(foreground),
            0,
            0,
            WINEVENT_OUTOFCONTEXT,
        );
        if event.is_null() {
            runtime
                .shared
                .fail("App monitoring failed. Smoothing is paused. Restart to retry.");
        }
        runtime.shared.refresh_target();
        if SetTimer(h, 7, 250, None) == 0 {
            runtime
                .shared
                .fail("App monitoring timer failed. Restart to retry.");
        }
        let mut msg = zeroed();
        let message_result = loop {
            let result = GetMessageW(&mut msg, null_mut(), 0, 0);
            if result <= 0 {
                break result;
            }
            if IsDialogMessageW(h, &msg) == 0 {
                TranslateMessage(&msg);
                DispatchMessageW(&msg);
            }
        };
        let message_error = (message_result < 0).then(std::io::Error::last_os_error);
        // A message-queue error is not a successful shutdown. Destroy owned UI
        // before its boxed state drops, even when GetMessage did not return WM_QUIT.
        if IsWindow(h) != 0 {
            DestroyWindow(h);
        }
        if !event.is_null() {
            UnhookWinEvent(event);
        }
        runtime.shared.pause();
        if let Some(app) = APP.get() {
            app.store(0, Ordering::Release);
        }
        DeleteObject(brush);
        match message_error {
            Some(error) => Err(format!("Windows interface message loop failed: {error}")),
            None => Ok(()),
        }
    }
}
pub fn single_instance() -> Result<Handle, String> {
    unsafe {
        let h = CreateMutexW(null(), 0, wide("Local\\ScrollPilot.Instance.v1").as_ptr());
        if h.is_null() {
            return Err("Cannot create the single-instance guard.".into());
        }
        let guard = Handle(h as usize);
        if GetLastError() == ERROR_ALREADY_EXISTS {
            let window = FindWindowW(wide("ScrollPilot.Main").as_ptr(), null());
            if !window.is_null() {
                // Let the running instance take the foreground when it restores.
                let mut pid = 0;
                GetWindowThreadProcessId(window, &mut pid);
                AllowSetForegroundWindow(pid);
                PostMessageW(window, RESTORE, 0, 0);
            }
            return Err("Scroll Mage is already running.".into());
        }
        Ok(guard)
    }
}

#[cfg(test)]
mod headless_tests;
