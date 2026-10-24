// A13: the hand-written workaround for a generic method behind `dyn` (E0038): take the visitor as
// `&mut dyn Visitor` instead of a type parameter. This is what the report proposes Ember generate.
trait Visitor { fn visit(&mut self, n: i64); }
struct Sum { total: i64 }
impl Visitor for Sum { fn visit(&mut self, n: i64) { self.total += n; } }
trait Inspect { fn inspect(&self, v: &mut dyn Visitor); }
struct Leaf { n: i64 }
impl Inspect for Leaf { fn inspect(&self, v: &mut dyn Visitor) { v.visit(self.n); } }
fn walk(xs: &[Box<dyn Inspect>]) -> i64 {
    let mut s = Sum { total: 0 };
    for x in xs { x.inspect(&mut s); }
    s.total
}
fn main() { println!("{}", walk(&[Box::new(Leaf { n: 1 }), Box::new(Leaf { n: 2 })])); }
