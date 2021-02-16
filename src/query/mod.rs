use std::io::{stderr, stdin, Write};

use crate::item::{BeginInput, EndInput, Prompt};
use crate::style::{DefaultStyle, Styler};
use crate::Result;

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

impl<'a, S> QueryBuilder<'a, S>
where
    S: Styler<Prompt> + Styler<BeginInput> + Styler<EndInput>,
{
    pub fn input(self) -> InputQuery<'a, S> {
        InputQuery {
            prompt: Prompt(self.prompt.unwrap_or_default()),
            style: self.style,
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

pub struct InputQuery<'a, S>
where
    S: Styler<Prompt> + Styler<BeginInput> + Styler<EndInput>,
{
    prompt: Prompt,
    style: &'a S,
}

impl<'a, S> Query for InputQuery<'a, S>
where
    S: Styler<Prompt> + Styler<BeginInput> + Styler<EndInput>,
{
    type Result = String;

    fn show_on(self, f: &mut impl Write) -> Result<Self::Result> {
        let Self { prompt, style } = self;

        style.style(f, prompt)?;
        style.style(f, BeginInput)?;
        let mut input = String::new();
        stdin().read_line(&mut input)?;
        style.style(f, EndInput)?;

        input.truncate(input.trim_end_matches(['\n', '\r'].as_ref()).len());
        Ok(input)
    }
}
