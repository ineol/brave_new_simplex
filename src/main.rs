extern crate num;

mod linear_system;

fn main() {
    let mut lp = linear_system::test::make_dict();

    lp.test_simplex();
}
