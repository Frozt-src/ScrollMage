use super::*;

struct RegistryKey(HKEY);
impl Drop for RegistryKey {
    fn drop(&mut self) {
        unsafe {
            RegCloseKey(self.0);
        }
    }
}

fn startup_command(exe: &std::path::Path) -> Result<Vec<u16>, String> {
    use std::os::windows::ffi::OsStrExt;
    let mut command = vec![b'"' as u16];
    command.extend(exe.as_os_str().encode_wide());
    command.extend("\" --tray".encode_utf16());
    if command.len() > 260 || command.contains(&0) {
        return Err("The sign-in command exceeds Windows' 260-character limit or contains invalid data. Move Scroll Mage to a shorter path.".into());
    }
    command.push(0);
    Ok(command)
}

fn startup_value_matches(kind: u32, value: &[u16], exe: &std::path::Path) -> bool {
    if kind != REG_SZ || value.last() != Some(&0) || value[..value.len() - 1].contains(&0) {
        return false;
    }
    startup_command(exe).is_ok_and(|expected| {
        value.len() == expected.len()
            && value.iter().zip(expected).all(|(a, b)| {
                let fold = |c: u16| if (65..=90).contains(&c) { c + 32 } else { c };
                fold(*a) == fold(b)
            })
    })
}

pub(super) unsafe fn startup_enabled() -> bool {
    let Ok(exe) = std::env::current_exe() else {
        return false;
    };
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
    let key = RegistryKey(key);
    let mut value = [0u16; 261];
    let mut kind = 0;
    let mut len = std::mem::size_of_val(&value) as u32;
    let result = RegQueryValueExW(
        key.0,
        wide("ScrollMage").as_ptr(),
        null(),
        &mut kind,
        value.as_mut_ptr().cast(),
        &mut len,
    );
    result == 0
        && len.is_multiple_of(2)
        && len as usize <= std::mem::size_of_val(&value)
        && startup_value_matches(kind, &value[..len as usize / 2], &exe)
}
pub(super) unsafe fn set_startup(enabled: bool) -> Result<(), String> {
    // Validate before opening the registry, and preserve Unicode path code units.
    let command = if enabled {
        Some(startup_command(
            &std::env::current_exe().map_err(|e| e.to_string())?,
        )?)
    } else {
        None
    };
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
    let key = RegistryKey(key);
    let result = if enabled {
        let v = command.as_ref().unwrap();
        RegSetValueExW(
            key.0,
            wide("ScrollMage").as_ptr(),
            0,
            REG_SZ,
            v.as_ptr().cast(),
            (v.len() * 2) as u32,
        )
    } else {
        let r = RegDeleteValueW(key.0, wide("ScrollMage").as_ptr());
        if r == ERROR_FILE_NOT_FOUND {
            0
        } else {
            r
        }
    };
    if result == 0 {
        Ok(())
    } else {
        Err(format!("Windows error {result}"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn hidden_native_startup_command_matches_this_executable_only() {
        let exe = PathBuf::from(r"C:\Apps\Scroll Mage\ScrollMage.exe");
        let correct = wide(r#""C:\Apps\Scroll Mage\ScrollMage.exe" --tray"#);
        assert!(startup_value_matches(REG_SZ, &correct, &exe));
        assert!(!startup_value_matches(
            REG_SZ,
            &wide(r#""C:\Old\ScrollMage.exe" --tray"#),
            &exe
        ));
        assert!(!startup_value_matches(REG_BINARY, &correct, &exe));
        assert!(!startup_value_matches(
            REG_SZ,
            &wide(r#""C:\Apps\Scroll Mage\ScrollMage.exe" --tray --other"#),
            &exe
        ));
        assert!(!startup_value_matches(
            REG_SZ,
            &correct[..correct.len() - 1],
            &exe
        ));
        let mut embedded_nul = correct.clone();
        embedded_nul.extend(wide("untrusted suffix"));
        assert!(!startup_value_matches(REG_SZ, &embedded_nul, &exe));
        assert!(startup_command(&PathBuf::from(r"C:\工具\ScrollMage.exe")).is_ok());
        assert!(startup_command(&PathBuf::from(format!(
            r"C:\{}\ScrollMage.exe",
            "a".repeat(260)
        )))
        .is_err());
    }
}
