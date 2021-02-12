use std::{
    io::{stderr, stdin, Write},
    marker::PhantomData,
    sync::{mpsc::sync_channel, Arc},
    thread,
    time::{Duration, Instant},
    writeln,
};

use crossterm::{
    cursor::{self, Hide, MoveToColumn, MoveToPreviousLine, Show},
    event::{Event, KeyCode},
    queue,
    style::Print,
    terminal::{disable_raw_mode, enable_raw_mode, Clear, ClearType},
};
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

impl<S: Style, I: Interactive<S, Result = R>, R, T: AsRef<str>> Interactive<S>
    for Prompted<S, I, T>
{
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
            prompt,
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

impl<S: Style> Default for Input<S> {
    fn default() -> Self {
        Self::new()
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
    height: Option<u16>,
    _style: PhantomData<S>,
}

impl<'a, S: Style, T: AsRef<str>> Select<'a, S, T> {
    pub fn new(choices: &'a [T]) -> Self {
        Self {
            choices,
            height: None,
            _style: Default::default(),
        }
    }

    pub fn with_height(self, height: u16) -> Self {
        Self {
            height: Some(height),
            ..self
        }
    }

    fn is_sized(&self) -> bool {
        self.height.is_some()
    }

    fn print_choices(&self, f: &mut impl Write, cursor: usize) -> crossterm::Result<u16> {
        if let Some(height) = self.height {
            print_choices_sized(PhantomData::<S>::default(), f, height, self.choices, cursor)
        } else {
            print_choices_unsized(PhantomData::<S>::default(), f, self.choices, cursor)
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
        let mut prev_printed = self.print_choices(f, cursor)?;
        queue!(f, Hide)?;
        enable_raw_mode()?;
        loop {
            if let Event::Key(event) = crossterm::event::read()? {
                let mut re_render = false;
                match event.code {
                    KeyCode::Down => {
                        if self.is_sized() {
                            cursor += 1;
                            cursor %= self.choices.len();
                            re_render = true;
                        } else if cursor + 1 < self.choices.len() {
                            cursor += 1;
                            re_render = true;
                        }
                    }
                    KeyCode::Up => {
                        if self.is_sized() {
                            cursor += self.choices.len() - 1;
                            cursor %= self.choices.len();
                            re_render = true;
                        } else if cursor > 0 {
                            cursor -= 1;
                            re_render = true;
                        }
                    }
                    KeyCode::Enter => {
                        break;
                    }
                    _ => {}
                }
                if re_render {
                    disable_raw_mode()?;
                    queue!(
                        f,
                        MoveToPreviousLine(prev_printed),
                        Clear(ClearType::FromCursorDown)
                    )?;
                    prev_printed = self.print_choices(f, cursor)?;
                    f.flush()?;
                    enable_raw_mode()?;
                }
            }
        }
        if prev_printed > 0 {
            queue!(f, MoveToPreviousLine(prev_printed))?;
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

pub struct DynamicSelect<
    S: Style,
    T: AsRef<str> + Send + 'static,
    F: (Fn(String) -> Vec<T>) + Send + Sync + 'static,
> {
    placeholder: Option<String>,
    waiting: Option<String>,
    height: Option<u16>,
    generator: F,
    _style: PhantomData<S>,
}

impl<
        S: Style,
        T: AsRef<str> + Send + 'static,
        F: (Fn(String) -> Vec<T>) + Send + Sync + 'static,
    > DynamicSelect<S, T, F>
{
    pub fn new(generator: F) -> Self {
        Self {
            placeholder: None,
            waiting: None,
            height: None,
            generator,
            _style: Default::default(),
        }
    }

    pub fn with_wait_message(self, waiting: impl AsRef<str>) -> Self {
        Self {
            waiting: Some(waiting.as_ref().to_string()),
            ..self
        }
    }

    pub fn with_placeholder(self, placeholder: impl AsRef<str>) -> Self {
        Self {
            placeholder: Some(placeholder.as_ref().to_string()),
            ..self
        }
    }

    pub fn with_height(self, height: u16) -> Self {
        Self {
            height: Some(height),
            ..self
        }
    }

    fn print_choices(
        f: &mut impl Write,
        input: &str,
        placeholder: Option<&str>,
        height: Option<u16>,
        choices: &[T],
        cursor: usize,
    ) -> crossterm::Result<u16> {
        let mut lines = 0;
        S::print_indicator(f)?;
        if input.is_empty() {
            if let Some(placeholder) = placeholder {
                S::print_placeholder(f, placeholder)?;
            }
        } else {
            queue!(f, Print(input))?;
        }
        queue!(f, Print('\n'))?;
        lines += 1;
        lines += if let Some(height) = height {
            print_choices_sized(PhantomData::<S>::default(), f, height, choices, cursor)
        } else {
            print_choices_unsized(PhantomData::<S>::default(), f, choices, cursor)
        }?;
        Ok(lines)
    }
}

impl<
        S: Style,
        T: AsRef<str> + Send + 'static,
        F: (Fn(String) -> Vec<T>) + Send + Sync + 'static,
    > Interactive<S> for DynamicSelect<S, T, F>
{
    type Result = Option<T>;

    fn interact_on(self, f: &mut impl Write) -> crossterm::Result<Self::Result> {
        const POLL_DURATION: Duration = Duration::from_millis(10);
        const DEBOUNCE_DURATION: Duration = Duration::from_secs(1);
        S::print_indicator(f)?;
        let Self {
            waiting,
            placeholder,
            height,
            generator,
            ..
        } = self;
        let generator = Arc::new(generator);
        let (choice_tx, choice_rx) = sync_channel(1);
        let new_tx = choice_tx.clone();
        let new_gen = generator.clone();
        thread::spawn(move || {
            let choices = new_gen(String::new());
            new_tx.send(choices).unwrap();
        });
        let (result_x, _) = cursor::position()?;
        let mut cursor = 0usize;
        let mut choices = vec![];
        let mut input = String::new();
        let mut is_waiting = false;
        writeln!(f)?;
        let mut prev_printed =
            Self::print_choices(f, &input, placeholder.as_deref(), height, &choices, cursor)?;
        let mut debounce_until = None;
        enable_raw_mode()?;
        loop {
            if let Ok(new_choices) = choice_rx.try_recv() {
                choices = new_choices;
                is_waiting = false;
                disable_raw_mode()?;
                cursor = 0;
                queue!(
                    f,
                    MoveToPreviousLine(prev_printed),
                    Clear(ClearType::FromCursorDown)
                )?;
                prev_printed = Self::print_choices(
                    f,
                    &input,
                    placeholder.as_deref(),
                    height,
                    &choices,
                    cursor,
                )?;
                f.flush()?;
                enable_raw_mode()?;
            }
            if matches!(debounce_until, Some(until) if until < Instant::now()) {
                let new_tx = choice_tx.clone();
                let input = input.clone();
                let generator = generator.clone();
                thread::spawn(move || {
                    let result = generator(input);
                    new_tx.send(result).unwrap();
                });
                debounce_until = None;
                is_waiting = true;
            }
            if crossterm::event::poll(POLL_DURATION)? {
                if let Event::Key(event) = crossterm::event::read()? {
                    let mut re_render = false;
                    match event.code {
                        KeyCode::Char(c) => {
                            input.push(c);
                            debounce_until = Some(Instant::now() + DEBOUNCE_DURATION);
                            //is_waiting = true;
                            re_render = true;
                        }
                        KeyCode::Backspace => {
                            if input.pop().is_some() {
                                debounce_until = Some(Instant::now() + DEBOUNCE_DURATION);
                                //is_waiting = true;
                                re_render = true;
                            }
                        }
                        KeyCode::Down => {
                            if height.is_some() {
                                cursor += 1;
                                cursor %= choices.len();
                                re_render = true;
                            } else if cursor + 1 < choices.len() {
                                cursor += 1;
                                re_render = true;
                            }
                        }
                        KeyCode::Up => {
                            if height.is_some() {
                                cursor += choices.len() - 1;
                                cursor %= choices.len();
                                re_render = true;
                            } else if cursor > 0 {
                                cursor -= 1;
                                re_render = true;
                            }
                        }
                        KeyCode::Enter => {
                            break;
                        }
                        _ => {}
                    }
                    if re_render {
                        disable_raw_mode()?;
                        queue!(
                            f,
                            MoveToPreviousLine(prev_printed),
                            Clear(ClearType::FromCursorDown)
                        )?;
                        let choices = if is_waiting { [].as_ref() } else { &choices };
                        prev_printed = Self::print_choices(
                            f,
                            &input,
                            placeholder.as_deref(),
                            height,
                            &choices,
                            cursor,
                        )?;
                        match waiting {
                            Some(ref waiting) if is_waiting => {
                                writeln!(f, "{}", &waiting)?;
                                prev_printed += 1;
                            }
                            _ => {
                                f.flush()?;
                            }
                        }
                        enable_raw_mode()?;
                    }
                }
            }
        }
        disable_raw_mode()?;
        queue!(
            f,
            MoveToPreviousLine(prev_printed),
            Clear(ClearType::FromCursorDown)
        )?;
        if cursor < choices.len() {
            let choice = choices.remove(cursor);
            queue!(
                f,
                MoveToPreviousLine(1),
                MoveToColumn(result_x + 1),
                Print(choice.as_ref()),
            )?;
            writeln!(f)?;
            Ok(Some(choice))
        } else {
            Ok(None)
        }
    }
}

fn print_choices_sized<S: Style, T: AsRef<str>>(
    _style: PhantomData<S>,
    f: &mut impl Write,
    height: u16,
    choices: &[T],
    cursor: usize,
) -> crossterm::Result<u16> {
    let mut lines = 0;

    let len = choices.len();
    let from = if len < height as usize {
        0
    } else if height >= 5 {
        (cursor + len - 2) % len
    } else if height >= 3 {
        (cursor + len - 1) % len
    } else {
        cursor
    };

    if len > 0 {
        for i in from..from + height as usize {
            let i = i % len;
            let is_current = i == cursor;
            S::print_list_item(f, choices[i].as_ref(), is_current)?;
            queue!(f, Print('\n'))?;
            lines += 1;
        }
    }

    Ok(lines)
}

fn print_choices_unsized<S: Style, T: AsRef<str>>(
    _style: PhantomData<S>,
    f: &mut impl Write,
    choices: &[T],
    cursor: usize,
) -> crossterm::Result<u16> {
    let mut lines = 0;

    for (i, choice) in choices.iter().enumerate() {
        let is_current = i == cursor;
        S::print_list_item(f, choice.as_ref(), is_current)?;
        queue!(f, Print('\n'))?;
        lines += 1;
    }

    Ok(lines)
}
