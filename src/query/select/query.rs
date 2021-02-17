use std::{fmt::Display, io::Write, writeln};

use crossterm::{
    cursor::{self, Hide, MoveToColumn, MoveToPreviousLine},
    event::{self, Event, KeyCode},
    queue,
    style::Print,
    terminal::{disable_raw_mode, enable_raw_mode},
};
use cursor::Show;

use crate::{
    item::{BeginInput, EndInput, ListItem, Prompt},
    query::Query,
    style::Styler,
    Result,
};

use super::{FixedRowHandler, SelectHandler};

pub struct SelectQuery<'a, S, T, H>
where
    S: Styler<Prompt> + Styler<BeginInput> + Styler<EndInput>,
    T: Display,
    H: SelectHandler<'a, T>,
{
    pub(crate) prompt: Prompt,
    pub(crate) style: &'a S,
    pub(crate) list: &'a [T],
    pub(crate) handler: H,
    pub(crate) is_many: bool,
}

impl<'a, S, T, H> SelectQuery<'a, S, T, H>
where
    S: Styler<Prompt> + Styler<BeginInput> + Styler<EndInput> + Styler<ListItem<'a, T>>,
    T: Display,
    H: SelectHandler<'a, T>,
{
    pub fn new(prompt: Prompt, style: &'a S, list: &'a [T], handler: H) -> Self {
        Self {
            prompt,
            style,
            list,
            handler,
            is_many: false,
        }
    }

    pub fn many(self) -> Self {
        Self {
            is_many: true,
            ..self
        }
    }

    pub fn fix_rows(self, rows: usize) -> SelectQuery<'a, S, T, FixedRowHandler<'a, S, T>> {
        assert!(rows > 0);
        SelectQuery {
            prompt: self.prompt,
            style: self.style,
            list: self.list,
            handler: FixedRowHandler::new(self.style, self.list, rows),
            is_many: self.is_many,
        }
    }
}

impl<'a, S, T, H> Query for SelectQuery<'a, S, T, H>
where
    S: Styler<Prompt> + Styler<BeginInput> + Styler<EndInput> + Styler<ListItem<'a, T>>,
    T: Display,
    H: SelectHandler<'a, T, Result = Vec<usize>>,
{
    type Result = Vec<usize>;

    fn show_on(self, f: &mut impl Write) -> Result<Self::Result> {
        let Self {
            prompt,
            style,
            list,
            mut handler,
            is_many,
        } = self;

        queue!(f, Hide)?;

        style.style(f, prompt)?;
        let (input_x, _) = cursor::position()?;
        writeln!(f)?;

        handler.show(f)?;
        enable_raw_mode()?;
        let result = loop {
            if let Event::Key(event) = event::read()? {
                match event.code {
                    KeyCode::Enter => {
                        disable_raw_mode()?;
                        handler.clear(f)?;
                        if !is_many {
                            handler.toggle();
                        }
                        break handler.get_selected();
                    }
                    KeyCode::Char(' ') if is_many => {
                        handler.toggle();
                        disable_raw_mode()?;
                        handler.clear(f)?;
                        handler.show(f)?;
                        enable_raw_mode()?;
                    }
                    _ => {
                        if handler.on_key(event) {
                            disable_raw_mode()?;
                            handler.clear(f)?;
                            handler.show(f)?;
                            enable_raw_mode()?;
                        }
                    }
                }
            }
        };
        assert!(is_many || result.len() == 1);

        if !is_many {
            queue!(f, MoveToPreviousLine(1), MoveToColumn(input_x),)?;
            style.style(f, BeginInput)?;
            queue!(f, Print(&list[result[0]]))?;
            style.style(f, EndInput)?;
            writeln!(f)?;
        }

        queue!(f, Show)?;

        Ok(result)
    }
}
