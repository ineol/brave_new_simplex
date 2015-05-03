extern crate num;
extern crate getopts;

mod linear_system;
mod parser;

use std::fs::File;
use std::path::Path;
use std::io::Read;
use std::env;

use getopts::{Options};

fn main() {
    let args: Vec<_> = env::args().collect();

    let mut opts = Options::new();
    opts.optflag("b", "bland", "Use Bland's rule");
    opts.optflag("l", "latex", "Print the steps in LaTeX");
    let matches = match opts.parse(&args[1..]) {
        Ok(m) => m,
        Err(err) => {
            println!("{}", err.to_string());
            return;
        },
    };

    if matches.free.len() != 1 {
        println!("USAGE: cargo run [--release] -- [-bl] file.lp");
        return;
    }

    let path = Path::new(&matches.free[0]);
    let mut file = match File::open(&path) {
        Err(why) => panic!("Could not open file because: {}", why),
        Ok(file) => file,
    };

    let mut src = String::new();
    file.read_to_string(&mut src);
    let mut lp = parser::Parser::parse_lp(&src);

    let mut d = lp.to_dict();
    d.run_simplex(linear_system::Heuristic::Dumb);
}
