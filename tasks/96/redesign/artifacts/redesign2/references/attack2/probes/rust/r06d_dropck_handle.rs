// Twin of the drop-order attack on A6 4.4: a counted handle to an object that holds a view and has a
// `drop` that reads it, declared before the text it views. Rust refuses it (drop check).
use std::rc::Rc;
pub struct Parser<'a> { pub src: &'a str }
impl<'a> Drop for Parser<'a> { fn drop(&mut self) { println!("closing {}", self.src); } }
pub fn main_like() {
    let p: Option<Rc<Parser>>;
    let text = String::from("let x = 1");
    p = Some(Rc::new(Parser { src: &text }));
    println!("{}", p.as_ref().unwrap().src.len());
}
