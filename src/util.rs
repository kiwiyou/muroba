use std::{fmt::Display, io::Write};

use crossterm::{cursor, queue, style::Print, terminal};
use unicode_width::UnicodeWidthStr;

use crate::{item::Overflow, style::Styler, Result};

pub fn trim_print<S>(style: &S, f: &mut impl Write, content: &str) -> Result<()>
where
    S: Styler<Overflow>,
{
    let mut str = trim_overflow(content)?;
    if str.len() < content.len() {
        style.style(f, &Overflow)?;
        str = trim_overflow(str)?;
    }
    queue!(f, Print(str))
}

fn trim_overflow(content: &str) -> Result<&str> {
    let remaining = remaining()?;
    let mut indices = content.char_indices();
    let mut step_bound = content.chars().count();
    while step_bound > 0 {
        let step = step_bound / 2;
        let mut moved = indices.clone();
        for _ in 0..step {
            moved.next();
        }
        if matches!(moved.next(), Some((i, _)) if content[i..].width_cjk() > remaining) {
            indices = moved;
            step_bound -= step + 1;
        } else {
            step_bound = step;
        }
    }

    Ok(indices.next().map_or("", |(i, _)| &content[i..]))
}

fn remaining() -> Result<usize> {
    let (x, _) = cursor::position()?;
    let (width, _) = terminal::size()?;
    Ok((width - x) as usize)
}
