use std::io::Write;

use crossterm::{
    execute, queue,
    style::{Color, Colorize, PrintStyledContent, ResetColor, SetForegroundColor, Styler},
    Result,
};

pub trait Style {
    fn queue_prompt(f: &mut impl Write, prompt: String) -> Result<()>;

    fn queue_indicator(f: &mut impl Write) -> Result<()>;

    fn format_input(f: &mut impl Write) -> Result<()>;

    fn unformat_input(f: &mut impl Write) -> Result<()> {
        execute!(f, ResetColor)
    }
}

pub struct DefaultStyle;

impl Style for DefaultStyle {
    fn queue_prompt(f: &mut impl Write, mut prompt: String) -> Result<()> {
        prompt.push(' ');
        queue!(
            f,
            PrintStyledContent("? ".green()),
            PrintStyledContent(prompt.bold()),
        )
    }

    fn queue_indicator(f: &mut impl Write) -> Result<()> {
        queue!(f, PrintStyledContent("> ".dark_grey()),)
    }

    fn format_input(f: &mut impl Write) -> Result<()> {
        execute!(f, SetForegroundColor(Color::Blue),)
    }
}
