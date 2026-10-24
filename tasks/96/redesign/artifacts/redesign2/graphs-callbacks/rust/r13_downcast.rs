// A13: Python's isinstance over a list of shapes in Rust: `dyn Any` and downcast_ref per element.
use std::any::Any;
trait Shape: Any { fn area(&self) -> f64; fn as_any(&self) -> &dyn Any; }
struct Circle { r: f64 }
struct Square { side: f64 }
impl Shape for Circle { fn area(&self) -> f64 { 3.0 * self.r * self.r } fn as_any(&self) -> &dyn Any { self } }
impl Shape for Square { fn area(&self) -> f64 { self.side * self.side } fn as_any(&self) -> &dyn Any { self } }
fn total_radius(shapes: &[Box<dyn Shape>]) -> f64 {
    let mut t = 0.0;
    for s in shapes { if let Some(c) = s.as_any().downcast_ref::<Circle>() { t += c.r; } }
    t
}
fn main() { println!("{}", total_radius(&[Box::new(Circle { r: 1.0 }), Box::new(Square { side: 2.0 })])); }
