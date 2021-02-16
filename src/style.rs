use std::io::Write;

use crossterm::{
    execute, queue,
    style::{
        Color, Colorize, Print, PrintStyledContent, ResetColor, SetForegroundColor,
        Styler as TermStyler,
    },
};

use crate::{
    item::{BeginInput, EndInput, Prompt},
    Result,
};

pub struct DefaultStyle;

pub trait Styler<I> {
    fn style(&self, f: &mut impl Write, item: I) -> Result<()>;
}

impl Styler<Prompt> for DefaultStyle {
    fn style(&self, f: &mut impl Write, Prompt(prompt): Prompt) -> Result<()> {
        queue!(
            f,
            PrintStyledContent("?".green()),
            Print(' '),
            PrintStyledContent(prompt.bold()),
            ResetColor,
        )
    }
}

impl Styler<BeginInput> for DefaultStyle {
    fn style(&self, f: &mut impl Write, _: BeginInput) -> Result<()> {
        execute!(
            f,
            PrintStyledContent(" > ".dark_grey()),
            SetForegroundColor(Color::Blue),
        )
    }
}

impl Styler<EndInput> for DefaultStyle {
    fn style(&self, f: &mut impl Write, _: EndInput) -> Result<()> {
        queue!(f, ResetColor,)
    }
}
