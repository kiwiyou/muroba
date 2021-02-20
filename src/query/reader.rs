use crossterm::event::{KeyCode, KeyEvent};

pub trait TextReader {
    fn on_key(&mut self, event: &KeyEvent) -> bool;
    fn text(&self) -> &str;
    fn get_result(self) -> String;
}

#[derive(Default)]
pub struct PlainReader {
    input: String,
}

impl TextReader for PlainReader {
    fn on_key(&mut self, event: &KeyEvent) -> bool {
        match event.code {
            KeyCode::Backspace => self.input.pop().is_some(),
            KeyCode::Char(c) => {
                self.input.push(c);
                true
            }
            _ => false,
        }
    }

    fn text(&self) -> &str {
        &self.input
    }

    fn get_result(self) -> String {
        self.input
    }
}

pub struct SecretReader<S> {
    shield: S,
    password: String,
}

impl<S> SecretReader<S> {
    pub fn new(shield: S) -> Self {
        Self {
            shield,
            password: String::new(),
        }
    }
}

impl<S> TextReader for SecretReader<S>
where
    S: Shield,
{
    fn on_key(&mut self, event: &KeyEvent) -> bool {
        match event.code {
            KeyCode::Backspace => {
                self.password.pop();
                self.shield.pop()
            }
            KeyCode::Char(c) => {
                self.password.push(c);
                self.shield.push()
            }
            _ => false,
        }
    }

    fn text(&self) -> &str {
        self.shield.text()
    }

    fn get_result(self) -> String {
        self.password
    }
}

pub trait Shield {
    fn push(&mut self) -> bool;
    fn pop(&mut self) -> bool;
    fn text(&self) -> &str;
}

pub struct EmptyShield;
impl Shield for EmptyShield {
    fn push(&mut self) -> bool {
        false
    }

    fn pop(&mut self) -> bool {
        false
    }

    fn text(&self) -> &str {
        ""
    }
}

pub struct CharacterShield {
    c: char,
    buffer: String,
}

impl CharacterShield {
    pub fn new(c: char) -> Self {
        Self {
            c,
            buffer: String::new(),
        }
    }
}

impl Shield for CharacterShield {
    fn push(&mut self) -> bool {
        self.buffer.push(self.c);
        true
    }

    fn pop(&mut self) -> bool {
        self.buffer.pop().is_some()
    }

    fn text(&self) -> &str {
        &self.buffer
    }
}
