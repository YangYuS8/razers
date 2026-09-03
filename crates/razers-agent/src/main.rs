// SPDX-License-Identifier: GPL-2.0-or-later

use std::io;

const HELP: &str = "razers-agent - local RazeRS device service\n\nUSAGE:\n  razers-agent --stdio\n\nThe current transport accepts newline-delimited JSON-RPC 2.0 over inherited standard I/O.\n";

fn main() {
    match std::env::args().skip(1).collect::<Vec<_>>().as_slice() {
        [argument] if argument == "--stdio" => {
            if let Err(error) = razers_agent::serve_stdio(io::stdin().lock(), io::stdout().lock()) {
                eprintln!("error: {error}");
                std::process::exit(1);
            }
        }
        [argument] if argument == "--help" || argument == "-h" => print!("{HELP}"),
        _ => {
            eprint!("{HELP}");
            std::process::exit(2);
        }
    }
}
