// Twin of x06a: two handles to one object; a shorter view stored through one of them.
use std::cell::RefCell;
use std::rc::Rc;
pub struct Lexer<'a> { pub src: &'a str }
pub struct Parser<'a> { pub lexer: Lexer<'a> }
fn reset<'a>(p: &Rc<RefCell<Parser<'a>>>, s: &'a str) { p.borrow_mut().lexer = Lexer { src: s }; }
fn make_and_reset(p: &Rc<RefCell<Parser<'_>>>) {
    let short = String::from("short lived");
    reset(p, &short);
}
pub fn main() {
    let text = String::from("let x = 1");
    let p = Rc::new(RefCell::new(Parser { lexer: Lexer { src: &text } }));
    let q = p.clone();
    make_and_reset(&p);
    println!("{}", q.borrow().lexer.src);
}
