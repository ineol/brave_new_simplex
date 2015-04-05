
use std::vec::Vec;
use num::Num;
use std::string::String;
use std::fmt::{Display, Formatter, Error};

#[derive(PartialEq)]
pub struct Matrix<F: Num + PartialEq> {
    h: usize,
    w: usize,
    m: Vec<F> // Size is (at least) w * h
}

impl<F: Num + Copy + PartialEq> Matrix<F> {
    pub fn at(&self, i: usize, j: usize) -> F {
        assert!(i < self.h);
        assert!(j < self.w);
        self.m[j + self.w * i]
    }

    pub fn set_at(&mut self, i: usize, j:usize, x: F) {
        assert!(i < self.h);
        assert!(j < self.w);
        self.m[j + self.w * i] = x;
    }

    pub fn transpose(&mut self) { // TODO(leo): don't copy maybe
        let mut new_m = unsafe {
            let mut res: Vec<F> = Vec::with_capacity(self.h * self.w);
            res.set_len(self.h * self.w);
            res
        };
        for i in 0 .. self.h {
            for j in 0 .. self.w {
                new_m[j + self.w * i] = self.at(j, i)
            }
        }
        self.m = new_m;
    }
}

#[derive(PartialEq)]
pub struct LinearSystem<F: Num + PartialEq> {
    m: Matrix<F>,
    ll: Vec<usize>, // lines labels
    lc: Vec<usize>,  // cols labels 
    obj: Vec<F>
}

impl<F: Num + Copy + PartialEq> LinearSystem<F> {
    fn check_integrity(&self) {
        assert!(self.m.h == self.ll.len());
        assert!(self.m.w == self.lc.len());
        assert!(self.m.w == self.obj.len());
    }

    fn add_lines(&mut self, i1: usize, i2: usize) { // i1 = i1 + i2
        // TODO(leo): is it useful? 
    }

    fn w(&self) -> usize {
        self.m.w
    }

    fn h(&self) -> usize {
        self.m.h
    }
}

impl<F: Num + Copy + PartialEq + Display> Display for LinearSystem<F> {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        for i in 0..self.h() {
            let _ = write!(f, "x_{} = ", self.ll[i]);

            let mut first = true;
            for j in 0..self.w() { // TODO(leo): ugly!
                if !first {
                    let _ = write!(f, " + ");
                }
                first = false;
                let _ = write!(f, "{} * x_{}", self.m.at(i, j), self.lc[j]);
            }
            let _ = write!(f, "\n");
            
        } 
        let _ = write!(f, "------------\n");
        let _ = write!(f, "z   = ");
        let mut first = true;
        for j in 0..self.w() {
            if !first {
                let _ = write!(f, " + ");
            }
            first = false;
            let _ = write!(f, "{} * x_{}", self.obj[j], self.lc[j]);
        }
        
        write!(f, "\n")
    }
}

pub mod test {
    use super::*;

    pub fn make_lp() -> LinearSystem<f64> {
        LinearSystem {
            m: Matrix {
                h: 2,
                w: 2,
                m: vec![1., 2., 3., 4.]
            },
            ll: vec![3, 4],
            lc: vec![1, 2],
            obj: vec![3., 8.]
        }
    }

    #[test]
    fn test_at() {
        let mut lp = make_lp();
        assert_eq!(lp.m.at(1, 1), 4.);
        assert_eq!(lp.m.at(0, 0), 1.);
        assert_eq!(lp.m.at(0, 1), 2.);
        assert_eq!(lp.m.at(1, 0), 3.);
    }
    
    #[test]
    fn test_print() {
        println!("{}", make_lp())
    }

    #[test]
    fn testcase_is_integre() {
        make_lp().check_integrity();
    }
}
