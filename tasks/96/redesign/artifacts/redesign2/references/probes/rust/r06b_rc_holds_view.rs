// Twin of p06b: a shared object holding a view.
use std::rc::Rc;
pub struct Lexer<'a> { pub src: &'a str, pub pos: usize }
pub struct Parser<'a> { pub lexer: Lexer<'a> }
pub fn make(text: &str) -> usize {
    let p = Rc::new(Parser { lexer: Lexer { src: text, pos: 0 } });
    let q = Rc::clone(&p);
    q.lexer.pos + p.lexer.src.len()
}
