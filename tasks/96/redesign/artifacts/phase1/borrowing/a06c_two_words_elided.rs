// A6/A5: the same without naming 'a on the method: elision ties the result to &mut self.
pub struct Parser<'a> { pub src: &'a str, pub pos: usize }
impl<'a> Parser<'a> {
    pub fn next_word(&mut self) -> &str {
        let rest = &self.src[self.pos..];
        let end = rest.find(' ').unwrap_or(rest.len());
        self.pos = (self.pos + end + 1).min(self.src.len());
        &rest[..end]
    }
}
pub fn two(text: &str) -> (usize, usize) {
    let mut p = Parser { src: text, pos: 0 };
    let a = p.next_word();
    let b = p.next_word();
    (a.len(), b.len())
}
