# Attack on A6's "an object holding a view carries the view's limit": two handles to one object.
# `reset` stores a shorter-lived view through one handle; the other handle, `q`, still believes the
# object views `text`, and reads `short` after it is freed. Rust refuses its twin (r06x_invariance.rs).
struct Lexer:
    src: str

class Parser:
    lexer: Lexer

fn reset(p: Parser, s: str):
    p.lexer = Lexer(src=s)

fn make_and_reset(p: Parser):
    short: String = "short lived"
    reset(p, short)

fn main():
    text: String = "let x = 1"
    p = Parser(lexer=Lexer(src=text))
    q = p
    make_and_reset(p)
    println(q.lexer.src)
