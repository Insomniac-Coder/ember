// Twin of y05e/y05f: reborrow what a `&mut i32` field points to, through `&mut self`. Rust accepts it.
pub struct Inner<'a> { pub r: &'a mut i32 }
pub struct Outer<'a> { pub inner: Inner<'a> }
impl<'a> Outer<'a> {
    pub fn get(&mut self) -> &mut i32 { &mut *self.inner.r }
}
pub fn main_like() -> i32 {
    let mut x = 1;
    let mut o = Outer { inner: Inner { r: &mut x } };
    let a = o.get();
    *a += 1;
    x
}
