#[macro_use]
extern crate serde_derive;
extern crate serde_json;

use std::path::Path;
use std::fs::File;

#[derive(Deserialize)]
struct Stuff {
    field: String,
    optional: Option<i32>,
}

fn parse_json(filename: &str) -> Stuff {
    let path = Path::new(filename);
    let f = File::open(&path).expect("open input json");
    serde_json::from_reader(f).expect("parse json")
}

fn main() {
    let filename = "filename.json";
    let s = parse_json(filename);
    println!("field is {}", s.field);
    match s.optional {
        Some(n) => println!("optional is {}", n),
        None => println!("optional was not specified")
    }
}
