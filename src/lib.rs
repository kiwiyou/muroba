use std::{
    io::{stderr, stdin, Write},
    marker::PhantomData,
};

use style::{DefaultStyle, Style};

pub mod style;

pub trait Interactive<S: Style>: Sized {
    type Result;
    fn interact(self) -> crossterm::Result<Self::Result> {
        self.interact_on(&mut stderr())
    }
    fn interact_on(self, f: &mut impl Write) -> crossterm::Result<Self::Result>;
}

pub struct Prompted<S: Style, I: Interactive<S>> {
    inner: I,
    prompt: String,
    _style: PhantomData<S>,
}

impl<S: Style, I: Interactive<S, Result = R>, R> Interactive<S> for Prompted<S, I> {
    type Result = R;

    fn interact_on(self, f: &mut impl Write) -> crossterm::Result<Self::Result> {
        S::queue_prompt(f, self.prompt)?;
        self.inner.interact_on(f)
    }
}

pub trait Promptable<S: Style, I: Interactive<S>> {
    fn with_prompt(self, prompt: impl Into<String>) -> Prompted<S, I>;
}

impl<S: Style, I: Interactive<S>> Promptable<S, Self> for I {
    fn with_prompt(self, prompt: impl Into<String>) -> Prompted<S, Self> {
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

impl Default for Input<DefaultStyle> {
    fn default() -> Self {
        Self {
            _style: Default::default(),
        }
    }
}

impl<S: Style> Interactive<S> for Input<S> {
    type Result = String;

    fn interact_on(self, f: &mut impl Write) -> crossterm::Result<Self::Result> {
        S::format_input(f)?;
        let mut input = String::new();
        let stdin = stdin();
        stdin.read_line(&mut input)?;
        S::unformat_input(f)?;
        input.truncate(input.trim_end().len());
        Ok(input)
    }
}
