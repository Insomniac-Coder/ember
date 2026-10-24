// Twin of x06c: a Weak handle kept longer than the view the object holds (the object is leaked).
use std::rc::{Rc, Weak};
pub struct Lexer<'a> { pub src: &'a str }
pub struct Parser<'a> { pub lexer: Lexer<'a> }
pub fn add_parser<'k>(seen: &mut Vec<Weak<Parser<'k>>>) {
    let text = String::from("let x = 1");
    let p = Rc::new(Parser { lexer: Lexer { src: &text } });
    seen.push(Rc::downgrade(&p));
    std::mem::forget(p.clone());
}
