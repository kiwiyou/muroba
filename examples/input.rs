use muroba::query::{Query, QueryBuilder};

fn main() {
    let name = QueryBuilder::default()
        .with_prompt("Hello! What's your name?")
        .input()
        .show()
        .unwrap();
    println!("Hello {}!", name);
}
