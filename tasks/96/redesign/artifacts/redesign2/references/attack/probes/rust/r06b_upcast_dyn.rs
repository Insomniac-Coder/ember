// Twin of x06b: an upcast to a trait object stored in a long-lived registry.
use std::rc::Rc;
pub trait Named { fn describe(&self) -> String; }
pub struct Lexer<'a> { pub src: &'a str }
pub struct Parser<'a> { pub lexer: Lexer<'a> }
impl<'a> Named for Parser<'a> { fn describe(&self) -> String { self.lexer.src.to_string() } }
pub struct Registry { pub items: Vec<Rc<dyn Named>> }
pub fn add_parser(reg: &mut Registry) {
    let text = String::from("let x = 1");
    reg.items.push(Rc::new(Parser { lexer: Lexer { src: &text } }));
}
