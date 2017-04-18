extern crate serde_json;

use std::path::Path;
use std::fs::File;

#[derive(Deserialize)]
pub struct PItem {
    pub c_var: String,
    pub c_type: String,
    pub help: Option<String>,
}

#[derive(Deserialize)]
pub struct NPItem {
    pub c_var: String,
    pub c_type: String,
    pub name: String,
    pub short: Option<String>,
    pub aliases: Option<Vec<String>>,
    pub help: Option<String>,
    pub required: String,
    pub default: u8,
}

#[derive(Deserialize)]
pub struct Spec {
    pub positional: Option<Vec<PItem>>,
    pub non_positional: Option<Vec<NPItem>>,
    pub c_file: String,
    pub optional: Option<String>,
}

pub fn parse_json(filename: &str) -> Spec {
    let path = Path::new(filename);
    let f = File::open(&path).expect("open input json");
    serde_json::from_reader(f).expect("parse json")
}
