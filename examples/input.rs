use muroba::{
    style::{DefaultStyle, Style},
    Interactive, Promptable,
};

fn main() {
    let name = DefaultStyle::input()
        .with_prompt("Hello! What's your name?")
        .interact()
        .unwrap();
    println!("Hello {}!", name);
}
