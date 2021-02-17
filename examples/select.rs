use muroba::query::{Query, QueryBuilder};

fn main() {
    let choices = ["Apple", "Banana", "Kiwi"];
    let selected = QueryBuilder::default()
        .with_prompt("What fruit(s) do you like?")
        .select(&choices)
        .many()
        .show()
        .unwrap();
    match selected.len() {
        0 => println!("You don't like fruit?"),
        1 => println!("Your favorite fruit is {}!", choices[selected[0]]),
        _ => println!(
            "Your favorite fruits are {}!",
            join_string(selected.iter().map(|i| choices[*i]))
        ),
    }

    let selected = QueryBuilder::default()
        .with_prompt("Which language is your favorite?")
        .select(LANGUAGES)
        .fix_rows(5)
        .show()
        .unwrap();
    println!("Your favorite language is {}!", LANGUAGES[selected[0]]);
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

fn join_string<'a>(mut iter: impl Iterator<Item = &'a str>) -> String {
    if let Some(first) = iter.next() {
        let concat = first.to_string();
        iter.fold(concat, |mut concat, s| {
            concat += ", ";
            concat += s;
            concat
        })
    } else {
        Default::default()
    }
}
