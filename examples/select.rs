use muroba::{Interactive, Promptable, style::{DefaultStyle, Style}};

fn main() {
    let choices = ["Apple", "Banana", "Kiwi"];
    let selected = DefaultStyle::select(&choices)
        .with_prompt("Which fruit is your favorite?")
        .interact()
        .unwrap();
    println!("Your favorite fruit is {}!", choices[selected]);
}