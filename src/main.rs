#[macro_use]
extern crate serde_derive;

mod codegen;

use std::env;
use std::fs::File;
use std::path::Path;
use codegen::Spec;


pub fn parse_input(filename: &str) -> Spec {
    let path = Path::new(filename);
    let f = File::open(&path).expect("open input json");
    Spec::from_reader(f)
}

fn main() {
    let filename = env::args()
        .nth(1)
        .unwrap_or(String::from("specs.json"));
    let s = args::parse_json(&filename);
    println!("{}", s.gen());
}
