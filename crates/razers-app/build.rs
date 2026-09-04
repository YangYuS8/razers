// SPDX-License-Identifier: GPL-2.0-or-later

#[cfg(windows)]
fn main() {
    let icon =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets/icons/razers.ico");
    assert!(icon.is_file(), "application icon must exist");
    let icon = icon.to_string_lossy().replace('\\', "/");
    let version = env!("CARGO_PKG_VERSION");
    let numeric_version = format!(
        "{},{},{},0",
        env!("CARGO_PKG_VERSION_MAJOR"),
        env!("CARGO_PKG_VERSION_MINOR"),
        env!("CARGO_PKG_VERSION_PATCH")
    );
    let resource = std::path::PathBuf::from(std::env::var_os("OUT_DIR").unwrap()).join("razers.rc");
    std::fs::write(
        &resource,
        format!(
            r#"#include <windows.h>
1 ICON "{icon}"
1 VERSIONINFO
FILEVERSION {numeric_version}
PRODUCTVERSION {numeric_version}
FILEOS VOS_NT_WINDOWS32
FILETYPE VFT_APP
BEGIN
  BLOCK "StringFileInfo"
  BEGIN
    BLOCK "040904B0"
    BEGIN
      VALUE "FileDescription", "RazeRS\0"
      VALUE "FileVersion", "{version}\0"
      VALUE "ProductName", "RazeRS\0"
      VALUE "ProductVersion", "{version}\0"
      VALUE "OriginalFilename", "razers.exe\0"
    END
  END
  BLOCK "VarFileInfo"
  BEGIN
    VALUE "Translation", 0x0409, 1200
  END
END
"#
        ),
    )
    .expect("write Windows application resource");
    embed_resource::compile_for(&resource, ["razers"], embed_resource::NONE)
        .manifest_required()
        .expect("embed Windows icon and version");
    println!("cargo:rerun-if-changed=../../assets/icons/razers.ico");
    println!("cargo:rerun-if-changed=build.rs");
}

#[cfg(not(windows))]
fn main() {}
