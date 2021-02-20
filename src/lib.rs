pub mod item;
pub mod query;
pub mod style;
pub(crate) mod util;

/// The result type of this crate.
type Result<T> = crossterm::Result<T>;
