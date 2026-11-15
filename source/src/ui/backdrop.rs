use super::*;

pub(super) struct Frost {
    origin: POINT,
    width: i32,
    height: i32,
    pub(super) pixels: Vec<u32>,
}

impl Ui {
    /// Render only this app's surfaces into memory. Never sample the desktop.
    pub(super) unsafe fn prepare_preset_menu_backdrop(&self, list: HWND) {
        let mut rect: RECT = zeroed();
        GetClientRect(list, &mut rect);
        let mut origin = POINT { x: 0, y: 0 };
        ClientToScreen(list, &mut origin);
        let (width, height) = (rect.right, rect.bottom);
        if width <= 0 || height <= 0 || width > 4096 || height > 2048 {
            return;
        }
        if self
            .preset_menu_backdrop
            .borrow()
            .as_ref()
            .is_some_and(|f| {
                f.origin.x == origin.x
                    && f.origin.y == origin.y
                    && f.width == width
                    && f.height == height
            })
        {
            return;
        }
        let radius = self.px(6).max(1);
        let pad = radius * 3;
        let (w, h) = (width + pad * 2, height + pad * 2);
        let dc = CreateCompatibleDC(null_mut());
        let info = bitmap_info(w, h);
        let mut bits = null_mut();
        let bitmap = CreateDIBSection(dc, &info, DIB_RGB_COLORS, &mut bits, null_mut(), 0);
        if bitmap.is_null() || bits.is_null() {
            DeleteDC(dc);
            return;
        }
        let old = SelectObject(dc, bitmap);
        let capture = POINT {
            x: origin.x - pad,
            y: origin.y - pad,
        };
        fill(dc, Rect::new(0, 0, w, h), 1., CANVAS);
        let mut root_origin = POINT { x: 0, y: 0 };
        ClientToScreen(self.hwnd as HWND, &mut root_origin);
        let saved = SaveDC(dc);
        SetViewportOrgEx(
            dc,
            root_origin.x - capture.x,
            root_origin.y - capture.y,
            null_mut(),
        );
        IntersectClipRect(dc, 0, 0, self.px(self.width), self.px(self.height));
        self.paint_main(dc);
        RestoreDC(dc, saved);
        let mut body_origin = POINT { x: 0, y: 0 };
        ClientToScreen(self.body as HWND, &mut body_origin);
        let saved = SaveDC(dc);
        SetViewportOrgEx(
            dc,
            body_origin.x - capture.x,
            body_origin.y - capture.y,
            null_mut(),
        );
        let mut body_client: RECT = zeroed();
        GetClientRect(self.body as HWND, &mut body_client);
        IntersectClipRect(dc, 0, 0, body_client.right, body_client.bottom);
        self.paint_body(dc);
        RestoreDC(dc, saved);
        for &(_, handle, _) in &self.controls {
            let child = handle as HWND;
            if GetWindowLongW(child, GWL_STYLE) as u32 & WS_VISIBLE == 0 {
                continue;
            }
            let mut child_origin = POINT { x: 0, y: 0 };
            ClientToScreen(child, &mut child_origin);
            let mut client: RECT = zeroed();
            GetClientRect(child, &mut client);
            if child_origin.x >= capture.x + w
                || child_origin.y >= capture.y + h
                || child_origin.x + client.right <= capture.x
                || child_origin.y + client.bottom <= capture.y
            {
                continue;
            }
            let saved = SaveDC(dc);
            SetViewportOrgEx(
                dc,
                child_origin.x - capture.x,
                child_origin.y - capture.y,
                null_mut(),
            );
            IntersectClipRect(dc, 0, 0, client.right, client.bottom);
            let mut parent = GetParent(child);
            let mut visible = true;
            while parent != self.hwnd as HWND && !parent.is_null() {
                if GetWindowLongW(parent, GWL_STYLE) as u32 & WS_VISIBLE == 0 {
                    visible = false;
                    break;
                }
                let mut p = POINT { x: 0, y: 0 };
                ClientToScreen(parent, &mut p);
                let mut r: RECT = zeroed();
                GetClientRect(parent, &mut r);
                IntersectClipRect(
                    dc,
                    p.x - child_origin.x,
                    p.y - child_origin.y,
                    p.x - child_origin.x + r.right,
                    p.y - child_origin.y + r.bottom,
                );
                parent = GetParent(parent);
            }
            if visible {
                SendMessageW(child, WM_PRINTCLIENT, dc as usize, PRF_CLIENT as isize);
            }
            RestoreDC(dc, saved);
        }
        GdiFlush();
        let mut pixels = std::slice::from_raw_parts(bits.cast::<u32>(), (w * h) as usize).to_vec();
        SelectObject(dc, old);
        DeleteObject(bitmap);
        DeleteDC(dc);
        scrollpilot::frost::blur_tint(
            &mut pixels,
            w as usize,
            h as usize,
            radius as usize,
            0xfaf7fb,
        );
        let mut cropped = Vec::with_capacity((width * height) as usize);
        for y in pad..height + pad {
            let start = (y * w + pad) as usize;
            cropped.extend_from_slice(&pixels[start..start + width as usize]);
        }
        *self.preset_menu_backdrop.borrow_mut() = Some(Frost {
            origin,
            width,
            height,
            pixels: cropped,
        });
    }

    pub(super) unsafe fn paint_preset_menu_backdrop(&self, list: HWND, dc: HDC, clip: RECT) {
        self.prepare_preset_menu_backdrop(list);
        let backdrop = self.preset_menu_backdrop.borrow();
        if let Some(frost) = backdrop.as_ref() {
            let saved = SaveDC(dc);
            IntersectClipRect(dc, clip.left, clip.top, clip.right, clip.bottom);
            StretchDIBits(
                dc,
                0,
                0,
                frost.width,
                frost.height,
                0,
                0,
                frost.width,
                frost.height,
                frost.pixels.as_ptr().cast(),
                &bitmap_info(frost.width, frost.height),
                DIB_RGB_COLORS,
                SRCCOPY,
            );
            RestoreDC(dc, saved);
        } else {
            let brush = CreateSolidBrush(color(0xfaf7fb));
            FillRect(dc, &clip, brush);
            DeleteObject(brush);
        }
    }
}

fn bitmap_info(width: i32, height: i32) -> BITMAPINFO {
    BITMAPINFO {
        bmiHeader: BITMAPINFOHEADER {
            biSize: size_of::<BITMAPINFOHEADER>() as u32,
            biWidth: width,
            biHeight: -height,
            biPlanes: 1,
            biBitCount: 32,
            biCompression: BI_RGB,
            ..unsafe { zeroed() }
        },
        ..unsafe { zeroed() }
    }
}
