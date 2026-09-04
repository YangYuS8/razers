// SPDX-License-Identifier: GPL-2.0-or-later

fn main() {
    cargo_packager::cli::run(std::env::args_os().skip(1), None);
}
