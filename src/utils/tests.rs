use float_cmp::{ApproxEq, F64Margin};

pub trait ApproxComparison {
    fn is_eq(&self, expected: Self, ulps: i64) -> bool;
}

impl ApproxComparison for f64 {
    fn is_eq(&self, expected: f64, ulps: i64) -> bool {
        let is_eq = self.approx_eq(expected, F64Margin { ulps, epsilon: 0.0 });
        if !is_eq {
            println!("approx_eq({}, {}, ulps = {}) didn't pass", self, expected, ulps);
        }
        is_eq
    }
}
