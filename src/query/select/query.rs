use std::{
    io::Write,
    sync::{mpsc::sync_channel, Arc},
    thread,
    time::{Duration, Instant},
    writeln,
};

use crossterm::{
    cursor::{self, Hide, MoveToColumn, MoveToPreviousLine},
    event::{self, Event, KeyCode},
    queue,
    style::Print,
    terminal::{disable_raw_mode, enable_raw_mode, Clear, ClearType},
};
use cursor::Show;

use crate::{
    item::{BeginInput, EndInput, ListItem, Prompt},
    query::Query,
    style::Styler,
    Result,
};

use super::{FixedRowHandler, ListHandler, SelectHandler};

pub struct SelectQuery<'a, S, H> {
    prompt: Prompt,
    style: &'a S,
    handler: H,
    is_many: bool,
}

impl<'a, S, H> SelectQuery<'a, S, H> {
    pub fn new(prompt: Prompt, style: &'a S, handler: H) -> Self {
        Self {
            prompt,
            style,
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
}

impl<'a, S> SelectQuery<'a, S, ListHandler<'a, S>> {
    pub fn fix_rows(self, rows: usize) -> SelectQuery<'a, S, FixedRowHandler<'a, S>> {
        assert!(rows > 0);
        SelectQuery {
            prompt: self.prompt,
            style: self.style,
            handler: FixedRowHandler::from_list_handler(self.handler, rows),
            is_many: self.is_many,
        }
    }
}

impl<'a, S, H> Query for SelectQuery<'a, S, H>
where
    S: Styler<Prompt> + Styler<BeginInput> + Styler<EndInput> + Styler<ListItem>,
    H: SelectHandler<Result = Vec<(usize, String)>>,
{
    type Result = Vec<(usize, String)>;

    fn show_on(self, f: &mut impl Write) -> Result<Self::Result> {
        let Self {
            prompt,
            style,
            mut handler,
            is_many,
        } = self;

        queue!(f, Hide)?;

        style.style(f, &prompt)?;
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
                        break handler.get_result();
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
            queue!(f, MoveToPreviousLine(1))?;
            style.style(f, &prompt)?;
            style.style(f, &BeginInput)?;
            queue!(f, Print(&result[0].1))?;
            style.style(f, &EndInput)?;
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

impl<'a, S, ListGen, HandlerGen> DynamicSelectQuery<'a, S, ListGen, HandlerGen> {
    pub fn fix_rows<'b, T>(
        self,
        rows: usize,
    ) -> DynamicSelectQuery<'a, S, ListGen, Box<dyn FnMut(&[T]) -> FixedRowHandler<'a, S> + 'b>>
    where
        HandlerGen: FnMut(&[T]) -> ListHandler<'a, S> + 'b,
    {
        let mut handler_gen = self.handler_gen;
        DynamicSelectQuery {
            prompt: self.prompt,
            style: self.style,
            list_gen: self.list_gen,
            handler_gen: Box::new(move |list| {
                FixedRowHandler::from_list_handler(handler_gen(list), rows)
            }),
        }
    }
}

impl<'a, S, T, H, ListGen, HandlerGen> Query for DynamicSelectQuery<'a, S, ListGen, HandlerGen>
where
    S: Styler<Prompt> + Styler<BeginInput> + Styler<EndInput>,
    H: SelectHandler<Result = Vec<(usize, String)>> + 'a,
    T: Send + 'static,
    ListGen: (Fn(String) -> Vec<T>) + Send + Sync + 'static,
    HandlerGen: FnMut(&[T]) -> H + 'a,
{
    type Result = Option<String>;

    fn show_on(self, f: &mut impl Write) -> Result<Self::Result> {
        let Self {
            prompt,
            style,
            list_gen,
            mut handler_gen,
        } = self;
        let list_gen = Arc::new(list_gen);

        queue!(f, Hide)?;

        style.style(f, &prompt)?;
        style.style(f, &BeginInput)?;
        style.style(f, &EndInput)?;
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
        let mut handler: Option<H> = None;

        const POLL_DURATION: Duration = Duration::from_millis(10);
        const DEBOUNCE_DURATION: Duration = Duration::from_secs(1);
        let mut debounce_until = Some(Instant::now());
        let mut result = loop {
            if let Ok(new_list) = rx.try_recv() {
                disable_raw_mode()?;
                if let Some(mut handler) = handler {
                    handler.clear(f)?;
                }
                queue!(f, MoveToPreviousLine(1), Clear(ClearType::CurrentLine))?;
                style.style(f, &prompt)?;
                style.style(f, &BeginInput)?;
                queue!(f, Print(&input))?;
                style.style(f, &EndInput)?;
                writeln!(f)?;
                let mut tmp_handler = handler_gen(&new_list);
                tmp_handler.show(f)?;
                handler = Some(tmp_handler);
                enable_raw_mode()?;
            }
            if matches!(debounce_until, Some(until) if until < Instant::now()) {
                debounce_until = None;
                disable_raw_mode()?;
                if let Some(mut handler) = handler {
                    handler.clear(f)?;
                }
                spawn_list_gen(input.clone());
                handler = None;
                queue!(f, MoveToPreviousLine(1), Clear(ClearType::CurrentLine))?;
                style.style(f, &prompt)?;
                style.style(f, &BeginInput)?;
                queue!(f, Print(&input))?;
                style.style(f, &EndInput)?;
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
                                break handler.get_result();
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
                        queue!(f, MoveToPreviousLine(1), Clear(ClearType::CurrentLine))?;
                        style.style(f, &prompt)?;
                        style.style(f, &BeginInput)?;
                        queue!(f, Print(&input))?;
                        style.style(f, &EndInput)?;
                        writeln!(f)?;
                        if let Some(handler) = &mut handler {
                            handler.show(f)?;
                        }
                        enable_raw_mode()?;
                    }
                }
            }
        };
        disable_raw_mode()?;
        assert!(result.len() <= 1);

        queue!(f, MoveToPreviousLine(1), Clear(ClearType::CurrentLine))?;
        style.style(f, &prompt)?;
        style.style(f, &BeginInput)?;
        let result = (!result.is_empty()).then(|| result.remove(0).1);
        if let Some(item) = &result {
            queue!(f, Print(&item))?;
        }
        style.style(f, &EndInput)?;
        writeln!(f)?;

        queue!(f, Show)?;

        Ok(result)
    }
}
