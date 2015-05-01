extern crate num;

mod linear_system;
mod parser;

use std::fs::File;
use std::path::Path;
use std::io::Read;

fn main() {
    let path = Path::new("generated-100-100.lp");
    let mut file = match File::open(&path) {
        Err(why) => panic!("Could not open file because: {}", why),
        Ok(file) => file,
    };

    let mut src = String::new();
    file.read_to_string(&mut src);
    println!("{}", &src);
    let mut lp = parser::Parser::parse_lp(&src);
    println!("{:?}\n\n\n\n", lp);

    let mut d = lp.to_dict();
    d.test_simplex();
}
