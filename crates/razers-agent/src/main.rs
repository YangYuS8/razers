// SPDX-License-Identifier: GPL-2.0-or-later

use razers_i18n::{Locale, language_args};
use std::io;

fn main() {
    let (language, args) =
        language_args(std::env::args().skip(1).collect()).unwrap_or_else(|error| {
            eprintln!(
                "{}: {}",
                Locale::system().text("error"),
                Locale::system().text(&error)
            );
            std::process::exit(2);
        });
    let locale = language.unwrap_or_default().resolve();
    match args.as_slice() {
        [argument] if argument == "--stdio" => {
            if let Err(error) = razers_agent::serve_stdio(io::stdin().lock(), io::stdout().lock()) {
                eprintln!("{}: {error}", locale.text("error"));
                std::process::exit(1);
            }
        }
        [argument] if argument == "--help" || argument == "-h" => {
            print!("{}", locale.text("agent.help"))
        }
        _ => {
            eprint!("{}", locale.text("agent.help"));
            std::process::exit(2);
        }
    }
}
