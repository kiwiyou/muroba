use std::{
    fmt::Display,
    io::Write,
    sync::{
        mpsc::{channel, sync_channel},
        Arc,
    },
    thread,
    time::{Duration, Instant},
    writeln,
};

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
    S: Styler<Prompt> + Styler<BeginInput> + Styler<EndInput> + Styler<ListItem<'a, T>>,
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

pub struct DynamicSelectQuery<'a, S, ListGen, HandlerGen> {
    prompt: Prompt,
    style: &'a S,
    list_gen: ListGen,
    handler_gen: HandlerGen,
}

impl<'a, S, ListGen, HandlerGen> DynamicSelectQuery<'a, S, ListGen, HandlerGen> {
    pub fn new(prompt: Prompt, style: &'a S, list_gen: ListGen, handler_gen: HandlerGen) -> Self {
        Self {
            prompt,
            style,
            list_gen,
            handler_gen,
        }
    }
}

impl<'a, S, T, H, ListGen, HandlerGen> Query for DynamicSelectQuery<'a, S, ListGen, HandlerGen>
where
    S: Styler<Prompt> + Styler<BeginInput> + Styler<EndInput>,
    T: Display + Send + 'static,
    H: SelectHandler<'a, T, Result = Vec<usize>>,
    ListGen: (Fn(String) -> Vec<T>) + Send + Sync + 'static,
    HandlerGen: FnMut(&[T]) -> H,
{
    type Result = usize;

    fn show_on(self, f: &mut impl Write) -> Result<Self::Result> {
        let Self {
            prompt,
            style,
            list_gen,
            mut handler_gen,
        } = self;
        let list_gen = Arc::new(list_gen);

        queue!(f, Hide)?;

        style.style(f, prompt)?;
        let (input_x, _) = cursor::position()?;
        style.style(f, BeginInput)?;
        writeln!(f)?;
        let (tx, rx) = sync_channel::<Vec<T>>(1);
        let spawn_list_gen = |input: String| {
            let tx = tx.clone();
            let list_gen = list_gen.clone();
            thread::spawn(move || {
                let result = list_gen(input);
                tx.send(result).unwrap();
            });
        };

        let mut input = String::new();
        spawn_list_gen(input.clone());

        let mut handler: Option<H> = None;
        let mut list;

        const POLL_DURATION: Duration = Duration::from_millis(100);
        const DEBOUNCE_DURATION: Duration = Duration::from_secs(1);
        let mut debounce_until = None;
        let result = loop {
            if let Ok(new_list) = rx.try_recv() {
                if let Some(mut handler) = handler {
                    handler.clear(f)?;
                }
                list = new_list;
                handler = Some(handler_gen(&list));
            }
            if matches!(debounce_until, Some(until) if until < Instant::now()) {
                disable_raw_mode()?;
                if let Some(mut handler) = handler {
                    handler.clear(f)?;
                }
                spawn_list_gen(input.clone());
                handler = None;
                queue!(f, MoveToPreviousLine(1), MoveToColumn(input_x))?;
                style.style(f, BeginInput)?;
                queue!(f, Print(&input))?;
                style.style(f, EndInput)?;
                writeln!(f)?;
                enable_raw_mode()?;
            }
            if event::poll(POLL_DURATION)? {
                if let Event::Key(event) = event::read()? {
                    let redraw = match event.code {
                        KeyCode::Char(c) => {
                            input.push(c);
                            debounce_until = Some(Instant::now() + DEBOUNCE_DURATION);
                            true
                        }
                        KeyCode::Backspace => {
                            if !input.is_empty() {
                                input.pop();
                                debounce_until = Some(Instant::now() + DEBOUNCE_DURATION);
                                true
                            } else {
                                false
                            }
                        }
                        KeyCode::Enter => {
                            if let Some(mut handler) = handler {
                                handler.toggle();
                                disable_raw_mode()?;
                                handler.clear(f)?;
                                break handler.get_selected();
                            } else {
                                false
                            }
                        }
                        _ => handler.as_mut().map(|h| h.on_key(event)).unwrap_or(false),
                    };
                    if redraw {
                        disable_raw_mode()?;
                        if let Some(handler) = &mut handler {
                            handler.clear(f)?;
                        }
                        queue!(f, MoveToPreviousLine(1), MoveToColumn(input_x))?;
                        style.style(f, BeginInput)?;
                        queue!(f, Print(&input))?;
                        style.style(f, EndInput)?;
                        if let Some(handler) = &mut handler {
                            handler.show(f)?;
                        }
                        writeln!(f)?;
                        enable_raw_mode()?;
                    }
                }
            }
        };
        disable_raw_mode()?;
        assert_eq!(1, result.len());

        queue!(f, MoveToPreviousLine(1), MoveToColumn(input_x))?;
        style.style(f, BeginInput)?;
        queue!(f, Print(&result[0]))?;
        style.style(f, EndInput)?;
        writeln!(f)?;

        queue!(f, Show)?;

        Ok(result[0])
    }
}
