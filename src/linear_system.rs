use std::vec::Vec;
use num::Num;
use std::fmt::{Display, Formatter, Error, Debug};
use std::cmp::{Ordering};
use std::mem;

pub use self::ObjectiveKind::*;

// Utils
pub fn init_zero_vec<T: Copy>(n: usize, val: T) -> Vec<T> {
    let mut vec: Vec<T> = Vec::with_capacity(n);
    for _ in 0..n {
        vec.push(val);
    }
    vec
}

// To avoid bound checking
pub fn vec_at<T: Copy>(v: &Vec<T>, n: usize) -> T {
    unsafe { *v.get_unchecked(n) }
}

pub fn vec_at_mut<T>(v: &mut Vec<T>, n: usize) -> &mut T {
    unsafe { v.get_unchecked_mut(n) }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Rect {
    pub i: usize,
    pub j: usize,
    pub h: usize,
    pub w: usize,
}

#[derive(PartialEq, Debug, Clone)]
pub struct Matrix<F: Num + PartialEq> {
    pub h: usize,
    pub w: usize,
    pub m: Vec<F> // Size is (at least) w * h
}

impl<F: Num + Copy + PartialEq> Matrix<F> {
    pub fn at(&self, i: usize, j: usize) -> F {
        // assert!(i < self.h);
        // assert!(j < self.w);
        unsafe { *self.m.get_unchecked(j + self.w * i) }
    }

    pub fn set_at(&mut self, i: usize, j:usize, x: F) {
        // assert!(i < self.h);
        // assert!(j < self.w);
        unsafe { *self.m.get_unchecked_mut(j + self.w * i) = x; }
    }

    pub unsafe fn allocate_mem(h: usize, w: usize) -> Matrix<F> {
        let mut m: Vec<F> = Vec::with_capacity(h * w);
        m.set_len(h * w);
        Matrix {
            h: h,
            w: w,
            m: m,
        }
    }

    pub fn allocate_zeroed(h: usize, w: usize) -> Matrix<F> {
        let mut m = unsafe { Self::allocate_mem(h, w) };
        for i in 0..h*w {
            m.m[i] = F::zero();
        }
        m
    }

    pub fn blit(&self, dst: &mut Matrix<F>, src: Rect, i_dst: usize, j_dst: usize) {
        for i in 0..src.h {
            for j in 0..src.w {
                let val = self.at(src.i + i, src.j + j);
                dst.set_at(i_dst+i, j_dst+j, val);
            }
        }
    }
}

pub trait OrdField: Num + PartialEq + Copy + PartialOrd + Display + Debug {}
impl<F: Num + PartialEq + Copy + PartialOrd + Display + Debug> OrdField for F {}

#[derive(PartialEq, Debug, Clone)]
pub struct Dictionary<F: OrdField> {
    pub m: Matrix<F>,
    pub ll: Vec<usize>, // lines labels
    pub lc: Vec<usize>, // cols labels
    pub obj: Vec<F>, // We maximize
    pub weq: Vec<F>,  // working equation
    pub var_name: &'static str,
}

#[derive(Clone, Copy)]
pub enum Heuristic {
    Bland,
    Dumb,
}

#[derive(PartialEq, Debug)]
enum LeavingCase<F: PartialOrd> {
    NonNeg, // +infty
    Pos(usize, F)
}

const FIRST_PHASE_IDX: usize = 1 << 30;

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
        //println!("{:?}", self);
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


    fn create_first_dict(&self) -> Dictionary<F> {
       let mut m = Matrix::allocate_zeroed(self.h(), self.w() + 1);
       self.m.blit(&mut m, Rect {i: 0, j: 0, h: self.h(), w: self.w()}, 0, 0);
       for i in 0..self.h() {
           m.set_at(i, self.w(), F::one());
       }
       let mut lc = self.lc.clone();
       lc.push(FIRST_PHASE_IDX);
       let mut obj = init_zero_vec(m.w -1, F::zero());
       obj.push(F::zero() - F::one());
       let res = Dictionary {
           m: m,
           ll: self.ll.clone(),
           lc: lc,
           obj: obj,
           weq: init_zero_vec(self.w()+1, F::zero()),
           var_name: self.var_name,
       };
       res.check_integrity();
       res
    }

    /// Creates a new `Dictionary` without the component `xr`
    /// It creates a copy of itself if xr is not a variable of `self`
    fn project_dict(&self, xr: usize, orig: &mut Dictionary<F>) {
        if let Some(ir) = self.ll.iter().position(|&x| x == xr) {
            self.m.blit(&mut orig.m, Rect{i: 0, j: 0, h: ir, w: self.w()}, 0, 0);
            self.m.blit(&mut orig.m, Rect{i: ir+1, j: 0, h: self.h()-ir-1, w: self.w()}, ir, 0);
            orig.fix_obj_after_first_phase(self, FIRST_PHASE_IDX);
            let mut ll = self.ll.clone();
            ll.remove(ir); // TODO(leo): optimize?
            orig.ll= ll;
            orig.lc = self.lc.clone();
        } else if let Some(jr) = self.lc.iter().position(|&x| x == xr) {
            self.m.blit(&mut orig.m, Rect{i: 0, j: 0, h: self.h(), w: jr}, 0, 0);
            self.m.blit(&mut orig.m, Rect{i: 0, j: jr+1, h: self.h(), w: self.w()-jr-1}, 0, jr);
            orig.fix_obj_after_first_phase(self, FIRST_PHASE_IDX);
            let mut obj = self.obj.clone();
            let mut lc = self.lc.clone();
            obj.remove(jr);
            lc.remove(jr);
            orig.ll= self.ll.clone();
            orig.lc = lc;
        }
    }

    fn fix_obj_after_first_phase(&mut self, afp: &Dictionary<F>, exc: usize) {
        let mut res = init_zero_vec(self.w(), F::zero());
        res[0] = self.obj[0];
        for (j, &c_j) in self.obj.iter().enumerate() {
            let x_j = self.lc[j];
            if x_j == exc { continue; }
            if c_j != F::zero() {
                if let Some(&x_i) = afp.ll.iter().find(|&x| *x == x_j) { // TODO(leo): WTF???
                    // x_i is a primary variable of afp.
                    for j in 0..res.len() {
                        res[j] = res[j] + c_j * self.m.at(x_i, j); // I FIXME(leo): i hate myself
                    }
                }
            }
        }
        self.obj = res;
    }

    fn find_first_pivot(&self) -> usize {
        let mut res = 0;
        let mut min = self.m.at(0, 0);
        for i in 1..self.h() {
            let x = self.m.at(i, 0);
            if x < min {
                min = x;
                res = i;
            }
        }
        res
    }

    pub fn find_leaving_variable(&self, je: usize) -> LeavingCase<F> { // TODO(leo): get rid of dumb LeavingCase :/
        use self::LeavingCase::*;
        //assert!(je != 0);
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

    fn eval_line(&self, sol: &[F], i: usize) -> F {
        let mut sum = self.m.at(i, 0);
        for j in 1..self.w() {
            sum = sum + self.m.at(i, j) * sol[j-1];
        }
        sum
    }

    pub fn is_solution(&self, sol: Vec<F>) -> bool {
        //assert!(sol.len() == self.obj.len() - 1);
        for i in 0..self.h() {
            if self.eval_line(&sol, i) < F::zero() {
                println!("BUURRRNNN");
                return false;
            }
        }
        true
    }

    pub fn find_entering_variable(&self) -> Step { //TODO(leo): handle all cases
        use self::Step::*;
        for j in 1..self.w() {
            if self.obj[j] > F::zero() {
                println!("found non neg {:?} {:?}", j, self.find_leaving_variable(j));
                if let LeavingCase::Pos(i, _) = self.find_leaving_variable(j) {
                    println!("ctn with i={} j={}", i, j);
                    return Continue(i, j)
                } else {
                    println!("UNBOUNDED");
                    return Unbounded(j);
                }
            }
        }
        println!("finished");
        Finished
    }

    pub fn find_entering_variable_dumb(&self) -> Step {
        use self::Step::*;
        let mut max = self.obj[1];
        let mut jmax = 1;
        for (j, &c_j) in self.obj[1..].iter().enumerate() {
            if c_j >= max { max = c_j; jmax = j; }
        }

        let j = jmax + 1;

        if max > F::zero() {
            if let LeavingCase::Pos(i, _) = self.find_leaving_variable(j) {
                println!("ctn with i={} j={}", i, j);
                return Continue(i, j)
            } else {
                println!("UNBOUNDED");
                return Unbounded(j);
            }
        }
        Finished
    }

    pub fn run_simplex(&mut self, heur: Heuristic) { // TODO: handle all cases
        //println!("INPUT OF PROGRAM:\n {}", self);

        let do_first_phase = {
            let nil_sol: Vec<F> = init_zero_vec(self.w()-1, F::zero());
            !self.is_solution(nil_sol)
        };

        println!("Should we do the first phase? {}", do_first_phase);

        if (do_first_phase) {
            let mut d = self.create_first_dict();
            let i = d.find_first_pivot();
            d.perform_pivot(self.w(), i);
            //println!("{}", d);
            d.run_simplex(heur);
            //println!("END OF FIRST PHASE with res {}", d.obj[0]);
            d.project_dict(FIRST_PHASE_IDX, self);
            //println!("DICT FOR BEGINNING OF SECOND PHASE:\n {}", self);
        }

        let fev: fn (&Self) -> Step = match heur {
            Heuristic::Bland => Self::find_entering_variable,
            Heuristic::Dumb => Self::find_entering_variable_dumb,
        };

        while let Step::Continue(i, j) = fev(self) {
            self.perform_pivot(j, i);
        }
        println!("Values of variables (except when zero)");
        for i in 0..self.h() {
            if self.m.at(i, 0) != F::zero() {
                println!("x_{} = {}", i, self.m.at(i, 0));
            }
        }
    }

    /// `je`: entering variable
    /// `il`: leaving varaible
    pub fn perform_pivot(&mut self, je: usize, il: usize) {
        println!("Performing pivot: entering {}, leaving {}", je, il);
        //assert!(je != 0);
        for x in self.weq.iter_mut() {
            *x = F::zero();
        }
        let k = F::zero() - F::one() / self.m.at(il, je);
        for j in 0..self.w() {
            if j != je {
                *vec_at_mut(&mut self.weq, j) = k * self.m.at(il, j);
            } else {
                *vec_at_mut(&mut self.weq, j) = F::zero() - k;
            }
        }

        // Perform the replacements
        for i in 0..self.h() {
            if i != il {
                let a = self.m.at(i, je);
                for j in 0..self.w() {
                    let old = self.m.at(i, j);
                    if j != je {
                        self.m.set_at(i, j, old + a*vec_at(&self.weq, j));
                    } else {
                        self.m.set_at(i, j, a*vec_at(&self.weq, j));
                    }
                }
            } else {
                for j in 0..self.w() {
                    self.m.set_at(i, j, vec_at(&self.weq, j));
                }
            }
        }
        let a = self.obj[je];
        for j in 0..self.w() {
            //assert!(a != F::zero());
            let old = vec_at(&self.obj, j);
            if j != je {
                self.obj[j] = old + a*vec_at(&self.weq, j);
            } else {
                self.obj[j] = a*vec_at(&self.weq, j);

            }
        }

        // Change variable names
        mem::swap(&mut self.ll[il], &mut self.lc[je]);
    }
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

#[derive(Eq, PartialEq, Debug, Copy, Clone)]
pub enum OrderRel {
    LT,
    GT,
    EQ,
}

#[derive(Eq, PartialEq, Debug, Copy, Clone)]
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

pub struct NormalLinearProgram<F: OrdField> {
    ineqs: Vec<Inequation<F>>,
    obj: Vec<F>,
    obj_kind: ObjectiveKind,
    // TODO(leo): names
}

impl<F: OrdField> NormalLinearProgram<F> {
    pub fn check_integrity(&self) {
        for v in self.ineqs.iter() {
            assert_eq!(v.size(), self.obj.len());
        }
    }
}

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

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_at() {
        let lp = make_dict();
        assert_eq!(lp.m.at(1, 2), -4.);
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
        assert_eq!(lp.find_leaving_variable(1), super::LeavingCase::Pos(1, 4.0)) // TODO(leo): correct??
    }

    #[test]
    fn testcase_is_integre() {
        make_dict().check_integrity();
    }
}
