# Attack on A6: a `Weak` handle kept where the strong ones may not go. The Parser is kept alive by a
# strong cycle (which Ember never collects, [WK-1]), so `upgrade` succeeds after `text` is freed.
struct Lexer:
    src: str

class Parser:
    lexer: Lexer
    me: Option[Parser] = None

class Registry:
    seen: Array[Weak[Parser]]

fn add_parser(reg: Registry):
    text: String = "let x = 1"
    p = Parser(lexer=Lexer(src=text))
    p.me = Some(p)
    reg.seen.push(Weak(p))

fn main():
    reg = Registry(seen=[])
    add_parser(reg)
    match reg.seen[0].upgrade():
        Some(p):
            println(p.lexer.src)
        None:
            println("gone")
