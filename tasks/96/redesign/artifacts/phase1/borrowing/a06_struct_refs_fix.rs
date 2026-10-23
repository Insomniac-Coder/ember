// A6 workaround: the lifetime parameter, repeated on every type and impl that holds one.
pub struct Lexer<'a> { pub src: &'a str, pub pos: usize }
pub struct Parser<'a> { pub lexer: Lexer<'a>, pub errors: Vec<String> }
impl<'a> Parser<'a> {
    pub fn new(src: &'a str) -> Parser<'a> {
        Parser { lexer: Lexer { src, pos: 0 }, errors: Vec::new() }
    }
    pub fn next_word(&mut self) -> &'a str {
        let rest = &self.lexer.src[self.lexer.pos..];
        let end = rest.find(' ').unwrap_or(rest.len());
        self.lexer.pos += end + 1;
        &rest[..end]
    }
}
// The EnTT `handle`: an entity bundled with a mutable reference to its registry.
pub struct Registry { pub hp: Vec<i32> }
pub struct Handle<'r> { pub reg: &'r mut Registry, pub entity: usize }
impl<'r> Handle<'r> {
    pub fn damage(&mut self, n: i32) { self.reg.hp[self.entity] -= n; }
}
