mod handler;
mod query;

use std::fmt::Display;

pub use handler::*;
pub use query::*;

use crate::{item::*, style::Styler};

use super::QueryBuilder;

impl<'a, S> QueryBuilder<'a, S>
where
    S: Styler<Prompt> + Styler<BeginInput> + Styler<EndInput>,
{
    pub fn select<T>(self, list: &'a [T]) -> SelectQuery<'a, S, T, ListHandler<'a, S, T>>
    where
        T: Display,
        S: Styler<ListItem<'a, T>>,
    {
        SelectQuery::new(
            Prompt(self.prompt.unwrap_or_default()),
            self.style,
            list,
            ListHandler::new(self.style, list),
        )
    }

    pub fn dyn_select<T, ListGen>(
        self,
        list_gen: ListGen,
    ) -> DynamicSelectQuery<'a, S, ListGen, Box<dyn FnMut(&'a [T]) -> ListHandler<'a, S, T> + 'a>>
    where
        S: Styler<ListItem<'a, T>>,
        T: Display + Send + 'static,
        ListGen: (Fn(String) -> Vec<T>) + Send + Sync + 'static,
    {
        let style = self.style;
        DynamicSelectQuery::new(
            Prompt(self.prompt.unwrap_or_default()),
            self.style,
            list_gen,
            Box::new(move |list| ListHandler::new(style, list)),
        )
    }
}
