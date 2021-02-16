use std::io::{stdin, Write};

use crossterm::{
    event::{self, Event},
    terminal::{disable_raw_mode, enable_raw_mode},
};
use event::KeyCode;

use crate::item::{BeginInput, ConfirmChoice, EndInput, Prompt};
use crate::style::Styler;
use crate::Result;

use super::{Query, QueryBuilder};

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

pub struct ConfirmQuery<'a, S>
where
    S: Styler<Prompt> + Styler<BeginInput> + Styler<EndInput>,
{
    prompt: Prompt,
    style: &'a S,
    default: Option<bool>,
}

impl<'a, S> Query for ConfirmQuery<'a, S>
where
    S: Styler<Prompt> + Styler<ConfirmChoice> + Styler<BeginInput> + Styler<EndInput>,
{
    type Result = bool;

    fn show_on(self, f: &mut impl Write) -> Result<Self::Result> {
        let Self {
            prompt,
            style,
            default,
        } = self;

        style.style(f, prompt)?;
        style.style(f, ConfirmChoice(default))?;
        style.style(f, BeginInput)?;

        enable_raw_mode()?;
        let is_yes = loop {
            if let Event::Key(event) = event::read()? {
                match event.code {
                    KeyCode::Char(c @ 'y') | KeyCode::Char(c @ 'Y') => {
                        disable_raw_mode()?;
                        println!("{}", c);
                        break true;
                    }
                    KeyCode::Char(c @ 'n') | KeyCode::Char(c @ 'N') => {
                        disable_raw_mode()?;
                        println!("{}", c);
                        break false;
                    }
                    KeyCode::Enter if default.is_some() => {
                        disable_raw_mode()?;
                        let default = default.unwrap();
                        let repr = if default { 'Y' } else { 'N' };
                        println!("{}", repr);
                        break default;
                    }
                    _ => {}
                }
            }
        };

        style.style(f, EndInput)?;

        Ok(is_yes)
    }
}

impl<'a, S> QueryBuilder<'a, S>
where
    S: Styler<Prompt> + Styler<BeginInput> + Styler<EndInput>,
{
    pub fn confirm(self, default: Option<bool>) -> ConfirmQuery<'a, S> {
        ConfirmQuery {
            prompt: Prompt(self.prompt.unwrap_or_default()),
            style: self.style,
            default,
        }
    }
}
