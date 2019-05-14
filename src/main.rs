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

    let mut table = TableView::<Framework, BasicColumn>::new()
        .column(BasicColumn::Framework, "Frameworks", |c| c.width_percent(60))
        .column(BasicColumn::Mem, "Mem", |c| c.align(HAlign::Center))
        .column(BasicColumn::CPUs, "CPUs", |c| {
            c.ordering(Ordering::Greater)
                .align(HAlign::Right)
                .width_percent(20)
        });

    let url = query_url_gen("Frameworks");
    let mut link_vec: Vec<String> = vec![];

    let mut res = reqwest::get(url).unwrap();
    info!("URL is [{:?}]", res);
    let v: Value = res.json().expect("Failed to parse json");

    match get_links(&v) {
        Ok(x) => link_vec = x,
        Err(e) => pop_error(&mut main, &handler(&e)),
    };


    let mut items = Vec::new();
    for item in link_vec {
        items.push(Framework {
            framework: format!("{}", item),
            mem: rng.gen_range(0, 255),
            cpus: rng.gen_range(0, 255),
            uptime: rng.gen_range(0, 255),
            upsince: rng.gen_range(0, 255),
            tasks: rng.gen_range(0, 255),
            tasksmap: rng.gen_range(0, 255),
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
            .call_on_id("table", move |table: &mut TableView<Framework, BasicColumn>| {
                format!("{:?}", table.borrow_item(index).unwrap())
            })
            .unwrap();

        siv.add_layer(
            Dialog::around(TextView::new(value))
                .title(format!("Removing row # {}", row))
                .button("Close", move |s| {
                    s.call_on_id("table", |table: &mut TableView<Framework, BasicColumn>| {
                        table.remove_item(index);
                    });
                    s.pop_layer();
                }),
        );
    });

    main.add_layer(Dialog::around(table.with_id("table").min_size((500, 200))).title("Fuzzy-Chainsaw"));


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
    Framework,
    Mem,
    CPUs,
    UpTime,
    UpSince,
    Tasks,
    TaksMap,
}

impl BasicColumn {
    fn as_str(&self) -> &str {
        match *self {
            BasicColumn::Framework => "Frameworks",
            BasicColumn::Mem => "Mem",
            BasicColumn::CPUs => "CPUs",
            BasicColumn::UpTime => "UpTime",
            BasicColumn::UpSince => "Up Since",
            BasicColumn::Tasks => "Tasks",
            BasicColumn::TaksMap => "Tasks Map",
        }
    }
}

#[derive(Clone, Debug)]
pub struct Framework {
    framework: String,
    mem: usize,
    cpus: usize,
    uptime: usize,
    upsince: usize,
    tasks: usize,
    tasksmap: usize,
}

impl TableViewItem<BasicColumn> for Framework {
    fn to_column(&self, column: BasicColumn) -> String {
        match column {
            BasicColumn::Framework => self.framework.to_string(),
            BasicColumn::Mem => format!("{}", self.mem),
            BasicColumn::CPUs => format!("{}", self.cpus),
            BasicColumn::UpTime => format!("{}", self.uptime),
            BasicColumn::UpSince => format!("{}", self.upsince),
            BasicColumn::Tasks => format!("{}", self.tasks),
            BasicColumn::TaksMap => format!("{}", self.tasksmap),
        }
    }

    fn cmp(&self, other: &Self, column: BasicColumn) -> Ordering
    where
        Self: Sized,
    {
        match column {
            BasicColumn::Framework => self.framework.cmp(&other.framework),
            BasicColumn::Mem => self.mem.cmp(&other.mem),
            BasicColumn::CPUs => self.cpus.cmp(&other.cpus),
            BasicColumn::UpTime => self.cpus.cmp(&other.uptime),
            BasicColumn::UpSince => self.cpus.cmp(&other.upsince),
            BasicColumn::Tasks => self.cpus.cmp(&other.tasks),
            BasicColumn::TaksMap => self.cpus.cmp(&other.tasksmap),
        }
    }
}

