use muroba::{Input, Interactive, Promptable};

fn main() {
    let name = Input::default()
        .with_prompt("Hello! What's your name?")
        .interact()
        .unwrap();
    println!("Hello {}!", name);
}
