
use std::vec::Vec;
use num::Num;
use std::fmt::{Display, Formatter, Error, Debug};
use std::cmp::{Ordering};
use std::mem;

use self::ObjectiveKind::*;

#[derive(PartialEq, Debug)]
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

    pub unsafe fn alocate_mem(h: usize, w: usize) -> Matrix<F> {
        let mut m: Vec<F> = Vec::with_capacity(h * w);
        m.set_len(h * w);
        Matrix {
            h: h,
            w: w,
            m: m,
        }
    }
}

pub trait OrdField: Num + PartialEq + Copy + PartialOrd + Display + Debug {}
impl<F: Num + PartialEq + Copy + PartialOrd + Display + Debug> OrdField for F {}

#[derive(PartialEq, Debug)]
pub struct Dictionary<F: OrdField> {
    m: Matrix<F>,
    ll: Vec<usize>, // lines labels
    lc: Vec<usize>, // cols labels
    obj: Vec<F>,
    weq: Vec<F>,  // working equation
    var_name: &'static str,
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

pub enum Step {
    Finished,
    Empty, // TODO(leo): witness??
    Unbounded(usize), // "entering variable"
    Continue(usize, usize), // (entering, leaving)
}

impl<F: OrdField> Dictionary<F> {
    pub fn check_integrity(&self) {
        println!("{:?}", self);
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

    pub fn find_leaving_variable(&self, je: usize) -> LeavingCase<F> { // TODO(leo): get rid of dumb LeavingCase :/
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

    pub fn find_entering_variable(&self) -> Step { //TODO(leo): handle all cases
        use self::Step::*;
        println!("{}", self);
        for j in 1..self.w() {
            if self.obj[j] > F::zero() {
                println!("found non neg {:?} {:?}", j, self.find_leaving_variable(j));
                if let LeavingCase::Pos(i, _) = self.find_leaving_variable(j) {
                    println!("ctn with i={} j={}", i, j);
                    return Continue(i, j)
                }
            }
        }
        println!("finished");
        Finished
    }

    pub fn test_simplex(&mut self) {
        while let Step::Continue(i, j) = self.find_entering_variable() {
            self.perform_pivot(j, i);
            println!("{}", self);
        }
    }

    /// `je`: entering variable
    /// `il`: leaving varaible
    pub fn perform_pivot(&mut self, je: usize, il: usize) {
        println!("Performing pivot: entering {}, leaving {}", je, il);
        assert!(je != 0);
        for x in self.weq.iter_mut() {
            *x = F::zero();
        }
        let k = F::zero() - F::one() / self.m.at(il, je);
        for j in 0..self.w() {
            if j != je {
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

    /*
    pub fn dual(&self) -> Dictionary<F> {
        let dobj = {
            let mut res: Vec<F> = vec![F::zero()]; // FIXME(leo): true???
            for i in 0..self.h() {
                res.push(self.m.at(i, 0));
            }
            res
        };
        let dm = unsafe {
            let mut m: Matrix<F> = Matrix::alocate_mem(self.w()-1, self.h()+1);
            for i in 0..self.h() {
                for j in 1..self.w() {
                    m.set_at(j-1, i+1, self.m.at(i, j));
                }
            }
            for j in 1..self.w() {
                m.set_at(j-1, 0, self.obj[j]);
            }
            m
        };
        Dictionary {
            m: dm,
            lc: {
                let mut res = vec![0];
                for i in 0..self.h() { res.push(self.ll[i]); }
                res
            },
            ll: Vec::from(&self.lc.clone()[1..]), // TODO(leo): do better ?
            weq: dobj.clone(),
            obj: dobj,
            var_name: "y",
        }
    } */
}

impl<F: OrdField> Display for Dictionary<F> {
    // TODO(leo): Print x_0 as a cte
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        for i in 0..self.h() {
            let _ = write!(f, "{}_{} = ", self.var_name, self.ll[i]);

            let mut first = true;
            for j in 0..self.w() { // TODO(leo): ugly!
                if !first {
                    let _ = write!(f, " + ");
                }
                first = false;
                let _ = write!(f, "{} * {}_{}", self.m.at(i, j), self.var_name, self.lc[j]);
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
            let _ = write!(f, "{} * {}_{}", self.obj[j], self.var_name, self.lc[j]);
        }

        write!(f, "\n")
    }
}

pub enum OrderRel {
    LT,
    GT,
    EQ,
}

pub enum ObjectiveKind {
    Maximize,
    Minimize,
}

pub struct Inequation<F: OrdField> {
    coeffs: Vec<F>,
    order: OrderRel,
    cst: F,
}

impl<F: OrdField> Inequation<F> {
    pub fn size(&self) -> usize {
        self.coeffs.len()
    }
}



pub mod test {
    use super::*;

    pub fn make_dict() -> Dictionary<f64> {
        Dictionary {
            m: Matrix {
                h: 2,
                w: 3,
                m: vec![8., 1., 2., 12.,  -3., -4.]
            },
            ll: vec![3, 4],
            lc: vec![0, 1, 2],
            obj: vec![0., 3., 8.],
            weq: vec![0., 0., 0.],
            var_name: "x",
        }
    }

    #[test]
    fn test_at() {
        let lp = make_dict();
        assert_eq!(lp.m.at(1, 2), 4.);
        assert_eq!(lp.m.at(0, 1), 1.);
        assert_eq!(lp.m.at(0, 2), 2.);
        assert_eq!(lp.m.at(1, 1), -3.);
    }

    #[test]
    fn test_print() {
        println!("{}", make_dict())
    }

    #[test]
    fn test_leaving_variable() {
        let lp = make_dict();
        assert_eq!(lp.find_leaving_variable(1), super::LeavingCase::Pos(1, -4.0))
    }

    #[test]
    fn testcase_is_integre() {
        make_dict().check_integrity();
    }
}
