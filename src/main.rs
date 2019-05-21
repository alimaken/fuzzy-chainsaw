extern crate cursive;
extern crate reqwest;
extern crate serde_json;
extern crate url;
extern crate chrono;
extern crate colored; // not needed in Rust 2018
    

    // test the example with `cargo run --example most_simple`

pub mod theme;
mod table_view;

use colored::*;
use chrono::*;
use chrono::prelude::*;
use cursive::traits::*;
use cursive::{Cursive, Printer};
use cursive::views::{
    Dialog, DummyView, EditView, LinearLayout, OnEventView, SelectView, TextView,
};
use theme::*;
use reqwest::Url;
use serde_json::Value;
use cursive::align::HAlign;
use std::cmp::Ordering;
use table_view::{TableView, TableViewItem};

use cursive::theme::{Color, ColorStyle};


use std::collections::HashMap;

macro_rules! hashmap {
    ($( $key: expr => $val: expr ),*) => {{
         let mut map = ::std::collections::HashMap::new();
         $( map.insert($key, $val); )*
         map
    }}
}

fn main() {
    println!("{} {} !", "it".green(), "works".blue().bold());
    let mut main = Cursive::default();

    
    main.add_layer(TextView::new("Loading..."));

    let mut table = TableView::<Framework, BasicColumn>::new()
        .column(BasicColumn::Name, "Framework" , |c| c.width_percent(40))
        .column(BasicColumn::MemStr, "Mem", |c| {
            c.ordering(Ordering::Greater).align(HAlign::Right).width(10)
        })
        .column(BasicColumn::CPUs, "CPUs", |c| {
            c.ordering(Ordering::Greater).align(HAlign::Right).width(5)
        })
        .column(BasicColumn::UpTime, "UpTime", |c| c.align(HAlign::Right).width(19))
        .column(BasicColumn::UpSince, "UpSince", |c| c.align(HAlign::Right).width(10))
        .column(BasicColumn::Tasks, "Tsks", |c| c.align(HAlign::Right).width(4))
        .column(BasicColumn::TaksMap, "TaksMap", |c| c.width(35))
        // .column(BasicColumn::URL, "URL", |c| c.align(HAlign::Right))
        ;

    let url = query_url_gen("Frameworks");
    let mut res = reqwest::get(url).unwrap();

    let v: Value = res.json().expect("Failed to parse json");

    let mut link_vec: Vec<Framework> = vec![];

    match get_links(&v) {
        Ok(x) => link_vec = x,
        Err(e) => pop_error(&mut main, &handler(&e)),
    };

    table.set_items(link_vec.clone());

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
                .title(format!("Details of row # {}", row))
                .button("Close", |s| {s.pop_layer();})
                // .button("Close", move |s| {
                //     s.call_on_id("table", |table: &mut TableView<Framework, BasicColumn>| {
                //         table.remove_item(index);
                //     });
                //     s.pop_layer();
                // }),
        );
    });

    // main.pop_layer();

    main.add_layer(Dialog::around(table.with_id("table").min_size((300, 100))).title("Fuzzy-Chainsaw"));
    // Set theme
    main.set_theme(theme_gen());
    main.add_global_callback('q', |s| s.quit());
    // main.add_global_callback('s', |s| search(s));
    main.run();
}

pub fn get_links(v: &Value) -> Result<Vec<Framework>, reqwest::Error> {
    let mut links: Vec<Framework> = vec![];
    match &v["frameworks"] {
        Value::Array(arr) => {
            for item in arr {

                let mut f = Framework { name: "Default".to_string(), ..Default::default() };
                match item["name"] {
                    Value::String(ref name) => f.name = name.to_string().clone(),
                    _ => (),
                }
                match item["resources"] {
                    Value::Object(ref resources) => {
                        match resources["mem"] {
                            Value::Number(ref mem) => {
                                let memory = mem.as_f64().unwrap_or(0.0) as i32;
                                f.mem = memory;
                                f.mem_str = get_mb_to_iec(memory);
                            },
                            _ => (),
                        };
                    },
                    _ => (),
                }
                match item["resources"] {
                    Value::Object(ref resources) => {
                        match resources["cpus"] {
                            Value::Number(ref cpus) => f.cpus = cpus.as_f64().unwrap_or(0.0) as i32,
                            _ => (),
                        };
                    },
                    _ => (),
                }
                match item["registered_time"] {
                    Value::Number(ref uptime_epoch) => {
                        let timestamp = uptime_epoch.as_f64().unwrap_or(0.0) as i32;
                        f.uptime = get_datetime(timestamp);
                        f.upsince = get_datetime(timestamp);
                    },
                    _ => (),
                }
                match item["tasks"] {
                    Value::Array(ref tasks_array) => {
                        let tasks_num = tasks_array.len() as i32;
                        f.tasks = tasks_num;
                        f.tasksmap = get_map_str(get_map_2(tasks_array));
                    },
                    _ => (),
                }
                match item["webui_url"] {
                    Value::String(ref url) => f.url = url.to_string().clone(),
                    _ => (),
                }

                links.push(f);

            }
        }
        _ => (),
    };

    Ok(links)
}

fn get_map_str(tasks: String) -> String {    
    let mut status_map = hashmap![
        "TASK_STAGING" => 0, 
        "TASK_STARTING" => 0, 
        "TASK_RUNNING" => 0, 
        "TASK_UNREACHABLE" => 0, 
        "TASK_FINISHED" => 0, 
        "TASK_KILLING" => 0, 
        "TASK_KILLED" => 0, 
        "TASK_FAILED" => 0, 
        "TASK_LOST" => 0];
    
    // tasks.trim().split(",").collect::<Vec<&str>>().join("-");
    let mut tasks_exist: bool = false;
    for status in tasks.trim().split(",").collect::<Vec<&str>>() {
        let current_count: &i32 = match status_map.get(status) {
            Some(status_count) => status_count,
            None => &0
        };
        status_map.insert(status, current_count + 1);        
        tasks_exist = true;
    }
    // println!("{:?}", status_map);
    
    let mut result : Vec<String> = vec![];
    
    for (key, val) in status_map.iter() {
        result.push(format!("{}{}", val, get_marker(key).to_string()));
    }
    if !tasks.is_empty() { result.join("|") }
    else { get_marker("None").to_string() }
}

fn get_marker(status: &str) -> &str {
    match status {
            "TASK_STAGING" => "○",
            "TASK_STARTING" => "◒",
            "TASK_RUNNING" => "●",
            "TASK_UNREACHABLE" => "○",
            "TASK_FINISHED" => "◕",
            "TASK_KILLING" => "◐",
            "TASK_KILLED" => "◑",
            "TASK_FAILED" => "●",
            "TASK_LOST" => "◓",
            "None" => "None",
            _ => "X",
    }
}

fn get_map_2(v: &Vec<serde_json::Value>) -> String {
    let mut states: Vec<String> = vec![];
    for task in v {
        match task["state"] {
            Value::String(ref state) => states.push(state.clone()),
            _ => ()
        }
    }
    states.join(",")
}

fn get_map(n: i32, m: i32, c: char) -> String {
    let mut str = String::from("");
    let mut i = 0;
    while i < n {
        if i > 0 && i % m == 0 { str.push('\n'); }
        str.push(c);
        i += 1;
    }
    return str;
}

fn get_mb_to_iec(number: i32) -> String {
    
    if number < 1025 { return format!("{} MB", number); }
    else if number < ((1024 * 1024) + 1) { 
        return format!("{:.2} GB", (number as f32/1024.0)); 
    }
    else if number < ((1024 * 1024 * 1024) + 1) { 
        return format!("{:.2} TB", (number as f32/1024.0)); 
    }
    else {
        return format!("{:.2} ??", number);
    }
}

fn get_datetime(timestamp: i32) -> String {
    
    let naive = NaiveDateTime::from_timestamp(timestamp as i64, 0);
    let datetime: DateTime<Utc> = DateTime::from_utc(naive, Utc);
    datetime.format("%Y-%m-%d %H:%M:%S").to_string()
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



pub fn query_url_gen(title: &str) -> Url {
    let url = Url::parse("http://odhecx54:5040/master/frameworks").unwrap();
    // let url = Url::parse("http://odhlab20:5040/master/frameworks").unwrap();

    
    return url;
}



// ################### TABLE #########################
#[derive(Copy, Clone, PartialEq, Eq, Hash)]
pub enum BasicColumn {
    Name,
    Mem,
    MemStr,
    CPUs,
    UpTime,
    UpSince,
    Tasks,
    TaksMap,
    URL,
}

impl BasicColumn {
    fn as_str(&self) -> &str {
        match *self {
            BasicColumn::Name => "Name",
            BasicColumn::Mem => "Mem",
            BasicColumn::MemStr => "MemStr",
            BasicColumn::CPUs => "CPUs",
            BasicColumn::UpTime => "UpTime",
            BasicColumn::UpSince => "Up Since",
            BasicColumn::Tasks => "Tasks",
            BasicColumn::TaksMap => "Tasks Map",
            BasicColumn::URL => "URL",
        }
    }
}

#[derive(Clone, Debug)]
pub struct Framework {
    name: String,
    mem: i32,
    mem_str: String,
    cpus: i32,
    uptime: String,
    upsince: String,
    tasks: i32,
    tasksmap: String,
    url: String,
}

impl Default for Framework {
    fn default() -> Framework {
        Framework {
            name: String::from(""),
            mem: 0,
            mem_str: String::from(""),
            cpus: 0,
            uptime: String::from(""),
            upsince: String::from(""),
            tasks: 0,
            tasksmap: String::from(""),
            url: String::from(""),
        }
    }
}

impl TableViewItem<BasicColumn> for Framework {
    fn to_column(&self, column: BasicColumn) -> String {
        match column {
            BasicColumn::Name => self.name.to_string(),
            BasicColumn::Mem => format!("{}", self.mem),
            BasicColumn::MemStr => format!("{}", self.mem_str),
            BasicColumn::CPUs => format!("{}", self.cpus),
            BasicColumn::UpTime => format!("{}", self.uptime),
            BasicColumn::UpSince => format!("{}", self.upsince),
            BasicColumn::Tasks => format!("{}", self.tasks),
            BasicColumn::TaksMap => format!("{}", self.tasksmap),
            BasicColumn::URL => format!("{}", self.url),
        }
    }

    fn cmp(&self, other: &Self, column: BasicColumn) -> Ordering
    where
        Self: Sized,
    {
        match column {
            BasicColumn::Name => self.name.cmp(&other.name),
            BasicColumn::Mem => self.mem.cmp(&other.mem),
            BasicColumn::MemStr => self.mem_str.cmp(&other.mem_str),
            BasicColumn::CPUs => self.cpus.cmp(&other.cpus),
            BasicColumn::UpTime => self.uptime.cmp(&other.uptime),
            BasicColumn::UpSince => self.upsince.cmp(&other.upsince),
            BasicColumn::Tasks => self.tasks.cmp(&other.tasks),
            BasicColumn::TaksMap => self.tasksmap.cmp(&other.tasksmap),
            BasicColumn::URL => self.url.cmp(&other.url),
        }
    }
}


