use std::io::{stderr, stdin};

use crossterm::{execute, queue, style::{Attribute, Color, Print, ResetColor, SetAttribute, SetForegroundColor}};

pub struct Input {
    prompt: Option<String>,
}

impl Input {
    pub fn new() -> Self {
        Self {
            prompt: None,
        }
    }

    pub fn with_prompt(self, prompt: impl Into<String>) -> Self {
        Self {
            prompt: Some(prompt.into()),
        }
    }

    pub fn interact(self) -> crossterm::Result<String> {
        let mut stderr = stderr();
        if let Some(mut prompt) = self.prompt {
            prompt.push_str(" ");
            queue!(
                stderr,
                SetForegroundColor(Color::Green),
                Print("? ".to_string()),
                ResetColor,
                SetAttribute(Attribute::Bold),
                Print(prompt),
                ResetColor,
            )?;
        }
        execute!(
            stderr,
            SetForegroundColor(Color::DarkGrey),
            Print("> ".to_string()),
            ResetColor,
        )?;
        let mut input = String::new();
        let stdin = stdin();
        stdin.read_line(&mut input)?;
        input.truncate(input.trim_end().len());
        Ok(input)
    }
}
