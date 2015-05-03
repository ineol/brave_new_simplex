extern crate num;
extern crate getopts;

mod linear_system;
mod parser;

use std::fs::File;
use std::path::Path;
use std::io::Read;
use std::env;

use getopts::{Options};

fn print_latex_header() {
    println!(r"\documentclass[9pt]{{article}}");
    println!(r"\usepackage[latin1]{{inputenc}}");
    println!(r"\usepackage[T1]{{fontenc}}");
    println!(r"\usepackage[french]{{babel}}");
    println!(r"\usepackage{{setspace}}");
    println!(r"\usepackage{{lmodern}}");
    println!(r"\usepackage{{soul}}");
    println!(r"\usepackage{{ulem}}");
    println!(r"\usepackage{{enumerate}}");
    println!(r"\usepackage{{amsmath,amsfonts, amssymb}}");
    println!(r"\usepackage{{mathrsfs}}");
    println!(r"\usepackage{{amsthm}}");
    println!(r"\usepackage{{float}}");
    println!(r"\usepackage{{array}}");
    println!(r"\usepackage{{mathabx}}");
    println!(r"\usepackage{{stmaryrd}}");
    println!("");
    println!(r"\begin{{document}}");
}

fn print_latex_footer() {
    println!(r"\end{{document}}");
}

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

    let kind = lp.goal;

    let mut d = lp.to_dict();
    let heur = if matches.opt_present("b") {
        linear_system::Heuristic::Bland
    } else {
        linear_system::Heuristic::Dumb
    };
    let latex = matches.opt_present("l");
    if latex {
        print_latex_header();
        println!("This is the initial dictionary: {}\n", d);
    }
    let x = d.run_simplex(heur, latex);

    if let Some(opt) = x {
        let opt = match kind {
            linear_system::Maximize => opt,
            linear_system::Minimize => -opt,
        };
        if latex {
            println!("The optimum is ${}$\n", opt);
        } else {
            println!("The optimum is {:.10}\n", opt);
        }

        println!("Values of non-nil variables: \n");
        for i in 0..d.h() {
            if d.m.at(i, 0) == 0.0 { continue; }
            if latex {
                println!("$x_{{ {} }} = {}$\n", d.ll[i], d.m.at(i, 0));
            } else {
                println!("x_{} = {}\n", d.ll[i], d.m.at(i, 0));
            }
        }
    }

    if latex {
        print_latex_footer();
    }
}
