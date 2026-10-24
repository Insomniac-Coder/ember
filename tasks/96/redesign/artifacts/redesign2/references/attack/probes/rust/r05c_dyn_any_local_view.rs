// Twin of x05c: a type-erased box holding views of a local.
use std::any::Any;
pub struct Pool<T> { pub dense: Vec<T> }
pub fn fill(keep: &mut Vec<Box<dyn Any>>) {
    let text = String::from("short lived");
    keep.push(Box::new(Pool { dense: vec![&text[0..5]] }));
}
