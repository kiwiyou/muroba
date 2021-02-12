use muroba::{
    style::{DefaultStyle, Style},
    Interactive, Promptable,
};

fn main() {
    let choices = ["Apple", "Banana", "Kiwi"];
    let selected = DefaultStyle::select(&choices)
        .with_prompt("Which fruit is your favorite?")
        .interact()
        .unwrap();
    println!("Your favorite fruit is {}!", choices[selected]);

    let selected = DefaultStyle::select(LANGUAGES)
        .with_height(5)
        .with_prompt("Which language is your favorite?")
        .interact()
        .unwrap();
    println!("Your favorite language is {}!", LANGUAGES[selected]);
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
