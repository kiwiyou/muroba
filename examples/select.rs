use muroba::query::{Query, QueryBuilder};

fn main() {
    let choices = ["Apple", "Banana", "Kiwi"];
    let selected = QueryBuilder::default()
        .with_prompt("Which fruit is your favorite?")
        .select(&choices)
        .show()
        .unwrap();
    println!("Your favorite fruit is {}!", choices[selected]);

    let selected = QueryBuilder::default()
        .with_prompt("Which language is your favorite?")
        .select(LANGUAGES)
        .fix_rows(5)
        .show()
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
