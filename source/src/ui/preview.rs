use super::*;
pub(super) unsafe extern "system" fn pad(hwnd: HWND, msg: u32, w: usize, l: isize) -> isize {
    let parent = GetAncestor(hwnd, GA_ROOT);
    let ptr = GetWindowLongPtrW(parent, GWLP_USERDATA) as *mut Ui;
    if ptr.is_null() {
        return DefWindowProcW(hwnd, msg, w, l);
    }
    let ui = &mut *ptr;
    match msg {
        WM_GETDLGCODE => DLGC_WANTARROWS as isize,
        WM_LBUTTONDOWN => {
            SetFocus(hwnd);
            SetCapture(hwnd);
            if let Ok(mut engine) = ui.shared.engine.lock() {
                engine.clear();
            }
            ui.shared.notify();
            let now = ui.shared.now();
            ui.drag = Some((
                (l as u16 as i16) as i32,
                ((l >> 16) as u16 as i16) as i32,
                now,
            ));
            for a in &mut ui.preview {
                a.begin();
            }
            KillTimer(hwnd, 1);
            0
        }
        WM_MOUSEMOVE if ui.drag.is_some() => {
            let (previous_x, previous_y, then) = ui.drag.unwrap();
            let x = (l as u16 as i16) as i32;
            let y = ((l >> 16) as u16 as i16) as i32;
            let now = ui.shared.now();
            ui.preview[0].change(f64::from(previous_x - x) / ui.scale, (now - then) / 1000.);
            ui.preview[1].change(f64::from(previous_y - y) / ui.scale, (now - then) / 1000.);
            ui.drag = Some((x, y, now));
            InvalidateRect(hwnd, null(), 0);
            0
        }
        WM_LBUTTONUP if ui.drag.is_some() => {
            let now = ui.shared.now();
            let (_, _, then) = ui.drag.take().unwrap();
            for a in &mut ui.preview {
                if now - then > 80. {
                    a.change(0., (now - then) / 1000.);
                }
                a.end(false);
            }
            ReleaseCapture();
            ui.frame_time = now;
            if ui.preview.iter().any(|a| a.active()) {
                SetTimer(hwnd, 1, 8, None);
            }
            0
        }
        WM_CAPTURECHANGED | WM_KILLFOCUS => {
            if ui.drag.take().is_some() {
                for a in &mut ui.preview {
                    a.cancel();
                }
                ui.frame_time = ui.shared.now();
                if ui.preview.iter().any(|a| a.active()) {
                    SetTimer(hwnd, 1, 8, None);
                }
            }
            0
        }
        WM_TIMER => {
            let now = ui.shared.now();
            let dt = ((now - ui.frame_time) / 1000.).max(0.);
            ui.frame_time = now;
            let opts = ui.shared.settings.read().unwrap();
            for a in &mut ui.preview {
                a.step(dt, opts.friction, opts.elastic_omega);
            }
            if !ui.preview.iter().any(|a| a.active()) {
                KillTimer(hwnd, 1);
            }
            InvalidateRect(hwnd, null(), 0);
            0
        }
        WM_MOUSEWHEEL | WM_MOUSEHWHEEL => {
            let delta = ((w >> 16) as u16 as i16) as i32;
            let axis = usize::from(msg == WM_MOUSEWHEEL);
            let units = if ui.wheel_lines[axis] == u32::MAX {
                ui.preview[axis].viewport
            } else {
                f64::from(ui.wheel_lines[axis]) * if axis == 1 { 20. } else { 10. }
            };
            let amount = f64::from(delta) / 120. * units * if axis == 1 { -1. } else { 1. };
            ui.preview[axis].cancel();
            ui.preview[axis].position =
                (ui.preview[axis].position + amount).clamp(0., ui.preview[axis].maximum);
            ui.frame_time = ui.shared.now();
            if ui.preview.iter().any(|a| a.active()) {
                SetTimer(hwnd, 1, 8, None);
            } else {
                KillTimer(hwnd, 1);
            }
            if axis == 1 {
                ui.wheel_total += delta as i64;
            } else {
                ui.horizontal_total += delta as i64;
            }
            ui.events += 1;
            InvalidateRect(hwnd, null(), 0);
            0
        }
        WM_KEYDOWN => {
            for a in &mut ui.preview {
                a.cancel();
            }
            match w as u16 {
                VK_UP => ui.preview[1].position = (ui.preview[1].position - 30.).max(0.),
                VK_DOWN => {
                    ui.preview[1].position =
                        (ui.preview[1].position + 30.).min(ui.preview[1].maximum)
                }
                VK_LEFT => ui.preview[0].position = (ui.preview[0].position - 30.).max(0.),
                VK_RIGHT => ui.preview[0].position = (ui.preview[0].position + 30.).min(500.),
                VK_HOME => ui.preview[1].position = 0.,
                VK_END => ui.preview[1].position = ui.preview[1].maximum,
                _ => return DefWindowProcW(hwnd, msg, w, l),
            }
            InvalidateRect(hwnd, null(), 0);
            0
        }
        WM_ERASEBKGND => 1,
        WM_SIZE => {
            let width = f64::from((l as u16).max(1)) / ui.scale;
            let height = f64::from(((l >> 16) as u16).max(1)) / ui.scale;
            if ui.preview[0].viewport != width || ui.preview[1].viewport != height {
                let maximum = (3400.
                    - (height - f64::from(scrollpilot::layout::TEST_HEADER)).max(0.))
                .max(0.);
                ui.preview[0] =
                    scrollpilot::motion::GestureAxis::new(ui.preview[0].position, 500., width);
                ui.preview[1] = scrollpilot::motion::GestureAxis::new(
                    ui.preview[1].position.min(maximum),
                    maximum,
                    height,
                );
                KillTimer(hwnd, 1);
            }
            0
        }
        WM_SETFOCUS => {
            InvalidateRect(hwnd, null(), 0);
            0
        }
        WM_PRINTCLIENT => {
            let mut rect = zeroed();
            GetClientRect(hwnd, &mut rect);
            ui.paint_preview_surface(hwnd, w as HDC, rect.right, rect.bottom);
            0
        }
        WM_PAINT => {
            ui.paint_preview(hwnd);
            0
        }
        _ => DefWindowProcW(hwnd, msg, w, l),
    }
}
