mod valid;

use crate::Error;
use rusqlite::{Connection, params};
use std::path::Path;

#[macro_export]
macro_rules! row_to_self {
    ($row:expr, $($field:ident),+ $(,)?) => {
        Ok(Self {
            $(
                $field: $row.get(stringify!($field))?,
            )+
            ..Default::default()
        })
    };
}

pub use valid::ValidEntry;

pub trait Model {
    fn schema() -> &'static str;
    fn row_to_self(row: &rusqlite::Row<'_>) -> rusqlite::Result<Self>
    where
        Self: Sized;
}
