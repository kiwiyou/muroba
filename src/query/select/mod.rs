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
    pub fn select<T: Display>(self, list: &'a [T]) -> SelectQuery<'a, S, T, ListHandler<'a, S, T>>
    where
        S: Styler<ListItem<'a, T>>,
    {
        SelectQuery::new(
            Prompt(self.prompt.unwrap_or_default()),
            self.style,
            list,
            ListHandler::new(self.style, list),
        )
    }
}
