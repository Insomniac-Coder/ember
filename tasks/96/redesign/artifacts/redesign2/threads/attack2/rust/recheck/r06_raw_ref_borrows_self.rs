// Rust: a safe wrapper returns a reference made from a raw pointer; its lifetime is tied to &self.
struct Wrapper { raw: *const i64 }
impl Wrapper {
    fn get(&self) -> &i64 {
        // SAFETY: `raw` points at memory this Wrapper owns for as long as it lives.
        unsafe { &*self.raw }
    }
}
fn main() {
    let v = Box::new(41i64);
    let w = Wrapper { raw: &*v };
    println!("{}", *w.get() + 1);
}
