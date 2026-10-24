// Twin of x05a: a result read through a `&mut` field, returned with the field's lifetime.
pub struct Cursor<'a> { buf: &'a mut Vec<i32> }
impl<'a> Cursor<'a> {
    pub fn cur(&mut self) -> &'a mut i32 { &mut self.buf[0] }
}
pub fn main() {
    let mut xs = vec![1, 2];
    let mut c = Cursor { buf: &mut xs };
    let a = c.cur();
    let b = c.cur();
    *a += 1;
    *b += 1;
}
