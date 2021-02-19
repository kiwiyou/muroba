use std::{fmt::Display, io::Write};

use crossterm::{
    cursor::MoveToPreviousLine,
    event::{KeyCode, KeyEvent},
    queue,
    terminal::{Clear, ClearType},
};

use crate::{item::ListItem, style::Styler, Result};

pub trait SelectHandler {
    type Result;

    fn show(&mut self, f: &mut impl Write) -> Result<()>;
    fn clear(&mut self, f: &mut impl Write) -> Result<()>;
    /// Handles a key event and returns `true` if redraw is required.
    ///
    /// It should only handle movement events, such as Up and Down key.
    fn on_key(&mut self, key_event: KeyEvent) -> bool;
    /// Toggles selection state of current cursor item.
    fn toggle(&mut self);
    fn get_result(self) -> Self::Result;
}

pub struct ListHandler<'a, S> {
    style: &'a S,
    list: Vec<ListItem>,
    cursor: usize,
    last_printed_rows: u16,
}

impl<'a, S> ListHandler<'a, S> {
    pub fn new(style: &'a S, list: &[impl Display]) -> Self {
        let mut list: Vec<_> = list
            .iter()
            .map(|item| ListItem {
                item: item.to_string(),
                is_cursor: false,
                is_selected: false,
            })
            .collect();
        if let Some(first) = list.get_mut(0) {
            first.is_cursor = true;
        }
        Self {
            style,
            list,
            cursor: 0,
            last_printed_rows: 0,
        }
    }
}

impl<'a, S> SelectHandler for ListHandler<'a, S>
where
    S: Styler<ListItem>,
{
    /// Returns a list of the index of the selected item and the value.
    type Result = Vec<(usize, String)>;

    fn show(&mut self, f: &mut impl Write) -> Result<()> {
        let mut printed_rows = 0;

        for item in self.list.iter() {
            self.style.style(f, item)?;
            writeln!(f)?;
            printed_rows += 1;
        }

        self.last_printed_rows = printed_rows;
        Ok(())
    }

    fn clear(&mut self, f: &mut impl Write) -> Result<()> {
        if self.last_printed_rows > 0 {
            queue!(
                f,
                MoveToPreviousLine(self.last_printed_rows),
                Clear(ClearType::FromCursorDown),
            )?;
            self.last_printed_rows = 0;
        }
        Ok(())
    }

    fn on_key(&mut self, key_event: KeyEvent) -> bool {
        match key_event.code {
            KeyCode::Up if self.cursor > 0 => {
                self.list[self.cursor].is_cursor = false;
                self.cursor -= 1;
                self.list[self.cursor].is_cursor = true;
                true
            }
            KeyCode::Down if self.cursor + 1 < self.list.len() => {
                self.list[self.cursor].is_cursor = false;
                self.cursor += 1;
                self.list[self.cursor].is_cursor = true;
                true
            }
            _ => false,
        }
    }

    fn toggle(&mut self) {
        if !self.list.is_empty() {
            let is_selected = &mut self.list[self.cursor].is_selected;
            *is_selected = !*is_selected;
        }
    }

    fn get_result(self) -> Self::Result {
        self.list
            .into_iter()
            .enumerate()
            .filter_map(|(i, item)| item.is_selected.then(|| (i, item.item)))
            .collect()
    }
}

pub struct FixedRowHandler<'a, S> {
    style: &'a S,
    list: Vec<ListItem>,
    cursor: usize,
    rows: usize,
    last_printed_rows: u16,
}

impl<'a, S> FixedRowHandler<'a, S> {
    pub fn new(style: &'a S, list: &[impl Display], rows: usize) -> Self {
        let mut list: Vec<_> = list
            .iter()
            .map(|item| ListItem {
                item: item.to_string(),
                is_cursor: false,
                is_selected: false,
            })
            .collect();
        if let Some(first) = list.get_mut(0) {
            first.is_cursor = true;
        }
        Self {
            style,
            list,
            cursor: 0,
            rows,
            last_printed_rows: 0,
        }
    }

    pub fn from_list_handler(list_handler: ListHandler<'a, S>, rows: usize) -> Self {
        Self {
            style: list_handler.style,
            list: list_handler.list,
            cursor: list_handler.cursor,
            rows,
            last_printed_rows: list_handler.last_printed_rows,
        }
    }
}

impl<'a, S> SelectHandler for FixedRowHandler<'a, S>
where
    S: Styler<ListItem>,
{
    /// Returns a list of the index of the selected item and the value.
    type Result = Vec<(usize, String)>;

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

        let count = std::cmp::min(self.list.len(), self.rows);

        let iter = self.list.iter().cycle().skip(start).take(count);
        for item in iter {
            self.style.style(f, item)?;
            writeln!(f)?;
            printed_rows += 1;
        }

        self.last_printed_rows = printed_rows;
        Ok(())
    }

    fn clear(&mut self, f: &mut impl Write) -> Result<()> {
        if self.last_printed_rows > 0 {
            queue!(
                f,
                MoveToPreviousLine(self.last_printed_rows),
                Clear(ClearType::FromCursorDown),
            )?;
            self.last_printed_rows = 0;
        }
        Ok(())
    }

    fn on_key(&mut self, key_event: KeyEvent) -> bool {
        match key_event.code {
            KeyCode::Up => {
                if self.rows > self.list.len() {
                    if self.cursor > 0 {
                        self.list[self.cursor].is_cursor = false;
                        self.cursor -= 1;
                        self.list[self.cursor].is_cursor = true;
                        true
                    } else {
                        false
                    }
                } else {
                    self.list[self.cursor].is_cursor = false;
                    self.cursor += self.list.len();
                    self.cursor -= 1;
                    self.cursor %= self.list.len();
                    self.list[self.cursor].is_cursor = true;
                    true
                }
            }
            KeyCode::Down => {
                if self.rows > self.list.len() {
                    if self.cursor + 1 < self.list.len() {
                        self.list[self.cursor].is_cursor = false;
                        self.cursor += 1;
                        self.list[self.cursor].is_cursor = true;
                        true
                    } else {
                        false
                    }
                } else {
                    self.list[self.cursor].is_cursor = false;
                    self.cursor += 1;
                    self.cursor %= self.list.len();
                    self.list[self.cursor].is_cursor = true;
                    true
                }
            }
            _ => false,
        }
    }

    fn toggle(&mut self) {
        if !self.list.is_empty() {
            let is_selected = &mut self.list[self.cursor].is_selected;
            *is_selected = !*is_selected;
        }
    }

    fn get_result(self) -> Self::Result {
        self.list
            .into_iter()
            .enumerate()
            .filter_map(|(i, item)| item.is_selected.then(|| (i, item.item)))
            .collect()
    }
}
