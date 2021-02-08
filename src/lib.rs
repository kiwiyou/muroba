use std::{io::{stderr, stdin, Write}, marker::PhantomData, writeln};

use crossterm::{cursor, event::{Event, KeyCode}, queue, terminal::{Clear, ClearType, disable_raw_mode, enable_raw_mode}};
use cursor::{Hide, MoveTo, MoveToColumn, MoveToPreviousLine, Show};
use style::Style;

pub mod style;

pub trait Interactive<S: Style>: Sized {
    type Result;
    fn interact(self) -> crossterm::Result<Self::Result> {
        self.interact_on(&mut stderr())
    }
    fn interact_on(self, f: &mut impl Write) -> crossterm::Result<Self::Result>;
}

pub struct Prompted<S: Style, I: Interactive<S>, T: AsRef<str>> {
    inner: I,
    prompt: T,
    _style: PhantomData<S>,
}

impl<S: Style, I: Interactive<S, Result = R>, R, T: AsRef<str>> Interactive<S> for Prompted<S, I, T> {
    type Result = R;

    fn interact_on(self, f: &mut impl Write) -> crossterm::Result<Self::Result> {
        S::print_prompt(f, self.prompt.as_ref())?;
        self.inner.interact_on(f)
    }
}

pub trait Promptable<S: Style, I: Interactive<S>> {
    fn with_prompt<T: AsRef<str>>(self, prompt: T) -> Prompted<S, I, T>;
}

impl<S: Style, I: Interactive<S>> Promptable<S, Self> for I {
    fn with_prompt<T: AsRef<str>>(self, prompt: T) -> Prompted<S, Self, T> {
        Prompted {
            inner: self,
            prompt: prompt.into(),
            _style: Default::default(),
        }
    }
}

pub struct Input<S: Style> {
    _style: PhantomData<S>,
}

impl<S: Style> Input<S> {
    pub fn new() -> Self {
        Self {
            _style: Default::default(),
        }
    }
}

impl<S: Style> Interactive<S> for Input<S> {
    type Result = String;

    fn interact_on(self, f: &mut impl Write) -> crossterm::Result<Self::Result> {
        S::print_indicator(f)?;
        S::format_input(f)?;
        f.flush()?;
        let mut input = String::new();
        let stdin = stdin();
        stdin.read_line(&mut input)?;
        S::unformat_input(f)?;
        f.flush()?;
        input.truncate(input.trim_end().len());
        Ok(input)
    }
}

pub struct Select<'a, S: Style, T: AsRef<str>> {
    choices: &'a [T],
    _style: PhantomData<S>,
}

impl<'a, S: Style, T: AsRef<str>> Select<'a, S, T> {
    pub fn new(choices: &'a [T]) -> Self {
        Self {
            choices,
            _style: Default::default(),
        }
    }
}

impl<S: Style, T: AsRef<str>> Interactive<S> for Select<'_, S, T> {
    type Result = usize;

    fn interact_on(self, f: &mut impl Write) -> crossterm::Result<Self::Result> {
        S::print_indicator(f)?;
        let (return_x, _) = cursor::position()?;
        writeln!(f)?;
        let mut cursor = 0;
        for (i , choice) in self.choices.iter().enumerate() {
            let current = i == cursor;
            S::print_list_item(f, choice.as_ref(), current)?;
            writeln!(f)?;
        }
        queue!(
            f,
            MoveToPreviousLine(self.choices.len() as u16),
            Hide,
        )?;
        enable_raw_mode()?;
        loop {
            if let Event::Key(event) = crossterm::event::read()? {
                let (_, mut y) = cursor::position()?;
                if event.code == KeyCode::Down && cursor < self.choices.len() - 1 {
                    queue!(
                        f,
                        MoveToColumn(0),
                        Clear(ClearType::CurrentLine),
                    )?;
                    S::print_list_item(f, self.choices[cursor].as_ref(), false)?;
                    cursor += 1;
                    y += 1;
                } else if event.code == KeyCode::Up && cursor > 0 {
                    queue!(
                        f,
                        MoveToColumn(0),
                        Clear(ClearType::CurrentLine),
                    )?;
                    S::print_list_item(f, self.choices[cursor].as_ref(), false)?;
                    cursor -= 1;
                    y -= 1;
                } else if event.code == KeyCode::Enter {
                    break
                }
                queue!(
                    f,
                    MoveTo(0, y),
                    Clear(ClearType::CurrentLine),
                )?;
                S::print_list_item(f, self.choices[cursor].as_ref(), true)?;
                f.flush()?;
            }
        }
        if cursor > 0 {
            queue!(f, MoveToPreviousLine(cursor as u16))?;
        }
        queue!(
            f,
            Clear(ClearType::FromCursorDown),
            MoveToPreviousLine(1),
            MoveToColumn(return_x + 1),
            Show,
        )?;
        S::format_input(f)?;
        write!(f, "{}", self.choices[cursor].as_ref())?;
        S::unformat_input(f)?;
        f.flush()?;
        disable_raw_mode()?;
        writeln!(f)?;
        Ok(cursor)
    }
}
