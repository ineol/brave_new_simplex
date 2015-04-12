extern crate num;

mod linear_system;
mod parser;

fn main() {
    let mut lp = linear_system::make_dict();

    lp.test_simplex();
}
