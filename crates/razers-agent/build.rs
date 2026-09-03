// SPDX-License-Identifier: GPL-2.0-or-later

use std::{env, fs, path::PathBuf};

fn main() {
    let manifest_directory = PathBuf::from(
        env::var_os("CARGO_MANIFEST_DIR").expect("Cargo must set CARGO_MANIFEST_DIR"),
    );
    let devices_directory = manifest_directory.join("../../devices");
    println!("cargo:rerun-if-changed={}", devices_directory.display());

    let mut paths = fs::read_dir(&devices_directory)
        .expect("devices directory must be readable")
        .map(|entry| {
            entry
                .expect("device manifest entry must be readable")
                .path()
        })
        .filter(|path| {
            path.extension()
                .is_some_and(|extension| extension == "toml")
        })
        .collect::<Vec<_>>();
    paths.sort();

    let mut generated = String::from("&[\n");
    for path in paths {
        let name = path
            .file_name()
            .expect("device manifest must have a file name")
            .to_string_lossy();
        let source = fs::read_to_string(&path).expect("device manifest must be valid UTF-8");
        generated.push_str(&format!("    ({name:?}, {source:?}),\n"));
    }
    generated.push_str("]\n");

    let output_directory = PathBuf::from(env::var_os("OUT_DIR").expect("Cargo must set OUT_DIR"));
    fs::write(output_directory.join("embedded_devices.rs"), generated)
        .expect("generated device manifest list must be writable");
}
