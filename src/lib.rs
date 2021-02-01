use std::{
    io::{stderr, stdin},
    marker::PhantomData,
};

use style::{DefaultStyle, Style};

pub mod style;

pub struct Input<S: Style> {
    prompt: Option<String>,
    _style: PhantomData<S>,
}

impl Default for Input<DefaultStyle> {
    fn default() -> Self {
        Self {
            prompt: None,
            _style: Default::default(),
        }
    }
}

impl<S: Style> Input<S> {
    fn with_style() -> Self {
        Self {
            prompt: None,
            _style: Default::default(),
        }
    }

    pub fn with_prompt(self, prompt: impl Into<String>) -> Self {
        Self {
            prompt: Some(prompt.into()),
            _style: self._style,
        }
    }

    pub fn interact(self) -> crossterm::Result<String> {
        let mut stderr = stderr();
        if let Some(prompt) = self.prompt {
            S::queue_prompt(&mut stderr, prompt)?;
        }
        S::queue_indicator(&mut stderr)?;
        S::format_input(&mut stderr)?;
        let mut input = String::new();
        let stdin = stdin();
        stdin.read_line(&mut input)?;
        S::unformat_input(&mut stderr)?;
        input.truncate(input.trim_end().len());
        Ok(input)
    }
}
