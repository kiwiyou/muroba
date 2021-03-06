use std::io::Write;

use crossterm::{
    execute, queue,
    style::{
        Attribute, Color, Colorize, Print, PrintStyledContent, ResetColor, SetAttribute,
        SetForegroundColor, Styler as TermStyler,
    },
};

use crate::{item::*, util, Result};

pub struct DefaultStyle;

pub trait Styler<I> {
    fn style(&self, f: &mut impl Write, item: &I) -> Result<()>;
}

impl Styler<Prompt> for DefaultStyle {
    fn style(&self, f: &mut impl Write, Prompt(prompt): &Prompt) -> Result<()> {
        queue!(
            f,
            PrintStyledContent("?".green()),
            Print(' '),
            PrintStyledContent(prompt.as_str().bold()),
            ResetColor,
        )
    }
}

impl Styler<BeginInput> for DefaultStyle {
    fn style(&self, f: &mut impl Write, _: &BeginInput) -> Result<()> {
        execute!(
            f,
            PrintStyledContent(" > ".dark_grey()),
            SetForegroundColor(Color::Blue),
        )
    }
}

impl Styler<EndInput> for DefaultStyle {
    fn style(&self, f: &mut impl Write, _: &EndInput) -> Result<()> {
        queue!(f, ResetColor,)
    }
}

impl Styler<ConfirmChoice> for DefaultStyle {
    fn style(&self, f: &mut impl Write, ConfirmChoice(default): &ConfirmChoice) -> Result<()> {
        match default {
            None => {
                queue!(f, Print(" [y/n]"))
            }
            Some(true) => {
                queue!(
                    f,
                    Print(" ["),
                    PrintStyledContent("Y".underlined()),
                    ResetColor,
                    Print("/n]")
                )
            }
            Some(false) => {
                queue!(
                    f,
                    Print(" [y/"),
                    PrintStyledContent("N".underlined()),
                    ResetColor,
                    Print("]")
                )
            }
        }
    }
}

impl Styler<ListItem> for DefaultStyle {
    fn style(&self, f: &mut impl Write, list_item: &ListItem) -> Result<()> {
        if list_item.is_cursor {
            queue!(f, SetForegroundColor(Color::Blue), Print("> "),)?;
            if list_item.is_selected {
                queue!(f, SetAttribute(Attribute::Bold))?;
            }
        } else if list_item.is_selected {
            queue!(f, SetForegroundColor(Color::Green), Print("✓ "),)?;
        } else {
            queue!(f, Print("  "),)?;
        }

        util::trim_print(self, f, &list_item.item)?;
        queue!(f, ResetColor, SetAttribute(Attribute::Reset))?;

        Ok(())
    }
}

impl Styler<WaitMessage> for DefaultStyle {
    fn style(&self, f: &mut impl Write, WaitMessage(message): &WaitMessage) -> Result<()> {
        queue!(
            f,
            PrintStyledContent(message.as_str().dark_grey().italic()),
            ResetColor,
            SetAttribute(Attribute::Reset)
        )
    }
}

impl Styler<Overflow> for DefaultStyle {
    fn style(&self, f: &mut impl Write, _: &Overflow) -> Result<()> {
        queue!(f, Print("…"))
    }
}
