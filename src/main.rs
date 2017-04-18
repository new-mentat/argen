#[macro_use]
extern crate serde_derive;
extern crate serde_json;

use std::path::Path;
use std::fs::File;

#[derive(Deserialize)]
struct PItem {
  c_var: String,
  c_type: String,
  help: Option<String>
}

#[derive(Deserialize)]
struct NPItem {
  c_var: String,
  c_type: String, 
  name: String,
  short: Option<String>, 
  aliases: Option <Vec<String>>,
  help: Option<String>,
  required: String,
  default: u8,  
}

#[derive(Deserialize)]
struct Spec {
    positional: Option<Vec<PItem>>, 
    non_positional: Option<Vec<NPItem>>, 
    c_file: String,
    optional: Option<String>
}

fn parse_json(filename: &str) -> Spec {
    let path = Path::new(filename);
    let f = File::open(&path).expect("open input json");
    serde_json::from_reader(f).expect("parse json")
}

fn main() {
    let filename = "specs.json";
    let s = parse_json(filename);

    println!("c_file is {}", s.c_file);

    match s.positional{
        Some(n) => println!("c_var for positional[0] is {}", n[0].c_var),
        None => println!("positional was not specified")
    }

    match s.non_positional {
        Some(n) => println!("c_var for non_positional[0] is {}", n[0].c_var),
        None => println!("non_positional was not specified")
    }
}
