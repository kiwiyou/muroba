use muroba::{
    style::{DefaultStyle, Style},
    Interactive, Promptable,
};

fn main() {
    let choice =
        DefaultStyle::dynamic_select("Type Something", "Waiting for Result...", |input| {
            input.split_ascii_whitespace().map(str::to_string).collect()
        })
        .with_prompt("Choose a word within what you typed")
        .interact()
        .unwrap();
    if let Some(choice) = choice {
        println!("You selected {}!", choice);
    } else {
        println!("You did not select anything.");
    }
}
