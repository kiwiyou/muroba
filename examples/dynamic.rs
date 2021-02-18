use muroba::query::QueryBuilder;

fn main() {
    let choice = QueryBuilder::default()
        .with_prompt("Choose a word within what you typed")
        .dyn_select(|input: String| input.split_ascii_whitespace().map(str::to_string).collect())
        .show()
        .unwrap();
    if let Some(choice) = choice {
        println!("You selected {}!", choice);
    } else {
        println!("You did not select anything.");
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
