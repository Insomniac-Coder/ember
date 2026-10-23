# A6/A5: keep two words from the parser. In Rust the words borrow the text ('a); in Ember rule 1 ties them to the parser.
struct Lexer:
    src: str
    pos: int = 0

struct Parser:
    lexer: Lexer
    errors: Array[String]

    fn next_word(mut self) -> str:
        rest = self.lexer.src[self.lexer.pos..]
        end = rest.find(" ").unwrap_or(rest.len())
        self.lexer.pos += end + 1
        return rest[..end]

fn main():
    text: String = "let x = 1"
    p = Parser(lexer=Lexer(src=text), errors=[])
    a = p.next_word()
    b = p.next_word()
    println(a, b)
