//! CSS-like selector engine: `parse` a selector string, then `select` matches
//! from a `DocModel`.

pub mod matcher;
pub mod parse;

pub use matcher::{select, select_in, Match};
pub use parse::{
    parse, AttrKey, Combinator, Compound, Element, Filter, ParseError, Pattern, Selector, Test,
};
