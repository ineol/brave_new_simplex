use std::option::Option;
use std::str;
use std::collections::HashMap;
use std::str::FromStr;

use std::option::Option::*;

use linear_system::*;

#[derive(Eq, PartialEq, Debug, Clone)]
pub struct PInequation<F: OrdField> {
    prods: Vec<(F, String)>,
    kind: OrderRel,
    cst: F,
}

#[derive(PartialEq, Debug, Clone)]
pub struct PBound {
    var: String,
    upper: Option<f64>,
    lower: Option<f64>,
}

#[derive(PartialEq, Debug, Clone)]
pub struct LinearProgram {
    obj: Vec<(f64, String)>,
    obj_cst: f64,
    goal: ObjectiveKind,
    ineqs: Vec<PInequation<f64>>,
    bounds: Vec<PBound>,

    vars: Vec<String>,
    vars_inv: HashMap<String, usize>,
    dummy_idx: usize,
}

impl LinearProgram {
    pub fn to_dict(&mut self) -> Dictionary<f64> {
        self.normalize_bounds();

        let mut m: Matrix<f64> =
            Matrix::allocate_zeroed(self.ineqs.len(), self.vars.len() + 1); // +1 for the cst term

        for (i, ineq) in self.ineqs.iter().enumerate() {
            let mult: f64 = match ineq.kind {
                OrderRel::EQ => panic!("The equalities must have been deleted before!"),
                OrderRel::LT => 1.0,
                OrderRel::GT => -1.0,
            };

            m.set_at(i, 0, mult * ineq.cst);
            for &(c_j, ref x_j) in ineq.prods.iter() {
                let j = self.var_idx(&x_j);
                m.set_at(i, j+1, -mult * c_j);
            }
        }

        let mut obj: Vec<f64> = init_zero_vec(self.vars.len() + 1, 0.0);
        let mult: f64 = match self.goal {
            Maximize => 1.0,
            Minimize => -1.0,
        };
        obj[0] = self.obj_cst;
        for &(c_j, ref x_j) in self.obj.iter() {
            obj[self.var_idx(&x_j) + 1] = mult * c_j;
        }

        let mut lc: Vec<usize> = init_zero_vec(m.w, 0);
        let mut i = 0;
        for label in lc.iter_mut() {
            *label = i;
            i += 1;
        }

        let mut ll: Vec<usize> = init_zero_vec(m.h, 0);
        for label in ll.iter_mut() {
            *label = i;
            i += 1;
        }

        let mw = m.w;
        Dictionary {
            m: m,
            ll: ll,
            lc: lc,
            obj: obj,
            weq: init_zero_vec(mw, 0.0),
            var_name: "x",
        }
    }

    fn normalize_bounds(&mut self) {
        let bounds = self.bounds.clone();
        for (i, _) in bounds.iter().enumerate() {
            self.handle_upper_bound(i);
            self.handle_lower_bound(i);
        }
    }

    fn handle_upper_bound(&mut self, bidx: usize) {
        let b = self.bounds[bidx].clone();
        if let Some(u) = b.upper {
            self.ineqs.push(PInequation {
                prods: vec![(1.0, b.var)],
                kind: OrderRel::LT,
                cst: u,
            });
            self.bounds[bidx].upper = None;
        }
    }

    // The bound has no upper bound
    fn handle_lower_bound(&mut self, bidx: usize) {
        let mut b = self.bounds[bidx].clone();
        match b {
            PBound { var: ref x_j, upper: None, lower: Some(lower) } => {
                if lower == 0.0 { return; }
                self.translate_var(&x_j, lower);
                let nvar = self.create_dummy_var(x_j, "tr");
                self.bounds[bidx] = PBound {
                    var: nvar,
                    upper: None,
                    lower: Some(0.0),
                };
            },
            _ => panic!("Problem while handling lower bounds"),
        }

    }

    fn translate_var(&mut self, x_j: &str, t: f64) {
        for ineq in self.ineqs.iter_mut() {
            let j = match ineq.prods.iter().position(|x| x.1 == x_j) {
                Some(j) => j,
                None => continue,
            };
            let k = ineq.prods[j].0;
            ineq.cst += k * t;
        }
        match self.obj.iter().position(|x| x.1 == x_j) {
            Some(j) => {
                self.obj_cst -= t * self.obj[j].0;
            },
            None => (),
        }
    }

    // TODO: record the relationships
    fn create_dummy_var(&mut self, var: &str, msg: &str) -> String {
        let res = format!("{}${}${}", var, self.dummy_idx, msg).to_string();
        self.dummy_idx += 1;
        res
    }

    #[inline(never)]
    fn var_idx(&self, var: &str) -> usize {
        if let Some(&res) = self.vars_inv.get(var) {
            return res;
        }
        panic!("Couldn't find variable {}", var);
    }

    fn build_vars_inv(vars: &Vec<String>) -> HashMap<String, usize> {
        let mut res = HashMap::new();
        for (i, s) in vars.iter().enumerate() {
            res.insert(s.clone(), i);
        }
        res
    }

}

#[derive(Clone)]
pub struct Parser<'a> {
    src: &'a str,
    cur: str::CharIndices<'a>,
}

// Suppose skip_ws has been called at begining of function
impl<'a> Parser<'a> {
    pub fn new(src: &'a str) -> Parser<'a> {
        Parser {
            src: src,
            cur: src.char_indices(),
        }
    }

    pub fn parse_lp(src: &str) -> LinearProgram {
        let mut p = Parser::new(src);
        let mut ineqs: Vec<PInequation<f64>> = Vec::new();
        let mut bounds: Vec<PBound> = Vec::new();
        p.ws();
        let goal = p.obj_kind();
        let obj = p.sum();
        p.ws();
        p.word(); p.ws(); p.word(); // TODO
        p.ws();
        while let Some(eq) = p.inequation() {
            ineqs.push(eq);
            p.ws();
        }
        let r = p.word(); // TODO: BOUNDS
        println!("{}", r);
        p.ws();
        while let Some(b) = p.bound() {
            bounds.push(b);
            p.ws();
        }
        let r = p.word(); // VARIABLES
        println!("{}", r);
        p.ws();
        let vars = p.variables();
        let vars_inv = LinearProgram::build_vars_inv(&vars);

        LinearProgram {
           obj: obj,
           obj_cst: 0.0,
           goal: goal,
           ineqs: ineqs,
           bounds: bounds,
           vars: vars,
           vars_inv: vars_inv,

           dummy_idx: 0,
        }
    }

    fn next_pos(&self) -> usize {
        let mut it = self.cur.clone();
        it.next().map(|p| p.0).unwrap_or(self.src.len())
    }

    fn eat(&mut self, c: char) -> bool {
        match self.peek(0) {
            Some((_, x)) if x == c => { self.cur.next(); true }
            Some(_) | None => false,
        }
    }

    /// Peek `n` characters ahead
    fn peek(&self, n: usize) -> Option<(usize, char)> {
        self.cur.clone().skip(n).next()
    }

    /// Parse a word
    fn word(&mut self) -> String {
        let mut res = String::new();
        loop {
            match self.peek(0) {
                Some((_, c)) if !Parser::is_sep(c) => { self.cur.next(); res.push(c); },
                _ => break,
            }
        }
        res
    }

    /// Skip whitespace
    fn ws(&mut self) {
        loop {
            match self.peek(0) {
                Some((_, c)) if c == ' ' || c == '\t' || c == '\n' => { self.cur.next(); }
                _ => break,
            }
        }
    }

    /// Parse the VARIABLE clause
    fn variables(&mut self) -> Vec<String> {
        let mut res: Vec<String> = Vec::new();
        loop {
            let v = self.word();
            if v == "" { println!("End of File"); break; }
            res.push(v);
            self.ws();
        }
        res
    }

    /// Parse the kind of objective
    fn obj_kind(&mut self) -> ObjectiveKind {
        let k = self.word();
        if k == "MINIMIZE" {
            Minimize
        } else if k == "MAXIMIZE" {
            Maximize
        } else {
            panic!("The file must begin with MAXIMIZE or MINIMIZE");
        }
    }

    /// Parse product `litteral * var`
    fn prod(&mut self) -> Option<(f64, String)> {
        match self.peek(0) {
            Some((_, c)) if Parser::is_number_start(c) => {
                let n = self.number();
                self.ws();
                self.eat('*');
                self.ws();
                let w = self.word();
                n.map(|x| (x, w))
            },
            Some((_, c)) => {
                Some((1.0, self.word()))
            },
            _ => None,
        }
    }

    /// Parse a sum of products
    fn sum(&mut self) -> Vec<(f64, String)> {
        let backup = self.clone();
        let mut res: Vec<(f64, String)> = Vec::new();
        let mut mult: f64 = 1.0;
        if self.eat('-') {
            mult = -1.0;
        } else {
            self.eat('+');
        }
        self.ws();
        loop {
            match self.prod() {
                Some((n, x)) => res.push((mult * n, x)),
                None => break,
            }
            self.ws();
            if self.eat('-') {
                mult = -1.0;
            } else if self.eat('+') {
                mult = 1.0;
            } else {
                break;
            }
            self.ws();
        }
        if res.len() == 0 {
            *self = backup;
        }
        res
    }

    /// Parse a comprator (`<=`, `>=`, `=`)
    fn cmp_op(&mut self) -> Option<OrderRel> {
        if self.eat('=') {
            Some(OrderRel::EQ)
        } else if self.eat('<') && self.eat('=') {
            Some(OrderRel::LT)
        } else if self.eat('>') && self.eat('=') {
            Some(OrderRel::GT)
        } else {
            None
        }
    }

    /// Parse an inequation
    fn inequation(&mut self) -> Option<PInequation<f64>> {
        let backup = self.clone();
        let sum = self.sum();
        self.ws();
        let cmp = self.cmp_op();
        self.ws();
        let cst = self.signed_number();
        if let (Some(c), Some(k)) = (cmp, cst) {
            Some(PInequation {
                prods: sum,
                kind: c,
                cst: k,
            })
        } else {
            *self = backup;
            None
        }
    }

    fn bound(&mut self) -> Option<PBound> {
        match self.peek(0) {
            Some((_, c)) if Parser::is_number_start(c) =>
                self.double_bound(),
            Some(_) =>
                self.single_bound(),
            None => None,
        }
    }

    fn double_bound(&mut self) -> Option<PBound> {
        let lb = self.signed_number();
        self.ws();
        if self.cmp_op() != Some(OrderRel::LT) {
            panic!("Parsing error: Bound the wrong way arroung: must be 1729 >= X >= 20015"); }
        self.ws();
        match lb {
            None => None,
            Some(_) => {
                self.single_bound().map(|mut b| {
                    b.lower = lb;
                    b
                })
            },
        }
    }

    fn single_bound(&mut self) -> Option<PBound> {
        let backup = self.clone();
        let x = self.word();
        self.ws();
        let cmp = self.cmp_op();
        if cmp.is_none() { *self = backup; return None; }
        let cmp = cmp.unwrap();
        self.ws();
        if let Some(b) = self.signed_number() {
            Some(PBound {
                var: x,
                upper: if cmp != OrderRel::GT { Some(b) } else { None },
                lower: if cmp != OrderRel::LT { Some(b) } else { None },
            })
        } else {
            None
        }
    }

    fn is_sep(c: char) -> bool {
        match c {
            '+' | '-' | '*' | ' ' | '\t' | '\n' | '>' | '<' => true,
            _ => false,
        }
    }

    fn is_number_start(c: char) -> bool {
        match c {
            '0'...'9' | '.' | '-' => true,
            _ => false
        }
    }

    fn signed_number(&mut self) -> Option<f64> {
        if self.eat('-') {
            self.ws();
            return self.number().map(|x| -x)
        }
        if self.eat('+') {
            self.ws();
        }
        self.number()
    }

    fn number(&mut self) -> Option<f64> {
        let mut seen_dot = false;
        let mut res = String::new();
        loop {
            match self.peek(0) {
                Some((_, c)) if Parser::is_number_start(c) => {
                    if c == '.' {
                        if seen_dot { panic!("Error while parsing number {}", res); }
                        seen_dot = true;
                    }
                    res.push(c);
                    self.cur.next();
                },
                _ => break,
            }
        }
        FromStr::from_str(&res).ok()
    }

}

#[cfg(test)]
mod test {
    use super::*;
    use linear_system::*;

    #[test]
    fn test_word() {
        let mut p = Parser::new("toto tata");
        let res = p.word();
        assert_eq!(res, "toto");
        p.ws();
        let res2 = p.word();
        assert_eq!(res2, "tata");
    }

    #[test]
    fn test_begining() {
        let mut p = Parser::new("MINIMIZE t_1 + ...");
        assert_eq!(p.obj_kind(), Minimize);
    }

    #[test]
    fn test_number() {
        let mut p1 = Parser::new("44.44 gg");
        assert_eq!(p1.number(), Some(44.44));

        let mut p2 = Parser::new(".44 gg");
        assert_eq!(p2.number(), Some(0.44));
    }

    #[test]
    fn test_prod() {
        let mut p = Parser::new("33.3 * xx");
        assert_eq!(p.prod(), Some((33.3, "xx".to_string())));
    }

    #[test]
    fn test_sum() {
        let mut p = Parser::new("2.8 x + 4.4 y - 2.2 z <= toto");
        assert_eq!(p.sum(), vec![(2.8, "x".to_string()), (4.4, "y".to_string()), (-2.2, "z".to_string())]);
    }

    #[test]
    fn test_cmp_op() {
        let mut p = Parser::new("<=");
        assert_eq!(p.cmp_op(), Some(OrderRel::LT));
    }

    #[test]
    fn test_ineq() {
        let mut p = Parser::new("2.8 x + 4.4 y - 2.2 z <= 33.3");
        let expected = PInequation {
            prods: vec![(2.8, "x".to_string()), (4.4, "y".to_string()), (-2.2, "z".to_string())],
            kind: OrderRel::LT,
            cst: 33.3,
        };
        assert_eq!(p.inequation(), Some(expected));
    }

    #[test]
    fn test_bounds() {
        let mut p = Parser::new("33.3 <= x <= 99");
        let expected = PBound {
            var: "x".to_string(),
            upper: Some(99.0),
            lower: Some(33.3),
        };
        assert_eq!(p.bound(), Some(expected));
    }
}
