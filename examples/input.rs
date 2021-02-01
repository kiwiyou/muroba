use muroba::Input;

fn main() {
    let name = Input::new()
        .with_prompt("Hello! What's your name?")
        .interact()
        .unwrap();
    println!("Hello {}!", name);
}
