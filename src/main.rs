extern crate cursive;
extern crate reqwest;
extern crate serde_json;
extern crate url;
// extern crate cursive_table_view;

#[macro_use]
extern crate log;
extern crate env_logger;
use cursive::traits::*;
use cursive::Cursive;
use cursive::views::{
    Dialog, DummyView, EditView, LinearLayout, OnEventView, SelectView, TextView,
};
// STD Dependencies -----------------------------------------------------------
// ----------------------------------------------------------------------------
pub mod theme;
use theme::*;
use reqwest::Url;
use serde_json::Value;
use cursive::align::HAlign;
use std::cmp::Ordering;
use rand::Rng;
// Modules --------------------------------------------------------------------
// ----------------------------------------------------------------------------
// use cursive_table_view::{TableView, TableViewItem};
// pub mod table_maker;
// use table::*;

mod table_view;
use table_view::{TableView, TableViewItem};


fn main() {

    env_logger::init();
    let mut rng = rand::thread_rng();
    // Initial setup
    let mut main = Cursive::default();

    let mut table = TableView::<Foo, BasicColumn>::new()
        .column(BasicColumn::Name, "Name", |c| c.width_percent(20))
        .column(BasicColumn::Count, "Count", |c| c.align(HAlign::Center))
        .column(BasicColumn::Rate, "Rate", |c| {
            c.ordering(Ordering::Greater)
                .align(HAlign::Right)
                .width_percent(20)
        });

    let mut items = Vec::new();
    for i in 0..50 {
        items.push(Foo {
            name: format!("Name {}", i),
            count: rng.gen_range(0, 255),
            rate: rng.gen_range(0, 255),
        });
    }

    table.set_items(items);

    table.set_on_sort(|siv: &mut Cursive, column: BasicColumn, order: Ordering| {
        siv.add_layer(
            Dialog::around(TextView::new(format!("{} / {:?}", column.as_str(), order)))
                .title("Sorted by")
                .button("Close", |s| {
                    s.pop_layer();
                }),
        );
    });

    table.set_on_submit(|siv: &mut Cursive, row: usize, index: usize| {
        let value = siv
            .call_on_id("table", move |table: &mut TableView<Foo, BasicColumn>| {
                format!("{:?}", table.borrow_item(index).unwrap())
            })
            .unwrap();

        siv.add_layer(
            Dialog::around(TextView::new(value))
                .title(format!("Removing row # {}", row))
                .button("Close", move |s| {
                    s.call_on_id("table", |table: &mut TableView<Foo, BasicColumn>| {
                        table.remove_item(index);
                    });
                    s.pop_layer();
                }),
        );
    });

    main.add_layer(Dialog::around(table.with_id("table").min_size((50, 20))).title("Table View"));


    // Set theme
    main.set_theme(theme_gen());

    main.add_global_callback('q', |s| s.quit());
    main.add_global_callback('s', |s| search(s));

//     main.add_layer(TextView::new(
// "    WELCOME
// Hit s to search
// Hit q to quit
// Hit t to pop layer",
//     ));

    main.run();
}


fn search(s: &mut Cursive) {
    s.add_layer(
        Dialog::text("something goes here").title("simeth")
        .button("OK", |s| on_submit_fn(s, "Frameworks"))
    )
}

fn on_submit_fn(s: &mut Cursive, name: &str) {
    let url = query_url_gen(name);
    // let mut extract = String::new();
    let mut link_vec: Vec<String> = vec![];

    let mut res = reqwest::get(url).unwrap();
    info!("URL is [{:?}]", res);
    let v: Value = res.json().expect("Failed to parse json");

    match get_links(&v) {
        Ok(x) => link_vec = x,
        Err(e) => pop_error(s, &handler(&e)),
    };

    let links = SelectView::<String>::new()
        .with_all_str(link_vec)
        .on_submit(on_submit_fn)
        .scrollable();
        // .fixed_width(20);


    let header = LinearLayout::horizontal().child(TextView::new("Framework"));

    s.add_layer(
        Dialog::around(
            OnEventView::new(
                LinearLayout::vertical()
                    .child(header.fixed_width(72))
                    .child(DummyView)
                    .child(links),
            ).on_event('t', |s| match s.pop_layer() {
                _ => (),
            }),
        ).title(name),
    );
}

pub fn handler(e: &reqwest::Error) -> String {
    let mut msg: String = String::new();
    if e.is_http() {
        match e.url() {
            None => msg.push_str(&"No URL given"),
            Some(url) => msg.push_str(&format!("Problem making request to: {}", url)),
        }
    }

    if e.is_redirect() {
        msg.push_str(&"server redirecting too many times or making loop");
    }

    msg
}

pub fn pop_error(s: &mut Cursive, msg: &str) {
    s.add_layer(Dialog::text(msg.to_string()).button("Ok", |s| s.quit()));
}

pub fn get_links(v: &Value) -> Result<Vec<String>, reqwest::Error> {
    let mut links = vec![];
    match &v["frameworks"] {
        Value::Array(arr) => {
            for item in arr {
                match item["name"] {
                    Value::String(ref name) => links.push(name.to_string()),
                    _ => links.push(String::from("lol")),
                }
            }
        }
        _ => links.push(String::from("lol")),
    };

    Ok(links)
}

pub fn query_url_gen(title: &str) -> Url {
    let url = Url::parse("http://odhecx54:5040/master/frameworks").unwrap();
    return url;
}



// ################### TABLE #########################
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

