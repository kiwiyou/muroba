use std::io::Write;

use crossterm::{
    queue,
    style::{Color, Colorize, Print, PrintStyledContent, ResetColor, SetForegroundColor, Styler},
    Result,
};

use crate::{DynamicSelect, Input, Select};

pub trait Style: Sized {
    fn print_prompt(f: &mut impl Write, prompt: &str) -> Result<()>;

    fn print_indicator(f: &mut impl Write) -> Result<()>;

    fn format_input(f: &mut impl Write) -> Result<()>;

    fn unformat_input(f: &mut impl Write) -> Result<()> {
        queue!(f, ResetColor)
    }

    fn print_list_item(f: &mut impl Write, item: &str, current: bool) -> Result<()>;

    fn print_placeholder(f: &mut impl Write, placeholder: &str) -> Result<()>;

    fn input() -> Input<Self> {
        Input::new()
    }

    fn select<T: AsRef<str>>(choices: &[T]) -> Select<Self, T> {
        Select::new(choices)
    }

    fn dynamic_select<
        T: AsRef<str> + Send + 'static,
        F: (Fn(String) -> Vec<T>) + Send + Sync + 'static,
    >(
        generator: F,
    ) -> DynamicSelect<Self, T, F> {
        DynamicSelect::new(generator)
    }
}

pub struct DefaultStyle;

impl Style for DefaultStyle {
    fn print_prompt(f: &mut impl Write, prompt: &str) -> Result<()> {
        queue!(
            f,
            PrintStyledContent("? ".green()),
            PrintStyledContent(prompt.bold()),
            Print(" "),
            ResetColor
        )
    }

    fn print_indicator(f: &mut impl Write) -> Result<()> {
        queue!(f, PrintStyledContent("> ".dark_grey()), ResetColor)
    }

    fn format_input(f: &mut impl Write) -> Result<()> {
        queue!(f, SetForegroundColor(Color::Blue),)
    }

    fn print_list_item(f: &mut impl Write, item: &str, current: bool) -> Result<()> {
        if current {
            queue!(f, SetForegroundColor(Color::Blue), Print("> "),)?;
        } else {
            queue!(f, Print("  "),)?;
        }
        queue!(f, Print(item), ResetColor,)
    }

    fn print_placeholder(f: &mut impl Write, placeholder: &str) -> Result<()> {
        queue!(f, PrintStyledContent(placeholder.grey()), ResetColor)
    }
}
