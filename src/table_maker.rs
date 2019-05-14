extern crate rand;
use rand::Rng;
use std::cmp::Ordering;
use cursive::align::HAlign;

// pub mod table_view;
// pub use table_view;

// use table_view::TableViewItem;

#[derive(Copy, Clone, PartialEq, Eq, Hash)]
pub enum BasicColumn {
    Name,
    Count,
    Rate,
}

impl BasicColumn {
    fn as_str(&self) -> &str {
        match *self {
            BasicColumn::Name => "Name",
            BasicColumn::Count => "Count",
            BasicColumn::Rate => "Rate",
        }
    }
}

#[derive(Clone, Debug)]
pub struct Foo {
    name: String,
    count: usize,
    rate: usize,
}

impl TableViewItem<BasicColumn> for Foo {
    fn to_column(&self, column: BasicColumn) -> String {
        match column {
            BasicColumn::Name => self.name.to_string(),
            BasicColumn::Count => format!("{}", self.count),
            BasicColumn::Rate => format!("{}", self.rate),
        }
    }

    fn cmp(&self, other: &Self, column: BasicColumn) -> Ordering
    where
        Self: Sized,
    {
        match column {
            BasicColumn::Name => self.name.cmp(&other.name),
            BasicColumn::Count => self.count.cmp(&other.count),
            BasicColumn::Rate => self.rate.cmp(&other.rate),
        }
    }
}

