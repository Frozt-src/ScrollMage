fn main() {
    println!("cargo:rerun-if-changed=app.manifest");
    println!("cargo:rerun-if-changed=app.rc");
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=assets/icons/ScrollMage.ico");
    println!("cargo:rerun-if-changed=assets/icons/ScrollMage-tray.ico");
    println!("cargo:rerun-if-env-changed=RC");
    println!("cargo:rerun-if-env-changed=WindowsSdkDir");
    if std::env::var("CARGO_CFG_TARGET_ENV").as_deref() == Ok("msvc") {
        let root = std::path::PathBuf::from(std::env::var_os("CARGO_MANIFEST_DIR").unwrap());
        let out = std::path::PathBuf::from(std::env::var_os("OUT_DIR").unwrap());
        let resource = out.join("app-icons.res");
        // Icons come from app.rc; version details come from Cargo so the file
        // properties Windows shows always match the package version.
        let version = std::env::var("CARGO_PKG_VERSION").unwrap();
        let numbers: Vec<u16> = version
            .split(['.', '-', '+'])
            .take(3)
            .map(|part| part.parse().expect("Package version must be numeric"))
            .collect();
        assert_eq!(
            numbers.len(),
            3,
            "Package version must be major.minor.patch"
        );
        let commas = format!("{},{},{},0", numbers[0], numbers[1], numbers[2]);
        let script = out.join("app-version.rc");
        std::fs::write(
            &script,
            format!(
                r#"#include "app.rc"
1 VERSIONINFO
FILEVERSION {commas}
PRODUCTVERSION {commas}
FILEFLAGSMASK 0x3fL
FILEFLAGS 0x0L
FILEOS 0x40004L
FILETYPE 0x1L
FILESUBTYPE 0x0L
BEGIN
  BLOCK "StringFileInfo"
  BEGIN
    BLOCK "040904b0"
    BEGIN
      VALUE "CompanyName", "Scroll Mage contributors"
      VALUE "FileDescription", "Scroll Mage"
      VALUE "FileVersion", "{version}"
      VALUE "InternalName", "scrollpilot"
      VALUE "LegalCopyright", "Copyright (c) 2026 Frozt and Scroll Mage contributors. MIT License."
      VALUE "OriginalFilename", "ScrollMage.exe"
      VALUE "ProductName", "Scroll Mage"
      VALUE "ProductVersion", "{version}"
    END
  END
  BLOCK "VarFileInfo"
  BEGIN
    VALUE "Translation", 0x409, 1200
  END
END
"#
            ),
        )
        .expect("Could not write the version resource script");
        let output = std::process::Command::new(resource_compiler())
            .current_dir(&root)
            .arg("/nologo")
            .arg("/i")
            .arg(&root)
            .arg("/fo")
            .arg(&resource)
            .arg(&script)
            .output()
            .expect("Could not run the Windows SDK resource compiler");
        assert!(
            output.status.success(),
            "Icon resource compilation failed: {}{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
        println!(
            "cargo:rustc-link-arg-bin=scrollpilot={}",
            resource.display()
        );
        let manifest = std::path::Path::new(&std::env::var("CARGO_MANIFEST_DIR").unwrap())
            .join("app.manifest");
        println!("cargo:rustc-link-arg-bin=scrollpilot=/MANIFEST:EMBED");
        println!(
            "cargo:rustc-link-arg-bin=scrollpilot=/MANIFESTINPUT:{}",
            manifest.display()
        );
    }
}

fn resource_compiler() -> std::path::PathBuf {
    if let Some(path) = std::env::var_os("RC") {
        return path.into();
    }
    let sdk = std::env::var_os("WindowsSdkDir")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|| {
            std::path::PathBuf::from(
                std::env::var_os("ProgramFiles(x86)")
                    .expect("Set RC to the Windows SDK rc.exe path"),
            )
            .join("Windows Kits/10")
        });
    let mut versions: Vec<_> = std::fs::read_dir(sdk.join("bin"))
        .expect("Windows SDK is required; set RC to its rc.exe path")
        .flatten()
        .map(|entry| entry.path())
        .filter(|path| {
            path.file_name()
                .is_some_and(|name| name.to_string_lossy().starts_with("10."))
        })
        .collect();
    versions.sort_by_key(|path| {
        path.file_name()
            .unwrap()
            .to_string_lossy()
            .split('.')
            .map(|part| part.parse::<u32>().unwrap_or(0))
            .collect::<Vec<_>>()
    });
    versions
        .into_iter()
        .rev()
        .map(|path| path.join("x64/rc.exe"))
        .find(|path| path.is_file())
        .expect("Windows SDK x64 resource compiler was not found; set RC explicitly")
}
