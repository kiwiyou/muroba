use muroba::{
    style::{DefaultStyle, Style},
    Interactive, Promptable,
};

fn main() {
    let choice = DefaultStyle::dynamic_select(|input| {
        input.split_ascii_whitespace().map(str::to_string).collect()
    })
    .with_placeholder("something containing spaces...")
    .with_prompt("Choose a word within what you typed")
    .interact()
    .unwrap();
    if let Some(choice) = choice {
        println!("You selected {}!", choice);
    } else {
        println!("You did not select anything.");
    }

    let selected = DefaultStyle::dynamic_select(|input| {
        if input.is_empty() {
            LANGUAGES.iter().copied().map(str::to_string).collect()
        } else {
            LANGUAGES
                .iter()
                .flat_map(|lang| {
                    if lang.starts_with(&input) {
                        Some(lang.to_string())
                    } else {
                        None
                    }
                })
                .collect()
        }
    })
    .with_height(5)
    .with_placeholder("programming language...")
    .with_wait_message("Waiting for response...")
    .with_prompt("Which language is your favorite?")
    .interact()
    .unwrap();
    if let Some(language) = selected {
        println!("Your favorite language is {}!", language);
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
