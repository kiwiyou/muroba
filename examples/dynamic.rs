use std::time::Duration;

use muroba::query::{Query, QueryBuilder};

fn main() {
    let choice = QueryBuilder::default()
        .with_prompt("Choose a word within what you typed")
        .dyn_select(|input: String| input.split_ascii_whitespace().map(str::to_string).collect())
        .show()
        .unwrap();
    if let Some(choice) = choice {
        println!("You selected {}!", choice);
    } else {
        println!("Please select a word...");
        return;
    }

    let choice = QueryBuilder::default()
        .with_prompt("Which langauge is your favorite?")
        .dyn_select(|input: String| {
            if input.is_empty() {
                LANGUAGES.into_iter().collect()
            } else {
                std::thread::sleep(Duration::from_millis(500));
                LANGUAGES
                    .into_iter()
                    .filter(|lang| lang.starts_with(&input))
                    .collect()
            }
        })
        .fix_rows(5)
        .debounce(Duration::from_secs(1))
        .wait_message("Filtering...")
        .show()
        .unwrap();
    if let Some(choice) = choice {
        println!("Your favorite langauge is {}!", choice);
    } else {
        println!("You don't like any language... Seriously?");
    }
}

const LANGUAGES: &[&str] = &[
    "Ada",
    "Basic",
    "Common Lisp",
    "Dart",
    "Erlang",
    "Fortran",
    "Groovy",
    "Haskell",
    "Idris",
    "Julia",
    "Kotlin",
    "Lua",
    "MATLAB",
    "Nim",
    "OCaml",
    "Perl",
    "Q#",
    "Rust",
    "Scala",
    "TypeScript",
    "Unicat",
    "Verliog",
    "WASM",
    "XQuery",
    "Yorick",
    "Zig",
];
