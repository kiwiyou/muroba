use std::io::{stderr, Write};

use crate::item::Prompt;
use crate::style::{DefaultStyle, Styler};
use crate::Result;

mod input;
mod reader;
mod select;

pub use input::*;
pub use reader::*;
pub use select::*;

pub struct QueryBuilder<'a, S>
where
    S: Styler<Prompt>,
{
    prompt: Option<String>,
    style: &'a S,
}

impl<'a, S> QueryBuilder<'a, S>
where
    S: Styler<Prompt>,
{
    pub fn with_style(style: &'a S) -> Self {
        Self {
            prompt: None,
            style,
        }
    }

    pub fn with_prompt(self, prompt: impl Into<String>) -> Self {
        Self {
            prompt: Some(prompt.into()),
            ..self
        }
    }
}

impl Default for QueryBuilder<'_, DefaultStyle> {
    fn default() -> Self {
        Self {
            prompt: None,
            style: &DefaultStyle,
        }
    }
}

pub trait Query: Sized {
    type Result;

    fn show(self) -> Result<Self::Result> {
        self.show_on(&mut stderr())
    }

    fn show_on(self, f: &mut impl Write) -> Result<Self::Result>;
}
