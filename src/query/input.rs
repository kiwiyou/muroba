use std::{
    ffi::OsString,
    io::{Read, Write},
    path::Path,
    process,
};

use crossterm::{
    cursor::{self, Hide, Show},
    event::{self, Event},
    queue,
    terminal::{disable_raw_mode, enable_raw_mode, Clear, ClearType},
};
use cursor::MoveToColumn;
use event::KeyCode;
use process::Command;

use crate::style::Styler;
use crate::Result;
use crate::{
    item::{BeginInput, ConfirmChoice, EndInput, Overflow, Prompt},
    util::trim_print,
};

use super::{
    reader::{CharacterShield, EmptyShield, PlainReader, SecretReader, TextReader},
    Query, QueryBuilder,
};

pub struct InputQuery<'a, S, R> {
    prompt: Prompt,
    style: &'a S,
    reader: R,
}

impl<'a, S, R> Query for InputQuery<'a, S, R>
where
    S: Styler<Prompt> + Styler<BeginInput> + Styler<EndInput> + Styler<Overflow>,
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
                    queue!(f, Hide, MoveToColumn(x + 1))?;
                    style.style(f, &BeginInput)?;
                    trim_print(style, f, reader.text())?;
                    queue!(f, Show, Clear(ClearType::UntilNewLine))?;
                    f.flush()?;
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
            reader: PlainReader::default(),
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

pub struct ConfirmQuery<'a, S> {
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

#[derive(Default)]
pub struct EditorQuery {
    editor: Option<OsString>,
}

impl EditorQuery {
    pub fn with_editor<P>(self, path: P) -> Self
    where
        P: AsRef<Path>,
    {
        Self {
            editor: Some(path.as_ref().as_os_str().to_owned()),
        }
    }

    fn editor(self) -> Command {
        let prog = self
            .editor
            .or_else(|| std::env::var_os("VISUAL"))
            .or_else(|| std::env::var_os("EDITOR"))
            .unwrap_or_else(|| if cfg!(windows) { "notepad.exe" } else { "vi" }.into());
        Command::new(prog)
    }
}

impl Query for EditorQuery {
    type Result = Option<String>;

    fn show_on(self, _: &mut impl Write) -> Result<Self::Result> {
        let temp = tempfile::NamedTempFile::new()?;

        if self.editor().arg(temp.path()).spawn()?.wait()?.success() {
            let mut text = String::new();
            temp.into_file().read_to_string(&mut text)?;
            Ok(Some(text))
        } else {
            Ok(None)
        }
    }
}

impl<'a, S> QueryBuilder<'a, S> {
    pub fn editor(self) -> EditorQuery {
        EditorQuery::default()
    }
}
