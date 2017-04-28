#[macro_use]
extern crate serde_derive;
extern crate getopts;
extern crate regex;
extern crate toml;

mod codegen;

use codegen::Spec;
use getopts::Options;
use std::env;
use std::fs::File;
use std::io;
use std::io::{Write, Read};
use std::path::Path;
use std::process;


fn print_usage(program: &str, opts: Options) {
    let brief = format!("Usage: {} [options] FILE", program);
    print!("{}", opts.usage(&brief));
}

pub fn codegen(filename: String, output: Option<String>) {
    let path = Path::new(&filename);
    let mut f = File::open(&path).expect("open input toml");
    let mut contents = String::new();
    f.read_to_string(&mut contents).expect("read input toml");
    let s = Spec::from_str(&contents);
    if let Err(e) = s {
        writeln!(&mut io::stderr(), "Spec Parse Error: {}", e).unwrap();
        process::exit(1);
    }
    let s = s.unwrap();
    match output {
        Some(f) => {
            let p = Path::new(&f);
            let mut f = File::create(&p).expect("open output file");
            s.writeout(&mut f)
        }
        None => s.writeout(&mut io::stdout()),
    };
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let program = args[0].clone();

    let mut opts = Options::new();
    opts.optopt("o", "", "set output file name", "NAME");
    opts.optflag("h", "help", "print this help menu");
    let matches = match opts.parse(&args[1..]) {
        Ok(m) => m,
        Err(f) => panic!(f.to_string()),
    };
    if matches.opt_present("h") {
        print_usage(&program, opts);
        return;
    }
    let output = matches.opt_str("o");
    let input = if !matches.free.is_empty() {
        matches.free[0].clone()
    } else {
        print_usage(&program, opts);
        return;
    };

    codegen(input, output)
}

#[cfg(test)]
mod tests {
    use super::codegen;

    #[test]
    fn it_works() {
        codegen(String::from("specs.json"), None)
    }
}
