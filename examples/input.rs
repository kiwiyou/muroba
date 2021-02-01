use muroba::{Interactive, Promptable, style::{DefaultStyle, Style}};

fn main() {
    let name = DefaultStyle::input()
        .with_prompt("Hello! What's your name?")
        .interact()
        .unwrap();
    println!("Hello {}!", name);
}
