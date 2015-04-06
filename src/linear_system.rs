
use std::vec::Vec;
use num::Num;
use std::fmt::{Display, Formatter, Error};
use std::cmp::{Ordering};
use std::mem;

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

    pub fn transpose(&mut self) {
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

pub trait OrdField: Num + PartialEq + Copy + PartialOrd + Display {}
impl<F: Num + PartialEq + Copy + PartialOrd + Display> OrdField for F {}

#[derive(PartialEq)]
pub struct LinearSystem<F: OrdField> {
    m: Matrix<F>,
    ll: Vec<usize>, // lines labels
    lc: Vec<usize>, // cols labels
    obj: Vec<F>,
    weq: Vec<F>,  // working equation
}

#[derive(PartialEq, Debug)]
enum LeavingCase<F: PartialOrd> {
    NonNeg, // +infty
    Pos(usize, F)
}

impl<F: PartialOrd + Copy> PartialOrd for LeavingCase<F> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        use self::LeavingCase::*;
        use std::cmp::Ordering::{Less, Greater};
        match (self, other) {
            (&NonNeg, _) => Some(Greater),
            (&Pos(_, x1), &Pos(_, x2)) => Some (if x1 < x2 { Less } else { Greater }),
            _ => Some(Less)
        }
    }
}

impl<F: OrdField> LinearSystem<F> {
    fn check_integrity(&self) {
        assert!(self.m.h == self.ll.len());
        assert!(self.m.w == self.lc.len());
        assert!(self.m.w == self.obj.len());
        assert!(self.weq.len() == self.m.w);
    }

    fn w(&self) -> usize {
        self.m.w
    }

    fn h(&self) -> usize {
        self.m.h
    }

    fn find_leaving_variable(&self, je: usize) -> LeavingCase<F> { // TODO(leo): get rid of dumb LeavingCase :/
        use self::LeavingCase::*;
        assert!(je != 0);
        let coeffs = (0..self.h()).map(|i| if self.m.at(i, je) < F::zero() {
            Pos(i, F::zero() - self.m.at(i, 0) / self.m.at(i, je))
        } else {
            NonNeg
        });

        let res = coeffs.fold(None, |min, x| match min {
            None => Some(x),
            Some(m) => if x < m {
                Some(x)
            } else {
                Some(m)
            }
        });
        
        res.unwrap_or(NonNeg)
    }

    /// `je`: entering variable
    /// `il`: leaving varaible
    pub fn perform_pivot(&mut self, je: usize, il: usize) {
        assert!(je != 0 && il != 0);
        for x in self.weq.iter_mut() {
            *x = F::zero();
        }
        let k = F::zero() - F::one() / self.m.at(il, je);
        for j in 0..self.w() {
            if (j != je) {
                self.weq[j] = k * self.m.at(il, j);
            } else {
                self.weq[j] = F::zero() - k;
            }
        }

        // Perform the replacements
        for i in 0..self.h() {
            if i != il {
                let a = self.m.at(i, je);
                for j in 0..self.w() {
                    let old = self.m.at(i, j);
                    if j != je {
                        self.m.set_at(i, j, old + a*self.weq[j]);
                    } else {
                        self.m.set_at(i, j, a*self.weq[j]);
                    }
                }
            } else {
                for j in 0..self.w() {
                    self.m.set_at(i, j, self.weq[j]);
                }
            }
        }
        let a = self.obj[je];
        for j in 0..self.w() {
            assert!(a != F::zero());
            let old = self.obj[j];
            if j != je {
                self.obj[j] = old + a*self.weq[j];
            } else {
                self.obj[j] = a*self.weq[j];
            }
        } 
        
        // Change variable names
        mem::swap(&mut self.ll[il], &mut self.lc[je]);
    }
}

impl<F: OrdField> Display for LinearSystem<F> {
    // TODO(leo): Print x_0 as a cte
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
                w: 3,
                m: vec![8., 1., 2., 12.,  -3., 4.]
            },
            ll: vec![3, 4],
            lc: vec![0, 1, 2],
            obj: vec![33., 3., 8.],
            weq: vec![0., 0., 0.],
        }
    }

    #[test]
    fn test_at() {
        let lp = make_lp();
        assert_eq!(lp.m.at(1, 2), 4.);
        assert_eq!(lp.m.at(0, 1), 1.);
        assert_eq!(lp.m.at(0, 2), 2.);
        assert_eq!(lp.m.at(1, 1), -3.);
    }
    
    #[test]
    fn test_print() {
        println!("{}", make_lp())
    }

    #[test]
    fn test_leaving_variable() {
        let lp = make_lp();
        assert_eq!(lp.find_leaving_variable(1), super::LeavingCase::Pos(1, -4.0))
    }
    
    #[test]
    fn testcase_is_integre() {
        make_lp().check_integrity();
    }
}
