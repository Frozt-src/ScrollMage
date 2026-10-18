use scrollpilot::{engine::Engine, policy::intercept, settings::Settings};
use std::{
    mem::{size_of, zeroed},
    ptr::null,
    sync::{
        atomic::{AtomicBool, AtomicU32, AtomicU64, AtomicUsize, Ordering},
        Arc, Mutex, OnceLock, RwLock,
    },
    thread::{self, JoinHandle},
    time::Instant,
};
use windows_sys::Win32::{
    Foundation::*,
    Security::*,
    System::{LibraryLoader::*, Threading::*},
    UI::{Input::KeyboardAndMouse::*, WindowsAndMessaging::*},
};

pub const UPDATE: u32 = WM_APP + 1;
pub const REFRESH: u32 = WM_APP + 2;
pub const TAG: usize = 0x5343504c;
pub fn wide(s: &str) -> Vec<u16> {
    s.encode_utf16().chain(Some(0)).collect()
}
pub fn conflicting_engine_running(settings: &Settings) -> bool {
    if settings.incompatible_processes.is_empty() {
        return false;
    }
    unsafe {
        use windows_sys::Win32::System::Diagnostics::ToolHelp::*;
        let snapshot = CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0);
        if snapshot == INVALID_HANDLE_VALUE {
            return true;
        }
        let snapshot = Handle(snapshot as usize);
        let mut p: PROCESSENTRY32W = zeroed();
        p.dwSize = size_of::<PROCESSENTRY32W>() as u32;
        let mut found = Process32FirstW(snapshot.raw(), &mut p);
        if found == 0 {
            return true;
        }
        while found != 0 {
            let n = p
                .szExeFile
                .iter()
                .position(|&x| x == 0)
                .unwrap_or(p.szExeFile.len());
            if settings
                .incompatible_processes
                .iter()
                .any(|name| name.eq_ignore_ascii_case(&String::from_utf16_lossy(&p.szExeFile[..n])))
            {
                return true;
            }
            found = Process32NextW(snapshot.raw(), &mut p);
        }
        GetLastError() != ERROR_NO_MORE_FILES
    }
}
pub struct Handle(pub usize);
impl Handle {
    pub fn raw(&self) -> HANDLE {
        self.0 as HANDLE
    }
}
impl Drop for Handle {
    fn drop(&mut self) {
        unsafe {
            if self.0 != 0 {
                CloseHandle(self.raw());
            }
        }
    }
}
#[derive(Default, Clone, Copy)]
pub struct Target {
    pub hwnd: usize,
    pub eligible: bool,
    pub own: bool,
    pub tuning: scrollpilot::settings::WheelTuning,
}
pub struct Shared {
    pub settings: RwLock<Settings>,
    pub engine: Mutex<Engine>,
    pub enabled: AtomicBool,
    pub quitting: AtomicBool,
    pub interrupted: AtomicBool,
    pub target: RwLock<Target>,
    pub ui: AtomicUsize,
    pub pad: AtomicUsize,
    pub latch: AtomicUsize,
    pub wake: Handle,
    pub epoch: Instant,
    pub hook_thread: AtomicU32,
    pub hook_live: AtomicBool,
    pub sent: AtomicU64,
    pub ticks: AtomicU64,
    pub hook_max_us: AtomicU64,
    pub fault: Mutex<String>,
}
static SHARED: OnceLock<Arc<Shared>> = OnceLock::new();
impl Shared {
    pub fn now(&self) -> f64 {
        self.epoch.elapsed().as_secs_f64() * 1000.
    }
    pub fn notify(&self) {
        unsafe {
            SetEvent(self.wake.raw());
        }
    }
    pub fn pause(&self) {
        self.enabled.store(false, Ordering::Release);
        if let Ok(mut e) = self.engine.lock() {
            e.clear();
        }
        self.notify();
        self.update_ui();
    }
    pub fn update_ui(&self) {
        unsafe {
            let w = self.ui.load(Ordering::Acquire);
            if w != 0 {
                PostMessageW(w as HWND, UPDATE, 0, 0);
            }
        }
    }
    pub fn fail(&self, reason: &str) {
        if let Ok(mut f) = self.fault.lock() {
            *f = reason.into();
        }
        self.pause();
    }
    pub fn refresh_target(&self) {
        unsafe {
            if self.enabled.load(Ordering::Acquire)
                && self
                    .settings
                    .read()
                    .map(|s| conflicting_engine_running(&s))
                    .unwrap_or(true)
            {
                self.fail("A configured competing scroll app is running (or cannot be checked). Pause it before enabling Scroll Mage.");
            }
            let w = GetForegroundWindow();
            let mut pid = 0;
            GetWindowThreadProcessId(w, &mut pid);
            let own = pid == GetCurrentProcessId();
            let mut eligible = !w.is_null();
            let mut tuning = self.settings.read().map(|s| s.wheel()).unwrap_or_default();
            if !own {
                let p = Handle(OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, 0, pid) as usize);
                if p.0 == 0 {
                    eligible = false;
                } else {
                    let mut name = [0u16; 32768];
                    let mut len = name.len() as u32;
                    if QueryFullProcessImageNameW(p.raw(), 0, name.as_mut_ptr(), &mut len) == 0 {
                        eligible = false;
                    } else {
                        let path = String::from_utf16_lossy(&name[..len as usize]);
                        let base = path.rsplit('\\').next().unwrap_or("");
                        eligible = self
                            .settings
                            .read()
                            .map(|s| {
                                tuning = s.for_app(base);
                                !s.excluded(base)
                            })
                            .unwrap_or(false);
                    }
                    eligible &= match (integrity(p.raw()), integrity(GetCurrentProcess())) {
                        (Some(t), Some(me)) => t <= me,
                        _ => false,
                    };
                }
            }
            if let Ok(mut target) = self.target.write() {
                *target = Target {
                    hwnd: w as usize,
                    eligible,
                    own,
                    tuning,
                };
            }
            if let Ok(mut e) = self.engine.lock() {
                e.clear();
            }
            self.notify();
        }
    }
}
unsafe fn integrity(process: HANDLE) -> Option<u32> {
    let mut t = std::ptr::null_mut();
    if OpenProcessToken(process, TOKEN_QUERY, &mut t) == 0 {
        return None;
    }
    let token = Handle(t as usize);
    let mut len = 0;
    GetTokenInformation(
        token.raw(),
        TokenIntegrityLevel,
        std::ptr::null_mut(),
        0,
        &mut len,
    );
    if len == 0 || len > 65536 {
        return None;
    }
    // usize storage keeps TOKEN_MANDATORY_LABEL aligned on both supported ABIs.
    let mut buf = vec![0usize; (len as usize).div_ceil(size_of::<usize>())];
    if GetTokenInformation(
        token.raw(),
        TokenIntegrityLevel,
        buf.as_mut_ptr().cast(),
        len,
        &mut len,
    ) == 0
    {
        return None;
    }
    let sid = (*(buf.as_ptr() as *const TOKEN_MANDATORY_LABEL)).Label.Sid;
    if sid.is_null() || IsValidSid(sid) == 0 {
        return None;
    }
    let n = *GetSidSubAuthorityCount(sid);
    if n == 0 {
        return None;
    }
    Some(*GetSidSubAuthority(sid, (n - 1) as u32))
}
unsafe fn under_cursor(target: Target, pad: usize, point: POINT) -> bool {
    let child = WindowFromPoint(point);
    if child.is_null() {
        return false;
    }
    GetAncestor(child, GA_ROOT) as usize == target.hwnd && (!target.own || child as usize == pad)
}
unsafe extern "system" fn mouse_hook(code: i32, msg: usize, data: isize) -> isize {
    if code >= 0 && matches!(msg as u32, WM_LBUTTONDOWN | WM_RBUTTONDOWN | WM_MBUTTONDOWN) {
        if let Some(s) = SHARED.get() {
            if let Ok(mut e) = s.engine.try_lock() {
                e.clear();
            }
            s.interrupted.store(true, Ordering::Release);
            s.notify();
        }
    }
    if code < 0 || (msg != WM_MOUSEWHEEL as usize && msg != WM_MOUSEHWHEEL as usize) {
        return CallNextHookEx(std::ptr::null_mut(), code, msg, data);
    }
    if let Some(s) = SHARED.get() {
        let began = Instant::now();
        let m = &*(data as *const MSLLHOOKSTRUCT);
        let delta = ((m.mouseData >> 16) as u16 as i16) as i32;
        let injected =
            m.flags & (LLMHF_INJECTED | LLMHF_LOWER_IL_INJECTED) != 0 || m.dwExtraInfo == TAG;
        let modified = GetAsyncKeyState(VK_CONTROL as i32) < 0
            || GetAsyncKeyState(VK_MENU as i32) < 0
            || GetAsyncKeyState(VK_RBUTTON as i32) < 0;
        let mut accepted = false;
        if let (Ok(t), Ok(opts)) = (s.target.try_read(), s.settings.try_read()) {
            let eligible = t.eligible
                && GetForegroundWindow() as usize == t.hwnd
                && under_cursor(*t, s.pad.load(Ordering::Acquire), m.pt);
            if intercept(
                delta,
                injected,
                modified,
                eligible,
                s.enabled.load(Ordering::Acquire),
            ) {
                let shift = GetAsyncKeyState(VK_SHIFT as i32) < 0;
                let native_horizontal = msg == WM_MOUSEHWHEEL as usize;
                if (!native_horizontal && !shift) || opts.horizontal {
                    let horizontal = native_horizontal || shift;
                    let mut amount = if shift && !native_horizontal {
                        -delta
                    } else {
                        delta
                    };
                    if opts.reverse {
                        amount = -amount;
                    }
                    if let Ok(mut engine) = s.engine.try_lock() {
                        if s.interrupted.swap(false, Ordering::AcqRel) {
                            engine.clear();
                        }
                        let child = WindowFromPoint(m.pt) as usize;
                        if s.latch.load(Ordering::Acquire) != child {
                            engine.clear();
                        }
                        accepted = engine.push_tuned(s.now(), amount, horizontal, t.hwnd, t.tuning);
                        if accepted {
                            s.latch.store(child, Ordering::Release);
                        }
                    }
                }
            }
        }
        if !accepted && m.dwExtraInfo != TAG {
            // Fine/OS-generated input, modifiers and bypasses interrupt our existing tail.
            if let Ok(mut engine) = s.engine.try_lock() {
                engine.clear();
            }
            s.interrupted.store(true, Ordering::Release);
            s.notify();
        }
        s.hook_max_us
            .fetch_max(began.elapsed().as_micros() as u64, Ordering::Relaxed);
        if accepted {
            s.notify();
            return 1;
        }
    }
    CallNextHookEx(std::ptr::null_mut(), code, msg, data)
}
pub fn send_wheel(delta: i32, horizontal: bool) -> bool {
    unsafe {
        if delta == 0 {
            return true;
        }
        let input = INPUT {
            r#type: INPUT_MOUSE,
            Anonymous: INPUT_0 {
                mi: MOUSEINPUT {
                    dx: 0,
                    dy: 0,
                    mouseData: delta as u32,
                    dwFlags: if horizontal {
                        MOUSEEVENTF_HWHEEL
                    } else {
                        MOUSEEVENTF_WHEEL
                    },
                    time: 0,
                    dwExtraInfo: TAG,
                },
            },
        };
        SendInput(1, &input, size_of::<INPUT>() as i32) == 1
    }
}
fn worker(s: Arc<Shared>, ready: std::sync::mpsc::Sender<Result<(), String>>) {
    unsafe {
        let mut raw = CreateWaitableTimerExW(
            null(),
            null(),
            CREATE_WAITABLE_TIMER_HIGH_RESOLUTION,
            TIMER_ALL_ACCESS,
        );
        if raw.is_null() {
            raw = CreateWaitableTimerExW(null(), null(), 0, TIMER_ALL_ACCESS);
        }
        if raw.is_null() {
            let _ = ready.send(Err("Windows could not create the animation timer.".into()));
            return;
        }
        let timer = Handle(raw as usize);
        let _ = ready.send(Ok(()));
        let mut active = false;
        while !s.quitting.load(Ordering::Acquire) {
            if active {
                let handles = [s.wake.raw(), timer.raw()];
                if WaitForMultipleObjects(2, handles.as_ptr(), 0, INFINITE) == WAIT_FAILED {
                    s.fail("Animation wait failed; smoothing paused.");
                    break;
                }
            } else if WaitForSingleObject(s.wake.raw(), INFINITE) == WAIT_FAILED {
                s.fail("Input wait failed; smoothing paused.");
                break;
            }
            if s.quitting.load(Ordering::Acquire) {
                break;
            }
            // Keep the short engine lock until SendInput returns, serializing pause/target changes with emission.
            if let Ok(mut e) = s.engine.lock() {
                // Read the latch only after input acceptance is serialized with this frame.
                let target = s.target.read().map(|x| *x).unwrap_or_default();
                let mut p = zeroed();
                GetCursorPos(&mut p);
                let safe = s.enabled.load(Ordering::Acquire)
                    && target.eligible
                    && GetForegroundWindow() as usize == target.hwnd
                    && under_cursor(target, s.pad.load(Ordering::Acquire), p)
                    && WindowFromPoint(p) as usize == s.latch.load(Ordering::Acquire)
                    && GetAsyncKeyState(VK_CONTROL as i32) >= 0
                    && GetAsyncKeyState(VK_MENU as i32) >= 0
                    && GetAsyncKeyState(VK_RBUTTON as i32) >= 0;
                if !safe || s.interrupted.swap(false, Ordering::AcqRel) {
                    e.clear();
                    active = false;
                } else {
                    let values = e.tick(s.now(), target.hwnd);
                    active = e.active();
                    for (axis, delta) in values.into_iter().enumerate() {
                        if delta != 0 {
                            if !send_wheel(delta, axis == 1) {
                                e.clear();
                                active = false;
                                s.enabled.store(false, Ordering::Release);
                                if let Ok(mut f) = s.fault.lock() {
                                    *f="Windows rejected scroll output; smoothing paused. Ordinary wheel input passes through.".into();
                                }
                                s.update_ui();
                                break;
                            }
                            s.sent.fetch_add(1, Ordering::Relaxed);
                        }
                    }
                }
            }
            if active {
                s.ticks.fetch_add(1, Ordering::Relaxed);
                if SetWaitableTimer(timer.raw(), &-83333, 0, None, null(), 0) == 0 {
                    s.fail("Animation timer failed; smoothing paused.");
                    active = false;
                }
            } else {
                CancelWaitableTimer(timer.raw());
            }
        }
    }
}
pub struct Runtime {
    pub shared: Arc<Shared>,
    hook: Option<JoinHandle<()>>,
    worker: Option<JoinHandle<()>>,
}
impl Runtime {
    pub fn start(settings: Settings, enabled: bool) -> Result<Self, String> {
        unsafe {
            let event = CreateEventW(null(), 0, 0, null());
            if event.is_null() {
                return Err("Windows could not create the input event.".into());
            }
            let conflict = conflicting_engine_running(&settings);
            let s = Arc::new(Shared {
                settings: RwLock::new(settings),
                engine: Mutex::new(Engine::default()),
                enabled: AtomicBool::new(enabled && !conflict),
                quitting: AtomicBool::new(false),
                interrupted: AtomicBool::new(false),
                target: RwLock::new(Target::default()),
                ui: AtomicUsize::new(0),
                pad: AtomicUsize::new(0),
                latch: AtomicUsize::new(0),
                wake: Handle(event as usize),
                epoch: Instant::now(),
                hook_thread: AtomicU32::new(0),
                hook_live: AtomicBool::new(false),
                sent: AtomicU64::new(0),
                ticks: AtomicU64::new(0),
                hook_max_us: AtomicU64::new(0),
                fault: Mutex::new(if conflict {
                    "A configured competing scroll app is running (or cannot be checked). Pause it before enabling Scroll Mage.".into()
                } else {
                    String::new()
                }),
            });
            SHARED
                .set(s.clone())
                .map_err(|_| "Scroll engine is already initialized.".to_string())?;
            let (tx, rx) = std::sync::mpsc::channel();
            let ws = s.clone();
            let worker = thread::spawn(move || worker(ws, tx));
            rx.recv()
                .map_err(|_| "Animation worker did not start.".to_string())??;
            let (tx, rx) = std::sync::mpsc::channel();
            let hs = s.clone();
            let hook = thread::spawn(move || {
                let mut message = zeroed();
                PeekMessageW(&mut message, std::ptr::null_mut(), 0, 0, PM_NOREMOVE);
                hs.hook_thread
                    .store(GetCurrentThreadId(), Ordering::Release);
                let h =
                    SetWindowsHookExW(WH_MOUSE_LL, Some(mouse_hook), GetModuleHandleW(null()), 0);
                hs.hook_live.store(!h.is_null(), Ordering::Release);
                let _ = tx.send(!h.is_null());
                if !h.is_null() {
                    while GetMessageW(&mut message, std::ptr::null_mut(), 0, 0) > 0 {}
                    UnhookWindowsHookEx(h);
                }
                hs.hook_live.store(false, Ordering::Release);
            });
            if !rx.recv().unwrap_or(false) {
                s.quitting.store(true, Ordering::Release);
                s.notify();
                let _ = hook.join();
                let _ = worker.join();
                return Err("Windows could not install the mouse hook.".into());
            }
            Ok(Self {
                shared: s,
                hook: Some(hook),
                worker: Some(worker),
            })
        }
    }
}
impl Drop for Runtime {
    fn drop(&mut self) {
        self.shared.pause();
        self.shared.quitting.store(true, Ordering::Release);
        self.shared.notify();
        unsafe {
            PostThreadMessageW(
                self.shared.hook_thread.load(Ordering::Acquire),
                WM_QUIT,
                0,
                0,
            );
        }
        if let Some(t) = self.hook.take() {
            let _ = t.join();
        }
        if let Some(t) = self.worker.take() {
            let _ = t.join();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::AtomicI64;
    use std::time::Duration;
    use windows_sys::Win32::Graphics::Gdi::ClientToScreen;
    static V: AtomicI64 = AtomicI64::new(0);
    static H: AtomicI64 = AtomicI64::new(0);
    static EVENTS: AtomicU64 = AtomicU64::new(0);
    unsafe extern "system" fn receiver(w: HWND, m: u32, p: usize, l: isize) -> isize {
        if m == WM_MOUSEWHEEL || m == WM_MOUSEHWHEEL {
            let delta = ((p >> 16) as u16 as i16) as i64;
            if m == WM_MOUSEWHEEL {
                V.fetch_add(delta, Ordering::Relaxed);
            } else {
                H.fetch_add(delta, Ordering::Relaxed);
            }
            EVENTS.fetch_add(1, Ordering::Relaxed);
            return 0;
        }
        DefWindowProcW(w, m, p, l)
    }
    unsafe fn pump(duration: Duration) {
        let start = Instant::now();
        while start.elapsed() < duration {
            let mut m = zeroed();
            while PeekMessageW(&mut m, null_mut(), 0, 0, PM_REMOVE) != 0 {
                TranslateMessage(&m);
                DispatchMessageW(&m);
            }
            thread::sleep(Duration::from_millis(2));
        }
    }
    unsafe fn accept_constructed(msg: u32, event: &MSLLHOOKSTRUCT) {
        // A constructed call has no physical fallback event. Wait for the refresh
        // worker if the hook intentionally fails open because its lock is busy.
        let start = Instant::now();
        loop {
            if mouse_hook(0, msg as usize, event as *const _ as isize) == 1 {
                return;
            }
            assert!(
                start.elapsed() < Duration::from_millis(100),
                "Constructed input was not accepted."
            );
            pump(Duration::from_millis(2));
        }
    }
    unsafe fn settle(s: &Shared) {
        let start = Instant::now();
        while s.engine.lock().unwrap().active() {
            assert!(
                start.elapsed() < Duration::from_secs(2),
                "Wheel follower did not settle."
            );
            pump(Duration::from_millis(10));
        }
        // SendInput can return before the queued wheel message reaches this window.
        pump(Duration::from_millis(50));
    }
    use std::ptr::null_mut;
    struct Surface {
        window: usize,
        foreground: usize,
        cursor: POINT,
    }
    impl Drop for Surface {
        fn drop(&mut self) {
            unsafe {
                DestroyWindow(self.window as HWND);
                SetCursorPos(self.cursor.x, self.cursor.y);
                if self.foreground != 0 {
                    SetForegroundWindow(self.foreground as HWND);
                }
            }
        }
    }
    #[test]
    fn owned_window_receives_smoothed_tagged_input_and_worker_stops_when_idle() {
        unsafe {
            assert_eq!(size_of::<INPUT>(), 40);
            assert_eq!(size_of::<MSLLHOOKSTRUCT>(), 32);
            let hinst = GetModuleHandleW(null());
            let class = wide("ScrollPilot.NativeVerification");
            let wc = WNDCLASSW {
                lpfnWndProc: Some(receiver),
                hInstance: hinst,
                lpszClassName: class.as_ptr(),
                ..zeroed()
            };
            assert_ne!(RegisterClassW(&wc), 0);
            let mut cursor = zeroed();
            GetCursorPos(&mut cursor);
            let foreground = GetForegroundWindow() as usize;
            let window = CreateWindowExW(
                0,
                class.as_ptr(),
                wide("ScrollPilot controlled input verification").as_ptr(),
                WS_OVERLAPPEDWINDOW | WS_VISIBLE,
                160,
                160,
                460,
                280,
                null_mut(),
                null_mut(),
                hinst,
                null(),
            );
            assert!(!window.is_null());
            let _surface = Surface {
                window: window as usize,
                foreground,
                cursor,
            };
            // Attach only for the owned test window's activation, and immediately detach.
            let previous = GetForegroundWindow();
            let tid = GetWindowThreadProcessId(previous, null_mut());
            let own = GetCurrentThreadId();
            let attached = tid != own && AttachThreadInput(own, tid, 1) != 0;
            ShowWindow(window, SW_SHOW);
            BringWindowToTop(window);
            SetForegroundWindow(window);
            SetFocus(window);
            if attached {
                AttachThreadInput(own, tid, 0);
            }
            let mut point = POINT { x: 120, y: 100 };
            ClientToScreen(window, &mut point);
            SetCursorPos(point.x, point.y);
            pump(Duration::from_millis(200));
            assert_eq!(
                GetForegroundWindow(),
                window,
                "Controlled window must be foreground; no input emitted elsewhere."
            );
            let runtime = Runtime::start(
                Settings {
                    acceleration_max: 1.,
                    ..Settings::default()
                },
                true,
            )
            .unwrap();
            let s = &runtime.shared;
            s.pad.store(window as usize, Ordering::Release);
            s.refresh_target();
            assert!(s.target.read().unwrap().eligible);
            let m = MSLLHOOKSTRUCT {
                pt: point,
                mouseData: 120u32 << 16,
                flags: 0,
                time: 0,
                dwExtraInfo: 0,
            };
            accept_constructed(WM_MOUSEWHEEL, &m);
            settle(s);
            eprintln!("vertical={} events={} sent={} ticks={} active={} fg={:?} expected={:?} target={} fault={:?}",V.load(Ordering::Relaxed),EVENTS.load(Ordering::Relaxed),s.sent.load(Ordering::Relaxed),s.ticks.load(Ordering::Relaxed),s.engine.lock().unwrap().active(),GetForegroundWindow(),window,s.target.read().unwrap().hwnd,s.fault.lock().unwrap());
            assert_eq!(V.load(Ordering::Relaxed), 120);
            assert!(EVENTS.load(Ordering::Relaxed) > 4);
            accept_constructed(WM_MOUSEHWHEEL, &m);
            settle(s);
            assert_eq!(H.load(Ordering::Relaxed), 120);
            let emitted = s.sent.load(Ordering::Relaxed);
            assert!(send_wheel(-120, false));
            pump(Duration::from_millis(100));
            assert_eq!(V.load(Ordering::Relaxed), 0);
            assert_eq!(
                s.sent.load(Ordering::Relaxed),
                emitted,
                "Tagged injected input must pass through once without being re-smoothed."
            );
            accept_constructed(WM_MOUSEWHEEL, &m);
            let fine = MSLLHOOKSTRUCT {
                mouseData: 15u32 << 16,
                ..m
            };
            mouse_hook(0, WM_MOUSEWHEEL as usize, &fine as *const _ as isize);
            pump(Duration::from_millis(100));
            assert!(
                !s.engine.lock().unwrap().active(),
                "Fine input must cancel our old tail."
            );
            let ticks = s.ticks.load(Ordering::Relaxed);
            pump(Duration::from_millis(350));
            assert_eq!(
                s.ticks.load(Ordering::Relaxed),
                ticks,
                "Idle worker must stop scheduling frames."
            );
            s.pause();
            assert!(!s.enabled.load(Ordering::Acquire));
            assert!(!s.engine.lock().unwrap().active());
            let result = serde_json::json!({"test":"owned Windows input receiver","vertical_smoothed_total":120,"horizontal_smoothed_total":120,"output_events":emitted,"received_events":EVENTS.load(Ordering::Relaxed),"hook_max_microseconds":s.hook_max_us.load(Ordering::Relaxed),"idle_frame_ticks_added":0,"hardware_input":"NOT RUN - hook entry used a constructed event; output used actual Windows SendInput and a live hook","injection_loop_prevention":"passed"});
            std::fs::write(
                "target/native-verification.json",
                serde_json::to_vec_pretty(&result).unwrap(),
            )
            .unwrap();
            let shared = s.clone();
            drop(runtime);
            assert!(!shared.hook_live.load(Ordering::Acquire));
        }
    }
}
