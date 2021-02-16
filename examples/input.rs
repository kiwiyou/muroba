use muroba::query::{Query, QueryBuilder};

fn main() {
    let confirm_show = QueryBuilder::default()
        .with_prompt("May I ask you a question?")
        .confirm(Some(true))
        .show()
        .unwrap();

    if confirm_show {
        let name = QueryBuilder::default()
            .with_prompt("Hello! What's your name?")
            .input()
            .show()
            .unwrap();
        println!("Hello {}!", name);
    }
}
