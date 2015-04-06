
#![feature(collections)]

extern crate num;

mod linear_system;

fn main() {
    let mut lp = linear_system::test::make_lp();
    println!("{}", lp);
    lp.perform_pivot(1, 1);
    println!("{}", lp);
    
}
