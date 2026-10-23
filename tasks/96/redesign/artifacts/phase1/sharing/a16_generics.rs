// A16: print a list of my vectors; sum any numbers; a special case for one type.
use std::fmt;
pub struct Vec3 { pub x: f32, pub y: f32, pub z: f32 }

impl fmt::Display for Vec<Vec3> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result { write!(f, "{} vectors", self.len()) }
}

pub fn sum<T>(xs: &[T]) -> T {
    let mut acc = xs[0];
    for x in &xs[1..] { acc = acc + *x; }
    acc
}

pub trait Describe { fn describe(&self) -> String; }
impl<T: fmt::Debug> Describe for T { fn describe(&self) -> String { format!("{:?}", self) } }
impl Describe for i32 { fn describe(&self) -> String { format!("int {}", self) } }
