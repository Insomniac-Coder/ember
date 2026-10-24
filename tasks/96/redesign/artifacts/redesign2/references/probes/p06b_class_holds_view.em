# A6: a class object holding a view struct (Python: self.lexer = Lexer(text)).
struct Lexer:
    src: str
    pos: int = 0

class Parser:
    lexer: Lexer

fn main():
    text: String = "let x = 1"
    p = Parser(lexer=Lexer(src=text))
    println(p.lexer.pos)
