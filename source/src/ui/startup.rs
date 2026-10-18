use super::*;
pub(super) unsafe fn startup_enabled() -> bool {
    let mut key = null_mut();
    if RegOpenKeyExW(
        HKEY_CURRENT_USER,
        wide("Software\\Microsoft\\Windows\\CurrentVersion\\Run").as_ptr(),
        0,
        KEY_QUERY_VALUE,
        &mut key,
    ) != 0
    {
        return false;
    }
    let mut len = 0;
    let result = RegQueryValueExW(
        key,
        wide("ScrollMage").as_ptr(),
        null(),
        null_mut(),
        null_mut(),
        &mut len,
    );
    RegCloseKey(key);
    result == 0 && len > 2
}
pub(super) unsafe fn set_startup(enabled: bool) -> Result<(), String> {
    let mut key = null_mut();
    let result = RegCreateKeyExW(
        HKEY_CURRENT_USER,
        wide("Software\\Microsoft\\Windows\\CurrentVersion\\Run").as_ptr(),
        0,
        null(),
        0,
        KEY_SET_VALUE,
        null(),
        &mut key,
        null_mut(),
    );
    if result != 0 {
        return Err(format!("Windows error {result}"));
    }
    let result = if enabled {
        let exe = std::env::current_exe().map_err(|e| e.to_string())?;
        let v = wide(&format!("\"{}\" --tray", exe.display()));
        RegSetValueExW(
            key,
            wide("ScrollMage").as_ptr(),
            0,
            REG_SZ,
            v.as_ptr().cast(),
            (v.len() * 2) as u32,
        )
    } else {
        let r = RegDeleteValueW(key, wide("ScrollMage").as_ptr());
        if r == ERROR_FILE_NOT_FOUND {
            0
        } else {
            r
        }
    };
    RegCloseKey(key);
    if result == 0 {
        Ok(())
    } else {
        Err(format!("Windows error {result}"))
    }
}
