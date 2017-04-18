#[macro_use]
extern crate serde_derive;

mod args;
mod codegen;

use codegen::CodeGen;

fn main() {
    let filename = "specs.json";
    let s = args::parse_json(filename);
    let cg = CodeGen::new(s);
    println!("{}", cg.gen());
}
