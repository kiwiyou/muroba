use std::{fmt::Display, io::Write};

use crossterm::{
    cursor::MoveToPreviousLine,
    event::{KeyCode, KeyEvent},
    queue,
    terminal::{Clear, ClearType},
};

use crate::{item::ListItem, style::Styler, Result};

pub trait SelectHandler<'a, T> {
    type Result;

    fn show(&mut self, f: &mut impl Write) -> Result<()>;
    fn clear(&mut self, f: &mut impl Write) -> Result<()>;
    /// Handles a key event and returns `true` if redraw is required.
    ///
    /// It should only handle movement events, such as Up and Down key.
    fn on_key(&mut self, key_event: KeyEvent) -> bool;
    /// Toggles selection state of current cursor item.
    fn toggle(&mut self);
    fn get_selected(self) -> Self::Result;
}

pub struct ListHandler<'a, S, T> {
    style: &'a S,
    list: &'a [T],
    cursor: usize,
    is_selected: Vec<bool>,
    last_printed_rows: u16,
}

impl<'a, S, T> ListHandler<'a, S, T>
where
    S: Styler<ListItem<'a, T>>,
    T: Display,
{
    pub fn new(style: &'a S, list: &'a [T]) -> Self {
        Self {
            style,
            list,
            cursor: 0,
            is_selected: vec![false; list.len()],
            last_printed_rows: 0,
        }
    }
}

impl<'a, S, T> SelectHandler<'a, T> for ListHandler<'a, S, T>
where
    S: Styler<ListItem<'a, T>>,
    T: Display,
{
    /// Returns a list of the index of the selected item.
    type Result = Vec<usize>;

    fn show(&mut self, f: &mut impl Write) -> Result<()> {
        let mut printed_rows = 0;

        for (i, item) in self.list.iter().enumerate() {
            self.style.style(
                f,
                ListItem {
                    item,
                    is_cursor: i == self.cursor,
                    is_selected: self.is_selected[i],
                },
            )?;
            writeln!(f)?;
            printed_rows += 1;
        }

        self.last_printed_rows = printed_rows;
        Ok(())
    }

    fn clear(&mut self, f: &mut impl Write) -> Result<()> {
        queue!(
            f,
            MoveToPreviousLine(self.last_printed_rows),
            Clear(ClearType::FromCursorDown),
        )?;
        self.last_printed_rows = 0;
        Ok(())
    }

    fn on_key(&mut self, key_event: KeyEvent) -> bool {
        match key_event.code {
            KeyCode::Up if self.cursor > 0 => {
                self.cursor -= 1;
                true
            }
            KeyCode::Down if self.cursor + 1 < self.list.len() => {
                self.cursor += 1;
                true
            }
            _ => false,
        }
    }

    fn toggle(&mut self) {
        self.is_selected[self.cursor] = !self.is_selected[self.cursor];
    }

    fn get_selected(self) -> Self::Result {
        self.is_selected
            .iter()
            .enumerate()
            .filter_map(|(i, selected)| selected.then(|| i))
            .collect()
    }
}

pub struct FixedRowHandler<'a, S, T> {
    style: &'a S,
    list: &'a [T],
    cursor: usize,
    rows: usize,
    is_selected: Vec<bool>,
    last_printed_rows: u16,
}

impl<'a, S, T> FixedRowHandler<'a, S, T>
where
    S: Styler<ListItem<'a, T>>,
    T: Display,
{
    pub fn new(style: &'a S, list: &'a [T], rows: usize) -> Self {
        Self {
            style,
            list,
            cursor: 0,
            rows,
            is_selected: vec![false; list.len()],
            last_printed_rows: 0,
        }
    }
}

impl<'a, S, T> SelectHandler<'a, T> for FixedRowHandler<'a, S, T>
where
    S: Styler<ListItem<'a, T>>,
    T: Display,
{
    /// Returns the index of the selected item.
    type Result = Vec<usize>;

    fn show(&mut self, f: &mut impl Write) -> Result<()> {
        let mut printed_rows = 0;

        let start = if self.list.len() < self.rows {
            0
        } else if self.rows >= 5 {
            (self.cursor + self.list.len() - 2) % self.list.len()
        } else if self.rows >= 3 {
            (self.cursor + self.list.len() - 1) % self.list.len()
        } else {
            self.cursor
        };

        let iter = self
            .list
            .iter()
            .enumerate()
            .cycle()
            .skip(start)
            .take(self.rows);
        for (i, item) in iter {
            self.style.style(
                f,
                ListItem {
                    item,
                    is_cursor: i == self.cursor,
                    is_selected: false,
                },
            )?;
            writeln!(f)?;
            printed_rows += 1;
        }

        self.last_printed_rows = printed_rows;
        Ok(())
    }

    fn clear(&mut self, f: &mut impl Write) -> Result<()> {
        queue!(
            f,
            MoveToPreviousLine(self.last_printed_rows),
            Clear(ClearType::FromCursorDown),
        )?;
        self.last_printed_rows = 0;
        Ok(())
    }

    fn on_key(&mut self, key_event: KeyEvent) -> bool {
        match key_event.code {
            KeyCode::Up => {
                if self.rows > self.list.len() {
                    if self.cursor > 0 {
                        self.cursor -= 1;
                        true
                    } else {
                        false
                    }
                } else {
                    self.cursor += self.list.len();
                    self.cursor -= 1;
                    self.cursor %= self.list.len();
                    true
                }
            }
            KeyCode::Down => {
                if self.rows > self.list.len() {
                    if self.cursor + 1 < self.list.len() {
                        self.cursor += 1;
                        true
                    } else {
                        false
                    }
                } else {
                    self.cursor += 1;
                    self.cursor %= self.list.len();
                    true
                }
            }
            _ => false,
        }
    }

    fn toggle(&mut self) {
        self.is_selected[self.cursor] = !self.is_selected[self.cursor];
    }

    fn get_selected(self) -> Self::Result {
        self.is_selected
            .iter()
            .enumerate()
            .filter_map(|(i, selected)| selected.then(|| i))
            .collect()
    }
}
