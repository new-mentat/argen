#[macro_use]
extern crate serde_derive;

mod args;
mod codegen;

use std::env;
use codegen::CodeGen;

fn main() {
    let filename = env::args()
        .nth(1)
        .unwrap_or(String::from("specs.json"));
    let s = args::parse_json(&filename);
    let cg = CodeGen::new(s);
    println!("{}", cg.gen());
}
