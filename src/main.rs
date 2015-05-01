extern crate num;

mod linear_system;
mod parser;

use std::fs::File;
use std::path::Path;
use std::io::Read;
use std::env;

fn main() {
    let args: Vec<_> = env::args().collect();
    if args.len() != 2 {
        println!("USAGE: cargo run [--release] -- file.lp");
        return;
    }

    let path = Path::new("/Users/ineol/Code/simplex/generated-1000-1000.lp");
    let mut file = match File::open(&path) {
        Err(why) => panic!("Could not open file because: {}", why),
        Ok(file) => file,
    };

    let mut src = String::new();
    file.read_to_string(&mut src);
    let mut lp = parser::Parser::parse_lp(&src);

    let mut d = lp.to_dict();
    println!("Call test_simplex");

    d.test_simplex();
}
