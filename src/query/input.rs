use std::io::{stdin, Write};

use crossterm::{
    cursor,
    event::{self, Event},
    queue,
    style::Print,
    terminal::{disable_raw_mode, enable_raw_mode, Clear, ClearType},
};
use cursor::MoveToColumn;
use event::KeyCode;

use crate::item::{BeginInput, ConfirmChoice, EndInput, Prompt};
use crate::style::Styler;
use crate::Result;

use super::{
    reader::{CharacterShield, EmptyShield, SecretReader, PlainReader, TextReader},
    Query, QueryBuilder,
};

pub struct InputQuery<'a, S, R> {
    prompt: Prompt,
    style: &'a S,
    reader: R,
}

impl<'a, S, R> Query for InputQuery<'a, S, R>
where
    S: Styler<Prompt> + Styler<BeginInput> + Styler<EndInput>,
    R: TextReader,
{
    type Result = String;

    fn show_on(self, f: &mut impl Write) -> Result<Self::Result> {
        let Self {
            prompt,
            style,
            mut reader,
        } = self;

        style.style(f, &prompt)?;
        let (x, _) = cursor::position()?;
        style.style(f, &BeginInput)?;
        enable_raw_mode()?;
        let result = loop {
            if let Event::Key(event) = event::read()? {
                let redraw = match event.code {
                    KeyCode::Enter => break reader.get_result(),
                    _ => reader.on_key(&event),
                };
                if redraw {
                    disable_raw_mode()?;
                    queue!(f, MoveToColumn(x + 1))?;
                    style.style(f, &BeginInput)?;
                    queue!(f, Print(reader.text()), Clear(ClearType::UntilNewLine))?;
                    enable_raw_mode()?;
                }
            }
        };
        disable_raw_mode()?;
        style.style(f, &EndInput)?;
        writeln!(f)?;

        Ok(result)
    }
}

impl<'a, S> QueryBuilder<'a, S>
where
    S: Styler<Prompt> + Styler<BeginInput> + Styler<EndInput>,
{
    pub fn input(self) -> InputQuery<'a, S, PlainReader> {
        InputQuery {
            prompt: Prompt(self.prompt.unwrap_or_default()),
            style: self.style,
            reader: PlainReader::new(),
        }
    }

    pub fn secret(self) -> InputQuery<'a, S, SecretReader<EmptyShield>> {
        InputQuery {
            prompt: Prompt(self.prompt.unwrap_or_default()),
            style: self.style,
            reader: SecretReader::new(EmptyShield),
        }
    }
}

impl<'a, S> InputQuery<'a, S, SecretReader<EmptyShield>> {
    pub fn with_replace_char(self, c: char) -> InputQuery<'a, S, SecretReader<CharacterShield>> {
        InputQuery {
            prompt: self.prompt,
            style: self.style,
            reader: SecretReader::new(CharacterShield::new(c)),
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

        style.style(f, &prompt)?;
        style.style(f, &ConfirmChoice(default))?;
        style.style(f, &BeginInput)?;

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

        style.style(f, &EndInput)?;

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
